# Measure and classify TradingView Desktop CDP transport behavior

This ExecPlan is a living document. The sections `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to
date as work proceeds. This document must be maintained in accordance with
`.agents/PLANS.md` at the repository root.

## Purpose / Big Picture

Desktop-backed `tv` commands can fail while listing targets, selecting a
target, opening a WebSocket, or waiting for a CDP response or event. Today the
user receives the existing error taxonomy, but the project cannot reliably
compare which transport stage failed or how much time each stage consumed.

After this plan, maintainers can run one bounded opt-in probe and see aggregate
public-safe stage latency and failure counts. Deterministic fixtures prove that
each stage and stale-target diagnosis is classified correctly under one
absolute deadline. Ordinary commands do not retry, reconnect, start a session,
or use a broker. Common success-envelope timing metadata is not added in this
slice.

## Progress

- [ ] Confirm the current transport and error-shaping call graph against the
  released `v0.28.0` source.
- [ ] Define internal typed stage, timing, and stale-target diagnosis models.
- [ ] Add deterministic stage timing and failure-classification fixtures.
- [ ] Add the bounded ignored live transport probe.
- [ ] Add the reviewed stable public `failure_stage` mapping to transport
  errors without changing existing kind, message, or exit code.
- [ ] Run one owner-approved bounded live probe and record aggregate evidence.
- [ ] Synchronize stable docs and development guidance for what actually ships.
- [ ] Run focused and full validation.
- [ ] Obtain focused independent implementation review and apply corrections.
- [ ] Archive this plan and promote or decline the next transport slice from
  evidence.

## Surprises & Discoveries

None yet. Record observations with concise deterministic or aggregate
public-safe evidence as work proceeds.

## Decision Log

- Decision: measurement precedes retry and reconnect behavior.
  Rationale: the v0.28 roadmap required a concrete Rust regression before CDP
  reconnect promotion, and current failure frequencies remain unconfirmed.
  Date/Author: 2026-07-17 / planning owner.
- Decision: keep internal diagnostics separate from the public JSON contract.
  Rationale: internal stages need enough precision for implementation and
  tests, while public fields require a smaller stable vocabulary and explicit
  non-leakage mapping.
  Date/Author: 2026-07-17 / planning owner.
- Decision: timing remains probe/internal evidence in this slice, while a
  small stable `failure_stage` field is added to transport error details.
  Rationale: the current shared envelopes have no metadata layer, and adding
  timing consistently to one-shot and JSONL workflows is a separate public
  contract decision. Failure stage is known at the error boundary and is
  needed to classify ordinary operational failures rather than probe-only
  failures.
  Date/Author: 2026-07-17 / planning owner.
- Decision: stale-target diagnosis may perform one post-failure read-only
  re-discovery only inside the probe/diagnostic path.
  Rationale: this measures detectability without converting diagnosis into
  retry behavior or changing an ordinary command result.
  Date/Author: 2026-07-17 / planning owner.

## Outcomes & Retrospective

Not started. At completion, record the observed stage distribution, whether a
stable public failure-stage field shipped, and which next slice was promoted or
declined.

## Context and Orientation

The repository is a virtual Cargo workspace. `crates/cdp/src/transport.rs`
owns the local CDP HTTP client and target discovery. `TransportConfig` contains
the host, port, and optional explicit target selection. `CdpHttpSession`
reuses one configured `reqwest::Client`, lists targets, and selects a target.
The current HTTP connect and total deadlines are one and three seconds.

`crates/cdp/src/client.rs` owns the WebSocket connection. `CdpClient::connect`
uses a five-second handshake timeout. Method-response and event waits use one
absolute deadline and preserve interleaved events in a FIFO bounded to 1024
events and 8 MiB. The client does not reconnect.

`crates/cli/src/app/runtime.rs` provides `connect_runtime()`, which discovers a
target and opens one `CdpClient`. Most Desktop-backed dispatch arms use this
helper. Some operations intentionally manage targets or connections directly;
this plan instruments transport ownership but does not restructure those
operations.

`crates/core/src/output.rs` defines `SuccessEnvelope` and `ErrorEnvelope`.
Neither has a shared metadata object. `AppError` carries `ErrorKind`, a message,
and optional details. Existing public-safe error shaping must remain intact.

In this plan, a transport stage is one bounded step: `target_list`,
`target_select`, `websocket_connect`, `method_call`, or `event_wait`. A stale-target
diagnosis is a probe-only classification after a WebSocket connection failure.
It compares a fresh read-only discovery result with the failed selection
without exposing either target or endpoint.

The public failure-stage vocabulary is: `target_list`, `target_select`,
`websocket_connect`, `method_call`, `event_wait`, and `transport_unknown`. The
public mapping is a separate serializable type from the internal observation
type. Unmapped internal stages use `transport_unknown`; they never serialize an
internal debug value.

The probe-only diagnosis vocabulary is: `unchanged`, `endpoint_changed`,
`selection_missing`, `selection_changed_or_ambiguous`, and `unavailable`.
These labels describe the bounded probe observation, not a claim about the
root cause or the failure rate of all normal commands.

## Plan of Work

### Milestone 1: typed internal transport observations

Inspect every error return in `crates/cdp/src/transport.rs` and
`crates/cdp/src/client.rs`. Add a small internal diagnostics module under
`crates/cdp/src/` containing a non-serializing `TransportStage`, a monotonic
duration sample type, and probe-only stale-target diagnosis types. Keep raw
targets and URLs outside these types. Record elapsed time with
`std::time::Instant`; use checked millisecond conversion for aggregate output.

Thread an optional observation sink or collector through the HTTP discovery,
WebSocket connect, and method/event wait boundaries without changing their
default behavior or public function results. The exact ownership style is
chosen during implementation, but it must not add global mutable state,
background tasks, or tracing output that can leak raw errors.

Add unit tests proving that each stage is recorded once, failures retain the
existing `ErrorKind` and message contract, durations are finite non-negative
integers, and no raw target or endpoint appears in the diagnostic structure.

At the end of this milestone, ordinary transport execution behaves exactly as
before and deterministic tests can inspect typed stage observations. The later
additive error-detail mapping is the only ordinary command contract change in
this plan.

### Milestone 2: deterministic failure and stale-target classification

Extend existing local HTTP and WebSocket fixtures. Cover successful listing,
selection, connection, method wait, and event wait. Inject connection refusal,
HTTP timeout, ambiguous selection, handshake timeout or controlled close,
method deadline, and event deadline. Assert the exact stage and preserve the
existing error classification.

Add a probe-only helper that, after a WebSocket connection failure, performs at
most one fresh target-list/selection observation under the same probe absolute
deadline. It must not reconnect or change the original result. Compare private
selection data in memory and return only one stale-target diagnosis label.

Fixtures must independently establish all five diagnosis outcomes. An explicit
target that resolves differently must never be relabeled as a successful
switch. Malformed or ambiguous fresh data produces
`selection_changed_or_ambiguous` or `unavailable`, not a guessed endpoint
change.

At the end of this milestone, classification is deterministic and no live
Desktop dependency is needed to prove it.

### Milestone 3: bounded public-safe live probe

Add `crates/cdp/tests/live_transport_measurement.rs` as an ignored test. It
requires `TV_LIVE_TRANSPORT_MEASUREMENT=1` and a non-empty
`TV_LIVE_TRANSPORT_MEASUREMENT_TARGET_ID` before opening a connection. Validate
`TV_LIVE_TRANSPORT_MEASUREMENT_ITERATIONS` with default 10 and range `1..=100`,
and `TV_LIVE_TRANSPORT_MEASUREMENT_DEADLINE_MS` with default 120000 and range
`1000..=300000`, before network access. The latter is the one absolute run
deadline. Do not infer an account or target.

Each iteration lists/selects the requested target, connects, and executes one
trivial read-only `Runtime.evaluate`. On connection failure, it may run the one
diagnostic re-discovery from Milestone 2. It never retries the failed operation.
Aggregate only iteration count, success/failure count, per-stage p50/p95
milliseconds when samples exist, failure counts by stage, and stale-target
diagnosis counts. Do not print raw JSON responses, exception text, target IDs,
URLs, symbols, bars, or account-local values.

The test must terminate under one outer deadline even if the endpoint stalls.
Add a non-live fixture path that executes the production aggregation and proves
empty and mixed samples, percentile selection, deadline termination, and
public-safe serialization.

At the end of this milestone, an owner can run one bounded read-only probe and
obtain evidence that is safe to summarize in this plan.

### Milestone 4: public failure-stage contract

Define a separate serializable public enum or fixed-string mapping for
`target_list`, `target_select`, `websocket_connect`, `method_call`,
`event_wait`, and `transport_unknown`; do not serialize the internal type
directly. Add `failure_stage` to transport error details in ordinary commands.
Preserve existing kind, message, exit code, and existing safe detail fields.
Prove unknown internal stages map to `transport_unknown` and private fixture
values do not appear. Do not add `--timing`, common envelope metadata, retry
counters, or recovery actions.

### Milestone 5: evidence, documentation, and closeout

After focused review of the probe, obtain separate owner approval for one live
run. Record only aggregate counts and latency summaries in Artifacts. A run
with zero failures is valid evidence and does not prove that transient failures
never occur. A variable live p50 or p95 is not a release blocker when the
deterministic timing and deadline fixtures are green.

Update `docs/architecture.md` and `docs/development.md` with the typed
diagnostic boundary, exact probe command, environment gate, and interpretation.
Update README, packaged guidance, or runtime skills only if a public error field
ships. Do not teach users or agents to retry until a later reviewed plan changes
behavior.

Run focused and full validation, obtain focused independent implementation
review, apply corrections, and then archive this plan. Use the evidence to
choose one next outcome: create a pre-dispatch retry ExecPlan, create a narrow
measurement follow-up, or record that no resilience behavior is currently
justified.

## Concrete Steps

All commands run from the repository root.

During Milestone 1 and 2, run focused CDP tests using the final module and test
names. At minimum:

    cargo test -p tradingview-cdp transport -- --nocapture
    cargo test -p tradingview-cdp client -- --nocapture

The ignored live test remains skipped in ordinary suites:

    cargo test -p tradingview-cdp --test live_transport_measurement -- --nocapture

Expect the test to be listed as ignored and no Desktop connection to occur.
After focused probe review and separate owner approval, run:

    TV_LIVE_TRANSPORT_MEASUREMENT=1 \
      TV_LIVE_TRANSPORT_MEASUREMENT_TARGET_ID=<TARGET_ID> \
      TV_LIVE_TRANSPORT_MEASUREMENT_ITERATIONS=10 \
      TV_LIVE_TRANSPORT_MEASUREMENT_DEADLINE_MS=120000 \
      cargo test -p tradingview-cdp --test live_transport_measurement -- --ignored --nocapture

Do not replace `<TARGET_ID>` in this tracked plan or retain the command's raw
output.

Run the project baseline after each completed implementation milestone:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    git diff --check

Interpret every command as successful only when it exits zero and the focused
tests execute at least one intended non-ignored test. The live test is an
additional owner-approved gate, not a replacement for deterministic tests.

## Validation and Acceptance

Acceptance requires all of the following:

- Deterministic fixtures classify target list, target selection, WebSocket
  connect, method wait, and event wait without changing existing error kinds or
  exit codes.
- One absolute deadline bounds the live probe and its optional diagnostic
  re-discovery.
- All five stale-target diagnosis outcomes have local fixture coverage.
- Ordinary commands perform zero new retry, reconnect, session, broker, or
  post-dispatch restart behavior.
- The live probe outputs only aggregate counts and timing summaries and remains
  ignored without its explicit gate and target selection.
- A live run with no failures is accepted as a bounded observation rather than
  an impossible requirement to demonstrate improvement.
- The additive `failure_stage` mapping is separately typed, uses only the six
  fixed public values, and is proven not to expose private transport values.
- Focused tests, the workspace baseline, public hygiene, docs consistency, and
  focused independent implementation review are green.

## Idempotence and Recovery

All deterministic tests and the live probe are read-only and may be rerun. The
live probe does not retry a failed operation or mutate TradingView state. If a
probe times out, stop and preserve only the aggregate evidence already held in
memory; do not start a recovery connection automatically.

Instrumentation is optional and must leave ordinary command behavior unchanged
when no collector is supplied. The additive public failure-stage field does not
enable timing or retry behavior. Each milestone is committed separately when
coherent and reviewed.

## Artifacts and Notes

Record the deterministic fixture matrix and the one bounded live summary here
as work proceeds. Store only stage labels, counts, durations, deadline status,
and the final go/no-go decision. State that the live probe uses explicit target
selection and therefore does not represent the latency or behavior of ordinary
heuristic target selection; deterministic fixtures cover that selection path.
Do not store raw output from a live target.

## Interfaces and Dependencies

No new production dependency is expected. Use `std::time::Instant` and existing
async deadline facilities. Do not use a transitive random-number crate or add
backoff logic in this plan.

The internal diagnostics module should provide equivalent concepts to:

    pub(crate) enum TransportStage {
        TargetList,
        TargetSelect,
        WebSocketConnect,
        MethodCall,
        EventWait,
    }

    pub(crate) struct StageSample {
        pub stage: TransportStage,
        pub elapsed: Duration,
        pub outcome: StageOutcome,
    }

    pub(crate) enum StageOutcome {
        Success,
        Failure,
    }

Exact names may change before implementation if the Decision Log records why,
but the types remain internal and non-serializing. If a public failure-stage
contract ships, define a separate serializable type and explicit conversion.

The public mapping should provide equivalent concepts to:

    #[serde(rename_all = "snake_case")]
    pub enum PublicFailureStage {
        TargetList,
        TargetSelect,
        WebsocketConnect,
        MethodCall,
        EventWait,
        TransportUnknown,
    }

The mapping helper merges `failure_stage` into an existing safe details object
or creates a new object when details are absent. It must not serialize internal
debug text or replace existing safe detail fields.

The live aggregation result contains exactly `iterations_requested`,
`iterations_completed`, `success_count`, `failure_count`, `deadline_reached`,
`stage_latency_ms`, `failure_stage_counts`, and
`stale_target_diagnosis_counts`. Each stage latency entry contains
`sample_count`, `p50`, and `p95`, with percentile values omitted or null when no
sample exists. Counts and durations are non-negative integers. The result must
not own or serialize `Target`, WebSocket URL, `AppError` details, or Runtime
response values.

## Open Questions

- Whether the internal collector is best passed explicitly, returned beside a
  result, or represented by a small optional observer trait is UNCONFIRMED. Do
  not use global mutable state.
- Whether any existing transport error needs the conservative
  `transport_unknown` mapping is UNCONFIRMED until Milestone 4.
- The frequency of each failure and stale-target diagnosis in the owner's
  environment is UNCONFIRMED until the bounded live run.
- Whether measurement justifies pre-dispatch retry is deliberately unresolved.

Revision note (2026-07-17): created this plan by splitting the measurement and
failure-taxonomy work from the former multi-phase CDP stability plan. Retry,
operation restart, topology optimization, recovery metadata, wait commands,
input preconditions, session mode, and broker feasibility now require separate
evidence and ExecPlans.
