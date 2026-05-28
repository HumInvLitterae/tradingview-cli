# Replay step-log contract

This ExecPlan is a living document. The sections `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to
date as work proceeds. This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

The `tv replay ...` commands can already read and control TradingView Replay,
but Replay changes the selected TradingView Desktop chart state. Before adding
any stable Replay extraction command, the project needs a clear contract for a
bounded step log: a public-safe record of Replay state before and after each
step. After the follow-up implementation, a user should be able to ask for a
bounded Replay step workflow and receive evidence that says what Replay did,
where it started and ended, and why it stopped, without confusing that output
with Desktop-free historical bars from `tv bars --from/--to`.

This plan does not implement a new command. It records the source boundary,
candidate interface, and acceptance criteria for the next implementation slice.

## Progress

- [x] (2026-05-28) Create this ExecPlan.
- [x] (2026-05-28) Archive the completed selected-chart export command plan.
- [x] (2026-05-28) Update the v0.23 roadmap, plan index, changelog, docs, and
  runtime skills for Replay step-log planning.
- [ ] Implement the bounded Replay step-log workflow in a later slice.

## Surprises & Discoveries

- Observation: existing Replay commands already expose the core state needed
  for a future step log.
  Evidence: `tv replay status` normalizes `replay_context` and optional
  `chart_context`; `tv replay step` preserves `previous_date` and
  `current_date`.

## Decision Log

- Decision: Plan Replay step logging before adding any stable Replay export.
  Rationale: Replay is stateful and changes the selected chart. A step log
  contract lets the project define start state, stop behavior, partial runs,
  and failure details before exposing a broader extraction surface.
  Date/Author: 2026-05-28 / Codex.

- Decision: Treat Replay step logging as Desktop-backed selected-chart
  workflow evidence, not as historical bars source preparation.
  Rationale: `tv bars --from/--to` is the reproducible Desktop-free historical
  bars source. Replay depends on the user's current Desktop chart, Replay mode,
  visible state, and UI availability.
  Date/Author: 2026-05-28 / Codex.

- Decision: Do not attach OHLCV summaries or screenshots automatically in the
  first step-log implementation.
  Rationale: adding chart bars or screenshots would mix evidence surfaces. If
  a later command supports attachments, they must be explicit options with
  source metadata and separate failure details.
  Date/Author: 2026-05-28 / Codex.

## Outcomes & Retrospective

This plan establishes the next `v0.23.0` slice after `tv export chart-bars`.
No runtime behavior has changed yet. The next implementation should add a
bounded Replay step-log workflow only after preserving the boundaries recorded
here.

## Context and Orientation

The repository builds a Rust CLI named `tv`. Desktop-backed commands connect
to the local TradingView Desktop app through Chrome DevTools Protocol. Replay
commands live under `crates/cli/src/ops/replay/`, with validation helpers in
`crates/model/src/replay.rs`.

Replay means TradingView's chart replay mode. In this repository,
`tv replay status` is a read: it inspects Replay state and should report
`source_category: "desktop_backed_read"` and `non_mutating: true`. The commands
`tv replay start`, `tv replay step`, `tv replay stop`, `tv replay autoplay`,
and `tv replay trade` are operations: they can change Replay state or Replay
trade state and should report `source_category: "desktop_backed_operation"`
and `non_mutating: false`.

A step log is a bounded record of a Replay run. Bounded means it has an
explicit stopping rule such as a maximum number of steps, duration, or failure.
It is not a daemon, not an unbounded watch loop, and not a trading
recommendation. It is also not a replacement for `tv bars --from/--to`, which
remains the Desktop-free historical bars source.

## Plan of Work

The next implementation should add one narrow Replay step-log surface. The
preferred command name is `tv replay log`; `tv replay extract-log` remains the
fallback name only if `log` conflicts with clap structure or existing help
wording. The command should require an explicit bounded control such as
`--steps <N>`, with a small default only if the implementation also makes the
default visible in readiness output.

The output should be a machine-readable log of Replay state transitions. The
minimum useful fields are `contract_version`, `step_index`, `operation`,
`previous_date`, `current_date`, `replay_context`, `chart_context`,
`end_reason`, and `failure_details` when a step fails. The command should
record initial `tv replay status`-style state before the first step and a final
summary after the bounded run.

The command must use only Replay APIs and selected-chart state already used by
the existing Replay commands. It must not call `tv bars`, `tv export
chart-bars`, scanner reads, quote-data, screenshots, or `tv ohlcv` as hidden
fallbacks. If a later version adds `--with-ohlcv-summary` or `--with-screenshot`,
those must be explicit options and must preserve their own source metadata.

Failure output should stay public-safe. Replay unavailable, Replay not
started, missing methods, step timeout, and interrupted bounded runs should be
source diagnostics. They must not expose raw DOM, raw payloads, target ids,
account-local metadata, credentials, or local paths.

## Concrete Steps

Run all commands from the repository root.

First inspect current Replay behavior:

    rg -n "ReplayCommand|replay_status|replay_step|replay_context" crates/cli/src crates/model/src
    cargo test -p tradingview-cli ops::replay -- --nocapture

Then implement the future slice by adding the command surface, validation,
operation loop, event or payload shaping, and tests. Keep the implementation in
the CLI layer because it orchestrates selected-chart state. Reuse existing
Replay normalization helpers from `crates/model/src/replay.rs` where possible.

For this planning slice, validate documentation only:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg -n "v0\\.23|Replay|step log|replay_context|tv replay|selected-chart export|tv export chart-bars|tv bars|source mixing|ranking|recommendation" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|raw JSONL|raw bars|account-local|target id|downstream-private)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true

## Validation and Acceptance

This planning slice is accepted when the active plan index points to this file,
the v0.23 roadmap identifies Replay step-log contract as the current slice, and
the user-facing docs and runtime skills describe Replay step logging as a
future bounded Desktop-backed workflow rather than a stable historical bars
source.

The later implementation slice is accepted when a user can run a bounded Replay
step workflow, observe initial state, per-step state, and final summary, and
see that the output records why the run stopped. A normal run should stop
because the step bound was reached. A failure run should stop with a structured
source diagnostic rather than raw TradingView payloads.

## Idempotence and Recovery

This planning slice is docs-only and safe to repeat. If the plan index or
roadmap already mentions this plan, update the wording rather than adding a
duplicate entry.

The future implementation is stateful because Replay operations change the
selected chart. A failed run should leave enough readback for the user or agent
to decide whether to run `tv replay status` or `tv replay stop`. It must not
silently switch to another source to recover.

## Artifacts and Notes

Do not paste live Replay output into tracked docs. If optional smoke is run in
a later slice, record only public-safe summary fields such as command, symbol,
timeframe, Replay started state, current date, step count, stop state, and
contract marker.

## Interfaces and Dependencies

No new dependency is planned. The future implementation should use the
existing Replay modules in `crates/cli/src/ops/replay/` and the normalization
helpers in `crates/model/src/replay.rs`.

The preferred future command-local contract marker is:

    contract_version: "replay_step_log.v1"

The minimum future step entry should include:

    step_index
    operation
    previous_date
    current_date
    replay_context
    chart_context

The minimum final summary should include:

    step_count
    end_reason
    started_at_replay_date
    ended_at_replay_date
    failure_count

The initial end-reason vocabulary should be small: `step_limit_reached`,
`replay_not_started`, `replay_unavailable`, `step_failed`, and `completed`.

## Open Questions

None for this planning slice. The future implementation may choose JSONL or a
single JSON payload, but it must make that choice explicitly in the
implementation ExecPlan before code changes begin.

## Change Note

Created on 2026-05-28 to move `v0.23.0` from selected-chart export into Replay
step-log contract planning while preserving the boundary between Replay state
operations and Desktop-free historical bars.
