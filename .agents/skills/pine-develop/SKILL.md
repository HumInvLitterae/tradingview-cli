---
name: pine-develop
description: Plan or support Pine Script work while respecting current Rust `tv` CLI gaps. Use when the user asks for Pine Script development, TradingView editor automation, compile checks, or migration notes from the old MCP Pine workflow.
---

# Pine Develop

Use this skill for Pine Script assistance around TradingView while staying honest about the current Rust CLI boundary.

## Current Reality

The Rust `tv` CLI can now read from the Pine Editor with `tv pine get`, `tv pine errors`, and `tv pine console`, can set the current editor buffer with `tv pine set`, can compile the current editor buffer with `tv pine compile`, and can list saved scripts with `tv pine list`.

The Rust `tv pine set` command changes only the local Pine Editor buffer. The Rust `tv pine compile` command compiles the current editor buffer and may add or update a chart-local study, but it intentionally refuses save-related buttons and does not save scripts in TradingView. The Rust `tv` CLI still does not create new Pine scripts, open saved Pine scripts, run raw compile helpers, run offline Pine analysis, or run server-side Pine checks. Those old MCP workflow capabilities remain migration backlog, not completed CLI features.

## Useful CLI Context

1. Use `tv status` and `tv state` to confirm the active chart context.
2. Use `tv info`, `tv quote`, and `tv ohlcv --summary` to gather market context for a script idea.
3. Use `tv values` to inspect already-visible study values when that helps validate behavior.
4. Use `tv screenshot --region chart --output <PATH>` when visual evidence helps discuss the script.
5. Use `tv pine get`, `tv pine set`, `tv pine compile`, `tv pine errors`, `tv pine console`, and `tv pine list` for Pine Editor context when TradingView Desktop is available.

## Pine Work

Write, review, or refactor Pine Script in normal project files when asked. You may use `tv pine set --file <PATH>` or stdin to place source into the editor, then use `tv pine compile` for TradingView editor compile verification when a live desktop session is available. Do not claim the script was saved in TradingView unless that was verified by another tool or by the user.

Read `references/workflow.md` when the task asks for the old MCP Pine workflow, current gaps, or future CLI migration notes.
