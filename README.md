# TradingView CLI

TradingView CLI is a Rust-native command-line replacement for the current TradingView MCP Bridge usage in sibling trading-analysis projects.

This project is inspired by practical workflows built around [TradingView MCP Bridge](https://github.com/tradesdontlie/tradingview-mcp) by `tradesdontlie`. That project established the useful bridge pattern this repository is now narrowing into a CLI-first tool. This repository is not affiliated with TradingView Inc.

## Current status

This repository now contains the first Rust-native `tv` CLI implementation.

The first implementation focuses on a narrow CLI surface for connecting to an already-running TradingView Desktop instance through Chrome DevTools Protocol on `localhost:9222`.

The broader TradingView MCP Bridge CLI migration is still in progress. Commands that are not implemented yet should be treated as migration backlog unless a repository decision explicitly marks them out of scope. The MCP server is different: implementing an MCP server is not planned for this project.

## Purpose

This is a CLI-first tool that focuses on the practical capabilities currently needed from the existing TradingView bridge:

- bounded TradingView Desktop connectivity
- reliable command-line interaction
- capability surfaces that can later support provider, review, and operator workflows

The replacement is Rust-native, CLI-centered, and narrower than a full MCP-compatible reimplementation.

An MCP server is not planned for this project. Downstream integration should start through ordinary process invocation and JSON CLI output rather than by recreating the original MCP server surface.

## Compatibility policy

This Rust CLI is intended to replace practical usage of the old `tv` CLI over time, but it is not a drop-in JSON wire-format clone.

The Rust CLI intentionally uses stable command envelopes:

```json
{
  "success": true,
  "command": "quote",
  "data": {
    "symbol": "NASDAQ:AAPL"
  }
}
```

Errors use the same envelope shape with structured details:

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

The old JavaScript CLI usually returned command fields at the top level, for example `{ "success": true, "symbol": "NASDAQ:AAPL" }`. Downstream adapters must therefore read command payloads from `data` when migrating to this Rust CLI.

The wire shape may differ, but information compatibility is required for migrated commands: information available from the old CLI should remain available from the Rust CLI once the corresponding command is implemented. New fields may be added. Removing old practical information requires an explicit decision and migration note.

## Non-goals

- no copied JavaScript bridge code
- no all-at-once feature parity promise
- no release packaging
- no MCP server implementation

## Quick Start

Launch TradingView Desktop with Chrome DevTools Protocol enabled:

```bash
/path/to/TradingView --remote-debugging-port=9222
```

Then run the Rust CLI:

```bash
cargo build
cargo run -- status
cargo run -- state
cargo run -- info
cargo run -- search AAPL
cargo run -- quote
cargo run -- values
cargo run -- discover
cargo run -- ui-state
cargo run -- ohlcv --summary --count 100
cargo run -- ohlcv --count 5
cargo run -- range
cargo run -- watchlist get
cargo run -- pane list
cargo run -- data strategy
cargo run -- data trades --max 5
cargo run -- data equity
cargo run -- data lines --verbose
cargo run -- data labels --max 5
cargo run -- data tables
cargo run -- data boxes
cargo run -- symbol BATS:IONQ
cargo run -- symbol
cargo run -- timeframe 15
cargo run -- timeframe
cargo run -- type Candles
cargo run -- type
cargo run -- scroll 2026-03-03
cargo run -- screenshot --region full --output target/tv-full.png
cargo run -- screenshot --region chart --output target/tv-chart.png
```

For local shell use, install the binary from the repository root:

```bash
cargo install --path .
tv status
```

The default CDP endpoint is `localhost:9222`. Override it with `TV_CDP_HOST` and `TV_CDP_PORT` when needed.

All commands print structured JSON. Successful commands print a `success: true` envelope to stdout. Failed commands print a `success: false` envelope to stderr.

Exit codes are:

- `0`: success
- `1`: usage, validation, target ambiguity, or unexpected internal failure
- `2`: TradingView or CDP connection failure
- `3`: TradingView internal API unavailable
- `4`: timeout

## What is included

- a first bootstrap ExecPlan
- a migration-source investigation note
- a first Rust v1 implementation ExecPlan
- a Rust v1 `tv` CLI implementation
- a post-v1 handoff prompt
- Rust CLI contract and command migration notes
- a first read/provider migration ExecPlan and implementation slice
- a read utilities migration ExecPlan and implementation slice
- a chart-region screenshot ExecPlan and implementation slice
- a diagnostic read commands ExecPlan and implementation slice
- an advanced data reads ExecPlan and implementation slice
- a chart type ExecPlan and implementation slice
- an operation-layer module refactor ExecPlan and implementation slice
- a repo-local development guideline for module layout, style, and validation
- repo-local CLI skills migrated from the original MCP workflow split

## Where to start

Read these in order:

1. `docs/notes/next-agent-handoff-prompt-2026-04-24.md`
2. `docs/notes/development-guidelines-2026-04-24.md`
3. `docs/notes/rust-cli-contract-migration-2026-04-24.md`
4. `docs/notes/legacy-cli-command-migration-inventory-2026-04-24.md`
5. `docs/plans/tradingview-cli-ops-module-refactor-v1-7.md`
6. `docs/plans/tradingview-cli-chart-type-v1-6.md`
7. `docs/plans/tradingview-cli-advanced-data-reads-v1-5.md`
8. `docs/plans/tradingview-cli-diagnostic-read-commands-v1-4.md`
9. `docs/plans/tradingview-cli-chart-region-screenshot-v1-3.md`
10. `docs/plans/tradingview-cli-read-utilities-v1-2.md`
11. `docs/plans/tradingview-cli-read-provider-migration-v1-1.md`
12. `docs/plans/tradingview-cli-rust-v1.md`
13. `docs/notes/tradingview-mcp-investigation-2026-04-24.md`
14. `docs/plans/tradingview-cli-bootstrap-and-bridge-replacement.md`

The first capability and boundary research milestone, the Rust v1 implementation milestone, the first read/provider migration slice, the read utilities migration slice, the chart-region screenshot slice, the diagnostic read commands slice, the advanced data reads slice, the chart type slice, the operation-layer module refactor, and the development guideline pass are complete. The next milestone is migration readiness: keep the improved Rust JSON contract documented, preserve information compatibility for migrated commands, and choose any deferred old CLI surfaces only after evidence shows they belong in this CLI.
