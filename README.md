# TradingView CLI

`tv` is a Rust-native command-line tool for TradingView workflows. It combines
TradingView Desktop automation through Chrome DevTools Protocol with
Desktop-free TradingView data reads for quotes, scanner rows, symbol metadata,
and fundamentals.

This project is inspired by practical workflows built around
[TradingView MCP Bridge](https://github.com/tradesdontlie/tradingview-mcp) by
`tradesdontlie`, but this repository is intentionally CLI-first. It is not an
MCP server and is not affiliated with TradingView Inc.

This tool requires the user's own valid TradingView account, data
entitlements, and, for Desktop-backed commands, a local TradingView Desktop
session. It does not bypass TradingView access controls, subscriptions,
paywalls, or exchange/data-provider licensing. Market data, Pine scripts,
alerts, layouts, and account state remain subject to TradingView and
data-provider terms.

## What `tv` Does

`tv` is one binary with several source categories:

- Desktop-free reads: symbol search, symbol info, scanner-backed quote reads,
  batch quotes, fundamentals, scanner scans, hotlists, and metainfo.
- Desktop-backed reads: chart state, OHLCV from the selected chart, screenshots,
  readiness diagnostics, and chart-source quotes.
- Desktop-backed operations: chart symbol/timeframe/type changes, watchlist,
  alerts, drawings, Pine Editor, Replay, Screener, panes, layouts, tabs, and
  compatibility UI automation.
- Hybrid commands: commands with explicit source or fallback behavior, such as
  `tv quote <SYMBOL> --source auto`.
- Experimental commands: lab-gated behavior such as browserless `tv bars`.

See `docs/command-source-taxonomy.md` for the durable command classification
and source/fallback semantics.

## Safety Boundary

Some commands only read public or browser-accessible TradingView data. Other
commands operate the user's local Desktop session or page state. Before using a
command that may change chart, account, editor, Replay, Screener, drawing,
alert, watchlist, or UI state, prefer read-only commands and dry-run modes where
available.

The default Chrome DevTools endpoint is `127.0.0.1:9222`. Override it with
`TV_CDP_HOST` and `TV_CDP_PORT` only when your local setup requires it.

## Installation

GitHub Releases are the first supported binary distribution path. Version tags
such as `v0.11.0` publish native archives like:

- `tv-v0.11.0-x86_64-unknown-linux-gnu.tar.gz`
- `tv-v0.11.0-x86_64-apple-darwin.tar.gz`
- `tv-v0.11.0-aarch64-apple-darwin.tar.gz`
- `tv-v0.11.0-x86_64-pc-windows-msvc.zip`
- `SHA256SUMS`

Each archive contains the `tv` or `tv.exe` binary, `README.md`,
`CHANGELOG.md`, `LICENSE`, user-facing agent guides, and runtime-oriented
TradingView CLI skills. Verify the archive against `SHA256SUMS`, unpack it,
place the executable on your `PATH`, and confirm the binary:

```bash
tv --version
```

During local development, build or install from the workspace root:

```bash
cargo install --path crates/cli
```

Package-manager installers, code signing, notarization, and crates.io
publication are not part of the current release workflow.

## Quick Start

Desktop-free reads do not require TradingView Desktop:

```bash
tv search "Apple"
tv info NASDAQ:AAPL
tv quote AAPL
tv snapshot NASDAQ:AAPL
tv compare NASDAQ:AAPL NYSE:IONQ
tv quotes AAPL MSFT NYSE:IONQ
tv fundamentals NYSE:IONQ --group earnings
tv fundamentals AAPL --group dividends
tv scanner scan --type stock --columns name,close,volume --limit 10
tv scanner metainfo --market america --field close --field premarket_close
```

`tv compare` returns raw per-symbol evidence plus a machine-readable summary
for resolution, section success, and missing-value counts. It does not rank,
score, or recommend symbols. See `docs/observation-workflows.md` for the
practical choice between `quotes`, `compare`, `snapshot`, and chart follow-up
commands.

To use Desktop-backed reads or operations, launch TradingView Desktop with CDP
enabled:

```bash
tv launch
tv readiness
tv tab list
tv state
tv ohlcv --summary --count 100
tv screenshot --region chart --output target/tv-chart.png
```

If multiple TradingView targets are open, use `target_cli_args` returned by
`tv tab list` or `tv readiness`:

```bash
tv --target-id <CDP_TARGET_ID> state
tv --target-id <CDP_TARGET_ID> ohlcv --count 1
```

Common Desktop operations:

```bash
tv symbol NASDAQ:AAPL
tv timeframe 1D
tv watchlist get
tv alert list
tv pine get
tv screener open --full-page
tv screener filters add --name RSI --min 70 --dry-run
tv draw position long --entry-price 100 --stop-loss 95 --take-profit 110
```

Bounded stream observations emit newline-delimited JSON:

```bash
tv observe chart --duration-ms 10000 --heartbeat-ms 2000
tv stream quote --duration-ms 10000 --heartbeat-ms 2000
tv stream bars --max-events 5
```

Use `tv observe chart` when you want one Desktop-backed window that starts
with readiness details and then follows the selected chart's last bar. Use the
lower-level `tv stream ...` commands when you already know which specific chart
sample type you need.

Experimental browserless historical bars are lab-gated:

```bash
TV_EXPERIMENTAL_BARS=1 tv bars NASDAQ:AAPL --timeframe 1D --count 5
```

Use `tv --help` for the full command list and `tv <COMMAND> --help` for command
details. See `docs/observation-workflows.md` for practical command sequences
that combine Desktop-free screening, Desktop-backed chart observation,
screenshots, experimental bars, and fundamentals reads.

## Output Contract

Most successful commands print one JSON envelope to stdout:

```json
{
  "success": true,
  "command": "quote",
  "data": {
    "symbol": "NASDAQ:AAPL"
  }
}
```

Errors use the same envelope shape on stderr:

```json
{
  "success": false,
  "command": "quote",
  "error": {
    "kind": "connection",
    "message": "CDP connection failed",
    "details": null
  }
}
```

`tv stream ...` commands print newline-delimited JSON envelopes. Stream samples
use `_event: "sample"` and optional heartbeats use `_event: "heartbeat"`.

Exit codes are:

- `0`: success
- `1`: usage, validation, target ambiguity, or unexpected internal failure
- `2`: TradingView or CDP connection failure
- `3`: TradingView internal API unavailable
- `4`: timeout

The Rust CLI intentionally differs from the old JavaScript CLI wire format.
Downstream adapters should read command payloads from `data`. For migration
details, see `docs/breaking-changes-from-js-cli.md`.

## Documentation

- `docs/command-source-taxonomy.md`: command source categories, fallback
  boundaries, mutation expectations, and recommended agent use.
- `docs/observation-workflows.md`: practical read sequences for screening,
  chart observation, screenshots, experimental bars, and fundamentals.
- `docs/architecture.md`: workspace architecture, crate boundaries, operation
  adapters, JSON contract, and safety model.
- `docs/rust-api.md`: currently documented typed Rust API boundary for internal
  reusable read crates.
- `docs/development.md`: coding style, validation, tests, and contribution
  workflow.
- `docs/release-packaging.md`: release archive contents and packaging checks.
- `docs/internal-tradingview-apis.md`: public-safe reference for non-public
  TradingView dependencies.
- `docs/v0.8-roadmap.md`: current roadmap direction.

Historical notes and completed ExecPlans live under `docs/notes/` and
`docs/plans/archives/`. They explain how the current surface was built, but
they are not the best starting point for normal use.

## Development

The repository is a Cargo workspace. The `tv` binary is provided by the
`tradingview-cli` package under `crates/cli/`.

Useful local commands:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo metadata --no-deps --format-version 1
```

Optional local Git guardrails can be installed with Git 2.54 or newer:

```bash
mise run hooks:install
```

Without `mise`, run `scripts/install-config-hooks.sh`; on Windows, run
`scripts/install-config-hooks.ps1` from PowerShell. These hooks are local
helpers only. CI remains the authoritative validation baseline.

## License

This project is licensed under the MIT License. See `LICENSE`.
