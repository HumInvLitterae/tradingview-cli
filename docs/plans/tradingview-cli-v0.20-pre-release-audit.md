# v0.20.0 pre-release completion and refactor audit

This ExecPlan is a living document. The sections `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md` from the repository root. Keep this
document self-contained so a contributor can finish the audit from this file
alone.

## Purpose / Big Picture

`v0.20.0` adds weekly and monthly date-range readback to the existing
Desktop-free `tv bars` historical bars surface. The implementation keeps
`bars.v1`, adds `range_alignment`, and preserves the boundary that `tv bars`
does not fall back to selected-chart commands, Replay, scanner quote, chart
quote, quote-data, observe/stream, or source mixing.

This slice is the completion / refactor audit before release readiness. It
does not add features, options, dependencies, payload semantics, or a version
bump. The expected result is a durable audit record saying whether `v0.20.0`
is ready to move to release readiness.

## Progress

- [x] (2026-05-26T04:00Z) Archive the completed weekly/monthly date-range
  implementation plan and make this audit the current plan.
- [x] (2026-05-26T04:05Z) Review the `bars.v1` date-range contract,
  source-boundary docs, runtime skills, and help text for consistency.
- [x] (2026-05-26T04:20Z) Run audit greps, focused contract tests, baseline
  Rust validation, runtime skill validation, and optional public-safe smoke.
- [x] (2026-05-26T04:35Z) Record lane judgments, refactor assessment, and
  release-readiness recommendation.

## Surprises & Discoveries

- No release blocker was found.
- The optional public-safe smoke confirmed `NASDAQ:CRUS` 2010 date-range
  reads across `1D`, `1W`, and `1M` with complete range coverage. Only symbol,
  timeframe, bar count, returned range, coverage status, and range alignment
  were inspected and recorded; no raw bars were added to tracked docs.

## Decision Log

- Decision: Treat weekly/monthly date-range readback as complete for
  `v0.20.0`.
  Rationale: Validation, payload tests, docs, runtime skills, and public-safe
  live smoke all agree that `1D`, `1W`, and `1M` date-range reads use the same
  Desktop-free `bars.v1` source boundary.
  Date/Author: 2026-05-26 / Codex.

- Decision: Treat `range_alignment` as sufficient coverage-semantics polish
  for this release.
  Rationale: It makes period-start timestamp semantics,
  `timestamp_within_requested_range`, and inclusive calendar-date input
  machine-readable without changing raw `bars[]`, `requested_range`,
  `returned_range`, `observed_range`, or `range_coverage_status`.
  Date/Author: 2026-05-26 / Codex.

- Decision: Do not perform additional pre-release refactoring.
  Rationale: The relevant implementation remains in the Desktop-free market
  crate behind the existing `bars_symbol(...)` / `bars_symbol_range(...)`
  facade, CLI `ops` remains a thin adapter, and the validation / payload /
  transport module split is still appropriate. Further abstraction would be
  speculative before another range mode or bars source exists.
  Date/Author: 2026-05-26 / Codex.

## Outcomes & Retrospective

The audit is complete and found no release blocker. The next step is
`v0.20.0 release readiness`.

Lane judgments:

- Lane 1: weekly / monthly date-range readback is complete for `v0.20.0`.
- Lane 2: date-range coverage semantics polish is complete for `v0.20.0`.
- Lane 3: source-guided historical sample workflow clarity is complete for
  `v0.20.0`.
- Deferred after `v0.20.0`: intraday date-range reads, large-range batching,
  automatic historical export, Replay-based extraction, realtime feeds,
  watch / JSONL compare, chart-backed compare, source mixing, ranking,
  scoring, and recommendation behavior.

Refactor assessment:

- No release-blocking refactor is needed.
- `tv bars` validation, range filtering, range-alignment payload construction,
  structured failure details, CLI adapter behavior, help text, docs, and
  runtime skills are aligned with the current contract.
- Keep any future refactor tied to a concrete follow-up such as intraday
  range support, large-range batching, or a second Desktop-free range source.

Validation passed:

- `git diff --check`
- `bash -n scripts/stage-release-package-files.sh`
- audit and hygiene `rg` scans
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo metadata --no-deps --format-version 1`
- `cargo test -p tradingview-market bars -- --nocapture`
- `cargo test -p tradingview-cli market::bars -- --nocapture`
- `cargo test -p tradingview-cli --test cli_contract_bars -- --nocapture`
- `cargo test -p tradingview-cli --test live_bars`
- `uvx --with pyyaml python .../quick_validate.py` for the updated
  `market-data-interpretation`, `chart-analysis`, and `multi-symbol-scan`
  runtime skills

The audit and optional smoke did not add raw bars, raw WebSocket frames, raw
payloads, raw JSONL output, session ids, credentials, account-local metadata,
target ids, downstream-private paths, or local absolute paths to tracked docs.

## Context and Orientation

The current `bars.v1` date-range readback fields are:

- `request_mode: "date_range"`;
- `requested_range`;
- `returned_range`;
- `observed_range`;
- `range_coverage_status`;
- `range_alignment`;
- `source_availability`;
- `wait_summary`;
- raw `bars[]` as source evidence.

For date-range mode, `range_coverage_status` is the primary coverage readback.
It can be complete even when count-oriented fields such as
`summary.coverage_status` or `data_quality.partial_result` indicate the count
safety cap was not fully consumed.

`range_alignment` is additive and currently reports:

- `timeframe`;
- `bar_timestamp_semantics: "period_start"`;
- `range_filter_policy: "timestamp_within_requested_range"`;
- `requested_range_interpretation: "inclusive_calendar_dates"`.

`tv range` and `tv ohlcv` are separate selected-chart commands. They are not
hidden historical export fallbacks for `tv bars`.

## Validation and Acceptance

Acceptance is met when:

- `docs/plans/README.md` points to this audit as the current plan;
- the completed weekly/monthly implementation plan is archived;
- `docs/v0.20-roadmap.md` records the three completed lanes and the deferred
  work;
- `tv bars` docs and runtime skills describe `1D`, `1W`, and `1M` date-range
  mode consistently;
- tests and hygiene checks listed above pass;
- no release-blocking refactor remains;
- the next step is clearly recorded as `v0.20.0 release readiness`.
