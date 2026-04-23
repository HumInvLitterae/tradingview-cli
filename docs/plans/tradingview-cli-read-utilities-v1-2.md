# Add read utility commands to the Rust CLI

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with `.agents/PLANS.md`.

## Purpose / Big Picture

After this plan is implemented, the Rust-native `tv` command will cover another practical read-only slice of the old JavaScript TradingView CLI. A user or downstream adapter will be able to inspect symbol metadata, search TradingView symbols, read visible indicator values, read the current watchlist panel, and list panes in a multi-chart layout without returning to the JavaScript bridge.

The Rust CLI keeps the improved JSON envelope `{ success, command, data }`. The compatibility goal is information compatibility: practical information exposed by the old CLI for these migrated commands must remain available inside `data`.

## Progress

- [x] (2026-04-24 19:35 JST) Read `.agents/PLANS.md`, current Rust command modules, migration notes, old JavaScript command implementations, and downstream watchlist/review references.
- [x] (2026-04-24 19:50 JST) Add CLI arguments for `info`, `search`, `values`, `watchlist get`, and `pane list`.
- [x] (2026-04-24 19:50 JST) Implement read utility operations and information compatibility fields.
- [x] (2026-04-24 19:55 JST) Add unit and CLI contract tests.
- [x] (2026-04-24 20:00 JST) Update README, migration inventory, contract note, handoff note, and agent guide.
- [x] (2026-04-24 20:05 JST) Run validation, live smoke checks when available, and record results.

## Surprises & Discoveries

- Observation: The previous read/provider slice already removed the immediate `range` / `scroll` blocker from the downstream TradingView provider flow, so the next highest-value work is the remaining read-only old CLI surface.
  Evidence: `docs/notes/legacy-cli-command-migration-inventory-2026-04-24.md` lists `info`, `search`, `values`, `watchlist get`, and `pane list` as planned read-oriented backlog after `ohlcv --count`, `range`, and `scroll` were implemented.

- Observation: Downstream docs repeatedly refer to TradingView watchlist inputs and watchlist-based review workflows, so `watchlist get` is a useful operator-facing bridge even if the downstream Rust provider does not call it directly yet.
  Evidence: sibling project docs mention TradingView watchlist input parsing and watchlist review workflows.

- Observation: The existing `reqwest` version in this repository did not expose `RequestBuilder::query` in the current feature set, so the search implementation builds the URL with `reqwest::Url::parse_with_params`.
  Evidence: `cargo test` initially failed with `no method named query found for struct RequestBuilder`; switching to a prebuilt URL fixed the compile error.

## Decision Log

- Decision: Implement `info`, `search`, `values`, `watchlist get`, and `pane list` in one read-utilities slice.
  Rationale: These commands are read-only, small, and share the same migration rule: preserve old practical information under the Rust `data` envelope.
  Date/Author: 2026-04-24 / Codex

- Decision: Keep `discover`, `ui-state`, chart-region screenshots, write-oriented watchlist commands, and pane mutation commands out of this slice.
  Rationale: Diagnostics and visual capture have different stability questions, while mutation commands need stronger safety and recovery rules.
  Date/Author: 2026-04-24 / Codex

## Outcomes & Retrospective

The read utilities slice is implemented. The CLI now supports symbol metadata, symbol search, indicator values, watchlist reads, and pane listing under the existing Rust JSON envelope.

Automated validation passed with `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test`, `git diff --check`, and a tracked-doc absolute-path scan. Live smoke testing also passed against a running TradingView Desktop CDP target for `info`, `search AAPL`, `values`, `watchlist get`, and `pane list`.

## Context and Orientation

The Rust CLI is a single binary named `tv`. `src/cli.rs` defines command-line parsing with `clap`. `src/main.rs` dispatches commands and wraps successful results with `src/output.rs`. `src/ops.rs` owns TradingView command behavior by evaluating JavaScript through the `RuntimeEvaluator` trait in `src/cdp.rs`, and also owns any non-CDP helper operations needed by commands.

Chrome DevTools Protocol, or CDP, is the local debugging protocol exposed by TradingView Desktop when it is launched with a remote debugging port. Most commands in this plan connect to the TradingView Desktop CDP target and evaluate JavaScript inside the page. The `search` command is different: it calls TradingView's public symbol search HTTP endpoint and does not require a live CDP target.

The old JavaScript CLI returned command payload fields at the top level. This Rust CLI returns payload fields under `data`. For example, old `tv info` returned `{ "success": true, "symbol": "AAPL" }`, while Rust returns `{ "success": true, "command": "info", "data": { "symbol": "AAPL" } }`.

## Plan of Work

First, extend `src/cli.rs`. Add top-level `Info`, `Search`, and `Values` commands. Add `Watchlist` and `Pane` commands with subcommands so the public surface is `tv watchlist get` and `tv pane list`. Keep only read subcommands in this slice.

Next, update `src/main.rs` dispatch. `info`, `values`, `watchlist get`, and `pane list` should connect to the CDP runtime and call `ops` functions. `search` should validate that the query is not empty and then call an HTTP-backed `ops::symbol_search` function without connecting to CDP.

Then, expand `src/ops.rs`. Add JavaScript-evaluation functions for symbol metadata, indicator data-window values, watchlist rows, and pane listing. Add an HTTP search function that calls TradingView's public symbol search endpoint and normalizes the response into `query`, `source`, `count`, and `results`.

Finally, update tests and docs. Tests must prove the new command parsing, validation behavior, operation contracts, and search response normalization without requiring a live TradingView Desktop or a live network call.

## Concrete Steps

Run commands from the repository root.

After editing, run:

    cargo fmt --check
    cargo clippy --all-targets --all-features
    cargo test
    git diff --check

If TradingView Desktop is already running with CDP enabled, run smoke checks:

    cargo run -- info
    cargo run -- search AAPL
    cargo run -- values
    cargo run -- watchlist get
    cargo run -- pane list

`watchlist get` should succeed even when the right watchlist panel is closed; in that case `data.source` may be `panel_closed` and `data.count` may be `0`.

## Validation and Acceptance

The plan is accepted when the Rust CLI supports `info`, `search`, `values`, `watchlist get`, and `pane list`, and when each command returns the practical information described below under `data`.

`info` returns at least `symbol`, `full_name`, `exchange`, `description`, `type`, `pro_name`, `typespecs`, `resolution`, and `chart_type`.

`search` returns at least `query`, `source`, `count`, and `results`, where each result has `symbol`, `description`, `exchange`, `type`, and `full_name`.

`values` returns at least `study_count` and `studies`, where each study has `name` and `values`.

`watchlist get` returns at least `count`, `source`, and `symbols`, where each symbol row has `symbol`, `last`, `change`, and `change_percent`.

`pane list` returns at least `layout`, `layout_name`, `chart_count`, `active_index`, and `panes`, where each pane has `index`, `symbol`, and `resolution` when available.

Automated acceptance requires `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test`, and `git diff --check` to pass. CLI contract tests must cover the new argument surface and structured error behavior.

## Idempotence and Recovery

The implementation is additive. Running tests repeatedly should not change tracked files. Smoke commands in this plan are read-only and should not mutate chart symbol, timeframe, range, watchlist content, or pane layout. If a live smoke command fails because TradingView Desktop is not running, the watchlist panel is closed, or the internal TradingView DOM has changed, keep the automated validation result and record the smoke blocker.

## Artifacts and Notes

Important source evidence:

    old JavaScript CLI: src/cli/commands/chart.js, src/cli/commands/data.js, src/cli/commands/watchlist.js, src/cli/commands/pane.js
    old JavaScript core: src/core/chart.js, src/core/data.js, src/core/watchlist.js, src/core/pane.js
    Rust CLI: src/cli.rs, src/main.rs, src/ops.rs, src/cdp.rs
    migration policy: docs/notes/rust-cli-contract-migration-2026-04-24.md

## Interfaces and Dependencies

No new Rust crates are required. The existing `reqwest` dependency is used for `search`.

`src/cli.rs` must expose:

    tv info
    tv search <QUERY>
    tv values
    tv watchlist get
    tv pane list

`src/ops.rs` must expose operation functions that return `serde_json::Value` payloads. The output layer remains unchanged and wraps payloads in `SuccessEnvelope`.

`RuntimeEvaluator` remains the test seam for JavaScript evaluation. Tests must continue using fake evaluators rather than requiring a running TradingView Desktop.

## Open Questions

No critical open questions block this implementation. Later slices still need decisions for diagnostics, chart-region screenshots, watchlist mutation, pane mutation, and advanced data extraction commands.

Revision note: initial plan for the second read-only migration slice after `ohlcv --count`, `range`, and `scroll` were implemented.
