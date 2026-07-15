# Verify and harden current-build indicator insertion

This ExecPlan is a living document. The sections `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to
date as work proceeds.

Maintain this document in accordance with `.agents/PLANS.md`.

## Purpose / Big Picture

After this work, `tv indicator add "Volume"` should either add exactly the
requested built-in indicator on the selected TradingView Desktop chart and
verify the resulting study and requested inputs, or fail before mutation with
public-safe diagnostics. It must not guess between duplicate descriptions,
report the first of several new studies as success, or silently fall back to
the legacy `createStudy` path when the current-build insertion capability is
unavailable.

This plan uses two feasibility gates because the relevant TradingView page APIs
are private and build-dependent. A read-only discovery gate may establish only
semantic ownership and candidate shape. It cannot prove insertion behavior. A
separately owner-authorized disposable mutation gate must prove keyed inputs,
awaitable completion, one-study creation, readback, and cleanup before a
production go decision. A no-go at either gate leaves production behavior
unchanged rather than introducing a DOM-click fallback.

## Progress

- [x] (2026-07-14) Reviewed the current Rust indicator adapter, the local
  upstream codebase, the v0.28 roadmap, and the ordered work inventory.
- [x] (2026-07-14) Confirmed that neither current implementation provides the
  required exact metainfo resolution, awaited insertion, and exactly-one-study
  verification contract.
- [x] (2026-07-14) Applied independent-review corrections to the two-stage
  feasibility, exact-comparison, entity-ID, milestone, and strict
  await-immediate-snapshot contracts.
- [x] (2026-07-14) Ran a bounded, read-only current-build capability probe for
  the study metainfo repository and insertion method without adding a study.
- [x] (2026-07-14) Recorded read-only provisional go with exact field names,
  semantic ownership, callable shape, and all remaining dynamic unknowns.
- [x] (2026-07-14) Ran additional bounded non-mutating discovery and selected
  exact mutation evidence input `length: 21` from public `Volume` metainfo.
- [x] (2026-07-14) Fixed the candidate factory and insertion invocation shape,
  receiver bindings, argument vectors, and exactly-once mutation boundary from
  upstream pull-request evidence.
- [x] (2026-07-15) Obtained focused review of the expanded discovery evidence,
  strict mutation-gate ordering, exact invocation, and failure cleanup; no
  findings remain.
- [x] (2026-07-15) Obtained explicit owner approval for the exact disposable
  mutation feasibility sequence before invoking the factory, configurator, or
  inserter.
- [x] (2026-07-15) Ran the one-attempt `Volume` mutation feasibility probe and
  removed the uniquely observed new study by chart-local ID.
- [x] (2026-07-15) Recorded production no-go because strict immediate instance
  description readback did not match `Volume`, despite thenable fulfillment,
  exactly-one creation, instance lookup, and `length: 21` input readback.
- [x] (2026-07-15) Took the no-go path: no production code or probe artifact was
  added, no alternate signature was tried, and the compatibility gap is
  retained as deferred evidence.
- [x] (2026-07-15) Ran docs-only validation and synchronized the plan index,
  roadmap, work inventory, changelog, and continuity ledger.
- [x] (2026-07-15) Obtained independent review of the original no-go evidence
  and cleanup. The review found no evidence or cleanup defect, but identified
  an inapplicable production-go smoke milestone and requested docs correction.
- [x] (2026-07-15) Kept the original probe no-go intact and ran a separate
  bounded non-mutating study-readback investigation before archive. Current
  inventory and lookup wrappers expose no `metaInfo()` or `getStudyMeta()`;
  inventory `name` is readable and can be joined by the same chart-local ID to
  an input-readable lookup instance.
- [x] (2026-07-15) Closed the production-go-only final smoke as not applicable
  to the original no-go branch. No additional mutation was performed.
- [x] (2026-07-15) At owner direction, proceeded directly with one revised
  bounded probe because the new contract uses already observed current-build
  inventory and lookup surfaces. The probe passed every condition and removed
  the uniquely added study by the same chart-local ID.
- [x] (2026-07-15) Replaced the legacy add path with exact metainfo resolution,
  awaited chart-owned insertion, immediate inventory-name verification,
  same-ID input readback, bounded failure cleanup, and public-safe diagnostics.
- [x] (2026-07-15) Added focused Rust coverage and a pinned executable
  JavaScript contract, then wired that contract into CI and release gates.
- [x] (2026-07-15) Ran the full local baseline, all three pinned JavaScript
  gates, metadata, public hygiene, workflow parsing, packaging syntax, and
  guide parity; all checks passed.
- [x] (2026-07-15) Ran the final owner-authorized public-safe CLI smoke on the
  selected ELVN chart. Add verified one exact `Volume` with `length: 21`, and
  remove restored the original study count.
- [x] (2026-07-15) Synchronized public docs and completed this plan. The owner
  directed implementation to proceed from direct current-build evidence rather
  than add another planning-only review gate.
- [x] (2026-07-15) Obtained independent implementation review. It found
  special input-key loss, cleanup reclassification from a later snapshot,
  insufficient Rust result typing, and incomplete executable failure coverage.
- [x] (2026-07-15) Implemented the review corrections: parse serialized inputs,
  use exact definition membership, validate result types/count relationships,
  reuse the first post-await delta for settled cleanup, and expand fixture
  counters and failure cases.
- [x] (2026-07-15) Re-ran the full Rust baseline, all three pinned JavaScript
  gates, metadata, public hygiene, package syntax, guide parity, and diff
  checks; all passed after the corrections.
- [x] (2026-07-15) Focused re-review confirmed the prior input-key, cleanup,
  sanitization, and project-state corrections, then found three deterministic
  residual gaps: unsafe integer rounding, remaining failure-only success
  markers, and incomplete failure-path exactly-once assertions.
- [x] (2026-07-15) Rejected numeric values that cannot cross into JavaScript
  losslessly, rejected every known failure-only success marker, and asserted
  factory/configurator/insert counts across the complete fixture matrix.
- [x] (2026-07-15) Re-ran focused Rust and pinned JavaScript contracts, the
  full workspace baseline, pre-connect CLI validation, metadata, public
  hygiene, package syntax, guide parity, and diff checks; all passed.
- [x] (2026-07-15) Obtained focused independent re-review of the corrected
  implementation; no findings remain.
- [x] (2026-07-15) Archived this plan again and unblocked launch environment
  hardening.

## Surprises & Discoveries

- Observation: The Rust adapter calls `chart.createStudy` without awaiting its
  result, sleeps for 1.5 seconds, and accepts the first new study ID.
  Evidence: `crates/cli/src/ops/indicator.rs::indicator_add` builds an input
  array, calls `createStudy`, then computes an ID difference after `sleep`.

- Observation: The latest local upstream implementation does not supply the
  desired contract. Its chart path still uses `createStudy` and applies inputs
  afterward; its search path clicks the first exact or substring row in a
  virtualized dialog.
  Evidence: `src/core/chart.js::manageIndicator` and
  `src/core/indicators.js::addStudyFromSearch` in the local upstream checkout.

- Observation: Current Desktop exposes the metainfo repository directly from
  the active chart and the candidate inserter directly from that chart's model.
  Evidence: A bounded read-only ELVN probe observed callable
  `chart.studyMetaIntoRepository()`, an array from
  `getInternalMetaInfoArray()`, and callable
  `chart._chartWidget.model().createStudyInserter`. The probe called none of
  these mutation methods and reported `mutation_count: 0`.

- Observation: The initial exact metainfo contract is present for `Volume`.
  Evidence: Among 241 metainfo rows, exactly one case-sensitive trimmed
  `description` matched `Volume`; that row had one non-empty string `id`, one
  `inputs` array, and two non-empty string `inputs[].id` values. Only aggregate
  counts were retained.

- Observation: `Volume` exposes one safe non-default scalar input for keyed
  mutation evidence.
  Evidence: Additional bounded non-mutating discovery observed `length` as an
  integer with default 20 and inclusive public bounds 1 through 2000, and
  `col_prev_close` as a boolean with default false. The probe selected
  `length: 21`, which differs from default and requires no coercion, clamp, or
  enum canonicalization. The probe again reported `mutation_count: 0`.

- Observation: Upstream pull request #334 supplies one concrete current-build
  candidate call shape rather than merely a method name.
  Evidence: It resolves `meta.id`, invokes the chart model's
  `createStudyInserter({ type: "java", studyId: meta.id }, [])`, then invokes
  the returned inserter's `insert` with one provider function that resolves to
  `{ inputs: keyedInputs, parentSources: [] }`. Its live report is evidence for
  this hypothesis, not proof for the Rust implementation.

- Observation: The exact upstream-derived invocation creates one readable
  study and applies the non-default keyed input, but fails the strict public
  description contract on current Desktop.
  Evidence: The owner-approved one-attempt probe recorded factory, overlay
  configurator, and insert call counts of one; a thenable fulfilled; the first
  immediate post-await snapshot contained exactly one new study; instance
  lookup and `length: 21` readback succeeded; strict trimmed case-sensitive
  `Volume` description comparison failed. No raw description or identifier was
  retained.

- Observation: Identity-based cleanup succeeded after the no-go result.
  Evidence: The fulfilled-outcome cleanup classified exactly one new
  chart-local entity, removed only that ID, and the immediate post-remove
  snapshot confirmed absence. There was no timeout, retry, polling, or
  alternate-signature attempt.

- Observation: The failed description condition used a metadata surface that
  current chart study wrappers do not expose.
  Evidence: A bounded non-mutating probe over 27 existing studies found zero
  callable `metaInfo()` and zero callable `getStudyMeta()` methods on both
  `getAllStudies()` rows and their `getStudyById()` lookup instances. All 27
  inventory rows had readable names, including exactly one trimmed
  case-sensitive `Volume`. A second chart showed the same wrapper split.

- Observation: Current Desktop supports a same-identity verification split:
  public inventory name from `getAllStudies()` and requested inputs from
  `getStudyById(row.id).getInputValues()`.
  Evidence: On an existing built-in `Volume`, one exact inventory-name match
  joined by its chart-local ID to one lookup instance with callable input
  readback and one scalar `length` entry. The same chart's metainfo repository
  independently contained one exact `Volume` description and one `length`
  definition. Only aggregate counts and booleans were retained.

- Observation: The local upstream add verifier also treats inventory name as
  the compatibility fallback rather than requiring instance metadata.
  Evidence: `src/core/indicators.js::addStudyFromSearch` reads
  `getStudyMeta().description` only when that method exists and otherwise uses
  the inventory row's `name`. This is supporting evidence, not a sufficient
  production contract by itself.

## Decision Log

- Decision: Preserve the public `tv indicator add` command and replace only
  its internal add path if current-build feasibility is proven.
  Rationale: Users should not need a second command for the same chart
  mutation, while the existing remove, toggle, set, and get operations do not
  depend on the insertion mechanism.
  Date/Author: 2026-07-14 / Codex.

- Decision: Do not use indicator-dialog search or DOM row clicking as an
  insertion fallback.
  Rationale: The dialog parser is separately deferred, virtualized rows are
  build-sensitive, and first-match clicking cannot prove unique identity
  before mutation.
  Date/Author: 2026-07-14 / Codex.

- Decision: Require zero-or-one exact metainfo resolution before mutation and
  exactly one new observed study afterward.
  Rationale: Ambiguous names and multiple new entities make the requested
  operation unverifiable. Failing closed is safer than selecting an arbitrary
  candidate or entity.
  Date/Author: 2026-07-14 / Codex.

- Decision: Separate read-only discovery from mutation feasibility and require
  owner approval for the latter.
  Rationale: A callable method cannot prove its accepted argument shape,
  return thenability, or insertion effects without being invoked. Read-only
  evidence therefore cannot authorize production replacement by itself.
  Date/Author: 2026-07-14 / Codex.

- Decision: Define exact description matching as ECMAScript `String.trim()` on
  both strings followed by case-sensitive equality, with no case folding,
  Unicode normalization, or internal whitespace collapsing.
  Rationale: This permits only irrelevant edge whitespace while avoiding
  locale-dependent or lossy matching.
  Date/Author: 2026-07-14 / Codex.

- Decision: Preserve the existing public `entity_id` success field as a
  chart-local study handle.
  Rationale: Follow-up remove, toggle, set, and get commands require that
  handle, and the existing add contract already returns it. It is not a CDP
  target ID or an account-local saved-object ID. Live evidence must not record
  its concrete value.
  Date/Author: 2026-07-14 / Codex.

- Decision: Record provisional go for the read-only discovery gate only.
  Rationale: Semantic ownership, exact `Volume` resolution, required metainfo
  fields, and callable shape are confirmed without mutation. Keyed-input
  acceptance, thenability, await-immediate inventory, readback, and cleanup
  remain unconfirmed and require a separately approved mutation probe.
  Date/Author: 2026-07-14 / Codex.

- Decision: Use exactly `{ "length": 21 }` for the owner-authorized `Volume`
  mutation feasibility probe.
  Rationale: `length` is a public integer input with default 20 and bounds 1 to
  2000. Value 21 is non-default, valid without canonicalization, and therefore
  proves whether the keyed override was applied rather than merely matching a
  default.
  Date/Author: 2026-07-14 / Codex.

- Decision: Permit exactly one candidate insertion signature in mutation
  feasibility, with explicit receiver binding and no signature fallback.
  Rationale: Owner approval must cover a concrete operation. Trying alternate
  argument orders or methods after a failure would turn one bounded probe into
  open-ended private-API mutation.
  Date/Author: 2026-07-14 / Codex.

- Decision: Preserve the original probe's no-go result, but do not treat the
  unavailable instance-description API as final insertion infeasibility.
  Rationale: The approved attempt correctly failed its reviewed contract, yet
  subsequent non-mutating evidence shows that current inventory and lookup
  wrappers do not expose the required metadata methods at all. The insertion,
  completion, identity, and keyed-input parts succeeded. A revised
  inventory-name contract therefore merits focused review rather than archive
  or an unreviewed fallback.
  Date/Author: 2026-07-15 / Codex.

## Outcomes & Retrospective

Planning, initial read-only discovery, owner approval, and the exact one-attempt
mutation feasibility probe are complete. That attempt remains no-go under its
reviewed acceptance contract: insertion and keyed-input readback succeeded,
but the required instance-description check did not. The uniquely observed
study was removed successfully. Follow-up non-mutating investigation found
that the required metadata method is absent from current inventory and lookup
wrappers, while an exact inventory name can be joined by the same chart-local
ID to input readback. At owner direction, the revised probe proceeded without
another planning-only review and passed. The production add path and executable
JavaScript contract are implemented, and the earlier CLI add/remove smoke
restored the selected chart. Independent review then found three
safety-boundary defects and incomplete fixtures. Those corrections are
implemented, and focused re-review found no remaining issue. This plan is
complete and archived; launch environment hardening is the next planned slice.

## Context and Orientation

`tv indicator add` is dispatched to
`crates/cli/src/ops/indicator.rs::indicator_add`. The operation runs JavaScript
inside TradingView Desktop through the `RuntimeEvaluator` trait from
`tradingview-cdp`. The selected chart is obtained through the shared
`CHART_API` expression in `crates/cli/src/ops/common.rs`.

A study is TradingView's page-side object for an indicator or strategy. A
metainfo repository is the page-side catalog that describes available studies,
including a stable identifier, public description, and input definitions. A
study inserter is the page-side capability that creates a study from resolved
metainfo. Both names describe private TradingView internals, so the probe must
verify method presence and behavior on the current build rather than assume a
specific upstream patch remains valid.

The existing add path converts `--inputs` from a JSON object into an array of
`{ id, value }`, passes that array to `chart.createStudy`, waits a fixed 1.5
seconds, and compares `getAllStudies()` IDs. It errors only when no new entity
ID is present. It does not reject multiple new studies, await the insertion
result, or read back applied inputs. Existing tests use `FakeRuntime` payloads
and mostly assert generated-source fragments.

Indicator dialog search is separate. Prior work found a limited structural
parser but did not establish reproducible positive-result readiness, so the
prototype was removed and preserved in a named stash. Do not apply, drop, or
use that stash as an insertion dependency.

## Plan of Work

First, add an ignored, opt-in feasibility test or a temporary untracked probe
that performs one `Runtime.evaluate` call with no mutation. Inspect the active
chart and nearby page-owned registries for a metainfo catalog and insertion
method. Return only aggregate public-safe facts: candidate counts, whether
descriptions and stable identifiers are readable, and whether an insertion
method is semantically owned by the same chart surface and callable. Do not
claim its argument shape, thenability, or effects. Do not return raw
objects, source code, account-local IDs, target IDs, stack traces, or method
source. Use one finite deadline. Remove temporary artifacts before commit.

The read-only gate may record provisional go only when it identifies one
semantically chart-owned repository, resolves `Volume` to exactly one candidate
using the exact matching rule below, reads that candidate's string
`description`, string `id`, and input definitions from `inputs[].id`, and finds
one semantically chart-owned callable insertion capability. These are the only
accepted metainfo fields for the initial contract; missing fields, aliases, or
multiple possible repositories/inserters are no-go until this plan is revised
and reviewed. Method-name presence alone is insufficient, but the read-only
gate explicitly leaves keyed-input acceptance, return thenability, actual
creation, and readback unconfirmed.

After provisional go, update this living plan with the exact repository and
inserter ownership path observed, without recording private values, and obtain
focused review. Then request separate owner approval for a disposable mutation
feasibility probe. That probe may invoke the candidate inserter once for
`Volume`, must pass exactly `{ "length": 21 }`, and must test whether
the actual return is thenable and await it. Immediately after the await
resolves, it must take one inventory snapshot and require exactly one new
study in that first snapshot. From the same snapshot it must resolve the new
instance and immediately read its description and inputs through
`getStudyById(entityId).getInputValues()`, verify the description and requested
input, and remove only that uniquely observed entity. Do not poll after await
for appearance or readback readiness: a non-thenable return, rejection,
timeout, zero or multiple new studies in the first post-await snapshot, or
unavailable immediate readback is no-go. If owner approval is not given, or
any dynamic property fails, record no-go. A production go decision is
forbidden until this mutation probe and its focused evidence review are green.

The candidate invocation is fixed to the following argument order and
receivers. Resolve the unique `Volume` metainfo row first and retain `meta.id`
only inside the page. Resolve `model` exactly as
`chart._chartWidget.model()`. Invoke the factory exactly once with `model` as
`this`:

    const inserter = model.createStudyInserter.call(
        model,
        { type: "java", studyId: meta.id },
        []
    );

Require the returned value to be a non-null object whose `insert` is callable.
Also require `meta.is_price_study` to be a boolean and
`inserter.setForceOverlay` to be callable. Invoke that configurator exactly once
as `inserter.setForceOverlay.call(inserter, meta.is_price_study)`. If either
requirement is missing, do not call `insert`; record no-go. Do not try another
placement method or coerce a missing field. Then invoke the only chart-mutating
method exactly once with `inserter` as `this` and one provider callback:

    const insertion = inserter.insert.call(inserter, function() {
        return Promise.resolve({
            inputs: { length: 21 },
            parentSources: []
        });
    });

`inserter.insert.call(...)` is the exactly-once study-mutation boundary. The
factory and placement-configurator calls are each also limited to once and are
part of the owner-approved call sequence. Require `insertion` to be thenable
before awaiting it. This shape is a hypothesis derived from upstream PR #334
and is what the owner would approve for feasibility; the probe validates
acceptance, thenability, strict completion, readback, and cleanup. If the
factory, configurator, or inserter rejects this exact shape, stop with no-go.
Do not try another `type`, argument vector, provider shape, receiver, method,
placement mode, or legacy `chart.createStudy` fallback.

Before invocation, capture one baseline study-ID snapshot. After invocation,
always perform cleanup classification, but keep it separate from completion
acceptance. On a normally settled thenable, the first immediate post-await
snapshot is both the acceptance snapshot and the cleanup identity source. On a
rejected thenable or malformed settled result, take exactly one immediate
cleanup-only inventory snapshot against the baseline; this snapshot cannot
rescue production-go evidence. If it has exactly one new chart-local ID, remove
only that ID and verify absence with one immediate post-remove snapshot. If it
has zero, remove nothing. If it has multiple or ambiguous identity, remove
nothing, stop, and report the aggregate count to the owner. If the bounded
operation times out, do not retry, poll, or remove automatically because the
page-side operation may still be running; stop and require separate owner
approval for a later read-only recovery observation before any further
mutation.

Exact description matching requires both values to be strings, applies
ECMAScript `String.trim()` to their edges, and then compares case-sensitively.
Do not normalize Unicode, fold case, or collapse internal whitespace. Requested
input keys must exactly equal a string `inputs[].id` from the resolved metainfo.
For this first slice, requested values must be JSON null, boolean, finite number,
or string; arrays and objects fail validation before mutation. Readback uses the
matching `id` entry from `getInputValues()`. Each requested key's JSON type and
value must equal readback exactly. Numeric coercion, enum rewriting, and
TradingView clamp/canonicalization are mismatches. Compare requested keys only;
defaults and unrequested inputs are diagnostics, not acceptance conditions.

On go, revise only the add branch in
`crates/cli/src/ops/indicator.rs`. Generate one bounded async page expression
that snapshots study IDs, resolves the requested public description exactly,
rejects zero or multiple candidates before mutation, invokes and awaits the
verified inserter with an object keyed by input ID, and takes exactly one
inventory snapshot immediately after await resolves. Success requires exactly
one new study in that snapshot and immediate description/input readback from
that instance. Do not poll after await; the thenable itself is the required
completion boundary. Unknown input IDs, rejected insertion, timeout, zero or
multiple new studies, missing immediate readback, or mismatched input values
must fail closed. Any later evidence that current Desktop needs post-await
polling requires a new plan revision and focused review rather than an implicit
relaxation.

At the Rust boundary, validate the returned object rather than trusting
page-side booleans. Build success output from a whitelist containing the
operation, requested and observed public indicator names, the new entity ID,
counts, requested inputs, observed inputs, verification booleans, and source
metadata already used by the command. Build failure details from fixed stage,
counts, public candidate descriptions only when bounded, and a fixed next
action hint. Never attach the raw Runtime payload or evaluation error details.
Preserve the original `ErrorKind` when sanitizing an evaluation failure.
The `entity_id` is the existing chart-local study handle required by follow-up
indicator commands. Do not relabel or expose any repository ID, CDP target ID,
layout ID, or account-local object ID as `entity_id`.

Keep `indicator_remove`, `indicator_toggle`, `indicator_set`, and
`indicator_get` behavior unchanged. Do not add dependencies, a new command,
automatic source mixing, retries, dialog clicks, or the old path as fallback.

Add deterministic tests that execute the production-generated JavaScript with
synthetic chart, repository, inserter, and study objects. Use the repository's
pinned Node.js gate pattern rather than making ordinary Cargo tests depend on
an undeclared Node binary. Cover one exact candidate, zero candidates,
duplicate exact candidates, non-awaitable insertion, rejected insertion,
timeout, zero and multiple new studies, keyed inputs, unknown inputs, and
input-readback mismatch. Rust tests must cover malformed success payloads,
evaluation-error sanitization, private-value non-leakage, and unchanged
remove/toggle/set behavior.

After implementation, update README help examples only if user-visible output
changes, plus `docs/development.md`, `docs/internal-tradingview-apis.md`, the
v0.28 roadmap and inventory, `CHANGELOG.md`, packaged agent guidance, and the
Pine development skill reference. Keep operational guidance concise and place
contract detail in references rather than enlarging a skill's core workflow.

## Milestones

### Read-only discovery and provisional decision

Create and run the bounded non-mutating probe. At completion, this plan must
record the observed semantic ownership, exact accepted field names, candidate
count, callable count, and a stop or provisional-go decision. Run the ignored
probe explicitly and expect aggregate public-safe output with mutation count
zero. A provisional go does not authorize production code or mutation.

### Owner-authorized disposable mutation feasibility

Only after the revised discovery evidence passes focused review and the owner
explicitly approves mutation, run one bounded `Volume` insertion attempt with
exact input `{ "length": 21 }`. At completion, evidence must show whether the
actual return was thenable, completion settled, exactly one study was present
in the first immediate post-await inventory snapshot, exact description and requested
input readback were available from that same snapshot, and the uniquely
observed entity was removed. Post-await polling cannot satisfy this milestone.
Record only booleans and counts. Any ambiguity or missing approval is no-go.
Revise this plan with the proven callable contract and obtain focused review
before production edits.

The approved operation must use only the exact
`model.createStudyInserter.call`, `inserter.setForceOverlay.call`, and
`inserter.insert.call` forms specified in Plan of Work. Count each separately
and require
`factory_call_count == 1`, `overlay_config_call_count == 1`, and
`insert_call_count == 1` in public-safe evidence. `insert_call_count` is the
study-mutation count. Any alternate-signature attempt is a contract violation,
not a recovery step.

The probe must capture one baseline inventory before invocation. Normally
settled outcomes use the first post-await snapshot for acceptance and cleanup
identity. Rejected or malformed settled outcomes receive exactly one separate
cleanup-only snapshot, which never counts as completion evidence. Remove only
one uniquely new chart-local ID and verify its absence once. Zero means no
removal; multiple means no removal and owner escalation. Timeout means no
automatic observation, retry, or cleanup until a separate owner-approved
read-only recovery step.

### Production replacement and deterministic contracts

On reviewed production go, replace only `indicator_add`, add Rust and pinned
JavaScript contracts, and wire the JavaScript gate into CI and release tests.
At completion, focused tests must prove exact pre-mutation rejection, awaited
completion, exactly-one post-check, strict requested-input equality, malformed
payload rejection, and public-safe errors. No live operation occurs here.

Before production go can be reconsidered, focused review must accept the
revised current-build readback boundary: the first immediate post-await
`getAllStudies()` snapshot supplies the uniquely new row and its exact public
`name`; the same row's chart-local `id` must resolve exactly one
`getStudyById()` instance whose `getInputValues()` strictly matches requested
inputs. The inventory row and lookup instance must be joined only by that ID.
Do not fall back to display-name lookup, another snapshot, polling, or an
unrelated source wrapper. The original mutation evidence does not prove the
new row's name, so a reviewed and separately owner-approved second probe is
required before production edits.

### Validation, documentation, and live acceptance

Run the full local baseline and synchronize public docs and skill references.
Only after a reviewed production implementation and separate owner approval,
run one bounded public-safe add/remove smoke. Completion requires all
deterministic and full checks green, one uniquely verified live study removed
by its chart-local entity ID, and no unrelated chart mutation. For a no-go
branch with no production implementation, this final implementation smoke is
not applicable and must not remain as an open archive prerequisite.

## Concrete Steps

Work from the repository root.

Inspect the current operation and its callers:

    rg -n "indicator_add|createStudy|IndicatorCommand" crates/cli/src crates/cli/tests

Inspect the local upstream evidence without copying it verbatim:

    rg -n "addStudyFromSearch|createStudy|study inserter|metainfo" ../tradingview-mcp/src ../tradingview-mcp/tests

Run focused tests during implementation:

    cargo test -p tradingview-cli indicator -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop indicator -- --nocapture

Run the dedicated executable JavaScript contract through a pinned `mise` task
added alongside the existing Pine JavaScript gates. The task name must describe
indicator insertion and must be included in CI and release validation before
the implementation is considered complete.

Run the full baseline:

    mise run check:baseline
    mise run check:indicator-insertion-js
    mise run check:study-values-js
    mise run check:pine-open-js
    cargo metadata --no-deps --format-version 1
    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    git diff --check

## Validation and Acceptance

The deterministic acceptance suite must prove that mutation is never invoked
for zero or duplicate exact descriptions, that the inserter is awaited, that
only one newly observed entity is accepted, and that requested keyed inputs
match the new study's readback. Removing the await, changing exact resolution
to first-match selection, or accepting two new study IDs must make a test fail.

Normal Cargo tests must remain runnable without Node.js on `PATH`. The dedicated
JavaScript gate must use the repository-pinned Node.js version and execute the
production-generated expression, not a copied approximation.

After independent implementation review is green, request separate owner
approval for the final live smoke. The smoke should add one ordinary built-in indicator to the currently
selected disposable chart, report only the requested/observed public name,
new-study count, input verification status, and removal status, then remove the
new entity. Do not record raw payloads or account-local values. A failed add
must not trigger cleanup of an unrelated entity.

## Idempotence and Recovery

The read-only probes are repeatable and must not call an insertion or UI-click
method. Each mutation-feasibility probe and any final smoke are separate,
non-idempotent owner-approved operations. Deterministic fixtures are local and
repeatable. For either mutation, exactly-one post-check and chart-local entity
identity are mandatory. If a new entity is uniquely observed within that same
attempt, cleanup may remove only that ID; otherwise stop and ask the owner.
Never remove by display name.

Preserve the named indicator-search prototype stash unchanged. Do not add or
drop a stash as part of this work.

## Artifacts and Notes

Current implementation evidence:

    chart.createStudy(indicator, false, false, inputArr)
    await sleep(1500)
    newIds = after - before

Required success shape, expressed conceptually rather than as a frozen wire
contract:

    exact candidate count = 1
    insertion completion awaited = true
    new study count = 1
    requested public name = observed public name
    every requested keyed input = observed input value

## Interfaces and Dependencies

Keep the existing Rust signature unless implementation evidence requires a
small private helper:

    pub async fn indicator_add(
        runtime: &mut impl RuntimeEvaluator,
        indicator: &str,
        inputs: Option<&serde_json::Value>,
    ) -> Result<serde_json::Value, AppError>

Private helpers may build the expression, normalize the returned object, and
sanitize errors. Do not expose a new public Rust API solely for tests. Reuse
`RuntimeEvaluator`, `AppError`, `ErrorKind`, `js_string`, and `CHART_API`. Add no
production dependency.

## Open Questions

- CONFIRMED: current Desktop exposes the metainfo array through
  `chart.studyMetaIntoRepository().getInternalMetaInfoArray()` and a callable
  candidate inserter through
  `chart._chartWidget.model().createStudyInserter`.
- CONFIRMED: the sole candidate invocation hypothesis is the upstream #334
  factory shape `{ type: "java", studyId: meta.id }, []` followed by one
  `setForceOverlay(meta.is_price_study)` and one `insert` provider resolving
  `{ inputs: { length: 21 }, parentSources: [] }`, with explicit model and
  inserter receiver bindings.
- UNCONFIRMED: whether the current-build inserter accepts keyed inputs directly
  for indicators other than the observed `Volume` attempt; this no-go slice
  does not generalize beyond its one approved probe.
- CONFIRMED: current inventory and lookup wrappers do not expose callable
  `metaInfo()` or `getStudyMeta()` in the observed build. Inventory `name` is
  readable, and the same chart-local ID can join an inventory row to an
  input-readable lookup instance.
- UNCONFIRMED: whether a newly inserted `Volume` row has exact inventory name
  `Volume` in the first immediate post-await snapshot. Existing-study evidence
  cannot substitute for a separately reviewed and owner-approved mutation
  probe of that condition.

Revision note (2026-07-14): Created this self-contained plan after Pine
saved-script safety and active-editor compatibility passed final focused
review. The plan deliberately starts with a read-only capability gate because
neither the current Rust path nor the latest local upstream implementation
meets the intended insertion contract.

Revision note (2026-07-14): Split feasibility into read-only discovery and a
separately owner-authorized disposable mutation probe, fixed exact description
and input equality rules, classified the existing chart-local `entity_id`, and
added independently verifiable milestones in response to plan review.

Revision note (2026-07-14): Defined the inserter thenable as a strict completion
boundary: exactly one new study and its description/input readback must be
available in the first inventory snapshot immediately after await, with no
post-await polling accepted as evidence.

Revision note (2026-07-14): Recorded read-only provisional go after an ELVN
probe confirmed chart-owned metainfo, exact `Volume` fields, and a
chart-model-owned callable inserter with zero mutation. Dynamic insertion
semantics remain unconfirmed pending focused review and separate owner approval.

Revision note (2026-07-14): Added non-mutating public input discovery, fixed the
mutation evidence input to non-default valid `length: 21`, defined baseline and
failure cleanup snapshots, prohibited automatic timeout recovery, and updated
confirmed versus unconfirmed questions after focused review.

Revision note (2026-07-14): Fixed the exact upstream-derived factory and
insertion call shape, explicit receiver bindings, argument order, exactly-once
mutation boundary, and no-alternate-signature policy before owner approval.

Revision note (2026-07-15): Recorded focused review green with no remaining
findings. The plan now waits for explicit owner approval of the exact mutation
feasibility sequence; production implementation remains unauthorized.

Revision note (2026-07-15): Recorded the owner-approved exact probe and final
production no-go. Thenable fulfillment, immediate exactly-one creation,
instance lookup, keyed input readback, and identity cleanup succeeded; strict
instance description readback failed. No alternate signature or retry was used.

Revision note (2026-07-15): Kept that attempt's no-go result but paused archive
after non-mutating follow-up showed that current inventory and lookup wrappers
do not expose the assumed instance metadata methods. Recorded a proposed
same-ID inventory-name/input-readback contract for focused review. No new study
was added or removed during this investigation.
