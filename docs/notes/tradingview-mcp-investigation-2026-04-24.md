# TradingView MCP investigation 2026-04-24

This note records confirmed facts from the migration source used to plan the first Rust-native `tv` CLI. It intentionally summarizes external findings instead of relying on local paths or private memory.

## Confirmed facts

The migration source is [`tradesdontlie/tradingview-mcp`](https://github.com/tradesdontlie/tradingview-mcp), a Node.js project named `tradingview-mcp`. Its `package.json` exposes an MCP server at `src/server.js` and a CLI binary named `tv` at `src/cli/index.js`. Runtime dependencies are `@modelcontextprotocol/sdk` and `chrome-remote-interface`.

The original JavaScript bridge connected to TradingView Desktop through Chrome DevTools Protocol on `localhost:9222`. Its connection layer fetched `/json/list`, chose a target whose URL looked like a TradingView chart, enabled `Runtime`, `Page`, and `DOM`, then evaluated JavaScript against TradingView's Electron page. The important internal paths included `window.TradingViewApi._activeChartWidgetWV.value()`, `window.TradingViewApi._chartWidgetCollection`, and the main series bars path under `_chartWidget.model().mainSeries().bars()`.

Rust update: the current Rust CLI defaults to `127.0.0.1:9222` and does not send initial `Runtime.enable`, `Page.enable`, or `DOM.enable` calls during CDP connection because newer TradingView Desktop / Electron builds can hang on those bootstrap calls.

The existing CLI surface is broad. Top-level commands include `status`, `launch`, `state`, `symbol`, `timeframe`, `type`, `info`, `search`, `range`, `scroll`, `discover`, `ui-state`, `quote`, `ohlcv`, `values`, `screenshot`, plus grouped commands for `data`, `pine`, `draw`, `alert`, `watchlist`, `indicator`, `layout`, `pane`, `tab`, `replay`, `stream`, and `ui`. This confirms that the Rust replacement should not inherit the whole surface by default.

The CLI router prints JSON for command results and uses exit code `2` for messages matching CDP, connection, `ECONNREFUSED`, or not-running failures. Other errors exit `1`. The router does not currently define a structured error object with machine-readable error kinds.

The screenshot implementation uses CDP `Page.captureScreenshot`. A full-page screenshot is straightforward. Chart-region screenshots depend on DOM selectors such as `[data-name="pane-canvas"]`, `[class*="chart-container"]`, or `canvas` and then pass a clip rectangle to CDP. This makes `chart` region support useful but less stable than `full`.

The local migration source had uncommitted changes at investigation time. The meaningful code change was dependency injection for `getVisibleRange`, `scrollToDate`, and `symbolInfo` in `src/core/chart.js`, with corresponding tests in `tests/sanitization.test.js`. This is evidence that the old bridge's chart operations benefited from an injectable evaluation boundary for testability. There was also a lockfile update adding the `tv` bin entry and an untracked `tv` file.

The targeted migration-source test command `node --test tests/sanitization.test.js` passed with 69 tests. The tests cover `safeString`, finite-number validation, sanitized chart and drawing evaluate calls, source audit checks against unsafe interpolation patterns, path traversal prevention, and the new injected-evaluator tests. An accidental broader `npm test -- --test-reporter=spec tests/sanitization.test.js` run started e2e tests and was interrupted; it had already shown live-environment-dependent failures around `tv_launch` and `ui_open_panel`, which reinforces the decision to keep Rust v1 narrow.

## Default hypotheses

The first Rust implementation should keep the CLI binary name `tv` because existing usage already expects that name.

The first implementation should be a single binary crate, not a multi-crate workspace. Splitting crates before the command boundary is proven would add structure before there is enough code to justify it.

The first CDP implementation should prefer a thin HTTP plus WebSocket plus JSON-RPC implementation over a higher-level browser automation crate. That keeps the implementation close to the actual behavior needed by this project: connect to an already-running TradingView Desktop target and call a small set of CDP methods.

The Rust implementation should have a mockable runtime-evaluation boundary. The old bridge's local `_deps` fix indicates that direct calls from operations into global CDP functions make useful unit tests harder.

## Implementation contracts to carry forward

The successor ExecPlan should define structured JSON success and error envelopes before any Rust code is written.

The successor ExecPlan should define distinct exit codes for usage/internal errors, connection failures, TradingView internal API unavailability, and timeout.

The successor ExecPlan should state that MCP server implementation is not planned for this project. This is stronger than excluding it from v1; ordinary process invocation and JSON CLI output are the intended integration path.

The successor ExecPlan should treat `screenshot --region full` as the v1 screenshot target and require a separate spike before advertising `chart` region screenshots.

The successor ExecPlan should avoid exact dependency-version lock-in in prose. Cargo semver requirements belong in `Cargo.toml`; exact resolved versions belong in `Cargo.lock` once implementation starts.
