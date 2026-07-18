# Measure consecutive read-only CLI invocation resilience

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while work proceeds.
Maintain it according to `.agents/PLANS.md`.

## Purpose / Big Picture

This investigation measures whether ordinary repeated use of Desktop-backed
read commands is stable without requiring an agent to perform a separate state
check before every invocation. It expands beyond the earlier explicit-target
transport probe by exercising production CLI subprocesses, heuristic and
explicit target selection, repeated and mixed reads, and short fixed intervals.

The outcome is evidence and routing: improve CLI transport/selection/state
ownership only when failures point there; otherwise improve the agent harness
workflow. This plan does not add retry, change production commands, or test
mutation-command repetition.

## Progress

- [x] (2026-07-18) Defined the investigation boundary and responsibility split.
- [x] (2026-07-18) Created this queued investigation plan.
- [ ] Start after the indicator-search reassessment closes.
- [ ] Obtain focused independent review of this plan.
- [ ] Inventory eligible read-only commands and freeze the exact matrix.
- [ ] Add a deterministic subprocess fixture and aggregate-only ignored live
  harness without production behavior changes.
- [ ] Execute the owner-approved bounded matrix.
- [ ] Classify evidence and record promote/defer/no-change decisions.
- [ ] Obtain focused evidence review and archive this plan.

## Surprises & Discoveries

- Observation: the existing 10-iteration measurement cannot answer this
  investigation.
  Evidence: it requires one explicit target ID and exercises typed transport
  boundaries directly; it does not represent heuristic selection, independent
  CLI process startup, command mixing, or prior-operation state.

## Decision Log

- Decision: begin with read-only commands only.
  Rationale: mutation repetition introduces identity, postcondition,
  restoration, and unknown-outcome contracts that cannot be inferred from read
  stability or made safe through automatic retry.
  Date/Author: 2026-07-18 / Codex

- Decision: use production CLI subprocesses and parse normal envelopes.
  Rationale: direct transport helpers omit process startup, dispatch, target
  selection, and command-specific ownership boundaries that agents experience.
  Date/Author: 2026-07-18 / Codex

- Decision: do not create or close targets automatically.
  Rationale: target-set mutation is a separate lifecycle operation. Compare a
  stable baseline with an owner-prepared changed target set, each recorded only
  by aggregate cardinality and ambiguity status.
  Date/Author: 2026-07-18 / Codex

## Outcomes & Retrospective

Not yet executed. Record whether failures cluster by transport stage, target
selection mode, command mix, interval, or target-set state, and whether the
correct next owner is CLI production code or the agent harness.

## Context and Orientation

Every `tv` invocation is a separate process. `crates/cli/src/app/dispatch.rs`
routes commands, `crates/cli/src/app/runtime.rs` creates the selected Desktop
runtime, and `tradingview-cdp` owns target listing, selection, connection, and
method/event waits. Existing public failures may include `failure_stage`.

Use only commands confirmed by source and help to be read-only and not to
change chart symbol, timeframe, viewport, replay, Pine, drawings, layout,
tabs, Screener state, or account data. Candidate families include `readiness`,
`status`, selected-chart values, selected-chart OHLCV summary, and read-only
data inventories. Milestone 1 must record the exact command vector and explain
why each is read-only before live execution.

An explicit-selection trial passes one target ID supplied through the existing
CLI option. A heuristic-selection trial omits it and therefore exercises the
normal selection and ambiguity rules. Target drift means the aggregate target
set or selected target evidence changed between invocations; raw IDs are never
recorded.

## Plan of Work

First, audit candidate commands and choose three archetypes: one light
readiness/status command, one selected-chart scalar or summary read, and one
larger selected-chart inventory read. Exclude any operation with temporary or
persistent mutation. Freeze exact vectors and prove validation, source, and
target behavior in deterministic subprocess fixtures.

Second, implement an ignored test harness or repository-local development
script that launches the built production binary. It must accept an explicit
target through an environment variable without printing it, parse one JSON
envelope per invocation, discard data payloads, and retain only allowlisted
aggregate status. Use one absolute deadline for each cohort and one for the
whole run; a timeout never restarts the cohort.

Third, run these stable-target cohorts with 20 invocations each:

- same light read, explicit target, no added delay;
- same light read, heuristic selection, no added delay;
- same larger read, explicit target, fixed 250 ms interval;
- same larger read, heuristic selection, fixed 250 ms interval;
- deterministic mixed rotation across all three reads, explicit target,
  fixed 250 ms interval;
- the same mixed rotation with heuristic selection.

This is 120 invocations. If a cohort stops on its absolute deadline, record the
completed count and do not compensate with extra invocations.

Fourth, optionally repeat only the two heuristic cohorts after the owner has
manually changed the target set. Record cardinality before and after, not IDs.
Do not automatically create, close, activate, or switch targets. If the changed
set is ambiguous by design, successful silent selection is a defect; the
expected result is the existing ambiguity error.

Finally, classify evidence. `target_list` failures may justify a bounded
pre-dispatch retry plan; `websocket_connect` failures may justify one stale
endpoint refresh; heuristic-only failures point to selection; mixed-only
failures point to command/Desktop state ownership. No reproduced CLI failure
points toward harness-side prechecks and clearer operation sequencing. No
finding directly authorizes a fix.

## Concrete Steps

Run from the repository root:

    target/debug/tv --help
    target/debug/tv readiness --help
    target/debug/tv status --help
    target/debug/tv ohlcv --help
    target/debug/tv values --help
    rg -n "Command::(Readiness|Status|Values|Ohlcv)|connect_runtime" crates/cli/src
    rg -n "failure_stage|target_select|websocket_connect" crates/cdp crates/cli/tests

The harness must first pass deterministic fixtures for success parsing,
allowlisted failure-stage counting, malformed output, child timeout, ambiguity,
deadline stop, and private-value rejection. The owner-approved live run emits
only one final aggregate object with:

    cohorts_requested
    cohorts_completed
    invocations_requested
    invocations_completed
    success_count
    failure_count
    failure_stage_counts
    ambiguity_count
    deadline_stop_count
    target_drift_count
    latency_p50_ms
    latency_p95_ms

Run focused fixtures, then the complete baseline:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    git diff --check

## Validation and Acceptance

Acceptance requires deterministic proof that the harness uses production CLI
subprocesses, bounded cohorts, normal explicit/heuristic selection, and
aggregate-only output. Requested/completed and success/failure counts must be
internally consistent. Raw command payloads, target IDs, URLs, titles, symbols,
study identities, and account metadata must not enter retained evidence.

The investigation closes with per-cohort evidence and one decision for each
observed failure family: promote a separate narrowly scoped plan, defer pending
more evidence, or route the issue to harness workflow. Zero failures is not
proof of universal reliability and does not authorize retry.

## Idempotence and Recovery

Deterministic fixtures are repeatable. Live cohorts are read-only but require
TradingView Desktop and explicit owner approval because they repeatedly access
the active environment. Use finite counts and deadlines. On unknown outcome,
deadline, malformed output, or unexpected target drift, stop the affected
cohort; do not retry or mutate Desktop state.

Do not run mutation commands, create/close/activate targets, change symbols or
timeframes, apply/drop stashes, push, tag, or create a release.

## Artifacts and Notes

Retain only aggregate counters, fixed cohort labels, latency summaries, and
decision text. Do not retain raw JSON envelopes, command data, target IDs,
endpoint URLs, symbols, layout/account metadata, credentials, or local paths.

## Interfaces and Dependencies

This investigation adds no production interface or dependency. A test harness
may use existing Rust test support, `assert_cmd`, Tokio, and JSON parsing. It
must remain ignored for live execution and must not add a production command or
general-purpose orchestration layer.

## Open Questions

- UNCONFIRMED: which three current read commands provide the best light,
  summary, and inventory archetypes without any temporary mutation.
- UNCONFIRMED: whether failures reproduce only under heuristic selection,
  mixed reads, or changed target cardinality.
- UNCONFIRMED: whether observed issues belong to CLI state ownership or agent
  workflow sequencing.

Revision note (2026-07-18): created as a bounded post-audit investigation after
the owner raised repeated agent/CLI use as a distinct failure surface. The plan
uses production subprocesses, excludes mutations and retry, and routes evidence
to CLI or harness ownership rather than assuming either in advance.
