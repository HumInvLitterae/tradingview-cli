# `tv bars` availability readback

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It plans
the second `v0.17.0` implementation slice after the browserless bars summary
readback work.

## Purpose / Big Picture

`tv bars <EXCHANGE:SYMBOL>` now has stable `bars.v1` source metadata and
first-pass `summary` / `range` readback. The remaining gap is source
availability: downstream agents need to distinguish a bounded historical bars
source that returned no evidence from a symbol that has no price or no useful
history.

This slice adds additive `source_availability` and `wait_summary` readback to
success payloads and structured failures while keeping no-bars as failure and
preserving the Desktop-free historical bars source boundary.

## Progress

- [x] Create this ExecPlan and switch current planning docs to bars
  availability readback.
- [x] Add `source_availability` and public-safe `wait_summary` to successful
  `tv bars` payloads.
- [x] Add `source_availability` to structured bars failures for timeout,
  WebSocket, connection, and protocol-error paths.
- [x] Update tests for success, partial timeout, and no-bars availability
  readback.
- [x] Update public docs and runtime skills.
- [x] Validate focused tests, baseline checks, and hygiene.

## Surprises & Discoveries

- The existing bars fetch loop already knows whether `series_completed`
  arrived, but it did not expose the intermediate WebSocket/message counts.
  Adding a small public-safe wait summary made partial completion and no-bars
  failures easier to explain without returning raw frames.
- No-bars remains a top-level structured failure. The new availability object
  clarifies that this is a bounded source readback result, not evidence that
  the requested symbol lacks market history.

## Decision Log

- Decision: Keep `contract_version: "bars.v1"` for the availability fields.
  Rationale: The fields are additive diagnostics on the same stable
  browserless historical bars source.
  Date/Author: 2026-05-14 / Codex.

- Decision: Use `source_availability.status: "available" | "unavailable"`
  and a small unavailable-reason enum instead of changing top-level success
  semantics.
  Rationale: Existing no-bars behavior should not silently become an empty
  success payload, while successful partial bars should remain usable evidence.
  Date/Author: 2026-05-14 / Codex.

- Decision: Keep wait summaries count-only and public-safe.
  Rationale: Consumers need bounded wait and completion diagnostics, not raw
  TradingView WebSocket frames, session ids, credentials, or payloads.
  Date/Author: 2026-05-14 / Codex.

## Outcomes & Retrospective

Implemented additive `source_availability` on successful `tv bars` payloads
and structured failure details. Successful payloads now report whether the
source was available, requested and returned counts, count fulfillment,
bounded timeout state, and a public-safe wait summary.

Failure details now carry `source_availability` for connection failures,
WebSocket close/read failures, protocol errors, and no-bars timeouts. No-bars
remains a structured failure instead of an empty success payload.

Raw `bars[]`, `summary`, `range`, `data_quality`, source metadata, and
warnings were preserved. The implementation does not add realtime behavior,
new data sources, source mixing, ranking, scoring, recommendation, or trading
actions.

## Plan of Work

Add `data.source_availability` to successful `tv bars` payloads with:

- `available: true`;
- `status: "available"`;
- `unavailable_reason: null`;
- `requested_count`;
- `bar_count`;
- `requested_count_fulfilled`;
- `timed_out`;
- `raw_frame_included: false`;
- `wait_summary`.

Extend structured failure details with the same object shape using
`available: false`, `status: "unavailable"`, and one of the public-safe
unavailable reasons:

- `connection_failed`;
- `websocket_closed`;
- `websocket_read_failed`;
- `protocol_error`;
- `timeout_no_bars`.

The wait summary should contain only bounded wait metadata and counts:
`timeout_ms`, `elapsed_ms`, `completed`, WebSocket message/packet counts,
update count, series-completed presence, error count, observed bars count, and
`raw_frame_included: false`.

## Concrete Steps

Implementation should:

1. Track public-safe wait counters in the bars WebSocket read loop.
2. Preserve existing `bars.v1` source metadata and no-bars failure behavior.
3. Add availability readback to success payloads.
4. Add availability readback to structured failures without exposing raw
   frames, raw payloads, session ids, credentials, account-local metadata, or
   target ids.
5. Update focused unit tests, CLI contract tests, and the ignored live smoke
   contract checks.
6. Update docs and runtime skills so agents read bars unavailable as source
   diagnostics, not price absence or trading signal.

## Acceptance Criteria

- `tv bars` success payloads include `source_availability.available == true`,
  `status == "available"`, `unavailable_reason == null`, and
  `raw_frame_included == false`.
- Full-count and partial-count success both report requested count
  fulfillment in `source_availability`.
- Partial success without `series_completed` reports
  `source_availability.timed_out == true`.
- No-bars structured failure reports
  `source_availability.unavailable_reason == "timeout_no_bars"`.
- WebSocket and protocol failures report public-safe unavailable reasons.
- `wait_summary` contains counts and bounded wait metadata only.
- `bars[]`, `summary`, `range`, `data_quality`, source metadata, and warnings
  are not removed or renamed.
- `tv ohlcv`, scanner quote, chart quote, quote-data, and stream commands are
  not changed.

## Validation

Run:

    cargo test -p tradingview-cli market::bars -- --nocapture
    cargo test -p tradingview-cli --test cli_contract bars -- --nocapture
    cargo test -p tradingview-cli --test live_bars
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

Optional live smoke:

    target/debug/tv bars NASDAQ:AAPL --timeframe 1D --count 5
    target/debug/tv bars NASDAQ:RKLB --timeframe 1 --count 10

Live output must not be pasted into tracked docs. Record only public-safe
summary if useful.

## Interfaces and Dependencies

No new command, option, dependency, data source, version bump, realtime feed,
automatic fallback, source mixing, ranking, scoring, recommendation, or
trading action is planned in this slice.
