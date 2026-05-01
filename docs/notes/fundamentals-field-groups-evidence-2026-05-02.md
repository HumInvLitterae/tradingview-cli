# Fundamentals field groups evidence - 2026-05-02

This note records public-safe field metadata evidence used to add grouped field
selection to `tv fundamentals <SYMBOL>`. It intentionally omits raw scanner
responses, account-local values, cookies, tokens, and local machine paths.

## Summary

`tv fundamentals` reads a single symbol through TradingView scanner REST without
TradingView Desktop or CDP. The first implementation supported explicit
`--field` selection and a curated default set. The field groups added after this
note are convenience bundles around scanner fields that were visible through
`tv scanner metainfo --market america`.

The groups are not a raw financial statement API. They are scanner field bundles
and should be treated as observed data. The CLI does not infer timezone,
before/after-market meaning, financial analysis, or investment recommendations.

## Field groups

The following group names are supported by `tv fundamentals --group`:

- `earnings`: earnings release date/time and publication type fields.
- `valuation`: market capitalization, P/E, and EPS-related fields.
- `dividends`: dividend yield, ex-date, and payment-date fields.
- `financials`: revenue, net income, and revenue forecast fields.

During this slice, scanner metainfo reported the candidate fields for those
groups with safe metadata such as field name and type. Examples include
`earnings_release_next_date`, `earnings_release_next_time`,
`earnings_publication_type_next_fq`, `price_earnings_ttm`,
`price_earnings_forward_fy`, `dividend_ex_date_upcoming`,
`dividend_payment_date_upcoming`, `total_revenue_ttm`, and `net_income_ttm`.

## Boundaries

Use `tv fundamentals <SYMBOL> --group earnings` when the workflow needs a
single-symbol earnings timing read. Use `--field` for precise field-level reads
or to add one field to a group.

Do not treat these groups as complete financial statements, event calendars, or
news feeds. If a future workflow needs richer earnings calendars, statements,
transcripts, or news, add a separate plan with endpoint evidence and a distinct
public surface.

