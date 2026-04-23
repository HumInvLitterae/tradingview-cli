# TradingView CLI

TradingView CLI is a Rust-native command-line replacement for the current TradingView MCP Bridge usage in sibling trading-analysis projects.

This project is inspired by practical workflows built around [TradingView MCP Bridge](https://github.com/tradesdontlie/tradingview-mcp) by `tradesdontlie`. That project established the useful bridge pattern this repository is now narrowing into a CLI-first tool. This repository is not affiliated with TradingView Inc.

## Current status

This repository now contains the first Rust-native `tv` CLI implementation.

The first implementation focuses on a narrow CLI surface for connecting to an already-running TradingView Desktop instance through Chrome DevTools Protocol on `localhost:9222`.

## Purpose

This is a CLI-first tool that focuses on the practical capabilities currently needed from the existing TradingView bridge:

- bounded TradingView Desktop connectivity
- reliable command-line interaction
- capability surfaces that can later support provider, review, and operator workflows

The replacement is Rust-native, CLI-centered, and narrower than a full MCP-compatible reimplementation.

An MCP server is not planned for this project. Downstream integration should start through ordinary process invocation and JSON CLI output rather than by recreating the original MCP server surface.

## Non-goals

- no copied JavaScript bridge code
- no full feature parity promise
- no release packaging
- no skill migration yet

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
cargo run -- quote
cargo run -- ohlcv --summary
cargo run -- symbol BATS:IONQ
cargo run -- timeframe 15
cargo run -- screenshot --region full --output target/tv-full.png
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

## Where to start

Read these in order:

1. `docs/notes/next-agent-handoff-prompt-2026-04-24.md`
2. `docs/plans/tradingview-cli-rust-v1.md`
3. `docs/notes/tradingview-mcp-investigation-2026-04-24.md`
4. `docs/plans/tradingview-cli-bootstrap-and-bridge-replacement.md`

The first capability and boundary research milestone and the Rust v1 implementation milestone are complete. The next milestone is operational readiness: keep the documentation accurate, exercise the CLI in real downstream workflows, and choose any post-v1 feature only after recording evidence in a new plan.
