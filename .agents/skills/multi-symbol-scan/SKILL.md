---
name: multi-symbol-scan
description: Scan or compare a small set of TradingView symbols with the Rust `tv` CLI. Use when the user wants a watchlist-style pass, cross-symbol quote/OHLCV comparison, technical screen, or shortlist based on currently available CLI reads.
---

# Multi-Symbol Scan

Use this skill to compare several TradingView symbols through the Rust `tv` CLI without pretending unavailable batch or watchlist mutation tools exist.

## Start With Scope

1. Confirm the symbol list, timeframe, and screening criteria from the user request.
2. Run `tv status`; if needed, run `tv watchlist get` to inspect the current TradingView watchlist.
3. Keep the first pass small and serial. The Rust CLI does not implement the old MCP `batch_run` helper.

## Scan Workflow

1. Set the timeframe once with `tv timeframe <RESOLUTION>` when the scan uses a shared timeframe.
2. For each symbol, run `tv symbol <SYMBOL>`, then gather `tv quote` and `tv ohlcv --summary`.
3. Use `tv values` only for studies that already exist on the active chart.
4. Capture screenshots selectively for finalists or ambiguous cases with `tv screenshot --region chart --output <PATH>`.
5. Present a ranked shortlist and explain which observations came from CLI data versus visual interpretation.

## Boundaries

The Rust CLI can inspect the current watchlist, but it does not currently add symbols to a watchlist. It also does not compute arbitrary historical indicator series, run strategy batches, or perform the old MCP `data_get_indicator` flow.

Read `references/workflow.md` when the task needs the original MCP scan shape translated into the current CLI surface.
