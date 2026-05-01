---
name: market-data-interpretation
description: Interpret TradingView market data returned by the Rust `tv` CLI. Use when explaining quote, batch quote, scanner, chart quote, OHLCV, extended-hours, freshness, missing-value, or source differences.
---

# Market Data Interpretation

Use this skill when a task depends on understanding market data returned by the
Rust `tv` CLI rather than merely running the command.

## Source First

Always name the data source before interpreting values:

- `tv quote <SYMBOL>` and `tv quotes <SYMBOL>...`: scanner REST by default.
- `tv quote <SYMBOL> --source scanner`: scanner REST only.
- `tv quote <SYMBOL> --source chart`: selected TradingView Desktop chart feed.
- `tv quote <SYMBOL> --source auto`: chart-first, scanner fallback only before
  chart mutation.
- `tv scanner scan` and `tv scanner metainfo`: scanner REST.
- `tv ohlcv`: selected chart bars through TradingView Desktop/CDP.
- screenshots and visible values: current visual chart state.

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
bounded readiness wait and one retry; do not add manual sleep or double-call
workarounds downstream. If it fails, report the structured freshness details or
use scanner source explicitly when scanner freshness is acceptable. `tv ohlcv`
is chart-dependent; do not describe it as Desktop-free historical bars.

For Desktop-backed reads, inspect structured readiness fields before escalating
to visual tools: `tv status` / `tv tab list` expose endpoint and target
readiness, `tv state` exposes chart readiness, chart-source quote exposes
`freshness_check`, and OHLCV failures expose chart-bars details. Computer Use is
useful for visual confirmation or UI recovery after these fields are
inconclusive; it should not replace them as the first diagnostic step.

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

## Reporting Shape

Keep reports compact:

1. State source and freshness metadata.
2. List the observed values that matter.
3. Separate interpretation from observed data.
4. Name what is still unknown.
5. Suggest the next read only if it changes confidence, such as
   `tv quote <SYMBOL> --source chart`, `tv info <SYMBOL>`,
   `tv ohlcv --summary`, or a screenshot.
