# Build the first Rust-native TradingView CLI

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with `.agents/PLANS.md`. It is self-contained and assumes the reader has only this repository checkout.

## Purpose / Big Picture

After this plan is implemented, a user will have a Rust-native `tv` command that connects to an already-running TradingView Desktop instance through Chrome DevTools Protocol and performs a narrow, reliable set of CLI-first operations. The first usable outcome is not feature parity with the JavaScript bridge. The outcome is a small command-line tool that can report connection health, read chart state and market data, perform basic symbol and timeframe changes, and capture a full screenshot while returning predictable JSON and exit codes.

Chrome DevTools Protocol, abbreviated CDP in this document, is the debugging protocol exposed by Chromium and Electron applications. TradingView Desktop is an Electron application, so when it is launched with a remote debugging port, this CLI can connect to the local debug endpoint and evaluate JavaScript inside the TradingView chart page. This project is not affiliated with TradingView Inc.; users remain responsible for complying with TradingView's terms and subscription requirements.

This project is inspired by [`tradesdontlie/tradingview-mcp`](https://github.com/tradesdontlie/tradingview-mcp), also referred to here as the migration source. That project proves the useful pattern, but this repository deliberately narrows the surface to a Rust-native CLI.

## Progress

- [x] (2026-04-24 03:45 JST) Read repository seed docs and `.agents/PLANS.md`.
- [x] (2026-04-24 03:45 JST) Investigated the migration source package structure, CLI router, CDP connection layer, chart/data/screenshot commands, and local uncommitted fixes.
- [x] (2026-04-24 03:45 JST) Added `docs/notes/tradingview-mcp-investigation-2026-04-24.md` with confirmed facts and implementation hypotheses.
- [x] (2026-04-24 03:45 JST) Added README attribution to the migration source.
- [x] (2026-04-24 03:59 JST) Create the Rust package skeleton for a single `tv` binary.
- [x] (2026-04-24 04:02 JST) Implement the CLI argument surface and common JSON envelopes.
- [x] (2026-04-24 04:06 JST) Implement target discovery and CDP transport.
- [ ] Implement `status`, `state`, `quote`, `ohlcv --summary`, `symbol`, `timeframe`, and `screenshot --region full`.
- [ ] Add unit and CLI integration tests for command contracts, CDP behavior, and error mapping.
- [ ] Run the full validation commands and record results in this plan.

## Surprises & Discoveries

- Observation: The migration source has a very broad CLI and MCP surface, with many commands that are useful historically but too large for this repository's first milestone.
  Evidence: Its CLI registers `status`, `launch`, `state`, `symbol`, `timeframe`, `type`, `info`, `search`, `range`, `scroll`, `discover`, `ui-state`, `quote`, `ohlcv`, `values`, `screenshot`, and grouped commands for `data`, `pine`, `draw`, `alert`, `watchlist`, `indicator`, `layout`, `pane`, `tab`, `replay`, `stream`, and `ui`.

- Observation: Full screenshot support is substantially simpler than chart-region screenshot support.
  Evidence: The migration source uses CDP `Page.captureScreenshot` directly for full screenshots, while chart-region screenshots depend on DOM selectors and clip rectangle calculation.

- Observation: The local migration source had uncommitted changes improving testability for chart operations.
  Evidence: `getVisibleRange`, `scrollToDate`, and `symbolInfo` accept injected dependencies, and `node --test tests/sanitization.test.js` passed 69 tests including the new injected-evaluator tests.

- Observation: Running the migration source through `npm test -- --test-reporter=spec tests/sanitization.test.js` accidentally appended arguments to the existing e2e-heavy npm script instead of limiting the run to one file.
  Evidence: The run started live e2e suites and was interrupted after showing failures in live-environment-dependent checks such as `tv_launch` and `ui_open_panel`.

- Observation: The CDP transport needed two small support crates beyond the initial dependency hypothesis.
  Evidence: `futures-util` provides WebSocket stream and sink helpers used by `tokio-tungstenite`, and `base64` decodes the `Page.captureScreenshot` response body.

## Decision Log

- Decision: Build a Rust-native CLI and do not plan an MCP server implementation.
  Rationale: The repository goal is a bounded CLI replacement, and downstream integration should start with process invocation and JSON output. Recreating the old MCP server surface is not only outside v1; it is not a planned project target.
  Date/Author: 2026-04-24 / Codex

- Decision: Keep the binary name `tv` for the first Rust CLI.
  Rationale: The migration source and downstream usage already use `tv`, so preserving the name reduces migration friction while still allowing the implementation and scope to change.
  Date/Author: 2026-04-24 / Codex

- Decision: Start with one binary crate rather than a workspace split.
  Rationale: The v1 surface is small. A workspace can be introduced later if real module boundaries justify it.
  Date/Author: 2026-04-24 / Codex

- Decision: Implement CDP as a small HTTP plus WebSocket JSON-RPC layer unless implementation disproves feasibility.
  Rationale: The required behavior is connecting to an already-running local Electron target and issuing a small number of CDP methods. Higher-level browser automation crates are unnecessary for the v1 boundary.
  Date/Author: 2026-04-24 / Codex

- Decision: Make operation code depend on a mockable runtime evaluator boundary.
  Rationale: The migration source's local `_deps` change shows that operations become easier to test when evaluation is injectable. Rust should carry this forward as an explicit design rule.
  Date/Author: 2026-04-24 / Codex

- Decision: Include `screenshot --region full` in v1 and require a later spike before advertising `chart` region screenshots.
  Rationale: Full screenshots map directly to CDP. Chart-region clipping depends on TradingView DOM selectors and should not be promised until proven stable.
  Date/Author: 2026-04-24 / Codex

## Outcomes & Retrospective

The research and planning portion of this ExecPlan is complete: the migration source has been inspected, the README now attributes the upstream project, and the implementation boundary is narrow enough for coding to begin. Update this section again after each implementation milestone with what was built, what passed validation, and any changes to the command contract.

## Context and Orientation

This repository currently contains documentation and agent instructions, not Rust source code. The seed planning document is `docs/plans/tradingview-cli-bootstrap-and-bridge-replacement.md`. The investigation note is `docs/notes/tradingview-mcp-investigation-2026-04-24.md`. This document is the first implementation-ready plan.

The migration source is [`tradesdontlie/tradingview-mcp`](https://github.com/tradesdontlie/tradingview-mcp). It is a Node.js project with an MCP server and a CLI. Its CLI binary is named `tv`; its CDP connection layer talks to `localhost:9222`, fetches `/json/list`, selects a TradingView page target, enables `Runtime`, `Page`, and `DOM`, and evaluates JavaScript in the page. Its important TradingView internal path for chart operations is `window.TradingViewApi._activeChartWidgetWV.value()`.

The Rust v1 should implement only `status`, `state`, `quote`, `ohlcv --summary`, `symbol`, `timeframe`, and `screenshot --region full`. It must not implement Pine editing, panes, tabs, alerts, watchlists, replay, streaming, arbitrary UI automation, or an MCP server.

## Plan of Work

First, create a normal Rust binary package in the repository root with a binary named `tv`. Use Rust edition 2024 and a `rust-toolchain.toml` that selects stable. Add a `Cargo.lock` when dependencies are resolved. Use semver dependency requirements in `Cargo.toml`; do not hard-code exact crate versions in prose or comments.

Use these runtime dependencies unless implementation shows a concrete reason to change them: `clap` for command parsing, `tokio` for async runtime, `reqwest` for HTTP calls to CDP endpoints, `tokio-tungstenite` for CDP WebSocket transport, `serde` and `serde_json` for JSON, `tracing` and `tracing-subscriber` for diagnostic logging, and `thiserror` for typed errors. Use `assert_cmd`, `predicates`, and `tempfile` as dev dependencies for CLI tests and temporary screenshot paths.

The implementation also uses `futures-util` for WebSocket stream and sink utilities and `base64` for decoding CDP screenshot data. These are implementation-support dependencies rather than new user-facing capabilities.

Organize the code into modules with these responsibilities. The `cli` module owns `clap` argument definitions and maps commands to operation calls. The `output` module owns the common JSON success and error envelopes. The `transport` module owns fetching `/json/list`, target selection, WebSocket connection setup, and connection retry policy. The `cdp` module owns CDP JSON-RPC request IDs, response correlation, event handling, timeouts, close handling, and `Runtime.evaluate`. The `ops` module owns command behavior and calls a small evaluator trait instead of talking to raw WebSocket code directly.

Define a mockable evaluator boundary before implementing operations. A suitable shape is a trait named `RuntimeEvaluator` with an async method that evaluates a JavaScript expression and returns `serde_json::Value`, plus a method or associated path for CDP screenshot capture where needed. The concrete CDP client implements this trait. Unit tests use fake implementations.

Implement output contracts before command internals. Every successful command prints only JSON to stdout with this shape:

    {
      "success": true,
      "command": "status",
      "data": {}
    }

Every failed command prints only JSON to stderr with this shape:

    {
      "success": false,
      "command": "status",
      "error": {
        "kind": "connection",
        "message": "CDP connection failed",
        "details": null
      }
    }

Use these exit codes: `0` for success, `1` for usage, validation, or unexpected internal errors, `2` for TradingView or CDP connection failure, `3` for TradingView internal API unavailability, and `4` for timeout.

Implement target discovery with deterministic ambiguity handling. Prefer page targets whose URL contains `tradingview.com/chart`. If exactly one preferred target exists, use it. If no preferred target exists but exactly one page target contains `tradingview`, use it. If multiple candidates remain, `tv status` returns the candidates in JSON and reports `connected: false`; other commands fail with error kind `target_ambiguous`.

Implement command behavior with the following minimum fields. `tv status` returns `connected`, `target_id`, `target_url`, `target_title`, `cdp_host`, and `cdp_port` when connected, or candidates and an error description when not connected. `tv state` returns `symbol`, `timeframe`, `chart_type`, and `visible_range` when available. `tv quote` returns `symbol`, `last`, `open`, `high`, `low`, and `volume`, allowing nullable numeric fields when TradingView does not expose them. `tv ohlcv --summary` returns `symbol`, `timeframe`, `bar_count`, `first_time`, `last_time`, `open`, `high`, `low`, `close`, and `volume`. `tv symbol <SYMBOL>` and `tv timeframe <RESOLUTION>` return the requested value and the observed value after the operation. `tv screenshot --region full --output <PATH>` writes a PNG and returns `output_path`, `region`, `size_bytes`, and target metadata.

For JavaScript evaluation, centralize string and numeric safety. Use JSON serialization for user-provided strings before they enter JavaScript expressions. Validate numeric inputs as finite before using them. Do not hand-roll quote escaping at operation call sites.

## Concrete Steps

Run all commands from the repository root.

Create the Rust package skeleton:

    cargo init --bin --name tradingview-cli .

Adjust `Cargo.toml` so the binary exposed to users is named `tv`. If `cargo init` creates a package name that differs from the binary name, use a `[[bin]]` section with `name = "tv"` and `path = "src/main.rs"`.

Add `rust-toolchain.toml` selecting the stable toolchain:

    [toolchain]
    channel = "stable"

Add dependencies with `cargo add` or by editing `Cargo.toml`:

    cargo add clap --features derive
    cargo add tokio --features macros,rt-multi-thread,time,net
    cargo add reqwest --features json
    cargo add tokio-tungstenite
    cargo add serde --features derive
    cargo add serde_json
    cargo add tracing
    cargo add tracing-subscriber --features env-filter
    cargo add thiserror
    cargo add --dev assert_cmd predicates tempfile

Create these source modules: `src/cli.rs`, `src/output.rs`, `src/error.rs`, `src/transport.rs`, `src/cdp.rs`, and `src/ops.rs`. Keep `src/main.rs` thin: initialize logging, parse CLI arguments, dispatch to the command handler, print JSON, and exit with the mapped code.

Implement tests as the behavior is added. Put CLI tests under `tests/cli_contract.rs` and module tests next to their modules. Tests must not require a running TradingView Desktop unless explicitly named as ignored smoke tests.

After implementation, run:

    cargo fmt --check
    cargo clippy --all-targets --all-features
    cargo test

For manual smoke testing, launch TradingView Desktop with a remote debugging port, then run:

    cargo run -- status
    cargo run -- state
    cargo run -- quote
    cargo run -- ohlcv --summary
    cargo run -- symbol AAPL
    cargo run -- timeframe D
    cargo run -- screenshot --region full --output target/tv-full.png

The smoke commands should print JSON and the screenshot command should create a PNG file. If TradingView is not running with CDP enabled, the commands should fail with structured JSON and exit code `2`.

## Validation and Acceptance

The implementation is accepted when `cargo fmt --check`, `cargo clippy --all-targets --all-features`, and `cargo test` pass, and when manual smoke testing proves that the CLI can connect to a local TradingView Desktop chart and run the v1 commands.

The CLI contract is accepted when every command prints valid JSON, success envelopes always use `success: true`, error envelopes always use `success: false`, and exit codes match the contract in this plan.

The CDP layer is accepted when tests cover response ID correlation with interleaved events, timeout mapping, connection-refused mapping, `Runtime.evaluate` exception mapping, WebSocket close handling, and multiple target ambiguity.

Screenshot support is accepted for v1 when `--region full` writes a PNG and reports a positive byte count. Do not advertise `--region chart` until a separate spike proves a stable DOM clipping strategy.

## Idempotence and Recovery

The implementation should be additive. Re-running `cargo test` or smoke commands should not change tracked files. Screenshot smoke output should go under `target/` or another ignored path.

If `cargo init` refuses to run because package files already exist, inspect the existing files and continue with the existing package instead of overwriting them. If dependency resolution changes exact versions, accept the resolver output in `Cargo.lock` and keep `Cargo.toml` on semver requirements.

If TradingView Desktop is not running or lacks a remote debugging port, do not attempt to kill or relaunch it in v1. Return a structured connection error and document manual launch instructions in README.

## Artifacts and Notes

The migration-source investigation note is `docs/notes/tradingview-mcp-investigation-2026-04-24.md`.

The targeted migration-source test run used this command:

    node --test tests/sanitization.test.js

It passed 69 tests. The relevant evidence is that injected evaluator tests passed for `getVisibleRange`, `scrollToDate`, and `symbolInfo`, and source audit tests passed for unsafe interpolation checks.

An accidental broad migration-source test command started e2e tests and was interrupted:

    npm test -- --test-reporter=spec tests/sanitization.test.js

This demonstrated that the old project's e2e surface is live-environment dependent and should not drive the Rust v1 boundary.

## Interfaces and Dependencies

The public interface is the `tv` CLI. The first public commands are `status`, `state`, `quote`, `ohlcv --summary`, `symbol <SYMBOL>`, `timeframe <RESOLUTION>`, and `screenshot --region full --output <PATH>`.

The internal interface that matters most is the runtime evaluator. Define it so operation tests can run without a TradingView Desktop process. The concrete CDP client should be replaceable by fake evaluators in unit tests.

The project must not introduce an MCP server interface. If a future integration need appears, it must be justified in a new plan after the CLI has proven useful.

## Open Questions

No critical open questions block implementation. The following non-blocking questions should be answered by later work:

- Whether chart-region screenshot can be made stable enough for a post-v1 command.
- Whether launch automation belongs in this CLI after v1, or should remain external runbook material.
- Whether downstream consumers need additional read-only commands after the initial provider and operator workflows exercise the CLI.

Revision note: created after migration-source investigation to turn the docs-seed repository into an implementation-ready Rust v1 CLI plan. The plan deliberately states that MCP server implementation is not planned, fixes the v1 command contract, and carries forward testability lessons from the migration source.
