# Split oversized ops module without changing CLI behavior

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with `.agents/PLANS.md`.

## Purpose / Big Picture

After this plan is implemented, the Rust-native `tv` CLI will have the same user-visible behavior but a clearer operation-layer structure. Today `src/ops.rs` is more than two thousand lines and contains every command implementation, JavaScript expression, helper, screenshot cropper, and operation unit test. That makes the next old-CLI migration slice riskier than it needs to be.

This refactor splits the operation layer by capability while preserving the public `ops::...` functions that `src/main.rs` already calls. A user should observe no command-line or JSON contract changes. A contributor should be able to add a later command to an obvious module instead of extending a single growing file.

## Progress

- [x] (2026-04-24 15:33 JST) Read the current continuity ledger, checked a clean working tree, measured source sizes, inspected `src/ops.rs` function boundaries, and confirmed baseline `cargo test` plus `cargo clippy --all-targets --all-features` passed.
- [x] (2026-04-24 15:40 JST) Keep `src/ops.rs` as the facade and move implementation bodies into `src/ops/` submodules.
- [x] (2026-04-24 15:43 JST) Extract shared helpers and feature-specific operation modules.
- [x] (2026-04-24 15:47 JST) Move tests to the relevant modules with shared test support.
- [x] (2026-04-24 16:02 JST) Run validation and live smoke checks against TradingView Desktop CDP.
- [x] (2026-04-24 16:07 JST) Record outcomes and commit the implementation refactor; this documentation update follows as the companion commit.

## Surprises & Discoveries

- Observation: `src/ops.rs` is currently 2,474 lines, while the next largest source file is `src/cdp.rs` at 269 lines.
  Evidence: `wc -l src/*.rs tests/*.rs` before this refactor.

- Observation: The module boundaries are already visible in the function order.
  Evidence: `src/ops.rs` groups status, chart state/control, diagnostics, OHLCV, advanced data reads, watchlist/pane reads, screenshots, helpers, and tests in one file.

## Decision Log

- Decision: Keep `src/ops.rs` as the facade instead of using `src/ops/mod.rs`.
  Rationale: This project uses Rust 2024 conventions, and the user explicitly requested avoiding `mod.rs`. `src/main.rs` and CLI behavior should not need to change in a behavior-preserving refactor.
  Date/Author: 2026-04-24 / Codex

- Decision: Split by command capability instead of by implementation mechanism.
  Rationale: Future migration work asks "where does this command belong?", so modules should map to user-visible surfaces: chart, market, diagnostics, data, layout, screenshot, and status.
  Date/Author: 2026-04-24 / Codex

- Decision: Do not fix unrelated behavior during this refactor.
  Rationale: The acceptance criterion is unchanged behavior. Any suspected bug or contract issue discovered during extraction should be recorded for later work.
  Date/Author: 2026-04-24 / Codex

## Outcomes & Retrospective

Implementation extraction is complete. The original `src/ops.rs` was reduced from 2,474 lines to a thin 26-line facade, with implementation code split under `src/ops/` by capability. No `src/ops/mod.rs` was created.

The post-extraction validation baseline passed: `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test`, and `git diff --check`. `cargo test` passed all 39 unit tests and 17 CLI contract tests, and test names now show feature module ownership such as `ops::chart::tests`, `ops::data::tests`, `ops::market::tests`, and `ops::screenshot::tests`.

Live/read smoke checks against TradingView Desktop CDP also passed for `status`, `state`, `quote`, `type`, `ohlcv --count 2`, `discover`, `ui-state`, `watchlist get`, `pane list`, and `screenshot --region chart --output target/refactor-smoke-chart.png`.

## Context and Orientation

The Rust CLI is a single binary named `tv`. `src/cli.rs` parses commands with `clap`. `src/main.rs` dispatches commands and wraps successful results in `src/output.rs`. `src/cdp.rs` defines `RuntimeEvaluator`, the trait used to evaluate JavaScript inside TradingView Desktop through Chrome DevTools Protocol, or CDP. `src/transport.rs` discovers the TradingView CDP target.

Before this refactor, `src/ops.rs` is the operation layer. It contains public functions such as `status`, `state`, `quote`, `ohlcv_bars`, `set_symbol`, `current_chart_type`, `data_strategy`, `watchlist_get`, `pane_list`, and `screenshot_chart`. It also contains helper functions and unit tests for all of those operations.

The important public contract is the module facade, not the file layout. `src/main.rs` should continue calling `ops::status`, `ops::state`, `ops::quote`, and the other existing operation functions. The CLI JSON envelope and command payloads must remain unchanged.

## Plan of Work

First, keep `src/ops.rs` as the operation facade and create submodules under `src/ops/`. `src/ops.rs` should declare the submodules and re-export the public operation functions so callers do not need to know the internal file layout. Do not use `src/ops/mod.rs`.

Next, extract shared constants and helpers into `src/ops/common.rs`. This includes TradingView internal JavaScript paths such as `CHART_API`, small validation and formatting helpers such as `js_string`, `round2`, `require_finite`, and object merge helpers used by status.

Then, extract operations by capability. `status.rs` owns CDP target status. `chart.rs` owns chart state/control commands including symbol, timeframe, type, range, scroll, and symbol metadata. `market.rs` owns quote, OHLCV, and TradingView symbol search. `diagnostics.rs` owns `discover` and `ui-state`. `data.rs` owns study values, indicator reads, strategy/trade/equity reads, and Pine drawing-derived read commands. `layout.rs` owns `watchlist get` and `pane list`. `screenshot.rs` owns full/chart screenshot capture and crop helpers.

Finally, split unit tests alongside the modules they verify. Put shared fake runtime and PNG fixture code in `src/ops/test_support.rs` under `#[cfg(test)]`, and import it from module tests as `super::test_support`.

## Concrete Steps

Run commands from the repository root.

After editing, run:

    cargo fmt --check
    cargo clippy --all-targets --all-features
    cargo test
    git diff --check

If TradingView Desktop is already running with CDP enabled, run read-oriented smoke checks:

    cargo run -- status
    cargo run -- state
    cargo run -- quote
    cargo run -- type
    cargo run -- ohlcv --count 2
    cargo run -- discover
    cargo run -- ui-state
    cargo run -- watchlist get
    cargo run -- pane list
    cargo run -- screenshot --region chart --output target/refactor-smoke-chart.png

Do not add new old-CLI commands in this plan. Do not intentionally change payload field names or command behavior.

## Validation and Acceptance

The refactor is accepted when `src/ops.rs` no longer exists as a monolithic file and the operation layer is split under `src/ops/` by capability.

The public operation names remain available as `ops::status`, `ops::state`, `ops::symbol_info`, `ops::symbol_search`, `ops::quote`, `ops::study_values`, `ops::discover`, `ops::ui_state`, `ops::ohlcv_bars`, `ops::ohlcv_summary`, `ops::set_symbol`, `ops::current_symbol`, `ops::set_timeframe`, `ops::current_timeframe`, `ops::current_chart_type`, `ops::set_chart_type`, `ops::validate_chart_type`, `ops::visible_range`, `ops::set_visible_range`, `ops::scroll_to_date`, `ops::data_indicator`, `ops::data_strategy`, `ops::data_trades`, `ops::data_equity`, `ops::data_lines`, `ops::data_labels`, `ops::data_tables`, `ops::data_boxes`, `ops::watchlist_get`, `ops::pane_list`, `ops::screenshot_full`, and `ops::screenshot_chart`.

Automated acceptance requires `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test`, and `git diff --check` to pass.

## Idempotence and Recovery

This is a behavior-preserving file layout change. Running tests repeatedly should not change tracked files. If a move or extraction fails halfway, use `git status --short` and the pre-refactor committed state to identify changed files, then continue by preserving existing function bodies rather than rewriting behavior.

If a live smoke command fails because TradingView Desktop is not running or a panel is closed, record that as a smoke blocker. Do not treat a live-environment blocker as a refactor failure when automated validation passes.

## Artifacts and Notes

Important source evidence:

    operation layer before refactor: src/ops.rs
    CLI dispatch caller: src/main.rs
    CDP runtime trait: src/cdp.rs
    validation baseline: cargo test and cargo clippy --all-targets --all-features passed before extraction

## Interfaces and Dependencies

No new Rust crates are required.

`src/ops.rs` must declare internal modules and publicly re-export the existing operation functions. `src/ops/mod.rs` must not be created.

`src/ops/common.rs` should be `pub(super)` or `pub(crate)` only where needed by sibling modules. Avoid making helpers part of the public API unless `src/main.rs` already depends on them, as with `validate_chart_type`.

`RuntimeEvaluator` remains the test seam for JavaScript evaluation. Tests must continue using fake evaluators rather than requiring a running TradingView Desktop.

## Open Questions

No critical open questions block this refactor.

Revision note: initial refactor plan after chart type slice completion. The plan deliberately pauses old CLI surface migration to make the current Rust codebase easier to extend safely.

Revision note: adjusted the module layout to keep `src/ops.rs` as the facade and avoid `mod.rs`, per user direction for Rust 2024 style.
