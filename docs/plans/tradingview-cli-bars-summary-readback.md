# `tv bars` summary readback

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It plans
the first `v0.17.0` implementation slice after the `v0.16.0` release.

## Purpose / Big Picture

`tv bars <EXCHANGE:SYMBOL>` is now a stable Desktop-free historical bars
command. Its current payload exposes raw `bars[]`, `bar_count`, and
`data_quality`, but downstream agents still need a stable first-pass readback
for count coverage and time range.

This slice adds additive summary / range / quality metadata to `bars.v1` while
leaving raw bars and source boundaries intact.

## Progress

- [x] Create this ExecPlan.
- [ ] Add `tv bars` summary / range readback to success payloads.
- [ ] Extend bars structured failure details only where needed for consistent
  source availability readback.
- [ ] Update docs and runtime skills for first-pass summary / range use.
- [ ] Validate focused bars tests, contract tests, baseline checks, and
  public-doc hygiene.

## Surprises & Discoveries

- Pending implementation.

## Decision Log

- Decision: Keep `contract_version: "bars.v1"` for additive summary and range
  metadata.
  Rationale: The planned fields are additive readback and do not break the
  existing raw bars contract.
  Date/Author: 2026-05-13 / Codex.

- Decision: Keep `tv bars` separate from `tv ohlcv`, scanner quote,
  quote-data, and stream commands.
  Rationale: The source, freshness, and operation model are different; merging
  them would recreate the source-boundary confusion previous releases avoided.
  Date/Author: 2026-05-13 / Codex.

## Outcomes & Retrospective

Pending implementation.

## Plan of Work

Add a `summary` object to successful `tv bars` payloads with requested count,
returned count, first/last time, ascending order, requested-count fulfillment,
and `coverage_status: "complete" | "partial"`. Add a `range` object with
timeframe, first time, last time, and bar count. Preserve raw `bars[]`,
`bar_count`, `data_quality`, source metadata, and warnings.

Extend `data_quality` additively only if needed, for example with
`partial_result` or `completion_status`. Do not turn no-bars failures into
success payloads.

Update docs and runtime skills so consumers read `summary` / `range` first and
raw `bars[]` when they need exact OHLCV evidence.

## Concrete Steps

Implementation should:

1. Compute summary / range from the normalized sorted `bars[]` already present
   in the `BarsResult`.
2. Treat `requested_count_fulfilled` as `bar_count == requested_count`.
3. Treat `coverage_status` as `complete` when requested count is fulfilled,
   otherwise `partial`.
4. Keep no-bars as structured failure with `bars.v1` source metadata.
5. Update bars unit tests and CLI contract tests.
6. Update README and docs / runtime skills only where they describe how to
   read `tv bars`.

## Acceptance Criteria

- `tv bars` success payloads include additive `summary` and `range` objects.
- `summary.requested_count`, `summary.bar_count`,
  `summary.requested_count_fulfilled`, and `summary.coverage_status` are
  machine-readable.
- `range.timeframe`, `range.first_time`, `range.last_time`, and
  `range.bar_count` are present when bars are returned.
- Raw `bars[]`, `bar_count`, `data_quality`, source metadata, and warnings are
  not removed or renamed.
- No-bars remains structured failure, not an empty success payload.
- Docs and runtime skills explain that `tv bars` is historical evidence, not
  realtime or trading recommendation output.
- Public docs and packaged assets contain no raw WebSocket frame, raw live
  payload, target id, account-local metadata, credential, local absolute path,
  or downstream-private path.

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
automatic fallback, source mixing, ranking, scoring, recommendation, or trading
action is planned in this slice.
