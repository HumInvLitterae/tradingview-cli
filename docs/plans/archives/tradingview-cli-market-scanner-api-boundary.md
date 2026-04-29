# Market and Scanner typed read API boundary

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository uses `.agents/PLANS.md` as the ExecPlan standard. This document follows that standard and is self-contained so a future contributor can resume the work without reading the chat history.

## Purpose / Big Picture

`tradingview-market` and `tradingview-scanner` now contain the most reusable Desktop-free read paths in the workspace. Before this change, their public functions mostly returned `serde_json::Value`, which was convenient for the CLI but awkward for other Rust callers. This slice adds typed result structs and typed read functions while preserving the existing JSON-returning functions that the CLI already uses.

The observable result is that the CLI output does not change, but Rust callers can use typed functions such as `quote_symbol_typed`, `quote_symbols_typed`, `scanner_scan_typed`, and `scanner_metainfo_typed`.

## Progress

- [x] (2026-04-30T00:00:00Z) Confirmed the working tree was clean before implementation.
- [x] (2026-04-30T00:00:00Z) Archived the completed quote source selection plan.
- [x] (2026-04-30T00:00:00Z) Added typed public structs and typed functions to `tradingview-market`.
- [x] (2026-04-30T00:00:00Z) Added typed public structs and typed functions to `tradingview-scanner`.
- [x] (2026-04-30T00:00:00Z) Kept existing JSON-returning public functions as compatibility wrappers around the typed API.
- [x] (2026-04-30T00:00:00Z) Added focused typed API unit coverage for market quote/search/info and scanner hotlist/scan/metainfo.
- [x] (2026-04-30T00:00:00Z) Completed CLI contract tests, full workspace validation, metadata generation, whitespace check, hygiene grep, and read-only smoke.

## Surprises & Discoveries

- Observation: Some quote and scanner values should stay as `serde_json::Value` inside typed structs for now.
  Evidence: Scanner fields can be number, string, or null depending on field and market state. Using `Option<f64>` everywhere would risk changing integer versus float JSON representation in CLI wrappers.

- Observation: The CLI does not need to switch to typed functions immediately.
  Evidence: The existing `ops` layer can keep calling JSON-returning wrappers; those wrappers now serialize typed results and keep the same payload shape.

## Decision Log

- Decision: Add typed APIs without removing JSON APIs.
  Rationale: The CLI and downstream command contracts already depend on the JSON payloads. Typed APIs improve crate reuse without creating a migration burden.
  Date/Author: 2026-04-30 / Codex

- Decision: Keep market-data values as `serde_json::Value` in typed structs when TradingView can return mixed or nullable values.
  Rationale: This preserves existing JSON compatibility and avoids pretending every scanner field has a stable numeric type.
  Date/Author: 2026-04-30 / Codex

- Decision: Do not add new CLI commands or options in this slice.
  Rationale: This is an internal API boundary cleanup. User-visible market read behavior was already handled in earlier v0.4 slices.
  Date/Author: 2026-04-30 / Codex

## Outcomes & Retrospective

Implemented. The market and scanner crates now expose typed API surfaces while retaining existing JSON wrappers for CLI compatibility. The CLI payload shape remains unchanged; contract tests and read-only smoke confirmed quote, batch quote, scanner scan, and scanner metainfo still return successful envelopes.

## Context and Orientation

`tradingview-market` owns Desktop-free symbol search, symbol info, single quote, and batch quote reads. `tradingview-scanner` owns Desktop-free scanner hotlist, scanner scan, and scanner metainfo reads. These crates do not use CDP, TradingView Desktop, UI automation, chart fallback, or account mutation.

The CLI package under `crates/cli` can keep using JSON wrappers because its public contract is JSON. The new typed APIs exist for Rust reuse and for clearer internal boundaries.

## Plan of Work

First, add `serde` as a dependency to `tradingview-market` and `tradingview-scanner` so typed structs can derive `Serialize`.

Second, add typed structs to `crates/market/src/types.rs` and `crates/scanner/src/types.rs`, then re-export them from each crate's `lib.rs`.

Third, add typed functions beside existing functions. Existing functions such as `quote_symbol` and `scanner_scan` should call the typed implementation and serialize it back to `serde_json::Value`. This keeps CLI payload compatibility.

Fourth, update docs to describe the boundary: typed API is the preferred internal Rust surface, JSON wrappers remain for CLI compatibility, and these crates remain Desktop-free read crates rather than chart/runtime adapters.

## Concrete Steps

Run commands from the repository root.

Focused validation:

    cargo test -p tradingview-market -- --nocapture
    cargo test -p tradingview-scanner -- --nocapture
    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract scanner -- --nocapture

Full validation:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Read-only smoke:

    target/debug/tv quote PLUG --source scanner
    target/debug/tv quotes AAPL MSFT NYSE:IONQ
    target/debug/tv scanner scan --limit 3 --columns name,close,premarket_close,postmarket_close
    target/debug/tv scanner metainfo --market america --field close --field premarket_close

## Validation and Acceptance

Acceptance is met when market/scanner typed unit tests pass, CLI contract tests prove quote and scanner JSON output remains compatible, and the read-only smoke commands still return successful CLI JSON envelopes.

The new typed structs must be public from their crates. The existing `serde_json::Value` functions must remain public and must not change command payload shape.

## Idempotence and Recovery

This is a behavior-preserving refactor. It can be retried safely. If a typed conversion changes CLI JSON shape, keep the typed function but hand-build the JSON wrapper to match the previous payload exactly.

If full validation fails outside the touched market/scanner paths, diagnose before broadening the scope. Do not change unrelated adapters in this slice.

## Artifacts and Notes

Focused tests already passed during implementation:

    cargo test -p tradingview-market -- --nocapture
    cargo test -p tradingview-scanner -- --nocapture
    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract scanner -- --nocapture

Full validation passed:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Read-only smoke passed with scrubbed output summaries:

    target/debug/tv quote PLUG --source scanner
    target/debug/tv quotes AAPL MSFT NYSE:IONQ
    target/debug/tv scanner scan --limit 3 --columns name,close,premarket_close,postmarket_close
    target/debug/tv scanner metainfo --market america --field close --field premarket_close

The tracked-doc hygiene grep reported existing policy language, archived validation-command examples, and this plan's safety wording. No new machine-specific path, account-local identifier, cookie, token, authorization value, or raw live payload was added.

Do not add raw scanner responses, account-local values, cookies, tokens, chart target ids, or local absolute filesystem paths to tracked docs.

## Interfaces and Dependencies

`tradingview-market` exposes typed functions:

    search_symbols_typed(query)
    symbol_info_typed(symbol)
    quote_symbol_typed(symbol)
    quote_symbols_typed(symbols)

`tradingview-scanner` exposes typed functions:

    scanner_hotlist_typed(slug, limit)
    scanner_scan_typed(request)
    scanner_metainfo_typed(request)

Both crates keep their existing JSON-returning functions for CLI compatibility.

## Open Questions

- UNCONFIRMED: Whether later `v0.4` work should add crate-level examples or README files for typed API consumers.
- UNCONFIRMED: Whether typed market values should become stronger numeric/string enums in a later breaking internal API review.
