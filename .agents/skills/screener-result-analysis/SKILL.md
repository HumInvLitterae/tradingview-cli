---
name: screener-result-analysis
description: Analyze TradingView scanner or Screener results from the Rust `tv` CLI as research candidates. Use when explaining why rows matched, what patterns appear, or what to research next after scanner or Screener reads.
---

# Screener Result Analysis

Use this skill when the user asks what scanner or Screener results mean, which
rows stand out, or what to do next after a screen.

## Core Rule

Treat scanner and Screener rows as research candidates, not buy or sell
recommendations. Explain what the screen found and what to research next.

## Start With Context

Before interpreting results, identify:

- source: scanner REST, visible full-page Screener rows, hotlist, or chart read;
  preserve `source_category`, `requires_desktop`, and `non_mutating` when they
  are present in the payload;
- filters, columns, sort, hotlist slug, or saved screen name;
- result count and whether the output is a full set or only the first rows;
- freshness notes from `market-data-interpretation` when quote fields are used.

If the task needs field discovery, use `tv scanner metainfo --field <FIELD>`.
If the task moves from scanner rows into chart observation, use
`docs/observation-workflows.md` to choose between `tv readiness`,
`tv observe chart`, lower-level `tv stream ...`, and screenshots.

## Explain Why Rows Matched

Tie conclusions to returned fields. Common angles:

- valuation: price, market cap, price/earnings, or other requested valuation
  fields;
- quality: margins, returns, debt, or stability fields when requested;
- growth: revenue, earnings, or performance fields when requested;
- momentum: performance, RSI, trend, or relative-volume fields when requested;
- liquidity and session behavior: volume, premarket, or postmarket fields;
- concentration: sector, industry, exchange, or market-cap clustering.

If a field is missing, say it is missing. Do not fill gaps from memory.

## Next Research Steps

Choose follow-up reads based on the question:

- `tv compare <SYMBOL>...` when several known candidates need Desktop-free
  quote, info, and fundamentals evidence side by side. Use `summary` for
  quick resolution, section-success, field-coverage, and requested-order
  readback, then inspect raw `items[]` before comparing candidates. Per-item
  `follow_up_hints` name possible next evidence commands; they are not
  recommendations.
- `tv snapshot <SYMBOL>` for a one-symbol Desktop-free evidence packet that
  combines quote, info, and fundamentals sections before chart observation.
- `tv info <SYMBOL>` for symbol metadata when that is the only needed section.
- `tv quote <SYMBOL>` or `tv quotes <SYMBOL>...` for scanner-backed quote
  checks.
- `tv quote <SYMBOL> --source chart` when the selected Desktop chart feed
  matters.
- `tv ohlcv --summary` only after switching to a chart when chart bars are
  needed.
- `tv observe chart --duration-ms <MS> --heartbeat-ms <MS>` when a bounded
  readiness-plus-last-bar observation window is more useful than one static
  chart read.
- `tv screenshot --region chart --output <PATH>` when visual chart evidence is
  useful.

Do not mutate Screener filters, columns, screens, or watchlists without user
approval.

## Output Shape

Prefer a short, educational structure:

1. What the screen asked for.
2. What pattern appeared.
3. Why the notable rows matched.
4. What the data cannot prove.
5. The next 2-4 research steps.

For comparisons or rankings, show the basis for the ranking and call out
missing fields. Avoid turning a ranking into a recommendation.
