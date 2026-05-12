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

## Which Read To Use

| Need | Prefer | Use when |
| --- | --- | --- |
| Several symbols, quote fields only | `tv quotes <SYMBOL>...` | You need ordered scanner-backed quote rows and do not need info or fundamentals sections. Inspect `time`, `update_mode`, and `delay_seconds` when freshness matters. |
| Several known symbols, first-pass evidence | `tv compare <SYMBOL>...` | You need quote, info, and default fundamentals side by side. Read `summary` for scanability and `items[]` for evidence. |
| One symbol, Desktop-free detail | `tv snapshot <SYMBOL>` | You need quote, info, and fundamentals for one symbol before chart follow-up. |
| Selected chart over a short window | `tv observe chart --duration-ms ...` | You need readiness plus selected-chart last-bar samples and heartbeats. |
| Finalist chart-feed quote | `tv quote <SYMBOL> --source chart` | The selected TradingView Desktop chart feed for one symbol is the source that matters. |
| Visible-state evidence gap | `tv screenshot --region chart|full --output <PATH>` | Structured reads do not explain the visible chart or Screener state. |

Use `tv snapshot <SYMBOL>` for a first-pass packet on one symbol. It combines
scanner quote, symbol info, and scanner-backed fundamentals without connecting
to TradingView Desktop. Use `tv compare <SYMBOL>...` when the task is a
Desktop-free comparison across several known symbols. Use the lower-level
commands when you need just one section, ordered quotes only, or a scanner row
set.

`tv snapshot` also includes compare-style contract readback for a single
symbol. Use `contract_version`, `summary.coverage_status`,
`summary.field_coverage`, `missing_evidence[]`, and `follow_up_hints[]` to
understand whether quote, info, or fundamentals evidence is complete and which
follow-up surface is available. Treat those fields as coverage and routing
metadata; the raw `sections.quote`, `sections.info`, and
`sections.fundamentals` remain the evidence. Snapshot metadata does not rank,
score, recommend, call chart-source quote, start observation, or capture a
screenshot.

`tv compare` includes a top-level `summary` for scanability. Use it to read
resolution counts, section success counts, missing counts, and requested to
resolved symbol mappings. Treat `summary` as readback only; inspect raw
`items[]` before drawing substantive conclusions or choosing follow-up
actions. Compare payloads also include `contract_version`,
`requested_index`, per-item `follow_up_hints`, and `summary.field_coverage`
as downstream readback helpers. `summary.coverage_status` is a compact
evidence-coverage readback: `complete` means every requested item has quote,
info, and fundamentals sections with no missing fields; `partial` means some
evidence exists but section errors or missing fields remain; `blocked` means
the structured compare payload has no usable per-item evidence.
`items[].missing_evidence[]` names the section with missing evidence, known
missing fields, the missing reason, and a stable follow-up kind such as
`snapshot` or `chart_quote`. These fields make ordering, schema guards,
follow-up surfaces, and evidence gaps easier to consume, but they do not rank
symbols or replace raw evidence.

For regular-session movement evidence, use
`items[].movement.regular_change_percent` as the stable compare-level readback.
It is derived from the scanner quote section's
`items[].sections.quote.data.change`, which remains the raw source evidence.
`movement.regular_change_abs` is `null` until the scanner quote source exposes
or this project defines a normalized absolute regular-change field. Do not
derive ranking, scoring, or trade action from `movement`; it is only a
machine-readable evidence path for downstream tools.

## Follow-up Vocabulary

`compare` and `snapshot` use the same stable follow-up vocabulary. These
values describe evidence surfaces an agent may choose next; they are not
recommendations and they are not executed automatically.

| Kind | Meaning | Desktop |
| --- | --- | --- |
| `snapshot` | One-symbol Desktop-free detail or retry surface for quote, info, and fundamentals sections. | No |
| `chart_quote` | Selected-chart single-symbol chart-feed quote follow-up. This is not scanner-style premarket or postmarket evidence. | Yes |
| `observe_chart` | Selected-chart time-window observation with readiness, samples, and heartbeats. | Yes |
| `screenshot` | Visual evidence when structured reads do not explain the visible state. | Yes |

Use these same meanings in `compare.items[].follow_up_hints[]`,
`compare.items[].missing_evidence[].suggested_follow_up`,
`snapshot.follow_up_hints[]`, and
`snapshot.missing_evidence[].suggested_follow_up`. The canonical chart-feed
quote kind is `chart_quote`; do not introduce or infer a `quote_chart` alias.

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

Do not use chart-source quote as premarket or postmarket evidence.
Chart-source quote reports `session_boundary` to make this explicit: the price
comes from the selected chart main-series last bar, the price session is
`unknown`, and scanner-style extended-hours fields are not included or
guaranteed. If a workflow needs `extended_hours.premarket` or
`extended_hours.postmarket`, use scanner-backed `tv quote`, `tv quotes`,
`tv snapshot`, or `tv compare` and preserve that Desktop-free source boundary.

Desktop page quote-session probes can expose `premarket_*`, `postmarket_*`,
and `market-status` field names, but they are not yet a stable public evidence
surface. During regular session, those fields may not mean the same thing as
scanner-backed extended-hours values. Treat them as opt-in live evidence for
source research until postmarket and premarket behavior is confirmed.
Postmarket probing has shown `market-status.phase=post-market` can appear, but
the selected quote-session pre/post close fields matched each other and
remained tied to quote-session streaming values in the public-safe summary, so
this is still not scanner-style `extended_hours` evidence.

The Desktop right-side symbol detail panel can also show a visible
after-market price that differs from scanner REST, chart main-series quote, and
the current quote-session selected fields. That value is useful for source
discovery, but it is not yet part of a stable public `tv quote` payload. When
the visible panel matters, use the dedicated opt-in smoke or screenshot-backed
inspection rather than assuming `quote --source chart` contains the same
after-hours value. The current postmarket source discovery narrowed the RKLB
visible value to the right-side detail widget's status/price nodes, with React
metadata present on the matched node; treat that as a visible UI source until a
separate contract exposes it. A bounded CDP Network/WebSocket smoke observed
symbol-related WebSocket traffic while that visible value was present, but did
not find the visible after-hours price token in captured communication
candidates. A later scoped in-page widget inspection found the right-panel
detail widget React chain and regular quote-like props, including current
session and regular last-price fields, but did not expose the visible
after-hours price token in compact prop/state hits. A later bounded WebSocket
correlation smoke sampled visible after-market prices during the same capture
window and found exact numeric matches in received WebSocket frame summaries,
supporting a push/WebSocket-backed source hypothesis without making it a stable
payload source yet. A follow-up HAR/live pass made `qsd.rtc` the strongest
current field-level candidate for that visible after-market value, while
`lp`/`regular_close` remain regular close-like readbacks. Use
`tv quote <SYMBOL> --source quote-data` when that explicit Desktop-backed
quote-data readback is needed. It is not an implicit extension of
`tv quote --source chart` and does not merge scanner REST `extended_hours`.
If no matching `qsd.rtc` frame arrives during the bounded wait, treat the
structured unavailable result as source availability rather than as a reason
to guess a price. Quote-data success payloads and unavailable details expose
`contract_version: "quote_data.v1"` and `source_availability` so agents can
distinguish an available source readback from a bounded-wait source
unavailable result without adding automatic fallback or source mixing.
`source_availability.unavailable_reason` is a source diagnostic such as
`no_websocket_events`, `no_qsd_messages`, `no_matching_symbol`, or `no_rtc`.
Use it to decide whether to retry quote-data, verify the Desktop streaming
symbol, or use scanner REST if delayed data is acceptable. Do not treat it as
price absence or a trading signal. Success payloads include
`quote_data.session_readback` for normalized spellings of TradingView-provided
session fields without inferring a session that TradingView did not report.
During regular session, this can happen because the current quote-data
contract waits for matching non-null `qsd.rtc`, while TradingView may expose
regular quote-like fields such as `lp` or `regular_close` instead. Treat that
as a source and field-semantics question. Use scanner freshness metadata or
chart main-series quote for regular-session price unless a later quote-data
contract adds explicit regular readback.
If an agent needs to explain why quote-data is unavailable, use
`tv diagnose quote-data <SYMBOL>`. The diagnostic reports sanitized Desktop
target state, quote-data availability, public-safe WebSocket/qsd counters, and
a separate scanner freshness reference in one packet. It is troubleshooting
metadata, not a blended price read, and it does not switch symbols or add
quote-data to `--source auto`.

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
