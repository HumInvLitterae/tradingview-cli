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
| Strategy tester screenshot | `tv screenshot --region strategy --output <PATH>` |

## Working Pattern Today

Use the CLI for chart, market, and strategy context. If the strategy commands return empty metrics or an `error`, report that as an observation rather than filling the gap with guesses.

All three structured strategy commands return an additive `strategy_context`.
Confirm matching `selected_entity_id`, `selection_reason`, visibility, and
report availability before combining their evidence. A hidden strategy must be
made visible explicitly. Multiple equally plausible strategies return
`ambiguous` rather than selecting chart order; leave one intended
report-bearing strategy before retrying. `panel_status: "unknown"` means the
current Desktop build did not expose a deterministic panel-state signal and
does not by itself make structured data unavailable.

Strategy Tester panel screenshots are visual evidence only. Use `tv data strategy`, `tv data trades`, and `tv data equity` for structured fields when available. The MCP server itself is not planned.
