# TradingView CLI

TradingView CLI is a planned Rust-native replacement for the current TradingView MCP Bridge usage in sibling trading-analysis projects.

This project is inspired by and planned as a Rust-native successor to practical workflows built around [TradingView MCP Bridge](https://github.com/tradesdontlie/tradingview-mcp) by `tradesdontlie`. That project established the useful bridge pattern this repository is now narrowing into a CLI-first tool. This repository is not affiliated with TradingView Inc.

## Current status

This repository is intentionally in docs-seed mode.

The goal of this first seed is not to start implementation immediately. The goal is to give the next engineer or agent a clean place to continue the work without dragging bridge-replacement planning back into another repository.

## Purpose

The intended project is a CLI-first tool that focuses on the practical capabilities currently needed from the existing TradingView bridge:

- bounded TradingView Desktop connectivity
- reliable command-line interaction
- capability surfaces that can later support provider, review, and operator workflows

The replacement is expected to be Rust-native, CLI-centered, and narrower than a full MCP-compatible reimplementation.

An MCP server is not planned for this project. Downstream integration should start through ordinary process invocation and JSON CLI output rather than by recreating the original MCP server surface.

## Non-goals for this seed

- no Rust implementation yet
- no copied JavaScript bridge code
- no full feature parity promise
- no release packaging
- no skill migration yet

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
