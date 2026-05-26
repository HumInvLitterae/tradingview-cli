# Expand narrow intraday date-range support for `tv bars`

This ExecPlan is a living document. Keep `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes` updated while working.

## Purpose

`tv bars --from/--to` is now mature for daily, weekly, and monthly historical
ranges, including `range_alignment`, `range_fetch_summary`, and a 5000-bar
date-range safety cap. Feasibility work first showed that count-only `15` and
`60` intraday reads behave through the existing `bars.v1` source, and the
first intraday slice unlocked those two timeframes.

This follow-up expands the narrow stable intraday date-range set to `5`,
`15`, `30`, and `60`. It does not unlock `1`, `3`, `45`, `120`, `180`, or
`240`.

## Progress

- [x] (2026-05-26T09:05Z) Create this ExecPlan and archive the completed
  large-range pagination plan.
- [x] (2026-05-26T09:10Z) Confirm current behavior with public-safe smoke:
  count-only intraday `15` and `60` reads return 500 bars, while intraday
  date-range remains a validation error before network access.
- [x] (2026-05-26T09:15Z) Update the `v0.21.0` roadmap, plan index, and
  changelog to make intraday date-range feasibility the current slice.
- [x] (2026-05-26T10:05Z) Decide to proceed from feasibility to narrow
  implementation for `15` and `60` only.
- [x] (2026-05-26T10:15Z) Expand date-range validation, help, docs, and
  runtime skills for the initial `15`, `60`, `1D`, `1W`, and `1M` supported
  list.
- [x] (2026-05-26T10:35Z) Run focused tests, full baseline, runtime skill
  validation, and public-safe live smoke for `15` and `60` date ranges.
- [x] (2026-05-26T12:05Z) Decide that large-range batching / pagination is
  sufficient for now with the 5000 date-range cap and `range_fetch_summary`.
- [x] (2026-05-26T12:10Z) Expand the narrow intraday stable set to include
  the high-value `5` and `30` timeframes while keeping `1`, `3`, `45`,
  `120`, `180`, and `240` guarded.
- [x] (2026-05-26T12:35Z) Run focused tests, runtime skill validation, and
  public-safe live smoke for `5` and `30` date ranges.

## Surprises & Discoveries

- Count-only intraday continues to work through the existing `bars.v1`
  Desktop-free source boundary. Public-safe smoke with `NASDAQ:AAPL`
  returned 500 bars for both `15` and `60` timeframes.
- The previous date-range guard was clean and early. That made it safe to
  widen the supported list narrowly without changing transport or payload
  semantics.
- `range_alignment` already describes period-start timestamps and the
  `timestamp_within_requested_range` filter policy, so the same additive
  readback can cover `15` and `60` without adding a new field.
- Public-safe live smoke returned complete coverage for both `NASDAQ:AAPL`
  `60` from 2026-05-01 to 2026-05-22 and `NASDAQ:AAPL` `15` from
  2026-05-20 to 2026-05-22.
- User feedback clarified that `5` and `30` are more useful than the less-used
  remaining intraday timeframes for near-term date-spanning reviews. They can
  use the same `bars.v1` range contract without changing pagination or source
  behavior.
- Public-safe live smoke returned complete coverage for both `NASDAQ:AAPL`
  `5` from 2026-05-20 to 2026-05-22 and `NASDAQ:AAPL` `30` from
  2026-05-01 to 2026-05-22.

## Decision Log

- Decision: Initially unlock only `15` and `60` for stable intraday date-range reads in
  this slice.
  Rationale: count-only smoke confirmed these two paths behave through
  `bars.v1`, while smaller or less-proven intraday ranges still carry higher
  retention and timeout risk.
- Decision: Initially keep `1`, `3`, `5`, `30`, `45`, `120`, `180`, and `240` guarded
  for date-range mode.
  Rationale: narrow support gives downstream a usable intraday range surface
  without implying all intraday history is equally available.
- Decision: Add `5` and `30` after the initial `15` / `60` slice.
  Rationale: these two are high-value intraday review intervals and fit the
  same `bars.v1` date-range contract. This still avoids presenting every
  count-only intraday timeframe as equally tested for date-range use.
- Decision: Keep `1`, `3`, `45`, `120`, `180`, and `240` guarded for
  date-range mode.
  Rationale: `1` and `3` can produce large date-spanning result sets quickly,
  while `45`, `120`, `180`, and `240` are lower-priority intervals for the
  current workflow. They should be revisited only with a separate acceptance
  pass.
- Decision: Treat intraday range as the same `bars.v1` source family, not a
  new source.
  Rationale: future implementation should reuse `tradingview_bars_ws`,
  `range_alignment`, `range_fetch_summary`, `source_availability`, and
  `wait_summary`.
- Decision: Do not use hidden fallbacks to prove intraday feasibility.
  Rationale: `tv range`, `tv ohlcv`, Replay, observe / stream, scanner, chart
  quote, and quote-data have different source meanings and would make results
  harder to reproduce.
- Decision: The next implementation slice, if feasible, should start with a
  narrow stable set rather than all intraday timeframes.
  Rationale: retention and entitlement risk is likely higher for older or
  smaller intraday timeframes.

## Outcomes

This plan implements the first stable intraday date-range slices. The accepted
timeframes are `5`, `15`, `30`, `60`, `1D`, `1W`, and `1M`. The still-guarded intraday
timeframes may be revisited only if a later slice can show that:

- unsupported, unavailable, partial, timeout, and count-cap cases remain
  structured source diagnostics;
- `bars.v1` remains additive and keeps `range_alignment`,
  `range_fetch_summary`, `range_coverage_status`, `source_availability`, and
  `wait_summary`;
- count-only intraday and daily / weekly / monthly date-range behavior remain
  unchanged;
- no hidden source fallback is introduced.

If future live evidence remains inconsistent for the remaining intraday
timeframes, they should stay guarded and the next work should improve
structured unavailable wording and docs rather than exposing a broad stable
CLI surface.

## Context

Current behavior:

- `tv bars <SYMBOL> --timeframe 1|3|5|15|30|45|60|120|180|240 --count N`
  is a count-only Desktop-free read with maximum count 500.
- `tv bars <SYMBOL> --timeframe 5|15|30|60|1D|1W|1M --from YYYY-MM-DD --to
  YYYY-MM-DD` is a date-range Desktop-free read with default cap 500 and
  maximum cap 5000.
- `1`, `3`, `45`, `120`, `180`, and `240` date-range requests still fail validation before network
  access.

Future intraday work should evaluate TradingView retention, entitlement,
symbol differences, and source exhaustion without writing raw bars, raw
WebSocket frames, raw payloads, session ids, credentials, target ids, or
account-local metadata into tracked files.

## Work Items

1. Update validation so `5`, `15`, `30`, `60`, `1D`, `1W`, and `1M` are accepted in
   date-range mode.
2. Keep `1`, `3`, `45`, `120`, `180`, and `240` guarded in
   date-range mode.
3. Preserve the payload contract:
   - `range_alignment` remains timeframe-specific timestamp readback;
   - `range_coverage_status` remains the primary date-range coverage field;
   - `range_fetch_summary` explains fetch windows, count caps, and
     truncation reasons;
   - `source_availability` and `wait_summary` explain unavailable or partial
     source behavior.
4. Document that `tv bars` remains the only source path for this work.
5. Run focused tests, baseline validation, and runtime skill validation.

## Validation

Completed validation for the feasibility portion:

- `git diff --check`
- `bash -n scripts/stage-release-package-files.sh`
- Public-safe smoke:
  - count-only `NASDAQ:AAPL` `60` returned 500 bars with complete coverage;
  - count-only `NASDAQ:AAPL` `15` returned 500 bars with complete coverage;
  - intraday date-range `15` remained a validation error with supported
    timeframes `1D`, `1W`, and `1M`.

Completed implementation validation, repeated after the `5` / `30`
expansion:

- `cargo test -p tradingview-market bars -- --nocapture`
- `cargo test -p tradingview-cli market::bars -- --nocapture` (0 matching
  unit tests in the CLI crate)
- `cargo test -p tradingview-cli --test cli_contract_bars -- --nocapture`
- `cargo test -p tradingview-cli --test live_bars`
- runtime skill validation for `market-data-interpretation`, `chart-analysis`,
  and `multi-symbol-scan`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo metadata --no-deps --format-version 1`
- `git diff --check`
- `bash -n scripts/stage-release-package-files.sh`

Public-safe live smoke:

- `NASDAQ:AAPL` `60` from 2026-05-01 to 2026-05-22 returned 112 bars,
  `range_coverage_status: "complete"`, `range_truncated: false`, and
  `range_truncation_reason: "none"`.
- `NASDAQ:AAPL` `15` from 2026-05-20 to 2026-05-22 returned 78 bars,
  `range_coverage_status: "complete"`, `range_truncated: false`, and
  `range_truncation_reason: "none"`.
- `NASDAQ:AAPL` `5` from 2026-05-20 to 2026-05-22 returned 234 bars,
  `range_coverage_status: "complete"`, `range_truncated: false`, and
  `range_truncation_reason: "none"`.
- `NASDAQ:AAPL` `30` from 2026-05-01 to 2026-05-22 returned 208 bars,
  `range_coverage_status: "complete"`, `range_truncated: false`, and
  `range_truncation_reason: "none"`.

Runtime skills edited in this slice were validated with the local skill
validator.
