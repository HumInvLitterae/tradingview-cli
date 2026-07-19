# Measure renderer foreground transitions without shipping UI behavior

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while work proceeds.
Maintain it according to `.agents/PLANS.md`.

## Purpose / Big Picture

The v0.29 indicator-search reassessment showed that a visibly selected Desktop
app tab can expose an embedded chart document as hidden, throttle page timers,
and materialize dialog rows only after an external render stimulus. Continuous
screenshot capture changed focus and is not an acceptable readiness mechanism.

This plan determines whether an existing CDP transition can improve renderer
timer/readiness evidence and then restore the prior target without UI clicks.
It compares HTTP target activation and `Page.bringToFront` as separate
boundaries. The outcome is feasibility evidence only. It does not add indicator
search, a foreground command, a session, a broker, or automatic activation.

## Progress

- [x] (2026-07-19) Re-read the archived indicator-search visibility evidence
  and current CDP target, app-tab, HTTP activation, and method-call ownership.
- [x] (2026-07-19) Created this ExecPlan and synchronized the v0.30 roadmap,
  ordered inventory, plan index, changelog, and local ledger.
- [x] (2026-07-19) Focused plan review completed without a blocker. Tightened
  `no_observable_need` so a ready probe baseline prevents every transition.
- [x] (2026-07-19) Implemented test-only snapshots, collision-checked timer
  markers, shared transition orchestration, deterministic fixtures, and an
  ignored two-target harness without changing ordinary command behavior.
- [x] (2026-07-19) Completed focused tests (8 passed, 1 ignored), strict Clippy,
  the full workspace baseline, metadata, hygiene, package syntax, guide parity,
  and diff checks.
- [x] (2026-07-19) Focused implementation review completed without a blocker.
  Canonicalized target IDs before distinctness validation and made unknown
  restore outcomes remain incomplete before the narrow correction re-review.
- [x] (2026-07-19) Focused correction re-review confirmed both findings closed
  with 8 focused tests passed and the live harness still ignored.
- [x] (2026-07-19) Ran the owner-authorized matrix once. It stopped with one
  baseline `unknown_stop` before candidate transitions; no HTTP activation or
  `Page.bringToFront` call ran. Marker cleanup is unconfirmed and no automatic
  recovery action was taken.
- [x] (2026-07-19) One separately approved read-only recovery observation found
  the marker absent and both callbacks incomplete. It performed no cleanup or
  transition.
- [x] (2026-07-19) Corrected the timer-trial budget so the fixed two-second
  observation window leaves one second inside the three-second trial for cleanup
  and verification. Focused tests pass 8 with 1 ignored and strict Clippy is
  green.
- [x] (2026-07-19) Focused correction review confirmed the deadline-budget
  defect closed with no new finding. Exact-pair rerun approval may be sought.
- [ ] Obtain focused evidence review, record go/defer/no-go, and archive.

## Milestones

### Milestone: freeze ownership and restoration

Separate Desktop app-tab state, HTTP target activation, DevTools
`Page.bringToFront`, and embedded document observations. Define exactly two
explicit targets: an originally active restore target and a distinct disposable
probe target. The milestone is complete when every transition has one matching
restoration and no target identity is inferred from app-tab order or title.

### Milestone: prove a bounded restoration-observed probe

Add test-only typed results and deterministic fakes for snapshots, one-shot
timer markers, transition calls, cleanup, restoration, deadlines, and aggregate
sanitization. The milestone is complete when every responsive failure performs
the matching restore call once and verifies only the declared target-side
observation, while an outer unknown-outcome timeout performs no automatic
second mutation.

### Milestone: collect one approved matrix

After implementation review and owner approval, compare baseline, HTTP
activation, and `Page.bringToFront` using only two disposable/restore targets.
Close with evidence review and a policy routing decision rather than a product
implementation.

## Surprises & Discoveries

- Observation: Desktop app-tab activation and CDP target activation are not the
  same proven operation.
  Evidence: `tab_list` reads app tabs from the app-window target, while
  `tab_switch` calls the HTTP target activation endpoint for a chart target.
  Archived evidence also states that app-tab order and CDP target order are not
  interchangeable.

- Observation: `Page.bringToFront` needs no new stable CDP API for feasibility.
  Evidence: `CdpClient::call_method` is public and can issue the exact method in
  a `#[cfg(test)]` harness; no ordinary command currently invokes it.

- Observation: `document.visibilityState` is not a sufficient acceptance
  signal.
  Evidence: the visibly foregrounded chart retained a hidden visibility state
  while the semantic launcher had rendered geometry. The probe therefore
  records visibility but judges timer completion and restoration separately.

## Decision Log

- Decision: require two explicit, distinct chart target IDs for any live run.
  Rationale: titles, chart IDs, app-tab indices, and heuristic selection cannot
  safely identify the restore target after a foreground transition.
  Date/Author: 2026-07-19 / Codex

- Decision: compare HTTP activation and `Page.bringToFront` independently.
  Rationale: combining them first would make any effect unattributable and
  would hide which ownership boundary requires product policy.
  Date/Author: 2026-07-19 / Codex

- Decision: use a reversible page-local timer marker instead of screenshots or
  dialog operations.
  Rationale: the prior screenshot stimulus changed focus, while indicator
  dialog manipulation would mix feature feasibility with shared renderer
  ownership. A fixed collision-checked marker can test `setTimeout(0)` and one
  `requestAnimationFrame` under Rust-side polling and can be removed exactly.
  Date/Author: 2026-07-19 / Codex

- Decision: do not infer success from document visibility or focus alone.
  Rationale: current Electron observations already show those labels can differ
  from rendered geometry. They are diagnostic dimensions, not go conditions.
  Date/Author: 2026-07-19 / Codex

## Outcomes & Retrospective

The test-only implementation runs both candidates through one injected
orchestration boundary. Deterministic tests cover the asymmetric ready-probe
stop, exact transition/restore order, responsive cleanup and restoration,
unknown-timeout no-restore precedence, expression anchors, aggregate wording,
and malformed/private snapshot rejection. Focused implementation and correction
review are green. The owner-gated live test remains ignored and unrun; exact
two-target owner approval was granted. The one authorized run stopped during
the probe baseline with `status: unknown_stop`, one unknown stop, zero baseline
responsive failures, and zero candidate results. No transition API ran. Marker
state was then observed once with separate approval: marker absent and both
callbacks incomplete. This revealed that the implementation incorrectly used
the whole three-second trial as its polling deadline instead of the planned
two-second observation window, leaving no cleanup budget. The harness correction
and focused review are the next gates.

## Context and Orientation

`crates/cli/src/ops/tab.rs::tab_list` reads chart targets and app-window tabs.
`tab_switch` uses `CdpHttpSession::activate_target`, which sends the existing
HTTP activation request. `crates/cdp/src/client.rs::CdpClient::call_method`
owns arbitrary CDP method request/response under the existing absolute method
deadline and public-safe `method_call` failure stage.

The archived indicator reassessment is
`docs/plans/archives/tradingview-cli-indicator-search-current-build-reassessment.md`.
Its evidence established hidden document state, throttled page timers, reliable
query assignment/restoration, missing ordinary result materialization, and
screenshot interference. It did not establish that any foreground transition
is safe or sufficient.

An observation snapshot contains only booleans and finite counts: document
visibility category, hidden, focus, positive viewport dimensions, marker
presence, `setTimeout` completion, and `requestAnimationFrame` completion. It
never returns title, URL, DOM text, target ID, geometry coordinates, or a raw
Runtime result.

The fixed marker property is
`window.__tvCliRendererForegroundProbeV1`. Snapshot fields are exactly
`visibility`, `hidden`, `has_focus`, `viewport_positive`, `marker_present`,
`timeout_completed`, and `animation_frame_completed`. No expression returns the
token object.

A responsive failure is a returned CDP/HTTP error or malformed snapshot for
which the harness remains in control. An unknown outcome is the outer harness
deadline expiring while a transition or restoration request is pending. A
responsive failure follows the one restoration sequence. An unknown outcome
stops immediately with no retry, poll, cleanup mutation, or second transition;
manual re-observation requires new owner approval.

The local stashes `fable-plan` and
`recovered-indicator-search-prototype-2026-07-12` remain preserved. Do not
apply, drop, rewrite, or edit them.

## Plan of Work

Add one `#[cfg(test)]` module under `crates/cli/src/ops/` and only test-only
crate-private re-exports needed to reuse `CdpHttpSession`, `CdpClient`, and
current connection ownership. Do not add a public command, dependency, generic
foreground helper, or production fallback.

Freeze these page expressions before live execution:

1. `snapshot` reads fixed allowlisted state. Visibility is normalized to
   `visible`, `hidden`, `prerender`, or `unknown`; dimensions become one
   `viewport_positive` boolean.
2. `install_marker` first requires the fixed probe property to be absent. It
   installs one token object, schedules `setTimeout(..., 0)` and one
   `requestAnimationFrame`, and each callback updates only when the same token
   is still active and installed.
3. Rust polls `snapshot` every 100 ms for at most two seconds. Page timers never
   own the observation deadline.
4. `cleanup_marker` sets the retained token inactive, deletes the fixed
   property, and returns only `marker_absent`. A callback checks identity and
   active state, so it cannot recreate the deleted property.
5. A final snapshot must confirm marker absence. Cleanup setter and verification
   getter each run exactly once on every responsive path after installation.

If the probe target baseline trial completes both callbacks within its own
bound, stop before any transition with `no_observable_need`, regardless of the
restore target baseline. The restore baseline is used only for later restore
comparison. A transition cannot prove an improvement when the candidate signal
is already ready.

Use two explicit, distinct chart targets named only in environment variables:
`TV_LIVE_RENDERER_RESTORE_TARGET_ID` and
`TV_LIVE_RENDERER_PROBE_TARGET_ID`. Validate both are non-empty and unequal
before HTTP or WebSocket access. The harness itself additionally requires
`TV_LIVE_RENDERER_FOREGROUND_FEASIBILITY=1` and remains `#[ignore]`.

Collect a baseline timer trial on each target without a transition. Then run
exactly these transition trials, in this order:

1. HTTP activation: call
   `CdpHttpSession::activate_target(probe_target_id)` exactly once, observe one
   probe-target timer trial, then call
   `CdpHttpSession::activate_target(restore_target_id)` exactly once and verify
   one restore-target snapshot and timer trial.
2. `Page.bringToFront`: on a connection to the probe target call
   `call_method("Page.bringToFront", json!({}))` exactly once, observe one
   probe-target timer trial, then on the retained restore-target connection call
   the same method exactly once and verify one restore-target snapshot and timer
   trial.

Do not try combined activation, app-tab DOM clicks, `Target.activateTarget`,
screenshot capture, focus/blur JavaScript, window APIs, alternate signatures,
or indicator dialog operations. A failed candidate does not fall back to the
other candidate; both are predeclared independent trials.

Each timer trial has one three-second absolute deadline covering installation,
Rust polling, cleanup, and verification. Each transition plus its probe and
restoration has one 12-second absolute deadline. The whole matrix has one
60-second absolute deadline. Nested work uses the earliest deadline and never
resets it. A responsive transition/probe failure attempts only the declared
restoration once. An outer unknown-outcome timeout stops the matrix without
automatic restoration because the remote transition outcome is unknown.

For each candidate, restoration evidence is limited to: the matching restore
API call returned success exactly once, the restore target's marker is absent,
and its allowlisted snapshot/timer result matches its own baseline categories.
Call this `restore_observation_matched`, not proof that the visible Desktop app
tab, OS focus, or window z-order was restored. If a later product requires those
stronger properties, it must first establish a separate machine-readable
readback or require an explicit user-visible transition policy.

The fixed aggregate schema contains candidate label, requested/completed,
transition call count, restoration call count, responsive failures, unknown
stops, `restore_observation_matched`, and before/after counts for visibility
categories, focus, positive viewport, timeout completion, and animation-frame
completion.
It may include existing allowlisted `failure_stage` counts. It contains no
target, title, URL, chart ID, symbol, DOM text, raw payload, exception, stack,
endpoint, account metadata, environment value, or machine path.

Go requires one candidate to improve a timer signal that was incomplete at its
own baseline, complete both callbacks under the same bound, and produce a
matching restore observation with verified marker cleanup. It does not claim
Desktop UI restoration. If baseline callbacks already complete for both
targets, record `no_observable_need` and do not call either transition. If
neither candidate qualifies, record no-go/defer. A go authorizes only a separate
product-policy ExecPlan deciding whether explicit foreground side effects and
the limited restoration evidence are acceptable.

## Concrete Steps

Run from the repository root. Reproduce current ownership:

    rg -n "activate_target|Page\\.|call_method\\(|app_tabs|visibilityState|hasFocus" crates/cdp/src crates/cli/src
    sed -n '1,280p' crates/cli/src/ops/tab.rs
    sed -n '1,340p' crates/cli/src/ops/desktop.rs
    sed -n '100,260p' crates/cdp/src/client.rs
    sed -n '1,240p' docs/plans/archives/tradingview-cli-indicator-search-current-build-reassessment.md

After implementation, run focused tests whose filters execute at least one
test, then:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    git diff --check

Do not run the ignored harness until focused implementation review is green and
the owner explicitly approves the exact two targets and transition matrix.

## Validation and Acceptance

Deterministic acceptance proves exact call counts, target distinction,
snapshot normalization, marker collision refusal, delayed callback observation,
callback inability to recreate cleaned state, responsive restoration,
unknown-timeout precedence, fixed deadlines, aggregate schema, and private-value
rejection.

Tests must also prove that a ready probe baseline produces zero transition and
restoration calls even when the restore baseline is incomplete, and that
aggregate wording never upgrades
`restore_observation_matched` into Desktop tab or OS-focus restoration.

Ordinary `tv tab` and Desktop-backed command behavior, target selection,
timeouts, JSON, source contracts, screenshot behavior, and process ownership
remain unchanged. No production call to `Page.bringToFront`, new automatic
activation, retry, session, broker, or indicator search may appear.

Live evidence is one bounded two-target observation. It cannot establish
cross-platform behavior or authorize foreground side effects for ordinary
commands. Windows evidence remains `UNCONFIRMED` unless separately collected.

## Idempotence and Recovery

Source inspection and deterministic tests are repeatable. Never rerun the live
matrix automatically. Before an owner-approved run, both markers must be absent
and both explicit targets must still resolve uniquely.

After a responsive failure, execute only the declared restoration once and
verify once. After an outer unknown-outcome timeout, execute no automatic
mutation; report the fixed unknown status and require owner-approved read-only
recovery observation. Never infer restoration from target order or title.

## Artifacts and Notes

Record deterministic test counts, candidate-level aggregate observations,
restoration result, and final policy routing. Never record either target ID,
tab title, chart ID, URL, symbol, DOM text, marker token, raw Runtime response,
exception, endpoint, account metadata, environment value, credential, or
machine path.

Prepare a self-contained read-only reviewer prompt after implementation. Do
not retain one-off reviewer instructions in tracked files.

## Interfaces and Dependencies

No production dependency or public interface is authorized. Reuse existing
`CdpClient::call_method`, `CdpHttpSession::activate_target`,
`TransportConfig::from_env_with_target_id`, and `RuntimeEvaluator::evaluate`
inside test-only code. Do not add a production foreground abstraction until
evidence review and a separate product-policy plan authorize it.

Create `crates/cli/src/ops/renderer_foreground_measurement.rs` and declare it
from `crates/cli/src/ops.rs` only as a `#[cfg(test)]` module. Keep the following
types private to that module: `ObservationSnapshot` for the seven fixed
snapshot fields, `TimerTrial` for one bounded marker lifecycle,
`TransitionCandidate` with only `HttpActivate` and `PageBringToFront`,
`CandidateResult` for one transition/restoration outcome, and `MatrixSummary`
for the aggregate allowlist. Implement one orchestration function that accepts
injected HTTP activation, CDP method-call, snapshot, and clock boundaries so
deterministic tests execute the same call ordering as the ignored live harness.
The injected boundaries are test-only traits or closures, not production APIs.

## Open Questions

- UNCONFIRMED: whether HTTP activation changes renderer timer behavior.
- UNCONFIRMED: whether `Page.bringToFront` changes renderer timer behavior or
  only DevTools target focus.
- UNCONFIRMED: whether either transition maps to the visible Desktop app tab.
- UNCONFIRMED: whether a useful effect can be restored without UI interaction.
- UNCONFIRMED: whether any go result should remain an explicit user action or
  can safely become an operation precondition.

Revision note (2026-07-19): created after chart-read attribution closeout from
the archived indicator-search renderer evidence. It separates HTTP activation,
DevTools foreground, and Desktop app-tab ownership; fixes exact marker cleanup,
deadline, restoration, and public-safe aggregate contracts before code changes.

Revision note (2026-07-19): after focused plan review, changed the early-stop
condition from both baselines ready to the probe baseline ready. A transition
cannot produce qualifying improvement when its own probe signal is already
complete, so the asymmetric case must not issue a live mutation.

Revision note (2026-07-19): implemented the reviewed test-only state machine.
Responsive marker failures now pass through one total cleanup-and-verification
boundary before candidate restoration, while cancellation by an outer timeout
performs neither cleanup nor restoration. Focused and full non-live validation
are green; live execution remains unapproved and unrun.

Revision note (2026-07-19): after focused implementation review, trimmed both
target IDs before non-empty and distinctness validation so whitespace variants
cannot resolve to the same target. Unknown restore outcomes now retain
`completed: 0`; the matrix and approval scope are unchanged.

Revision note (2026-07-19): focused correction re-review is green with no new
finding. The exact two-target live matrix may now be presented for separate
owner approval; no live transition has run.

Revision note (2026-07-19): the owner-approved matrix ran once and reached the
three-second outer bound during the initial probe-target baseline. It returned
one aggregate `unknown_stop` before either transition candidate, performed no
automatic cleanup or retry, and retained no target or raw payload. Marker state
remains `UNCONFIRMED` pending separately approved read-only recovery evidence.

Revision note (2026-07-19): the approved read-only recovery observation found
the marker absent with both callbacks incomplete. Source inspection then found
that Rust polling used the full three-second trial deadline rather than the
planned two-second observation window. The correction separates those bounds,
preserving one second for cleanup and final verification; no rerun is authorized.

Revision note (2026-07-19): focused correction review is green and confirmed
the two-second observation/three-second trial split preserves responsive cleanup
without changing hard-timeout precedence. The same exact target pair and matrix
may be presented for separate rerun approval; no rerun has occurred.
