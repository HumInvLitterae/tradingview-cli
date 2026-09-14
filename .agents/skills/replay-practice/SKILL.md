---
name: replay-practice
description: Run or review bounded TradingView Replay practice with tv when Replay state and bar-by-bar observations are the task.
---

# Replay practice

Use Replay for practice or investigation of the selected Desktop chart.
For historical datasets, use [historical bars](../market-data/references/historical-bars.md).
Resolve the [Desktop session](../chart-analysis/references/desktop-session.md)
only when needed. Inspect `tv replay status` and chart context before mutation.

| Requested action | Command |
| --- | --- |
| Inspect Replay | `tv replay status` |
| Start at a date | `tv replay start --date <YYYY-MM-DD>` |
| Advance one step | `tv replay step` |
| Record a bounded sequence | `tv replay log --steps <N>` |
| Autoplay at a chosen speed | `tv replay autoplay [--speed <MS>]` |
| Record an approved practice trade | `tv replay trade buy`, `sell`, or `close` |
| Stop Replay | `tv replay stop` |

Start/step/log/autoplay/trade/stop change Replay or Replay-trade state. Keep the
requested bounds and end state explicit. Reuse an already agreed practice scope;
do not close a pre-existing practice position or stop Replay as automatic cleanup.
Use symbol/timeframe/range changes only when the requested setup needs them.

For per-step bars or images, read [attachments and end-state checks](references/workflow.md).
Report the observations, Replay outcome, attachment outcome, and whether the
requested end state was confirmed. Replay logs are workflow evidence, not a
stable historical export or a trading recommendation.
