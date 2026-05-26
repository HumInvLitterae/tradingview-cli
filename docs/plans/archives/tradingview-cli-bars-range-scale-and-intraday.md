# Bars range scale and intraday readiness

This ExecPlan is a living document. The sections `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md` from the repository root. Keep this
document self-contained so a contributor can finish the work from this file
alone.

## Purpose / Big Picture

After `v0.20.0`, `tv bars --from/--to` supports daily, weekly, and monthly
historical range reads through the Desktop-free `bars.v1` source. The next
useful direction is to plan how the same command family should handle larger
historical ranges and future intraday date-range reads.

Large-range batching / pagination and intraday date-range support are related
features. Both need the same foundations: bounded fetch windows, coverage
readback, truncation reporting, source availability diagnostics, and source
boundaries that do not fall back to selected-chart state.

This implementation slice keeps intraday date-range guarded and adds
machine-readable range-scale diagnostics to the existing daily / weekly /
monthly `tv bars --from/--to` surface. It changes only additive `bars.v1`
readback and related docs / tests. It does not add a new command, source,
dependency, or version number.

## Progress

- [x] (2026-05-26T00:00Z) Treat `v0.20.0` as released and archive the completed
  release-readiness plan.
- [x] (2026-05-26T00:05Z) Add the `v0.21.0` roadmap with large-range and
  intraday date-range work under the same historical range maturity theme.
- [x] (2026-05-26T00:10Z) Update the current plan index, previous roadmap, and
  changelog.
- [x] (2026-05-26T00:15Z) Run docs validation and public hygiene checks.
- [x] (2026-05-26T00:30Z) Fix the first implementation slice as
  range-scale readback rather than intraday date-range rollout.
- [x] (2026-05-26T00:40Z) Add `range_fetch_summary` to `bars.v1` success
  payloads and structured failure details.
- [x] (2026-05-26T00:50Z) Add focused unit coverage for count-cap truncation,
  added fetch windows, and count-only intraday preservation.
- [x] (2026-05-26T01:10Z) Sync README, source docs, runtime skills, packaged
  runtime guide, and live smoke contract assertions.
- [x] (2026-05-26T01:25Z) Run focused tests, full Rust baseline, skill
  validation, docs validation, and optional public-safe live smoke summaries.

## Surprises & Discoveries

- Observation: The current `tv bars` transport already has a date-range loop
  that can request more data while the oldest observed bar is newer than the
  requested `from` time.
  Evidence: `crates/market/src/bars/transport.rs` uses
  `request_more_data` through `should_request_more`.

- Observation: `tv bars` date-range mode still validates only daily, weekly,
  and monthly timeframes, while count-only mode accepts intraday timeframes.
  Evidence: `crates/market/src/bars/validation.rs` has
  `DATE_RANGE_TIMEFRAMES` as `["1D", "1W", "1M"]`.

- Observation: Range-scale behavior already needed a clearer distinction
  between observed bars, bars retained by the requested date range, and final
  bars returned after the `--count` safety cap.
  Evidence: `crates/market/src/bars/transport.rs` now builds
  `BarsFetchSummary` from observed, filtered, and returned counts during
  `finalize_result`.

## Decision Log

- Decision: Plan large-range batching / pagination and intraday date-range as
  one `v0.21.0` theme.
  Rationale: Both features need the same range-scale contract and coverage
  vocabulary. Keeping them together avoids building separate semantics for
  daily/weekly/monthly history and future intraday history.
  Date/Author: 2026-05-26 / Codex.

- Decision: Do not make the first implementation slice unlock every intraday
  timeframe at once.
  Rationale: Intraday retention, entitlement, and no-bars behavior can vary by
  symbol and exchange. The safer first step is to harden range-scale contract
  and diagnostics so intraday support can be introduced without confusing
  unavailable, partial, truncated, and unsupported outcomes.
  Date/Author: 2026-05-26 / Codex.

- Decision: Keep `tv bars` as the only source path for this work.
  Rationale: Using `tv range`, `tv ohlcv`, Replay, observe/stream, scanner,
  chart quote, or quote-data as hidden fallbacks would make reproducibility
  and source attribution harder for downstream users.
  Date/Author: 2026-05-26 / Codex.

- Decision: Add `range_fetch_summary` before unlocking intraday date-range.
  Rationale: daily / weekly / monthly date-range reads already exercise the
  same fetch-loop and count-cap mechanics that intraday support will need.
  Making those mechanics explicit first lets future intraday work explain
  unsupported, unavailable, partial, and truncated outcomes without changing
  source boundaries.
  Date/Author: 2026-05-26 / Codex.

- Decision: Keep the `range_truncation_reason` vocabulary small:
  `count_cap`, `source_exhausted`, `timeout`, and `none`.
  Rationale: These are enough to separate returned-count cap, source
  exhaustion, bounded wait timeout, and untruncated success without exposing
  raw WebSocket frames or adding source-specific error strings.
  Date/Author: 2026-05-26 / Codex.

## Outcomes & Retrospective

Implementation is complete. `docs/v0.21-roadmap.md` now treats large-range
batching / pagination and intraday date-range as one historical range
maturity theme. The first implementation slice adds `range_fetch_summary` as
additive `bars.v1` readback while leaving intraday date-range validation
guarded.

Validation passed:

- `git diff --check`
- `bash -n scripts/stage-release-package-files.sh`
- roadmap / docs grep for `v0.21`, range scale, large-range, batching,
  pagination, intraday, `bars.v1`, range coverage, and deferred work
- public hygiene grep, with existing policy / archive / test-example matches
  and no newly introduced private data in the changed v0.21 docs

Rust code adds additive JSON readback only. CLI options, source boundary, Rust
public API, intraday date-range guard, and version number are unchanged.

Validation passed:

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
- `uvx --with pyyaml python ... quick_validate.py` for
  `.agents/skills/market-data-interpretation`, `.agents/skills/chart-analysis`,
  and `.agents/skills/multi-symbol-scan`

Optional public-safe live smoke summaries:

- `NASDAQ:AAPL`, `1D`, 2010-01-01 to 2020-12-31: returned 500 bars,
  `range_coverage_status: "partial"`, `fetch_window_count: 9`,
  `request_more_count: 8`, `range_truncation_reason: "count_cap"`.
- `NASDAQ:CRUS`, `1W`, 2009-01-01 to 2015-12-31: returned 365 bars,
  `range_coverage_status: "complete"`, `fetch_window_count: 2`,
  `request_more_count: 1`, `range_truncation_reason: "none"`.

## Context and Orientation

`tv bars` currently exposes:

- recent-count mode for supported intraday and higher timeframes;
- date-range mode for `1D`, `1W`, and `1M`;
- `bars.v1` source metadata;
- `requested_range`, `returned_range`, `observed_range`,
  `range_coverage_status`, `range_alignment`, `source_availability`, and
  `wait_summary`.

The next implementation keeps those fields stable and adds
`range_fetch_summary` where range-scale behavior needs clearer readback.

## Plan of Work

Extend the market crate `tv bars` range result with a private
`BarsFetchSummary` type. Populate it from the existing transport fetch loop,
tracking initial fetch count, `request_more_data` count, bounded fetch window
count, observed bar count, date-range-filtered count, final returned count,
and truncation reason.

Expose that summary as `range_fetch_summary` in success payloads and
structured failure details. Keep existing `request_mode`, `requested_range`,
`returned_range`, `observed_range`, `range_coverage_status`,
`range_alignment`, `summary`, `source_availability`, `wait_summary`, and
`bars[]` fields unchanged.

Keep intraday date-range validation unchanged: `--from/--to` supports only
`1D`, `1W`, and `1M` in this slice. Count-only intraday `tv bars` remains
unchanged.

Update README, source taxonomy, observation workflows, internal API docs,
runtime skills, roadmap, and changelog to explain `range_fetch_summary`.

## Concrete Steps

From the repository root:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg -n "v0\\.21|range scale|large-range|batching|pagination|intraday|date-range|bars\\.v1|range_alignment|range_coverage_status|source_availability|historical bars|Replay|watch|JSONL compare|chart-backed compare|source mixing|MCP|daemon|ranking|recommendation" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|raw JSONL|raw bars|account-local|target id|downstream-private)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true

Run focused tests and baseline after code changes:

    cargo test -p tradingview-market bars -- --nocapture
    cargo test -p tradingview-cli market::bars -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_bars -- --nocapture
    cargo test -p tradingview-cli --test live_bars
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

## Validation and Acceptance

Acceptance is met when:

- Date-range success payloads include `range_fetch_summary`.
- Full range success reports `range_truncated: false` and
  `range_truncation_reason: "none"`.
- Returned-count-cap truncation reports `range_truncated: true` and
  `range_truncation_reason: "count_cap"`.
- Added `request_more_data` fetch windows are reflected by
  `request_more_count` and `fetch_window_count`.
- Timeout / no-bars / protocol error details retain public-safe
  `range_fetch_summary`, `source_availability`, `wait_summary`, and
  `range_alignment` where available.
- Intraday date-range remains a validation error; count-only intraday remains
  supported.
- `tv range`, `tv ohlcv`, scanner quote, chart quote, quote-data, observe,
  and stream contracts are unchanged.
- No raw live output, raw bars, raw WebSocket frames, raw JSONL output, target
  ids, account-local identifiers, credentials, or local absolute paths are
  added to tracked docs.

## Idempotence and Recovery

This slice is additive. It is safe to rerun validation commands. If the
range-scale vocabulary changes, edit `docs/v0.21-roadmap.md`, this plan,
payload tests, and runtime skills together so the contract remains coherent.

If this slice needs to be reverted, remove `range_fetch_summary` from the
market crate types / payload / tests and revert the docs updates that mention
it. Do not change the already released v0.20 docs or archive state.

## Artifacts and Notes

Do not paste live command output or raw bars into this plan. If later live
smoke is useful, record only public-safe summary fields such as symbol,
timeframe, requested range, returned count, coverage status, and source
availability.
