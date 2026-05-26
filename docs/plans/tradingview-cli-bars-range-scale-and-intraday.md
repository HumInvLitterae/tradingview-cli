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

This planning slice adds the v0.21 roadmap and records the first
implementation candidate. It does not change CLI behavior, JSON payloads,
Rust APIs, or version numbers.

## Progress

- [x] (2026-05-26T00:00Z) Treat `v0.20.0` as released and archive the completed
  release-readiness plan.
- [x] (2026-05-26T00:05Z) Add the `v0.21.0` roadmap with large-range and
  intraday date-range work under the same historical range maturity theme.
- [x] (2026-05-26T00:10Z) Update the current plan index, previous roadmap, and
  changelog.
- [x] (2026-05-26T00:15Z) Run docs validation and public hygiene checks.

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

## Outcomes & Retrospective

Planning is complete. `docs/v0.21-roadmap.md` now treats large-range
batching / pagination and intraday date-range as one historical range maturity
theme. This plan is the current ExecPlan, and the completed `v0.20.0`
release-readiness plan is archived.

Validation passed:

- `git diff --check`
- `bash -n scripts/stage-release-package-files.sh`
- roadmap / docs grep for `v0.21`, range scale, large-range, batching,
  pagination, intraday, `bars.v1`, range coverage, and deferred work
- public hygiene grep, with existing policy / archive / test-example matches
  and no newly introduced private data in the changed v0.21 docs

No Rust code, CLI behavior, JSON payload, Rust API, or version number changed.

## Context and Orientation

`tv bars` currently exposes:

- recent-count mode for supported intraday and higher timeframes;
- date-range mode for `1D`, `1W`, and `1M`;
- `bars.v1` source metadata;
- `requested_range`, `returned_range`, `observed_range`,
  `range_coverage_status`, `range_alignment`, `source_availability`, and
  `wait_summary`.

The next implementation candidate should keep those fields stable and add only
additive diagnostics where range-scale behavior needs clearer readback.

## Plan of Work

Create `docs/v0.21-roadmap.md` with lanes for range scale foundation,
intraday date-range feasibility and contract, unified range coverage
semantics, sample preparation workflow, and deferred work.

Make this plan the current ExecPlan. Move
`docs/plans/tradingview-cli-v0.20.0-release-readiness.md` into
`docs/plans/archives/`, update `docs/plans/README.md`, and update
`docs/v0.20-roadmap.md` so it records the transition to v0.21.

Record the first implementation candidate as range-scale / intraday readiness,
not a full intraday rollout. The candidate should plan additive readback such
as `range_fetch_summary`, `fetch_window_count`, `requested_count_cap`,
`returned_count`, `range_truncated`, and `range_truncation_reason` if later
implementation proves those fields useful.

Update `CHANGELOG.md` under `Unreleased` as a roadmap/docs update.

## Concrete Steps

From the repository root:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg -n "v0\\.21|range scale|large-range|batching|pagination|intraday|date-range|bars\\.v1|range_alignment|range_coverage_status|source_availability|historical bars|Replay|watch|JSONL compare|chart-backed compare|source mixing|MCP|daemon|ranking|recommendation" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|raw JSONL|raw bars|account-local|target id|downstream-private)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true

This slice is docs-only. Do not run Rust baseline unless Rust code or
packaging behavior changes.

## Validation and Acceptance

Acceptance is met when:

- `docs/v0.21-roadmap.md` exists and treats large-range batching / pagination
  and intraday date-range as one historical range maturity theme.
- The first ExecPlan is
  `docs/plans/tradingview-cli-bars-range-scale-and-intraday.md`.
- `docs/plans/README.md` and `docs/v0.20-roadmap.md` point to the v0.21
  transition.
- `CHANGELOG.md` records the roadmap/docs update under `Unreleased`.
- Deferred work is explained by function, without promising source mixing,
  automatic exports, ranking, recommendations, MCP server work, or daemon
  behavior.
- No raw live output, raw bars, raw WebSocket frames, raw JSONL output, target
  ids, account-local identifiers, credentials, or local absolute paths are
  added to tracked docs.

## Idempotence and Recovery

This slice only edits docs. It is safe to rerun validation commands. If the
roadmap direction changes, edit `docs/v0.21-roadmap.md` and this plan together
so the current plan and roadmap remain consistent.

If this slice needs to be reverted, move the archived v0.20 release-readiness
plan back to `docs/plans/`, remove `docs/v0.21-roadmap.md`, remove this plan,
and restore the previous current-plan entry.

## Artifacts and Notes

Do not paste live command output or raw bars into this plan. If later live
smoke is useful, record only public-safe summary fields such as symbol,
timeframe, requested range, returned count, coverage status, and source
availability.
