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
- [ ] Reconfirm the current Desktop and upstream baseline without changing a
  chart study, saved script, or account state.
- [ ] Build the smallest test-only bounded probe needed to distinguish query
  dispatch, result rows, loading, explicit empty state, and dialog closure.
- [ ] Exercise a known built-in query and a deliberate no-result query from an
  initially closed dialog, then verify closed-state restoration.
- [ ] Exercise the same bounded paths from an initially open dialog with a
  pre-existing query, then verify query and open-state restoration.
- [ ] If stable semantic anchors exist, add deterministic fixtures that cover
  localization, title fragments, virtualization, loading, empty, drift, and
  restoration. If they do not, record no-go without inventing selectors.
- [ ] Record the stop/go decision and synchronize the roadmap, work inventory,
  changelog, plan index, and local continuity ledger.
- [ ] Run focused and full validation required by the actual changed files.
- [ ] Obtain independent read-only review before archiving R6 or creating R6b.

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

## Outcomes & Retrospective

R6 planning is complete. Feasibility work and the stop/go decision remain.
Until a go is recorded, the Rust CLI has no indicator search command and the
provisional `indicator_search.v1` contract is not public.

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

Next add a test-only probe in the smallest existing Desktop test boundary. A
preferred location is a private `#[cfg(test)]` helper near
`crates/cli/src/ops/indicator.rs` plus an ignored integration test if a live
Desktop connection is required. Production command dispatch must remain
unchanged. Gate the live test behind
`TV_LIVE_INDICATOR_SEARCH_FEASIBILITY=1` so normal workspace tests remain
deterministic.

The probe must use one absolute five-second readiness deadline per query and
sample no faster than every 200 milliseconds. It records whether the intended
input contains the normalized query, whether an explicit query-dispatch signal
can be distinguished from value assignment, whether a semantic result root and
row exist, whether loading is present, whether an explicit empty state exists,
and whether the dialog closes unexpectedly. It may return aggregate booleans,
counts capped at 51, semantic-anchor categories, and restoration status. It
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

If live semantic anchors are found, encode only their normalized meaning in
I/O-free fixtures. Prefer small Rust structures over copied HTML. Cover a
known-result row, fragmented highlighted title text, Japanese and English
section labels where observed semantics support them, optional author label,
strategy badge, virtualized rendered order, the 50-return/51-observation
boundary, explicit empty state, loading timeout, missing result root,
unexpected close, and both restoration paths. A fixture invented without a
matching live semantic anchor cannot support go.

Finally record one decision. Go requires every acceptance condition below. If
any required condition remains unconfirmed after the bounded probes, record
no-go, remove disposable probe-only code that has no lasting regression value,
keep useful deterministic diagnostics only when justified, and leave R6b
pending. Do not weaken the provisional contract merely to produce go.

## Concrete Steps

Run from the repository root. Confirm the baseline:

    git status --short --branch
    target/debug/tv readiness
    target/debug/tv tab list
    target/debug/tv indicator --help

Inspect only relevant current code and the reviewed upstream snapshot:

    rg -n "IndicatorCommand|indicator_add|createStudy|RuntimeEvaluator" crates/cli/src
    rg -n "addStudyFromSearch|indicators-dialog" ../tradingview-mcp

After adding deterministic test support, run its focused tests. Use the exact
test names introduced by the implementation; the expected result is all
focused tests passing with the live test ignored by default. When Desktop and
the dedicated layout are ready, run the opt-in live probe once:

    TV_LIVE_INDICATOR_SEARCH_FEASIBILITY=1 \
      cargo test -p tradingview-cli indicator_search_feasibility -- --ignored --nocapture

The live output may contain only aggregate evidence such as:

    known_query_dispatched=true
    known_result_root_observed=true
    known_result_count=1
    empty_query_dispatched=true
    explicit_empty_observed=true
    initially_closed_restored=true
    initially_open_restored=true

If Rust source changes, run the full baseline:

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

Record go only if one bounded current-build run establishes every condition:
the known built-in query is demonstrably dispatched; a semantic result root and
at least one normalized row are observed; the deliberate no-result query is
demonstrably dispatched and exposes an explicit query-associated empty state;
loading can be distinguished from ready and empty; no generated class fragment
is required as primary identity; initially closed restoration is verified;
initially open query and open-state restoration are verified; and all
deterministic fixtures plus the applicable full baseline pass.

Record no-go if any required signal is absent, ambiguous, dependent on a broad
class fragment, or cannot be restored within the bounded operation. No-go is a
valid completion: it adds no CLI command, publishes no JSON contract, performs
no chart-study mutation, and records which semantic evidence remains missing.

R6 is complete only after the decision and evidence are written into this
living plan, roadmap/work inventory/current-plan state agree, public hygiene is
green, and independent review has no unresolved finding. Create an R6b
implementation ExecPlan only after reviewed go. A reviewed no-go leaves R6b
deferred and advances to the next independent roadmap item.

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
    Result root and rows: unconfirmed
    Loading and explicit empty state: unconfirmed
    Initially closed restoration: observed in R5
    Initially open restoration: unconfirmed
    Current decision: pending bounded feasibility probes

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

- UNCONFIRMED: which current semantic attributes identify the result root and
  rows.
- UNCONFIRMED: which current signal proves that TradingView processed a query
  rather than only reflecting the input value.
- UNCONFIRMED: whether the current dialog exposes a query-associated explicit
  empty state and a distinguishable loading state.
- UNCONFIRMED: whether an initially open query can be restored through the same
  event path without changing category selection.
- UNCONFIRMED: whether any response-local row identity beyond DOM order is
  stable enough to inform R7. R6 does not promise one.

Revision note (2026-07-12): created after the independently reviewed R5
contract concluded that current live evidence is insufficient for immediate
command implementation. This plan makes R6 a bounded stop/go feasibility slice
and preserves R6b as a separate, conditional implementation step.
