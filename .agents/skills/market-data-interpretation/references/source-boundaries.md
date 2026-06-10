# Market Data Source Boundaries

Use this reference when a market-data task needs source details beyond the
short workflow in `SKILL.md`.

## Desktop-Free Reads

Desktop-free reads do not require TradingView Desktop or CDP. They should
report `source_category: "desktop_free_read"`, `requires_desktop: false`, and
`non_mutating: true` when the payload supports those fields.

Common Desktop-free reads:

- `tv quote <SYMBOL>` and `tv quotes <SYMBOL>...`
- `tv snapshot <SYMBOL>`
- `tv compare <SYMBOL>...`
- `tv watch compare <SYMBOL>...`
- `tv fundamentals <SYMBOL>`
- `tv events <SYMBOL>`
- `tv scanner scan` and `tv scanner metainfo`
- `tv bars <SYMBOL>`

Scanner REST price reads are useful for screening, but they are not realtime
entitlement guarantees. If a scanner quote payload includes `time`,
`update_mode`, or `delay_seconds`, report those fields when freshness matters.

## `tv bars`

`tv bars` is a bounded historical bars read using the TradingView bars
WebSocket path. It is Desktop-free and separate from selected-chart `tv ohlcv`.

Report `contract_version: "bars.v1"`, `source: "tradingview_bars_ws"`,
`requested_symbol`, `resolved_symbol`, `symbol_resolution`, `summary`, `range`,
`requested_range`, `returned_range`, `range_coverage_status`,
`range_alignment`, `range_fetch_summary`, `source_availability`,
`data_quality`, and warnings when present.

Bare symbols such as `AAPL` may resolve through Desktop-free symbol search. If
the exchange matters, retry with `EXCHANGE:SYMBOL`.

Use `--from YYYY-MM-DD --to YYYY-MM-DD` with `--timeframe 5`, `15`, `30`,
`60`, `1D`, `1W`, or `1M` for reproducible date-range samples. In date-range
mode, `--count` is a returned-bar safety cap that defaults to 500 and can be
raised to 5000. Recent count mode remains capped at 500. `--to` is an
inclusive calendar date.

`range_coverage_status` is the primary date-range coverage readback.
`range_alignment` explains timestamp anchoring and
`timestamp_within_requested_range` filtering. `range_fetch_summary` explains
fetch windows, observed / filtered / returned counts, and truncation reason.

## `tv events`

`tv events <SYMBOL>` is a scanner fundamentals read shaped as `events.v1`. It
reports scanner-backed earnings and dividend fields as event entries.
`tv events compare <SYMBOL>...` uses the same source for several symbols and
returns `events_compare.v1` with ordered item status and summary counts.

Use these commands when an event-shaped payload is easier than raw
`field_values`. They are not a full event calendar, chart read, ranking,
recommendation, or trading judgment. Do not infer timezone,
before/after-market, confirmed/estimated status, or publication meaning unless
TradingView explicitly returns that value.

When event fields are missing or null, describe them as unavailable or unknown,
not as proof that no event exists.

## Snapshot And Compare

`tv snapshot <SYMBOL>` is a one-symbol Desktop-free evidence packet. It groups
scanner-backed quote, symbol info, and fundamentals sections.

`tv compare <SYMBOL>...` is an ordered multi-symbol Desktop-free evidence
packet. It preserves input order and returns per-symbol quote, info, and
fundamentals sections.

`tv chart compare <SYMBOL>...` is different: it is Desktop-backed selected
chart evidence for a small finalist set. It returns `chart_compare.v1`, may
temporarily switch the selected chart, and reports item status plus restore
readback. Do not treat it as scanner-backed compare, ranking, or automatic
source mixing.

For both commands, read `summary`, section errors, `missing_evidence[]`, and
`follow_up_hints[]`. Follow-up hints are advisory surfaces only; they do not
execute commands automatically and are not rankings or recommendations.

For compare movement, use `items[].movement.regular_change_percent` as the
stable first-pass readback and confirm against raw quote section data when
needed. Do not infer absolute regular change from last / close when
`regular_change_abs` is null.

## JSONL Workflows

`tv watch compare <SYMBOL>...` is Desktop-free scanner-backed quote polling for
a bounded window. Read `watch_compare.v1` readiness, sample, heartbeat, and
summary events. It is not a daemon, selected-chart feed, ranking, or trading
recommendation.

`tv observe chart` is Desktop-backed selected-chart observation. It emits
readiness first, then sample and heartbeat events, then summary for bounded
normal exits.

`tv stream ...` is lower-level Desktop-backed current-chart streaming. A
heartbeat means the stream is alive; it is not a market update.

## Desktop-Backed Reads

Desktop-backed reads depend on the selected TradingView Desktop chart. Run
`tv readiness` before escalating to visual or chart-specific reads. If multiple
targets are open, run `tv tab list` and pass the desired `target_cli_args`.

Use selected-chart reads such as `tv quote --source chart`, current-chart
`tv quote`, `tv ohlcv`, `tv state`, screenshots, and selected-chart export
only when the selected chart itself is the source being studied.

`tv export chart-bars --from <UNIX_SECONDS> --to <UNIX_SECONDS>` is explicit
Desktop-backed export evidence. It is not a fallback for `tv bars`.

Replay commands are Desktop-backed selected-chart operations. `tv replay log
--steps <N>` is bounded workflow evidence, not historical bars input.

## Quote-Data And Extended Hours

Scanner-backed quotes may include `extended_hours.premarket` and
`extended_hours.postmarket`. Missing extended-hours fields can mean the
session is inactive or TradingView did not return the value.

`tv quote <SYMBOL> --source quote-data` reads explicit Desktop-backed
quote-data such as `qsd.rtc` when available. Keep it separate from scanner
extended-hours fields and chart main-series quote. If quote-data is
unavailable, use `source_availability.unavailable_reason` as source
diagnostics only.

`tv diagnose quote-data <SYMBOL>` is troubleshooting metadata, not a blended
quote.
