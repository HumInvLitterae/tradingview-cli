---
name: multi-symbol-scan
description: Scan or compare a small set of TradingView symbols with the Rust `tv` CLI. Use when the user wants a watchlist-style pass, cross-symbol quote/OHLCV comparison, technical screen, or shortlist based on currently available CLI reads.
---

# Multi-Symbol Scan

Use this skill to compare several TradingView symbols through the Rust `tv` CLI without pretending an unavailable bulk batch-run helper exists.

## Start With Scope

1. Confirm the symbol list, timeframe, and screening criteria from the user request.
2. Run `tv status`; if needed, run `tv watchlist get` to inspect the current TradingView watchlist.
3. If more than one chart target is open, run `tv tab list` and use `target_cli_args`, for example `tv --target-id <ID> ...`, for any chart-specific follow-up. Do not use `TV_CDP_TARGET_ID`.
4. For broad discovery, prefer `tv scanner hotlist` or `tv scanner scan` before mutating the chart across many symbols.
5. Keep chart-by-chart inspection small and serial. The Rust CLI does not implement the old MCP `batch_run` helper.

## Scan Workflow

1. Use `tv scanner hotlist <SLUG> --limit <N>` or `tv scanner scan ...` for
   broad read-only discovery when the criteria can be expressed as scanner
   filters. Use `tv scanner metainfo --field <FIELD>` when you need to confirm
   scanner field availability.
2. For a small finalist set, use `tv quotes <SYMBOL>...` for ordered
   Desktop-free batch quote reads, `tv quote <SYMBOL>` for one-off quote
   checks, and `tv info <SYMBOL>` for symbol metadata.
3. Treat scanner-backed price reads as screening evidence rather than a
   realtime entitlement guarantee. Use `tv quote <SYMBOL> --source chart` only
   for symbols where the selected TradingView Desktop chart feed matters.
4. Set the timeframe once with `tv timeframe <RESOLUTION>` when the scan uses a shared timeframe.
5. Switch the chart with `tv symbol <SYMBOL>` only when OHLCV, visible studies,
   drawings, or screenshots are needed. After switching, confirm fresh chart
   data with `tv ohlcv --count 1` or `tv ohlcv --summary`.
6. Gather `tv ohlcv --summary`, `tv values`, and drawing-derived reads such as
   `tv data lines`, `tv data labels`, or `tv data boxes` only for symbols that
   need chart context.
7. Use `tv stream quote`, `tv stream bars`, or `tv stream all` only for short
   live-monitoring windows after the scan identifies symbols worth watching.
8. After user approval, add selected symbols with
   `tv watchlist add-bulk <SYMBOL>... --allow-partial`; it inherits the
   API-backed single-symbol add path and reports duplicates or partial
   failures.
9. Capture screenshots selectively for finalists or ambiguous cases with `tv screenshot --region chart --output <PATH>`.
10. Present a ranked shortlist and explain which observations came from scanner REST data, chart reads, or visual interpretation.

Use `market-data-interpretation` when quote freshness, source differences,
extended-hours fields, or missing values matter. Use `screener-result-analysis`
when explaining why scanner or Screener rows matched a screen.

## OHLCV Recovery

If a finalist chart read returns an `ohlcv` failure while symbol or quote reads
still work, do not keep retrying the same target. Preserve the full JSON error
envelope, inspect `error.details`, rerun `tv tab list`, choose the active chart
target's `target_cli_args`, run `tv --target-id <ID> state`, and retry
`tv --target-id <ID> ohlcv --count 1`.

Use `tv timeframe <RESOLUTION>` for shared timeframe setup. `tv interval` is
not a command. Use `tv info <SYMBOL>` for Desktop-free symbol metadata, and use
`tv info` without a symbol only for current-chart metadata.

## Boundaries

The Rust CLI can inspect and mutate the current watchlist, read scanner REST data, read chart-model data, and stream read-only chart samples as JSONL. Watchlist add/remove prefer API-backed mutation with readback checks, but still require user approval because they change account state. The CLI does not compute arbitrary historical indicator series, run strategy batches, or provide a generic batch-run helper.

Read `references/workflow.md` when the task needs the original MCP scan shape translated into the current CLI surface.
