---
name: market-data-interpretation
description: Interpret TradingView market data returned by the Rust `tv` CLI. Use when explaining quote, batch quote, scanner, chart quote, OHLCV, extended-hours, freshness, missing-value, or source differences.
---

# Market Data Interpretation

Use this skill when a task depends on understanding market data returned by the
Rust `tv` CLI rather than merely running the command.

## Source First

Always name the data source before interpreting values:

- Desktop-free reads: `tv quote <SYMBOL>`, `tv quotes <SYMBOL>...`,
  `tv fundamentals <SYMBOL>`, `tv scanner scan`, and
  `tv scanner metainfo`.
- Desktop-backed reads: `tv quote <SYMBOL> --source chart`, current-chart
  `tv quote`, `tv ohlcv`, screenshots, and current visible values.
- Hybrid reads: `tv quote <SYMBOL> --source auto`, which is chart-first with
  scanner fallback only before chart mutation.
- `TV_EXPERIMENTAL_BARS=1 tv bars <EXCHANGE:SYMBOL>`: experimental
  Desktop-free WebSocket bars.
- `tv stream ...`: Desktop-backed current-chart JSONL observation, not
  browserless WebSocket streaming. Prefer bounded windows with
  `--duration-ms`, `--max-events`, and optional `--heartbeat-ms`.

Do not blend scanner REST, chart feed, and visible chart observations as if
they were the same source.

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
source explicitly when scanner freshness is acceptable. `tv ohlcv` is
chart-dependent; do not describe it as Desktop-free historical bars.

`tv bars` is different from both scanner REST and `tv ohlcv`. It is a
lab-gated browserless historical bars prototype using an undocumented
TradingView WebSocket path. Use it only when the user or workflow explicitly
accepts experimental data; report `source`, `experimental`, `data_quality`,
and warnings. Do not treat it as a stable replacement for chart-sourced OHLCV.

`tv fundamentals` is a Desktop-free scanner read, not a chart read. Use it for
raw fields such as market cap, P/E, EPS, dividend yield, and earnings
date/time. Prefer `--group earnings`, `--group valuation`, `--group dividends`,
or `--group financials` when the task needs a coherent bundle; use `--field`
for exact scanner fields. Treat `field_values` as the source of truth. Do not
infer timezone, before/after-market meaning, financial analysis, or investment
recommendations from these fields unless another source explicitly supplies
that interpretation.

For Desktop-backed reads, inspect structured readiness fields before escalating
to visual checks: `tv status` / `tv tab list` expose endpoint and target
readiness, `tv state` exposes chart readiness, chart-source quote exposes
`freshness_check`, and OHLCV failures expose chart-bars details. The portable
visual fallback is `tv screenshot --region chart|full --output <PATH>` plus
user/manual inspection when needed.

For `tv stream ...`, interpret each JSONL line by `data._event`. A `sample`
event means the chart/page sample changed after metadata-insensitive dedupe. A
`heartbeat` event means the stream is still alive but no changed sample was
emitted in that heartbeat window. Do not count heartbeat events as market
updates. Stream events should identify `source: "desktop_chart_stream"`,
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
   `tv quote <SYMBOL> --source chart`, `tv info <SYMBOL>`,
   `tv fundamentals <SYMBOL>`, `tv ohlcv --summary`, or a screenshot.
