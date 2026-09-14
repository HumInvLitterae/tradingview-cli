# Screening and comparison

## Fetch only the needed evidence

For unknown candidates use `tv scanner scan` or the requested hotlist. Record
filters, columns, sort, result count, and source. Consult `tv scanner metainfo
--field <FIELD>` when a field is unclear. For saved/visible Desktop screens,
use [screener-workflow](../../screener-workflow/SKILL.md).

Use `--max-results <N> --page-size <N>` only when one scanner page is
insufficient. The per-request cap is 100 rows. Aggregation deduplicates in
first-seen order and reports page, total-count, duplicate, timing, and drift
metadata. It is sequential observation, not an atomic snapshot. A failed page
returns an error, not a successful partial aggregate. Use `--offset` only for
one diagnostic page, not together with aggregate mode.

For known symbols, choose quote-only `tv quotes` or richer `tv compare`.
`tv snapshot` supplies the same sort of detail for one symbol. Quotes, compare,
and events compare accept at most 25 symbols and preserve input order.
`requested_index` is a zero-based position, not a score.

## Interpret what matched

Use the returned fields to explain the user's criteria: valuation, quality,
growth, momentum, liquidity/session behavior, or sector/industry concentration
as relevant. Show the basis of any user-requested comparison or ordering and
its missing fields. Do not add an unrequested scoring model or promote research
candidates into buy/sell recommendations.

For packets, inspect `summary` for resolution, section success, and field
coverage, then inspect the underlying `sections` or `items[]`. Coverage is not
company quality. Section errors and `missing_evidence[]` show gaps.
For regular-session movement, use
`items[].movement.regular_change_percent`, checking
`items[].sections.quote.data.change` when needed. Leave a null
`regular_change_abs` unknown rather than deriving it from unrelated price fields.

## Decide the next read

`follow_up_hints[]` identify possible evidence surfaces (`snapshot`,
`chart_quote`, `observe_chart`, `screenshot`). Check `requires_desktop`,
`source_category`, `non_mutating`, `evidence_role`, and `auto_execute` before
choosing a relevant follow-up. `chart_quote` is the stable kind; `quote_chart`
is not an alias. Hints neither execute reads nor rank candidates.

Stay with the known set for a requested bounded watch. Move to
[chart-analysis](../../chart-analysis/SKILL.md) only when chart-specific evidence
is needed. `tv chart compare` serially uses the selected Desktop chart and may
temporarily switch it; it is not a broad scanner comparison loop. Watchlist
writes, including `tv watchlist add-bulk`, require intent to change that saved
state. Do not add them as a routine final step of analysis.
