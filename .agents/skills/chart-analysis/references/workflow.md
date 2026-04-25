# Chart Analysis Workflow Reference

## Original MCP Intent

The original chart-analysis skill reviewed a TradingView chart by setting symbol/timeframe, adding indicators, navigating ranges, drawing annotations, taking screenshots, reading quote/OHLCV data, and cleaning temporary chart objects.

## Current Rust CLI Mapping

| Original MCP capability | Rust CLI status |
| --- | --- |
| `chart_get_state` | `tv state` |
| `chart_set_symbol` | `tv symbol <SYMBOL>` |
| `chart_set_timeframe` | `tv timeframe <RESOLUTION>` |
| Chart type read/set | `tv type [CHART_TYPE]` |
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
| Indicator lifecycle | `tv indicator add/remove/toggle/set` |
| Drawing lifecycle | `tv draw shape/list/get/remove/clear` |
| Replay controls | `tv replay status/start/step/autoplay/trade/stop` |
| `capture_screenshot` | `tv screenshot --region full|chart --output <PATH>` |

## Remaining Gaps

Strategy tester panel screenshots and arbitrary historical indicator-series computation are not implemented in the Rust CLI. Prefer implemented high-level commands before generic UI automation.

The MCP server itself is not planned.
