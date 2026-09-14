# Observation Workflows

Choose the command from the maintained runtime tables below. The same skills
and references ship with the binary. [Source taxonomy](command-source-taxonomy.md)
defines source and side-effect contracts; this page routes practical usage.

## Desktop-Free Screening

Use [market-data](../.agents/skills/market-data/SKILL.md) for discovery, prices,
known-symbol comparison, fundamentals, and historical bars. Its table pairs the
first command with the condition for a further read; do not execute all rows.

## Which Read To Use

The [market command table](../.agents/skills/market-data/SKILL.md#choose-the-first-command-and-the-next-step)
distinguishes quote-only reads, richer packets, scanner discovery, events,
historical bars, and bounded watches. The
[chart table](../.agents/skills/chart-analysis/SKILL.md#choose-the-evidence)
covers selected-chart evidence and operations. Visible/saved Screener work uses
[screener-workflow](../.agents/skills/screener-workflow/SKILL.md).

## Follow-up Vocabulary

Read [screening and comparison](../.agents/skills/market-data/references/screening-and-comparison.md)
for packet coverage, missing evidence, stable follow-up kinds, and why rows
matched. Hints do not run commands or authorize new effects.

## Desktop-Backed Chart Observation

Use [Desktop session guidance](../.agents/skills/chart-analysis/references/desktop-session.md)
for connection and target reuse, and
[bounded observations](../.agents/skills/market-data/references/observations.md)
for observe/stream event semantics.

### Verified native parallel channels

See [drawing and chart evidence](../.agents/skills/chart-analysis/references/workflow.md)
for paired third-point parameters and verified entity identity.

## Visual Evidence Recovery

Use [chart screenshot guidance](../.agents/skills/chart-analysis/references/workflow.md)
when structured reads do not explain the visible state. Computer-control tools
are optional and environment-dependent; the packaged workflow does not require them.
For Strategy Tester evidence, use [strategy-report](../.agents/skills/strategy-report/SKILL.md).

## Browserless Historical Bars

Use [historical bars](../.agents/skills/market-data/references/historical-bars.md)
for supported date-range timeframes, count limits, timestamp boundaries,
completeness, and downstream window splitting.

## Selected-Chart Historical Export

Read [range and export evidence](../.agents/skills/chart-analysis/references/workflow.md)
when the selected chart itself is the required source and viewport movement is
in scope. A successful viewport change alone is not export completeness.

## Replay Extraction Feasibility

Use [replay-practice](../.agents/skills/replay-practice/SKILL.md) for bounded
Replay state transitions and optional per-step attachments. Replay is practice
and workflow evidence, not a stable historical data export.

## Fundamentals And Event-Like Fields

Use [quote and event semantics](../.agents/skills/market-data/references/quotes-and-events.md)
for earnings, dividends, extended hours, and source availability. Event fields
are not a complete calendar.

## Deferred Surfaces

Current priorities and conditions for further work belong to the
[planning index](plans/README.md). A runtime limitation does not itself authorize
new fallback behavior, a daemon, a different provider, or new product scope.
