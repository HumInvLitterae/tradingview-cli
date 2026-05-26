---
name: multi-symbol-scan
description: Scan or compare a small set of TradingView symbols with the Rust `tv` CLI. Use when the user wants a watchlist-style pass, cross-symbol quote/OHLCV comparison, technical screen, or shortlist based on currently available CLI reads.
---

# Multi-Symbol Scan

Use this skill to compare several TradingView symbols through the Rust `tv`
CLI using Desktop-free comparison first.
Default to Desktop-free reads for broad comparison. Move to Desktop-backed
reads only for finalists that need selected-chart OHLCV, visible studies, or
screenshots. Treat `tv quote --source auto` as hybrid source selection and
`tv bars` as Desktop-free historical bars evidence, not realtime data.

For the current observation command sequence, use
`docs/observation-workflows.md`. This skill should stay focused on shortlist
construction, not on duplicating the full source taxonomy.

## Read Selection

| Need | Prefer |
| --- | --- |
| Broad scanner discovery | `tv scanner scan` or `tv scanner hotlist` |
| Known-symbol first-pass comparison | `tv compare <SYMBOL>...` |
| Known-symbol short-window watch | `tv watch compare <SYMBOL>...` |
| Quote-only comparison | `tv quotes <SYMBOL>...` |
| One-symbol finalist detail | `tv snapshot <SYMBOL>` |
| Finalist time-window chart evidence | `tv observe chart --duration-ms ...` |
| Finalist chart-feed quote | `tv quote <SYMBOL> --source chart` |
| Visual ambiguity | `tv screenshot --region chart --output <PATH>` |

## Start With Scope

1. Confirm the symbol list, timeframe, and screening criteria from the user request.
2. Run `tv readiness`; if needed, run `tv watchlist get` to inspect the current TradingView watchlist.
3. Inspect `ready`, `target_selection`, `chart_readiness`, `bars_readiness`,
   and `next_action_hint` before assuming the Desktop session is usable. If
   more than one chart target is
   open, run `tv tab list` and use `target_cli_args`, for example
   `tv --target-id <ID> ...`, for any chart-specific follow-up. Do not use
   `TV_CDP_TARGET_ID`.
4. For broad discovery, prefer `tv scanner hotlist` or `tv scanner scan` before mutating the chart across many symbols.
5. Keep chart-by-chart inspection small and serial. The Rust CLI does not implement the old MCP `batch_run` helper.

## Scan Workflow

1. Use `tv scanner hotlist <SLUG> --limit <N>` or `tv scanner scan ...` for
   broad read-only discovery when the criteria can be expressed as scanner
   filters. Use `tv scanner metainfo --field <FIELD>` when you need to confirm
   scanner field availability.
2. For several known symbols, use `tv compare <SYMBOL>...` for an ordered
   Desktop-free comparison packet with quote, info, and fundamentals sections.
   Read `summary` for requested/resolved/error counts, section success counts,
   missing totals, requested-to-resolved symbol mapping, field coverage, and
   requested-order indexes. `summary.coverage_status` is only evidence
   completeness (`complete`, `partial`, or `blocked`); keep raw `items[]` as
   the evidence source for any actual comparison. For regular-session movement,
   use `items[].movement.regular_change_percent` as the stable first-pass
   readback and confirm against raw `items[].sections.quote.data.change` when
   needed; do not infer absolute change from last/close. Per-item `follow_up_hints`
   are available next evidence surfaces, not recommendations. Stable follow-up
   kinds are `snapshot`, `chart_quote`, `observe_chart`, and `screenshot`;
   keep `chart_quote` as the canonical selected-chart quote kind and do not
   rename it to `quote_chart`. Use `items[].missing_evidence[]` to route
   section gaps to stable follow-up kinds such as `snapshot` or `chart_quote`;
   do not treat those hints as symbol ranking.
   Use `tv quotes <SYMBOL>...` only when ordered quote fields are enough. Use
   `tv snapshot <SYMBOL>` for one-symbol first-pass evidence before mutating
   the chart; it combines quote, symbol info, and fundamentals sections.
   Snapshot `summary` and `missing_evidence[]` provide one-symbol coverage and
   follow-up routing readback, while raw `sections` remain the evidence. Use
   lower-level `tv quote`, `tv info`, or
   `tv fundamentals <SYMBOL> --group earnings|valuation|dividends|financials`
   only when one section is enough. Preserve
   `source_category: "desktop_free_read"`, `requires_desktop: false`, and
   `non_mutating: true` when reporting this REST-backed evidence.
   Use `tv watch compare <SYMBOL>... --duration-ms <MS> --interval <MS>` when
   the same known candidate set needs a short scanner-backed JSONL watch
   window. Read `watch_compare.v1` readiness / sample / heartbeat / summary
   events and report sample count, poll count, error count, source marker, and
   end reason. This is not a daemon, selected-chart feed, ranking, or trading
   recommendation.
3. Treat scanner-backed price reads as screening evidence rather than a
   realtime entitlement guarantee. Use `tv quote <SYMBOL> --source chart` only
   for symbols where the selected TradingView Desktop chart feed matters. Do
   not implement manual sleep or double-call workarounds; chart-source quote
   readiness is handled by the CLI with consecutive stable samples and will
   fail if stale chart bars remain. Do not build multi-symbol realtime loops
   on chart-source quote; concurrent or nearby chart mutations can still
   contend with the visible chart. Prefer Desktop-free reads for broad
   comparison, then use chart-source quote only for finalist symbols that need
   the selected Desktop feed. Do not use chart-source quote for premarket or
   postmarket fields; use scanner-backed `tv quote`, `tv quotes`,
   `tv snapshot`, or `tv compare` when extended-hours evidence matters.
   Desktop quote-session probes can expose pre/post field names, but they are
   phase-sensitive live evidence and not a stable multi-symbol screening
   source. A `post-market` phase observation does not make quote-session
   pre/post close fields equivalent to scanner `extended_hours`.
   Use `tv bars <EXCHANGE:SYMBOL> --count <N>` when bounded Desktop-free
   historical bars are useful; use `--from YYYY-MM-DD --to YYYY-MM-DD` with
   `--timeframe 5`, `15`, `30`, `60`, `1D`, `1W`, or `1M` for reproducible older
   intraday, daily, weekly, or monthly samples. Other intraday timeframes
   remain guarded in date-range mode. In that mode, `--count` defaults to 500
   and may be raised up to 5000 as a returned-bar safety cap; recent count mode
   remains capped at 500. The `--to` date is inclusive. Read
   `contract_version: "bars.v1"`, `source: "tradingview_bars_ws"`,
   `summary`, `range`, `requested_range`, `returned_range`,
   `range_coverage_status`, `range_alignment`, `range_fetch_summary`,
   `source_availability`, and `data_quality`, and
   keep it separate from `tv ohlcv` chart evidence. Treat unavailable bars as
   source diagnostics, not as proof that a symbol has no history.
4. Set the timeframe once with `tv timeframe <RESOLUTION>` when the scan uses a shared timeframe.
5. Switch the chart with `tv symbol <SYMBOL>` only when OHLCV, visible studies,
   drawings, or screenshots are needed. After switching, confirm fresh chart
   data with `tv ohlcv --count 1` or `tv ohlcv --summary`.
6. Gather `tv ohlcv --summary`, `tv values`, and drawing-derived reads such as
   `tv data lines`, `tv data labels`, or `tv data boxes` only for symbols that
   need chart context.
7. Use `tv observe chart --duration-ms <MS> --heartbeat-ms <MS>` for a bounded
   readiness-plus-last-bar observation window after the scan identifies symbols
   worth watching. Use lower-level `tv stream quote`, `tv stream bars`, or
   `tv stream all` only when a specific stream sample type is needed. Read
   JSONL `source_category`, `requires_desktop`, and `non_mutating` metadata
   before comparing those observations with Desktop-free scanner or quote
   results. Apply the same source metadata check to `tv state`, `tv ohlcv`,
   and chart-source quote payloads. Use `tv watch compare` instead when the
   short-window question is about a known candidate set using Desktop-free
   scanner-backed quote evidence.
8. After user approval, add selected symbols with
   `tv watchlist add-bulk <SYMBOL>... --allow-partial`; it inherits the
   API-backed single-symbol add path and reports duplicates or partial
   failures.
9. Capture screenshots selectively for finalists or ambiguous cases with
   `tv screenshot --region chart --output <PATH>`. Treat screenshot output as
   Desktop-backed visual evidence with `writes_file: true`, not as a market
   data source.
10. Present a shortlist or candidate comparison and explain which observations
    came from scanner REST data, chart reads, or visual interpretation. Do not
    imply that `tv compare` itself ranked or recommended the symbols.

Use `market-data-interpretation` when quote freshness, source differences,
extended-hours fields, fundamentals, earnings date/time fields, or missing
values matter. Use `screener-result-analysis` when explaining why scanner or
Screener rows matched a screen.

## OHLCV Recovery

If a finalist chart read returns an `ohlcv` failure while symbol or quote reads
still work, do not keep retrying the same target. Preserve the full JSON error
envelope, inspect `error.details.phase`, `bar_index_state`, and
`next_action_hint`, rerun `tv readiness`, choose the active chart target's
`target_cli_args`, run `tv --target-id <ID> state`, and retry
`tv --target-id <ID> ohlcv --count 1`. If structured target and chart
readiness fields do not explain the visible state, capture
`tv screenshot --region chart --output <PATH>` or ask the user to inspect the
chart.

If the current environment is the Codex app and Computer Use is available, it
can be used as an optional visual inspection aid after the CLI checks. Do not
assume it is available for Codex CLI, packaged agents, or other CLI-only
runtimes.

Use `tv timeframe <RESOLUTION>` for shared timeframe setup. `tv interval` is
not a command. Use `tv compare <SYMBOL>...` for multi-symbol Desktop-free
evidence, `tv snapshot <SYMBOL>` for one-symbol Desktop-free quote, info, and
fundamentals context, including coverage and missing-evidence readback, and
`tv info <SYMBOL>` or `tv fundamentals <SYMBOL>` with `--group` or `--field`
only when that narrower section is all you need. Use `tv info` without a
symbol only for current-chart metadata.

## Boundaries

The Rust CLI can inspect and mutate the current watchlist, read scanner REST data, read chart-model data, and stream read-only chart samples as JSONL. Watchlist add/remove prefer API-backed mutation with readback checks, but still require user approval because they change account state. The CLI does not compute arbitrary historical indicator series, run strategy batches, or provide a generic batch-run helper.

Read `references/workflow.md` when the task needs the original MCP scan shape translated into the current CLI surface.
