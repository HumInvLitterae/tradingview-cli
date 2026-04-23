# Replay Practice Workflow Reference

## Original MCP Intent

The original replay-practice skill automated TradingView bar replay: set up the chart, start replay at a point in time, step or autoplay bars, record practice trades, inspect replay status, and stop replay.

## Current Rust CLI Mapping

| Original MCP capability | Rust CLI status |
| --- | --- |
| `chart_set_symbol` | `tv symbol <SYMBOL>` |
| `chart_set_timeframe` | `tv timeframe <RESOLUTION>` |
| `chart_scroll_to_date` | `tv scroll <DATE>` |
| Visible range setup | `tv range --from <UNIX_SECONDS> --to <UNIX_SECONDS>` |
| Context reads | `tv state`, `tv quote`, `tv ohlcv --summary` |
| Screenshots | `tv screenshot --region full|chart --output <PATH>` |
| `replay_start`, `replay_step`, `replay_autoplay`, `replay_status`, `replay_trade`, `replay_stop` | Not implemented |

## Working Pattern Today

Use the CLI to position and document the chart. If replay mode is required, ask the user to operate replay manually and then use CLI reads/screenshots for observation.

Replay automation is migration backlog. The MCP server itself is not planned.
