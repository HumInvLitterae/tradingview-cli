---
name: chart-analysis
description: Analyze a live TradingView chart with the Rust `tv` CLI. Use when the user wants chart review, technical analysis, quote/OHLCV context, study values, symbol/timeframe setup, visible range checks, or screenshot-backed chart evidence.
---

# Chart Analysis

Use this skill for live TradingView chart review through the Rust `tv` CLI.

## Start With Readiness

1. Run `tv status`.
2. If the connected chart is unclear, run `tv discover` and `tv ui-state`.
3. If there is no connection, explain that the user must launch TradingView with a remote debugging port. `tv launch` is not implemented.

## Core Workflow

1. Set the requested market context with `tv symbol <SYMBOL>`, `tv timeframe <RESOLUTION>`, and `tv type <CHART_TYPE>` when needed.
2. Read chart context with `tv state`, `tv quote`, and `tv ohlcv --summary`.
3. Read visible study values with `tv values` when indicators already exist on the chart.
4. Read Pine drawing-derived levels or zones with `tv data lines`, `tv data labels`, `tv data tables`, or `tv data boxes` when the chart includes such primitives.
5. Inspect or adjust the date window with `tv range`, `tv scroll <DATE>`, or `tv range --from <UNIX_SECONDS> --to <UNIX_SECONDS>`.
6. Use `tv stream quote` or `tv stream bars` only when the task needs short-lived live monitoring; ordinary chart reads should use `tv quote` and `tv ohlcv --summary`.
7. Capture visual evidence only when useful: `tv screenshot --region chart --output <PATH>`.

## Reporting

Lead with the practical market read, then cite the observed CLI evidence. Separate observed data from inference, and avoid inventing indicator values that were not returned by `tv values` or visible in an inspected screenshot.

## Boundaries

This Rust CLI can read chart state, perform basic chart navigation, inspect chart-model data, manage individual indicators and drawings, and stream read-only chart samples as JSONL. It does not currently bulk-clear drawings, save Pine scripts, launch TradingView Desktop, or provide generic UI automation.

Read `references/workflow.md` when the task needs an old MCP-to-CLI command mapping or a reminder of unsupported chart-analysis features.
