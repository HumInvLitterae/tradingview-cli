# Reassess current-build indicator search readiness

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while work proceeds.
Maintain it according to `.agents/PLANS.md`.

## Purpose / Big Picture

This investigation determines whether the current TradingView Desktop build
now exposes a repeatable, bounded, and restorable positive-result search path
for the Indicators dialog. The earlier trial proved a class-free structural
parser but deferred the command because result readiness was inconsistent
between already-open, freshly opened, and initially closed dialog states.

The outcome is evidence and a go/no-go decision, not a public command. A go
permits creation of a separate implementation ExecPlan. A no-go leaves the CLI
unchanged and records the exact missing current-build signal. The preserved
prototype stash is read-only research material and must not be applied.

## Progress

- [x] (2026-07-18) Re-read the three archived indicator-search plans and
  inspected the preserved prototype without applying it.
- [x] (2026-07-18) Confirmed that the prior defer decision was caused by
  inconsistent positive-result readiness, not parser impossibility.
- [x] (2026-07-18) Created this queued reassessment plan.
- [x] (2026-07-18) Started after the v0.29 completion audit received focused
  audit review with no finding and was archived.
- [x] (2026-07-18) Obtained focused independent review of this reassessment
  plan; the corrected maximum scope is 33 live trials.
- [x] (2026-07-18) Reconfirmed current dialog ownership and semantic anchors
  from current source and archived evidence without recording private result
  text or raw DOM.
- [x] (2026-07-18) Added an ignored, explicitly gated 33-trial harness with
  deterministic aggregate-contract fixtures; focused tests and strict Clippy
  are green.
- [x] (2026-07-18) Completed the full non-live workspace baseline, metadata,
  hygiene, package-script syntax, guide parity, and diff checks. Live execution
  was still unrun at this checkpoint.
- [x] (2026-07-18) Applied the focused-review recommendations: explicit
  assignment readback and `dispatch_failed` restoration for both candidates,
  plus a distinct `unstable_sampled` aggregate outcome. Focused re-review is
  complete and green. Focused and full non-live validation are green after
  correction.
- [x] (2026-07-18) Ran the owner-approved harness once. It stopped before the
  first preflight dispatch because the old launcher selector was absent and the
  Electron chart document reported hidden visibility; zero search trials
  completed.
- [x] (2026-07-18) Confirmed read-only that the current build exposes the
  semantic launcher as `open-indicators-dialog`. Foregrounding the visible
  Desktop tab did not change the embedded document's hidden visibility, while
  the launcher retained rendered geometry. Updated the harness to use that
  single selector and rendered geometry rather than document visibility.
- [x] (2026-07-19) Replaced page-timer polling with Rust-side 200 ms polling
  after confirming that Electron throttles timers in the embedded chart
  document. Kept the eight-second outer deadline and page-local signatures.
- [x] (2026-07-19) Completed the six-trial owner-approved dispatch preflight.
  Both candidates assigned all three fixed queries and all six restorations
  passed, but no result rows materialized during any five-second ordinary CLI
  observation window.
- [x] (2026-07-19) Confirmed visually and through aggregate DOM attributes that
  results do exist and expose current `data-title` row semantics after a render
  stimulus. Continuous screenshot stimulation interfered with dialog focus and
  is not an acceptable production readiness mechanism.
- [x] (2026-07-19) Recorded a readiness-specific defer. The selected-candidate
  27-trial matrix was not run because neither candidate qualified without
  external render stimulation.
- [x] (2026-07-19) Obtained focused evidence review with no finding. The review
  confirmed the readiness-specific defer, the six-trial evidence, and omission
  of the gated 27-trial matrix.
- [x] (2026-07-19) Archived this plan without applying or dropping the
  prototype stash.

## Surprises & Discoveries

- Observation: the preserved stash is incomplete implementation evidence, not
  a patch that can safely be restored wholesale.
  Evidence: it contains tracked CLI routing changes plus untracked model and
  adapter modules from the 2026-07-12 trial, based on old dialog assumptions.

- Observation: the old feasibility work reached limited go before the
  implementation trial was deferred.
  Evidence: three built-in queries produced one class-free structural host,
  positive query-sensitive rows, stable samples, and verified query
  restoration. Later fresh/reopened trials sometimes showed no host before the
  deadline even though screenshots eventually showed results.

- Observation: returning normalized rows to Rust is unnecessary for this
  reassessment and would widen the evidence surface.
  Evidence: the harness computes query matching and two-sample signatures
  inside the page, then returns only fixed statuses, counts, latency, and
  restoration state.

- Observation: the first owner-approved execution produced no search-readiness
  evidence because it stopped before query dispatch.
  Evidence: the current build no longer exposed the old dialog-launcher
  selector, and Electron reported the embedded chart document as hidden even
  while its Desktop tab was visibly foregrounded. A read-only aggregate
  inspection found the replacement semantic launcher with rendered geometry;
  no result title, target identifier, URL, or raw DOM was retained.

- Observation: Electron throttled page timers enough for the original
  page-side observation promise to overrun the eight-second outer deadline.
  Evidence: moving the same 200 ms cadence to Tokio completed all six bounded
  preflight trials while keeping signatures page-local and the outer deadline
  unchanged.

- Observation: query dispatch and parsing are not the current blockers.
  Evidence: prototype input events and native CDP text insertion each assigned
  all three fixed queries, and all six query restorations succeeded. A visual
  capture and accessibility inspection showed positive rows, while aggregate
  DOM inspection found one parent of query-matching `div[data-title]` rows.

- Observation: positive rows do not materialize reliably during an ordinary
  background CLI observation window.
  Evidence: all six production-like preflight trials returned `host_missing`,
  but a later CDP screenshot forced the rows into the DOM. Repeated screenshot
  stimulation changed dialog focus/preparation and therefore cannot be treated
  as a transparent readiness primitive.

- Observation: app-tab order and CDP target order are not interchangeable.
  Evidence: read-only per-target status identified the disposable active chart;
  rerunning on that exact target produced the same six `host_missing` outcomes.

## Decision Log

- Decision: investigate current readiness before reconsidering implementation.
  Rationale: increasing sleeps or restoring old code would reproduce the exact
  ambiguity that caused the defer decision.
  Date/Author: 2026-07-18 / Codex

- Decision: positive-result-only feasibility remains acceptable; successful
  empty results remain out of scope without an explicit semantic empty state.
  Rationale: zero parsed rows cannot distinguish no result, unresolved loading,
  missed dispatch, or DOM drift.
  Date/Author: 2026-07-18 / Codex

- Decision: do not apply, pop, drop, or edit the preserved stash.
  Rationale: current production has changed substantially and the stash mixes
  tracked routing with untracked prototype modules. Use it only to identify
  old hypotheses and missing tests.
  Date/Author: 2026-07-18 / Codex

- Decision: use fixed public built-in queries `RSI`, `MACD`, and `EMA`, with
  `SMA` as the different-prior-query baseline.
  Rationale: fixed query tokens make the matrix reproducible without retaining
  account-local result titles. They remain inside the ignored probe and are
  never emitted in aggregate evidence.
  Date/Author: 2026-07-18 / Codex

- Decision: keep row titles and stability signatures page-local.
  Rationale: the investigation needs only proof of positive query-sensitive
  stable rows. Returning titles or derived signatures adds no acceptance value
  and could expose account-local entries.
  Date/Author: 2026-07-18 / Codex

- Decision: treat query assignment as a separate stage before positive-result
  observation.
  Rationale: a failed assignment must not be misclassified as host readiness or
  stability timeout. Both dispatch candidates now require exact input-value
  readback; a known failure restores once and returns `dispatch_failed`.
  Date/Author: 2026-07-18 / Codex

- Decision: classify a sampled but never stable observation as
  `unstable_sampled`.
  Rationale: this distinguishes the prior readiness problem from a trial that
  never observed a structural host at all.
  Date/Author: 2026-07-18 / Codex

- Decision: require the current launcher to have rendered geometry, but do not
  use `document.visibilityState` as a Desktop-tab precondition.
  Rationale: the foregrounded Electron tab still reports its embedded document
  as hidden even though the launcher has a nonzero client rectangle. Geometry
  preserves a bounded rendered-element check without rejecting the actual
  visible Desktop target.
  Date/Author: 2026-07-18 / Codex

- Decision: defer production search after the
  six-trial preflight rather than running the 27-trial matrix.
  Rationale: the plan requires at least one candidate to pass all three
  preflight queries without an external visual operation. Both candidates
  assigned and restored correctly but produced three `host_missing` outcomes,
  so no candidate qualified. Screenshot-driven rendering is observable,
  interferes with focus, and is outside the intended search contract.
  Date/Author: 2026-07-19 / Codex

## Outcomes & Retrospective

The reassessment defers production indicator search because the current build
does not provide a stable background-CLI readiness boundary. Six of the maximum
33 trials were required and completed: both dispatch candidates assigned the
three fixed queries, all six restorations succeeded, and all six ordinary
observations reported `host_missing`. Because neither candidate qualified, the
plan's gate correctly prohibited the selected-candidate 27-trial matrix.

This is not a parser or permanent-capability no-go. Positive rows were visible
and the current `data-title` row boundary was class-free and query-sensitive
after rendering. The unresolved gap is making those rows materialize without a
screenshot, foreground-control side effect, or harness intervention. A future
implementation plan would need a reviewed, nonvisual readiness trigger or an
explicit product decision that foreground activation is acceptable.

Focused evidence review found no blocker and confirmed that the evidence is
sufficient to close this current-build reassessment. The retained `deadline`
status is not emitted by the current Rust-side polling path: bounded sampling
ends as `unstable_sampled`, while an outer trial timeout stops the harness.
Therefore a zero `deadline_stops` count must not be read as proof that no
deadline-related stop was possible.

The result also does not promote a broker or shared connection. Persistent
transport ownership alone does not cause hidden renderer content to
materialize. Future shared-connection feasibility may measure foreground and
renderer lifecycle ownership, but production indicator search remains deferred
until that work establishes a reviewed nonvisual readiness boundary or an
explicit foreground-control policy.

## Context and Orientation

The public CLI currently has indicator add/remove/toggle/set/get operations but
no search command. Indicator dispatch is in `crates/cli/src/app/dispatch.rs`,
CLI definitions are in `crates/cli/src/cli.rs`, and current operations are in
`crates/cli/src/ops/indicator.rs`.

The archived contract is
`docs/plans/archives/tradingview-cli-indicator-search-contract.md`. The parser
feasibility evidence is
`docs/plans/archives/tradingview-cli-indicator-search-parser-feasibility.md`.
The removed positive-result implementation trial is documented in
`docs/plans/archives/tradingview-cli-indicator-search-positive-results.md`.
The preserved stash is named
`recovered-indicator-search-prototype-2026-07-12` and currently identifies
object `55c37078aa8b3a0fc1f271500520fc1befb3bc12`.

A semantic anchor is an attribute intended to identify behavior, such as a
stable QA identifier or ARIA role. Generated style classes are not semantic
anchors. A readiness signal is evidence that the requested query has produced
the observed result state; input-value equality alone is insufficient.

## Plan of Work

First, inventory the current dialog shell, search input, category state,
result-host candidates, loading/empty semantics, and close/restoration controls.
Use aggregate counts and anchor categories only. Compare current observations
with old assumptions; do not preserve selectors merely because the stash used
them.

Second, use the test-only bounded probe in
`crates/cli/tests/live_indicator_search_reassessment.rs`. The probe uses one
absolute eight-second deadline per trial, sample no faster than 200 ms, and
retain no raw DOM or result titles. It must distinguish query assignment,
query dispatch evidence, first structural host appearance, two stable positive
samples, timeout, unexpected close, and restoration completion.

Compare the prototype setter/input event and native CDP typing in a six-trial
preflight: each candidate runs each of the three public built-in queries once
from the same initially open empty-query baseline. A candidate qualifies only
if all three trials prove dispatch, stable positive rows, and exact restoration.
If neither qualifies, record no-go. If both qualify, select the prototype
setter/input event because it changes only the input through the same path used
for restoration and avoids keyboard side effects. Do not combine or alternate
candidates after selection.

Run the selected candidate through the three queries. For each query, cover an
initially open dialog with empty query, an initially open dialog with a
different prior query, and an initially closed dialog that the probe opens and
later closes. Run each cell three times, for 27 bounded trials. Random ordering
is unnecessary and would weaken reproducibility. Stop the matrix immediately
on restoration failure and return the target to its recorded baseline before
any continuation.

The maximum authorized scope is six preflight trials plus 27 trials for the
selected candidate, or 33 trials total. Do not try unreviewed events, class
selectors, coordinate clicks, multiple signatures, or increasing timeouts
after a failure. A candidate that works only in one initial state is no-go.

Require the disposable target to begin with the Indicators dialog closed. The
harness also requires the current semantic dialog launcher to have rendered
geometry before any mutation. It does not use document visibility because the
foregrounded Electron chart reports a hidden document. Each trial constructs
and restores its declared state; a normal preflight no-go closes the dialog
before returning, and a completed matrix verifies the same closed outer
baseline. Unknown outcomes and restoration failures still stop without further
automatic mutation.

Finally, record one decision. Go requires one qualifying preflight candidate
and all 27 selected-candidate trials to observe exactly one class-free host,
positive query-sensitive rows, two stable samples, and exact restoration within
the existing deadline. No-go records failure counts by candidate, initial state,
and stage. Neither result adds a production command.

## Concrete Steps

Run from the repository root before live work:

    git status --short --branch
    git stash list --format='%gd %H %s'
    git diff --quiet HEAD -- crates Cargo.toml Cargo.lock
    rg -n "IndicatorCommand|indicator_add|RuntimeEvaluator" crates/cli/src
    rg -n "search|readiness|result host|restoration" docs/plans/archives/tradingview-cli-indicator-search-*.md

Any test-only probe must be ignored in ordinary Cargo tests and require an
explicit environment gate plus explicit target ID. Run deterministic fixtures
first, then the owner-approved live matrix. Record only aggregate fields:

    cargo test -p tradingview-cli --test live_indicator_search_reassessment -- --nocapture

    trials_requested
    trials_completed
    successes
    failures_by_initial_state
    failures_by_stage
    deadline_stops
    restoration_failures
    host_ambiguities
    latency_p50_ms
    latency_p95_ms

After evidence collection, run relevant test-only fixtures and:

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

The investigation is complete when every matrix cell finishes or a documented
restoration stop condition ends the run, output is aggregate and public-safe,
ordinary Cargo tests remain live- and Node-free, and an independent reviewer
can reproduce the deterministic parser/readiness contract.

Go requires a qualifying preflight candidate and all 27 selected-candidate
matrix trials to pass. It does not authorize implementation. It authorizes a new ExecPlan defining
the current production parser, Rust validation, public schema, diagnostics,
docs, executable fixtures, and one bounded live smoke. No-go must distinguish
an unavailable current signal from permanent impossibility.

## Idempotence and Recovery

The deterministic inspection is repeatable. Live trials transiently alter the
Indicators dialog and therefore require an owner-approved disposable target.
Capture initial dialog presence, query, and category before each trial. Restore
the exact state afterward. On unknown outcome or restoration failure, stop;
do not automatically retry or continue the matrix.

Never apply or drop either stash. Do not click a search result, add/remove a
study, save a script, change a layout, push, tag, or create a release.

## Artifacts and Notes

Record only aggregate counts, fixed status labels, bounded latency summaries,
and semantic anchor categories. Do not record raw DOM, selectors containing
generated classes, result titles, account-local script names or IDs, target
IDs, URLs, payloads, credentials, or machine-specific paths.

The implemented harness is
`crates/cli/tests/live_indicator_search_reassessment.rs`. Its ordinary run has
four passing deterministic tests and one ignored live matrix. The live gate is
`TV_LIVE_INDICATOR_SEARCH_REASSESSMENT=1`; an explicit disposable target is
selected through `TV_LIVE_INDICATOR_SEARCH_TARGET_ID`. Do not retain either
value in tracked evidence.

## Interfaces and Dependencies

This investigation adds no public interface or production dependency. A
test-only probe may use existing `RuntimeEvaluator`, CDP input primitives,
Tokio, and ignored integration-test infrastructure. It must not add a generic
browser automation layer.

## Open Questions

- UNCONFIRMED: whether the current Desktop build now exposes a stable
  query-associated readiness or loading marker.
- Confirmed: current `data-title` rows are parseable after rendering, but no
  reviewed nonvisual trigger makes them materialize within the ordinary CLI
  observation window.
- UNCONFIRMED: whether initially closed operation can be restored reliably
  without introducing a coordinate or generated-class dependency.
- UNCONFIRMED: whether the corrected current-build launcher succeeds through
  the full bounded matrix.

Revision note (2026-07-18): created after the owner requested a more complete
reassessment of the earlier defer decision. The plan treats the saved prototype
as research only and requires a 27-trial current-build matrix before any new
implementation plan.

Revision note (2026-07-18): focused plan review found that two dispatch
candidates and the 27-trial count were ambiguous. Added a six-trial preflight
that chooses one candidate before the 27-trial matrix, fixing the maximum live
scope at 33 trials, and added the two missing standard documentation/package
checks.

Revision note (2026-07-18): the initial completion audit passed focused review
and was archived. Marked this reassessment current and ready for deterministic
preparation; live execution remains separately owner-authorized.

Revision note (2026-07-18): added the test-only reassessment harness, fixed its
queries and prior-query baseline, kept raw rows and stability signatures inside
the page, and recorded green focused and full non-live validation. The live
matrix remains unexecuted and separately owner-authorized.

Revision note (2026-07-18): focused implementation review was green and
recommended finer no-go evidence. Added exact assignment readback for both
dispatch candidates, one restoration path for known assignment failure,
`dispatch_failed`, and `unstable_sampled`. Focused re-review is pending; the
live scope and authorization boundary are unchanged.

Revision note (2026-07-18): focused correction re-review was green with no
finding. The next gate is explicit owner approval for the bounded live matrix;
no live execution or production implementation is authorized by review alone.

Revision note (2026-07-19): the owner-approved run stopped before query
dispatch because the old launcher selector was absent and Electron reported
the embedded chart document as hidden. Recorded zero completed search trials,
updated the harness to the read-only-confirmed current semantic launcher, and
used rendered geometry after confirming that the visibly foregrounded Desktop
tab still reports hidden document visibility.

Revision note (2026-07-19): Electron timer throttling prevented page-side
polling from returning within the outer deadline, so the harness moved the same
200 ms cadence to Tokio while retaining page-local signatures. The completed
six-trial preflight assigned and restored every query for both dispatch
candidates, but all six ordinary observations reported `host_missing`. Visual,
accessibility, and aggregate DOM inspection then proved that current
`data-title` rows appear after rendering. Reclassified the result from parser
no-go to readiness-specific defer and did not run the gated 27-trial matrix.
