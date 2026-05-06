# Fundamentals and events field evidence - 2026-05-06

This note records public-safe scanner field metadata evidence for fundamentals,
earnings, dividend, and event-like reads. It intentionally omits raw scanner
responses, live response bodies, account-local values, cookies, tokens, and
machine-local paths.

## Summary

`tv fundamentals <SYMBOL>` remains a Desktop-free scanner REST read. Existing
field groups still cover the strongest practical needs:

- `earnings`: earnings release date/time and publication type fields.
- `dividends`: dividend yield, ex-date, and payment-date fields.
- `valuation`: market capitalization, P/E, and EPS-related fields.
- `financials`: revenue, net income, and revenue forecast fields.

The v0.7 evidence pass found additional scanner fields around earnings and
dividends, but not a distinct complete event calendar or news surface. The
recommended next implementation, if needed, is to add small missing fields to
existing `earnings` or `dividends` groups rather than creating `tv events`.

## Confirmed existing group fields

Scanner metainfo confirmed the current earnings fields used by
`tv fundamentals --group earnings`:

| Field | Type | Notes |
| --- | --- | --- |
| `earnings_release_next_date` | `time` | Existing earnings group |
| `earnings_release_date` | `time` | Existing earnings group |
| `earnings_release_next_time` | `number` | Existing earnings group |
| `earnings_release_next_calendar_date` | `time` | Existing earnings group |
| `earnings_release_calendar_date` | `time` | Existing earnings group |
| `earnings_release_next_trading_date_fy` | `time` | Existing earnings group |
| `earnings_release_trading_date_fy` | `time` | Existing earnings group |
| `earnings_publication_type_next_fq` | `number` | Existing earnings group |

Scanner metainfo confirmed the current dividend fields used by
`tv fundamentals --group dividends`:

| Field | Type | Notes |
| --- | --- | --- |
| `dividend_yield_recent` | `number` | Existing dividends group |
| `dividends_yield_current` | `percent` | Existing dividends group |
| `dividend_ex_date_recent` | `time` | Existing dividends group |
| `dividend_ex_date_upcoming` | `time` | Existing dividends group |
| `dividend_payment_date_recent` | `time` | Existing dividends group |
| `dividend_payment_date_upcoming` | `time` | Existing dividends group |

## Additional candidates

Additional scanner fields were visible and may be useful later:

| Field | Type | Candidate use |
| --- | --- | --- |
| `earnings_release_next_trading_date_fq` | `time` | Add to `earnings` if fiscal-quarter trading date matters |
| `earnings_release_trading_date_fq` | `time` | Add to `earnings` if fiscal-quarter trading date matters |
| `earnings_release_time` | `number` | Add to `earnings` only if meaning can be documented safely |
| `earnings_publication_type_fq` | `number` | Add to `earnings` only if current-quarter publication type is needed |
| `dividend_amount_recent` | `fundamental_price` | Add to `dividends` if dividend amount is useful |
| `dividend_amount_upcoming` | `fundamental_price` | Add to `dividends` if upcoming amount is useful |
| `dividend_frequency_recent` | `text` | Add to `dividends` if frequency is useful |
| `dividend_frequency_upcoming` | `text` | Add to `dividends` if frequency is useful |
| `next_dividend_date` | `time` | Add to `dividends` if a compact next-date field is useful |
| `expected_annual_dividends` | `number` | Add to `dividends` if annual dividend expectation is useful |

The metainfo keyword pass also surfaced broad price/update fields such as
`time`, `update_mode`, `premarket_time`, `postmarket_time`, and range-date
fields. Those are not fundamentals event fields and should not be grouped into
`tv fundamentals` without a separate use case.

## Boundary

Do not treat these scanner fields as a complete TradingView event calendar,
news feed, transcript source, or financial statement API. The CLI returns raw
scanner values and does not infer timezone, before/after-market meaning,
publication-code meaning, or investment significance.

For current workflows, use:

- `tv fundamentals <SYMBOL> --group earnings` for single-symbol earnings
  timing fields;
- `tv fundamentals <SYMBOL> --group dividends` for dividend yield/date fields;
- `tv scanner metainfo --market america --field <FIELD>` to confirm a field
  exists before adding new groups or columns.

Future implementation should start with small group additions backed by
scanner metainfo evidence. A standalone `tv events` command remains deferred.
