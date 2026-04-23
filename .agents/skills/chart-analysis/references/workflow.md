# Chart Analysis Workflow Reference

## Original MCP Intent

The original chart-analysis skill reviewed a TradingView chart by setting symbol/timeframe, adding indicators, navigating ranges, drawing annotations, taking screenshots, reading quote/OHLCV data, and cleaning temporary chart objects.

## Current Rust CLI Mapping

| Original MCP capability | Rust CLI status |
| --- | --- |
| `chart_get_state` | `tv state` |
| `chart_set_symbol` | `tv symbol <SYMBOL>` |
| `chart_set_timeframe` | `tv timeframe <RESOLUTION>` |
| `chart_scroll_to_date` | `tv scroll <DATE>` |
| `chart_set_visible_range` | `tv range --from <UNIX_SECONDS> --to <UNIX_SECONDS>` |
| `chart_get_visible_range` | `tv range` |
| `quote_get` | `tv quote` |
| `symbol_info` | `tv info` |
| `data_get_ohlcv` | `tv ohlcv --summary` or `tv ohlcv --count <N>` |
| `data_get_study_values` | `tv values` |
| Pine line levels | `tv data lines [--filter <TEXT>] [--verbose]` |
| Pine labels | `tv data labels [--filter <TEXT>] [--max <N>] [--verbose]` |
| Pine tables | `tv data tables [--filter <TEXT>]` |
| Pine boxes | `tv data boxes [--filter <TEXT>] [--verbose]` |
| `capture_screenshot` | `tv screenshot --region full|chart --output <PATH>` |

## Not Yet Available

Adding/removing indicators, changing indicator inputs, drawing annotations, removing drawings, and strategy tester screenshots are not implemented in the Rust CLI. Treat these as migration backlog unless the project later excludes them explicitly.

The MCP server itself is not planned.
