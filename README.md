# TradingView CLI

TradingView CLI is a planned Rust-native replacement for the current TradingView MCP Bridge usage in sibling trading-analysis projects.

## Current status

This repository is intentionally in docs-seed mode.

The goal of this first seed is not to start implementation immediately. The goal is to give the next engineer or agent a clean place to continue the work without dragging bridge-replacement planning back into another repository.

## Purpose

The intended project is a CLI-first tool that focuses on the practical capabilities currently needed from the existing TradingView bridge:

- bounded TradingView Desktop connectivity
- reliable command-line interaction
- capability surfaces that can later support provider, review, and operator workflows

The replacement is expected to be Rust-native, CLI-centered, and narrower than a full MCP-compatible reimplementation.

## Non-goals for this seed

- no Rust implementation yet
- no copied JavaScript bridge code
- no full feature parity promise
- no release packaging
- no skill migration yet

## What is included

- a first bootstrap ExecPlan
- a next-agent handoff prompt
- a clean Git repo boundary for the future implementation

## Where to start

Read these in order:

1. `docs/plans/tradingview-cli-bootstrap-and-bridge-replacement.md`
2. `docs/notes/next-agent-handoff-prompt-2026-04-21.md`

The first planned milestone is capability and boundary research, not coding.
