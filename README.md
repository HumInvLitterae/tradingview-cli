# TradingView CLI

TradingView CLI is a planned Rust-native replacement for the current TradingView MCP Bridge usage in sibling trading-analysis projects.

This project is inspired by and planned as a Rust-native successor to practical workflows built around [TradingView MCP Bridge](https://github.com/tradesdontlie/tradingview-mcp) by `tradesdontlie`. That project established the useful bridge pattern this repository is now narrowing into a CLI-first tool. This repository is not affiliated with TradingView Inc.

## Current status

This repository now contains the first Rust-native `tv` CLI implementation.

The first implementation focuses on a narrow CLI surface for connecting to an already-running TradingView Desktop instance through Chrome DevTools Protocol on `localhost:9222`.

## Purpose

The intended project is a CLI-first tool that focuses on the practical capabilities currently needed from the existing TradingView bridge:

- bounded TradingView Desktop connectivity
- reliable command-line interaction
- capability surfaces that can later support provider, review, and operator workflows

The replacement is expected to be Rust-native, CLI-centered, and narrower than a full MCP-compatible reimplementation.

An MCP server is not planned for this project. Downstream integration should start through ordinary process invocation and JSON CLI output rather than by recreating the original MCP server surface.

## Non-goals for this seed

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
cargo run -- status
cargo run -- state
cargo run -- quote
cargo run -- ohlcv --summary
cargo run -- screenshot --region full --output target/tv-full.png
```

The default CDP endpoint is `localhost:9222`. Override it with `TV_CDP_HOST` and `TV_CDP_PORT` when needed.

## What is included

- a first bootstrap ExecPlan
- a migration-source investigation note
- a first Rust v1 implementation ExecPlan
- a next-agent handoff prompt
- a clean Git repo boundary for the future implementation

## Where to start

Read these in order:

1. `docs/plans/tradingview-cli-bootstrap-and-bridge-replacement.md`
2. `docs/notes/tradingview-mcp-investigation-2026-04-24.md`
3. `docs/plans/tradingview-cli-rust-v1.md`
4. `docs/notes/next-agent-handoff-prompt-2026-04-21.md`

The first capability and boundary research milestone is complete. The next milestone is implementing the Rust v1 CLI from `docs/plans/tradingview-cli-rust-v1.md`.
