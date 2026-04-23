# Pine Develop Workflow Reference

## Original MCP Intent

The original pine-develop skill supported a TradingView Pine Script loop: pull the current script, edit locally, push it back, compile/check it in TradingView, and iterate on errors.

## Current Rust CLI Mapping

| Original MCP capability | Rust CLI status |
| --- | --- |
| Chart context | `tv state`, `tv info`, `tv quote`, `tv ohlcv --summary` |
| Visible chart evidence | `tv values`, `tv screenshot --region chart --output <PATH>` |
| Pull Pine script from editor | Not implemented |
| Push Pine script to editor | Not implemented |
| Compile/check Pine script in TradingView | Not implemented |
| Save/open Pine script in TradingView | Not implemented |

## Working Pattern Today

Use the CLI for chart context and visual evidence. Write or review Pine Script in normal local files, and clearly say when TradingView compile verification has not been performed.

Pine editor automation is migration backlog. The MCP server itself is not planned.
