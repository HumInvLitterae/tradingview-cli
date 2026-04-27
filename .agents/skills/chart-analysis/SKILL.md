---
name: chart-analysis
description: Analyze a live TradingView chart with the Rust `tv` CLI. Use when the user wants chart review, technical analysis, quote/OHLCV context, study values, symbol/timeframe setup, visible range checks, or screenshot-backed chart evidence.
---

# Chart Analysis

Use this skill for live TradingView chart review through the Rust `tv` CLI.

## Start With Readiness

1. Run `tv status`.
2. If there is no connection, run `tv launch` once. If it still cannot connect, explain that the user must launch TradingView with a remote debugging port or provide `tv launch --path <PATH>`.
3. If multiple chart targets are open or the connected chart is unclear, run `tv tab list` and use the desired target's `target_cli_args`, for example `tv --target-id <ID> quote`, for follow-up chart commands. `target_env.TV_CDP_TARGET_ID` is a v0.2.x fallback only.
4. Run `tv discover` and `tv ui-state` when the chart surface itself is unclear.

## Core Workflow

1. Set the requested market context with `tv symbol <SYMBOL>`, `tv timeframe <RESOLUTION>`, and `tv type <CHART_TYPE>` when needed.
2. After changing the chart symbol, confirm fresh chart data with `tv ohlcv --count 1` or `tv ohlcv --summary` before relying on `tv quote`.
3. Read chart context with `tv state`, `tv quote`, and `tv ohlcv --summary`.
   Use `tv quote <SYMBOL>` for a one-off symbol quote when needed; it first
   tries a non-mutating scanner read and may fall back to temporary chart
   switching. `restored: true` only means the original chart symbol was
   restored; quote freshness still depends on the observed symbol matching the
   request.
4. Read visible study values with `tv values` when indicators already exist on the chart.
5. Read Pine drawing-derived levels or zones with `tv data lines`, `tv data labels`, `tv data tables`, or `tv data boxes` when the chart includes such primitives.
6. Inspect or adjust the date window with `tv range`, `tv scroll <DATE>`, or `tv range --from <UNIX_SECONDS> --to <UNIX_SECONDS>`.
7. Use `tv stream quote` or `tv stream bars` only when the task needs short-lived live monitoring; ordinary chart reads should use `tv quote` and `tv ohlcv --summary`.
8. Capture visual evidence only when useful: `tv screenshot --region chart --output <PATH>`.

## Reporting

Lead with the practical market read, then cite the observed CLI evidence. Separate observed data from inference, and avoid inventing indicator values that were not returned by `tv values` or visible in an inspected screenshot.

## Boundaries

This Rust CLI can launch or reconnect to TradingView Desktop, read chart state, perform chart navigation, inspect chart-model data, manage indicators and drawings, list and switch saved chart layouts, work with Pine Editor state, use replay controls, and stream read-only chart samples as JSONL. Generic UI automation exists for compatibility, but prefer higher-level commands and use generic UI commands only after explicit user approval.

Read `references/workflow.md` when the task needs an old MCP-to-CLI command mapping or a reminder of unsupported chart-analysis features.
