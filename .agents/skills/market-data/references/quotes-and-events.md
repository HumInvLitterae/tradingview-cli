# Quotes, sessions, and events

## Price sources

| Source | When to use | Meaning and limits |
| --- | --- | --- |
| `tv quote <SYMBOL>`, `tv quotes`, snapshot/compare quote sections | Ordinary Desktop-free price or extended-hours checks | Scanner REST; inspect `time`, `update_mode`, `delay_seconds` when freshness matters. Desktop-free does not guarantee realtime entitlements. |
| `tv quote --source chart` | The selected chart's main series is the evidence | Desktop-backed. Supplying a different symbol can temporarily switch and restore the chart; follow [Desktop session guidance](../../chart-analysis/references/desktop-session.md). |
| `tv quote <SYMBOL> --source quote-data` | Explicit Desktop quote-data such as `qsd.rtc` is requested | Separate Desktop-backed source; inspect availability instead of substituting scanner or chart values. |
| `tv quote <SYMBOL> --source auto` | The caller intentionally accepts its documented source choice | Chart-first with scanner fallback only before chart mutation. Report which source was actually used. |

Scanner `extended_hours.premarket` and `extended_hours.postmarket` are separate
from the regular quote. Missing fields may mean an inactive session or missing
provider data. Main-series chart quotes and quote-session phase names do not
establish equivalence with scanner extended-hours prices.

For unavailable quote-data, report `source_availability.unavailable_reason`.
`tv diagnose quote-data <SYMBOL>` is a separate bounded troubleshooting read,
not a blended quote. An unavailable source does not prove that no price exists.

## Official MCP discovery and fields

`tv mcp search` returns `mcp_search.v1` candidates; choose the intended listing
explicitly instead of treating the first candidate as resolved identity.
`tv mcp columns` returns `mcp_columns.v1`: grouped overview without group/search,
detailed entries with them. Preserve markets and variant names when selecting
columns; catalog categories differ from regional scanner markets.

`tv mcp symbol <EXCHANGE:SYMBOL> --columns close,volume` returns `mcp_symbol.v1`
from `tradingview_mcp`. `fields` preserves requested values and JSON types;
`missing_fields` distinguishes absent from null, and zero remains zero.
`fields_status:present` means the requested values are present, not realtime,
complete, comparable or suitable for a particular analysis. Client receipt time
is not market-data time. Missing provider symbol echoes, delay and session
conditions stay unconfirmed; never relabel these results as scanner quotes.
These commands do not replace existing search/quote commands or automatically
fall back to them. Login is explicit and interactive, not an automatic read step.

`tv mcp symbols <EXCHANGE:SYMBOL>... --columns close,volume` uses one official
batch request for up to 50 distinct symbols. Read `mcp_symbols.v1` items in request
order: `returned` may still have missing fields, `missing` is provider-declared,
and `unreported` means neither result nor missing declaration arrived. Never
convert either unavailable state to zeros or infer its cause. `success:true`
and `symbols_status:partial` can occur together. More than 50 symbols or duplicate
input fails locally; the command does not split, retry or fall back automatically.

## Earnings and dividends

`tv events <SYMBOL>` shapes `scanner_fundamentals_rest` fields as `events.v1`;
`tv events compare <SYMBOL>...` returns ordered `events_compare.v1` items.
Use these when event-shaped evidence is useful; use `tv fundamentals` for raw
fundamental field groups. They are not complete event calendars.

Preserve requested/resolved symbols, event types and field availability. Do not
infer timezone, before/after-market timing, confirmation, or publication meaning
when TradingView did not return it. Null or missing event fields mean unknown,
not proof that no event exists.


## Official MCP financial data

Use `tv mcp financials <SYMBOL> --period ttm [--metric revenue,pe]` for a current
snapshot; `financial-history <SYMBOL> --period fq|fy [--from DATE --to DATE]`
for fiscal labels and aligned series; `forecasts <SYMBOL>` for provider consensus;
and `earnings <SYMBOL>... [--from DATE --to DATE]` for explicit-symbol event reads.
Check command help for bounds and availability. These use separate MCP contracts,
not the scanner-backed fundamentals/events source.

Keep returned field names and nulls. Missing currency/unit/as-of values stay
unknown. Fiscal labels do not establish start/end dates or full requested-window
coverage. Analyst recommendations and EPS/revenue forecasts are provider
observations, not actual earnings or a trading decision. Earnings can return
multiple rows per symbol; follow `symbol_results` indices and preserve
`unreported`. Empty results do not prove there were no events. Never silently
retry a failed data call or substitute the existing source.
