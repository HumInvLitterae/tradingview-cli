# Multi-Symbol Scan Workflow Reference

## Original MCP Intent

The original multi-symbol-scan skill scanned several symbols with a shared timeframe and criteria. It used batch execution, chart symbol changes, OHLCV reads, indicator data, screenshots, and optional watchlist updates.

## Current Rust CLI Mapping

| Original MCP capability | Rust CLI status |
| --- | --- |
| `chart_set_symbol` | `tv symbol <SYMBOL>` |
| `chart_set_timeframe` | `tv timeframe <RESOLUTION>` |
| `quote_get` | `tv quote` |
| `data_get_ohlcv` | `tv ohlcv --summary` or `tv ohlcv --count <N>` |
| `data_get_study_values` | `tv values` for visible studies |
| `data_get_indicator` | `tv data indicator <ENTITY_ID>` |
| Pine lines / labels / tables / boxes | `tv data lines`, `tv data labels`, `tv data tables`, `tv data boxes` |
| `watchlist_get` | `tv watchlist get` |
| `capture_screenshot` | `tv screenshot --region full|chart --output <PATH>` |

## Not Yet Available

`batch_run`, arbitrary historical indicator series, and `watchlist_add` are not implemented in the Rust CLI. Run scans serially and avoid promising watchlist mutation.

The MCP server itself is not planned.
