# `tv observe` / `tv stream` JSONL summary event

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It records
the additive JSONL summary-event slice for existing Desktop-backed observation
commands.

## Purpose / Big Picture

`v0.18.0` matures existing JSONL observation surfaces. The previous slice added
command-local contract metadata to readiness, sample, and heartbeat events.
This slice adds a final `_event: "summary"` line to bounded `tv observe chart`
and `tv stream ...` runs so downstream tools can read how the observation
window ended without aggregating the entire JSONL stream themselves.

This is not a new data source, watch loop, multi-symbol realtime feed, source
mixing, ranking, or recommendation feature. The summary event is an
observation-window readback, not a market-data sample.

## Progress

- [x] (2026-05-17T02:00Z) Create this ExecPlan and make it the current
  `v0.18.0` plan before release readiness.
- [x] (2026-05-17T02:10Z) Add `stream.v1` summary event construction and emit
  it after normal bounded stream loop exits.
- [x] (2026-05-17T02:10Z) Wrap stream summary payloads for `tv observe chart`
  as `observe_chart.v1` events with `_observe: "chart"`.
- [x] (2026-05-17T02:20Z) Update tests, help, docs, and runtime skills for the
  final summary event.
- [x] (2026-05-17T02:45Z) Run focused tests, full baseline, docs validation,
  skill validation, and hygiene checks.
- [x] (2026-05-17T02:45Z) Record final validation.
- [x] (2026-05-17T02:50Z) Commit the completed slice as
  `e5097e6 feat(cli): Add JSONL observation summary events`.

## Surprises & Discoveries

- Observation: The previous pre-release audit correctly validated metadata for
  readiness, sample, and heartbeat events, but downstream still had to
  aggregate JSONL lines to know the final bounded-window result.
  Evidence: `docs/plans/tradingview-cli-v0.18-pre-release-audit.md` did not
  define a summary event; this plan adds one as an additive follow-up before
  release readiness.

## Decision Log

- Decision: Add a final `summary` event instead of changing sample or heartbeat
  events.
  Rationale: The new line is additive, keeps existing event meaning stable, and
  lets downstream read counts and end reason without replaying event history.
  Date/Author: 2026-05-17 / Codex.

- Decision: Use the existing command-local contract versions:
  `stream.v1` for lower-level stream summary events and `observe_chart.v1` for
  observe summary events.
  Rationale: Summary is part of each command's event contract, not a new CLI
  envelope version.
  Date/Author: 2026-05-17 / Codex.

- Decision: Guarantee summary only for normal bounded loop exits in this slice.
  Rationale: Validation and connection failures already use structured error
  envelopes, and interrupted process termination needs a separate signal
  handling contract if it is ever required.
  Date/Author: 2026-05-17 / Codex.

## Outcomes & Retrospective

Implementation is complete. Bounded normal `tv stream ...` exits now emit a
final `stream.v1` summary event. Bounded normal `tv observe chart` exits now
emit an observe-wrapped `observe_chart.v1` summary event with `_observe:
"chart"` and underlying `_stream: "bars"` metadata.

The summary line reports `elapsed_ms`, `sample_count`, `heartbeat_count`,
`last_sample_ts`, `duration_ms`, `max_events`, and `end_reason`. Duration
limits produce `duration_elapsed`, and max sample limits produce
`max_events_reached`. Existing readiness, sample, heartbeat, validation error,
and connection failure behavior remains unchanged.

Docs and runtime skills now describe JSONL observation as readiness, sample /
heartbeat, then summary. Summary is documented as bounded observation-window
readback rather than a market-data sample.

## Context and Orientation

`tv stream ...` is a selected-chart Desktop-backed JSONL observation surface.
It emits sample events when the selected chart/page sample changes after
dedupe, and heartbeat events when a heartbeat interval passes without a new
sample.

`tv observe chart` is a workflow wrapper around selected-chart bars streaming.
It emits readiness first, then selected-chart bar sample or heartbeat events.
It does not switch symbols, activate tabs, capture screenshots, mutate chart
state, or read browserless historical bars.

Both surfaces now carry additive contract metadata. This slice adds the final
bounded-window readback line.

## Plan of Work

Add a `stream_summary` helper next to `stream_sample` and `stream_heartbeat`.
Track `sample_count`, `heartbeat_count`, `last_sample_ts`, elapsed time, and
the normal bounded end reason in stream and observe loops. Emit one summary
line after the loop ends normally.

For observe, reuse the stream summary payload and pass it through the existing
observe wrapper so it becomes an `observe_chart.v1` event with
`_observe: "chart"` while retaining underlying `_stream: "bars"` and selected
Desktop chart source metadata.

Update help, docs, runtime skills, and tests to treat the summary event as the
last bounded-window readback.

## Concrete Steps

Code changes:

1. Add `StreamEndReason` and `stream_summary(...)`.
2. Emit summary after normal bounded exits in `run_stream_command`.
3. Emit observe-wrapped summary after normal bounded exits in
   `run_observe_command`.
4. Update unit, contract, and live smoke tests for summary metadata and final
   event ordering.
5. Update README, source taxonomy, observation workflows, development docs,
   runtime skills, roadmap, and changelog.

Validation commands:

    cargo test -p tradingview-cli stream -- --nocapture
    cargo test -p tradingview-cli observe -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop -- --nocapture
    cargo test -p tradingview-cli --test live_observe_chart
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

If runtime skills changed, run the existing skill validator for those skills.

Validation completed:

    cargo test -p tradingview-cli stream -- --nocapture
    cargo test -p tradingview-cli observe -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop -- --nocapture
    cargo test -p tradingview-cli --test live_observe_chart
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh
    python3 <skill-creator>/scripts/quick_validate.py .agents/skills/chart-analysis
    python3 <skill-creator>/scripts/quick_validate.py .agents/skills/market-data-interpretation
    rg hygiene and JSONL/source-boundary scans

The hygiene scan reported existing policy language, archived validation
commands, fake example paths, and existing test examples. It did not identify a
new raw JSONL live output, raw DOM, raw WebSocket frame, target id,
account-local metadata, credential, or local validation path from this slice.

Optional live smoke:

    TV_LIVE_OBSERVE_CHART_SMOKE=1 cargo test -p tradingview-cli --test live_observe_chart -- --ignored --nocapture

Do not paste raw JSONL live output into tracked docs. Record only public-safe
event counts and source-boundary summaries.

## Validation and Acceptance

Acceptance is met when:

- `tv stream ...` bounded normal exits emit one final summary event with
  `contract_version: "stream.v1"`, `_event: "summary"`, `_stream`, source
  metadata, `elapsed_ms`, `sample_count`, `heartbeat_count`,
  `last_sample_ts`, `duration_ms`, `max_events`, and `end_reason`.
- `tv observe chart` bounded normal exits emit one final summary event with
  `contract_version: "observe_chart.v1"`, `_observe: "chart"`,
  `_event: "summary"`, `_stream: "bars"`, and the same summary counters.
- duration-limited exits report `end_reason: "duration_elapsed"`.
- max-event exits report `end_reason: "max_events_reached"`.
- validation and connection failures keep the existing error-envelope behavior
  and do not emit summary success events.
- sample and heartbeat event shapes remain additive and unchanged.
- docs and skills describe summary as an observation-window readback, not a
  market-data sample.

## Idempotence and Recovery

This is an additive implementation. If validation fails, fix only issues in
the summary-event implementation, tests, or docs. Do not introduce new
commands, options, sources, dependency changes, or version bumps.

If an unrelated working-tree change appears, inspect whether it affects this
slice before proceeding. Do not revert unrelated user work.

## Interfaces and Dependencies

No new dependency is planned. Public CLI command names and options stay the
same. The JSONL event contract is additive only.

## Open Questions

- Should interrupted process termination such as SIGINT ever guarantee a
  summary event? Deferred. This slice only guarantees summary for normal
  bounded exits.
