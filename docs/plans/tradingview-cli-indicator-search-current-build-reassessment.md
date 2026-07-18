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
- [ ] Start only after the v0.29 completion audit receives focused audit review.
- [ ] Obtain focused independent review of this reassessment plan.
- [ ] Reconfirm current-build dialog ownership and semantic anchors without
  recording private result text or raw DOM.
- [ ] Run the bounded positive-readiness and restoration matrix on an
  owner-approved disposable target.
- [ ] Record go/no-go evidence and obtain focused evidence review.
- [ ] Archive this plan without applying the prototype stash.

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

## Outcomes & Retrospective

Not yet executed. Record whether the current build provides a stable dispatch
and positive-result readiness boundary across all required initial states,
whether restoration is exact, and whether a separate implementation plan is
justified.

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

Second, implement or use a test-only bounded probe. The probe must use one
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

## Interfaces and Dependencies

This investigation adds no public interface or production dependency. A
test-only probe may use existing `RuntimeEvaluator`, CDP input primitives,
Tokio, and ignored integration-test infrastructure. It must not add a generic
browser automation layer.

## Open Questions

- UNCONFIRMED: whether the current Desktop build now exposes a stable
  query-associated readiness or loading marker.
- UNCONFIRMED: whether class-free host structure remains unique across all
  required initial states.
- UNCONFIRMED: whether initially closed operation can be restored reliably
  without introducing a coordinate or generated-class dependency.

Revision note (2026-07-18): created after the owner requested a more complete
reassessment of the earlier defer decision. The plan treats the saved prototype
as research only and requires a 27-trial current-build matrix before any new
implementation plan.

Revision note (2026-07-18): focused plan review found that two dispatch
candidates and the 27-trial count were ambiguous. Added a six-trial preflight
that chooses one candidate before the 27-trial matrix, fixing the maximum live
scope at 33 trials, and added the two missing standard documentation/package
checks.
