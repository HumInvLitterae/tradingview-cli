# Attribute chart-read latency without changing command behavior

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while work proceeds.
Maintain it according to `.agents/PLANS.md`.

## Purpose / Big Picture

The bounded v0.29 consecutive-invocation run had a 36 ms median and a 7,581 ms
95th percentile across completed commands, while two chart-read subprocesses
reached the harness's 12-second child deadline. The outer harness could not say
whether time was spent in process startup, target discovery, WebSocket setup,
Runtime evaluation, operation normalization, or output.

This plan adds deterministic and opt-in measurement that attributes `ohlcv
--summary --count 20` and `values` separately. It does not change ordinary
command output, timeouts, retry, connection ownership, or source behavior. A
reviewed result may justify one later correction ExecPlan or a documented
defer. It does not directly authorize public `--timing`.

## Progress

- [x] (2026-07-19) Inspected current OHLCV, study-values, dispatch, and Runtime
  paths and corrected the earlier polling hypothesis.
- [x] (2026-07-19) Created this attribution ExecPlan and synchronized v0.30
  roadmap, inventory, plan index, changelog, strategy, and local ledger.
- [ ] Obtain focused independent review of the measurement contract.
- [x] (2026-07-19) Froze the exact phase ownership matrix: fresh runtime
  connection, one Runtime evaluation, enclosing operation, harness payload
  serialization, whole in-process trial, and two explicitly derived residuals.
- [ ] Implement internal/test-only observations and deterministic delayed and
  malformed boundaries without changing ordinary construction.
- [ ] Add an ignored, explicitly gated, aggregate-only live harness with fixed
  explicit-target OHLCV and values cohorts.
- [ ] Run focused tests, full baseline, metadata, hygiene, package syntax,
  guide parity, and diff checks.
- [ ] Obtain focused implementation review before seeking any live approval.
- [ ] If separately authorized, run one bounded live matrix and record only
  aggregate public-safe evidence.
- [ ] Obtain focused evidence review, decide promote/defer/no-go, and archive.

## Milestones

### Milestone: freeze the truthful phase model

Inventory where the current code can start and stop a clock without changing
semantics. Record which durations are direct observations and which can only be
derived. The milestone is complete when every proposed phase has one owner and
no overlapping values are represented as independent additive durations.

### Milestone: prove attribution deterministically

Add typed non-serialized observations and fake delayed boundaries that prove
phase labels, one absolute budget, saturation, malformed handling, and
aggregate safety. Ordinary commands must construct no active observer and emit
no timing field. This milestone is complete when deterministic tests fail if a
delay is assigned to the wrong owned phase.

### Milestone: collect bounded operational evidence

Add an ignored explicit-target matrix for separate OHLCV and values cohorts.
After focused implementation review and owner approval, one run may record
counts and p50/p95 phase summaries. The milestone closes with focused evidence
review and an explicit routing decision, not an automatic fix.

## Surprises & Discoveries

- Observation: neither target operation contains an explicit Rust or
  JavaScript polling loop.
  Evidence: `ohlcv_bars` in `crates/cli/src/ops/market/ohlcv.rs` executes one
  synchronous Runtime expression that reads bars; `study_values` in
  `crates/cli/src/ops/data/indicator.rs` executes one synchronous Runtime
  expression that reads existing studies.

- Observation: the existing CDP `method_call` stage combines request send and
  response wait.
  Evidence: the reviewed v0.29 recovery inventory deferred recovery mapping
  because `CdpClient::call_method` does not expose those as separate public
  ownership boundaries.

## Decision Log

- Decision: call the first slice latency attribution, not readiness or polling
  measurement.
  Rationale: the slow phase is unconfirmed and current operations contain no
  explicit polling loop.
  Date/Author: 2026-07-19 / Codex

- Decision: keep public `--timing` outside this plan.
  Rationale: a public contract is justified only after phase stability and an
  actionable consumer are demonstrated.
  Date/Author: 2026-07-19 / Codex

- Decision: compare operations in separate cohorts and require an explicit
  target.
  Rationale: mixed rotation and heuristic ambiguity obscured the v0.29 tail.
  Exact target handoff and separate cohorts isolate operation differences
  without weakening normal fail-closed selection.
  Date/Author: 2026-07-19 / Codex

## Outcomes & Retrospective

No implementation or live measurement has started. Record final phase
ownership, deterministic evidence, bounded live aggregate, and routing decision
here as milestones complete.

## Context and Orientation

`crates/cli/src/app/dispatch.rs` connects a Runtime client and dispatches both
commands. `crates/cli/src/app/runtime.rs::connect_runtime` performs one normal
CDP discovery and connection. `crates/cli/src/ops/market/ohlcv.rs::ohlcv_bars`
builds one JavaScript expression, calls `RuntimeEvaluator::evaluate`, validates
an operation-specific failure marker, and optionally summarizes the returned
bars in Rust. `crates/cli/src/ops/data/indicator.rs::study_values` similarly
runs one expression and normalizes study identity rows in Rust.

`crates/cdp/src/diagnostics.rs` and transport/client code own the internal
transport-stage vocabulary shipped in v0.29. The existing test observer is not
an ordinary command option. `crates/cdp/src/measurement.rs` and
`crates/cli/tests/live_consecutive_invocation_resilience.rs` are historical
measurement references; do not extend their conclusions by assumption.

An absolute deadline is one fixed end instant created before the bounded work.
Nested phases may observe remaining time but may not reset or extend it. A
phase duration is an elapsed value around one owned boundary. A derived value
is subtraction between directly observed enclosing and enclosed durations; it
must be labeled derived and clamped safely rather than presented as a direct
clock.

The local stashes `fable-plan` and
`recovered-indicator-search-prototype-2026-07-12` are preserved. Do not apply,
drop, rewrite, or edit them.

## Plan of Work

Use a `#[cfg(test)]` module at
`crates/cli/src/ops/latency_measurement.rs`, declared from
`crates/cli/src/ops.rs`. Do not broaden the visibility of the crate-private CDP
observer and do not add a production observer hook. The module owns a
`TimedRuntime<R>` wrapper around `RuntimeEvaluator`. It delegates every method
without changing arguments, return values, ordering, or errors, and records
only elapsed time around `evaluate`. A valid OHLCV or values trial must observe
exactly one evaluation; zero or multiple samples are `invalid_trial` and stop
the cohort.

The phase matrix is fixed as follows. All direct observations use the same
monotonic clock and convert elapsed values to saturating integer milliseconds.

- `connect_ms` directly surrounds one fresh `connect_runtime` call.
- `evaluate_ms` directly surrounds the operation's one
  `RuntimeEvaluator::evaluate` request and response. It includes the existing
  combined CDP method send/wait boundary and remote expression execution; this
  plan does not claim to separate them.
- `operation_ms` directly surrounds the real `ohlcv_summary` or `study_values`
  call, including its one evaluation and Rust-side validation/normalization.
- `payload_serialize_ms` directly surrounds `serde_json::to_vec` of the value
  returned by the operation. This is harness-payload serialization, not the
  ordinary CLI success envelope.
- `trial_ms` starts immediately before the fresh connection and stops after
  payload serialization. It is an in-process trial and excludes process
  startup.
- `normalization_residual_ms` is derived as
  `operation_ms.saturating_sub(evaluate_ms)`.
- `unattributed_residual_ms` is derived as
  `trial_ms.saturating_sub(connect_ms + operation_ms +
  payload_serialize_ms)`, using saturating arithmetic throughout.

The two residuals are always labeled derived. They are not independent clocks,
and scheduler overhead may remain in them. Never subtract the v0.29 subprocess
p50 or p95 from these in-process observations; that evidence is comparison
context only.

Implement the test-only trial runner with replaceable connector and serializer
closures plus a delayed fake `RuntimeEvaluator`. Deterministic fixtures inject
delays independently into connection, evaluation, operation-side
normalization, and serialization. They must prove the direct and derived phase
labels, exactly-one-evaluation invariant, one absolute deadline, bounded sample
counts, nearest-rank p50/p95, unknown/malformed failure classification, and
removal of raw values. A delay that crosses the deadline stops the cohort
without retry or replacement samples.

The ignored live harness requires both
`TV_LIVE_CHART_READ_LATENCY_ATTRIBUTION=1` and a non-empty
`TV_LIVE_CHART_READ_LATENCY_ATTRIBUTION_TARGET_ID`. Reject invalid cohort sizes
and deadlines before connection. Freeze two separate in-process cohorts,
OHLCV summary and values, with 20 trials each. Every trial creates a fresh
runtime through `connect_runtime`, wraps it in `TimedRuntime`, invokes the real
operation, and serializes the returned value. Do not substitute a
transport-only call or reuse a connection between trials. Use one 12-second
trial deadline, one 300-second cohort deadline, and one 600-second run deadline.
Stop a cohort on timeout, an invalid evaluation count, or malformed output and
never compensate with extra trials.

The aggregate allowlist contains operation label, requested/completed,
success/failure, existing `failure_stage`, deadline stops, invalid-trial stops,
direct p50/p95 for `connect_ms`, `evaluate_ms`, `operation_ms`,
`payload_serialize_ms`, and `trial_ms`, and derived p50/p95 for
`normalization_residual_ms` and `unattributed_residual_ms`. It contains no
target ID, URL, symbol, study name, bar data, study values, raw envelope,
exception, environment value, or local path.

After deterministic and full validation, obtain focused implementation review.
Live execution remains a separate owner-approved action. After evidence review,
record exactly one outcome: promote one named correction plan, defer with a
re-evaluation trigger, or close no-go. Do not change timeout, wait, retry,
session, broker, or public output in this plan.

## Concrete Steps

Run from the repository root. Reproduce source ownership:

    rg -n "connect_runtime|ohlcv_summary|ohlcv_bars|study_values|evaluate\(" crates/cli/src crates/cdp/src
    sed -n '1,230p' crates/cli/src/ops/market/ohlcv.rs
    sed -n '1,190p' crates/cli/src/ops/data/indicator.rs
    sed -n '1,160p' crates/cli/src/app/runtime.rs
    sed -n '1,260p' crates/cdp/src/diagnostics.rs

After implementation, run focused tests named by the final files. Every filter
must execute at least one test; revise this living plan if names differ. Then
run:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    git diff --check

Do not run the ignored live harness without separate owner approval. If
approved, use only the exact gate, target, cohort, and deadlines recorded in the
reviewed implementation.

## Validation and Acceptance

Acceptance before live evidence requires the fixed phase matrix above,
deterministic attribution tests, exact 2-by-20 cohort bounds, one absolute
budget per trial/cohort/run, public-safe aggregate serialization, and the
complete non-live baseline. Tests must prove that the wrapper observes exactly
one evaluation and that direct values and derived residuals cannot be confused.

Ordinary `tv ohlcv --summary --count 20` and `tv values` behavior, JSON, errors,
timeouts, discovery count, evaluation count, and source contracts must remain
unchanged. No public timing field, retry, reconnect, wait, session, broker, or
background work may appear.

Live evidence is supplementary and cannot prove a repository-wide latency
distribution. A quiet run is a valid defer result. A slow run promotes nothing
until focused review confirms phase ownership and reproducibility.

## Idempotence and Recovery

Source inspection and non-live tests are repeatable. The live harness is not
rerun automatically after timeout or failure. Do not reset, clean, stash,
apply, or drop unrelated work.

On malformed or unknown live outcome, retain only the allowlisted aggregate,
stop the cohort, and perform no retry or state mutation. If phase ownership
cannot be represented without changing production contracts, stop and revise
the plan before code changes.

## Artifacts and Notes

Record phase definitions, test counts, bounded aggregate counts, percentiles,
and the final routing decision. Never store raw payloads, bar values, study
values, symbols, target IDs, endpoints, account metadata, environment values,
credentials, or machine paths.

Prepare a self-contained read-only reviewer prompt after implementation. Do
not retain one-off reviewer instructions in tracked files.

## Interfaces and Dependencies

No production dependency or public interface is authorized. Add only the
`#[cfg(test)]` CLI operation module described above. Reuse `RuntimeEvaluator`,
`connect_runtime`, `ohlcv_summary`, `study_values`, and existing transport
diagnostics. Do not change CDP observer visibility, create an alternate source,
or add a fallback.

## Open Questions

- UNCONFIRMED: whether the slow tail belongs to Runtime method response,
  expression execution, Rust normalization, serialization, process scheduling,
  or an external Desktop condition.
- UNCONFIRMED: whether any stable actionable phase should become public after
  evidence review.

Revision note (2026-07-19): created after v0.29 release and review of its
consecutive-invocation evidence. Corrected the polling hypothesis from current
source, kept public timing conditional, and separated measurement from every
latency correction. The initial planning pass was then tightened to a
test-only `TimedRuntime` design with five direct durations, two explicitly
derived residuals, exact fresh-connection cohorts, and no CDP observer
visibility change.
