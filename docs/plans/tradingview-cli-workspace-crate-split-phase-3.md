# Workspace crate split phase 3: Desktop-free support crates

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md` from the repository root. It is self-contained so a future contributor can continue from this file without relying on chat history.

## Purpose / Big Picture

The workspace already has `tradingview-core` for shared contracts and `tradingview-market` for Desktop-free symbol reads. This phase finishes the next clean library boundary before `v0.3.0`: Desktop-free scanner reads and Pine static/check helpers move into internal crates, while CDP, UI automation, account mutation, and chart-state operations remain in the root CLI crate.

Users should see no command behavior change. The observable proof is that `tv scanner ...`, `tv pine analyze`, `tv pine alertconditions`, `tv pine check`, `tv info`, and `tv quote` still return the same JSON envelopes, while `cargo metadata` shows the new `tradingview-scanner` and `tradingview-pine` packages.

## Progress

- [x] (2026-04-28T06:35Z) Confirmed the working tree was clean and inspected the current workspace, scanner, Pine, market, and docs layout.
- [x] (2026-04-28T06:35Z) Created this ExecPlan and archived the completed market crate extraction plan.
- [x] (2026-04-28T06:38Z) Added `tradingview-scanner` and `tradingview-pine` workspace crates.
- [x] (2026-04-28T06:38Z) Moved scanner direct HTTP implementation into `tradingview-scanner`.
- [x] (2026-04-28T06:38Z) Moved Pine static analysis and Pine facade check implementation into `tradingview-pine`.
- [x] (2026-04-28T06:38Z) Split `crates/market/src/lib.rs` into facade plus smaller modules.
- [x] (2026-04-28T06:41Z) Updated architecture, development, roadmap, changelog, plan index, and continuity docs.
- [x] (2026-04-28T06:45Z) Ran validation and read-only smoke checks.
- [ ] Commit related changes.

## Surprises & Discoveries

- Observation: The mechanical code move compiled without any import redesign once root `src/ops/scanner.rs` and `src/ops/pine.rs` became thin facades.
  Evidence: `cargo check --workspace` completed successfully after the first code move.

- Observation: The new crates gave a clearer test split immediately: market has 10 unit tests, pine has 21, and scanner has 13, while root operation tests keep CDP/UI behavior.
  Evidence: `cargo test --workspace` reported all workspace tests passing with those crate-local test groups.

## Decision Log

- Decision: Extract only Desktop-free scanner and Pine support code in this phase.
  Rationale: Scanner hotlist/scan and Pine static/check paths do not need a TradingView Desktop CDP target. Pine editor operations, alert creation, Screener, watchlist, and chart OHLCV still depend on page state, account state, or CDP and should stay in the root crate until a future plan proves a safer boundary.
  Date/Author: 2026-04-28 / Codex.

- Decision: Keep all workspace support crates internal and JSON-Value based for now.
  Rationale: The CLI contract is the public surface today. A typed Rust API would be useful later, but adding it during a behavior-preserving extraction would expand scope and risk changing payload semantics.
  Date/Author: 2026-04-28 / Codex.

## Outcomes & Retrospective

The Desktop-free scanner implementation now lives in `crates/scanner/`, Pine static analysis and Pine facade check logic live in `crates/pine/`, and `crates/market/` is split into a small facade plus focused modules. The root CLI operation layer still exports the same names for command dispatch and alert integration, so user-facing CLI behavior is unchanged.

Validation passed: `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace`, focused `cli_contract` tests for scanner, Pine, and alert, `cargo metadata --no-deps --format-version 1`, `git diff --check`, and tracked-doc hygiene grep. Read-only smoke confirmed scanner hotlist, scanner scan, Pine analyze, Pine alertconditions, Pine check, `info NYSE:IONQ`, and `quote NYSE:IONQ`.

## Context and Orientation

The repository is a Cargo workspace. The root package is `tradingview-cli`, and the installed binary is `tv`. `crates/core/` contains `tradingview-core`, which owns shared errors and JSON envelope types. `crates/market/` contains `tradingview-market`, which owns Desktop-free symbol search, symbol metadata, and symbol quote reads.

The files `src/ops/scanner/{common,hotlist,scan}.rs` currently implement credential-free HTTP reads against TradingView scanner endpoints. They do not depend on CDP. The files `src/ops/pine/analysis.rs` and `src/ops/pine/check.rs` also do not depend on CDP: static analysis is local source parsing, and check uses TradingView's public Pine facade endpoint. `src/ops/pine/editor.rs` does depend on CDP and must stay in the root crate.

This phase creates two new internal packages: `tradingview-scanner`, imported as `tradingview_scanner`, and `tradingview-pine`, imported as `tradingview_pine`.

## Plan of Work

Add `crates/scanner/` and `crates/pine/` with small `lib.rs` facades. Move scanner HTTP code into `crates/scanner/src/`, and move Pine analysis/check code into `crates/pine/src/`. Update root `src/ops/scanner.rs` and `src/ops/pine.rs` so existing `src/main.rs` dispatch and other operation modules continue using the same `ops::...` names.

Split `crates/market/src/lib.rs` into focused modules without changing its public exports. The facade should continue exposing `symbol_search`, `symbol_info`, and `quote_symbol`.

Update the workspace manifest and docs to describe the new boundaries. Archive the completed market extraction plan under `docs/plans/archives/`. Do not move CDP, transport, UI mutation, Pine editor, Screener, alert/watchlist mutation, or chart OHLCV code in this phase.

## Concrete Steps

Work from the repository root.

1. Add `crates/scanner/Cargo.toml`, `crates/scanner/src/lib.rs`, and module files for scanner common, hotlist, and scan logic.
2. Add `crates/pine/Cargo.toml`, `crates/pine/src/lib.rs`, and module files for Pine analysis and check logic.
3. Update root `Cargo.toml` workspace members and dependencies.
4. Replace root scanner and Pine static/check modules with thin re-exports or wrappers.
5. Split `crates/market/src/lib.rs` into `search`, `info`, `quote`, and `normalize` modules.
6. Update docs and `CONTINUITY.md`.
7. Run validation:

       cargo fmt --check
       cargo clippy --workspace --all-targets --all-features -- -D warnings
       cargo test --workspace
       cargo test --test cli_contract scanner -- --nocapture
       cargo test --test cli_contract pine -- --nocapture
       cargo test --test cli_contract alert -- --nocapture
       cargo metadata --no-deps --format-version 1
       git diff --check

8. Run read-only smoke checks:

       target/debug/tv scanner hotlist volume_gainers --limit 3
       target/debug/tv scanner scan --limit 3
       target/debug/tv pine analyze --file <test pine file>
       target/debug/tv pine alertconditions --file <test pine file>
       target/debug/tv pine check --file <test pine file>
       target/debug/tv info NYSE:IONQ
       target/debug/tv quote NYSE:IONQ

9. Commit with:

       refactor(workspace): Extract Desktop-free support crates

## Validation and Acceptance

The change is accepted when all workspace tests pass, focused CLI contract tests for scanner, Pine, and alert pass, and read-only smoke confirms the existing commands still work. `cargo metadata --no-deps --format-version 1` must show packages `tradingview-core`, `tradingview-market`, `tradingview-scanner`, and `tradingview-pine`, plus the existing `tv` binary target.

The CLI JSON envelope and payload fields must remain compatible. Existing command names and flags must not change.

## Idempotence and Recovery

This plan is safe to repeat. If the new crate directories already exist, inspect and edit them in place. If imports fail, search for the old root module paths and replace them with the new crate imports or root facade exports. If any CLI contract changes unexpectedly, stop and restore the previous payload shape before continuing.

Do not move CDP, OHLCV, Screener, Pine editor, alert/watchlist mutation, or transport logic into the new crates in this phase.

## Artifacts and Notes

Do not paste machine-specific absolute paths, live target ids, cookies, tokens, account-local identifiers, or raw account payloads into repository docs. Terminal evidence should be short and scrubbed.

## Interfaces and Dependencies

`tradingview-scanner` must expose:

    pub async fn scanner_hotlist(slug: &str, limit: Option<usize>) -> Result<serde_json::Value, tradingview_core::AppError>
    pub async fn scanner_scan(request: ScannerScanRequest) -> Result<serde_json::Value, tradingview_core::AppError>
    pub struct ScannerScanRequest { ... }

`tradingview-pine` must expose:

    pub fn pine_analyze(source: &str, input_source: &str) -> serde_json::Value
    pub fn pine_alertconditions(source: &str, input_source: &str) -> serde_json::Value
    pub fn pine_alertcondition_candidates(source: &str) -> Vec<PineAlertconditionCandidate>
    pub struct PineAlertconditionCandidate { ... }
    pub async fn pine_check(source: &str, input_source: &str) -> Result<serde_json::Value, tradingview_core::AppError>

The root `tradingview_cli::ops` facade should continue exporting the same names used by `src/main.rs` and `src/ops/alert.rs`.

## Open Questions

No critical open questions block this phase. After this extraction is stable, the next planned step should be `v0.3.0` release readiness unless a small follow-up is needed to document or stabilize the new crate boundaries.
