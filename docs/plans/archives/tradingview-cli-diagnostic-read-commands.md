# Add diagnostic read commands to the Rust CLI

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with `.agents/PLANS.md`.

## Purpose / Big Picture

After this plan is implemented, the Rust-native `tv` command will cover the remaining high-priority read-only diagnostic backlog from the old JavaScript TradingView CLI. A user or downstream adapter will be able to run `tv discover` to see which known TradingView internal API paths are available and `tv ui-state` to inspect visible panels, buttons, chart state, and replay state without returning to the JavaScript bridge.

The Rust CLI keeps the improved JSON envelope `{ success, command, data }`. The compatibility goal is information compatibility: practical information exposed by the old CLI for these migrated commands must remain available inside `data`.

## Progress

- [x] (2026-04-24 07:06 JST) Read `.agents/PLANS.md`, current Rust command modules, migration notes, old JavaScript `discover` and `ui-state` implementations, and live old CLI output.
- [x] (2026-04-24 07:08 JST) Add CLI arguments for `discover` and `ui-state`.
- [x] (2026-04-24 07:08 JST) Implement diagnostic read operations and information compatibility fields.
- [x] (2026-04-24 07:09 JST) Add unit and CLI contract tests.
- [x] (2026-04-24 07:10 JST) Update README, migration inventory, contract note, handoff note, and agent guide.
- [x] (2026-04-24 07:10 JST) Run validation, live smoke checks when available, and record results.

## Surprises & Discoveries

- Observation: The only remaining high-priority planned migration backlog entries are `discover` and `ui-state`.
  Evidence: `docs/notes/legacy-cli-command-migration-inventory-2026-04-24.md` lists only those two commands under `Planned migration backlog`.

- Observation: The old JavaScript CLI exposes `discover` and `ui-state` as top-level commands even though their implementation lives in health and chart command modules.
  Evidence: `src/cli/commands/chart.js` in the migration source registers `discover` and `ui-state`, and delegates to `src/core/health.js`.

- Observation: The old JavaScript CLI includes a `success: true` field inside each command payload, but the Rust CLI already provides success at the envelope level.
  Evidence: live old CLI output for `discover` and `ui-state` includes top-level `success: true`; Rust migration policy keeps command payload under `data`.

## Decision Log

- Decision: Implement `discover` and `ui-state` in one diagnostic read slice.
  Rationale: Both commands are read-only, diagnostic, and complete the current planned high-priority backlog.
  Date/Author: 2026-04-24 / Codex

- Decision: Preserve practical payload information but do not duplicate `success` inside `data`.
  Rationale: The Rust envelope already has top-level `success`, and previous migration decisions intentionally avoid cloning the old top-level wire shape.
  Date/Author: 2026-04-24 / Codex

## Outcomes & Retrospective

The diagnostic read commands slice is implemented. The CLI now supports `tv discover` and `tv ui-state` under the existing Rust JSON envelope, and the high-priority planned read-only migration backlog is empty.

Automated validation passed with `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test`, `git diff --check`, and a tracked-doc absolute-path scan. Live smoke testing passed against a running TradingView Desktop CDP target for `discover` and `ui-state`.

The live `discover` smoke returned `apis_available: 5` and `apis_total: 6`. The live `ui-state` smoke returned panel state, visible button groups, chart summary, and replay state. The main remaining risk is DOM and internal API drift in TradingView; these commands are diagnostic and should surface that drift through changed payloads or structured errors rather than mutating chart state.

## Context and Orientation

The Rust CLI is a single binary named `tv`. `src/cli.rs` defines command-line parsing with `clap`. `src/main.rs` dispatches commands and wraps successful results with `src/output.rs`. `src/ops.rs` owns TradingView command behavior by evaluating JavaScript through the `RuntimeEvaluator` trait in `src/cdp.rs`.

Chrome DevTools Protocol, or CDP, is the local debugging protocol exposed by TradingView Desktop when it is launched with a remote debugging port. Both commands in this plan connect to the TradingView Desktop CDP target and evaluate JavaScript inside the page. They are read-only and should not mutate chart symbol, timeframe, range, watchlist content, layout, or UI state.

The old JavaScript CLI returned command payload fields at the top level. This Rust CLI returns payload fields under `data`. For example, old `tv discover` returned `{ "success": true, "apis_available": 5 }`, while Rust returns `{ "success": true, "command": "discover", "data": { "apis_available": 5 } }`.

## Plan of Work

First, extend `src/cli.rs`. Add top-level `Discover` and `UiState` commands. Use an explicit clap name for `ui-state` so the public command is hyphenated.

Next, update `src/main.rs` dispatch. Both commands should connect to the CDP runtime and call `ops` functions. They should use the same structured connection errors as other CDP-backed read commands.

Then, expand `src/ops.rs`. Add `discover` and `ui_state` functions that evaluate JavaScript equivalent to the old implementation in `src/core/health.js` from the migration source. `discover` should return `apis_available`, `apis_total`, and `apis`. `ui_state` should return panel state, visible button groups, key buttons, chart summary, and replay state.

Finally, update tests and docs. Tests must prove the new command parsing, connection error behavior, and operation contracts without requiring a live TradingView Desktop.

## Concrete Steps

Run commands from the repository root.

After editing, run:

    cargo fmt --check
    cargo clippy --all-targets --all-features
    cargo test
    git diff --check

If TradingView Desktop is already running with CDP enabled, run smoke checks:

    cargo run -- discover
    cargo run -- ui-state

Both commands should print `success: true` and return payloads under `data`. `discover` should include `apis_available`, `apis_total`, and `apis`. `ui-state` should include panel entries such as `bottom_panel`, `right_panel`, `pine_editor`, `strategy_tester`, and `widgetbar`.

## Validation and Acceptance

The plan is accepted when the Rust CLI supports `discover` and `ui-state`, and when each command returns the practical information described below under `data`.

`discover` returns at least `apis_available`, `apis_total`, and `apis`. The `apis` object includes `chartApi`, `chartWidgetCollection`, `chartApiInstance`, `bottomWidgetBar`, `replayApi`, and `alertService`. API entries include availability and either path/method information or an error message when unavailable.

`ui-state` returns at least `bottom_panel`, `right_panel`, `pine_editor`, `strategy_tester`, `widgetbar`, `buttons`, `key_buttons`, `chart`, and `replay`.

Automated acceptance requires `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test`, and `git diff --check` to pass. CLI contract tests must cover the new argument surface and structured connection error behavior.

## Idempotence and Recovery

The implementation is additive. Running tests repeatedly should not change tracked files. Smoke commands in this plan are read-only and should not mutate TradingView state. If a live smoke command fails because TradingView Desktop is not running or the internal TradingView DOM has changed, keep the automated validation result and record the smoke blocker.

## Artifacts and Notes

Important source evidence:

    old JavaScript CLI: src/cli/commands/chart.js
    old JavaScript core: src/core/health.js
    Rust CLI: src/cli.rs, src/main.rs, src/ops.rs, src/cdp.rs
    migration policy: docs/notes/rust-cli-contract-migration-2026-04-24.md

## Interfaces and Dependencies

No new Rust crates are required.

`src/cli.rs` must expose:

    tv discover
    tv ui-state

`src/ops.rs` must expose:

    pub async fn discover(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>
    pub async fn ui_state(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>

`RuntimeEvaluator` remains the test seam for JavaScript evaluation. Tests must continue using fake evaluators rather than requiring a running TradingView Desktop.

## Open Questions

No critical open questions block this implementation. Once this slice is complete, the planned high-priority read-only migration backlog should be empty, and later work can choose among deferred larger surfaces such as launch automation, pane mutation, watchlist mutation, Pine, alerts, replay, stream, and UI automation.

Revision note: initial plan for the diagnostic read commands slice after chart-region screenshot support and compatibility improvements were completed.
