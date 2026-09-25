# Screening and comparison

## Fetch only the needed evidence

For unknown candidates, prefer `tv mcp screener` when authenticated and it meets
the requested coverage. For the existing scanner source use `tv scanner scan`
or the requested hotlist. Record
filters, columns, sort, result count, and source. Consult `tv scanner metainfo
--field <FIELD>` when a field is unclear. For saved/visible Desktop screens,
use the optional `screener-workflow` skill.

Use `--max-results <N> --page-size <N>` only when one scanner page is
insufficient. The per-request cap is 100 rows. Aggregation deduplicates in
first-seen order and reports page, total-count, duplicate, timing, and drift
metadata. It is sequential observation, not an atomic snapshot. A failed page
returns an error, not a successful partial aggregate. Use `--offset` only for
one diagnostic page, not together with aggregate mode.

`tv spec scanner scan` describes the three modes and supported fields offline;
use help on older binaries. Normal `--limit` defaults to 20, rejects zero and
clamps above 100. Aggregate `--max-results` is a population ceiling, not a top-N
request: a larger reported population fails. It accepts 1–10000, page size
1–100 (default 100), and at most 100 planned pages. Missing totals or incomplete
pages fail instead of producing a successful partial population. Unchanged total
counts do not prove unchanged membership or ordering.

Column/sort fields must belong to the CLI allowlist even if metainfo knows more.
Min/max filters use provider greater/less operators, not guaranteed inclusive
thresholds. RSI inputs accept 0–100 and recommendations -1–1; invalid ordering
between min and max is not checked locally. Review the returned filters.

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
is not an alias. Hints neither execute reads nor rank candidates. Existing hints can label
chart_quote and screenshot `non_mutating` even though the command can switch a
chart symbol or write a file. Consult the command's own spec/help and obtain the
needed intent; that flag is not a complete side-effect declaration.

Stay with the known set for a requested bounded watch. Move to
the optional `chart-analysis` skill only when chart-specific evidence
is needed. `tv chart compare` serially uses the selected Desktop chart and may
temporarily switch it; it is not a broad scanner comparison loop. Watchlist
writes, including `tv watchlist add-bulk`, require intent to change that saved
state. Do not add them as a routine final step of analysis.


## Official MCP screens

For official-source screening, `tv mcp screener` performs one query
and returns `mcp_screener.v1`, independent of scanner REST and Desktop Screener.
Use `tv mcp columns` to find field names. Supply numeric filters as a JSON object
of `[min,max]` bounds (null is unbounded, min must not exceed max). The fields
index, sector, industry and analyst_rating also accept string values. Optional
presets are provider-defined selection rules, not trading recommendations. Never treat a preset label as an
independent assessment of the returned companies.

Keep provider row order, requested fields and their missing/null distinctions.
Report `client_observation.returned_count` alongside the provider's `total_count`.
`coverage_status:limited` means the reported total exceeds returned rows;
`all_reported` only means those counts agree; missing totals remain unconfirmed.
Neither status proves exhaustive market coverage or realtime data. Empty success
is distinct from a failed request. No pagination, retry or source fallback is
performed; the maximum per request is 1000 rows.


## Snapshot and comparison completeness

Use `tv spec snapshot` or `tv spec compare` for offline details when supported.
One successful section is enough for a symbol packet to succeed. For compare,
resolved_count counts such packets, not symbols with all evidence available.
Complete coverage does not audit every quote/info field: tracked missing-field
counts primarily cover fundamentals. Inspect the sections and freshness fields.
Top-level symbol selection prefers quote, then fundamentals, then info; it does
not establish cross-section identity agreement or a synchronized observation.
Snapshot group/field options affect fundamentals only; compare uses defaults.


`tv spec mcp screener` lists supported preset names and input bounds offline;
use help on older binaries. Filters allow at most 50 fields and 16384 UTF-8 bytes.
Columns default when omitted and reject duplicates. `--types` and `--symbolset`
accept up to 50 unique nonblank strings each; local validation does not establish
provider support for those values. The market argument uses regional names, not
the category catalog used by `mcp columns`. Do not assume scanner REST filter
semantics apply to this official path. OAuth refresh can update local credentials
even though the provider operation is read-only.
