---
name: chart-analysis
description: Inspect a TradingView Desktop chart with tv when the user needs chart bars, studies, viewport changes, or screenshot evidence.
---

# Chart analysis

Use the selected Desktop chart as the source. For Desktop-free prices, symbol
comparison, or historical bars, use the optional `market-data` skill.
Resolve the [Desktop session](references/desktop-session.md) when the target is
new, uncertain, or changed; reuse a confirmed target across the workflow.

On binaries with `spec`, inspect `tv spec <command path>` when arguments or
effects are uncertain. Symbol, timeframe, chart type, range and info have
argument-dependent variants: null common effects do not mean read-only. Use
`tv tab list` to select a target ID before actual Desktop operations. Older
binaries still use `--help`; specification lookup itself never connects.

## Choose the evidence

| Need | Command | Condition or readback |
| --- | --- | --- |
| Chart identity and current state | `tv state` | Confirm symbol and resolution on the intended target. |
| Main-series quote | `tv quote --source chart` | A supplied different symbol may switch and restore the chart. |
| Chart bars | `tv ohlcv --summary` or `tv ohlcv --count <N>` | After an authorized symbol/timeframe change, confirm fresh data before interpreting it. |
| Visible study values | `tv values` | Use entity identity and inputs for same-name studies. |
| Drawing-derived data | `tv data lines`, `labels`, `tables`, or `boxes` | Report returned values only. |
| Chart image | `tv screenshot --region chart --output <PATH>` | Add `--wait-for-render` after state changes when stable context is needed. |
| Read / change viewport | `tv range` / `tv range --from <UNIX_SECONDS> --to <UNIX_SECONDS>` | Bounds change the viewport; use [range evidence](references/workflow.md). |
| Change symbol, timeframe, or chart type | `tv symbol <SYMBOL>`, `tv timeframe <RESOLUTION>`, `tv type <TYPE>` | Only for a requested change or approved chart evidence workflow. |
| Compare a small set using the chart feed | `tv chart compare <SYMBOL>...` | Temporary chart switching; check item status and restoration. |
| Export this chart's historical range | `tv export chart-bars --from <UNIX_SECONDS> --to <UNIX_SECONDS>` | Moves the viewport; read [export evidence](references/workflow.md). |

Read [bounded observations](references/observations.md) for
`tv observe chart` or lower-level streams. Use
the optional `strategy-report` skill for strategy results and
the optional `replay-practice` skill for Replay operations.

Lead with the chart finding and its evidence. Distinguish observed bars/studies,
visual observations, and inference. Do not invent indicator values, rankings,
or buy/sell recommendations. A chart image or viewport change alone does not
prove historical export completeness.
