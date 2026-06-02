# Multi-Symbol Scan Workflow Reference

This reference keeps detailed source and command notes out of the core skill.

## Current Rust CLI Mapping

| Need | Rust CLI |
| --- | --- |
| Broad scanner discovery | `tv scanner scan` or `tv scanner hotlist` |
| Scanner field availability | `tv scanner metainfo --field <FIELD>` |
| Known-symbol comparison | `tv compare <SYMBOL>...` |
| Bounded known-symbol watch | `tv watch compare <SYMBOL>...` |
| Quote-only batch | `tv quotes <SYMBOL>...` |
| One-symbol packet | `tv snapshot <SYMBOL>` |
| Event-shaped earnings/dividends | `tv events <SYMBOL>` |
| Desktop-free historical bars | `tv bars <SYMBOL> --from ... --to ...` |
| Selected chart symbol | `tv symbol <SYMBOL>` |
| Selected chart timeframe | `tv timeframe <RESOLUTION>` |
| Selected-chart quote | `tv quote <SYMBOL> --source chart` |
| Selected-chart OHLCV | `tv ohlcv --summary` or `tv ohlcv --count <N>` |
| Visible study values | `tv values` |
| Drawing-derived data | `tv data lines`, `tv data labels`, `tv data tables`, `tv data boxes` |
| Selected-chart export | `tv export chart-bars --from ... --to ...` |
| Replay workflow evidence | `tv replay status`, `tv replay log --steps <N>` |
| Watchlist read | `tv watchlist get` |
| Watchlist add | `tv watchlist add-bulk <SYMBOL>... --allow-partial` |
| Screenshot | `tv screenshot --region full|chart --output <PATH>` |

## Scanner And Compare Notes

- `tv compare` returns ordered Desktop-free evidence with quote, info, and
  fundamentals sections. `summary.coverage_status` means evidence
  completeness only; use raw `items[]` for the actual comparison.
- For regular-session movement, prefer
  `items[].movement.regular_change_percent` as first-pass readback and confirm
  against raw quote fields when needed.
- `follow_up_hints[]` and `missing_evidence[]` route possible next evidence
  reads. They are not ranking or automatic execution. Stable follow-up kinds
  include `snapshot`, `chart_quote`, `observe_chart`, and `screenshot`.
- `tv watch compare` emits `watch_compare.v1` readiness / sample / heartbeat /
  summary events. It is bounded scanner-backed JSONL, not a daemon or selected
  chart feed.

## Bars And Events Notes

- `tv bars` is Desktop-free historical bars evidence. Date-range mode supports
  `5`, `15`, `30`, `60`, `1D`, `1W`, and `1M`; other intraday date-range
  timeframes remain guarded. Date-range `--count` defaults to 500 and may be
  raised to 5000 as a safety cap. Recent count mode remains capped at 500.
- For `tv bars`, read `requested_symbol`, `resolved_symbol`,
  `symbol_resolution`, `range_coverage_status`, `range_alignment`,
  `range_fetch_summary`, `source_availability`, and `data_quality` before
  summarizing coverage.
- `tv events` is `events.v1` from `scanner_fundamentals_rest`. It is a
  symbol-scoped earnings/dividend readback, not a full event calendar, ranking,
  or trading signal.

## Desktop Escalation Notes

- Use Desktop-backed selected-chart reads only for finalists. Run
  `tv readiness` first, and use `tv tab list` / `--target-id` when target
  selection is ambiguous.
- Do not build broad multi-symbol realtime loops on chart-source quote. Chart
  mutation is serial and can contend with the visible chart.
- `tv export chart-bars` is selected-chart export diagnostics with
  `export_chart_bars.v1`, chart context, range operation, returned bars range,
  and `selected_chart_range_match`. It is not a hidden fallback for `tv bars`.
- `tv replay log` records bounded Replay step evidence. Replay state changes
  belong to the selected Desktop chart and are not historical bars input.

## Remaining Gaps

`batch_run`, arbitrary historical indicator-series computation, stable
chart-backed compare, stable Replay export, and a complete event calendar are
not implemented. The MCP server itself is not planned.
