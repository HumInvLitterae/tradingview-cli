# Pine Develop Workflow Reference

## Original MCP Intent

The original pine-develop skill supported a TradingView Pine Script loop: pull the current script, edit locally, push it back, compile/check it in TradingView, and iterate on errors.

## Current Rust CLI Mapping

| Original MCP capability | Rust CLI status |
| --- | --- |
| Chart context | `tv state`, `tv info`, `tv quote`, `tv ohlcv --summary` |
| Visible chart evidence | `tv values`, `tv screenshot --region chart --output <PATH>` |
| Pull Pine script from editor | `tv pine get` |
| Push Pine script to editor | `tv pine set --file <PATH>` or stdin |
| Offline Pine static analysis | `tv pine analyze --file <PATH>` or stdin |
| Compile/check Pine script in TradingView | `tv pine check --file <PATH>` for server-side check; `tv pine compile` for live editor compile |
| Broad old compile behavior | `tv pine raw-compile` |
| Open saved Pine script in TradingView | `tv pine open <NAME...>` |
| Create new Pine script buffer | `tv pine new [indicator|strategy|library]` |
| Save existing Pine script in TradingView | `tv pine save` |
| Read Pine editor diagnostics | `tv pine errors` |
| Read Pine console/log output | `tv pine console` |
| List saved Pine scripts | `tv pine list` |

## Working Pattern Today

Use the CLI for chart context, visual evidence, Pine read context, source validation, editor-buffer source replacement, saved-script open, fresh template creation, explicit save, and live editor compile verification. Write or review Pine Script in normal local files, run `tv pine analyze` for quick offline checks, run `tv pine check` for TradingView server-side compile validation, open or create the editor buffer with `tv pine open`, `tv pine new`, or `tv pine set` when useful, then run `tv pine compile` when a TradingView Desktop session is available. Use `tv pine raw-compile` only when the user explicitly accepts that it preserves the old broad behavior and may click save-related Pine actions. Use `tv pine save` only when the user wants TradingView cloud persistence for the current saved script. Be clear that live editor compile verification may add or update a chart-local study, and that it is not a save operation. Named new-script save remains deferred because the TradingView naming dialog can be outside the CDP page target.

The MCP server itself is not planned.
