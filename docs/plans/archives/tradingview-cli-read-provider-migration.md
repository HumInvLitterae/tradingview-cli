# Add read/provider migration commands to the Rust CLI

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with `.agents/PLANS.md`. It is self-contained and assumes the reader has only this repository checkout.

## Purpose / Big Picture

After this plan is implemented, the Rust-native `tv` command will cover the first provider-oriented slice of the old JavaScript `tv` CLI migration. A user or downstream adapter will be able to read raw OHLCV bars, inspect and restore visible chart ranges, scroll the chart to a date, read current symbol and timeframe without mutating the chart, and receive richer status/state/quote data while keeping the Rust JSON envelope.

The Rust CLI intentionally returns successful command payloads inside a `{ success, command, data }` envelope rather than cloning the old JavaScript CLI top-level payload shape. The compatibility goal for this plan is information compatibility: practical information exposed by the old CLI for these migrated commands must be available under `data`.

## Progress

- [x] (2026-04-24 18:10 JST) Read `.agents/PLANS.md`, current Rust modules, migration policy notes, old JavaScript command implementations, and downstream provider expectations.
- [x] (2026-04-24 18:45 JST) Add CLI arguments for `ohlcv --count`, `range`, `scroll`, optional `symbol`, and optional `timeframe`.
- [x] (2026-04-24 18:45 JST) Implement provider-oriented operations and information compatibility fields.
- [x] (2026-04-24 18:45 JST) Add unit and CLI contract tests.
- [x] (2026-04-24 18:55 JST) Update README, migration inventory, contract note, and handoff note.
- [x] (2026-04-24 19:05 JST) Run validation and record results.

## Surprises & Discoveries

- Observation: The downstream provider uses `state`, `symbol`, `timeframe`, `ohlcv --count`, `range`, and `scroll` as one coherent provider flow.
  Evidence: `crates/backtest-data/src/provider/tradingview_desktop.rs` in the sibling project calls those commands when fetching bounded historical bars and restoring chart state.

- Observation: The old JavaScript CLI allowed read mode for `symbol` and `timeframe`.
  Evidence: `src/cli/commands/chart.js` in the migration source returns current state fields when no positional argument is passed.

- Observation: The raw OHLCV implementation should also return chart identity fields, because `ohlcv --summary` depends on the same extraction path and the project policy forbids losing practical information that already existed in the Rust v1 summary.
  Evidence: `README.md` and `docs/notes/rust-cli-contract-migration-2026-04-24.md` require information compatibility for migrated commands.

## Decision Log

- Decision: Keep the Rust structured JSON envelope and add old practical fields under `data`.
  Rationale: The repository has already documented the envelope as an intentional breaking wire-format change, while requiring information compatibility for migrated commands.
  Date/Author: 2026-04-24 / Codex

- Decision: Put `ohlcv --count`, `range`, `scroll`, optional `symbol`, optional `timeframe`, and status/state/quote field completion in one slice.
  Rationale: These commands are tightly coupled in the downstream provider flow; splitting them would leave the provider migration blocked.
  Date/Author: 2026-04-24 / Codex

## Outcomes & Retrospective

The read/provider slice is implemented. The CLI now supports raw OHLCV bars, count-bounded OHLCV summaries, visible range read/set, scroll-to-date, read-only symbol/timeframe, and richer status/state/quote payloads under the existing Rust JSON envelope.

Automated validation passed with `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test`, `git diff --check`, and an absolute-path scan of tracked docs. Live smoke testing also passed against a running TradingView Desktop CDP target for `status`, `state`, `quote`, `ohlcv --summary --count 100`, `ohlcv --count 5`, `range`, `symbol`, `timeframe`, and `scroll 2026-03-03`; the chart range was restored afterward with `range --from ... --to ...`.

## Context and Orientation

The Rust CLI is implemented as a single binary named `tv`. `src/cli.rs` defines command-line parsing with `clap`. `src/main.rs` dispatches commands and wraps results with `src/output.rs`. `src/ops.rs` owns TradingView command behavior by evaluating JavaScript through the `RuntimeEvaluator` trait from `src/cdp.rs`. `src/transport.rs` discovers the TradingView Desktop Chrome DevTools Protocol target.

Chrome DevTools Protocol, or CDP, is the local debugging protocol exposed by TradingView Desktop when it is launched with a remote debugging port. This CLI connects to the CDP target and evaluates JavaScript inside the TradingView page.

The old JavaScript CLI returned payload fields at the top level. This Rust CLI returns payload fields under `data`. For example, old `tv quote` returned `{ "success": true, "symbol": "AAPL" }`, while Rust returns `{ "success": true, "command": "quote", "data": { "symbol": "AAPL" } }`.

## Plan of Work

First, extend `src/cli.rs`. Add `count: Option<usize>` to `Ohlcv`, make `Symbol` and `Timeframe` accept optional positional values, and add `Range` and `Scroll` commands. `Range` takes optional `--from` and `--to` values as floating-point seconds so validation can reject non-finite values. `Scroll` takes one positional date or Unix timestamp string.

Next, update `src/main.rs` dispatch. `ohlcv` should support either raw bars or summary. `symbol` and `timeframe` should read current values when no argument is supplied. `range` should reject only one of `--from` or `--to`; with both it mutates visible range, with neither it reads visible range. `scroll` requires a non-empty date string.

Then, expand `src/ops.rs`. Add helper functions for reading bars, summarizing bars, reading chart state with studies, reading status with chart API availability, reading and setting visible range, scrolling to a date, and read-only symbol/timeframe commands. Clamp OHLCV count to `1..=500`. Preserve existing fields where possible and add old JavaScript practical fields such as `resolution`, `chartType`, `studies`, `period`, `range`, `change`, `change_pct`, `avg_volume`, `last_5_bars`, `time`, and `close`.

Finally, update tests and docs. Tests must prove the new command parsing and operation contracts without requiring a live TradingView Desktop. Documentation must move the implemented commands in the migration inventory and record remaining gaps.

## Concrete Steps

Run commands from the repository root.

After editing, run:

    cargo fmt --check
    cargo clippy --all-targets --all-features
    cargo test
    git diff --check

If TradingView Desktop is already running with CDP enabled, run smoke checks:

    cargo run -- status
    cargo run -- state
    cargo run -- quote
    cargo run -- ohlcv --summary --count 100
    cargo run -- ohlcv --count 5
    cargo run -- range
    cargo run -- scroll 2026-03-03
    cargo run -- symbol
    cargo run -- timeframe

When smoke commands mutate chart range, save the old `range` result first and restore it with `range --from <old_from> --to <old_to>` if possible.

## Validation and Acceptance

The plan is accepted when the Rust CLI supports `ohlcv --count`, `range`, `scroll`, read-only `symbol`, and read-only `timeframe`, and when status/state/quote/ohlcv responses expose the practical information described in the migration note under `data`.

Automated acceptance requires `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test`, and `git diff --check` to pass. The CLI contract tests must cover the new argument surface and structured error behavior.

Live acceptance, when a TradingView Desktop target is available, is that the smoke commands print JSON envelopes with `success: true`, and `ohlcv --count 5` returns five or fewer recent bars under `data.bars`.

## Idempotence and Recovery

The implementation is additive. Running tests repeatedly should not change tracked files. Smoke commands may change the active chart symbol, timeframe, or visible range only when explicitly invoked; this plan's smoke list avoids symbol/timeframe mutation. If a live smoke command fails because TradingView Desktop is not running or not on a chart page, keep the automated validation result and record the smoke blocker.

## Artifacts and Notes

Important source evidence:

    old JavaScript CLI: src/cli/commands/chart.js, src/cli/commands/data.js, src/core/chart.js, src/core/data.js
    Rust CLI: src/cli.rs, src/main.rs, src/ops.rs, src/cdp.rs
    migration policy: docs/notes/rust-cli-contract-migration-2026-04-24.md

## Interfaces and Dependencies

No new Rust crates are required.

`src/cli.rs` must expose:

    tv ohlcv [--summary] [--count <N>]
    tv range [--from <UNIX_SECONDS> --to <UNIX_SECONDS>]
    tv scroll <DATE_OR_UNIX_SECONDS>
    tv symbol [SYMBOL]
    tv timeframe [RESOLUTION]

`src/ops.rs` must expose operation functions that return `serde_json::Value` payloads. The output layer remains unchanged and wraps payloads in `SuccessEnvelope`.

`RuntimeEvaluator` remains the test seam for JavaScript evaluation. Tests must continue using fake evaluators rather than requiring a running TradingView Desktop.

## Open Questions

No critical open questions block this implementation. Later slices still need decisions for `values`, `watchlist get`, `pane list`, `search`, and chart-region screenshot.

Revision note: initial plan for the first read/provider migration slice after the Rust CLI migration policy was documented.
