# Add Desktop-free batch quotes

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository uses `.agents/PLANS.md` as the ExecPlan standard. This document follows that standard and is self-contained so a future contributor can resume the work without reading the chat history.

## Purpose / Big Picture

Users can already run `tv quote <SYMBOL>` for a Desktop-free scanner-backed single-symbol quote. After this change, users can run `tv quotes AAPL MSFT NYSE:IONQ` to fetch several symbol quotes without TradingView Desktop or CDP.

The new command keeps `tv quote [SYMBOL]` unchanged. Batch quote results are ordered by input and each result item contains the original requested symbol plus either the same quote payload used by single-symbol `tv quote <SYMBOL>` or a public-safe structured error.

## Progress

- [x] (2026-04-30 00:05Z) Confirmed the working tree was clean before implementation.
- [x] (2026-04-30 00:08Z) Confirmed `tradingview-market::quote_symbol` already returns the desired single-symbol Desktop-free payload and can be reused by batch quotes.
- [x] (2026-04-30 00:18Z) Added `tradingview-market` batch quote request/result normalization and tests.
- [x] (2026-04-30 00:20Z) Added `tv quotes <SYMBOL>...` CLI dispatch and contract tests.
- [x] (2026-04-30 00:25Z) Updated README, changelog, internal API reference, roadmap, plan index, and local continuity.
- [x] (2026-04-30 00:35Z) Ran focused tests, full workspace validation, read-only smoke, and hygiene checks.

## Surprises & Discoveries

- Observation: The existing CLI envelope cannot return `success: false` with a top-level `data` object.
  Evidence: command dispatch returns `Result<Value, AppError>`, and the output layer maps errors to `ErrorEnvelope`.
  Consequence: all-failure batches return non-zero with ordered diagnostics in `error.details.items[]`.

## Decision Log

- Decision: Add top-level `tv quotes <SYMBOL>...` instead of `quote --symbols`.
  Rationale: `tv quote` already means either current-chart quote or single-symbol quote. A separate plural command keeps batch quotes Desktop-free and unambiguous.
  Date/Author: 2026-04-30 / Codex
- Decision: Use ordered `items[]` as the source of truth rather than separate `quotes[]` and `errors[]` arrays.
  Rationale: Ordered items let downstream callers align each result with the input symbol without extra joins.
  Date/Author: 2026-04-30 / Codex
- Decision: Reuse the existing single-symbol `quote_symbol` payload for each successful item.
  Rationale: This preserves practical information compatibility and avoids inventing a second quote shape.
  Date/Author: 2026-04-30 / Codex
- Decision: For all-failure batches, return a top-level error whose `details` contains the same counts and ordered `items[]`.
  Rationale: The existing CLI envelope supports either success data or error details. This keeps non-zero exit behavior while preserving per-symbol diagnostics.
  Date/Author: 2026-04-30 / Codex

## Outcomes & Retrospective

Implemented. `tv quotes <SYMBOL>...` performs Desktop-free scanner-backed batch quote reads, keeps input order in `items[]`, and embeds the same quote payload shape that `tv quote <SYMBOL>` returns from the Desktop-free scanner path.

Read-only smoke confirmed that `tv quotes AAPL MSFT NYSE:IONQ` succeeds without Desktop/CDP and returns three ordered success items. A mixed request with a valid symbol plus unresolved symbols returned `success: true` with one quote item and structured validation error items. An all-failure request returned `success: false` with counts and ordered `items[]` under `error.details`, preserving per-symbol diagnostics while using the existing non-zero error envelope.

## Context and Orientation

`tradingview-market` owns Desktop-free symbol search, symbol metadata, and scanner-backed quote reads. `quote_symbol` validates one requested symbol, reads TradingView scanner REST, normalizes the result, and enriches validation failures with symbol-search candidates where possible.

The CLI dispatch currently treats `tv quote <SYMBOL>` as scanner-backed first, with chart fallback only for pre-mutation technical scanner failure. `tv quotes` should not use chart fallback. It is a Desktop-free batch read only.

## Plan of Work

Add `quote_symbols` to `tradingview-market`. It accepts a non-empty list of symbols, trims them, rejects blank values before network access, and then calls `quote_symbol` for each requested symbol in input order. The implementation may perform sequential requests in this first slice; one-request scanner batching is an optimization for a later plan.

For each symbol, build an item:

- success: `{ requested_symbol, ok: true, quote }`
- failure: `{ requested_symbol, ok: false, error: { kind, message, details } }`

If at least one item succeeds, return success data with `source`, `requested_count`, `resolved_count`, `error_count`, and `items`. If every item fails, return an `AppError` with the same counts and `items` in `details`. Use the first error kind for the top-level error kind.

Expose the function from `crates/market/src/lib.rs`, re-export it through the CLI market adapter, add `Command::Quotes { symbols: Vec<String> }`, and dispatch it without CDP connection.

Update docs to show the new command and clarify that batch quote freshness has the same scanner REST boundary as single-symbol quote.

## Concrete Steps

Run commands from the repository root.

Focused validation:

    cargo test -p tradingview-market quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture

Full validation:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Read-only smoke:

    target/debug/tv quotes AAPL MSFT NYSE:IONQ
    target/debug/tv quotes AAPL NASDAQ:IONQ BANANA
    target/debug/tv quotes

The first command should succeed without TradingView Desktop. The mixed command should preserve input order and show both successful quotes and structured errors. The no-symbol command should fail before network access.

## Validation and Acceptance

Acceptance is met when `tv quotes` runs without CDP, successful item quote payloads match single-symbol Desktop-free quote shape, per-symbol errors are returned in input order, all-failure batches still expose ordered diagnostics, existing `tv quote [SYMBOL]` behavior remains unchanged, and docs explain the scanner REST freshness boundary.

## Idempotence and Recovery

This is a read-only additive feature. Tests and smoke commands can be rerun safely. If TradingView scanner REST becomes unavailable during smoke, record the command-level failure without storing raw endpoint payloads.

## Artifacts and Notes

Do not paste raw scanner responses, cookies, tokens, account-local identifiers, or local absolute paths into tracked docs. It is safe to record endpoint category, command shape, and high-level payload shape.

## Interfaces and Dependencies

The new public CLI interface is `tv quotes <SYMBOL>...`. No existing command payload is changed. `items[].quote` uses the same shape as `tv quote <SYMBOL>` when the Desktop-free scanner path succeeds.

## Open Questions

None.
