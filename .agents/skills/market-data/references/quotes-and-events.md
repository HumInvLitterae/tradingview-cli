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

## Earnings and dividends

`tv events <SYMBOL>` shapes `scanner_fundamentals_rest` fields as `events.v1`;
`tv events compare <SYMBOL>...` returns ordered `events_compare.v1` items.
Use these when event-shaped evidence is useful; use `tv fundamentals` for raw
fundamental field groups. They are not complete event calendars.

Preserve requested/resolved symbols, event types and field availability. Do not
infer timezone, before/after-market timing, confirmation, or publication meaning
when TradingView did not return it. Null or missing event fields mean unknown,
not proof that no event exists.
