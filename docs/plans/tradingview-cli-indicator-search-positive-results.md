# Add fail-closed indicator search for positive results

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while work proceeds.
Maintain it according to `.agents/PLANS.md`.

## Purpose / Big Picture

Users who know only part of an indicator or strategy title cannot currently
discover the matching rendered entries through the Rust CLI. R6 proved that the
current Indicators dialog has a repeatable class-free row structure for
positive results, but it did not expose a trustworthy explicit empty or loading
state.

This R6b slice adds `tv indicator search <QUERY> [--limit <N>]` for positive
rendered results only. A known-result query returns normalized rows without
clicking or adding anything. A query that does not produce one unique stable
result host fails with `dom_contract_unavailable`; it never returns a false
successful empty array. The command restores the original dialog open/closed
state and query, or fails if restoration cannot be verified.

## Progress

- [x] (2026-07-12) Completed R5 contract work and R6 feasibility with upstream
  quality comparison and a three-query class-free structural prototype.
- [x] (2026-07-12) Created this R6b implementation ExecPlan with positive-result
  success and fail-closed no-result semantics.
- [ ] Add pre-connection query and limit validation plus I/O-free normalized
  result/readiness models.
- [ ] Add deterministic structural parser and restoration fixtures without raw
  live DOM or generated class names.
- [ ] Implement the Desktop adapter with an absolute readiness deadline, two
  stable samples, and finally-style restoration.
- [ ] Add `IndicatorCommand::Search`, dispatch, help, success payload, and
  public-safe error details without changing existing indicator commands.
- [ ] Add focused unit and CLI contract tests, including zero-result and DOM
  drift failure paths.
- [ ] Run one bounded public-safe live smoke on the dedicated test layout and
  restore its initial dialog state.
- [ ] Update README, stable workflow docs, packaged agent guidance, and focused
  runtime-skill references without overloading core workflows.
- [ ] Run the full Rust, docs, packaging, and hygiene validation baseline.
- [ ] Obtain independent review before commit or R7 exact-add planning.

## Surprises & Discoveries

- Observation: upstream's current search implementation is not an acceptable
  quality baseline for this project.
  Evidence: it uses broad class fragments and fixed sleeps, treats zero parsed
  rows as successful empty, always closes the dialog, and has no search/add
  behavior tests despite 152 passing unit tests overall.

- Observation: current positive results have a strict class-free structure.
  Evidence: three distinct built-in queries each exposed exactly one dialog
  descendant with all absolute-positioned children and direct-child `h3`
  section headers. Every result row had a non-empty first-child title, and
  query-word matches were positive.

- Observation: current no-result UI is visible but not safely machine-readable
  as explicit empty.
  Evidence: neither the localized message nor its illustration has a
  purpose-specific QA ID, ARIA state, `data-name`, or live-region marker.

## Decision Log

- Decision: success requires at least one normalized rendered result.
  Rationale: successful zero results cannot currently be distinguished from
  loading, parser drift, or a changed dialog structure.
  Date/Author: 2026-07-12 / Codex

- Decision: prohibit CSS class selectors in the result parser.
  Rationale: generated class fragments change across TradingView builds and are
  the primary weakness of the upstream implementation.
  Date/Author: 2026-07-12 / Codex

- Decision: identify the result host structurally and fail on ambiguity.
  Rationale: the proven host is the only dialog descendant with at least two
  children, all direct children positioned absolutely, and at least one direct
  child containing a direct `h3` section header. Zero or multiple candidates are
  unavailable, not empty.
  Date/Author: 2026-07-12 / Codex

- Decision: retain rendered-only scope with default 25, maximum 50 returned
  rows, and at most 51 observed rows.
  Rationale: the extra observed row proves truncation without adding virtualized
  scrolling or its own progress/termination contract.
  Date/Author: 2026-07-12 / Codex

## Outcomes & Retrospective

R6b planning is complete. Implementation, live confirmation, docs/skills
synchronization, and independent review remain.

## Context and Orientation

CLI command definitions live in `crates/cli/src/cli.rs`, and indicator dispatch
lives in `crates/cli/src/app/dispatch.rs`. Existing indicator operations are in
`crates/cli/src/ops/indicator.rs`; split search into
`crates/cli/src/ops/indicator/search.rs` with a small facade if adding it to the
current file would obscure add/remove/toggle/set behavior.

I/O-free validation and normalized payload shaping belong in `crates/model`.
The Desktop adapter may execute JavaScript through the existing
`RuntimeEvaluator`, but selector evaluation and UI restoration must not leak
into the model crate. Shared JSON envelopes and `AppError`/`ErrorKind` remain in
`tradingview-core`.

The archived R5 contract is
`docs/plans/archives/tradingview-cli-indicator-search-contract.md`. R6 evidence
is in
`docs/plans/archives/tradingview-cli-indicator-search-parser-feasibility.md`.
This plan narrows the earlier provisional contract: positive results may
succeed, while empty success is deferred.

## Contract

Add this command:

    tv indicator search <QUERY> [--limit <N>]

Join positional query words with one space and trim the result. Reject an empty
query or more than 200 Unicode scalar values before CDP connection. `--limit`
defaults to 25 and accepts `1..=50`.

Positive success uses `contract_version: "indicator_search.v1"` and contains:

    query: string
    result_count: integer
    observed_rendered_count: integer
    result_limit: integer
    result_scope: "rendered_rows"
    rendered_rows_truncated: boolean
    results: array
    dialog_state_before: "open" | "closed"
    dialog_state_after: "open" | "closed"
    restoration_status: "restored"
    search_readiness: object
    source: "indicators_dialog_dom"
    source_category: "desktop_backed_operation"
    requires_desktop: true
    non_mutating: false
    operation: "indicator_search"

`result_count` equals `results.length`. Observe at most
`min(result_limit + 1, 51)` rows and return at most `result_limit`.
`rendered_rows_truncated` is true only when the observed count exceeds the
return limit. It does not imply anything about unrendered virtualized rows.

Each result is response-ordered and contains:

    result_index: zero-based integer
    title: non-empty string
    section_label: non-empty string | null
    section_kind: enum string
    script_kind: enum string
    author_label: non-empty string | null
    access_scope: enum string
    classification_status: "observed" | "partial"

Use the R5 enums for section, script kind, and access scope. Return `unknown`
and `partial` when current structure does not prove a classification. Never
infer strategy kind from title text. `result_index` is response-local and is
not accepted by `tv indicator add`.

`search_readiness` on success is:

    status: "ready"
    observed_query_matches: true
    result_root_observed: true
    explicit_empty_observed: false
    loading_observed: false
    stable_sample_count: 2
    elapsed_ms: non-negative integer

No-result, unresolved loading, no/multiple structural host, malformed rows,
unexpected dialog close, or unstable samples use the normal JSON error envelope
with `ErrorKind::InternalApiUnavailable`, public
`error.kind: "internal_api_unavailable"`, exit code 3, and
`diagnostic_code: "dom_contract_unavailable"`, except a five-second absolute
deadline uses `ErrorKind::Timeout`, `error.kind: "timeout"`, exit code 4, and
`diagnostic_code: "search_timeout"`.

Error details retain query, source metadata, dialog state, restoration status,
public readiness booleans/counters, and a next action. They contain no result
titles, raw DOM, generated classes, selectors, raw JavaScript errors, target
IDs, account-local IDs, or credentials. Restoration failure overrides the
search error with `diagnostic_code: "restoration_failed"` and preserves the
prior diagnostic in `prior_diagnostic_code`.

## Structural Parser

Within `[data-name="indicators-dialog"]`, find candidate `div` elements whose
direct child count is at least two, every direct child has computed
`position: absolute`, and at least one direct child contains a direct-child
`h3`. Exactly one candidate is required. Do not read `className` and do not use
`querySelector` with a class selector.

Walk direct children in DOM order. A child with direct `h3` is a section header;
store its trimmed text. Other children are result rows. The trimmed text of the
row's first element child is the title and must be non-empty. Later direct child
text may provide author/source evidence only when deterministic fixtures prove
the position for that row class; otherwise return null. Reject malformed rows
rather than silently skipping them, because skipping could turn parser drift
into incomplete success.

Take normalized samples 200 milliseconds apart. Both must have the same query,
section/result order, titles, and observed count. At least one returned title
must contain every whitespace-separated query word case-insensitively; this is
dispatch evidence, not a result filter. If the signature differs, continue
within the same absolute deadline. Never reset the deadline inside the loop.

## Dialog Restoration

Before UI action, record whether the dialog is open and its query when open. If
closed, open it through `[data-name="open-indicators-dialog"]`; after search,
click the stable close QA control and verify absence. If already open, restore
the exact prior query with the same prototype setter and bubbling `input` event
used for search, wait for the input value to match, and leave the dialog open.

Restoration runs after success and failure. Do not return successful results if
restoration fails. The command must not click a result, add/remove/toggle a
study, change a category, save a script, or alter account state.

## Plan of Work

First add I/O-free request validation, normalized row models, count/truncation
logic, readiness shaping, and exhaustive fixtures. Include fixtures for two
sections, highlighted title fragments normalized into first-child text,
optional author, malformed empty title, zero/multiple host diagnostics,
50-return/51-observation truncation, unstable samples, and restoration
precedence.

Then add the Desktop adapter and facade. Keep the JavaScript expression small,
class-free, and limited to normalized public-safe values. Rust performs final
validation and payload shaping. Add the CLI variant and dispatch only after the
parser tests are green.

Finally update help, README command examples, source taxonomy, observation
workflow docs, development docs, packaged agent guidance, and focused
`chart-analysis`/`market-data-interpretation` references. Keep detailed edge
cases in a small reference rather than expanding core skill workflows.

## Concrete Steps

Run focused tests as implementation lands:

    cargo test -p tradingview-model indicator_search -- --nocapture
    cargo test -p tradingview-cli indicator_search -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop indicator -- --nocapture

Run the full baseline:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    git diff --check

For the opt-in live smoke, use the dedicated test target and record only
aggregate evidence. Run one known positive query and one deliberate no-result
query. Positive search must return results and restore state. No-result must
exit with the documented fail-closed diagnostic and restore state. Do not add
raw output or private titles to tracked files.

## Validation and Acceptance

The command is accepted when deterministic tests prove class-free structural
selection, row parsing, count/truncation, stable samples, fail-closed no-result,
and restoration precedence; CLI tests prove pre-connection validation and the
wire contract; all existing indicator commands remain unchanged; the bounded
live smoke proves one positive success and one no-result diagnostic with state
restored; docs and skills describe the narrower behavior; and the complete
baseline is green.

Search must never produce successful `results: []`, use generated class
selectors, scroll the virtualized list, choose or click a result, call
`createStudy`, become a fallback inside `tv indicator add`, or expose private
identifiers/raw DOM.

## Idempotence and Recovery

Tests are deterministic and repeatable. Live smoke uses only the persistent
owner-approved test layout and restores dialog state in finally-style cleanup.
Stop after restoration failure and restore manually before retrying. Do not
delete the test layout without separate owner approval.

No push, tag, release, dependency update, Pine save, study mutation, or account
mutation is authorized in this slice.

## Artifacts and Notes

Starting evidence:

    Released baseline: v0.26.0
    R5 closeout: d54dac1
    R6 closeout: 3df2a3b
    Upstream snapshot: 55534aa
    Known queries structurally parsed: 3
    Known query rendered results per probe: 35
    Stable samples: 2 at 200 ms
    Explicit empty marker: unavailable
    Initial open/closed restoration: verified in R6
    Public indicator search command: absent

## Interfaces and Dependencies

Add `IndicatorCommand::Search { query: Vec<String>, limit: usize }`. Add an
I/O-free request/result module in `tradingview-model` and a Desktop operation
such as:

    pub async fn indicator_search(
        runtime: &mut impl RuntimeEvaluator,
        query: &str,
        limit: usize,
    ) -> Result<serde_json::Value, AppError>

Use only existing dependencies, `RuntimeEvaluator`, `AppError`, and JSON
envelopes. Do not add a generic DOM framework or import CDP into the model
crate.

## Open Questions

- UNCONFIRMED: whether author/source child positions are stable for every
  rendered row class. Return null unless fixtures and live evidence agree.
- UNCONFIRMED: whether script-kind/access-scope badges have stable structural or
  semantic positions. Return unknown rather than infer.
- UNCONFIRMED: when TradingView will expose an explicit empty/loading marker.
  Successful empty remains deferred until then.

Revision note (2026-07-12): created after R6 rejected upstream's weak
class-fragment and false-empty behavior but proved a strict class-free
positive-result parser. The first implementation intentionally fails closed for
all no-host cases and leaves search-result mutation to R7.
