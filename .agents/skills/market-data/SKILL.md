---
name: market-data
description: Fetch and interpret TradingView data with tv for symbol discovery, comparisons, historical bars, or source and freshness questions.
---

# Market data with tv

Choose the row that answers the user's question. These first commands are
Desktop-free unless the row says otherwise. Do not run the whole table or load
all references. Use `tv --version` when the binary is new or build identity
matters; use `tv <family> --help` when arguments are uncertain.

## Choose the first command and the next step

| User needs | First command | Continue only when |
| --- | --- | --- |
| Resolve a name or exchange | `tv search "<QUERY>"` | Fix `EXCHANGE:SYMBOL` when multiple listings matter. |
| One price / several prices | `tv quote <SYMBOL>` / `tv quotes <SYMBOL>...` | Freshness or session fields need [quote interpretation](references/quotes-and-events.md). |
| Symbol metadata only | `tv info <SYMBOL>` | A specific missing fact needs a separate source. |
| One symbol's quote, info, fundamentals | `tv snapshot <SYMBOL>` | `summary` or section errors show a gap relevant to the question. |
| Same detail for known symbols | `tv compare <SYMBOL>...` | Inspect ordered `items[]` after coverage; narrow to the user's criteria. |
| Discover candidates by filters | `tv scanner scan` | Inspect fields, sort, coverage and [screen interpretation](references/screening-and-comparison.md); use metainfo for unknown fields. |
| A TradingView hotlist | `tv scanner hotlist <SLUG>` | The returned list needs explanation or selected candidates need more evidence. |
| Discover available scanner fields | `tv scanner metainfo --field <FIELD>` | Use the returned definition to construct the intended scan. |
| Fundamentals only | `tv fundamentals <SYMBOL> --group <GROUP>` | A missing field changes the answer; do not fetch an entire packet by default. |
| Earnings or dividends | `tv events <SYMBOL> --event-type earnings` (or `dividends`); `tv events compare <SYMBOL>...` for several | Read [event semantics](references/quotes-and-events.md) when dates or missing fields matter. |
| Explicit official MCP recent bars | `tv mcp status`, then `tv mcp bars <EXCHANGE:SYMBOL> --timeframe <1D/1W/1M> --count <N>` | The binary supports `tv mcp --help` and the user explicitly selected this source. Login is interactive; do not run it silently. See [MCP semantics](references/historical-bars.md#official-mcp). |
| Recent OHLCV / a historical date interval | `tv bars <SYMBOL> --timeframe <TF> --count <N>` / add `--from <YYYY-MM-DD> --to <YYYY-MM-DD>` | Read [bars coverage](references/historical-bars.md) before claiming completeness or splitting requests. |
| Observe a known set over a short window | `tv watch compare <SYMBOL>... --duration-ms <MS> --interval <MS>` | Interpret event types using [bounded observations](references/observations.md); stop at the requested bound. |
| The Desktop chart, its studies, or an image | [chart-analysis](../chart-analysis/SKILL.md) | The selected chart is required evidence; resolve its target before using it. |
| Visible or saved Desktop Screener state | [screener-workflow](../screener-workflow/SKILL.md) | The task concerns that screen, its filters/columns, or saved state. |

For detailed multi-symbol packets or why rows matched, read
[screening and comparison](references/screening-and-comparison.md). For a simple
price read, this entrypoint and the returned fields usually suffice.

## Interpret and finish

Keep the actual `source`, `source_category`, symbol resolution, observation time,
and relevant freshness/coverage fields with the result. Missing or null is
unknown, not zero. If sources disagree, explain their different contexts rather
than silently selecting or blending values.

Compare using the user's criteria. Input order, coverage summaries, and
`follow_up_hints[]` are data/diagnostics, not CLI-generated rankings or trading
recommendations. `auto_execute: false` means the CLI did not run the hint; an
agent may choose a separate needed read within the task's authority. Explain
observations and inferences separately and stop when the question is answered.
