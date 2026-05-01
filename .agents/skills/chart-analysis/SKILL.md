---
name: chart-analysis
description: Analyze a live TradingView chart with the Rust `tv` CLI. Use when the user wants chart review, technical analysis, quote/OHLCV context, study values, symbol/timeframe setup, visible range checks, or screenshot-backed chart evidence.
---

# Chart Analysis

Use this skill for live TradingView chart review through the Rust `tv` CLI.

## Start With Readiness

1. Run `tv status`.
2. Inspect `desktop_readiness`, `target_cli_args`, `chart_readiness`, and
   `next_action_hint` before using visual fallback. If there is no connection,
   run `tv launch` once. If it still cannot connect, explain that the user must
   launch TradingView with a remote debugging port or provide
   `tv launch --path <PATH>`.
3. If multiple chart targets are open or the connected chart is unclear, run
   `tv tab list` and use the desired target's `target_cli_args`, for example
   `tv --target-id <ID> quote`, for follow-up chart commands. Do not use
   `TV_CDP_TARGET_ID`.
4. Run `tv state` to confirm chart API and bars readiness before asking for
   visual confirmation. Use Computer Use only when structured readiness fields
   do not explain the visible chart state or the user explicitly needs visual
   UI recovery.
5. Run `tv discover` and `tv ui-state` when the chart surface itself is unclear.

## Core Workflow

1. For a one-off symbol quote or metadata check, prefer Desktop-free
   `tv quote <SYMBOL>` and `tv info <SYMBOL>` before mutating the chart.
   Scanner-backed quotes expose `time`, `update_mode`, `delay_seconds`, and
   extended-hours fields when TradingView returns them, but they are not a
   realtime entitlement guarantee.
2. Set the requested chart context with `tv symbol <SYMBOL>`,
   `tv timeframe <RESOLUTION>`, and `tv type <CHART_TYPE>` only when OHLCV,
   visible studies, drawings, screenshots, or current-chart metadata are
   needed.
3. After changing the chart symbol, confirm fresh chart data with
   `tv ohlcv --count 1` or `tv ohlcv --summary` before relying on
   current-chart reads.
4. Read chart context with `tv state`, `tv info`, `tv quote`, and
   `tv ohlcv --summary`. `tv info` without a symbol and `tv quote` without a
   symbol read the current chart; `tv info <SYMBOL>` and
   `tv quote <SYMBOL>` use Desktop-free reads by default.
5. Use `tv quote <SYMBOL> --source chart` when the selected Desktop chart feed
   matters, and `tv quote <SYMBOL> --source auto` when chart-first behavior
   with scanner fallback is acceptable. Do not add manual sleep or double-call
   loops around chart-source quotes; the CLI handles bounded readiness waiting
   and returns a structured failure if fresh chart bars do not arrive.
6. Read visible study values with `tv values` when indicators already exist on
   the chart.
7. Read Pine drawing-derived levels or zones with `tv data lines`,
   `tv data labels`, `tv data tables`, or `tv data boxes` when the chart
   includes such primitives.
8. Inspect or adjust the date window with `tv range`, `tv scroll <DATE>`, or
   `tv range --from <UNIX_SECONDS> --to <UNIX_SECONDS>`.
9. Use `tv stream quote` or `tv stream bars` only when the task needs
   short-lived live monitoring; ordinary chart reads should use `tv quote` and
   `tv ohlcv --summary`.
10. Capture visual evidence only when useful: `tv screenshot --region chart --output <PATH>`.

Use `market-data-interpretation` when source selection, scanner delay metadata,
extended-hours fields, or chart-vs-scanner differences matter.

## OHLCV Recovery

If `tv ohlcv` fails but `tv quote` or `tv symbol` works, keep the full JSON
error envelope and inspect `error.kind`, `error.details.phase`,
`error.details.chart_readiness` / `bar_index_state`, and `next_action_hint`
instead of piping through `head` or `tail`. Then rerun `tv tab list`, choose the
active chart target's `target_cli_args`, run `tv --target-id <ID> state`, and
retry `tv --target-id <ID> ohlcv --count 1`. Ask the user to foreground or
click the chart, or use Computer Use to inspect the visible surface, only after
target reselection and structured readiness inspection do not explain the
failure.

Use `tv timeframe <RESOLUTION>` for timeframe changes. `tv interval` is not a
command. Use `tv info <SYMBOL>` for Desktop-free symbol metadata, and use
`tv info` without a symbol only when you need metadata for the current chart's
loaded symbol.

## Reporting

Lead with the practical market read, then cite the observed CLI evidence. Separate observed data from inference, and avoid inventing indicator values that were not returned by `tv values` or visible in an inspected screenshot.

## Boundaries

This Rust CLI can launch or reconnect to TradingView Desktop, read chart state, perform chart navigation, inspect chart-model data, manage indicators and drawings, list and switch saved chart layouts, work with Pine Editor state, use replay controls, and stream read-only chart samples as JSONL. Generic UI automation exists for compatibility, but prefer higher-level commands and use generic UI commands only after explicit user approval.

Read `references/workflow.md` when the task needs an old MCP-to-CLI command mapping or a reminder of unsupported chart-analysis features.
