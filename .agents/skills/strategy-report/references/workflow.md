# Strategy Report Workflow Reference

## Original MCP Intent

The original strategy-report skill collected strategy tester results, trade lists, equity curves, drawdown data, symbol/chart context, screenshots, and then produced a strategy report.

## Current Rust CLI Mapping

| Original MCP capability | Rust CLI status |
| --- | --- |
| `chart_get_state` | `tv state` |
| `symbol_info` | `tv info` |
| `quote_get` | `tv quote` |
| `data_get_ohlcv` | `tv ohlcv --summary` or `tv ohlcv --count <N>` |
| Visible values | `tv values` when available on chart |
| Chart screenshot | `tv screenshot --region full|chart --output <PATH>` |
| `data_get_strategy_results` | `tv data strategy` |
| `data_get_trades` | `tv data trades --max <N>` |
| `data_get_equity` | `tv data equity` |
| Strategy tester screenshot | Not implemented |

## Working Pattern Today

Use the CLI for chart, market, and strategy context. If the strategy commands return empty metrics or an `error`, report that as an observation rather than filling the gap with guesses.

Strategy tester panel screenshots remain migration backlog. The MCP server itself is not planned.
