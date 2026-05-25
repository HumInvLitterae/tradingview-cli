# Extend `tv bars` date-range readback to weekly and monthly bars

This ExecPlan is a living document. The sections `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md` from the repository root. Keep this
document self-contained so a contributor can implement the slice from this file
alone.

## Purpose / Big Picture

After `v0.19.0`, users can request reproducible daily historical bars with
`tv bars <EXCHANGE:SYMBOL> --timeframe 1D --from YYYY-MM-DD --to YYYY-MM-DD`.
That solved the immediate downstream need for old daily source-guided samples,
but many historical chart examples also need weekly and monthly context. This
plan extends the same Desktop-free `tv bars` date-range surface to weekly and
monthly bars while preserving the existing `bars.v1` contract and source
boundary.

The user-visible outcome is that a command such as
`tv bars NASDAQ:CRUS --timeframe 1W --from 2010-01-01 --to 2010-12-31`
returns a normal `bars.v1` payload with weekly bars for the requested range,
without relying on selected-chart viewport movement, Replay, screenshots,
or source mixing.

## Progress

- [x] (2026-05-26T00:00Z) Create the `v0.20.0` roadmap and this first
  implementation ExecPlan.
- [x] (2026-05-26T01:00Z) Incorporate downstream feedback by fixing the
  weekly/monthly range filter policy and acceptance criteria before
  implementation.
- [x] (2026-05-26T02:00Z) Confirm the current daily range implementation points that must be
  widened for weekly and monthly timeframes.
- [x] (2026-05-26T02:15Z) Implement weekly/monthly date-range validation and market crate request
  handling while preserving count-only behavior.
- [x] (2026-05-26T02:25Z) Add `range_alignment` readback and confirm payload coverage semantics
  for daily, weekly, and monthly range reads.
- [x] (2026-05-26T02:30Z) Confirm structured failure details keep range/source readback public-safe.
- [x] (2026-05-26T02:40Z) Update public docs, runtime skills, and help text for weekly/monthly
  date-range reads.
- [x] (2026-05-26T03:15Z) Run focused tests, baseline validation, and public-safe optional live
  smoke if useful.
- [x] (2026-05-26T03:20Z) Record outcomes. Archive this plan in the next
  planning slice.

## Surprises & Discoveries

- A CLI contract probe for weekly date-range behavior unexpectedly reached the
  Desktop-free WebSocket path and returned a successful weekly `bars.v1`
  payload. The raw output was not recorded in tracked docs. The test was
  changed back to validation-only coverage so `cli_contract_bars` stays
  network-independent, while weekly/monthly validation and payload behavior are
  covered in market crate tests.

## Decision Log

- Decision: Treat weekly/monthly date-range reads as the first `v0.20.0`
  implementation candidate.
  Rationale: They are a natural extension of daily historical sample
  preparation and are less likely than intraday history to run into retention
  and entitlement ambiguity.
  Date/Author: 2026-05-26 / Codex.

- Decision: Keep intraday date-range reads deferred.
  Rationale: Intraday historical availability is more likely to vary by
  retention, entitlement, and symbol. Weekly/monthly support should be proven
  first on the existing Desktop-free bars path.
  Date/Author: 2026-05-26 / Codex.

- Decision: Use `timestamp_within_requested_range` for weekly/monthly
  date-range filtering in the first slice.
  Rationale: Filtering by returned bar timestamp is reproducible and matches
  the existing daily range mechanics. A policy based on whether an inferred
  weekly or monthly period intersects the requested range is more natural for
  some human chart reads, but it requires period inference and is deferred.
  Date/Author: 2026-05-26 / Codex.

- Decision: Add `range_alignment` as additive `bars.v1` readback.
  Rationale: Downstream agents need to know that weekly/monthly bar timestamps
  are period anchors and that `range_coverage_status` is a date-range source
  readback, not a pattern-quality or trading judgment. An explicit field is
  safer than relying on prose alone.
  Date/Author: 2026-05-26 / Codex.

- Decision: Keep CLI contract tests network-independent for weekly/monthly
  date-range support.
  Rationale: The weekly/monthly feature uses a Desktop-free live WebSocket
  source after validation. Integration contract tests should continue to prove
  help text and network-before-validation behavior, while market crate tests
  cover request validation and payload construction deterministically.
  Date/Author: 2026-05-26 / Codex.

## Outcomes & Retrospective

Implementation is complete. The user-visible outcome is additive
weekly/monthly date-range support for `tv bars` with no behavior change to
recent-count mode and no hidden fallback to selected-chart commands. `bars.v1`
date-range payloads and structured failure details now include
`range_alignment`, using
`timestamp_within_requested_range` and `period_start` timestamp semantics as
explicit source readback.

Focused and baseline validation passed:

- `cargo test -p tradingview-market bars -- --nocapture`
- `cargo test -p tradingview-cli market::bars -- --nocapture`
- `cargo test -p tradingview-cli --test cli_contract_bars -- --nocapture`
- `cargo test -p tradingview-cli --test live_bars`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo metadata --no-deps --format-version 1`
- `git diff --check`
- `bash -n scripts/stage-release-package-files.sh`
- `uvx --with pyyaml python .../quick_validate.py` for the updated
  `market-data-interpretation`, `chart-analysis`, and `multi-symbol-scan`
  runtime skills

Optional live smoke was run with public-safe summary output only. For
`NASDAQ:CRUS` over `2010-01-01` through `2010-12-31`, `1W` returned 52 bars
with complete range coverage and `1M` returned 12 bars with complete range
coverage. Both reported `range_alignment.range_filter_policy:
"timestamp_within_requested_range"` and `bar_timestamp_semantics:
"period_start"`. No raw bars, raw WebSocket frames, raw payloads, session ids,
credentials, account-local metadata, target ids, or local paths were added to
tracked docs.

## Context and Orientation

`tv bars` is the Desktop-free historical OHLCV command. Desktop-free means it
does not require a running TradingView Desktop chart or Chrome DevTools
Protocol target. Its public JSON contract is marked with
`contract_version: "bars.v1"` and `source: "tradingview_bars_ws"`.

The command is implemented in the market crate and exposed through a thin CLI
adapter:

- `crates/market/src/bars.rs` is the facade for the market crate bars API.
- `crates/market/src/bars/validation.rs` validates symbols, timeframes,
  counts, and date ranges.
- `crates/market/src/bars/types.rs` defines request modes and result state.
- `crates/market/src/bars/transport.rs` performs the browserless WebSocket
  historical bars read.
- `crates/market/src/bars/payload.rs` shapes the `bars.v1` success and
  structured failure payloads.
- `crates/cli/src/ops/market/bars.rs` adapts CLI arguments to the market
  crate and should stay thin.
- `crates/cli/tests/cli_contract_bars.rs` contains CLI contract tests for
  help text and validation behavior.

`tv range` is different: it moves the visible Desktop chart viewport. `tv
ohlcv` is also different: it reads selected-chart bars through Desktop-backed
CDP. This plan must not use those selected-chart commands as hidden fallbacks.

The existing date-range mode is daily-only. It accepts paired `--from` and
`--to` dates, treats `--to` as an inclusive calendar date for the user, and
uses an internal exclusive timestamp bound for filtering. In date-range mode,
`--count` is a safety cap and defaults to `500`; in recent-count mode,
`--count` remains the requested number of recent bars and defaults to `100`.

For weekly and monthly bars, TradingView commonly timestamps a bar at the
start of the covered period. In this plan, the date-range filter is
`timestamp_within_requested_range`: a bar is returned only when its timestamp
falls inside the user's requested calendar range. This means a weekly or
monthly period that overlaps the first requested days may still be omitted if
the bar timestamp is before `--from`. That tradeoff is intentional for this
slice because it avoids hidden period inference.

Date-range payloads already include `range_coverage_status`. Downstream
helpers treat this top-level field as the primary date-range coverage
readback. It can be `complete` even when count-oriented fields such as
`summary.coverage_status` or `data_quality.partial_result` say the bounded
count was not fulfilled. The implementation and docs must preserve that
distinction.

## Plan of Work

First, inspect `crates/market/src/bars/validation.rs` and remove the
date-range-only restriction that currently permits only `1D`. Replace it with
support for `1D`, `1W`, and `1M` in date-range mode. Keep invalid date,
paired-date, `from > to`, count zero, and count greater than the safety limit
validation behavior unchanged.

Second, confirm that `crates/market/src/bars/types.rs`,
`crates/market/src/bars/transport.rs`, and
`crates/market/src/bars/protocol.rs` already pass normalized timeframe strings
through the browserless chart-session request path. If weekly and monthly
bars use the same TradingView WebSocket path as daily bars, keep the
implementation narrow: widen supported range timeframes and preserve the
existing pagination and range filtering behavior.

Third, update `crates/market/src/bars/payload.rs` with additive range
alignment readback. Success payloads in date-range mode should include
`range_alignment` with at least these fields:

    "timeframe": "<normalized timeframe>",
    "bar_timestamp_semantics": "period_start",
    "range_filter_policy": "timestamp_within_requested_range",
    "requested_range_interpretation": "inclusive_calendar_dates"

The same `range_alignment` object should be present in structured date-range
failure details whenever the request reached range-aware validation. Do not
remove or rename `requested_range`, `returned_range`, `observed_range`,
`range_coverage_status`, `summary`, `range`, `source_availability`,
`wait_summary`, `data_quality`, or raw `bars[]`.

Fourth, keep structured failure details stable. No-bars, timeout, WebSocket
close, read failure, protocol error, and unsupported timeframe outcomes should
return source diagnostics rather than implying that a symbol has no historical
price or that a pattern is invalid. Where available, details should include
`contract_version`, `requested_symbol`, `requested_timeframe`, `request_mode`,
`requested_range`, `range_alignment`, `source`, `source_category`,
`requires_desktop`, `non_mutating`, `source_availability`, `wait_summary`, and
`next_action_hint`. Never include raw bars, raw WebSocket frames, raw payloads,
session ids, credentials, account-local metadata, or local paths.

Fifth, update CLI help and docs. README, `docs/command-source-taxonomy.md`,
`docs/observation-workflows.md`, `docs/internal-tradingview-apis.md`, and
runtime skills should describe `1D`, `1W`, and `1M` date-range mode as the
formal historical sample preparation entrypoint. Daily bars are for detailed
historical pattern work; weekly and monthly bars are for higher-timeframe
context. The docs must explain that `range_coverage_status` is the primary
date-range coverage field and that `tv range`, `tv ohlcv`, Replay,
observe/stream, scanner quote, chart quote, and quote-data are separate
sources and not hidden fallbacks.

Sixth, update tests. Add or modify market crate tests and
`cli_contract_bars` tests so weekly and monthly date-range validation passes
before network access, while intraday and unsupported range timeframes remain
rejected in this slice. Add payload tests for `range_alignment` on success and
structured failure details. Preserve all count-only behavior and existing
`bars.v1` contract tests.

## Concrete Steps

Work from the repository root.

1. Inspect the current daily implementation:

       rg -n "validate_bars_range_request|DateRange|range_coverage_status|supported_timeframes|1D" crates/market/src/bars crates/cli/src crates/cli/tests

2. Implement the narrow timeframe widening for date-range mode. The expected
   supported range timeframes after this slice are `1D`, `1W`, and `1M`.

3. Add `range_alignment` readback and structured failure coverage. Confirm
   the chosen policy is visible in payload tests:

       rg -n "range_alignment|bar_timestamp_semantics|range_filter_policy|timestamp_within_requested_range" crates/market/src/bars crates/cli/tests

4. Update tests and docs, then run focused validation:

       cargo test -p tradingview-market bars -- --nocapture
       cargo test -p tradingview-cli market::bars -- --nocapture
       cargo test -p tradingview-cli --test cli_contract_bars -- --nocapture
       cargo test -p tradingview-cli --test live_bars

5. Run baseline validation:

       cargo fmt --check
       cargo clippy --workspace --all-targets --all-features -- -D warnings
       cargo test --workspace
       cargo metadata --no-deps --format-version 1
       git diff --check
       bash -n scripts/stage-release-package-files.sh

6. Optional live smoke can be run when useful. Do not paste raw bars into
   tracked docs; record only symbol, timeframe, bar count, returned range,
   coverage status, and range alignment.

       target/debug/tv bars NASDAQ:CRUS --timeframe 1W --from 2010-01-01 --to 2010-12-31
       target/debug/tv bars NASDAQ:CRUS --timeframe 1M --from 2010-01-01 --to 2010-12-31

## Validation and Acceptance

Acceptance is met when:

- `tv bars --help` describes date-range mode for daily, weekly, and monthly
  historical bars;
- `tv bars <SYMBOL> --timeframe 1W --from YYYY-MM-DD --to YYYY-MM-DD` passes
  validation and can return a `bars.v1` payload when the source returns bars;
- `tv bars <SYMBOL> --timeframe 1M --from YYYY-MM-DD --to YYYY-MM-DD` passes
  validation and can return a `bars.v1` payload when the source returns bars;
- date-range mode still rejects intraday timeframes in this slice;
- date-range filtering uses `timestamp_within_requested_range`;
- date-range payloads include `range_alignment.timeframe`,
  `range_alignment.bar_timestamp_semantics: "period_start"`,
  `range_alignment.range_filter_policy: "timestamp_within_requested_range"`,
  and `range_alignment.requested_range_interpretation:
  "inclusive_calendar_dates"`;
- docs explain that weekly/monthly period bars can overlap the requested start
  date but be omitted when the bar timestamp is before `--from`;
- `range_coverage_status` is treated as the primary date-range coverage
  readback in tests and docs;
- count-only mode remains unchanged;
- successful range payloads preserve `contract_version: "bars.v1"`,
  `source: "tradingview_bars_ws"`, `source_category: "desktop_free_read"`,
  `requires_desktop: false`, `non_mutating: true`, `summary`, `range`,
  `source_availability`, `wait_summary`, and raw `bars[]`;
- structured failures remain public-safe and do not include raw WebSocket
  frames, raw payloads, session ids, credentials, account-local metadata, or
  local paths;
- structured date-range failures include source metadata, requested
  symbol/timeframe, `request_mode`, `requested_range`, `range_alignment`,
  `source_availability`, `wait_summary`, and a follow-up hint when those
  fields are available;
- `tv range`, `tv ohlcv`, scanner quote, chart quote, quote-data,
  observe/stream, and Replay behavior are unchanged.

## Idempotence and Recovery

This slice is additive and safe to retry. If weekly or monthly support proves
infeasible on the existing browserless chart-session path, revert the narrow
validation widening, record public-safe evidence in this plan, and do not add
a hidden fallback to selected-chart commands. Keep raw live payloads and raw
WebSocket frames out of tracked files.

If tests fail after docs-only changes, inspect whether the failure is related
to this slice before changing code. Do not mix dependency updates, release
version bumps, or CI workflow changes into this implementation.

## Artifacts and Notes

Important planned proof points:

- CLI help and validation should show that date-range mode supports `1D`,
  `1W`, and `1M`.
- `range_alignment` should make weekly/monthly timestamp and filtering
  semantics visible to downstream tools without requiring them to infer the
  policy from prose.
- Focused tests should prove that intraday date-range remains deferred.
- Optional live smoke should record only public-safe summary such as
  timeframe, returned bar count, returned range, coverage status, and range
  alignment.

## Interfaces and Dependencies

The existing public Rust API must remain:

    pub async fn bars_symbol(symbol: &str, timeframe: &str, count: u16) -> Result<serde_json::Value, TradingViewError>

The narrow range API should remain available for CLI use:

    pub async fn bars_symbol_range(symbol: &str, timeframe: &str, from: &str, to: &str, count_cap: u16) -> Result<serde_json::Value, TradingViewError>

Do not add a new crate dependency unless implementation proves it is strictly
necessary. Do not add a new command, new source, new source-mixing behavior,
ranking, scoring, or buy/sell recommendation surface.

## Open Questions

- Do weekly and monthly range reads work through the same browserless
  TradingView chart-session request path as daily bars?
  Current expectation: yes, because count-only mode already accepts normalized
  weekly and monthly timeframes. Confirm with tests and optional public-safe
  live smoke.

- Does `range_coverage_status` need additive clarification for weekly/monthly
  timestamp semantics?
  Answer: yes. Add `range_alignment` with period-start timestamp semantics and
  the `timestamp_within_requested_range` filter policy. This keeps the
  existing fields intact while preventing downstream over-interpretation.

Revision note 2026-05-26: Created after `v0.19.0` release to plan
weekly/monthly historical bars range maturity as the first `v0.20.0` slice.

Revision note 2026-05-26: Updated after downstream review to lock the
`timestamp_within_requested_range` filter policy, add `range_alignment`
readback, and require structured date-range failure metadata.
