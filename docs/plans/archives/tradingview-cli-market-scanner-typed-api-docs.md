# Market and Scanner typed API docs and examples

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository uses `.agents/PLANS.md` as the ExecPlan standard. This document follows that standard and is self-contained so a future contributor can resume the work without reading the chat history.

## Purpose / Big Picture

`tradingview-market` and `tradingview-scanner` now expose typed read APIs, but a Rust caller still has to discover those APIs by reading source exports. After this change, generated rustdoc and a stable `docs/rust-api.md` guide explain that the typed functions are the preferred Rust surface, while the `serde_json::Value` functions remain compatibility wrappers for the `tv` CLI JSON contract.

The observable result is documentation-only: `cargo doc --workspace --no-deps` should show crate-level descriptions and compile-checked examples for Desktop-free market and scanner reads. The CLI output must not change.

## Progress

- [x] (2026-04-30T00:00:00Z) Confirmed the working tree was clean before implementation.
- [x] (2026-04-30T00:00:00Z) Archived the completed market/scanner typed API boundary plan.
- [x] (2026-04-30T00:00:00Z) Added crate-level rustdoc and compile-checked no-run examples for `tradingview-market`.
- [x] (2026-04-30T00:00:00Z) Added crate-level rustdoc and compile-checked no-run examples for `tradingview-scanner`.
- [x] (2026-04-30T00:00:00Z) Added stable Rust API guide and updated docs indexes.
- [x] (2026-04-30T00:00:00Z) Ran rustdoc validation, full baseline, metadata generation, whitespace check, read-only smoke, and hygiene grep.

## Surprises & Discoveries

- Observation: `no_run` examples still compile as doctests, so they are useful for API drift detection without making tests depend on network access.
  Evidence: `cargo test --doc -p tradingview-market -- --nocapture` and `cargo test --doc -p tradingview-scanner -- --nocapture` each compiled two doctests successfully.

## Decision Log

- Decision: Add examples as `no_run` doctests instead of runnable doctests.
  Rationale: The examples call live TradingView scanner endpoints. They should compile, but test runs must not depend on network availability.
  Date/Author: 2026-04-30 / Codex

- Decision: Keep this slice documentation-only.
  Rationale: The previous slice already added typed APIs. This slice should improve discoverability without changing CLI behavior or typed struct shape.
  Date/Author: 2026-04-30 / Codex

## Outcomes & Retrospective

Implemented. `tradingview-market` and `tradingview-scanner` now have crate-level rustdoc examples, typed result/request documentation, and a stable `docs/rust-api.md` guide. Validation and read-only smoke passed, and no CLI behavior or payload shape was changed.

## Context and Orientation

The workspace contains several internal crates. `tradingview-market` owns Desktop-free symbol search, symbol metadata, single-symbol quote, and batch quote reads. `tradingview-scanner` owns Desktop-free scanner hotlist, scanner scan, and scanner metainfo reads. Desktop-free means these functions use credential-free HTTP endpoints and do not connect to TradingView Desktop through CDP.

Both crates expose two kinds of functions. The typed functions, whose names end in `_typed`, return Rust structs such as `Quote`, `BatchQuotes`, `ScannerScanResult`, and `ScannerMetainfoResult`. The older functions return `serde_json::Value` and are kept so the CLI can preserve its existing JSON payload contract.

## Plan of Work

First, add crate-level documentation to `crates/market/src/lib.rs` and `crates/scanner/src/lib.rs`. Each crate doc should explain the read-only Desktop-free boundary, name the preferred typed functions, and include `no_run` examples that compile without executing network calls during doctests.

Second, add concise doc comments to the typed result structs and request structs that a Rust caller is expected to touch. The comments should explain what the type represents and avoid promising stable crates.io API compatibility.

Third, add `docs/rust-api.md` as the stable guide for internal Rust API reuse. It should explain the current crate boundaries, recommend typed market/scanner APIs for new Rust callers, and describe JSON wrappers as CLI compatibility helpers.

Fourth, update `docs/architecture.md`, `docs/development.md`, `docs/v0.4-roadmap.md`, `docs/plans/README.md`, and `CHANGELOG.md` so future contributors can find the Rust API guide.

## Concrete Steps

Run commands from the repository root.

Rustdoc validation:

    cargo test --doc -p tradingview-market -- --nocapture
    cargo test --doc -p tradingview-scanner -- --nocapture
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

Full baseline:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Read-only compatibility smoke:

    target/debug/tv quote PLUG --source scanner
    target/debug/tv quotes AAPL MSFT NYSE:IONQ
    target/debug/tv scanner scan --limit 3 --columns name,close,premarket_close,postmarket_close
    target/debug/tv scanner metainfo --market america --field close --field premarket_close

Completed validation:

    cargo test --doc -p tradingview-market -- --nocapture
    cargo test --doc -p tradingview-scanner -- --nocapture
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Read-only smoke returned successful JSON envelopes for scanner quote, ordered batch quotes, scanner scan with extended-hours columns, and scanner metainfo. The tracked-doc hygiene grep reported only existing policy language, archived validation-command examples, and this plan's safety wording.

## Validation and Acceptance

Acceptance is met when rustdoc builds without warnings, doctests compile, the full Rust baseline passes, and smoke commands still return successful JSON envelopes. Because this slice is documentation-only, any CLI payload or public option change is a failure.

## Idempotence and Recovery

This work is safe to retry. If a doctest tries to execute network calls, convert it to a `no_run` example. If `cargo doc` fails because a public item is under-documented by wording rather than compiler rules, keep comments short and factual instead of broadening the API surface.

## Artifacts and Notes

Do not write raw scanner responses, account-local values, cookies, tokens, chart target ids, or local absolute filesystem paths into tracked docs. It is safe to name public function names, public example symbols, and high-level endpoint categories.

## Interfaces and Dependencies

The documented typed functions are:

    tradingview_market::search_symbols_typed
    tradingview_market::symbol_info_typed
    tradingview_market::quote_symbol_typed
    tradingview_market::quote_symbols_typed
    tradingview_scanner::scanner_hotlist_typed
    tradingview_scanner::scanner_scan_typed
    tradingview_scanner::scanner_metainfo_typed

The compatibility JSON wrappers remain public and unchanged.

## Open Questions

- UNCONFIRMED: Whether later work should add standalone README files inside each crate directory. This slice uses crate-level rustdoc and `docs/rust-api.md` first.
