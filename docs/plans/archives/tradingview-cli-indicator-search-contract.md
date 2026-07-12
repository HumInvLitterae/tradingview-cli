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
runtime payload is added in this contract slice. Current inventory did not
establish result semantics, so R6 is a stop/go parser feasibility spike rather
than command implementation. Only a green feasibility result may promote the
provisional command contract into a later implementation slice.

## Progress

- [x] (2026-07-12) Completed and archived Strategy Tester compatibility,
  including final independent review and the closeout commit.
- [x] (2026-07-12) Inspected the current Rust indicator lifecycle, the current
  localized Indicators dialog evidence, and upstream search/add code at the
  reviewed `55534aa` snapshot.
- [x] (2026-07-12) Created this R5 contract ExecPlan and fixed the first-slice
  boundary: read-oriented search only, with no study add or partial-match
  selection.
- [x] (2026-07-12) Completed the bounded current-build inventory on the
  dedicated test layout. Stable searchbox, close, sidebar, and built-in tab
  semantics plus closed-state restoration were observed. Result rows, result
  root, loading, and explicit empty semantics were not exposed and remain
  unconfirmed; that observation limit is the inventory result, not proof of an
  empty search result.
- [x] (2026-07-12) Defined the provisional go-path field types, count rules,
  failure mappings, limits, timeout, and explicit empty-state requirement.
- [x] (2026-07-12) Defined deterministic parser fixtures for the R6 feasibility
  spike, including localization,
  virtualization, loading, empty, drift, and restoration cases.
- [x] (2026-07-12) Recorded the current Desktop result surface as no-go evidence
  without changing chart studies or account data.
- [x] (2026-07-12) Synchronized the roadmap, work inventory, plan index,
  changelog, and local continuity ledger with the feasibility boundary.
- [x] (2026-07-12) Completed independent review and two focused correction
  rounds. No unresolved finding remains. Archive this plan before creating the
  R6 feasibility ExecPlan.

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

- Observation: the current dialog exposes stable semantic QA identifiers for
  the search input, close control, sidebar groups, and category items. Built-in
  subcategories expose `role="tab"`, selected state, and localized tooltip
  labels.
  Evidence: the bounded inventory observed one searchbox, one close control,
  stable sidebar QA identifiers, and four built-in tabs without recording any
  private result title or raw DOM.

- Observation: no result surface appeared for native input, programmatic
  input, an empty built-in category, or the built-in strategy tab. No loading
  marker or explicit empty marker appeared either.
  Evidence: each bounded sample contained the dialog/sidebar semantics but no
  result-root role, result-row QA identifier, scrollable result region,
  loading state, or empty-state semantics. The contract classifies this as
  `dom_contract_unavailable`, not a successful zero-result search.

- Observation: closing through the stable close QA control restored the
  initially closed dialog state.
  Evidence: the post-close check found no dialog. No study, visibility, script,
  or account state was changed.

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

- Decision: allow stable `data-qa-id`, ARIA role/state, and semantic attributes
  as parser anchors; reject hashed class fragments as primary identity.
  Rationale: current QA identifiers survived localization, while the upstream
  class-fragment parser returned a false empty result. Fixtures must prove every
  accepted semantic anchor.
  Date/Author: 2026-07-12 / Codex

## Outcomes & Retrospective

R5 contract drafting is complete, but live result semantics are unconfirmed.
The command boundary, source classification, empty-result rule, privacy
posture, restoration behavior, provisional wire schema, failure mapping, and
separation from exact-add are fixed. Current Desktop evidence is a no-go for
immediate command implementation because it establishes only the conservative
unavailable path, not a working result-row parser. Independent review is green,
so R5 closes with R6 reclassified as feasibility.

The expected R6 outcome is a stop/go decision backed by bounded probes and
deterministic parser fixtures. A go decision requires trustworthy result-row
and empty-state semantics plus restoration evidence; only then may R6b
implement a command. A no-go decision records the current unsupported state
and leaves the CLI unchanged.

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
neither R6 feasibility nor a later R6b command may make search a hidden
fallback inside add.

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
present in the DOM simultaneously. The provisional contract returns rendered
rows only and does not scroll: the default return limit is 25, `--limit` may
raise the return limit to 50, and the parser may observe at most 51 rows solely
to determine rendered-row truncation. R6 tests whether that contract is
feasible; it does not redefine the paging policy. Bounded scrolling can be
proposed later only with stable progress and termination evidence.

## Provisional Go-Path Contract

The future CLI surface is:

    tv indicator search <QUERY> [--limit <N>]

`QUERY` is required after trimming, may contain at most 200 Unicode scalar
values, and must be validated before CDP connection. `--limit` defaults to 25
and accepts `1..=50`. The return limit is 50 results. The parser observation
limit is 51 rendered rows so it can inspect one row beyond the maximum return
limit for truncation evidence. The implementation does not scroll the
virtualized list.

This schema is provisional until R6 produces a go decision. A no-go decision
adds no command and therefore publishes no `indicator_search.v1` contract. If
go is established, the success payload uses
`contract_version: "indicator_search.v1"` and these exact JSON types:

    query: string
    result_count: integer
    observed_rendered_count: integer
    result_limit: integer
    result_scope: "rendered_rows"
    rendered_rows_truncated: boolean
    results: array
    dialog_state_before: "open" | "closed" | "unknown"
    dialog_state_after: "open" | "closed" | "unknown"
    restoration_status: "restored" | "not_needed"
    search_readiness: object
    source: "indicators_dialog_dom"
    source_category: "desktop_backed_operation"
    requires_desktop: true
    non_mutating: false
    operation: "indicator_search"

`query` is the trimmed request. `result_count` equals `results.length` after
the requested limit. The parser observes at most
`min(result_limit + 1, 51)` rendered rows. `observed_rendered_count` is that
parsed observation count and is therefore at most 51. The one extra row exists
solely to establish truncation. `rendered_rows_truncated` is true exactly when
`observed_rendered_count > result_limit`; it says nothing about unrendered rows
in the virtualized list. The results array contains at most `result_limit`
entries and therefore never more than 50.

Each result preserves current DOM order and has these exact fields:

    result_index: integer
    title: string
    section_label: string | null
    section_kind: enum string
    script_kind: enum string
    author_label: string | null
    access_scope: enum string
    classification_status: "observed" | "partial"

`result_index` is zero-based within the returned array. `title` is trimmed and
must be non-empty or the row is rejected as malformed. Optional labels are
trimmed non-empty strings or JSON null; empty strings are never returned.

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

The provisional contract does not expose a reusable selection token. `result_index`
is explicitly scoped to this response and is not accepted by `tv indicator
add`. R7 must define its own exact-match mutation request after R6 proves which
stable result identity is available.

## Search Readiness and Empty Results

R6 must verify that the provisional readiness rule is feasible within a
five-second absolute deadline: the intended dialog target exists, the intended
input contains the normalized query, a result container or an explicit empty
state is recognized, loading is absent, and two observations 200 milliseconds
apart have the same result or empty-state signature. After a go decision, R6b
uses this rule in the public command.

A successful empty result requires an explicit, fixture-covered empty-state
element associated with the current query. An empty row list without that
element is `dom_contract_unavailable`, not `results: []`. A loading state that
does not settle is `search_timeout`. A dialog that closes unexpectedly is
`dialog_closed`. These are source diagnostics, not evidence that no matching
study exists.

On success, or inside search-specific error details, `search_readiness`
contains these exact fields:

    status: "ready" | "empty" | "search_timeout" | "dom_contract_unavailable" | "dialog_closed"
    observed_query_matches: boolean
    result_root_observed: boolean
    explicit_empty_observed: boolean
    loading_observed: boolean
    stable_sample_count: integer
    elapsed_ms: integer

`loading_observed` means loading was seen at any sample, not that it remains
active at return. `stable_sample_count` is zero, one, or two. `elapsed_ms` is a
non-negative integer measured from the first dialog operation.

It does not contain selectors, DOM excerpts, script names beyond normalized
results, or raw exception text.

## Failure Envelope

Validation errors occur before CDP connection. Empty or overlong query and a
limit outside `1..=50` use `ErrorKind::Validation`, existing `error.kind:
"validation"`, and exit code 1. Existing connection and target-selection
errors retain their current envelopes and exit codes because no dialog action
has started.

After a dialog action, every search-specific failure uses the existing JSON
error envelope. Its public `error.details` contains:

    diagnostic_code: enum string
    contract_version: "indicator_search.v1-candidate"
    query: string
    source: "indicators_dialog_dom"
    source_category: "desktop_backed_operation"
    requires_desktop: true
    non_mutating: false
    operation: "indicator_search"
    dialog_state_before: "open" | "closed" | "unknown"
    dialog_state_after: "open" | "closed" | "unknown"
    restoration_status: "restored" | "not_needed" | "failed" | "unknown"
    search_readiness: object
    prior_diagnostic_code: string | null
    next_action_hint: string

The readiness and failure mapping is exhaustive:

| Readiness or restoration outcome | Envelope | `ErrorKind` | Public kind / code | Exit |
| --- | --- | --- | --- | --- |
| `ready` | success | none | none | 0 |
| `empty` with explicit evidence | success | none | none | 0 |
| `dom_contract_unavailable` | error | `InternalApiUnavailable` | `internal_api_unavailable` / `dom_contract_unavailable` | 3 |
| `dialog_closed` | error | `InternalApiUnavailable` | `internal_api_unavailable` / `dialog_closed` | 3 |
| `search_timeout` | error | `Timeout` | `timeout` / `search_timeout` | 4 |
| restoration failed | error | `InternalApiUnavailable` | `internal_api_unavailable` / `restoration_failed` | 3 |

Success payloads use `search_readiness.status: "ready" | "empty"`. Search
errors retain the observed failure status in `search_readiness`. Restoration
failure does not add a readiness enum value; it is represented by
`restoration_status: "failed"` and the primary `diagnostic_code`.

Restoration failure always becomes the primary error, even if parsing or
timeout failed first. It uses `ErrorKind::InternalApiUnavailable`, exit code 3,
and `diagnostic_code: "restoration_failed"`. The pre-restoration failure is
retained only as `prior_diagnostic_code`. Successful result rows and titles are
not copied into error details. If the dialog closes unexpectedly and cannot be
restored, `restoration_failed` is primary and `prior_diagnostic_code` is
`dialog_closed`.

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

Next write the provisional normalized contract and fixture inventory into this
plan. R6 should place I/O-free normalized parser models in `crates/model` if
the feasibility spike needs executable parser fixtures. It must not add CLI
parsing or dispatch. After a go decision, an R6b implementation should place
the Desktop adapter under `crates/cli/src/ops/indicator/` if adding search would
make the existing `indicator.rs` facade materially larger; CLI parsing remains
in `crates/cli/src/cli.rs`, and dispatch validation remains in
`crates/cli/src/app/dispatch.rs`.

Define deterministic parser fixtures without copying live raw DOM. Handcraft
the smallest semantic HTML or normalized row structures needed to cover:
English and Japanese section labels, fragmented highlighted titles, optional
author labels, an explicit strategy badge, a Community result, an
account-local saved result, virtualized rendered order, explicit empty state,
loading timeout, missing result root, unexpected dialog close, initially open
query restoration, and initially closed dialog restoration.

After contract review is green, archive this R5 plan and create a separate R6
parser-feasibility ExecPlan. R6 adds no public command. It performs bounded
semantic probes and deterministic parser work, then records a stop/go decision.
Only a go decision may create an R6b implementation ExecPlan. Do not implement
the command opportunistically while finishing this contract or feasibility
work.

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

R5 is complete when it records the current observation limit without
misclassifying absent rows as an empty result; fixes the provisional command
validation, source/mutation metadata, field types and nullability, count and
truncation rules, privacy classifications, timeout, failure precedence, and
restoration rules; defines self-contained parser fixtures and R6 stop/go
criteria; synchronizes current project documents; and independent review
reports no unresolved finding. R5 does not require live result-row, loading,
or explicit-empty semantics that the current Desktop build did not expose.

R6 produces a go decision only when bounded current-build probes establish all
of the following: a known built-in query produces a stable semantic result
root and normalized rendered row; query dispatch is distinguishable from input
value assignment; a deliberate no-result query exposes a stable explicit empty
state; initially open and initially closed dialog states both restore; and the
deterministic localization, loading, empty, drift, virtualization, and
restoration fixtures pass. The accepted anchors must be public-safe semantic
attributes rather than hashed class fragments.

If those conditions cannot be established within the bounded feasibility
work, R6 records no-go, adds no command, and leaves R6b deferred. After a go,
R6b must make these implementation outcomes testable: a known built-in query
returns normalized rendered rows; an account-local row is marked as such; a
true no-result query succeeds only with explicit empty-state evidence; a
changed DOM returns `dom_contract_unavailable`; and both initially open and
initially closed dialog states are restored.

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
cannot be identified reliably, R6 records no-go and adds no command. After a
go decision, an R6b command must omit successful empty results and return
`dom_contract_unavailable` when the required runtime evidence is absent.

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
    Stable dialog anchors: searchbox, close control, sidebar QA IDs, ARIA tabs
    Result root/rows: not observed in bounded current-build inventory
    Loading/explicit empty marker: not observed
    Current-build classification: dom_contract_unavailable
    Initial dialog state: closed; final state: closed; restoration verified

Do not add raw DOM, result titles from private/account-local sections, script
IDs, target IDs, account identities, or machine-specific filesystem paths to
this section as work proceeds.

## Interfaces and Dependencies

R5 and R6 add no runtime interface. R6 is a parser-feasibility spike and must
not publish `indicator_search.v1`. Only an R6 go decision may create R6b, whose
provisional CLI shape is:

    IndicatorCommand::Search {
        query: Vec<String>,
        limit: usize,
    }

The provisional R6b operation signature is:

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
  loading, and explicit empty state. Current inventory found none.
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

Revision note (2026-07-12): completed the bounded localized-dialog inventory.
Stable QA and ARIA controls plus successful close restoration were observed,
but no result root, loading marker, or explicit empty marker appeared. The
contract therefore requires R6 to record no-go for this shape and begin with
deterministic parser fixtures rather than claiming a working live result
parser. Only a later R6b command may return `dom_contract_unavailable` at
runtime after feasibility is green.

Revision note (2026-07-12): corrected the contract after independent review.
R5 now records the current-build no-go evidence instead of requiring
unobserved result semantics for completion. R6 is a stop/go parser-feasibility
slice with no public CLI; R6b implementation exists only after a documented go
decision. The provisional success schema now fixes field types, nullability,
rendered-row count/truncation semantics, and response-local indexing, while the
failure contract maps every readiness state and restoration precedence to an
existing error kind and exit code.

Revision note (2026-07-12): completed focused re-review after resolving the
50-result/51-observation boundary and removing the remaining R6 implementation
wording. R5 is complete and ready to archive; R6 is the next stop/go
parser-feasibility plan.
