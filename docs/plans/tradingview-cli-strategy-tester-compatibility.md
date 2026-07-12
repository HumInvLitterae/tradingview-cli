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
- [ ] Prepare a disposable strategy test chart and record the bounded live
  state matrix using public-safe summaries only.
- [ ] Update `Surprises & Discoveries` and `Decision Log` with the matrix and
  finalize candidate-selection rules before editing production behavior.
- [ ] Harden shared candidate detection and report selection for strategy,
  trades, and equity reads.
- [ ] Add additive strategy context and public-safe unavailable/ambiguous
  diagnostics without breaking existing practical fields.
- [ ] Add deterministic focused tests and run the required Desktop smoke.
- [ ] Synchronize help where applicable, stable docs, packaged agent guidance,
  and the `strategy-report` and `chart-analysis` runtime skills.
- [ ] Run the complete workspace, hygiene, skill, and packaging baseline.
- [ ] Obtain independent review, correct findings, archive this plan, and only
  then promote the next v0.27 work item.

## Surprises & Discoveries

- Observation: current Rust code already reads several modern report shapes,
  including `_reportData.performance`, `_reportData.trades`, and
  `reportData()`, but candidate selection can reject a current strategy before
  those readers run.
  Evidence: `crates/cli/src/ops/data/strategy.rs` implements the report readers
  after `__findStrategy`, whose fallback requires
  `is_price_study === false`.

- Observation: upstream current-build work reports that a strategy may expose
  `isTVScriptStrategy` or `is_strategy` and may have
  `is_price_study === true`.
  Evidence: the reviewed upstream changes are `653c273` and `51384e1` in the
  local research boundary `4795784..55534aa`. This is design evidence, not yet
  proof of failure in the released Rust binary.

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

## Outcomes & Retrospective

Planning is complete. Current-build compatibility remains unconfirmed until
the live matrix is executed. No runtime command, option, payload, dependency,
or package version has changed in this planning slice.

The intended implementation outcome is a common, explainable strategy
selection path with additive context and explicit unavailable or ambiguous
states. If the matrix proves that panel opening or visibility changes are
required, this plan must record that discovery and either add a separately
explicit milestone or stop for a dedicated follow-up ExecPlan.

## Context and Orientation

The `tv` binary command definitions live in `crates/cli/src/cli.rs`, and command
dispatch lives in `crates/cli/src/app/dispatch.rs`. The three relevant commands
are `tv data strategy`, `tv data trades`, and `tv data equity`. Each connects to
the selected TradingView Desktop target through CDP, short for Chrome DevTools
Protocol, and evaluates a JavaScript expression inside the chart page.

The operation code is in `crates/cli/src/ops/data/strategy.rs`. The functions
`data_strategy`, `data_trades`, and `data_equity` send expressions built by
`strategy_metrics_expression`, `strategy_trades_expression`, and
`strategy_equity_expression`. All three embed `STRATEGY_HELPERS` and call its
`__findStrategy` function before reading report data.

Current `__findStrategy` first returns a source whose metadata ID begins with
`StrategyScript`. Its fallback returns the first source whose metadata says
`is_price_study === false` and that exposes one of several report-like methods.
That fallback is vulnerable to metadata drift and chart-order ambiguity.

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

Refactor `STRATEGY_HELPERS` so it builds public-safe candidate descriptors
before selecting a strategy. A descriptor should retain the source object only
inside the evaluated JavaScript and expose safe context separately. Inspect
metadata defensively because fields may be values or callable wrappers.

Recognize the verified signals from the live matrix. The initial candidate set
should consider the existing `StrategyScript` ID prefix plus boolean
`isTVScriptStrategy` and `is_strategy` signals. Retain a legacy report-capable
fallback only if the matrix and fixtures show it is needed, and do not treat
`is_price_study` alone as sufficient evidence.

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
    selected_title
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
    cargo test -p tradingview-cli --test cli_contract_desktop strategy -- --nocapture

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
    cargo test -p tradingview-cli --test cli_contract_desktop strategy -- --nocapture
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
use a disposable chart or layout. Capture original strategy visibility and
panel state before each mutation, restore immediately afterward, and stop if
restoration cannot be verified. Never delete a strategy or save a layout as
part of this plan.

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
    Current selector: StrategyScript prefix, then legacy is_price_study false
    Current report readers: performance, trades, orders, equity, DOM fallback
    Live compatibility matrix: pending

When the live matrix is complete, add a public-safe summary here. Do not add
the symbol, strategy name if account-linked, performance values, trades, equity
rows, report payloads, chart target, or local environment paths.

## Interfaces and Dependencies

The primary implementation remains in
`crates/cli/src/ops/data/strategy.rs`. Keep these public Rust operation
signatures unless evidence requires a separately reviewed change:

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

The expected internal JavaScript design is one helper that returns both the
selected source object for the report reader and a public-safe context object
for the payload. Every command must call the same helper logic. Do not return
the source object, metadata object, or report object in the public JSON.

## Open Questions

- UNCONFIRMED: whether current visible single-strategy reads fail in the
  released binary or only less common states do.
- UNCONFIRMED: whether Strategy Tester exposes a reliable current-selection
  marker when multiple strategies are present.
- UNCONFIRMED: whether report generation requires the panel to have been opened
  in the current Desktop build.
- UNCONFIRMED: whether a hidden strategy can provide valid existing report data
  without a visibility mutation.

These questions are not delegated to the implementer as design choices. They
must be resolved by Milestone 1, recorded in this plan, and used to update the
prescriptive selection and readiness rules before production edits begin.

Revision note (2026-07-12): created as the first v0.27 ExecPlan after the
post-v0.26 upstream review. The initial scope deliberately separates
current-build diagnosis from selector implementation and excludes silent panel
or visibility mutation.
