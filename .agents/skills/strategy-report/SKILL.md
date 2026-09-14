---
name: strategy-report
description: Read and explain TradingView strategy metrics, trades, and equity with tv when the user requests a Strategy Tester report.
---

# Strategy report

Resolve the [Desktop session](../chart-analysis/references/desktop-session.md)
if the intended chart is not already known. Gather only the requested evidence:

| Need | Command |
| --- | --- |
| Metrics | `tv data strategy` |
| Trade list | `tv data trades --max <N>` |
| Equity data | `tv data equity` |
| Visible Strategy Tester image | `tv screenshot --region strategy --output <PATH>` |

Check that structured results identify the same strategy through
`strategy_context` before combining them. `strategy_hidden`, `report_not_ready`,
`ambiguous`, and `not_found` are diagnostics, not zero performance. These reads
do not open Strategy Tester or unhide studies.

Read [strategy identity and availability](references/workflow.md) for incomplete
or conflicting results. Fetch chart, price, or study context only when it helps
answer the report question. Separate metrics, visual evidence, and analysis;
leave missing trades or equity unavailable rather than reconstructing them from
summary metrics. Follow-up state changes require the user's intent.
