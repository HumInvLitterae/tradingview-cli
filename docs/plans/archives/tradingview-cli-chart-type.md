# Add chart type read and set commands to the Rust CLI

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with `.agents/PLANS.md`.

## Purpose / Big Picture

After this plan is implemented, the Rust-native `tv` command will cover the old JavaScript CLI's `type` surface. A user or downstream adapter will be able to read the current TradingView chart type, switch the visible chart between candles, line, area, and other supported TradingView chart types, then restore the previous type after a smoke test or visual workflow.

This is intentionally a small chart mutation slice. The command complements the already implemented chart setup commands such as `symbol`, `timeframe`, `range`, `scroll`, and screenshots. It does not introduce general UI automation, account-level watchlist mutation, alert creation, Pine editor automation, replay automation, or multi-pane layout mutation.

The Rust CLI keeps the improved JSON envelope `{ success, command, data }`. The compatibility goal is information compatibility: practical information exposed by the old CLI for `type`, especially `chart_type` and `type_num`, must remain available inside `data`.

## Progress

- [x] (2026-04-24 15:09 JST) Read `.agents/PLANS.md`, current Rust command modules, migration notes, old JavaScript `type` implementation, and the approved candidate-ranking plan.
- [x] (2026-04-24 15:12 JST) Add CLI arguments and dispatch for `tv type [CHART_TYPE]`.
- [x] (2026-04-24 15:14 JST) Implement chart type read/set operations and information compatibility fields.
- [x] (2026-04-24 15:16 JST) Add unit and CLI contract tests.
- [x] (2026-04-24 15:18 JST) Update README, migration inventory, contract note, handoff note, agent guide, and chart-analysis skill note.
- [x] (2026-04-24 15:24 JST) Run validation and live smoke checks against TradingView Desktop CDP, then record results.

## Surprises & Discoveries

- Observation: The old JavaScript `type` command is smaller and less DOM-dependent than the other deferred surfaces considered for the next slice.
  Evidence: the old `src/cli/commands/chart.js` command maps read mode to `getState()` and set mode to `core.setType()`, and `src/core/chart.js` implements set mode through `chart.setChartType(typeNum)`.

- Observation: `data depth` remains read-only but has more fragile prerequisites than `type`.
  Evidence: the completed advanced data reads plan records that old `src/core/data.js` searches visible DOM selectors such as order book, depth, DOM panel, rows, and cells before returning bids and asks.

## Decision Log

- Decision: Implement `type` as the next migration slice.
  Rationale: It is an old CLI command that fills a practical gap in chart setup and screenshot workflows, while staying much smaller and safer than watchlist mutation, pane mutation, alerts, Pine, replay, drawing, tab, stream, or UI automation.
  Date/Author: 2026-04-24 / Codex

- Decision: Support both read mode and set mode in one command: `tv type` and `tv type <CHART_TYPE>`.
  Rationale: This matches the old CLI surface and lets smoke tests safely read, change, verify, and restore chart type.
  Date/Author: 2026-04-24 / Codex

- Decision: Preserve old practical fields `chart_type` and `type_num` under `data`, while adding request/observation fields for set mode.
  Rationale: Downstream callers migrating from the JavaScript CLI need the same practical information, but the Rust envelope remains the stable outer contract.
  Date/Author: 2026-04-24 / Codex

## Outcomes & Retrospective

The chart type slice is implemented. The CLI now supports `tv type` and `tv type <CHART_TYPE>` under the existing Rust JSON envelope.

Automated validation passed with `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test`, `git diff --check`, a tracked-doc absolute-path scan, and skill validation for `.agents/skills/chart-analysis`. Live smoke testing passed against a running TradingView Desktop CDP target. The live session initially reported `HollowCandles` on `BATS:IONQ` at resolution `1`, changed to `Line`, changed to `Candles`, and was restored to `HollowCandles`.

## Context and Orientation

The Rust CLI is a single binary named `tv`. `src/cli.rs` defines command-line parsing with `clap`. `src/main.rs` dispatches parsed commands and wraps successful results with `src/output.rs`. `src/ops.rs` owns TradingView command behavior by evaluating JavaScript through the `RuntimeEvaluator` trait in `src/cdp.rs`.

Chrome DevTools Protocol, or CDP, is the local debugging protocol exposed by TradingView Desktop when it is launched with a remote debugging port. The `type` command connects to the TradingView Desktop CDP target and evaluates JavaScript inside the page. Read mode is non-mutating. Set mode mutates only the active chart's visual chart type by calling TradingView's internal `chart.setChartType(number)` method.

The old JavaScript CLI returned command payload fields at the top level. This Rust CLI returns payload fields under `data`. For example, old `tv type` returned `{ "success": true, "chart_type": "Candles", "type_num": 1 }`, while Rust returns `{ "success": true, "command": "type", "data": { "chart_type": "Candles", "type_num": 1 } }`.

## Plan of Work

First, extend `src/cli.rs`. Add a top-level `Type` command with one optional positional value named `chart_type`. Add `"type"` to `Command::name()`.

Next, update `src/main.rs` dispatch. `tv type` should connect to the CDP runtime and call `ops::current_chart_type`. `tv type <CHART_TYPE>` should reject an empty value before connecting, then call `ops::set_chart_type`.

Then, expand `src/ops.rs`. Add a chart type mapping that accepts the old JavaScript CLI chart type names and numeric values from 0 through 9. The supported names are `Bars`, `Candles`, `Line`, `Area`, `Renko`, `Kagi`, `PointAndFigure`, `LineBreak`, `HeikinAshi`, and `HollowCandles`. Name matching should be case-insensitive and should also tolerate separators such as hyphen, underscore, and spaces, so `heikin-ashi` maps to `HeikinAshi`. Invalid values must return a validation error before evaluating JavaScript.

Read mode should evaluate `chart.chartType()` and return at least `chart_type`, `type_num`, `symbol`, and `resolution` when available. Set mode should read the previous type, call `chart.setChartType(typeNum)`, then read the observed type. The payload should include at least `chart_type`, `type_num`, `requested_chart_type`, `requested_type_num`, `previous_chart_type`, `previous_type_num`, `observed_chart_type`, and `observed_type_num`.

Finally, update tests and docs. Tests must prove command parsing, structured connection errors, invalid chart type validation, chart type mapping, and operation contracts without requiring a live TradingView Desktop.

## Concrete Steps

Run commands from the repository root.

After editing, run:

    cargo fmt --check
    cargo clippy --all-targets --all-features
    cargo test
    git diff --check

If TradingView Desktop is already running with CDP enabled, run smoke checks:

    cargo run -- type
    cargo run -- type Line
    cargo run -- type Candles

If the initial `tv type` result reports a type other than `Candles`, restore that initial type after the smoke test by running `cargo run -- type <INITIAL_TYPE>`.

## Validation and Acceptance

The plan is accepted when the Rust CLI supports `tv type` and `tv type <CHART_TYPE>`, and both modes return practical information under `data`.

`tv type` returns at least `chart_type`, `type_num`, `symbol`, and `resolution`.

`tv type <CHART_TYPE>` returns at least `chart_type`, `type_num`, `requested_chart_type`, `requested_type_num`, `previous_chart_type`, `previous_type_num`, `observed_chart_type`, and `observed_type_num`.

Automated acceptance requires `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test`, and `git diff --check` to pass. CLI contract tests must cover help output, structured connection error behavior, and invalid chart type validation.

## Idempotence and Recovery

The implementation is additive. Running tests repeatedly should not change tracked files. Smoke testing mutates the active chart type, so it must read the initial type first and restore it after testing. If a live smoke command fails because TradingView Desktop is not running or the internal TradingView API has changed, keep the automated validation result and record the smoke blocker.

## Artifacts and Notes

Important source evidence:

    old JavaScript CLI: src/cli/commands/chart.js
    old JavaScript core: src/core/chart.js
    Rust CLI: src/cli.rs, src/main.rs, src/ops.rs, src/cdp.rs
    migration policy: docs/notes/rust-cli-contract-migration-2026-04-24.md

## Interfaces and Dependencies

No new Rust crates are required.

`src/cli.rs` must expose:

    tv type [CHART_TYPE]

`src/ops.rs` must expose:

    pub async fn current_chart_type(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>
    pub async fn set_chart_type(runtime: &mut impl RuntimeEvaluator, chart_type: &str) -> Result<Value, AppError>

`RuntimeEvaluator` remains the test seam for JavaScript evaluation. Tests must continue using fake evaluators rather than requiring a running TradingView Desktop.

## Open Questions

No critical open questions block this implementation. Later work can decide whether the next surface should be `data depth`, `alert list`, or a non-command readiness task such as CI, release packaging, downstream adapter validation, or upstream PR triage.

Revision note: created after comparing deferred old CLI surfaces and selecting `type` as the next low-risk chart setup slice.

Revision note: completed the chart type implementation and recorded automated plus live smoke validation results.
