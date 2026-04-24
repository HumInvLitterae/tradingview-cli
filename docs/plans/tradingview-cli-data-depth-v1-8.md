# Add DOM-dependent data depth read command

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with `.agents/PLANS.md`.

## Purpose / Big Picture

After this plan is implemented, the Rust-native `tv` command will cover the old JavaScript CLI's `data depth` read surface. A user or downstream adapter will be able to request bid and ask depth levels from a visible TradingView DOM or Depth of Market panel without returning to the JavaScript bridge.

This is intentionally a small read-only slice. The command depends on TradingView DOM selectors and a visible order book panel, so it should not open panels, mutate UI state, create orders, or promise availability when the panel is closed. The Rust CLI keeps the improved JSON envelope `{ success, command, data }`; practical old CLI information remains under `data`.

## Progress

- [x] (2026-04-24 16:17 JST) Read continuity, current CLI/data modules, migration inventory, development guidelines, and the old JavaScript `getDepth()` implementation from `tradesdontlie/tradingview-mcp`.
- [x] (2026-04-24 16:22 JST) Add `tv data depth` CLI and dispatch.
- [x] (2026-04-24 16:23 JST) Implement `data_depth` in a separate operation module.
- [x] (2026-04-24 16:24 JST) Add operation and CLI contract tests.
- [x] (2026-04-24 16:25 JST) Update README, migration inventory, contract note, handoff note, and agent guide.
- [x] (2026-04-24 16:27 JST) Run validation and live smoke against TradingView Desktop CDP.
- [x] (2026-04-24 16:31 JST) Record outcomes and commit implementation as `82f996f feat(data): Add depth read command`; this documentation update follows as the companion commit.

## Surprises & Discoveries

- Observation: The old JavaScript `getDepth()` read is not chart-model based.
  Evidence: `src/core/data.js` in `tradesdontlie/tradingview-mcp` searches DOM selectors containing `depth`, `orderBook`, `dom-`, `DOM`, and `[data-name="dom"]`, then classifies rows as bid or ask.

- Observation: The local development guideline says the next substantial data-related change should avoid further growing `src/ops/data.rs`.
  Evidence: `docs/notes/development-guidelines-2026-04-24.md` names `src/ops/data.rs` as the main watch point and recommends splitting by sub-surface.

- Observation: The available live TradingView Desktop session did not have a visible DOM or Depth of Market panel.
  Evidence: `cargo run -- data depth` connected and returned structured `internal_api_unavailable` with message `DOM / Depth of Market panel not found.`

## Decision Log

- Decision: Implement `data depth` as a read-only DOM-dependent slice.
  Rationale: It is still old CLI migration backlog and is lower risk than mutation surfaces such as alerts, watchlist add, pane mutation, Pine, replay, tab, stream, or UI automation.
  Date/Author: 2026-04-24 / Codex

- Decision: Put the implementation in `src/ops/data_depth.rs` instead of appending to `src/ops/data.rs`.
  Rationale: This follows the new development guideline and keeps the already-large data module from becoming another catch-all.
  Date/Author: 2026-04-24 / Codex

- Decision: Return `internal_api_unavailable` when the DOM or Depth of Market panel is unavailable.
  Rationale: The command cannot produce meaningful depth without a visible panel, and automatically opening panels would turn this slice into UI automation.
  Date/Author: 2026-04-24 / Codex

## Outcomes & Retrospective

The data depth slice is implemented. The CLI now supports `tv data depth` under the existing Rust JSON envelope, with operation code isolated in `src/ops/data_depth.rs` so `src/ops/data.rs` does not grow.

Automated validation passed with `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test`, `git diff --check`, and a tracked-doc local absolute path scan. Unit tests cover success payload mapping, raw numeric fallback, missing panel errors, and unusable panel payloads. CLI contract tests now cover help output and structured connection errors for `tv data depth`.

Live smoke against TradingView Desktop CDP reached the page and returned the expected structured blocker because the DOM or Depth of Market panel was not visible. This validates failure handling, but it does not prove a live bid/ask success payload.

## Context and Orientation

The Rust CLI is a single binary named `tv`. `src/cli.rs` defines command-line parsing with `clap`. `src/main.rs` dispatches parsed commands and wraps successful results with `src/output.rs`. `src/cdp.rs` defines `RuntimeEvaluator`, the trait used to evaluate JavaScript inside TradingView Desktop through Chrome DevTools Protocol, or CDP.

The operation layer uses `src/ops.rs` as a thin facade. Feature implementations live under `src/ops/`; `src/ops/data.rs` currently owns chart-model data reads such as strategy metrics, trades, equity, indicator inputs, and Pine drawing-derived reads. Because `data depth` is a DOM panel scrape rather than a chart-model read, it belongs in a new sibling module named `src/ops/data_depth.rs`.

The old JavaScript CLI returned command payload fields at the top level. This Rust CLI returns command payloads under `data`. For example, old `tv data depth` returned `{ "success": true, "bid_levels": 3 }`, while Rust should return `{ "success": true, "command": "data", "data": { "bid_levels": 3 } }`.

## Plan of Work

First, extend the CLI surface. Add a `Depth` variant to `DataCommand` in `src/cli.rs` with help text that says it reads the visible DOM or Depth of Market panel. Update `src/main.rs` so `DataCommand::Depth` connects to CDP and calls `ops::data_depth`.

Next, add `src/ops/data_depth.rs`. The new operation should evaluate JavaScript equivalent to the old `getDepth()` implementation: find a DOM panel through depth/order-book selectors, read row or table-cell text, classify rows into bids and asks by class or row HTML, sort bids descending and asks ascending, compute spread when both sides exist, and return counts plus levels. If rows cannot be classified but numeric cells exist, return `raw_values` and a note. If the panel is missing or no useful data exists, return `AppError::new(ErrorKind::InternalApiUnavailable, ...)`.

Then, update `src/ops.rs` to declare `mod data_depth;` and re-export `data_depth`.

Finally, update tests and docs. Operation tests should use `FakeRuntime` and should not require TradingView Desktop. CLI contract tests should include `depth` in `tv data --help` and include `tv data depth` in the structured connection error loop. Documentation updates should move `data depth` from deferred backlog to implemented surface and record its DOM-panel dependency.

## Concrete Steps

Run commands from the repository root.

After editing, run:

    cargo fmt --check
    cargo clippy --all-targets --all-features
    cargo test
    git diff --check

Also scan tracked docs for local absolute filesystem paths:

    git grep -nE '(/Users/|C:\\)' -- README.md AGENTS.md docs .agents/skills || true

If TradingView Desktop is already running with CDP enabled, run:

    cargo run -- data depth

If the DOM or Depth of Market panel is closed, a structured `internal_api_unavailable` error is an expected smoke blocker rather than an automated validation failure. If the panel is open and data is readable, the command should print a success envelope whose `data` includes `bid_levels`, `ask_levels`, `spread`, `bids`, and `asks`.

## Validation and Acceptance

The implementation is accepted when `tv data --help` lists `depth`, `tv data depth` attempts a CDP connection like the other data reads, and `ops::data_depth` returns practical old CLI fields under the Rust `data` envelope.

On success, the payload must include `bid_levels`, `ask_levels`, `spread`, `bids`, and `asks`. It may include `raw_values` and `note` when bid/ask classification is not possible but numeric DOM values were found.

On missing panel or unusable panel data, the operation must fail with `internal_api_unavailable` rather than returning a misleading empty success payload.

Automated acceptance requires `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test`, and `git diff --check` to pass.

## Idempotence and Recovery

The implementation is additive. Running tests repeatedly should not change tracked files. The command is read-only and must not change chart symbol, timeframe, range, watchlist content, pane layout, DOM panel visibility, orders, alerts, or Pine editor state.

If live smoke fails because TradingView Desktop is not running, no chart target is available, or the DOM panel is closed, keep the automated validation result and record the smoke blocker in this plan.

## Artifacts and Notes

Important source evidence:

    old JavaScript core: https://github.com/tradesdontlie/tradingview-mcp/blob/main/src/core/data.js
    old JavaScript CLI command group: https://github.com/tradesdontlie/tradingview-mcp/blob/main/src/cli/commands/data.js
    Rust CLI modules: src/cli.rs, src/main.rs, src/ops.rs, src/ops/data.rs, src/cdp.rs
    migration policy: docs/notes/rust-cli-contract-migration-2026-04-24.md
    development guideline: docs/notes/development-guidelines-2026-04-24.md

## Interfaces and Dependencies

No new Rust crates are required.

`src/cli.rs` must expose:

    tv data depth

`src/ops.rs` must expose:

    pub async fn data_depth(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>

`RuntimeEvaluator` remains the test seam for JavaScript evaluation. Tests must continue using fake evaluators rather than requiring a running TradingView Desktop.

## Open Questions

No critical open questions block this implementation. Later work can decide whether other DOM or UI-dependent old surfaces deserve their own slices.

Revision note: initial plan for the DOM-dependent `data depth` migration slice after choosing old CLI migration work over CI setup.
