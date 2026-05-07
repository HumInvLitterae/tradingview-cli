# Observation Workflows

This guide shows how to choose `tv` commands when an agent or human needs to
observe TradingView state before deciding what to do next. It complements the
source taxonomy in `docs/command-source-taxonomy.md`; it does not define new
commands.

## Desktop-Free Screening

Use Desktop-free reads when the task is broad discovery, quote comparison,
symbol metadata, fundamentals, or scanner-style filtering. These commands do
not require TradingView Desktop or CDP:

```bash
tv snapshot NASDAQ:AAPL
tv compare NASDAQ:AAPL NYSE:IONQ
tv quotes AAPL MSFT NYSE:IONQ
tv scanner scan --type stock --columns name,close,volume --limit 10
tv fundamentals NYSE:IONQ --group earnings
tv scanner metainfo --market america --field close
```

Use `tv snapshot <SYMBOL>` for a first-pass packet on one symbol. It combines
scanner quote, symbol info, and scanner-backed fundamentals without connecting
to TradingView Desktop. Use `tv compare <SYMBOL>...` when the task is a
Desktop-free comparison across several known symbols. Use the lower-level
commands when you need just one section, ordered quotes only, or a scanner row
set.

Treat scanner-backed price reads as screening evidence, not as a realtime
entitlement guarantee. Preserve `source_category`, `requires_desktop`,
`non_mutating`, and freshness fields such as `time`, `update_mode`, or
`delay_seconds` when they are present.

## Desktop-Backed Chart Observation

Use Desktop-backed reads when the selected TradingView Desktop chart or visible
chart feed is the source of truth. If `tv snapshot` gives enough static
symbol context, do not start chart observation just to re-read quote, info, or
fundamentals. When chart state over time matters, start with readiness:

```bash
tv readiness
```

If the readiness payload is clear and you need a short observation window, use:

```bash
tv observe chart --duration-ms 10000 --heartbeat-ms 2000
```

`tv observe chart` emits newline-delimited JSON. The first event is readiness;
later events are selected-chart bar samples or heartbeats. Use this when the
workflow needs readiness plus last-bar observation in one bounded command.

Use lower-level stream commands only when you already know which chart sample
type you need:

```bash
tv stream quote --duration-ms 10000 --heartbeat-ms 2000
tv stream bars --max-events 5
```

Do not add manual sleeps or double-call loops around chart-source quote reads.
The CLI performs its own readiness checks and returns structured errors when
chart data is not ready.

Avoid building multi-symbol realtime loops on
`tv quote <SYMBOL> --source chart`. Chart-source quote is a serial,
correctness-first read for the selected TradingView chart feed, and it may
switch and restore the visible chart. For broad comparison, use Desktop-free
reads such as `tv compare`, `tv quotes`, `scanner scan`, or `snapshot`; move
to chart-source quote only for a finalist where the selected chart feed itself
matters.

## Visual Evidence Recovery

Structured fields should come first. Use screenshots only when readiness,
state, OHLCV, stream, or observe output does not explain the visible chart or
Screener state:

```bash
tv screenshot --region chart --output target/tv-chart.png
tv screenshot --region full --output target/tv-full.png
```

Screenshots are Desktop-backed visual evidence. They do not mutate
TradingView state, but they write a local file, so screenshot payloads report
`writes_file: true`.

## Experimental Historical Bars

`tv bars` is a lab-gated Desktop-free historical bars prototype:

```bash
TV_EXPERIMENTAL_BARS=1 tv bars NASDAQ:AAPL --timeframe 1D --count 5
```

It uses an undocumented TradingView WebSocket path and remains experimental.
Read `experimental`, `source`, and `data_quality` before using the result, and
do not treat it as a stable replacement for chart-backed `tv ohlcv`.

## Fundamentals And Event-Like Fields

Use `tv fundamentals` for scanner-backed fundamentals and event-like fields:

```bash
tv fundamentals NYSE:IONQ --group earnings
tv fundamentals AAPL --group dividends
```

The earnings and dividend groups are scanner field bundles, not a complete
TradingView event calendar or news feed. The groups include scanner-confirmed
earnings date/publication fields and dividend yield/date/amount/frequency
fields. Treat `field_values` as the source of truth and avoid inferring
timezone, before/after-market meaning, publication-code meaning, or investment
significance unless another source supplies that interpretation.

## Deferred Surfaces

The following are not normal observation workflow steps today:

- standalone `tv events`;
- stable browserless historical bars;
- browserless streaming;
- binary split such as separate Desktop-free and Desktop-backed executables;
- MCP server, daemon, dashboard, or trading-bot behavior;
- Computer Use-specific workflow skills.
