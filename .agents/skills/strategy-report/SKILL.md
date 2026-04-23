---
name: strategy-report
description: Build or review TradingView strategy reports with current Rust `tv` CLI evidence and explicit gaps. Use when the user asks for strategy tester summaries, trade/equity analysis, report drafting, or migration notes from the old MCP strategy workflow.
---

# Strategy Report

Use this skill for TradingView strategy-report work that can combine available Rust CLI chart evidence with user-provided strategy tester data.

## Current Reality

The Rust `tv` CLI does not currently extract strategy tester results, trade lists, equity curves, drawdown series, or strategy-tester screenshots. Those old MCP strategy commands remain migration backlog.

## Useful CLI Evidence

1. Confirm the active context with `tv status` and `tv state`.
2. Gather market context with `tv info`, `tv quote`, and `tv ohlcv --summary`.
3. Use `tv values` when visible strategy-related studies expose useful values on the chart.
4. Capture chart context with `tv screenshot --region chart --output <PATH>`.
5. Combine CLI evidence with any CSV, screenshot, or exported metrics supplied by the user.

## Reporting

Do not infer missing strategy metrics. Label unavailable metrics clearly, cite user-provided data when used, and keep conclusions separate from evidence.

Read `references/workflow.md` when the task needs old MCP strategy command mapping or future migration notes.
