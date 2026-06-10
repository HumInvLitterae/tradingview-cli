---
name: strategy-report
description: Build or review TradingView strategy reports with current Rust `tv` CLI evidence and explicit gaps. Use when the user asks for strategy tester summaries, trade/equity analysis, report drafting, or migration notes from the old MCP strategy workflow.
---

# Strategy Report

Use this skill for TradingView strategy-report work that combines Rust CLI chart evidence with strategy metrics, trades, and equity data when those are available from the active chart.

## Current Reality

The Rust `tv` CLI can read strategy metrics, trades, equity-style data, and Strategy Tester panel screenshots when TradingView exposes them. It still cannot guarantee full equity-curve availability when TradingView only exposes summary metrics.

## Useful CLI Evidence

1. Confirm the active context with `tv status` and `tv state`.
2. Gather market context with `tv info`, `tv quote`, and `tv ohlcv --summary`.
3. Read strategy evidence with `tv data strategy`, `tv data trades --max <N>`, and `tv data equity`.
4. Use `tv values` when visible strategy-related studies expose useful values on the chart.
5. Capture chart context with `tv screenshot --region chart --output <PATH>`.
6. Capture Strategy Tester panel context with `tv screenshot --region strategy --output <PATH>` when the visible panel image matters.

## Reporting

Do not infer missing strategy metrics. If a command returns an empty payload or `error` such as no strategy found, label that plainly and keep conclusions separate from evidence.

Read `references/workflow.md` when the task needs old MCP strategy command mapping or future migration notes.
