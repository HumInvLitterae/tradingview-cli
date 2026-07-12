# Define the indicator and strategy search contract

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while work proceeds.
Maintain it according to `.agents/PLANS.md`.

## Purpose / Big Picture

Users can currently add a study only when they already know a value accepted by
`tv indicator add`. TradingView Desktop's Indicators dialog can discover
built-in indicators, built-in strategies, Community scripts, invite-only
scripts, purchased scripts, and account-local saved scripts, but the Rust CLI
does not expose that discovery surface.

This work defines a safe, read-oriented `tv indicator search <QUERY>` contract
before implementation. The visible outcome of this planning slice is a
self-contained contract that an implementation agent can follow without
guessing how to distinguish a true empty result from DOM drift, how to restore
the dialog, or which result metadata may be exposed. No command, option, or
runtime payload is added in this contract slice. Once this plan is complete and
reviewed, R6 can implement the command against deterministic fixtures and a
bounded live smoke.

## Progress

- [x] (2026-07-12) Completed and archived Strategy Tester compatibility,
  including final independent review and the closeout commit.
- [x] (2026-07-12) Inspected the current Rust indicator lifecycle, the current
  localized Indicators dialog evidence, and upstream search/add code at the
  reviewed `55534aa` snapshot.
- [x] (2026-07-12) Created this R5 contract ExecPlan and fixed the first-slice
  boundary: read-oriented search only, with no study add or partial-match
  selection.
- [ ] Inventory current dialog states and result classes using only a dedicated
  test layout. Record public-safe structure summaries, not raw DOM or script
  names from account-local sections.
- [ ] Finalize normalized result fields, classification confidence, query and
  result limits, timeout, and explicit empty-state proof.
- [ ] Define deterministic parser fixtures for R6, including localization,
  virtualization, loading, empty, drift, and restoration cases.
- [ ] Validate the contract against current Desktop behavior and upstream
  lessons without changing chart studies or account data.
- [ ] Synchronize the roadmap, work inventory, plan index, changelog, and local
  continuity ledger as decisions mature.
- [ ] Obtain independent review, correct findings, archive this plan, and only
  then create the R6 implementation ExecPlan.

## Surprises & Discoveries

- Observation: the current localized dialog accepted a programmatically set
  query value but the upstream result parser returned zero rows even while a
  prior visual inspection showed matching strategy rows.
  Evidence: the upstream parser searches class-name fragments such as
  `container` and `title`; the current dialog exposed localized navigation text
  but no rows matching that parser. Therefore `results: []` cannot by itself
  mean no search result.

- Observation: the dialog is localized and its visible categories include
  personal, saved-script, built-in, technical, fundamental, Community, and
  store-oriented sections.
  Evidence: the dedicated test layout showed Japanese section labels. A result
  contract cannot derive stable section kinds from English labels alone.

- Observation: setting `HTMLInputElement.value` and dispatching `input` was not
  sufficient evidence that TradingView executed the search.
  Evidence: the input value changed, but no result or explicit empty state was
  observed. R6 needs a bounded readiness condition beyond input equality.

- Observation: upstream `addStudyFromSearch` permits a contains match after an
  exact match is unavailable.
  Evidence: current upstream code selects `exact || contains`. This is not safe
  for the later Rust add workflow, which must stop on ambiguity and require an
  exact, classified result.

## Decision Log

- Decision: use `tv indicator search <QUERY>` as the preferred CLI surface.
  Rationale: discovery belongs beside existing `indicator add/remove/toggle/
  set/get`; a new top-level group would obscure that lifecycle relationship.
  Date/Author: 2026-07-12 / Codex

- Decision: classify search as `source_category:
  "desktop_backed_operation"`, `requires_desktop: true`, and `non_mutating:
  false`, even though it does not add or remove a study.
  Rationale: opening, typing into, and closing the dialog changes transient UI
  state. Calling it a non-mutating read would hide restoration obligations.
  Date/Author: 2026-07-12 / Codex

- Decision: require explicit empty-state evidence before returning a successful
  zero-result payload.
  Rationale: the observed upstream parser failure looked identical to a valid
  empty result. Unknown DOM, missing result root, unresolved loading, or query
  dispatch failure must return a public-safe unavailable diagnostic instead.
  Date/Author: 2026-07-12 / Codex

- Decision: return raw display labels only when the user explicitly requested
  the search, and classify account scope separately. Do not return script IDs,
  source code, account-local IDs, or hidden internal selectors.
  Rationale: titles are the requested search evidence, but saved/private script
  names can be account-local. The payload must identify that scope so agents do
  not copy those values into shared notes by default.
  Date/Author: 2026-07-12 / Codex

- Decision: do not promise a reusable opaque selector in R5 unless current DOM
  inventory proves one exists and remains valid after dialog restoration.
  Rationale: a row index is meaningful only within one rendered result snapshot
  and cannot safely drive a later process invocation. R7 will require exact
  title plus classification or another independently verified stable selector.
  Date/Author: 2026-07-12 / Codex

- Decision: keep search and exact-add in separate slices.
  Rationale: search may transiently mutate UI state but must not mutate chart
  studies. Exact-add requires separate ambiguity checks, before/after study
  identity, and mutation/restoration readback.
  Date/Author: 2026-07-12 / Codex

## Outcomes & Retrospective

R5 contract drafting is underway. The command boundary, source classification,
empty-result rule, privacy posture, and separation from exact-add are fixed.
Current DOM inventory, normalized classification confidence, parser fixture
shapes, and final review remain before this plan can close.

The expected R6 outcome is a bounded command that either returns trustworthy,
input-ordered normalized results or an explicit diagnostic. It must never turn
selector drift into a false zero-result success and must leave the dialog in
its original open/closed state and restore any pre-existing query when it can
be observed.

## Context and Orientation

The Rust CLI command definitions live in `crates/cli/src/cli.rs`.
`IndicatorCommand` currently contains `add`, `remove`, `toggle`, `set`, and
`get`. Dispatch and pre-connection validation live in
`crates/cli/src/app/dispatch.rs`. Indicator operations live in
`crates/cli/src/ops/indicator.rs` and use `RuntimeEvaluator` to execute
JavaScript against the selected TradingView Desktop chart.

`tv indicator add` calls the chart API's `createStudy` method using a supplied
name and verifies that exactly one new chart-local study identity appears. It
does not search the Indicators dialog. R5 does not change that behavior, and
R6 must not make search a hidden fallback inside add.

The researched upstream implementation opens
`[data-name="indicators-dialog"]`, sets the first input value, waits a fixed
delay, then searches class-name fragments for section headers and title rows.
It closes the dialog after reading. That implementation is useful evidence for
the basic workflow, but current localized Desktop evidence shows that its
parser and readiness conditions can return a false empty result. The Rust
contract therefore requires explicit parser/readiness diagnostics and
restoration evidence.

A virtualized list renders only a subset of rows near the visible scroll
position. Search results may therefore exist without every matching row being
present in the DOM simultaneously. R6 must define whether it returns only the
bounded rendered result set or performs bounded scrolling. The initial
contract chooses rendered results only, capped at 25, and must say so in the
payload. Bounded scrolling can be proposed later only with stable progress and
termination evidence.

## Proposed Contract

The future CLI surface is:

    tv indicator search <QUERY> [--limit <N>]

`QUERY` is required after trimming, may contain at most 200 Unicode scalar
values, and must be validated before CDP connection. `--limit` defaults to 25
and accepts `1..=50`; the first implementation reads at most 50 rendered rows
and does not scroll the virtualized list.

The success payload uses `contract_version: "indicator_search.v1"` and returns
these workflow fields:

    query
    result_count
    result_limit
    result_scope: "rendered_rows"
    truncated
    results[]
    dialog_state_before
    dialog_state_after
    restoration_status
    search_readiness
    source: "indicators_dialog_dom"
    source_category: "desktop_backed_operation"
    requires_desktop: true
    non_mutating: false
    operation: "indicator_search"

Each result preserves DOM order and contains:

    result_index
    title
    section_label
    section_kind
    script_kind
    author_label
    access_scope
    classification_status

`section_kind` uses `built_in`, `technical`, `fundamental`, `community`,
`my_scripts`, `invite_only`, `purchased`, `store`, or `unknown` only when a
stable DOM signal or explicitly tested localized mapping supports it.
`script_kind` uses `indicator`, `strategy`, `library`, or `unknown` only when
the result row exposes a stable badge or semantic attribute. Title text alone
must not classify a strategy. `access_scope` uses `built_in`, `public`,
`account_local`, `invite_only`, `purchased`, or `unknown`.

`classification_status` is `observed` when the kind/scope came from stable row
or section semantics and `partial` when one or more fields are unknown. Missing
optional classification never removes a result, but the command must not
invent values from title keywords. No result contains script source, internal
script ID, saved-script ID, target ID, raw DOM, event handlers, or account
identity.

The first contract does not expose a reusable selection token. `result_index`
is explicitly scoped to this response and is not accepted by `tv indicator
add`. R7 must define its own exact-match mutation request after R6 proves which
stable result identity is available.

## Search Readiness and Empty Results

R6 must treat a search as ready only after all of these are observed within a
five-second absolute deadline: the intended dialog target exists, the intended
input contains the normalized query, a result container or an explicit empty
state is recognized, loading is absent, and two observations 200 milliseconds
apart have the same result or empty-state signature.

A successful empty result requires an explicit, fixture-covered empty-state
element associated with the current query. An empty row list without that
element is `dom_contract_unavailable`, not `results: []`. A loading state that
does not settle is `search_timeout`. A dialog that closes unexpectedly is
`dialog_closed`. These are source diagnostics, not evidence that no matching
study exists.

The payload's `search_readiness` contains only public-safe fields:

    status: "ready" | "empty" | "timeout" | "dom_contract_unavailable"
    query_observed
    result_root_observed
    explicit_empty_observed
    loading_observed
    stable_sample_count
    elapsed_ms

It does not contain selectors, DOM excerpts, script names beyond normalized
results, or raw exception text.

## Dialog Restoration

Before opening or typing, R6 records whether the Indicators dialog is open and
the existing query text when observable. If the command opens a previously
closed dialog, it closes that same dialog after search and verifies absence. If
the dialog was already open, it leaves it open, restores the prior query using
the same input event path, and verifies the restored value. Search results are
not required to match the prior snapshot because remote content may change.

Restoration runs on success and failure after any UI action. The result uses:

    dialog_state_before: "open" | "closed" | "unknown"
    dialog_state_after: "open" | "closed" | "unknown"
    restoration_status: "restored" | "not_needed" | "failed" | "unknown"

If restoration cannot be verified, the command returns an
`internal_api_unavailable` error with public-safe details and a next action to
inspect or close the dialog manually. It must not report successful search
results while hiding failed restoration.

## Plan of Work

First complete a bounded live inventory on the dedicated test layout. Record
only element roles, semantic attributes, count ranges, localization category,
loading/empty presence, and whether open/query/close restoration succeeded.
Do not record result titles from account-local, invite-only, purchased, or
saved-script sections.

Next write the final normalized contract and fixture inventory into this plan.
The implementation plan must place I/O-free validation and normalized parser
models in `crates/model` if they do not depend on live DOM. The Desktop adapter
belongs under `crates/cli/src/ops/indicator/` if adding search would make the
existing `indicator.rs` facade materially larger. CLI parsing remains in
`crates/cli/src/cli.rs`, and dispatch validation remains in
`crates/cli/src/app/dispatch.rs`.

Define deterministic parser fixtures without copying live raw DOM. Handcraft
the smallest semantic HTML or normalized row structures needed to cover:
English and Japanese section labels, fragmented highlighted titles, optional
author labels, an explicit strategy badge, a Community result, an
account-local saved result, virtualized rendered order, explicit empty state,
loading timeout, missing result root, unexpected dialog close, initially open
query restoration, and initially closed dialog restoration.

After contract review is green, archive this R5 plan and create a separate R6
implementation ExecPlan. Do not implement the command opportunistically while
finishing this contract.

## Concrete Steps

Run all commands from the repository root. Confirm the planning baseline:

    git status --short --branch
    target/debug/tv readiness
    target/debug/tv tab list
    target/debug/tv indicator --help

Use only the dedicated test layout for live inventory. Opening and closing the
Indicators dialog is authorized for this inventory; adding/removing studies,
changing study visibility, saving scripts, or changing account state is not.
Summarize observations in this plan and restore the initial dialog/query state.

Validate this docs/contract slice with:

    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    git diff --check

Rust tests are not required unless Rust source changes. If any Rust source is
changed, run formatting, strict Clippy, the full workspace tests, and Cargo
metadata before review.

## Validation and Acceptance

R5 is complete when the current Desktop inventory distinguishes rendered
results, explicit empty, loading, and parser drift; the contract fixes command
validation, source/mutation metadata, result fields, privacy classifications,
timeout, and restoration; parser fixture requirements are self-contained; and
independent review reports no unresolved finding.

The plan must make these future R6 outcomes testable: a known built-in query
returns normalized rendered rows; an account-local row is marked as such; a
true no-result query returns successful empty only with explicit empty-state
evidence; a changed DOM returns `dom_contract_unavailable`; and both initially
open and initially closed dialog states are restored.

The search command must not add a study, invoke `createStudy`, click a result
row, choose a partial match, emit raw DOM or private identifiers, or become a
fallback for `tv indicator add`. Those prohibitions are acceptance criteria,
not optional implementation details.

## Idempotence and Recovery

Contract editing and deterministic fixtures are repeatable. Live inventory is
bounded to the dedicated test layout and must restore the dialog after every
query. If the dialog cannot be restored, stop inventory and restore it manually
before continuing. Do not delete the persistent test layout without separate
owner approval.

If localization or DOM structure differs from this plan, record the observed
semantic boundary and use `unknown` classifications. Do not add broad hashed
class selectors merely to make one live run pass. If explicit empty state
cannot be identified reliably, R6 must omit successful empty results and return
an unavailable diagnostic until a stable signal is found.

No push, tag, GitHub Release, package-version change, study mutation, Pine
save, or account mutation is authorized by this contract slice.

## Artifacts and Notes

Planning evidence:

    Released baseline: v0.26.0
    Strategy Tester compatibility closeout: fe632b3
    Upstream indicator research snapshot: 55534aa
    Existing Rust indicator search command: none
    Existing direct add path: chart createStudy plus one-new-study post-check
    Current dialog locale observed: Japanese
    Upstream parser on current dialog: zero rows without explicit empty proof
    Safe multiple-strategy exact add during R1: unavailable
    Search/add split: required
    Reusable result selector: unconfirmed and not promised

Do not add raw DOM, result titles from private/account-local sections, script
IDs, target IDs, account identities, or machine-specific filesystem paths to
this section as work proceeds.

## Interfaces and Dependencies

R5 adds no runtime interface. R6 should add this CLI shape unless live contract
evidence requires a reviewed revision:

    IndicatorCommand::Search {
        query: Vec<String>,
        limit: usize,
    }

The likely operation signature is:

    pub async fn indicator_search(
        runtime: &mut impl RuntimeEvaluator,
        query: &str,
        limit: usize,
    ) -> Result<serde_json::Value, AppError>

Use existing `RuntimeEvaluator`, `AppError`, JSON envelope, target selection,
and CDP timeout behavior. Add no dependency. Keep normalized validation/parser
types I/O-free and testable. Keep selectors and UI restoration inside the CLI
Desktop adapter. Do not reuse upstream fixed sleeps or contains-match add
behavior as contract requirements.

## Open Questions

- UNCONFIRMED: which current semantic attributes distinguish result rows,
  section headers, loading, and explicit empty state across localization.
- UNCONFIRMED: whether a stable public-safe script-kind or access-scope marker
  exists for every result class.
- UNCONFIRMED: whether an initially open dialog query can be restored through
  the same event path without disturbing category selection.
- UNCONFIRMED: whether current rendered rows expose any reusable selector safe
  enough for R7. R5 does not promise one.

Revision note (2026-07-12): created after Strategy Tester compatibility
closeout. The plan incorporates current localized dialog drift and upstream
search lessons, classifies transient dialog interaction as an explicit
Desktop-backed operation, and requires proof before treating zero rows as a
successful empty result.
