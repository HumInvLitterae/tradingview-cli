# Desktop-free market crate extraction

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md` from the repository root. It is self-contained so a future contributor can continue from this file without relying on chat history.

## Purpose / Big Picture

The CLI now has a small core contract crate, but Desktop-free market reads still live inside `src/ops/market.rs` next to CDP-dependent chart quote and OHLCV logic. After this change, the reusable direct HTTP reads for `tv search <QUERY>`, `tv info <SYMBOL>`, and `tv quote <SYMBOL>` will live in a new internal crate at `crates/market/`.

Users should see no command behavior change. The observable proof is that direct symbol reads still succeed without TradingView Desktop, while `cargo metadata` shows package `tradingview-market` and the existing `tv` binary target.

## Progress

- [x] (2026-04-28T06:12Z) Confirmed the working tree was clean and inspected `src/ops/market.rs`, `Cargo.toml`, and the existing workspace shape.
- [x] (2026-04-28T06:12Z) Created this ExecPlan and selected `crates/market/` as the next internal crate boundary.
- [x] (2026-04-28T06:13Z) Archived the completed core crate extraction plan.
- [x] (2026-04-28T06:15Z) Added the `tradingview-market` workspace crate and path dependency.
- [x] (2026-04-28T06:17Z) Moved Desktop-free search/info/quote implementation and tests into `tradingview_market`.
- [x] (2026-04-28T06:17Z) Updated root operation wrappers and kept chart-dependent quote/OHLCV in `tradingview_cli`.
- [x] (2026-04-28T06:18Z) Updated architecture, development, roadmap, changelog, plan index, and continuity docs.
- [x] (2026-04-28T06:21Z) Ran validation, read-only smoke checks, and hygiene checks.
- [ ] Commit the related changes.

## Surprises & Discoveries

- Observation: `src/ops/common.rs` still had `SYMBOL_SEARCH_URL` after the direct HTTP code moved.
  Evidence: `cargo check` reported the constant as unused. Removing it kept the root operation layer free of direct symbol-search endpoint details.

## Decision Log

- Decision: Extract only Desktop-free market reads in this slice.
  Rationale: `tv quote` current-chart reads, temporary chart switching, and OHLCV depend on `RuntimeEvaluator` and chart state. Moving them with direct HTTP reads would mix two different dependency models and make the boundary less reusable.
  Date/Author: 2026-04-28 / Codex.

- Decision: Keep the public payloads as `serde_json::Value` for this first market crate.
  Rationale: The root CLI already exposes JSON payloads and contract tests assert the current shape. Introducing typed market structs would be a separate API-design step and is not required for behavior-preserving extraction.
  Date/Author: 2026-04-28 / Codex.

## Outcomes & Retrospective

The Desktop-free market read implementation now lives in `crates/market/`, and `src/ops/market.rs` delegates `symbol_search`, `symbol_info_direct`, and `quote_symbol` to `tradingview_market`. Chart-dependent quote fallback and OHLCV remain in the root crate.

Validation passed: `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace`, focused `cli_contract` tests for quote and info, `cargo metadata --no-deps --format-version 1`, `cargo build`, `git diff --check`, and tracked-doc hygiene grep. Read-only smoke confirmed that `search`, `info NYSE:IONQ`, and `quote NYSE:IONQ` still succeed, including with `TV_CDP_PORT=9` for `info` and `quote`.

## Context and Orientation

The repository is a Cargo workspace. The root package is `tradingview-cli`, and the installed binary is `tv`. `crates/core/` contains `tradingview-core`, whose crate name is `tradingview_core`; it owns shared errors and JSON envelope types.

The file `src/ops/market.rs` currently contains two kinds of market logic. The Desktop-free logic uses ordinary HTTP requests with `reqwest` to TradingView symbol search and scanner endpoints. The chart-dependent logic uses the `RuntimeEvaluator` trait to read or mutate the active TradingView Desktop chart through Chrome DevTools Protocol. This plan separates only the Desktop-free logic.

The new package will be named `tradingview-market`, and Rust code will import it as `tradingview_market`.

## Plan of Work

First, move the completed `docs/plans/tradingview-cli-core-crate-extraction.md` plan into `docs/plans/archives/` and update `docs/plans/README.md` so the market extraction plan is the active crate-split plan.

Second, add `crates/market/Cargo.toml` and `crates/market/src/lib.rs`. Add `crates/market` to the workspace members and add `tradingview-market = { path = "crates/market" }` to the root package dependencies. The new crate should depend on `tradingview-core`, `reqwest`, and `serde_json`.

Third, move the direct HTTP functions from `src/ops/market.rs` into `crates/market/src/lib.rs`: `symbol_search`, `symbol_info`, `quote_symbol`, scanner quote request/normalization, symbol search match resolution, candidate generation, and symbol comparison helpers. Keep helper functions private unless the root crate needs them.

Fourth, update `src/ops/market.rs` so its `symbol_search`, `symbol_info_direct`, and `quote_symbol` functions delegate to `tradingview_market`. Leave `quote`, `ohlcv_bars`, `ohlcv_summary`, the quote symbol lock, and chart readiness helpers in the root crate.

Fifth, move the direct HTTP normalization tests into `tradingview-market` and leave chart-dependent tests in `src/ops/market.rs`. Update docs and `CONTINUITY.md`, then validate and commit.

## Concrete Steps

Work from the repository root.

1. Archive the completed core extraction plan:

       git mv docs/plans/tradingview-cli-core-crate-extraction.md docs/plans/archives/

2. Edit `Cargo.toml` to include `crates/market` in `[workspace].members` and add the root dependency on `tradingview-market`.

3. Add `crates/market/Cargo.toml` and `crates/market/src/lib.rs`.

4. Edit `src/ops/market.rs` so direct HTTP functions call `tradingview_market`, while CDP/chart logic remains in place.

5. Update docs and continuity.

6. Run validation:

       cargo fmt --check
       cargo clippy --workspace --all-targets --all-features -- -D warnings
       cargo test --workspace
       cargo test --test cli_contract quote -- --nocapture
       cargo test --test cli_contract info -- --nocapture
       cargo metadata --no-deps --format-version 1
       cargo build
       target/debug/tv search IONQ
       target/debug/tv info NYSE:IONQ
       target/debug/tv quote NYSE:IONQ
       TV_CDP_PORT=9 target/debug/tv info NYSE:IONQ
       TV_CDP_PORT=9 target/debug/tv quote NYSE:IONQ
       git diff --check
       git grep -nE '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' -- README.md CHANGELOG.md docs .agents/skills packaging scripts || true

7. Commit with:

       refactor(market): Extract Desktop-free market crate

## Validation and Acceptance

The change is accepted when workspace tests pass, `cargo metadata --no-deps --format-version 1` shows package `tradingview-market` with target `tradingview_market`, and the read-only smoke commands prove that search/info/quote direct reads still work without CDP.

The CLI JSON payloads must remain compatible. In particular, `tv info NYSE:IONQ` must still report `source: "symbol_search_rest"`, `tv quote NYSE:IONQ` must still report `source: "scanner_scan_rest"` and `non_mutating: true`, and symbol-resolution validation failures must not fall back to target selection.

## Idempotence and Recovery

This plan is safe to repeat. If `crates/market/` already exists, inspect it and edit in place. If imports fail, search for remaining direct helper calls in `src/ops/market.rs` and update them to either the new crate or the local chart-dependent helper. If a CLI contract changes, stop and restore the previous JSON shape before continuing.

Do not move CDP or OHLCV logic into `crates/market/` in this slice.

## Artifacts and Notes

Expected `cargo metadata --no-deps --format-version 1` should include package `tradingview-market` and target `tradingview_market`. Do not paste machine-specific metadata paths into repository docs.

Expected `TV_CDP_PORT=9 target/debug/tv info NYSE:IONQ` and `TV_CDP_PORT=9 target/debug/tv quote NYSE:IONQ` should succeed, proving those reads do not require a TradingView Desktop CDP connection.

## Interfaces and Dependencies

The new crate must expose:

    pub async fn symbol_search(query: &str) -> Result<serde_json::Value, tradingview_core::AppError>
    pub async fn symbol_info(symbol: &str) -> Result<serde_json::Value, tradingview_core::AppError>
    pub async fn quote_symbol(symbol: &str) -> Result<serde_json::Value, tradingview_core::AppError>

The root crate keeps its existing operation functions:

    pub async fn symbol_search(query: &str) -> Result<Value, AppError>
    pub async fn symbol_info_direct(symbol: &str) -> Result<Value, AppError>
    pub async fn quote_symbol(symbol: &str) -> Result<Value, AppError>

These root functions should delegate to `tradingview_market`.

## Open Questions

No critical open questions block this slice. After this extraction is stable, a later plan can evaluate whether `crates/cdp/` or a typed market response API is worth adding.
