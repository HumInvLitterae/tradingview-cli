---
name: pine-develop
description: Plan or support Pine Script work while respecting current Rust `tv` CLI gaps. Use when the user asks for Pine Script development, TradingView editor automation, compile checks, or migration notes from the old MCP Pine workflow.
---

# Pine Develop

Use this skill for Pine Script assistance around TradingView while staying honest about the current Rust CLI boundary.

## Current Reality

The Rust `tv` CLI can now read from the Pine Editor with `tv pine get`, `tv pine errors`, and `tv pine console`, can set the current editor buffer with `tv pine set`, can create a fresh editor template with `tv pine new`, can open a saved script into the editor with `tv pine open`, can save the current editor buffer with `tv pine save`, can compile the current editor buffer with `tv pine compile`, can run offline static analysis with `tv pine analyze`, can run TradingView server-side compile checks with `tv pine check`, and can list saved scripts with `tv pine list`.

The Rust `tv pine set`, `tv pine new`, and `tv pine open` commands change only the local Pine Editor buffer. `tv pine new` creates an unsaved indicator, strategy, or library template in the editor. `tv pine open` loads a saved Pine script by exact name or unique partial name into the editor. `tv pine save` explicitly persists the current Pine Editor buffer to TradingView cloud state; use `tv pine save --name <NAME>` for a new unsaved script, and expect existing saved-name conflicts to be rejected. The Rust `tv pine compile` command compiles the current editor buffer and may add or update a chart-local study, but it intentionally refuses save-related buttons and does not save scripts in TradingView. `tv pine analyze` does not require TradingView Desktop or network access. `tv pine check` uses TradingView's pine-facade endpoint but does not connect to CDP or mutate the editor. The Rust `tv` CLI still does not run raw compile helpers. That old MCP workflow capability remains migration backlog, not a completed CLI feature.

## Useful CLI Context

1. Use `tv status` and `tv state` to confirm the active chart context.
2. Use `tv info`, `tv quote`, and `tv ohlcv --summary` to gather market context for a script idea.
3. Use `tv values` to inspect already-visible study values when that helps validate behavior.
4. Use `tv screenshot --region chart --output <PATH>` when visual evidence helps discuss the script.
5. Use `tv pine analyze --file <PATH>` and `tv pine check --file <PATH>` for pre-editor validation.
6. Use `tv pine get`, `tv pine set`, `tv pine new`, `tv pine open`, `tv pine save`, `tv pine compile`, `tv pine errors`, `tv pine console`, and `tv pine list` for Pine Editor context when TradingView Desktop is available.

## Pine Work

Write, review, or refactor Pine Script in normal project files when asked. Use `tv pine analyze --file <PATH>` for offline checks and `tv pine check --file <PATH>` for server-side compile validation when network access is available. You may use `tv pine open <NAME...>` to load a saved script, `tv pine new [indicator|strategy|library]` to start a fresh unsaved template, or `tv pine set --file <PATH>` or stdin to place source into the editor. Then use `tv pine compile` for live TradingView editor verification when a desktop session is available. Use `tv pine save [--name <NAME>]` only when the user wants TradingView cloud persistence, and do not claim the script was saved unless that command succeeded or the user verified it.

Read `references/workflow.md` when the task asks for the old MCP Pine workflow, current gaps, or future CLI migration notes.
