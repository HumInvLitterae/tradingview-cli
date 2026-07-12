# Harden current-build Strategy Tester reads

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while work proceeds.
Maintain it according to `.agents/PLANS.md`.

## Purpose / Big Picture

Users already rely on `tv data strategy`, `tv data trades`, and
`tv data equity` for structured evidence from the strategy applied to the
selected TradingView Desktop chart. Those commands must not miss a current
strategy because an old metadata flag changed, and they must not silently read
the wrong strategy when several studies are present.

This work first establishes a public-safe compatibility matrix against the
current Desktop build. It then hardens strategy candidate detection, chooses a
report-bearing strategy only when the choice is explainable, and adds enough
context for callers to understand what was read. The existing commands and
their practical metrics, trades, and equity fields remain available.

The visible proof is that a prepared chart containing a strategy returns the
same identified strategy across all three commands, while missing, hidden,
unready, and ambiguous states produce explicit diagnostics rather than an
arbitrary empty result. The default commands do not silently open Strategy
Tester or change study visibility.

## Progress

- [x] (2026-07-12) Reviewed the post-`4795784` upstream compatibility changes
  and compared them with current Rust strategy selection.
- [x] (2026-07-12) Confirmed the current helper first matches a
  `StrategyScript` ID and still falls back to
  `is_price_study === false` plus report-like methods.
- [x] (2026-07-12) Created this first v0.27 ExecPlan and fixed the initial
  boundary: diagnose current state before adding any panel or visibility
  mutation.
- [x] (2026-07-12) Built the current CLI, inspected all three existing chart
  targets without mutation, recorded the zero-count/no-fixture baseline, and
  passed four strategy unit tests plus the grouped data-command connection
  contract test.
- [x] (2026-07-12) Preserved the three existing chart targets, created an
  owner-approved dedicated test layout, added one exact built-in strategy, and
  recorded visible, hidden, panel-open, and panel-closed states with only
  public-safe summaries. Multiple-strategy state remains `UNCONFIRMED` because
  current dialog search could not produce a unique exact second result.
- [x] (2026-07-12) Updated `Surprises & Discoveries` and `Decision Log` with the matrix and
  finalize candidate-selection rules before editing production behavior.
- [x] (2026-07-12) Hardened shared candidate detection and report selection for strategy,
  trades, and equity reads.
- [x] (2026-07-12) Added additive strategy context and public-safe unavailable/ambiguous
  diagnostics without breaking existing practical fields.
- [x] (2026-07-12) Added deterministic focused tests and completed the final
  visible, hidden, restored, and no-strategy Desktop smokes.
- [x] (2026-07-12) Synchronized stable docs, packaged agent guidance,
  and the `strategy-report` and `chart-analysis` runtime skills.
- [x] (2026-07-12) Ran the complete workspace, hygiene, skill, and packaging
  baseline; all required checks passed.
- [x] (2026-07-12) Received the first independent review, corrected all four
  findings, reran focused live smokes and the complete baseline, and recorded
  the correction architecture and evidence.
- [x] (2026-07-12) Corrected the focused re-review findings by sharing the
  `id` / `entityId` resolver contract, normalizing inspection exceptions into
  nonterminal read payloads, updating orientation, and adding end-to-end
  handoff and failure tests. Focused, live, and full validation passed again.
- [x] (2026-07-12) Completed final independent re-review with no remaining
  findings. This plan is ready to archive before promoting the next v0.27 work
  item.

## Surprises & Discoveries

- Observation: the pre-change Rust code already read several modern report shapes,
  including `_reportData.performance`, `_reportData.trades`, and
  `reportData()`, but candidate selection can reject a current strategy before
  those readers run.
  Evidence: the original `crates/cli/src/ops/data/strategy.rs` implemented the
  report readers after `__findStrategy`, whose fallback required
  `is_price_study === false`.

- Observation: upstream current-build work reports that a strategy may expose
  `isTVScriptStrategy` or `is_strategy` and may have
  `is_price_study === true`.
  Evidence: the reviewed upstream changes are `653c273` and `51384e1` in the
  local research boundary `4795784..55534aa`. This is design evidence, not yet
  proof of failure in the released Rust binary.

- Observation: all three existing chart targets returned zero strategy
  evidence in the read-only baseline. Metrics and trades fell back to the DOM
  and reported unavailable state, while equity returned zero points from the
  internal API without an error field.
  Evidence: the public-safe aggregate run recorded `metric_count: 0`,
  `trade_count: 0`, and `data_points: 0` for each target. Current payloads do
  not reveal whether equity selected a false-positive candidate or merely
  reached a different no-data path.

- Observation: filtering `cli_contract_desktop` by `strategy` runs only the
  Strategy Tester screenshot test. The three data commands are covered by
  `data_read_commands_attempt_connection_when_cdp_is_unavailable`.
  Evidence: the first filtered run executed one screenshot test; source
  inspection located the data commands in the grouped connection test.

- Observation: a single visible current-build strategy exposed both the
  `StrategyScript` ID prefix and `isTVScriptStrategy: true`, while
  `is_price_study` was also true. Its structured report remained readable with
  Strategy Tester visibly open and closed.
  Evidence: all three commands selected one candidate and returned nonzero
  public-safe counts in both panel states. The existing panel DOM selector did
  not reliably distinguish the visibly open panel, so panel status remains
  `unknown` rather than guessed.

- Observation: hiding that strategy removed usable report containers while
  leaving the strategy metadata candidate present. Before the fix, metrics and
  trades reported DOM fallback errors while equity returned an unexplained
  empty internal result.
  Evidence: the final smoke returned `strategy_hidden`, zero counts,
  `report_available: false`, and the same explicit next action for all three
  commands; visibility was restored and verified immediately afterward.

- Observation: the current Japanese Indicators dialog can expose a changed
  DOM and locale shape that the upstream search parser reads as zero results.
  Exact direct-add attempts for a second built-in strategy did not create a
  study.
  Evidence: no second study was created and the dialog was closed. The
  multiple-strategy matrix row is therefore `UNCONFIRMED`; no partial match or
  guessed script identifier was used.

- Observation: retaining the old `is_price_study === false` plus generic
  report-method fallback classified ordinary studies as candidates on a chart
  with no strategy.
  Evidence: after removing that heuristic, the same chart returned zero
  candidates and `not_found`, while the dedicated strategy fixture still
  returned one candidate through two explicit signals.

- Observation: splitting candidate inspection from report reading introduced
  two handoff risks that the first correction tests did not cover: an
  `entityId`-only source could not be re-resolved, and chart/model inspection
  exceptions escaped as terminal `AppError`s.
  Evidence: focused re-review identified both paths. The final reader resolver
  checks `id` and `entityId`, while the candidate expression returns a typed
  public-safe inspection failure that all three read kinds normalize into
  their existing success-envelope style.

## Decision Log

- Decision: treat this as correctness hardening for existing commands rather
  than a richer-report feature.
  Rationale: selecting no strategy or the wrong strategy invalidates every
  downstream metric, trade, and equity interpretation. Correct identification
  must precede additional evidence fields.
  Date/Author: 2026-07-12 / Codex

- Decision: run a bounded live matrix before changing strategy readiness or
  mutation behavior.
  Rationale: static comparison strongly suggests compatibility gaps, but it
  does not establish which combinations fail on the current Desktop build.
  Mutation semantics should be based on observed need.
  Date/Author: 2026-07-12 / Codex

- Decision: do not silently open Strategy Tester or unhide a strategy.
  Rationale: panel state and study visibility are selected-chart mutations.
  Existing `tv data ...` commands are reads, and hidden state may be deliberate.
  Any ensure-ready workflow requires explicit user intent, post-check, and
  restoration semantics.
  Date/Author: 2026-07-12 / Codex

- Decision: use one shared selection result across strategy metrics, trades,
  and equity.
  Rationale: the three commands must not independently choose different
  candidates from the same selected chart.
  Date/Author: 2026-07-12 / Codex

- Decision: do not use any existing chart target as a disposable strategy
  fixture without owner confirmation.
  Rationale: adding a strategy, opening Strategy Tester, or changing
  visibility can mutate an account-linked selected chart. The current targets
  do not identify themselves as test state.
  Date/Author: 2026-07-12 / Codex

- Decision: use only explicit current strategy metadata signals and remove the
  broad legacy `is_price_study` plus report-method fallback.
  Rationale: collecting the legacy tier classified ordinary report-like
  studies as strategies on a no-strategy chart, while the current fixture
  exposed two explicit strategy signals. No live evidence justified retaining
  the false-positive-prone fallback.
  Date/Author: 2026-07-12 / Codex

- Decision: do not claim deterministic panel status on the current build.
  Rationale: structured data was available in both panel states, while the
  existing DOM selector did not track the visibly open panel reliably.
  `unknown` is more accurate and panel mutation is unnecessary.
  Date/Author: 2026-07-12 / Codex

- Decision: retain the dedicated owner-approved test layout until cleanup is
  separately authorized.
  Rationale: the localized new-tab landing page offered only persistent layout
  creation. Deleting account-linked test state is a separate mutation and is
  not implied by approval to create the fixture.
  Date/Author: 2026-07-12 / Codex

## Outcomes & Retrospective

The live matrix and first implementation are complete except for the safely
unavailable multiple-strategy fixture. Visible strategy reads work with the
panel open or closed. Hidden state is now diagnosed consistently without
mutation. Current metadata recognition and report-based selection share one
helper across all three commands, and `strategy_context` is additive. No new
command, option, dependency, or package version was added.

Implementation, review corrections, final re-review, and validation are
complete. A future exact-match study search command can provide a safe
multiple-strategy fixture; this slice does not weaken exact-match safety to
manufacture one.

## Context and Orientation

The `tv` binary command definitions live in `crates/cli/src/cli.rs`, and command
dispatch lives in `crates/cli/src/app/dispatch.rs`. The three relevant commands
are `tv data strategy`, `tv data trades`, and `tv data equity`. Each connects to
the selected TradingView Desktop target through CDP, short for Chrome DevTools
Protocol, and evaluates a JavaScript expression inside the chart page.

The report readers are in `crates/cli/src/ops/data/strategy.rs`; candidate
inspection and selection are in
`crates/cli/src/ops/data/strategy_selection.rs`. JavaScript first returns only
public-safe candidate descriptors. Rust `select_strategy` then chooses a
candidate and shapes `strategy_context`. A second reader expression resolves
the selected chart-local identity through the same `id` / `entityId` contract
and reads metrics, trades, or equity.

Candidate inspection recognizes `StrategyScript`, `isTVScriptStrategy`, and
`is_strategy` signals. It does not treat `is_price_study` or generic report
methods as strategy identity. Multiple candidates select the sole
report-bearing strategy or return ambiguity instead of using chart order.

The report readers are broader than the selector. Metrics inspect
`_reportData.performance`, `reportData()`, and `performance()`. Trades inspect
`_reportData.trades`, `ordersData`, `_orders`, and `tradesData`, then use a
visible DOM fallback. Equity inspects `_reportData.buyHold`, `equityData`, bar
data, and performance summaries. The implementation should preserve useful
fallbacks while making the selected strategy explicit and consistent.

`crates/cli/src/ops/data/indicator.rs` already returns chart-local entity IDs
and filtered inputs for `tv data indicator`. Entity IDs are acceptable
strategy context; Chrome target IDs, raw internal metadata, report payloads,
and account-local identifiers are not.

The relevant user and agent guidance is in `README.md`,
`docs/command-source-taxonomy.md`, `docs/observation-workflows.md`,
`docs/development.md`, `packaging/agent/AGENTS.md`,
`.agents/skills/strategy-report/`, and `.agents/skills/chart-analysis/`. Runtime
skill content also has packaged copies under `.claude/skills/` where the
release staging allowlist includes them.

## Milestone 1: Establish the current-build state matrix

Prepare a disposable or test-only TradingView chart. Use a strategy whose
identity can be recognized without recording its source or private results.
Observe these states where they can be prepared safely: one visible strategy
with Strategy Tester open, one visible strategy with the panel closed, one
hidden strategy, and more than one strategy. For each state, run all three data
commands.

Before changing visibility, record the current entity ID and visible state with
`tv indicator get <ENTITY_ID>`. Use the existing explicit
`tv indicator toggle <ENTITY_ID> --hidden` or `--visible` only on the disposable
chart, and restore the original visibility immediately after the observation.
Opening or closing Strategy Tester may be performed manually for the matrix;
do not add automation merely to prepare this evidence.

Record only a compact table in this ExecPlan containing the state name,
candidate count if observable, whether the intended strategy was detected,
source marker, metric/trade/data-point counts, high-level availability or error
classification, and whether the original state was restored. Do not record
metric values, trade rows, equity rows, raw DOM, raw runtime payloads, study
source, target IDs, or account-linked names.

If the intended strategy cannot be prepared safely, mark that row
`UNCONFIRMED`. If the panel or visibility mutation cannot be restored, stop the
matrix and restore the chart manually before implementation begins.

At the end of this milestone, update the candidate and readiness decisions in
this plan. Implementation must not begin while it is unclear whether report
availability identifies the selected strategy or whether multiple candidates
remain ambiguous.

## Milestone 2: Harden candidate detection and selection

Build public-safe candidate descriptors in evaluated JavaScript, then select
from those descriptors in I/O-free Rust. The descriptor carries no source
object, title, metadata object, or report payload. Inspect metadata defensively
because fields may be values or callable wrappers.

Recognize the verified signals from the live matrix. The initial candidate set
uses the existing `StrategyScript` ID prefix plus boolean
`isTVScriptStrategy` and `is_strategy` signals. The live no-strategy smoke
showed that the legacy report-capable fallback creates false positives, so it
is not retained.

For each candidate, determine report availability without copying the report
into diagnostics. Inspect only whether usable performance, trade, equity, or
general report containers exist. Build one selection helper used by all three
commands. A single recognized candidate may be selected even if its report is
not ready, so the command can explain readiness. With multiple candidates,
prefer an observed current-selection signal if the matrix establishes one. If
there is exactly one report-bearing candidate, selecting it is acceptable and
must be reported as `selection_reason: "only_report_available"` or equivalent.
If several candidates remain equally plausible, return an ambiguous readback
instead of choosing chart order.

Do not introduce a new command or strategy selector option in this milestone.
If users need to select among truly ambiguous strategies, record that as the
next explicit CLI contract decision rather than hiding it in the selector.

## Milestone 3: Add consistent public-safe readback

Keep existing metrics, trades, equity, counts, source markers, notes, and error
text available. Add a consistent `strategy_context` object, or an equivalently
named additive object, to all three command payloads. It should contain only
fields that the implementation can observe reliably:

    candidate_count
    selected_entity_id
    selected_title (reserved as null unless a later contract proves public classification)
    detection_signals
    selection_reason
    visible
    report_available
    panel_status
    availability_status

Fields that cannot be observed should be `null` or omitted consistently; do
not infer them. `panel_status` should distinguish at least `open`, `closed`, and
`unknown` only if deterministic panel detection exists. `availability_status`
should use a small vocabulary such as `available`, `report_not_ready`,
`strategy_hidden`, `ambiguous`, and `not_found`, based on live evidence.

Preserve the current nonterminal payload style for no-data strategy reads in
this slice unless a focused contract review proves that changing to an
`AppError` is required. Do not change top-level JSON envelopes or exit codes as
an incidental part of selector hardening.

If the matrix proves that the panel must be opened or a strategy must be made
visible, add a `next_action_hint` that states the required explicit action. Do
not perform the action automatically. An explicit ensure-ready option, if
approved later, must use `source_category: "desktop_backed_operation"`,
`non_mutating: false`, capture pre-state, report every mutation, and attempt
restoration.

## Plan of Work

First execute Milestone 1 and update this document. Then edit
`crates/cli/src/ops/data/strategy.rs` to replace the first-match helper with the
common candidate descriptor and selector. Keep all JavaScript literals escaped
through existing helpers; this plan does not introduce user-provided JavaScript.

Use a small Rust helper only if it removes meaningful payload-shaping
duplication after the JavaScript result is returned. Do not add a JavaScript
engine dependency merely to unit-test the expression. Existing fake runtime
tests should verify the generated expression contains each accepted signal,
the report-availability ranking path, and ambiguity handling, while returned
payload fixtures prove all existing fields and the new context survive the
adapter. The required live matrix supplies current-build behavioral evidence
that local fake evaluation cannot provide.

Add focused cases for one modern metadata candidate, legacy ID candidate,
ordinary price study rejection, one report-bearing candidate among multiple
strategies, multiple equally report-bearing candidates, hidden or unready
strategy, and no strategy. Ensure metrics, trades, and equity use the same
context fixture and selection reason.

After implementation, update help only if the current command descriptions
need a short readiness caveat. Update stable docs and packaged guidance to say
that structured reads report the selected strategy context and do not silently
open the panel or unhide studies. Revise `strategy-report` and
`chart-analysis`; put detailed state troubleshooting in a reference file if it
would make either Core Workflow substantially longer.

## Concrete Steps

Run commands from the repository root. Establish the local baseline before
editing production code:

    git status --short --branch
    cargo test -p tradingview-cli ops::data::strategy -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop data_read_commands -- --nocapture

Build a current binary for the live matrix:

    cargo build -p tradingview-cli
    target/debug/tv readiness
    target/debug/tv state
    target/debug/tv data strategy
    target/debug/tv data trades --max 5
    target/debug/tv data equity

Run the same three data commands in each safely prepared matrix state. Do not
paste their raw JSON into this plan. Summarize only counts, statuses, and
restoration outcomes.

After implementation, run focused validation:

    cargo test -p tradingview-cli ops::data::strategy -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop data_read_commands -- --nocapture
    cargo test -p tradingview-cli screenshot -- --nocapture

Validate changed runtime skills with the repository's configured skill
validator. Use the portable `CODEX_HOME` environment variable rather than a
machine-specific path.

Run the full baseline:

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

Acceptance requires live evidence from the current TradingView Desktop build
for every safely preparable matrix state. A visible single strategy must be
identified consistently by strategy, trades, and equity. A closed panel must
either remain readable or return a specific readiness state. A hidden strategy
must not be silently made visible. Multiple candidates must select the same
explainable report-bearing strategy across all three commands or return
ambiguity without arbitrary chart-order selection.

Existing practical fields remain present. New context is additive and contains
no raw report data, raw DOM, target ID, credential, session ID, account-local
identifier, or machine-specific path. Default commands remain Desktop-backed
reads with `non_mutating: true`; if the payload does not currently expose those
source fields, this plan must not invent inconsistent metadata in only one of
the three commands without updating all three and their contract tests.

Focused tests, full workspace tests, strict Clippy, formatting, metadata,
public hygiene, package-script syntax, contributor-guide parity, skill
validation, and diff checks must pass. A skipped required live state keeps the
corresponding behavior `UNCONFIRMED` and prevents a broad compatibility claim.

## Idempotence and Recovery

Local tests and read-only data commands are repeatable. The live matrix must
use a dedicated test chart or layout. Capture original strategy visibility and
panel state before each mutation, restore immediately afterward, and stop if
restoration cannot be verified. Do not delete account-linked test state without
separate owner authorization.

If current metadata differs from the upstream evidence, update this plan with
the observed signals rather than adding every guessed fallback. If report
availability changes while the chart is calculating, use a bounded readiness
observation and report timeout; do not add an unbounded sleep or retry loop.

No push, tag, GitHub Release, account-linked save, or package-version change is
authorized by this plan. Commit behavior follows the project owner's
instruction for the implementation turn.

## Artifacts and Notes

Planning evidence:

    Released baseline: v0.26.0 at 5e7f48f
    Upstream research boundary: 4795784..55534aa
    Relevant upstream changes: 653c273, 51384e1
    Current selector: explicit StrategyScript, isTVScriptStrategy, is_strategy
    Current report readers: performance, trades, orders, equity, DOM fallback
    Existing chart targets inspected: 3
    Confirmed applied strategy fixtures: 1 dedicated built-in strategy
    Metrics/trades baseline: zero count with unavailable DOM fallback
    Equity baseline: zero points without an error field
    Visible, panel closed: one candidate; all three structured reads available
    Visible, panel open: one candidate; all three structured reads available
    Hidden: one candidate; all three report strategy_hidden; state restored
    Multiple strategies: UNCONFIRMED; exact second add unavailable
    Panel status readback: unknown because current DOM detection drifted
    Focused strategy tests: 14 passed after focused re-review corrections
    Desktop data-command contract: 1 passed
    Screenshot-focused tests: 7 passed
    Workspace tests, strict Clippy, formatting, metadata: passed
    Public hygiene, skill validation, packaging syntax, guide parity: passed
    Initial independent review: four findings corrected
    Focused re-review: three findings corrected
    Final independent re-review: no findings

Final outcome: `tv data strategy`, `tv data trades`, and `tv data equity` now
share current-build candidate inspection and I/O-free selection, preserve all
existing reader capabilities, expose public-safe context without strategy
titles, and keep unavailable states nonterminal and non-mutating. The only
retained live uncertainty is multiple-strategy current-selection evidence.

When the live matrix is complete, add a public-safe summary here. Do not add
the symbol, strategy name if account-linked, performance values, trades, equity
rows, report payloads, chart target, or local environment paths.

## Interfaces and Dependencies

The report readers remain in `crates/cli/src/ops/data/strategy.rs`; candidate
inspection, Rust selection, context shaping, and fixture tests live in
`crates/cli/src/ops/data/strategy_selection.rs`. Keep these public Rust
operation signatures unless evidence requires a separately reviewed change:

    pub async fn data_strategy(
        runtime: &mut impl RuntimeEvaluator,
    ) -> Result<Value, AppError>

    pub async fn data_trades(
        runtime: &mut impl RuntimeEvaluator,
        max_trades: Option<usize>,
    ) -> Result<Value, AppError>

    pub async fn data_equity(
        runtime: &mut impl RuntimeEvaluator,
    ) -> Result<Value, AppError>

Do not add a new dependency. Use the existing `RuntimeEvaluator`, JSON value
handling, chart API constant, count cap, JSON envelope, and error contract.
Keep chart-target discovery and CDP transport ownership unchanged.

Candidate JavaScript returns only chart-local entity ID, explicit detection
signals, visibility, and capability booleans. Rust selects one candidate and
shapes context; the reader evaluation then resolves only that chart-local
entity. Do not return the source object, title, metadata object, or report
object in public JSON.

## Open Questions

- UNCONFIRMED: whether Strategy Tester exposes a reliable current-selection
  marker when multiple strategies are present.
- UNCONFIRMED: live multiple-strategy selection, because the current localized
  dialog did not provide a safe unique exact second strategy fixture.

Visible single-strategy reads are confirmed with the panel open and closed.
Hidden state is confirmed unavailable without mutation. The owner-approved
dedicated test layout remains isolated from the three pre-existing targets.

Revision note (2026-07-12): created as the first v0.27 ExecPlan after the
post-v0.26 upstream review. The initial scope deliberately separates
current-build diagnosis from selector implementation and excludes silent panel
or visibility mutation.

Revision note (2026-07-12): recorded the three-target no-strategy baseline and
corrected the focused integration-test filter. At that checkpoint, positive
matrix states had not yet been prepared and production selection code was
unchanged.

Revision note (2026-07-12): completed the dedicated live matrix except for the
safe multiple-strategy fixture, implemented shared selection/context, and
passed the full baseline. Review then identified reader-capability gating,
private title exposure, inconsistent DOM fallback semantics, weak selector
tests, and stale plan text. The correction moves selection/context shaping to
I/O-free Rust, covers every preserved reader capability with executable
fixtures, reserves `selected_title` as null, and makes not-found behavior
consistent across all three commands.

Revision note (2026-07-12): focused re-review found mismatched `entityId`
handoff, terminal inspection exceptions, and one stale orientation paragraph.
The final correction shares `id` / `entityId` resolution, catches chart/model
inspection inside the evaluated expression, normalizes typed or malformed
inspection failure into nonterminal read payloads, and adds operation-level
tests for both paths.
