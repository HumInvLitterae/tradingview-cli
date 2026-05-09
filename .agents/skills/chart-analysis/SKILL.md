---
name: chart-analysis
description: Analyze a live TradingView chart with the Rust `tv` CLI. Use when the user wants chart review, technical analysis, quote/OHLCV context, study values, symbol/timeframe setup, visible range checks, or screenshot-backed chart evidence.
---

# Chart Analysis

Use this skill for live TradingView chart review through the Rust `tv` CLI.
This is primarily a Desktop-backed read workflow: the selected TradingView
Desktop chart, visible studies, chart bars, and screenshots are the source of
truth. Use Desktop-free reads only for one-symbol snapshot context, quick symbol
metadata, scanner-backed quotes, or fundamentals before mutating chart state.

For the current command-choice map, use `docs/observation-workflows.md`. Keep
this skill focused on chart analysis rather than re-listing every `tv`
observation surface.

## Start With Readiness

1. Run `tv readiness`.
2. Inspect `ready`, `target_selection`, `selected_target.target_cli_args`,
   `chart_readiness`, `bars_readiness`, and `next_action_hint` before using
   visual fallback. If there is no connection,
   run `tv launch` once. If it still cannot connect, explain that the user must
   launch TradingView with a remote debugging port or provide
   `tv launch --path <PATH>`.
3. If multiple chart targets are open or the connected chart is unclear, run
   `tv tab list` and use the desired target's `target_cli_args`, for example
   `tv --target-id <ID> quote`, for follow-up chart commands. Do not use
   `TV_CDP_TARGET_ID`.
4. Use `tv state` or `tv ohlcv --count 1` only when `tv readiness` shows that
   a specific chart or bars detail still needs follow-up. For portable visual
   evidence, use `tv screenshot --region chart --output <PATH>` rather than
   assuming an external visual-control tool exists.
5. Run `tv discover` and `tv ui-state` when the chart surface itself is unclear.

## Core Workflow

1. For one-symbol static context before mutating the chart, prefer
   Desktop-free `tv snapshot <SYMBOL>`. It combines quote, symbol info, and
   fundamentals sections. Snapshot follow-up metadata uses the same stable
   vocabulary as compare: `chart_quote`, `observe_chart`, and `screenshot`
   name evidence surfaces, not recommendations or automatic reads. Use
   lower-level `tv quote <SYMBOL>` and `tv info <SYMBOL>` only when that
   narrower read is enough. Scanner-backed
   quotes expose `time`, `update_mode`, `delay_seconds`, and extended-hours
   fields when TradingView returns them, but they are not a realtime entitlement
   guarantee.
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
   `tv quote <SYMBOL>` use Desktop-free reads by default. Preserve
   `source_category`, `requires_desktop`, and `non_mutating` when comparing
   Desktop-backed chart reads with Desktop-free scanner reads.
5. Use `tv quote <SYMBOL> --source chart` when the selected Desktop chart feed
   matters, and `tv quote <SYMBOL> --source auto` when chart-first behavior
   with scanner fallback is acceptable. Do not add manual sleep or double-call
   loops around chart-source quotes; the CLI handles bounded readiness waiting
   with consecutive stable samples and returns a structured failure if fresh
   chart bars do not arrive. Do not treat chart-source quote as premarket or
   postmarket evidence; read `session_boundary`, and use scanner-backed
   `tv quote`, `tv quotes`, `tv snapshot`, or `tv compare` when extended-hours
   fields matter. Desktop page quote-session probes are separate experimental
   evidence and should not be substituted for chart main-series quote or
   scanner extended-hours values unless phase-specific live evidence has
   confirmed the semantics. Postmarket `market-status.phase=post-market` is
   useful source evidence, but it is not enough by itself to treat quote-session
   pre/post close fields as scanner-style extended-hours values.
   The visible right-side symbol detail panel can show a distinct after-hours
   price that is not in chart main-series quote or the current quote-session
   selected fields. Treat that as visible UI evidence until a stable CLI
   payload explicitly exposes it. Current source discovery narrowed the
   observed RKLB postmarket value to the right-side detail widget status/price
   node, but it remains a visible UI source rather than chart-source quote.
   A bounded CDP Network/WebSocket smoke did not find the visible price token
   in captured communication candidates, so do not treat Network traffic as a
   confirmed backing source yet. A later scoped in-page widget probe found the
   right-panel detail widget React chain and regular quote-like props,
   including current session and regular last-price fields, but did not expose
   the visible after-hours price token in compact prop/state hits, so do not
   claim a known widget-store source for the after-hours value.
   A bounded WebSocket correlation smoke later found exact numeric matches for
   sampled visible after-market prices in received WebSocket frame summaries.
   Treat that as source-discovery evidence only; it is not yet a stable field
   schema or `quote --source chart` payload. Follow-up HAR/live evidence makes
   TradingView `qsd.rtc` the strongest current candidate for the visible
   after-market value. Use `tv quote <SYMBOL> --source quote-data` when this
   explicit Desktop-backed quote-data readback is needed. It is not an
   implicit chart main-series quote field and may return structured
   unavailable details if no matching `qsd.rtc` arrives during the bounded
   wait. Treat unavailable quote-data as source availability, not as proof
   that the symbol has no price.
6. Read visible study values with `tv values` when indicators already exist on
   the chart.
7. Read Pine drawing-derived levels or zones with `tv data lines`,
   `tv data labels`, `tv data tables`, or `tv data boxes` when the chart
   includes such primitives.
8. Inspect or adjust the date window with `tv range`, `tv scroll <DATE>`, or
   `tv range --from <UNIX_SECONDS> --to <UNIX_SECONDS>`.
9. Use `tv observe chart --duration-ms 10000 --heartbeat-ms 2000` when the
   task needs a short-lived Desktop-backed window that starts with readiness
   and then follows the selected chart's last bar. Use lower-level bounded
   `tv stream quote`, `tv stream bars`, or `tv stream all` only when a
   specific sample type is needed. Ordinary chart reads should use `tv quote`
   and `tv ohlcv --summary`. JSONL sample events should carry
   `source_category: "desktop_backed_read"` and `non_mutating: true`; use
   those fields to keep them separate from Desktop-free scanner reads.
10. Capture visual evidence only when useful: `tv screenshot --region chart --output <PATH>`.
    Treat screenshot payloads as Desktop-backed visual evidence reads; they are
    `non_mutating: true` but `writes_file: true`.

Use `market-data-interpretation` when source selection, scanner delay metadata,
extended-hours fields, or chart-vs-scanner differences matter.

## OHLCV Recovery

If `tv ohlcv` fails but `tv quote` or `tv symbol` works, keep the full JSON
error envelope and inspect `error.kind`, `error.details.phase`,
`error.details.chart_readiness` / `bar_index_state`, and `next_action_hint`
instead of piping through `head` or `tail`. Then rerun `tv tab list`, choose the
active chart target's `target_cli_args`, run `tv --target-id <ID> state`, and
retry `tv --target-id <ID> ohlcv --count 1`. If the structured fields still do
not explain the failure, capture a chart screenshot or ask the user to
foreground/click the chart.

If the current environment is the Codex app and Computer Use is available, it
can be used as an optional visual inspection or UI recovery aid after the
structured CLI checks. Do not make Computer Use part of the default workflow for
Codex CLI, packaged agents, or other CLI-only runtimes.

Use `tv timeframe <RESOLUTION>` for timeframe changes. `tv interval` is not a
command. Use `tv snapshot <SYMBOL>` for Desktop-free one-symbol context before
chart mutation, `tv info <SYMBOL>` for narrower Desktop-free symbol metadata,
and `tv info` without a symbol only when you need metadata for the current
chart's loaded symbol.

## Reporting

Lead with the practical market read, then cite the observed CLI evidence. Separate observed data from inference, and avoid inventing indicator values that were not returned by `tv values` or visible in an inspected screenshot.

## Boundaries

This Rust CLI can launch or reconnect to TradingView Desktop, read chart state, perform chart navigation, inspect chart-model data, manage indicators and drawings, list and switch saved chart layouts, work with Pine Editor state, use replay controls, stream read-only chart samples as JSONL, and run `tv observe chart` for readiness-plus-last-bar observation. Generic UI automation exists for compatibility, but prefer higher-level commands and use generic UI commands only after explicit user approval.

Read `references/workflow.md` when the task needs an old MCP-to-CLI command mapping or a reminder of unsupported chart-analysis features.
