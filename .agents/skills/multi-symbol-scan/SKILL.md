---
name: multi-symbol-scan
description: Scan or compare a small set of TradingView symbols with the Rust `tv` CLI. Use when the user wants a watchlist-style pass, cross-symbol quote/OHLCV comparison, technical screen, or shortlist based on currently available CLI reads.
---

# Multi-Symbol Scan

Use this skill to compare several TradingView symbols through the Rust `tv` CLI without pretending an unavailable bulk batch-run helper exists.

## Start With Scope

1. Confirm the symbol list, timeframe, and screening criteria from the user request.
2. Run `tv status`; if needed, run `tv watchlist get` to inspect the current TradingView watchlist.
3. If more than one chart target is open, run `tv tab list` and use `target_cli_args`, for example `tv --target-id <ID> ...`, for any chart-specific follow-up. `target_env.TV_CDP_TARGET_ID` is a v0.2.x fallback only.
4. For broad discovery, prefer `tv scanner hotlist` or `tv scanner scan` before mutating the chart across many symbols.
5. Keep chart-by-chart inspection small and serial. The Rust CLI does not implement the old MCP `batch_run` helper.

## Scan Workflow

1. Use `tv scanner hotlist <SLUG> --limit <N>` or `tv scanner scan ...` for broad read-only discovery when the criteria can be expressed as scanner filters.
2. For a small finalist set, use `tv quote <SYMBOL>` for quick symbol-targeted quotes; it first tries a non-mutating scanner read and only falls back to temporary chart switching when needed.
3. Set the timeframe once with `tv timeframe <RESOLUTION>` when the scan uses a shared timeframe.
4. Switch the chart with `tv symbol <SYMBOL>` only when OHLCV, visible studies, drawings, or screenshots are needed. After switching, confirm fresh chart data with `tv ohlcv --count 1` or `tv ohlcv --summary`.
5. Gather `tv ohlcv --summary`, `tv values`, and drawing-derived reads such as `tv data lines`, `tv data labels`, or `tv data boxes` only for symbols that need chart context.
6. Use `tv stream quote`, `tv stream bars`, or `tv stream all` only for short live-monitoring windows after the scan identifies symbols worth watching.
7. After user approval, add selected symbols with `tv watchlist add-bulk <SYMBOL>... --allow-partial`; it inherits the API-backed single-symbol add path and reports duplicates or partial failures.
8. Capture screenshots selectively for finalists or ambiguous cases with `tv screenshot --region chart --output <PATH>`.
9. Present a ranked shortlist and explain which observations came from scanner REST data, chart reads, or visual interpretation.

## Boundaries

The Rust CLI can inspect and mutate the current watchlist, read scanner REST data, read chart-model data, and stream read-only chart samples as JSONL. Watchlist add/remove prefer API-backed mutation with readback checks, but still require user approval because they change account state. The CLI does not compute arbitrary historical indicator series, run strategy batches, or provide a generic batch-run helper.

Read `references/workflow.md` when the task needs the original MCP scan shape translated into the current CLI surface.
