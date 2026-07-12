# Prove indicator search parser feasibility

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while work proceeds.
Maintain it according to `.agents/PLANS.md`.

## Purpose / Big Picture

The released CLI can add a study when the caller already knows a name accepted
by TradingView's chart API, but it cannot safely discover indicators and
strategies from the selected TradingView Desktop chart. The preceding contract
work found stable dialog controls but did not find trustworthy result rows,
loading state, or an explicit empty state. Treating that absence as an empty
search would be incorrect.

This R6 slice determines whether the current Desktop build exposes enough
stable, public-safe semantics to implement search later. It does not add
`tv indicator search`, publish `indicator_search.v1`, click a result, or add a
study. A successful outcome is an explicit go or no-go decision supported by
bounded live probes, deterministic fixtures when semantic anchors exist, and
verified dialog restoration. Only a go decision permits a separate R6b
implementation ExecPlan.

## Progress

- [x] (2026-07-12) Completed and independently reviewed the R5 provisional
  contract and current-build observation boundary.
- [x] (2026-07-12) Created this R6 parser-feasibility ExecPlan without adding a
  public command or runtime payload.
- [x] (2026-07-12) Reconfirmed the current Desktop and upstream baseline on the
  dedicated persistent test layout without changing a chart study, saved
  script, or account state.
- [x] (2026-07-12) Used existing unsafe-gated `tv ui eval`, native CDP typing,
  keyboard, and screenshot surfaces as the smallest disposable bounded probe;
  no production or test-only Rust helper was needed.
- [x] (2026-07-12) Exercised a known built-in query and a deliberate no-result
  query from an initially closed dialog, then verified final closed-state
  restoration.
- [x] (2026-07-12) Exercised an initially open dialog with a pre-existing query.
  Query restoration could not be verified through the same clear and CDP input
  path, so this required go condition is not met.
- [x] (2026-07-12) Found no stable QA/ARIA result-row/root or explicit-empty
  anchors and initially recorded no-go without inventing selectors.
- [x] (2026-07-12) Compared the current upstream implementation and its tests.
  Upstream uses broad class fragments, fixed sleeps, false-empty semantics, and
  has no search/add behavior tests; that quality boundary is not adopted.
- [x] (2026-07-12) Proved a strict class-free structural result parser across
  three known built-in queries. Query-matching rows and exact open-state query
  restoration were observed in a single bounded probe. A final known-result
  probe produced two equal normalized samples 200 milliseconds apart and
  restored the overall closed dialog state.
- [x] (2026-07-12) Kept Rust source unchanged. R6b must encode the proven
  structure as deterministic fixtures before exposing a command; R6 evidence
  does not need disposable production-adjacent parser code.
- [x] (2026-07-12) Revised the decision to limited go: positive rendered
  results may proceed to R6b, while no-result success remains unsupported and
  must fail closed as `dom_contract_unavailable`.
- [x] (2026-07-12) Synchronized the revised decision across the roadmap, work
  inventory, changelog, plan index, and local continuity ledger.
- [x] (2026-07-12) Ran the applicable docs-only validation. Rust source and
  Cargo metadata are unchanged, so the Rust baseline was not rerun.
- [x] (2026-07-12) Replaced the planned formal no-go review with the
  owner-requested upstream quality comparison and additional structural
  prototype. Applicable validation is green; archive R6 before creating R6b.

## Surprises & Discoveries

- Observation: R5 found stable `data-qa-id` controls for the search input,
  close action, sidebar groups, and built-in tabs, but no semantic result root,
  row, loading marker, or explicit empty marker.
  Evidence: native and programmatic input attempts both left only the dialog
  shell observable. The final state restored from closed to closed.

- Observation: the reviewed upstream search implementation can return a false
  empty result on the current dialog.
  Evidence: the upstream `55534aa` snapshot relies on broad class-name
  fragments and a fixed delay, while the current localized dialog did not
  expose rows through that parser.

- Observation: native CDP typing did cause the visible dialog to transition to
  a populated known-result view and later to a no-result view, but neither view
  exposed the semantic parser anchors required by R5.
  Evidence: the populated view had one scrollable region and 37 rendered
  absolute-positioned header/row elements. Result elements and their ancestors
  had no QA ID, ARIA role, `data-name`, or purpose-specific data attribute; only
  generated class fragments and layout structure distinguished them.

- Observation: the visible no-result view was not a machine-readable explicit
  empty state under the provisional contract.
  Evidence: four nested elements represented the localized empty message, but
  none had a QA ID, ARIA role, `data-name`, or purpose-specific state attribute.
  The only matching QA identifiers belonged to the search input and clear
  button.

- Observation: initially closed restoration succeeded, while the first
  initially open restoration attempt using clear-control plus native insertion
  did not verify.
  Evidence: the final close control returned the dialog to its initial absent
  state. This failure prompted the exact setter/input comparison below rather
  than being treated as the final restoration result.

- Observation: the initial restoration failure came from the clear-control and
  native-insert sequence, not from an inherent inability to restore an open
  dialog query.
  Evidence: a bounded prototype using the same exact HTML input setter and
  bubbling `input` event for search and restoration verified the captured prior
  query after three independent known-result cases.

- Observation: a strict class-free structural parser identified the same
  result model across three distinct built-in queries.
  Evidence: each case exposed exactly one dialog descendant whose children
  were all absolute-positioned rows and included direct-child `h3` section
  headers. Each case produced 35 rendered results with non-empty first-child
  titles; query-word matching counts were positive for every query. No CSS
  class name was read by the parser.

- Observation: upstream does not provide a quality argument for adopting its
  looser behavior.
  Evidence: `src/core/indicators.js` uses broad `scroll`, `container`, and
  `title` class fragments despite its comment, waits a fixed 1.2 seconds,
  returns zero parsed rows as successful empty, always closes the dialog, and
  permits first contains-match add. The 152 green unit tests include no
  `searchStudies` or `addStudyFromSearch` behavior test; indicator e2e covers
  toggle and input reads only.

## Decision Log

- Decision: R6 is a stop/go parser-feasibility slice and publishes no command,
  option, JSON contract, or reusable selector.
  Rationale: current live evidence proves only the conservative unavailable
  path. A public surface before parser proof would turn DOM drift into false
  empty results.
  Date/Author: 2026-07-12 / Codex

- Decision: accept stable QA identifiers, ARIA roles/states, and semantic data
  attributes as candidate anchors; reject hashed or presentation-only class
  fragments as primary identity.
  Rationale: semantic controls survived localization while the upstream
  class-fragment parser did not.
  Date/Author: 2026-07-12 / Codex

- Decision: use rendered rows only for feasibility and do not scroll the
  virtualized result list.
  Rationale: the provisional R6b contract defaults to 25 returned rows, allows
  at most 50, and permits one extra observed row for truncation. Scrolling adds
  a separate progress and termination problem that is not needed to prove the
  first parser boundary.
  Date/Author: 2026-07-12 / Codex

- Decision: never interpret an empty row array as a successful empty search.
  Rationale: a successful empty result requires an explicit state associated
  with the dispatched query. Missing rows without that state are no-go
  evidence during R6 and would be `dom_contract_unavailable` only in a later
  R6b command.
  Date/Author: 2026-07-12 / Codex

- Decision: record a limited go for positive-result search and permit a revised
  R6b ExecPlan after review.
  Rationale: the class-free structural host is strict, unique, query-sensitive,
  and repeatable across three known queries; title and section extraction do
  not require generated classes, and exact open-query restoration is proven.
  The absence of an explicit empty marker still prevents successful zero-result
  payloads, so R6b must fail closed when no structural result host appears.
  Date/Author: 2026-07-12 / Codex

- Decision: do not require R6b to expose successful `results: []` in its first
  slice.
  Rationale: no-result UI lacks a stable purpose-specific marker. Returning
  `dom_contract_unavailable` is less convenient than upstream's behavior but
  preserves the core invariant that DOM drift never becomes a false empty
  success. A later additive revision may enable empty success only after new
  current-build evidence.
  Date/Author: 2026-07-12 / Codex

## Outcomes & Retrospective

R6 feasibility is complete with a limited go decision. A strict structural
parser can identify one virtualized result host, section headers, and rendered
rows without reading generated class names. It was query-sensitive across
three known built-in searches, and exact setter/event restoration preserved an
initially open query. Upstream comparison confirms that adopting its broader
selectors, fixed sleep, false-empty success, or contains-match add would lower
this project's quality.

The first R6b must be narrower than the original provisional contract. It may
return positive rendered results after two stable samples, but it must return
`dom_contract_unavailable` when no unique structural host appears; it cannot
publish successful zero-result semantics. No Rust source, public command,
option, JSON contract, study, or account state changed during R6. The
owner-requested upstream comparison and final prototype close R6; R6b is the
next separately planned implementation slice.

## Context and Orientation

The command definitions are in `crates/cli/src/cli.rs`; the existing
`IndicatorCommand` supports `add`, `remove`, `toggle`, `set`, and `get` only.
Indicator operations are in `crates/cli/src/ops/indicator.rs` and run
JavaScript through `RuntimeEvaluator` against one selected TradingView Desktop
chart. R6 must not modify either public surface.

The reviewed provisional contract is archived at
`docs/plans/archives/tradingview-cli-indicator-search-contract.md`. It defines
the possible R6b behavior: query length at most 200 Unicode scalar values,
`--limit` default 25 and maximum 50, at most 51 rendered rows observed for
truncation, no scrolling, explicit empty-state proof, and verified restoration.
This R6 plan tests whether those rules can be implemented against the current
Desktop build; it does not silently revise them.

The existing upstream reference is the local reviewed `55534aa` snapshot of
`tradingview-mcp`. It is research evidence only. Do not copy its fixed delay,
contains-match selection, or class-fragment parser into production code.

A "semantic anchor" means an attribute intended to describe behavior or
identity, such as a stable `data-qa-id`, an ARIA role, selected state, or a
purpose-specific data attribute. A generated CSS class used only for styling is
not a semantic anchor. A "bounded probe" means a test-only operation with an
absolute deadline, fixed sample interval, fixed observation limit, and cleanup
that can be rerun without accumulating state.

## Plan of Work

First reconfirm that TradingView Desktop is ready and the dedicated persistent
test layout remains isolated from the owner's three original charts. Record
only public-safe aggregate state. Do not save scripts, add or remove studies,
change visibility, or delete the test layout.

Next use the smallest existing Desktop probe boundary. Existing unsafe-gated
`tv ui eval`, native CDP input, keyboard, and screenshot operations are
sufficient for R6 and avoid adding disposable Rust helpers. A future R6b must
add deterministic parser tests before exposing command dispatch.

The probe must use one absolute five-second readiness deadline per query and
sample no faster than every 200 milliseconds. It records whether the intended
input contains the normalized query, whether query-sensitive result titles
distinguish dispatch from value assignment, whether exactly one strict
class-free structural result host and normalized rows exist, whether loading is
present, whether an explicit empty state exists, and whether the dialog closes
unexpectedly. It may return aggregate booleans, counts capped at 51,
structural-anchor categories, and restoration status. It
must not print raw DOM, selectors containing generated class names, result
titles from account-local sections, script IDs, account identity, target IDs,
or raw JavaScript errors.

Use two public-safe query classes. The known-result query targets one built-in
indicator whose visible title can be asserted inside the live test without
printing it. The deliberate no-result query is a fixed synthetic string that
does not resemble an account-local name. Before relying on either result,
prove that the query was dispatched rather than merely assigned to the input.
If the current DOM exposes no such proof, record no-go.

Run both query classes from an initially closed dialog. The probe may open the
dialog, but must close it and verify absence afterward. Then run an initially
open case after recording its current query. Leave the dialog open, restore the
prior query through the same input path, and verify that value. If the initial
query cannot be observed safely, restoration cannot be proven and the decision
is no-go. Stop immediately on restoration failure and restore manually before
continuing.

If a limited go is found, specify the deterministic fixture work that R6b must
complete before command dispatch is added. Prefer small normalized structures
over copied HTML. Cover a known-result row, fragmented highlighted title text,
Japanese and English section labels where observed structure supports them,
optional author label, strategy badge, virtualized rendered order, the
50-return/51-observation boundary, absent result host, unexpected close, and
both restoration paths. A fixture invented without matching live structure
cannot support implementation.

Finally record one decision. Limited go requires every positive-result
acceptance condition below and a fail-closed rule for the unavailable empty
state. If any positive-result condition remains unconfirmed, record no-go and
leave R6b pending. Do not adopt upstream's false-empty behavior merely to
produce go.

## Concrete Steps

Run from the repository root. Confirm the baseline:

    git status --short --branch
    target/debug/tv readiness
    target/debug/tv tab list
    target/debug/tv indicator --help

Inspect only relevant current code and the reviewed upstream snapshot:

    rg -n "IndicatorCommand|indicator_add|createStudy|RuntimeEvaluator" crates/cli/src
    rg -n "addStudyFromSearch|indicators-dialog" ../tradingview-mcp

Run the bounded probe through existing unsafe-gated UI commands. The output may
contain only aggregate evidence such as:

    known_query_dispatched=true
    known_result_root_observed=true
    known_result_count=1
    empty_query_dispatched=true
    explicit_empty_observed=false
    initially_closed_restored=true
    initially_open_restored=true

R6 leaves Rust source unchanged. If that boundary changes during correction,
run the full baseline:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1

Always run the documentation and packaging checks:

    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    git diff --check

## Validation and Acceptance

Record limited go only if bounded current-build probes establish that known
built-in queries are demonstrably dispatched; exactly one class-free structural
result host is found; direct-child `h3` rows define sections; every result row
has a non-empty first-child title; two samples have a stable normalized
signature; no generated class fragment is required; and initially closed plus
initially open query/state restoration are verified. R6 met this positive-result
boundary across three queries.

The absence of explicit empty and loading markers narrows, rather than
invalidates, the go decision. R6b must not return successful empty results. A
query that does not produce one unique, stable structural host by the absolute
deadline returns `dom_contract_unavailable`. This fail-closed behavior also
covers unresolved loading and parser drift.

R6 is complete only after the limited-go decision and evidence are written into this
living plan, roadmap/work inventory/current-plan state agree, public hygiene is
green, and the upstream quality comparison confirms that the proposed boundary
does not merely copy weaker behavior. Those conditions are met. Create a
separate R6b implementation ExecPlan after archiving this plan.

## Idempotence and Recovery

The deterministic tests and bounded probes must be repeatable. Every live probe
captures initial dialog state before typing and restores it in a finally-style
cleanup path. Never continue to another case after restoration failure. A
manual recovery may close the dialog or restore the prior query on the
dedicated layout, but must be verified before rerunning.

Do not delete the persistent test layout without separate owner approval. Do
not add/remove studies, toggle visibility, save Pine scripts, switch the
owner's original charts, push commits, create tags, or mutate GitHub state as
part of R6.

## Artifacts and Notes

Starting evidence:

    Released baseline: v0.26.0
    R5 closeout commit: d54dac1
    Upstream research snapshot: 55534aa
    Public indicator search command: absent
    Stable dialog shell anchors: observed
    Known-result visual transition: observed
    Rendered scroll regions: 1
    Rendered absolute header/row elements: 37
    Class-free structural result host: unique across 3 known queries
    Query-sensitive title matches: observed across 3 known queries
    Generated class dependency: none in structural prototype
    Stable normalized samples: 2 at 200 ms interval
    Explicit empty semantic marker: unavailable
    Loading semantic marker: unavailable
    Initially closed restoration: verified
    Initially open query restoration: verified with exact setter/input path
    Upstream search/add behavior tests: absent
    Current decision: limited go for positive results only

Tracked evidence must remain aggregate and public-safe. Do not include raw DOM,
raw payloads, private result titles, account-local identifiers, target IDs,
credentials, or machine-specific filesystem paths.

## Interfaces and Dependencies

R6 adds no public interface and no dependency. `IndicatorCommand` and the CLI
JSON contracts remain unchanged. Test-only helpers may use existing
`RuntimeEvaluator`, Tokio support, and current CLI test infrastructure.

If stable anchors justify I/O-free fixture code, put normalized parser and
readiness types in `crates/model` only when they can be reused by a future R6b
without importing CDP or DOM concerns. Keep live selector evaluation and dialog
restoration in the CLI Desktop adapter. Do not add a general browser automation
abstraction for this one feasibility slice.

The provisional R6b interface remains documented only in the archived R5 plan.
R6 must not add `IndicatorCommand::Search`, an `indicator_search` operation, or
`indicator_search.v1`.

## Open Questions

- UNCONFIRMED: whether future Desktop builds preserve the strict class-free
  structural host and row hierarchy.
- UNCONFIRMED: whether the current dialog exposes a query-associated explicit
  empty state and a distinguishable loading state.
- UNCONFIRMED: whether category selection needs independent restoration when a
  future R6b supports explicit category filtering. R6 changed only query text.
- UNCONFIRMED: whether any response-local row identity beyond DOM order is
  stable enough to inform R7. R6 does not promise one.

Revision note (2026-07-12): created after the independently reviewed R5
contract concluded that current live evidence is insufficient for immediate
command implementation. This plan makes R6 a bounded stop/go feasibility slice
and preserves R6b as a separate, conditional implementation step.

Revision note (2026-07-12): completed bounded live feasibility using existing
unsafe-gated UI evaluation and native CDP input. Visible populated and empty
states were observed, but neither exposed the required semantic anchors and
initially open query restoration did not verify. R6 therefore records no-go,
adds no Rust fixture or public surface, and leaves R6b deferred pending future
Desktop evidence.

Revision note (2026-07-12): reopened the decision after owner feedback requested
an upstream quality comparison. Upstream's class-fragment parser and untested
false-empty behavior were rejected, but a class-free structural prototype
proved stable positive-result parsing and exact open-query restoration across
three known queries. R6 now records limited go; R6b may implement positive
results only and must fail closed when no unique result host appears.

Revision note (2026-07-12): closed R6 after the owner-requested upstream quality
comparison, three-query structural prototype, two-sample stability check, and
restoration verification. A separate R6b ExecPlan must encode deterministic
fixtures and the positive-result-only fail-closed contract before implementation.
