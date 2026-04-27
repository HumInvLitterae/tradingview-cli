# Desktop-free symbol HTTP reads

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md` from the repository root. It is self-contained so a future contributor can continue from this file without relying on chat history.

## Purpose / Big Picture

Agents often need basic symbol information before they need the live chart. The Rust `tv` CLI already has direct HTTP reads for `search`, `scanner hotlist`, `scanner scan`, symbol-targeted `quote`, and `pine check`, but the current `info` command still means “read the selected chart”. A downstream incident also showed that `tv quote NASDAQ:IONQ` can fall through to Desktop target selection when the HTTP scanner quote finds no rows because the symbol is actually listed as `NYSE:IONQ`. After this change, a user can run `tv info IONQ`, `tv info NYSE:IONQ`, `tv quote IONQ`, or `tv quote NYSE:IONQ` without a running TradingView Desktop session, and incorrect exchange-qualified symbols should fail with useful candidates rather than `target_ambiguous`.

## Progress

- [x] (2026-04-27 20:20Z) Created this ExecPlan and confirmed the current direct HTTP boundary from code and docs.
- [x] (2026-04-28) Implemented `tv info [SYMBOL]` with Desktop-free direct metadata reads when `SYMBOL` is present.
- [x] (2026-04-28) Tightened `tv quote <SYMBOL>` fallback so scanner symbol-resolution validation does not fall back to chart/CDP.
- [x] (2026-04-28) Updated README, changelog, internal API reference, runtime skills, packaged guide, plan index, and continuity.
- [x] (2026-04-28) Ran targeted tests, full validation, read-only smoke, skill validation, packaging shell validation, and hygiene checks.
- [ ] Commit the related tracked changes.

## Surprises & Discoveries

- Observation: `target/debug/tv quote NASDAQ:IONQ` currently falls through to `target_ambiguous`, while `target/debug/tv quote IONQ` and `target/debug/tv quote NYSE:IONQ` succeed through `scanner_scan_rest`.
  Evidence: A local read-only check showed `NYSE:IONQ` as the first `search NASDAQ:IONQ` candidate and quote success for `IONQ` / `NYSE:IONQ`.
- Observation: `tv info NYSE:IONQ` succeeded through symbol-search metadata. The first smoke of `tv info IONQ` returned an ambiguity validation because multiple markets expose exact `IONQ` symbols.
  Evidence: The symbol-search result order puts `NYSE:IONQ` first, followed by other exact `IONQ` listings.

## Decision Log

- Decision: Extend `tv info` with an optional positional symbol rather than adding a new top-level command.
  Rationale: `info` already means symbol metadata. Keeping `tv info` as current-chart read and adding `tv info <SYMBOL>` as Desktop-free metadata is discoverable and mirrors `tv quote [SYMBOL]`.
  Date/Author: 2026-04-27 / Codex.

- Decision: Do not add `tv ohlcv <SYMBOL>` in this slice.
  Rationale: OHLCV currently means current chart bars. A Desktop-free historical bars endpoint needs separate evidence and a separate contract decision.
  Date/Author: 2026-04-27 / Codex.

- Decision: Treat scanner/symbol-search no rows, mismatches, and ambiguity as symbol-resolution failures that must not chart fallback.
  Rationale: Falling back to chart/CDP turns an input issue into target ambiguity and makes single-symbol reads depend on Desktop state.
  Date/Author: 2026-04-27 / Codex.

- Decision: For `tv info <SYMBOL>`, resolve exchange-qualified input strictly, but let bare symbol input use the first exact match in TradingView's search ordering.
  Rationale: The practical one-off metadata workflow should make `tv info IONQ` work without Desktop when TradingView's own search ranks the intended listing first, while `tv info NASDAQ:IONQ` should still fail with candidates when the exchange qualifier is wrong.
  Date/Author: 2026-04-28 / Codex.

## Outcomes & Retrospective

Implemented. `tv info [SYMBOL]` now has two modes: with no symbol it keeps the existing current-chart CDP metadata behavior, and with a symbol it uses symbol-search HTTP without connecting to TradingView Desktop. Bare symbol input such as `IONQ` uses TradingView's first exact search result, while exchange-qualified input such as `NYSE:IONQ` is strict and returns a validation error with candidates when the exchange is wrong.

`tv quote <SYMBOL>` still uses scanner REST for ordinary symbol quotes. Scanner technical failures may still use the existing chart fallback, but scanner no-row, ambiguous-row, or returned-symbol mismatch validation now stops before CDP fallback and enriches the error with symbol-search candidates when available.

Read-only smoke passed for `search IONQ`, `info IONQ`, `info NYSE:IONQ`, `quote IONQ`, `quote NYSE:IONQ`, and `quote NASDAQ:IONQ`. The incorrect exchange-qualified quote returned a validation error with candidates instead of `target_ambiguous`. `TV_CDP_PORT=9 info NYSE:IONQ` and `TV_CDP_PORT=9 quote NYSE:IONQ` both succeeded, confirming these symbol-targeted reads do not need a live Desktop target.

Desktop-free OHLCV remains out of scope. No safe historical bars endpoint was selected in this implementation slice; `tv ohlcv` continues to mean current chart bars.

Validation passed: `cargo test market -- --nocapture`, `cargo test --test cli_contract info -- --nocapture`, `cargo test --test cli_contract quote -- --nocapture`, `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `git diff --check`, changed-skill validation, `bash -n scripts/stage-release-package-files.sh`, and tracked-doc hygiene grep. The hygiene grep returned existing policy text and validation-command examples only.

## Context and Orientation

The Rust CLI command definitions live in `src/cli.rs`. Dispatch lives in `src/main.rs`. Market reads live in `src/ops/market.rs`, which already uses TradingView's symbol-search HTTP endpoint for `tv search` and scanner REST for `tv quote <SYMBOL>`. Current-chart metadata is implemented separately as `symbol_info` in `src/ops/chart.rs` and requires a CDP runtime. CDP is the Chrome DevTools Protocol connection to the running TradingView Desktop page.

Direct HTTP in this plan means ordinary `reqwest` calls made by the CLI process without connecting to the TradingView Desktop CDP target and without extracting cookies or session tokens. It is acceptable only for credential-free read-only endpoints.

## Plan of Work

First, add a direct symbol metadata operation in `src/ops/market.rs`. It should call the existing symbol-search endpoint, resolve exchange-qualified input strictly, and resolve bare symbol input to the first exact symbol match returned by TradingView search. It should return a metadata object under the existing success envelope. The object should include `symbol`, `full_name`, `exchange`, `description`, `type`, `pro_name`, `source: "symbol_search_rest"`, `non_mutating: true`, and `requested_symbol`. If the qualified request is mismatched or no match is available, return `ErrorKind::Validation` with a small `candidates` array built from normalized search results.

Second, update `src/cli.rs` and `src/main.rs` so `Command::Info` accepts `Option<String>`. When a symbol is provided, trim and validate it, then call the direct symbol metadata operation without connecting to CDP. When no symbol is provided, keep the existing current-chart metadata behavior unchanged.

Third, update `tv quote <SYMBOL>` fallback behavior. Keep chart fallback only when the scanner quote endpoint itself is technically unavailable. If scanner quote returns a validation error for no rows, ambiguous rows, or returned symbol mismatch, return that validation error directly. Enrich no-row and mismatch errors with symbol-search candidates where possible.

Fourth, update docs and runtime skills so agents prefer `tv quote <SYMBOL>` and `tv info <SYMBOL>` for one-off symbol checks, and mutate the chart only when OHLCV bars, visible studies, drawings, screenshots, or current-chart metadata are actually needed.

Finally, run validation and commit the related changes.

## Concrete Steps

Work from the repository root.

1. Inspect the relevant implementation:

       sed -n '1,260p' src/ops/market.rs
       sed -n '130,175p' src/main.rs
       sed -n '30,70p' src/cli.rs

2. Edit `src/ops/market.rs`, `src/ops.rs`, `src/cli.rs`, `src/main.rs`, and tests.

3. Update `README.md`, `CHANGELOG.md`, `docs/internal-tradingview-apis.md`, `docs/plans/README.md`, `.agents/skills/chart-analysis/SKILL.md`, `.agents/skills/multi-symbol-scan/SKILL.md`, `packaging/agent/AGENTS.md`, and `CONTINUITY.md`.

4. Run validation:

       cargo test market -- --nocapture
       cargo test --test cli_contract info -- --nocapture
       cargo test --test cli_contract quote -- --nocapture
       cargo fmt --check
       cargo clippy --all-targets --all-features -- -D warnings
       cargo test
       git diff --check
       git grep -nE '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' -- README.md CHANGELOG.md docs .agents/skills packaging scripts || true

5. Run read-only smoke:

       target/debug/tv search IONQ
       target/debug/tv info IONQ
       target/debug/tv info NYSE:IONQ
       target/debug/tv quote IONQ
       target/debug/tv quote NYSE:IONQ
       target/debug/tv quote NASDAQ:IONQ
       TV_CDP_PORT=9 target/debug/tv info NYSE:IONQ
       TV_CDP_PORT=9 target/debug/tv quote NYSE:IONQ

6. Validate changed skills using the repo-local validator if available, then commit with:

       feat(market): Add direct symbol info reads

## Validation and Acceptance

The change is accepted when `tv info <SYMBOL>` succeeds without CDP for valid symbols, `tv info` without a symbol still attempts current-chart CDP access, `tv quote IONQ` and `tv quote NYSE:IONQ` remain non-mutating scanner reads, and `tv quote NASDAQ:IONQ` returns a symbol-resolution error with candidates instead of target ambiguity. Full Rust validation and docs hygiene must pass.

## Idempotence and Recovery

All smoke commands are read-only. Do not run `tv symbol`, `tv timeframe`, or any chart mutation for this slice. If the live TradingView HTTP endpoints are unavailable during smoke, keep automated tests as authority and record the blocker in this plan. If candidate details accidentally include account-local or machine-local data, remove them before committing.

## Artifacts and Notes

Expected `tv info IONQ` success should look like a success envelope with `source: "symbol_search_rest"`, `non_mutating: true`, and a `full_name` such as `NYSE:IONQ`. Expected `tv quote NASDAQ:IONQ` should be a failure envelope whose error is about symbol resolution and includes candidate symbols such as `NYSE:IONQ`, not a CDP `target_ambiguous` error.

## Interfaces and Dependencies

No new Rust dependency is required. Use existing `reqwest`, `serde_json`, and `AppError` types. Public CLI after this slice:

    tv info [SYMBOL]
    tv quote [SYMBOL]

The new operation should be exported from `src/ops.rs` as a market operation. Keep the existing JSON envelope behavior in `src/main.rs` unchanged.

## Open Questions

No critical open questions block implementation. Desktop-free OHLCV remains outside this slice until a credential-safe historical bars endpoint is proven.
