# Quote source selection and market data timestamp metadata

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository uses `.agents/PLANS.md` as the ExecPlan standard. This document follows that standard and is self-contained so a future contributor can resume the work without reading the chat history.

## Purpose / Big Picture

`tv quote <SYMBOL>` can read quote data through two different TradingView paths: a Desktop-free scanner REST read and the selected TradingView Desktop chart API. These sources can legitimately differ by exchange feed, subscription entitlement, timestamp, or delayed-feed mode. After this change, users can choose the source explicitly with `--source scanner`, `--source chart`, or `--source auto`, and scanner-backed quote payloads expose `time`, `update_mode`, and `delay_seconds` when TradingView returns them.

The observable result is that `tv quote PLUG --source scanner` works without a CDP target and returns scanner feed metadata, while `tv quote PLUG --source chart` uses the selected chart target. `tv quote PLUG --source auto` is a chart-first compatibility mode that falls back to scanner only when the chart path is unavailable before any chart mutation.

## Progress

- [x] (2026-04-29T20:22:00Z) Confirmed scanner REST quote and Desktop chart quote can return different symbol source, price, volume, and feed metadata for the same requested bare symbol.
- [x] (2026-04-29T20:22:00Z) Archived the completed Desktop-free bars feasibility plan.
- [x] (2026-04-29T20:22:00Z) Added `--source scanner|chart|auto` to `tv quote`.
- [x] (2026-04-29T20:22:00Z) Added scanner quote `time`, `update_mode`, and parsed `delay_seconds` normalization.
- [x] (2026-04-29T20:22:00Z) Added focused unit and CLI contract coverage for scanner metadata and source selection validation.
- [x] (2026-04-30T00:00:00Z) Updated stable docs, roadmap, changelog, and plan index with the new source-selection contract.
- [x] (2026-04-30T00:00:00Z) Ran focused tests, full workspace validation, and live/read-only smoke for scanner, chart, and auto source behavior.

## Surprises & Discoveries

- Observation: Scanner REST can expose feed mode through `update_mode`.
  Evidence: A read-only scanner probe for a US equity returned a `time` column and an `update_mode` value shaped like `delayed_streaming_900`; the implementation records only the public-safe field behavior, not raw response payloads.

- Observation: The existing symbol quote command already had two meanings hidden behind one surface.
  Evidence: Before this change, `tv quote <SYMBOL>` tried the scanner path first and used chart switching only after a technical scanner failure. That made source differences hard to see in payloads and docs.

## Decision Log

- Decision: Keep the default for `tv quote <SYMBOL>` as `scanner`.
  Rationale: The Desktop-free path is the most ergonomic default for one-off symbol checks and matches the current post-v0.3 behavior.
  Date/Author: 2026-04-29 / Codex

- Decision: Make `--source auto` chart-first.
  Rationale: When the user asks for automatic source selection, the Desktop chart path is more likely to reflect the authenticated chart feed and realtime entitlement. The scanner path remains the fallback only if chart access fails before any chart mutation.
  Date/Author: 2026-04-29 / Codex

- Decision: Do not add source selection to `tv quotes <SYMBOL>...`.
  Rationale: Batch quotes are explicitly Desktop-free scanner reads. Adding chart fallback would require serial chart mutations and would blur the batch command's read-only contract.
  Date/Author: 2026-04-29 / Codex

- Decision: Reuse the existing top-level `time` field for scanner market-data timestamp.
  Rationale: Chart quotes already use `time` for the chart-side timestamp. Adding a second field such as `market_data_time` would duplicate the same value and make the payload noisier.
  Date/Author: 2026-04-29 / Codex

## Outcomes & Retrospective

The quote command now exposes source choice instead of silently mixing scanner and chart paths. Scanner-backed quote payloads retain the existing practical fields and add feed metadata. Chart-backed quote behavior remains compatible with the existing temporary switch, read, freshness check, and restore path.

The remaining open question is not code shape but data entitlement: scanner REST reads are convenient and Desktop-free, but they are not a realtime guarantee. Docs now state that `update_mode` and `delay_seconds` should be inspected when freshness matters.

Automated validation passed for the focused quote tests and the full workspace baseline. Live smoke confirmed scanner source works without CDP and returns feed metadata, chart source reads the selected chart feed, and auto source prefers chart when available but falls back to scanner when the CDP port is intentionally unavailable.

## Context and Orientation

The CLI surface is defined in `crates/cli/src/cli.rs`. Application dispatch lives in `crates/cli/src/app/dispatch.rs`. Desktop-free symbol quote normalization is implemented in `crates/market/src/quote.rs` and exposed through the `tradingview-market` crate. Chart-dependent current-chart quote and temporary chart switching live under `crates/cli/src/ops/market/quote.rs`.

The term scanner REST means a direct HTTPS request to TradingView's scanner endpoint. It does not require TradingView Desktop or CDP. The term chart API means JavaScript evaluated inside the selected TradingView Desktop chart target over CDP. It can reflect the logged-in chart context but requires a running Desktop target and may temporarily switch the chart symbol for symbol-targeted reads.

## Plan of Work

First, add a `QuoteSource` enum to `crates/cli/src/cli.rs` using clap value enums. Add `source: Option<QuoteSource>` to the `Quote` command so the CLI accepts `--source scanner`, `--source chart`, and `--source auto`. Update the help text to explain that symbol quotes default to scanner and symbol-less `tv quote` remains a current-chart read.

Second, update `crates/cli/src/app/dispatch.rs`. Symbol-less `tv quote` should keep using the current chart and reject `--source scanner` as validation. Symbol quotes with no source or `--source scanner` should call `ops::quote_symbol` and must not connect to CDP. Symbol quotes with `--source chart` should connect to CDP and call the chart quote path. Symbol quotes with `--source auto` should try the chart path first and fall back to scanner only when the chart runtime cannot be connected before any chart mutation.

Third, update `crates/market/src/quote.rs` so scanner quote requests include `time` and `update_mode`. Normalize those fields into the existing quote payload. Parse `delay_seconds` only for values that clearly follow `delayed_streaming_<seconds>`; otherwise return `null`.

Fourth, update docs and contract tests. The docs must no longer say that ordinary `tv quote <SYMBOL>` silently falls back from scanner to chart. They must describe explicit source selection, chart-first `auto`, scanner freshness metadata, and the unchanged Desktop-free `tv quotes` batch command.

## Concrete Steps

Run commands from the repository root.

Focused validation:

    cargo test -p tradingview-market quote -- --nocapture
    cargo test -p tradingview-cli market::quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture

Full validation:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Read-only or bounded smoke:

    target/debug/tv quote PLUG --source scanner
    target/debug/tv quote PLUG --source chart
    target/debug/tv quote PLUG --source auto
    TV_CDP_PORT=9 target/debug/tv quote PLUG --source scanner
    TV_CDP_PORT=9 target/debug/tv quote PLUG --source auto

The scanner source should work without CDP and include `time`, `update_mode`, and `delay_seconds` when TradingView returns them. The chart source should require CDP. The auto source should prefer chart when CDP is available and fall back to scanner when CDP is unavailable before mutation.

## Validation and Acceptance

Acceptance is met when `tv quote <SYMBOL>` still succeeds through scanner by default, `tv quote <SYMBOL> --source scanner` does not require CDP, `tv quote <SYMBOL> --source chart` uses the existing chart path, and `tv quote <SYMBOL> --source auto` is chart-first with a scanner fallback only for pre-mutation chart unavailability.

Scanner-backed payloads must keep the existing quote fields and now include public-safe feed metadata. A known delayed mode such as `delayed_streaming_900` must normalize to `"delay_seconds": 900`; unknown or missing modes must leave `delay_seconds` as `null`.

`tv quotes <SYMBOL>...` must remain Desktop-free and keep its existing ordered `items[]` payload shape.

## Idempotence and Recovery

The scanner source and docs updates are read-only. The chart and auto smoke commands can touch the selected chart if the requested symbol differs from the current chart; use a disposable chart or request the current chart's symbol when avoiding mutation matters.

If live chart smoke fails because TradingView Desktop is closed, keep the automated validation and record the skipped live evidence rather than changing the implementation.

## Artifacts and Notes

Focused tests passed during implementation:

    cargo test -p tradingview-market quote -- --nocapture
    cargo test -p tradingview-cli market::quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture

Full validation passed:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Tracked-doc hygiene grep reported only existing policy language, archived validation-command examples, and this plan's safety wording.

Live smoke summary:

    quote --source scanner: success, source scanner_scan_rest, returned time/update_mode/delay_seconds
    quote --source chart: success, source chart_api, update_mode and delay_seconds null
    quote --source auto: success, source chart_api when CDP was available
    TV_CDP_PORT=9 quote --source auto: success, source scanner_scan_rest

Do not add raw scanner response payloads, account-local values, cookies, tokens, chart target ids, or local absolute filesystem paths to tracked docs.

## Interfaces and Dependencies

`crates/cli/src/cli.rs` exposes:

    pub enum QuoteSource {
        Scanner,
        Chart,
        Auto,
    }

`crates/market/src/quote.rs` keeps the public `quote_symbol(symbol: &str) -> Result<Value, AppError>` function. Its returned JSON quote payload now includes `time`, `update_mode`, and `delay_seconds` in addition to the existing fields.

No new crate dependency is required.

## Open Questions

- UNCONFIRMED: Whether every TradingView scanner market uses `update_mode` values with the same naming convention.
- UNCONFIRMED: Whether scanner REST and chart API timestamps are always comparable across all asset classes and entitlement states.
