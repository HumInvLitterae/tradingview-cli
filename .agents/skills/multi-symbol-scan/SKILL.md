---
name: multi-symbol-scan
description: Scan or compare a small set of TradingView symbols with the Rust `tv` CLI. Use when the user wants a watchlist-style pass, cross-symbol quote/OHLCV comparison, technical screen, or shortlist based on currently available CLI reads.
---

# Multi-Symbol Scan

Use this skill to compare several TradingView symbols with the Rust `tv` CLI.
Default to Desktop-free reads for broad comparison. Move to Desktop-backed
selected-chart reads only for finalists that need chart bars, visible studies,
Replay context, or screenshots.

Use this `SKILL.md` for routine scans. Read `references/workflow.md` only when
you need the detailed command map, source notes, or unsupported-feature
reminders. If command choice is unclear, check
`docs/observation-workflows.md`.

## Read Selection

| Need | Prefer |
| --- | --- |
| Broad discovery | `tv scanner scan` or `tv scanner hotlist` |
| Known-symbol comparison | `tv compare <SYMBOL>...` |
| Short bounded watch of known symbols | `tv watch compare <SYMBOL>...` |
| Quote-only comparison | `tv quotes <SYMBOL>...` |
| One-symbol detail | `tv snapshot <SYMBOL>` |
| Earnings or dividend context | `tv events <SYMBOL>` |
| Historical bars | `tv bars <SYMBOL> --from ... --to ...` |
| Finalist chart evidence | `tv quote --source chart`, `tv ohlcv`, `tv values`, `tv screenshot` |

## Core Workflow

1. Clarify the symbol list, timeframe, and screening criteria from the user
   request. Do not invent a ranking method unless the user gave one.
2. For broad discovery, use `tv scanner hotlist` or `tv scanner scan`. Use
   `tv scanner metainfo --field <FIELD>` when field availability is unclear.
3. For a known list, use `tv compare <SYMBOL>...` first. Read `summary` for
   coverage and `items[]` for the actual evidence. Treat `follow_up_hints[]`
   as advisory next evidence surfaces, not recommendations.
4. Use `tv watch compare <SYMBOL>... --duration-ms <MS> --interval <MS>` only
   when the same candidate set needs a short scanner-backed watch window.
   Report `watch_compare.v1` event counts, source marker, and end reason.
5. Use `tv bars` for reproducible historical bars. When a bare symbol resolves
   automatically, report `requested_symbol`, `resolved_symbol`, and
   `symbol_resolution`; retry with `EXCHANGE:SYMBOL` if the exchange matters.
6. Use `tv events <SYMBOL>` for scanner-backed earnings and dividend evidence.
   Treat it as event context, not a complete calendar or recommendation.
7. Escalate to TradingView Desktop only for finalists. Run `tv readiness`; if
   multiple targets are open, run `tv tab list` and use `target_cli_args`.
8. For finalist chart checks, set `tv symbol` and `tv timeframe`, confirm with
   `tv ohlcv --count 1` or `tv ohlcv --summary`, then read selected-chart
   evidence with `tv quote --source chart`, `tv values`, drawing reads, or
   screenshots.
9. Add watchlist entries only after user approval with
   `tv watchlist add-bulk <SYMBOL>... --allow-partial`.

## Source Boundaries

- Keep scanner REST data, `tv bars`, selected-chart reads, chart export,
  Replay, screenshots, and event readback as separate evidence sources.
- `tv compare` and `tv watch compare` are Desktop-free scanner-backed
  workflows. Chart-backed compare is not a stable command.
- `tv export chart-bars` is selected-chart export evidence, not a fallback for
  multi-symbol historical sample preparation.
- `tv replay log` is bounded Replay workflow evidence, not historical bars
  input.
- Do not present scanner results, event fields, or follow-up hints as
  ranking, recommendation, scoring, buy/sell advice, or automatic source
  mixing.

## Recovery

If finalist selected-chart OHLCV fails while symbol or quote reads work, keep
the full JSON error envelope. Inspect `error.details.phase`,
`bar_index_state`, and `next_action_hint`; rerun `tv readiness`, choose a
target with `tv tab list` if needed, and retry with that `--target-id`.

## Reporting

Present any shortlist as evidence grouped by the user's criteria, with the
evidence source for each point. Name missing fields and freshness limitations.
Use `market-data-interpretation` when the main issue is quote freshness, source
differences, fundamentals, events, extended-hours fields, or missing values.
Use `screener-result-analysis` when explaining why scanner or Screener rows
matched a screen.
