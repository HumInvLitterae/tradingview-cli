---
name: replay-practice
description: Prepare or assist TradingView replay practice while respecting current Rust `tv` CLI gaps. Use when the user asks for replay setup, bar-by-bar drills, trade journal practice, or migration notes from the old MCP replay workflow.
---

# Replay Practice

Use this skill to support replay-style practice with the Rust `tv` CLI, while keeping replay automation boundaries explicit.

## Current Reality

The Rust `tv` CLI does not currently start replay mode, step bars, autoplay replay, read replay status, record replay trades, or stop replay. Those old MCP replay capabilities remain migration backlog.

## Useful CLI Workflow

1. Set up the chart with `tv symbol <SYMBOL>` and `tv timeframe <RESOLUTION>`.
2. Move near the practice area with `tv scroll <DATE>` or `tv range --from <UNIX_SECONDS> --to <UNIX_SECONDS>`.
3. Gather context with `tv state`, `tv quote`, and `tv ohlcv --summary`.
4. Capture a starting chart image with `tv screenshot --region chart --output <PATH>`.
5. If the user operates TradingView replay manually, use `tv state` and screenshots to document what is visible.

## Reporting

Frame the output as a practice plan, observation log, or debrief. Do not represent manual replay actions as automated CLI actions.

Read `references/workflow.md` when the task needs old MCP replay command mapping or future migration notes.
