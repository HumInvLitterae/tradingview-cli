---
name: market-data-interpretation
description: Interpret TradingView market data returned by the Rust `tv` CLI. Use when explaining quote, batch quote, scanner, chart quote, OHLCV, extended-hours, freshness, missing-value, or source differences.
---

# Market Data Interpretation

Use this skill when a task depends on understanding market data returned by the
Rust `tv` CLI rather than merely running the command.

Use `docs/observation-workflows.md` when you need the current practical order
for Desktop-free screening, Desktop-backed chart observation, screenshots,
experimental bars, and fundamentals/event-like reads.

## Source First

Always name the data source before interpreting values:

- Desktop-free reads: `tv snapshot <SYMBOL>`, `tv compare <SYMBOL>...`,
  `tv quote <SYMBOL>`, `tv quotes <SYMBOL>...`,
  `tv fundamentals <SYMBOL>`, `tv scanner scan`, and `tv scanner metainfo`.
  Stable Desktop-free payloads should report
  `source_category: "desktop_free_read"`, `requires_desktop: false`, and
  `non_mutating: true`.
- Desktop-backed reads: `tv quote <SYMBOL> --source chart`, current-chart
  `tv quote`, `tv ohlcv`, screenshots, and current visible values.
- Hybrid reads: `tv quote <SYMBOL> --source auto`, which is chart-first with
  scanner fallback only before chart mutation.
- `TV_EXPERIMENTAL_BARS=1 tv bars <EXCHANGE:SYMBOL>`: experimental
  Desktop-free WebSocket bars.
- `tv observe chart`: Desktop-backed JSONL workflow that emits readiness first,
  then selected-chart last-bar samples and heartbeats.
- `tv stream ...`: Desktop-backed current-chart JSONL observation, not
  browserless WebSocket streaming. Use it when you need a specific stream
  sample type rather than the readiness-plus-chart workflow.

Do not blend scanner REST, chart feed, and visible chart observations as if
they were the same source.

## Read Selection

| Need | Prefer |
| --- | --- |
| Several symbols, quote fields only | `tv quotes <SYMBOL>...` |
| Several known symbols with quote, info, and fundamentals | `tv compare <SYMBOL>...` |
| One symbol with Desktop-free detail | `tv snapshot <SYMBOL>` |
| Selected chart over a short window | `tv observe chart --duration-ms ...` |
| One finalist's selected-chart quote | `tv quote <SYMBOL> --source chart` |
| Visual evidence after structured reads | `tv screenshot --region chart|full --output <PATH>` |

When `compare` or `snapshot` returns follow-up metadata, use the stable kind
values literally: `snapshot`, `chart_quote`, `observe_chart`, and
`screenshot`. `chart_quote` means selected-chart single-symbol chart-feed
quote; it is not scanner-style premarket or postmarket evidence. Do not rename
it to `quote_chart`. These hints are available evidence surfaces, not
recommendations, rankings, or automatic reads.

## Freshness And Session Boundaries

Scanner REST price reads are useful for screening, but they are not realtime
entitlement guarantees. When scanner quote payloads include `time`,
`update_mode`, or `delay_seconds`, report those fields when freshness matters.
If `update_mode` shows a delayed feed, say so plainly.

Chart-sourced reads can be closer to the user's selected TradingView Desktop
feed, but they depend on the active chart target, chart readiness, symbol
switching, and post-checks. `tv quote <SYMBOL> --source chart` performs its own
bounded readiness wait, requires consecutive stable requested-symbol samples,
and retries once; do not add manual sleep or double-call workarounds
downstream. If it fails, report the structured freshness details or use scanner
source explicitly when scanner freshness is acceptable. Chart-source quote
reports `session_boundary`; treat `price_session: "unknown"` and
`extended_hours_status: "not_provided"` as meaning the chart source does not
guarantee scanner-style premarket or postmarket evidence. Use scanner-backed
`tv quote`, `tv quotes`, `tv snapshot`, or `tv compare` when premarket or
postmarket fields matter. `tv ohlcv` is
chart-dependent; do not describe it as Desktop-free historical bars.
TradingView Desktop's page quote session can expose `premarket_*`,
`postmarket_*`, and `market-status` field names during opt-in probes, but this
is a separate source candidate from chart main-series quote. During regular
session, those fields may track current streaming values rather than
scanner-backed extended-hours values, so do not treat them as stable
premarket/postmarket evidence until phase-specific smoke evidence confirms the
semantics. Postmarket probing can show `market-status.phase=post-market`, but
that alone is not enough to treat `premarket_close` or `postmarket_close` as
scanner-style `extended_hours`; in observed postmarket output, those selected
fields matched each other and remained tied to quote-session streaming values
rather than forming a scanner-style `extended_hours` object.
The visible TradingView Desktop right-side symbol detail panel can show a
separate after-hours price that differs from scanner REST, chart main-series
quote, and the current quote-session selected field set. Treat that as
Desktop-backed visible UI evidence until a stable CLI payload explicitly
exposes it; do not infer it from `quote --source chart`. Current source
discovery has narrowed this to the right-side detail widget status/price node
for the observed RKLB postmarket case, but that is still visible UI evidence,
not a stable data API.
Do not use chart-source quote loops as a multi-symbol realtime batch source.
They may contend with visible chart mutations, so prefer `tv quotes`, scanner
reads, `tv compare`, or `tv snapshot` for broad symbol lists unless the
selected chart feed for one symbol is the point of the task.

`tv bars` is different from both scanner REST and `tv ohlcv`. It is a
lab-gated browserless historical bars prototype using an undocumented
TradingView WebSocket path. Use it only when the user or workflow explicitly
accepts experimental data; report `source`, `experimental`, `data_quality`,
and warnings. Do not treat it as a stable replacement for chart-sourced OHLCV.

`tv snapshot <SYMBOL>` is the first-pass Desktop-free evidence packet for one
symbol. It groups scanner-backed quote, symbol info, and fundamentals sections.
Read each section's `ok`, `data`, or `error` independently, and preserve the
top-level source metadata, section-level errors, and `next_action_hints`.
Treat `contract_version`, `summary.coverage_status`,
`summary.field_coverage`, `missing_evidence[]`, and `follow_up_hints[]` as
one-symbol readback helpers for coverage and follow-up routing. They do not
replace raw `sections`, do not call the follow-up commands, and are not
ranking, scoring, or recommendations. Use lower-level `tv quote`, `tv info`,
or `tv fundamentals` when the task needs only one section, and use
`tv compare` when the task needs a structured multi-symbol evidence packet.
Do not treat snapshot as batch, JSONL, chart-backed, screenshot, or
experimental bars evidence.

`tv compare <SYMBOL>...` is the Desktop-free comparison packet for several
known symbols. It preserves input order and returns per-symbol quote, info, and
fundamentals sections with section errors and missing summaries. Treat it as
evidence comparison, not scoring or recommendation. Read `summary` first for
resolution counts, section success counts, missing counts, and requested to
resolved symbol mappings, then inspect raw `items[]` before drawing
substantive conclusions. Treat `contract_version`, `requested_index`,
per-item `follow_up_hints`, `summary.field_coverage`, and
`summary.coverage_status` as downstream readback helpers for schema guards,
input-order joins, follow-up surfaces, and evidence gaps. `coverage_status`
describes evidence completeness only; it is not a ranking or recommendation.
Use `items[].missing_evidence[]` to see which section is missing evidence and
whether a stable follow-up such as `snapshot` or `chart_quote` is the relevant
next readback surface; confirm conclusions against raw `items[]`. Use
`tv quotes` when only ordered quote fields are needed, and use
`tv snapshot <SYMBOL>` for a single symbol that needs more detail after
comparison.

`tv fundamentals` is a Desktop-free scanner read, not a chart read. Use it for
raw fields such as market cap, P/E, EPS, dividend yield, dividend dates/amounts,
and earnings date/time. Prefer `--group earnings`, `--group valuation`,
`--group dividends`, or `--group financials` when the task needs a coherent
bundle; use `--field` for exact scanner fields. Treat `field_values` as the
source of truth. Do not infer timezone, before/after-market meaning,
publication-code meaning, financial analysis, or investment recommendations
from these fields unless another source explicitly supplies that
interpretation. The current event-like evidence is still scanner field
metadata, not a complete event calendar.

For Desktop-backed reads, inspect `tv readiness` before escalating to visual
checks. It summarizes endpoint, target selection, chart readiness, bars
readiness, and next-action hints. Use `tv tab list`, `tv state`, or
`tv ohlcv --count 1` as follow-up reads when the readiness payload points
there. Chart-source quote exposes `freshness_check`, and OHLCV failures expose
chart-bars details. Core Desktop-backed read payloads should expose
`source_category`, `requires_desktop`, and `non_mutating`; keep those fields
when explaining provenance. The portable visual fallback is
`tv screenshot --region chart|full --output <PATH>` plus user/manual
inspection when needed. Screenshot payloads are visual evidence, not market
data; preserve `source`, `source_category`, `writes_file`, and
`visual_evidence` when citing them.

For `tv observe chart`, interpret the first JSONL line as readiness
(`data._event: "readiness"`), then read later `sample` and `heartbeat` events
as selected-chart bar observations. Use bounded windows such as
`--duration-ms`, `--max-events`, and optional `--heartbeat-ms` for agent
workflows.

For `tv stream ...`, interpret each JSONL line by `data._event`. A `sample`
event means the chart/page sample changed after metadata-insensitive dedupe. A
`heartbeat` event means the stream is still alive but no changed sample was
emitted in that heartbeat window. Do not count heartbeat events as market
updates. Stream and observe sample events should identify
`source: "desktop_chart_stream"`,
`source_category: "desktop_backed_read"`, `requires_desktop: true`, and
`non_mutating: true`; treat them as current Desktop chart observations, not
Desktop-free scanner reads.

If the current environment is the Codex app and Computer Use is available, it
can help inspect or recover visible UI state after structured CLI checks. Do not
assume Computer Use exists in Codex CLI, release archives, or non-Codex agents.

## Extended Hours

Scanner-backed quotes may include `extended_hours.premarket` and
`extended_hours.postmarket`. Treat these as additive fields:

- top-level `last` and `close` remain regular scanner quote fields;
- missing extended-hours fields can mean the session is inactive or TradingView
  did not return that value;
- compare premarket/postmarket values only when the corresponding fields are
  present.

## Missing Or Mismatched Data

Handle gaps explicitly:

- `null` or missing fields are unknown, not zero.
- symbol mismatch or exchange ambiguity is a resolution problem, not a market
  signal.
- mixed batch quote results should preserve input order and explain failed
  items separately from successful quotes.
- if scanner and chart sources differ, report both source names and avoid
  forcing a single "correct" value without further evidence.
- if a fundamentals field is missing or `null`, report it as unknown rather
  than as zero or "no earnings".

## Reporting Shape

Keep reports compact:

1. State source and freshness metadata.
2. List the observed values that matter.
3. Separate interpretation from observed data.
4. Name what is still unknown.
5. Suggest the next read only if it changes confidence, such as
   `tv snapshot <SYMBOL>`, `tv quote <SYMBOL> --source chart`,
   `tv fundamentals <SYMBOL>`, `tv ohlcv --summary`, or a screenshot.
