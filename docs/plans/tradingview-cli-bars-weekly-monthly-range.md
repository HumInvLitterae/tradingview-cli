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
- [ ] Confirm the current daily range implementation points that must be
  widened for weekly and monthly timeframes.
- [ ] Implement weekly/monthly date-range validation and market crate request
  handling while preserving count-only behavior.
- [ ] Confirm payload coverage semantics for daily, weekly, and monthly range
  reads.
- [ ] Update public docs, runtime skills, and help text for weekly/monthly
  date-range reads.
- [ ] Run focused tests, baseline validation, and public-safe optional live
  smoke if useful.
- [ ] Record outcomes and archive this plan after implementation.

## Surprises & Discoveries

- None yet.

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

## Outcomes & Retrospective

No implementation has been completed yet. The expected outcome is additive
weekly/monthly date-range support for `tv bars` with no behavior change to
recent-count mode and no hidden fallback to selected-chart commands.

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

Third, review `crates/market/src/bars/payload.rs` for coverage wording. Weekly
and monthly bars often timestamp bars at the start of the covered week or
month. If the existing `requested_range`, `returned_range`,
`observed_range`, and `range_coverage_status` are sufficient, keep them
unchanged. If ambiguity remains, add only additive readback and document it.
Do not remove or rename `summary`, `range`, `source_availability`,
`wait_summary`, `data_quality`, or raw `bars[]`.

Fourth, update CLI help and docs. README, `docs/command-source-taxonomy.md`,
`docs/observation-workflows.md`, `docs/internal-tradingview-apis.md`, and
runtime skills should describe `1D`, `1W`, and `1M` date-range mode as the
formal historical sample preparation entrypoint. They should continue to say
that `tv range`, `tv ohlcv`, Replay, observe/stream, scanner quote, chart
quote, and quote-data are separate sources and not hidden fallbacks.

Fifth, update tests. Add or modify market crate tests and
`cli_contract_bars` tests so weekly and monthly date-range validation passes
before network access, while intraday and unsupported range timeframes remain
rejected in this slice. Preserve all count-only behavior and existing
`bars.v1` contract tests.

## Concrete Steps

Work from the repository root.

1. Inspect the current daily implementation:

       rg -n "validate_bars_range_request|DateRange|range_coverage_status|supported_timeframes|1D" crates/market/src/bars crates/cli/src crates/cli/tests

2. Implement the narrow timeframe widening for date-range mode. The expected
   supported range timeframes after this slice are `1D`, `1W`, and `1M`.

3. Update tests and docs, then run focused validation:

       cargo test -p tradingview-market bars -- --nocapture
       cargo test -p tradingview-cli market::bars -- --nocapture
       cargo test -p tradingview-cli --test cli_contract_bars -- --nocapture
       cargo test -p tradingview-cli --test live_bars

4. Run baseline validation:

       cargo fmt --check
       cargo clippy --workspace --all-targets --all-features -- -D warnings
       cargo test --workspace
       cargo metadata --no-deps --format-version 1
       git diff --check
       bash -n scripts/stage-release-package-files.sh

5. Optional live smoke can be run when useful. Do not paste raw bars into
   tracked docs; record only symbol, timeframe, bar count, returned range, and
   coverage status.

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
- count-only mode remains unchanged;
- successful range payloads preserve `contract_version: "bars.v1"`,
  `source: "tradingview_bars_ws"`, `source_category: "desktop_free_read"`,
  `requires_desktop: false`, `non_mutating: true`, `summary`, `range`,
  `source_availability`, `wait_summary`, and raw `bars[]`;
- structured failures remain public-safe and do not include raw WebSocket
  frames, raw payloads, session ids, credentials, account-local metadata, or
  local paths;
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
- Focused tests should prove that intraday date-range remains deferred.
- Optional live smoke should record only public-safe summary such as
  timeframe, returned bar count, returned range, and coverage status.

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
  Current expectation: only docs wording may be needed. If a payload field is
  needed, add it without changing existing fields.

Revision note 2026-05-26: Created after `v0.19.0` release to plan
weekly/monthly historical bars range maturity as the first `v0.20.0` slice.
