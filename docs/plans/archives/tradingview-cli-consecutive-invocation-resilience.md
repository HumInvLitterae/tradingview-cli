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
- [x] (2026-07-19) Started after the indicator-search reassessment received
  focused evidence review and was archived.
- [x] (2026-07-18) Obtained focused independent review of this plan with no
  implementation blocker.
- [x] (2026-07-19) Inventoried eligible commands and froze `readiness`, `ohlcv
  --summary --count 20`, and `values` as the light, summary, and inventory
  archetypes.
- [x] (2026-07-19) Added deterministic aggregate fixtures and an ignored
  production-subprocess live harness without production behavior changes.
- [x] (2026-07-19) Completed formatting, strict Clippy, focused fixtures, full
  workspace tests, metadata, hygiene, package-script syntax, guide parity, and
  diff checks.
- [x] (2026-07-19) Obtained focused implementation review with no blocker.
  Corrected the public stage allowlist and added the recommended ambiguity,
  child-timeout, and deadline fixtures.
- [x] (2026-07-19) Obtained focused correction re-review with no finding. The
  bounded 120-invocation live matrix now awaits separate owner authorization.
- [x] (2026-07-19) Executed the owner-approved stable-target matrix once without
  optional target-set mutation. Four cohorts completed; two stopped on one
  invocation timeout each. No compensating invocation or retry was run.
- [x] (2026-07-19) Recorded a preliminary classification: repeated explicit
  reads were stable, heuristic ambiguity was expected with four chart targets,
  and the two unclassified process timeouts require evidence review rather than
  automatic retry or a production change.
- [x] (2026-07-19) Obtained focused evidence review with no finding. The review
  confirmed no retry/session/broker promotion, routed slow-tail attribution to
  a future narrow measurement candidate, and approved archive before release
  readiness.
- [x] (2026-07-19) Archived this plan without rerunning the live matrix.

## Surprises & Discoveries

- Observation: the existing 10-iteration measurement cannot answer this
  investigation.
  Evidence: it requires one explicit target ID and exercises typed transport
  boundaries directly; it does not represent heuristic selection, independent
  CLI process startup, command mixing, or prior-operation state.

- Observation: readiness and status expose target cardinality through different
  public data paths.
  Evidence: readiness uses `data.cdp.target_count`, while status uses
  `data.desktop_readiness.target_count`. The harness accepts both public shapes,
  retains only the numeric cardinality, and never retains a target identity.

- Observation: repeated explicit reads did not reproduce a transport or target
  selection failure, but the mixed explicit cohort stopped on one invocation
  timeout.
  Evidence: the explicit light and explicit large cohorts completed 20/20 each
  with no failure or ambiguity. The mixed explicit cohort completed 13/20 with
  13 successes, then the next fixed rotation entry exceeded the 12-second child
  deadline. No `failure_stage` was available because the harness terminated the
  child at its outer process boundary.

- Observation: heuristic behavior depended on each command's existing
  ambiguity contract rather than silently selecting among four chart targets.
  Evidence: heuristic readiness completed 20/20 as successful diagnostic
  envelopes while reporting ambiguity 20 times. Heuristic values and the
  corresponding entries in the mixed cohort returned `target_select` failures;
  one values invocation also exceeded the child deadline. No target-cardinality
  drift occurred during the run.

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

- Decision: use `readiness`, `ohlcv --summary --count 20`, and `values` as the
  three read archetypes.
  Rationale: source and CLI help identify all three as Desktop-backed reads.
  `readiness` performs bounded readiness inspection, OHLCV summary reads current
  bars and summarizes them without changing range or chart state, and `values`
  reads the current study inventory. None dispatches input or changes symbol,
  timeframe, viewport, target lifecycle, or account state.
  Date/Author: 2026-07-19 / Codex

- Decision: map unknown future failure stages to `transport_unknown` inside the
  aggregate harness.
  Rationale: retaining an unreviewed stage string would widen the evidence
  surface. The fixed known vocabulary remains useful while an unknown value
  stops short of exposing raw details.
  Date/Author: 2026-07-19 / Codex

- Decision: do not promote pre-dispatch retry from this run.
  Rationale: the 24 classified failures were expected `target_select` outcomes
  under a deliberately ambiguous four-chart environment, not transient target
  listing or WebSocket failures. The two process timeouts had no stage evidence
  and therefore cannot safely authorize a transport retry.
  Date/Author: 2026-07-19 / Codex

- Decision: route the timeout evidence to focused review of command latency and
  Desktop state ownership before deciding whether another plan is warranted.
  Rationale: one timeout occurred in the fixed heuristic values cohort and one
  at the next entry of the explicit mixed rotation. Repeating or compensating
  those invocations would violate the predeclared matrix. The existing evidence
  can determine whether to defer, create a narrower measurement plan, or adjust
  harness guidance without changing production here.
  Date/Author: 2026-07-19 / Codex

## Outcomes & Retrospective

The owner-approved stable-target matrix completed 104 of 120 requested
invocations across four fully completed cohorts and two timeout-stopped cohorts.
It recorded 80 successes, 24 failures, 24 `target_select` stages, 51 ambiguity
observations, two deadline stops, and zero target-cardinality drift. Whole-run
latency was 36 ms at p50 and 7,581 ms at p95.

The evidence does not justify retry. Explicit repeated readiness and values
reads completed 40/40 without failure. Heuristic readiness correctly returned
successful diagnostic ambiguity, while heuristic chart-dependent reads refused
to select silently. The two child timeouts remain the only unexpected evidence:
one interrupted repeated heuristic values and one interrupted the explicit
mixed rotation. Because a harness-enforced child timeout has no operation
`failure_stage`, the safe outcome is focused evidence review and defer-or-narrow
measurement routing, not a production change in this plan.

Focused evidence review independently reconstructed all aggregate counters from
the fixed rotations and confirmed the result. The recorded zero target drift is
limited to total target cardinality sampled by readiness-bearing invocations;
it does not prove that the chart-target subset was unchanged when total
cardinality stayed constant.

The only promoted follow-up is a future narrow chart-read latency attribution
measurement candidate. It should separate operation-layer phases for OHLCV and
study-value reads before changing timeout or wait policy. This plan adds no
production timing contract and requires no completion-audit refresh because it
changed no production behavior.

## Context and Orientation

Every `tv` invocation is a separate process. `crates/cli/src/app/dispatch.rs`
routes commands, `crates/cli/src/app/runtime.rs` creates the selected Desktop
runtime, and `tradingview-cdp` owns target listing, selection, connection, and
method/event waits. Existing public failures may include `failure_stage`.

Use only commands confirmed by source and help to be read-only and not to
change chart symbol, timeframe, viewport, replay, Pine, drawings, layout,
tabs, Screener state, or account data. Candidate families include `readiness`,
`status`, selected-chart values, selected-chart OHLCV summary, and read-only
data inventories. This inventory selected `tv readiness`, `tv ohlcv --summary
--count 20`, and `tv values`. Explicit cohorts prepend `--target-id <ID>`;
heuristic cohorts use the same vectors without that option.

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
    cohort_summaries

`cohort_summaries` contains the same count, stage, ambiguity, deadline,
target-drift, and latency fields for each of the six fixed public cohort labels.
It contains no command output or target identity. This per-cohort breakdown is
required to distinguish explicit from heuristic selection and repeated from
mixed reads; the top-level fields remain the whole-run totals.

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

The implementation is
`crates/cli/tests/live_consecutive_invocation_resilience.rs`. Its ordinary test
run executes deterministic aggregate, malformed-output, allowlist, and exact
matrix fixtures while leaving the live matrix ignored. The live gate is
`TV_LIVE_CONSECUTIVE_INVOCATION_RESILIENCE=1`, and the explicit target is read
from `TV_LIVE_CONSECUTIVE_INVOCATION_TARGET_ID` without being printed or stored.

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

Revision note (2026-07-19): started after the reviewed indicator reassessment,
froze the three source-confirmed read vectors, and added the ignored 120-process
harness plus deterministic aggregate-only fixtures. No live cohort, retry, or
production behavior was added.

Revision note (2026-07-19): after focused implementation review, replaced the
nonexistent `http_client` aggregate stage with the shipped `event_wait` stage
and added deterministic fixtures for all three ambiguity envelope shapes, a
blocked production child timeout, and cohort/run deadline stops. Matrix bounds
and live authorization remain unchanged.

Revision note (2026-07-19): focused correction re-review confirmed the shipped
stage vocabulary, all promised deterministic fixtures, unchanged matrix bounds,
and the separate owner-authorization gate. No live invocation was run.

Revision note (2026-07-19): recorded the single owner-approved stable-target
run. It completed 104/120 invocations, reproduced expected multi-target
ambiguity and `target_select` behavior, and stopped two cohorts on one child
timeout each. No optional target-set mutation, retry, or compensating run was
performed; focused evidence review is pending.

Focused evidence review is now complete and green. The reviewed evidence does
not promote retry, a shared connection, or production behavior; it routes the
unattributed slow tail to a separate future measurement candidate.

Revision note (2026-07-19): focused evidence review found no blocker, confirmed
the aggregate reconstruction and no-retry decision, limited the meaning of
zero target drift, and routed slow-tail attribution to a future narrow
measurement candidate. The plan is archived without another live run.
