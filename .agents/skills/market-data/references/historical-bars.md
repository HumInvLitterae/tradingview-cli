# Historical bars and completeness

`tv bars` reads Desktop-free `tradingview_bars_ws` data with `bars.v1`. Use it
for historical input rather than moving the Desktop viewport or using Replay.
Bare symbols resolve through Desktop-free search; inspect `requested_symbol`,
`resolved_symbol`, and `symbol_resolution`. Specify `EXCHANGE:SYMBOL` before
fetching when the exchange must be fixed.

## Select the request

| Need | Request |
| --- | --- |
| Recent bars | `tv bars <SYMBOL> --timeframe <TF> --count <N>`; at most 500 bars |
| An explicit date interval | Add `--from YYYY-MM-DD --to YYYY-MM-DD`; supported timeframes: `1`/`1m`, `5`, `15`, `30`, `60`, `1D`, `1W`, `1M` |
| Up to 5,000 returned bars in that interval | Add `--count 5000`; the date-range default is 500 |
| A larger historical corpus | Split into explicit non-overlapping calendar windows; merge downstream by period-start timestamp |

`--to` is an inclusive calendar date; timestamp filtering uses a half-open
interval ending at the following UTC day. Other intraday timeframes remain
guarded in date-range mode. The count is a returned-bar safety cap, not a promise
that many bars exist and not the number of transport windows.

Example for a one-minute interval:

```bash
tv bars EXCHANGE:SYMBOL --timeframe 1 \
  --from YYYY-MM-DD --to YYYY-MM-DD --count 5000
```

## Read coverage before raw bars

Use `summary` and `range` for a compact view; retain `requested_range`,
`returned_range`, and any observed range when explaining mismatches.

- `range_coverage_status` and `range_fetch_summary.range_truncated` determine
  date-range completeness together. `data_quality.partial_result` alone may
  only mean fewer bars arrived than the requested count.
- `range_alignment` describes period-start anchoring and timestamp filtering.
- `range_fetch_summary` describes fetch windows, observed/filtered/returned
  counts, count caps, and truncation reason.
- `source_availability`, `wait_summary`, warnings, and errors explain missing
  evidence. Do not synthesize bars or infer an exchange calendar to fill gaps.

## Failure handling

Retain the JSON error and existing coverage/availability details.
`source_failure_stage` locates the boundary: `symbol_search` is bare-symbol
resolution; `session_setup` is common bootstrap; `series_setup` is request
setup; `response_wait`, `protocol`, `pagination`, and `source_result` identify
later stages. `heartbeat_send` and `pagination` are send boundaries with unknown
remote receipt. A stage alone does not authorize retry, a longer timeout, or a
fallback to another provider, chart, or Replay. Report the failed attempt and
choose any further action from the user's approved scope.

## Official MCP

`tv mcp bars` is separate from the existing WebSocket `tv bars`. Check help for
binary support. It requires explicit OAuth login and supports only recent-count
1D/1W/1M reads. Read `mcp_bars.v1` provider/client observations: count satisfaction
is not calendar completeness; null volume and unknown delay, adjustment, session
and finality must remain unknown. Symbol/interval echoes are not independent
listing proof. Never substitute it automatically for date-range requests or
fall back to the old source on failure. `tv mcp status` is local-only; login and
OS consent require explaining the user's action before opening a dialog.
