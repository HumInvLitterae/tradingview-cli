# Desktop quote-data source implementation

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root.

## Purpose / Big Picture

TradingView Desktop can display an after-hours price in the right-side detail
panel that is not the chart main-series last bar and is not scanner REST. The
strongest current candidate is TradingView WebSocket quote-data field
`qsd.rtc`. This slice adds an explicit source for that readback:
`tv quote <SYMBOL> --source quote-data`.

The new source is deliberately separate from scanner `extended_hours` and from
`quote --source chart`. It observes bounded CDP WebSocket events, extracts
public-safe quote-data fields when a matching `qsd.rtc` arrives, and fails
with structured unavailable details when no matching readback appears.

## Progress

- [x] (2026-05-10T00:05Z) Created this ExecPlan and archived the completed
  quote-data RTC source-design plan.
- [x] (2026-05-10T00:10Z) Added `quote-data` to the quote source enum and
  dispatch path without changing `auto`, `scanner`, or `chart`.
- [x] (2026-05-10T00:15Z) Added a Desktop-backed bounded quote-data observer
  that parses public-safe `qsd.rtc` readback from CDP WebSocket events.
- [x] (2026-05-10T00:30Z) Updated contract tests, docs, runtime skills, and
  changelog for the explicit quote-data source boundary.
- [x] (2026-05-10T00:45Z) Ran focused and workspace validation. Commit is
  being prepared.

## Surprises & Discoveries

- Observation: The production CDP client only exposed request/response method
  calls before this slice. Quote-data needs event observation.
  Evidence: `CdpClient::call_method` waits for matching response ids and
  ignores events. The implementation added a small event-reading method that
  returns public CDP event objects without exposing raw frame payloads.

## Decision Log

- Decision: Add `QuoteSource::QuoteData` rather than overloading `Chart`.
  Rationale: chart-source quote means selected chart main-series last bar.
  Quote-data WebSocket readback has different timing and session semantics.
  Date/Author: 2026-05-10 / Codex.

- Decision: Do not include quote-data in `--source auto`.
  Rationale: `auto` is chart-first with scanner fallback before chart
  mutation. Quote-data is a new explicit Desktop-backed source and should not
  be selected implicitly.
  Date/Author: 2026-05-10 / Codex.

- Decision: Do not subscribe to quote sessions or switch symbols in the first
  implementation.
  Rationale: previous live investigation showed quote-session probing can
  disturb visible state. The first public source should only observe existing
  page WebSocket traffic and fail safely if no matching readback appears.
  Date/Author: 2026-05-10 / Codex.

## Outcomes & Retrospective

Implemented `tv quote <SYMBOL> --source quote-data` as an explicit
Desktop-backed quote-data readback source. The implementation observes bounded
TradingView WebSocket quote-data events, returns `desktop_quote_data_ws`
payloads with `quote_data.rtc` readback when a matching symbol frame appears,
and returns structured unavailable details when no matching `qsd.rtc` arrives.

The source is intentionally not part of `--source auto`, does not switch chart
symbols, does not mutate quote-session fields, does not call scanner fallback,
and does not merge scanner `extended_hours` or chart main-series fields into
the payload.

## Context and Orientation

`tv quote <SYMBOL>` currently uses scanner REST by default. `tv quote
<SYMBOL> --source chart` connects to TradingView Desktop and reads the
selected chart main-series last bar. This plan adds a third explicit source:
`quote-data`, which connects to the selected TradingView target and observes
Network WebSocket events for TradingView quote-data messages.

The relevant implementation files are `crates/cli/src/cli.rs` for CLI source
selection, `crates/cli/src/app/dispatch.rs` for routing, and
`crates/cli/src/ops/market/quote_data.rs` for the new readback logic. The CDP
event helper lives in `crates/cdp/src/client.rs`.

## Plan of Work

Add `QuoteSource::QuoteData` so clap accepts `--source quote-data`. Dispatch
requires a symbol and connects to Desktop. It must not use scanner fallback
and must not call the chart-source quote path.

In the quote-data operation, enable the CDP Network domain, read WebSocket
events for a short bounded window, and parse TradingView framed socket
messages. Track `quote_add_symbols` and `quote_remove_symbols` sent frames
when they are observed, and accept `qsd` messages only when they can be
attributed to the requested symbol by explicit symbol fields or a single-symbol
quote-session mapping. Return the first matching `qsd.rtc` readback.

The success payload uses `source: "desktop_quote_data_ws"`,
`source_category: "desktop_backed_read"`, `requires_desktop: true`, and
`non_mutating: true`. It has a `quote_data` object with `rtc`, `rtc_time`,
`rch`, `rchp`, `current_session`, `market_phase`, and `update_mode`. It must
not add scanner-style `extended_hours`, and it must say that chart main-series
data is not included.

If no matching `qsd.rtc` arrives, return `internal_api_unavailable` with
public-safe details: source metadata, requested symbol, bounded wait summary,
counts, and `raw_frame_included: false`. Do not include raw frames, URLs,
target ids, account-local metadata, cookies, or local paths.

## Concrete Steps

Run from the repository root:

    cargo test -p tradingview-cli market::quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture
    cargo test -p tradingview-cli --test live_after_hours_ws_correlation -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

Optional live smoke during postmarket:

    target/debug/tv quote NASDAQ:RKLB --source quote-data

The live output must not be pasted into tracked docs. If no frame arrives, the
command should fail with structured unavailable details rather than reporting
stale or guessed data.

## Validation and Acceptance

Acceptance is met when `quote --help` lists `quote-data`, symbol-less
`--source quote-data` fails before network, a simulated matching `qsd.rtc`
produces a payload with `source: "desktop_quote_data_ws"` and
`quote_data.rtc`, and failure details never contain raw WebSocket frames.

Existing scanner, chart, and auto quote tests must continue to pass unchanged.

## Idempotence and Recovery

The operation is read-only from the user's perspective. It enables CDP Network
observation and waits for events; it does not click, type, switch symbols,
subscribe to quote-session fields, take screenshots, or call scanner fallback.
If the page does not emit matching quote-data frames, retrying is safe.

## Artifacts and Notes

Expected success shape, abbreviated:

    source: "desktop_quote_data_ws"
    source_category: "desktop_backed_read"
    requires_desktop: true
    non_mutating: true
    quote_data.rtc: <number or string from qsd>

Expected unavailable shape, abbreviated:

    error.kind: "internal_api_unavailable"
    details.source: "desktop_quote_data_ws"
    details.wait_summary.raw_frame_included: false

## Interfaces and Dependencies

The new public CLI source is:

    tv quote <SYMBOL> --source quote-data

No new external dependency is added. The code uses existing CDP WebSocket
transport and serde JSON parsing.

## Open Questions

Premarket evidence remains uncollected. This source may work there too, but
the implementation must describe `current_session` and `market_phase` as
readbacks rather than asserting premarket/postmarket semantics.
