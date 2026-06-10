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
| `replay_start`, `replay_step`, `replay_autoplay`, `replay_status`, `replay_trade`, `replay_stop` | `tv replay start`, `tv replay step`, `tv replay autoplay`, `tv replay status`, `tv replay trade`, `tv replay stop` |
| Bounded step log with bars context | `tv replay log --steps <N> --attach-ohlcv-summary [--ohlcv-count <N>]` |

## Working Pattern Today

Use the CLI to position and document the chart, then use replay commands when the user wants CLI-controlled practice. Check `tv replay status` before and after mutation. If replay trade state is opened during practice, close it with `tv replay trade close` when cleanup is desired, then stop replay with `tv replay stop`.

Replay automation is now available through the Rust CLI. The MCP server itself is not planned.
