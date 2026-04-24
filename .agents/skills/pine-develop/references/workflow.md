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
| Compile/check Pine script in TradingView | `tv pine compile` for editor compile; server-side checks are not implemented |
| Save/open Pine script in TradingView | Not implemented |
| Read Pine editor diagnostics | `tv pine errors` |
| Read Pine console/log output | `tv pine console` |
| List saved Pine scripts | `tv pine list` |

## Working Pattern Today

Use the CLI for chart context, visual evidence, Pine read context, editor-buffer source replacement, and live editor compile verification. Write or review Pine Script in normal local files, push it to the Pine Editor with `tv pine set` when useful, then run `tv pine compile` when a TradingView Desktop session is available. Be clear that compile verification may add or update a chart-local study, and that it is not a save operation.

Pine raw compile, save, new/open, offline analyze, and server-side check automation remain migration backlog. The MCP server itself is not planned.
