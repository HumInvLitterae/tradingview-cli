# Chart Analysis Workflow Reference

This reference holds details that are useful for chart work but should not be
loaded every time the skill triggers.

## Current Rust CLI Mapping

| Need | Rust CLI |
| --- | --- |
| Read selected chart state | `tv state` |
| Set selected chart symbol | `tv symbol <SYMBOL>` |
| Set selected chart timeframe | `tv timeframe <RESOLUTION>` |
| Read or set chart type | `tv type [CHART_TYPE]` |
| Scroll to a date | `tv scroll <DATE>` |
| Read visible range | `tv range` |
| Set visible range | `tv range --from <UNIX_SECONDS> --to <UNIX_SECONDS>` |
| Desktop-free quote | `tv quote <SYMBOL>` |
| Selected-chart quote | `tv quote <SYMBOL> --source chart` |
| Explicit quote-data readback | `tv quote <SYMBOL> --source quote-data` |
| Symbol metadata | `tv info <SYMBOL>` |
| Current-chart metadata | `tv info` |
| Selected-chart bars | `tv ohlcv --summary` or `tv ohlcv --count <N>` |
| Desktop-free historical bars | `tv bars <SYMBOL> --from ... --to ...` |
| Selected-chart export | `tv export chart-bars --from ... --to ...` |
| Study values | `tv values` |
| Pine drawing data | `tv data lines`, `tv data labels`, `tv data tables`, `tv data boxes` |
| Strategy metrics, trades, equity | `tv data strategy`, `tv data trades --max <N>`, `tv data equity` |
| Indicator lifecycle | `tv indicator add/remove/toggle/set` |
| Drawing lifecycle | `tv draw shape/list/get/remove/clear` |
| Replay controls | `tv replay status/start/step/autoplay/trade/stop` |
| Replay step log | `tv replay log --steps <N>` |
| Screenshot | `tv screenshot --region full|chart|strategy --output <PATH> [--wait-for-render]` |

For bounded range changes, report `history_paging.coverage_status`,
`stop_reason`, and `request_count`, then report
`viewport_application.status`, `matching_bar_count`, and `applied_range`.
Do not infer that the viewport moved from endpoint coverage alone.

`--wait-for-render` is an opt-in bounded readiness check for screenshots taken
after selected-chart state changes. Success includes
`screenshot_render_wait.v1`; timeout captures nothing and leaves the requested
file untouched. Use `--wait-timeout-ms <500..30000>` only with the wait flag.

## Source Notes

- `tv snapshot <SYMBOL>` is a good one-symbol Desktop-free context read before
  chart mutation. Its follow-up hints are advisory; they are not ranking or
  automatic follow-up execution.
- Scanner-backed quote fields can include `time`, `update_mode`,
  `delay_seconds`, and extended-hours fields when TradingView returns them.
  They are not a realtime entitlement guarantee.
- `tv bars` is the reproducible historical bars entry point. Date-range mode
  supports `5`, `15`, `30`, `60`, `1D`, `1W`, and `1M`; other intraday
  date-range timeframes remain guarded. Date-range `--count` defaults to 500
  and may be raised to 5000 as a safety cap. Recent count mode remains capped
  at 500. Use `range_coverage_status`, `range_alignment`, and
  `range_fetch_summary` for coverage and truncation diagnostics.
- `tv ohlcv` reads the selected chart. `chart_context`,
  `returned_bars_range`, and `selected_chart_range_match` are diagnostics;
  they do not prove a reproducible historical export by themselves.
- `tv export chart-bars` is an explicit Desktop-backed selected-chart export
  workflow. Keep it separate from `tv bars`.
- Replay commands depend on and mutate selected-chart Replay state, except
  status reads. `tv replay log` records bounded Replay step evidence; it is not
  historical bars input.
- Strategy data reads share `strategy_context`. Hidden, unready, missing, and
  ambiguous states are diagnostics and never trigger automatic panel opening
  or visibility changes.
- `tv quote <SYMBOL> --source quote-data` is explicit Desktop-backed
  quote-data readback such as `qsd.rtc`. If unavailable, report
  `source_availability`; do not treat unavailability as proof that a symbol has
  no price.
- `tv diagnose quote-data <SYMBOL>` can inspect sanitized target state,
  quote-data availability, public-safe WebSocket/qsd counters, and scanner
  freshness reference without blending sources.

## Remaining Gaps

Arbitrary historical indicator-series computation is not implemented in the
Rust CLI. For a small finalist set that needs selected-chart evidence, use
`tv chart compare <SYMBOL>...`; for broad first-pass comparison, prefer
Desktop-free `tv compare` or `tv watch compare`. Prefer implemented high-level
commands before generic UI automation.

The MCP server itself is not planned.
