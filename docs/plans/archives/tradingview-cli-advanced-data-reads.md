# Add advanced data read commands to the Rust CLI

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with `.agents/PLANS.md`.

## Purpose / Big Picture

After this plan is implemented, the Rust-native `tv` command will cover the next useful read-only slice of the old JavaScript TradingView CLI. A user or downstream adapter will be able to inspect strategy metrics, strategy trades, equity data, indicator inputs, and Pine drawing primitives without returning to the JavaScript bridge.

The Rust CLI keeps the improved JSON envelope `{ success, command, data }`. The compatibility goal is information compatibility: practical information exposed by the old CLI for these migrated commands must remain available inside `data`.

## Progress

- [x] (2026-04-24 07:52 JST) Read `.agents/PLANS.md`, current Rust command modules, migration notes, old JavaScript `data` command implementation, and newly migrated skill gap notes.
- [x] (2026-04-24 07:55 JST) Add CLI arguments for the read-only `data` subcommands.
- [x] (2026-04-24 07:57 JST) Implement advanced data read operations and information compatibility fields.
- [x] (2026-04-24 07:58 JST) Add unit and CLI contract tests.
- [x] (2026-04-24 07:59 JST) Update README, migration inventory, contract note, handoff note, agent guide, and migrated skills.
- [x] (2026-04-24 08:01 JST) Run validation, live smoke checks, and record results.

## Surprises & Discoveries

- Observation: The old `data depth` command is read-only but depends on a DOM or order book panel being visibly open, unlike strategy and Pine drawing reads that primarily inspect chart model objects.
  Evidence: the old `src/core/data.js` implementation searches DOM selectors such as order book, depth, and DOM panel classes before returning bids and asks.

- Observation: The newly migrated repo-local skills expose exactly the gap this slice should close for strategy reports, chart analysis, and multi-symbol scans.
  Evidence: `.agents/skills/strategy-report/SKILL.md` currently says strategy tester results, trade lists, and equity curves are not extracted; `.agents/skills/chart-analysis/references/workflow.md` marks drawing-derived data as unavailable.

- Observation: A live TradingView session can expose equity-like data even when no strategy trades are available.
  Evidence: `cargo run -- data equity` returned `data_points: 400`, while `cargo run -- data trades --max 5` returned `trade_count: 0` with `error: "No strategy found on chart."`.

## Decision Log

- Decision: Implement `data strategy`, `data trades`, `data equity`, `data indicator`, `data lines`, `data labels`, `data tables`, and `data boxes` in one read-only slice.
  Rationale: These commands are all old CLI read operations, share the same chart-model evaluation style, and unblock the migrated skills without introducing UI mutation.
  Date/Author: 2026-04-24 / Codex

- Decision: Exclude `data depth` from this slice.
  Rationale: It is panel and DOM dependent, so it needs a separate evidence pass and live smoke plan. Leaving it deferred avoids mixing a fragile panel scrape with chart-model reads.
  Date/Author: 2026-04-24 / Codex

- Decision: Preserve practical old CLI payload fields but do not duplicate `success` inside `data`.
  Rationale: The Rust envelope already has top-level `success`, and prior migration decisions intentionally keep command payloads under `data`.
  Date/Author: 2026-04-24 / Codex

## Outcomes & Retrospective

The advanced data reads slice is implemented. The CLI now supports `tv data indicator`, `tv data strategy`, `tv data trades`, `tv data equity`, `tv data lines`, `tv data labels`, `tv data tables`, and `tv data boxes` under the existing Rust JSON envelope.

Automated validation passed with `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test`, `git diff --check`, a tracked-doc absolute-path scan, and repo-local skill validation for the changed skills. Live smoke testing passed against a running TradingView Desktop CDP target for every new command. The live session returned an indicator payload for study `gYwGZx`, empty lines and boxes payloads, 4 label studies, 1 table study, 400 equity data points, no trades, and an empty strategy metrics payload.

## Context and Orientation

The Rust CLI is a single binary named `tv`. `src/cli.rs` defines command-line parsing with `clap`. `src/main.rs` dispatches commands and wraps successful results with `src/output.rs`. `src/ops.rs` owns TradingView command behavior by evaluating JavaScript through the `RuntimeEvaluator` trait in `src/cdp.rs`.

Chrome DevTools Protocol, or CDP, is the local debugging protocol exposed by TradingView Desktop when it is launched with a remote debugging port. The commands in this plan connect to the TradingView Desktop CDP target and evaluate JavaScript inside the page. They are read-only and should not mutate chart symbol, timeframe, range, watchlist content, layout, strategy settings, Pine source, or UI state.

The old JavaScript CLI returned command payload fields at the top level. This Rust CLI returns payload fields under `data`. For example, old `tv data strategy` returned `{ "success": true, "metric_count": 10 }`, while Rust returns `{ "success": true, "command": "data", "data": { "metric_count": 10 } }`.

## Plan of Work

First, extend `src/cli.rs`. Add a top-level `Data` command with subcommands for `indicator`, `strategy`, `trades`, `equity`, `lines`, `labels`, `tables`, and `boxes`. Keep old option names: `--filter` / `-f`, `--verbose` / `-v`, and `--max` / `-n`.

Next, update `src/main.rs` dispatch. All `data` subcommands should connect to the CDP runtime and call one `ops` function each. Validate that required positionals such as `ENTITY_ID` are non-empty. Clamp count-style limits inside `ops`, matching existing `ohlcv` style.

Then, expand `src/ops.rs`. Add JavaScript-evaluation functions equivalent to the old implementation in `src/core/data.js` from the migration source. `data indicator` reads study inputs by entity ID. `data strategy`, `data trades`, and `data equity` inspect strategy study internals. `data lines`, `data labels`, `data tables`, and `data boxes` inspect Pine drawing primitive collections.

Finally, update tests and docs. Tests must prove the new command parsing, connection error behavior, JavaScript string escaping for user filters/entity IDs, and operation contracts without requiring a live TradingView Desktop.

## Concrete Steps

Run commands from the repository root.

After editing, run:

    cargo fmt --check
    cargo clippy --all-targets --all-features
    cargo test
    git diff --check

If TradingView Desktop is already running with CDP enabled, run smoke checks:

    cargo run -- data strategy
    cargo run -- data trades --max 5
    cargo run -- data equity
    cargo run -- data lines
    cargo run -- data labels --max 5
    cargo run -- data tables
    cargo run -- data boxes

Run `cargo run -- state` first to find a study entity ID before smoking `data indicator`, then run:

    cargo run -- data indicator <ENTITY_ID>

Commands should print `success: true` and return payloads under `data`. Empty strategy or drawing results should be represented as successful empty payloads with an explanatory `error` or zero count when that matches the old CLI behavior.

## Validation and Acceptance

The plan is accepted when the Rust CLI supports the new `data` read commands and each command returns the practical information described below under `data`.

`data indicator` returns at least `entity_id`, `visible`, and `inputs`.

`data strategy` returns at least `metric_count`, `source`, `metrics`, and optional `error`.

`data trades` returns at least `trade_count`, `source`, `trades`, and optional `error`.

`data equity` returns at least `data_points`, `source`, `data`, and optional `equity_summary`, `note`, and `error`.

`data lines` returns at least `study_count` and `studies`; each study has `name`, `total_lines`, and `horizontal_levels`, with `all_lines` only when verbose.

`data labels` returns at least `study_count` and `studies`; each study has `name`, `total_labels`, `showing`, and `labels`, with verbose label fields only when verbose.

`data tables` returns at least `study_count` and `studies`; each study has `name` and `tables`.

`data boxes` returns at least `study_count` and `studies`; each study has `name`, `total_boxes`, and `zones`, with `all_boxes` only when verbose.

Automated acceptance requires `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test`, and `git diff --check` to pass. CLI contract tests must cover the new argument surface and structured connection error behavior.

## Idempotence and Recovery

The implementation is additive. Running tests repeatedly should not change tracked files. Smoke commands in this plan are read-only and should not mutate TradingView state. If a live smoke command fails because TradingView Desktop is not running, no strategy is present, no Pine drawing primitives are present, or the internal TradingView model has changed, keep the automated validation result and record the smoke blocker.

## Artifacts and Notes

Important source evidence:

    old JavaScript CLI: src/cli/commands/data.js
    old JavaScript core: src/core/data.js
    Rust CLI: src/cli.rs, src/main.rs, src/ops.rs, src/cdp.rs
    migration policy: docs/notes/rust-cli-contract-migration-2026-04-24.md

## Interfaces and Dependencies

No new Rust crates are required.

`src/cli.rs` must expose:

    tv data indicator <ENTITY_ID>
    tv data strategy
    tv data trades [--max <N>]
    tv data equity
    tv data lines [--filter <TEXT>] [--verbose]
    tv data labels [--filter <TEXT>] [--max <N>] [--verbose]
    tv data tables [--filter <TEXT>]
    tv data boxes [--filter <TEXT>] [--verbose]

`src/ops.rs` must expose:

    pub async fn data_indicator(runtime: &mut impl RuntimeEvaluator, entity_id: &str) -> Result<Value, AppError>
    pub async fn data_strategy(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>
    pub async fn data_trades(runtime: &mut impl RuntimeEvaluator, max_trades: Option<usize>) -> Result<Value, AppError>
    pub async fn data_equity(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>
    pub async fn data_lines(runtime: &mut impl RuntimeEvaluator, filter: Option<&str>, verbose: bool) -> Result<Value, AppError>
    pub async fn data_labels(runtime: &mut impl RuntimeEvaluator, filter: Option<&str>, max_labels: Option<usize>, verbose: bool) -> Result<Value, AppError>
    pub async fn data_tables(runtime: &mut impl RuntimeEvaluator, filter: Option<&str>) -> Result<Value, AppError>
    pub async fn data_boxes(runtime: &mut impl RuntimeEvaluator, filter: Option<&str>, verbose: bool) -> Result<Value, AppError>

`RuntimeEvaluator` remains the test seam for JavaScript evaluation. Tests must continue using fake evaluators rather than requiring a running TradingView Desktop.

## Open Questions

No critical open questions block this implementation. Later work can decide whether `data depth` deserves its own DOM-dependent slice.

Revision note: initial plan for the advanced data read commands slice after repo-local skill migration identified strategy and Pine drawing read gaps.
