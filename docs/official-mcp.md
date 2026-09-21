# Official TradingView MCP commands

The development checkout provides a separate `tv mcp` command group. It does
not replace `tv bars`, add a backend switch to it, or change `bars.v1`. One `tv`
binary contains both paths. See the [work record](plans/tradingview-cli-official-mcp-client.md)
for remaining platform/release qualification; these commands are not in v0.31.4.

## Use

```sh
tv mcp login
tv mcp status
tv mcp search Apple --type stock
tv mcp columns --search volume
tv mcp symbol NASDAQ:AAPL --columns close,volume,market_cap_basic
tv mcp bars NASDAQ:AAPL --timeframe 1D --count 20
tv mcp bars NASDAQ:AAPL --timeframe 1W --count 20
tv mcp bars NASDAQ:AAPL --timeframe 1M --count 20
tv mcp logout
```

The [official service](https://www.tradingview.com/mcp/docs) requires an eligible
paid TradingView account. Login opens the default browser for OAuth; the user
chooses the account. If the sign-in redirect fails, try normal TradingView
homepage sign-in in that same browser before repeating login. No browser/Codex
cookies or tokens are imported. Do not paste an authorization URL or token into
logs or shell commands.

macOS uses Keychain. When the OS asks, verify the executable and dedicated
`tradingview-cli.mcp` item, then choose **Always Allow** for later noninteractive
reads. Replacing an unsigned/development executable can require new OS consent;
run `tv mcp login` explicitly to handle it. Windows uses Credential Manager,
with local-machine persistence and one atomic credential blob; oversized records
fail rather than truncate. Windows CI and an owner-reported basic machine check
have passed; this does not establish every command or long-running refresh behavior.
Linux credential operations currently return `credential_store_unavailable`;
there is no plaintext fallback or claim of uniform platform support yet.

`status` reads local presence/expiry only, without contacting TradingView or
proving that a token is still accepted. `logout` deletes only this client's
local record and does not revoke remote authorization. Ordinary bars/status
operations never open a browser or OS permission prompt. Locked/inaccessible
stores and missing/revoked credentials return structured errors with a next action.

Coordination files contain counters and timestamps, never tokens or market data.
They live under `~/Library/Application Support/tradingview-cli/mcp` on macOS,
`%LOCALAPPDATA%\tradingview-cli\mcp` on Windows, and
`$XDG_STATE_HOME/tradingview-cli/mcp` (or `~/.local/state/tradingview-cli/mcp`) on
Linux. Windows uses native APIs to create missing coordination directories with
protected ACLs for the current user, SYSTEM and administrators. Child state
files inherit those permissions. Existing directories and opened state files
are checked without changing their ACLs; untrusted allow entries, unsupported
ACE types and reparse points are rejected. The directory handle stays open for
the operation without delete sharing to prevent replacement of the checked leaf.
No PowerShell process is started for these checks. SDK/wire tracing is suppressed
for MCP commands even when `RUST_LOG=trace` is set.

## Symbol discovery and data

These explicit MCP commands complement existing `tv search`, `tv quote` and
scanner commands. They do not select a different backend for those commands.

```sh
tv mcp search "Example Corp" --type stock
tv mcp columns
tv mcp columns --search volume
tv mcp columns --market stock --group technicals
tv mcp symbol NASDAQ:EXAMPLE --columns close,volume,market_cap_basic
```

`search` returns candidates without choosing a listing or making another request.
The optional `--type` accepts all, stock, etf, bond, forex, index, futures or crypto.
`columns` without a group/search returns a grouped overview; group/search returns
detailed column entries, including provider-supplied markets and variants.
`--market` accepts all, stock, etf, crypto or bond (these are catalog categories,
not the separate screener query's regional markets).

`symbol` requires an exchange-qualified symbol. `--columns` is a comma-separated
list of distinct column names. Without it, the client explicitly requests name,
description, close, change, change_abs, volume, market_cap_basic,
price_earnings_ttm, sector and industry. Column names come from `columns`, not
from guessed aliases. The client bounds lists to 50 columns and each name to
128 bytes; these are local input bounds, not claimed provider limits. Search,
group and filter text is bounded to 256 bytes. Blank/control-character input,
invalid filters and duplicate columns fail before credential or provider I/O.

The shared success envelope still has `command:"mcp"`. Data contracts are:

| Contract | Result | Interpretation |
| --- | --- | --- |
| `mcp_search.v1` | `symbols[]` with symbol, description, type, exchange and logo identifiers | Provider candidates; absent optional fields remain null. `returned_count` is measured; search coverage is unconfirmed. |
| `mcp_columns.v1` | `mode:overview` with `groups[]`, or `mode:details` with `columns[]`; the unused array is empty | Group/column counts are client observations. Provider-reported counts stay separate, and overview counts do not imply complete market coverage. |
| `mcp_symbol.v1` | `fields` keyed by the requested columns | Values retain JSON types. Null and absent fields are identified separately in `missing_fields`; zero remains zero. Extra unrequested fields are not forwarded. |

Each includes the actual request, explicit MCP source, client receipt time and
transport outcome. `symbol` returns `fields_status:present|incomplete`; successful
communication does not guarantee every field is available. Missing symbol echoes
remain `identity_match:unconfirmed`. An explicit mismatching echo fails. Current
responses do not establish data-as-of, delay or session semantics; these stay
null/unconfirmed even when a request explicitly includes similarly named data
columns. The client does not infer units, realtime entitlement or financial
meaning from a column name. `fields_status:present` establishes presence only.

For example, the existing `tv quote NASDAQ:EXAMPLE` retains its existing scanner
contract. The new `tv mcp symbol NASDAQ:EXAMPLE --columns close,volume` instead
uses `mcp_symbol.v1`. If the response has `close:0` and `volume:null`, its data
contains the following excerpt (synthetic values; other envelope fields omitted):

```json
{
  "contract_version": "mcp_symbol.v1",
  "source": "tradingview_mcp",
  "fields": {
    "close": 0,
    "volume": null
  },
  "client_observation": {
    "identity_match": "unconfirmed",
    "fields_status": "incomplete",
    "missing_fields": [
      {"field": "volume", "reason": "null"}
    ]
  }
}
```

If volume is absent rather than null, `fields.volume` remains null and the reason
is `absent`. An empty search returns an empty candidate array, not an invented
symbol or an automatic lookup elsewhere. Invalid provider wrappers, inconsistent
row counts and incompatible input schemas fail with `mcp_error.v1`. Authentication,
429, timeout and invalid-response handling is shared with bars; no read is
silently retried or redirected to the old scanner/market path.

## Multi-symbol data

```sh
tv mcp symbols NASDAQ:AAPL NASDAQ:MSFT --columns close,volume
```

`symbols` calls the official batch tool once for 1..50 distinct,
exchange-qualified symbols. It shares the explicit column defaults and validation
of `symbol`. Duplicate symbols and more than 50 inputs fail before authentication
or network access. There is no automatic splitting, individual-symbol fallback
or retry.

The `mcp_symbols.v1` result contains `items[]` in input order, with zero-based
`requested_index`, `requested_symbol` and a per-symbol status:

| Status | Meaning |
| --- | --- |
| `returned` | The provider identified this requested symbol and returned its field map. The item's fields, missing-field reasons and unknown freshness follow the single-symbol rules. |
| `missing` | The provider explicitly included this symbol in its missing list. `fields` is null, not fabricated zero values. |
| `unreported` | Neither data nor an explicit missing declaration was returned for this requested symbol. `fields` is null; do not infer why. |

`client_observation` counts requested, returned, missing and unreported symbols;
`symbols_status` is `all_returned`, `partial` or `none_returned`. These counts do
not establish field completeness. For example, a returned row with `volume:null`
is still a returned symbol with incomplete fields. Envelope success means a valid
response was interpreted; it does not require every symbol or field to exist.

Before: separate `tv mcp symbol` processes each make their own request. After:
`tv mcp symbols NASDAQ:FIRST NASDAQ:SECOND NASDAQ:THIRD --columns close,volume`
makes one tool call and can report FIRST as returned (with a missing volume),
SECOND as explicitly missing and THIRD as unreported, without changing order.
Unexpected provider symbols, duplicate missing entries, contradictory data
and missing entries, or
malformed rows fail the whole response with `mcp_error.v1`; positions are never
used to guess which requested symbol a row belongs to. Freeform provider missing reasons are not
interpreted as proof that a symbol does not exist or that a particular permission
is missing. Transport/authentication failure also fails the operation rather than fabricating individual missing rows.

## Screener queries

```sh
tv mcp screener --market america --filters '{"close":[10,100]}' --sort-by volume --sort-order desc --limit 20 --columns name,close,volume
tv mcp screener --preset new_highs --types stock --limit 20
tv mcp screener --symbolset 'SYML:SP;SPX' --limit 20
```

This is an independent official MCP query; existing `tv scanner scan` and
Desktop `tv screener` commands retain their sources and behavior. The command
makes one `run_screener` call. Defaults are market america, empty filters, sort
by volume descending, limit 100 and the same explicit columns as `mcp symbol`.
The official maximum is 1000 rows. No offset, automatic pagination, retry or
fallback is added.

`--filters` is a JSON object mapping columns to `[minimum,maximum]`; null means
an unbounded side. The special index, sector, industry and analyst_rating keys
also accept string values. Inverted ranges, malformed JSON, invalid limits and
sort directions fail before credentials or network access. Local input bounds
are 50 filters, 50 column/selection entries and 16 KiB filter JSON. These local
bounds are not claims about additional server capabilities. Use `mcp columns`
for column names; screener regional `--market` names are different from catalog
market categories. `--types` and `--symbolset` are comma-separated lists.

Supported `--preset` names are pullback_with_reversal, breakout_with_volume,
oversold_healthy, earnings_runup, relative_strength and new_highs. Filters,
presets and selections are forwarded as explicit request conditions; the CLI
does not reinterpret a preset as a trading recommendation or assert independent
verification that each provider result satisfies every condition.

`mcp_screener.v1` keeps the request and `items[]` in provider order. Each item
contains its zero-based index, provider symbol, requested `fields` and per-field
absence/null information. Extra unrequested columns are not forwarded. Symbols
are provider observations, not matches to an independently requested listing.
Receipt time remains separate from unknown market-data time, delay and session.

`provider_observation.total_count` preserves the reported `totalCount`, if given.
`client_observation.returned_count` measures returned rows; `coverage_status` is:

- `limited`: the reported total exceeds the returned row count.
- `all_reported`: the returned count equals the provider-reported total. This is
  not independent proof of full-market coverage or a realtime snapshot.
- `unconfirmed`: no usable total was supplied.

For example, three rows and `totalCount:120` produce `returned_count:3` and
`coverage_status:limited`, even though the envelope is successful. Zero rows and
`totalCount:0` are a valid empty result. Missing total stays null/unconfirmed;
contradictory totals, excess rows, duplicate symbols and malformed rows fail with
`mcp_error.v1`. Transport failure does not become an empty successful screen.

## Economic indicators and calendars

These are independent authenticated reads. Existing `tv events`, scanner data
and chart operations keep their existing behavior.

```sh
tv mcp economic-symbols
tv mcp economic-symbols --category prce --search inflation
tv mcp economic-symbols --country US --search inflation
tv mcp economic-data '<SYMBOL_FROM_CATALOG>' --from 2025-01-01 --to 2026-09-21
tv mcp economic-calendar --countries US,JP --min-importance 1 --from 2026-09-17 --to 2026-09-18
tv mcp dividends NASDAQ:AAPL NASDAQ:MSFT
tv mcp dividends --market america --from 2026-09-21 --to 2026-09-25 --limit 20
```

| Command | Contract | Meaning |
| --- | --- | --- |
| `economic-symbols` | `mcp_economic_symbols.v1` | No filters returns `mode:overview`, categories/countries and provider counts. Filters without country return `mode:indicators` with codes/names. A country returns `mode:symbols` with actual qualified symbols. |
| `economic-data` | `mcp_economic_data.v1` | Exact catalog symbol; dated actual values with provider unit/scale. Optional date-only bounds. |
| `economic-calendar` | `mcp_economic_calendar.v1` | Separate actual/forecast/previous and raw values, event dates, periods, units/scale and source fields when provided. |
| `dividends` | `mcp_dividends.v1` | Explicit symbols or market screening; recent/upcoming amounts and ex/payment dates remain distinct. |

Catalog categories are `gdp`, `lbr`, `prce`, `hlth`, `mny`, `trd`, `gov`,
`bsnss`, `cnsm`, `hse`, `txs`, `enrg` and `clmt`.

Never construct a ticker from an indicator code and country. Discover with
`--country` and copy the returned `items[].symbol`. The series must echo the
requested symbol; duplicates or inconsistent counts fail as `invalid_response`.
Rows retain provider order. The client derives the actual date range without
resampling, scaling values, filling gaps or treating it as requested-window
coverage. Unit, scale and notices are source metadata, not client guarantees.

Calendar countries default to `US`; country/currency filters use uppercase
comma-separated two/three-letter codes. `--min-importance` accepts `-1` (all,
default), `0` (medium/high), or `1` (high). Calendar bounds accept dates or
canonical UTC second timestamps such as `2026-09-17T00:00:00Z`. Omitted bounds
remain provider defaults. Category names differ from the indicator catalog;
calendar categories are `all`, `gdp`, `bonds`, `business`, `consumer`,
`goverment`, `health`, `housing`, `labor`, `money`, `prices`, `trade`, and
`taxes`. The provider's exact spelling `goverment` is preserved. Historical and
forward availability remain provider constraints. Event times and reference
periods are retained as reported; no timezone conversion or economic-surprise
judgment is inferred. Unknown status values or malformed rows fail closed.

Dividend symbol mode accepts up to 50 distinct qualified symbols and rejects
market/date/limit options. Market mode requires `--market`, accepts date-only
bounds and limit 1..200 (default 50). It does not fetch additional pages.
`outcomes` in symbol mode follows requested order and distinguishes `returned`
from `unreported`; an omitted symbol is not declared nonexistent. Returned
items keep provider order. A missing next dividend remains `null`, not zero or
proof that no dividend is scheduled. Provider `dividends_yield` is not
recalculated. No exchange calendar or currency conversion is inferred.

All four contracts retain `request`, explicit MCP source, client `received_at`
and `transport` evidence. Completeness and requested-window coverage remain
`unconfirmed`, even for a successful response. These synthetic excerpts omit
unchanged envelope and metadata fields:

```json
{
  "contract_version": "mcp_economic_data.v1",
  "source": "tradingview_mcp",
  "symbol": "ECONOMICS:USEXAMPLE",
  "series": [{"date": "2026-01-01", "value": 0}, {"date": "2026-02-01", "value": null}],
  "provider_metadata": {"unit": "%", "scale": 1},
  "client_observation": {"returned_count": 2, "actual_range": {"from": "2026-01-01", "to": "2026-02-01"}, "requested_window_coverage": "unconfirmed"},
  "transport": {"status": "succeeded", "tool_attempts": 1}
}
```

```json
{
  "contract_version": "mcp_dividends.v1",
  "mode": "symbols",
  "outcomes": [
    {"requested_symbol": "NYSE:OTHER", "status": "unreported"},
    {"requested_symbol": "NASDAQ:EXAMPLE", "status": "returned"}
  ],
  "items": [{"symbol": "NASDAQ:EXAMPLE", "dividend_amount_recent": 0, "dividend_amount_upcoming": null}],
  "client_observation": {"returned_count": 1, "completeness": "unconfirmed"}
}
```

Mixed dividend modes are `invalid_request` with zero tool attempts. Authentication,
rate limit and timeout failures use the shared MCP error contract; no implicit
retry or source fallback occurs. Wrong identity, count contradictions, wrong
field types and response-schema changes fail explicitly without returning raw
provider errors. Optional missing values remain unknown. See the existing work
record for fixture and native qualification evidence.

## News and company documents

These independent official-source commands read one response per invocation.
The placeholders below must be replaced with unchanged IDs from the preceding
list response; they are not IDs to construct from a title, URL or symbol.

```sh
tv mcp news NASDAQ:AAPL --lang en --limit 2 --offset 0
tv mcp news-story '<STORY_ID>' --lang en
tv mcp documents NASDAQ:AAPL --limit 2
tv mcp document '<VIEW_ID>'
tv mcp documents NASDAQ:AAPL --category annual_reports --start-date 2025-01-01T00:00:00Z --end-date 2026-09-21T23:59:59Z
```

| Command | Contract | Selection and bounds |
| --- | --- | --- |
| `news` | `mcp_news.v1` | Qualified symbol, language (default `en`), limit 1..200 (default 25), offset 0..200 |
| `news-story` | `mcp_news_story.v1` | Exact news `items[].id`, language, optional viewer declarations |
| `documents` | `mcp_documents.v1` | Qualified symbol; category/event filters; limit 1..100 (default 20); optional event-window endpoints |
| `document` | `mcp_document.v1` | Exact `items[].views[].id` from documents |

News uses the documented language set, including `en`, `ja`, `zh-Hans` and
`zh-Hant`. Story IDs can be opaque strings as well as URNs: an observed list
returned a non-URN ID that the story tool accepted unchanged. Both detail
commands reject empty/whitespace/control-containing IDs and cap them at 2048
bytes. This syntax check cannot prove the ID came from a list; the caller must
retain the actual source reference. No URL rewriting, ID construction or link
fetching occurs.

`news-story --user-prostatus pro|non_pro` defaults to `non_pro` and is passed as
declared, not inferred from the OAuth grant or paid subscription. Optional
`--user-country` is a two-letter uppercase country code. The CLI never changes
these declarations to recover restricted content. It preserves the provider's
permission/copyright metadata and does not fall back to a publisher URL.

Documents accepts `all`, `annual_reports`, `quarterly_reports`, `interim_reports`,
`company_events`, `insider_transactions`, `transcripts`, `presentations`,
`press_releases`, `other`, and the provider aliases `10-K`, `10-Q`, `8-K`.
`--event` accepts `earning` or `corporate_event`. The limit of 100 is a client
bound, not a claimed provider maximum. This CLI currently accepts the exact UTC
second syntax `YYYY-MM-DDTHH:MM:SSZ` for `--start-date` / `--end-date`, a subset of
the provider's RFC3339 syntax. Both endpoints are inclusive instants; fractional
seconds, other offsets and date-only inputs are rejected before I/O. These are
**event-date** filters; an item's `reported` timestamp is kept separately and
is not used to infer full event-window coverage. No document offset/pagination
is advertised by this CLI.

Every response has `source:tradingview_mcp`, `source_category:desktop_free_read`,
`requires_desktop:false`, the exact request and separate client receipt time.
`client_observation.completeness` stays `unconfirmed`. Transport success is not
proof of full history, full text, access rights or a stable multi-page snapshot.

News `items` retains ID, title, publication timestamp, provider identity,
urgency, story path, link, permission, paywall and related symbols. `pagination`
retains the reported offset, next offset, `has_more` and `total_available`, using
null for absent values. To continue, pass the returned next offset explicitly;
there is no automatic page loop. The provider exposes at most its recent pool
(documented as up to 200 headlines), not unrestricted historical access. Counts
and request limits are checked, duplicate IDs within a page are rejected, and
an advancing page cannot point back to the same offset.

For example, a synthetic first page with one restricted headline and more
available items retains both the usable reference and the restriction:

```json
{
  "contract_version": "mcp_news.v1",
  "items": [{"id": "opaque-example-id", "permission": "restricted", "paywall": true}],
  "pagination": {"offset": 0, "next_offset": 1, "has_more": true, "total_available": 5},
  "client_observation": {"returned_count": 1, "completeness": "unconfirmed"}
}
```

Documents keeps document IDs, provider/category/form, event/fiscal metadata,
reported timestamp, symbols, and exact view IDs/types. `provider_total` is kept
as reported and does not promise that increasing the limit retrieves everything.
An empty list is an empty observation, not proof that no documents exist.

Both detail contracts return `content` and `provider_metadata`. News keeps the
string fields `ast_description`, `short_description` and `summary`. Document
views keep the structured `astDescription` object. These are inert provider
content: no AST execution, HTML rendering, embedded instructions or reference
fetching. Preserve attribution and access metadata alongside saved results.
`content_status:returned` means content was supplied, not that full-text access
or completeness was established. A metadata-only/restricted response uses
`not_returned` and null content rather than a fabricated empty article.
`id_echo_matches` is true for a matching supplied response ID, or null when no
ID is reported. A contradictory response ID fails with `reference_id_mismatch`;
the client does not reinterpret it as an alias or return a different body.

Invalid bounds, dates, filters or IDs fail before credentials/network access.
A 401, 429, timeout or malformed response uses the shared `mcp_error.v1` contract;
data calls are never automatically replayed. Schema drift and contradictory
pagination/counts fail explicitly. The client does not attempt broader viewer
settings, scraping, another source or arbitrary MCP tool forwarding on failure.

Native macOS verification covered two news pages, the empty page at offset 200,
a news story, document listing, a document view and a dated annual-report query.
Both detail responses echoed the exact requested IDs. Restricted/metadata-only
responses and malformed cases are fixture evidence, not live paywall bypass
checks. Windows qualification remains separate and pending.

## Financial data, forecasts and earnings

These commands read the official MCP source independently of scanner-backed
`tv fundamentals` / `tv events`; neither existing command is redirected.

```sh
tv mcp financials NASDAQ:EXAMPLE --period ttm
tv mcp financials NASDAQ:EXAMPLE --period fq --metric revenue,pe
tv mcp financial-history NASDAQ:EXAMPLE --period fq --from 2025-01-01 --to 2026-09-21
tv mcp forecasts NASDAQ:EXAMPLE
tv mcp earnings NASDAQ:EXAMPLE NYSE:OTHER --from 2026-07-01 --to 2026-12-31
```

| Command | Contract | Selection |
| --- | --- | --- |
| `financials` | `mcp_financials.v1` | `--period fy\|fq\|ttm\|fh\|current`, default `ttm`; optional `--metric` aliases or raw column names |
| `financial-history` | `mcp_financial_history.v1` | `--period fq\|fy`, default `fq`; optional `--from` / `--to` |
| `forecasts` | `mcp_forecasts.v1` | One exchange-qualified symbol |
| `earnings` | `mcp_earnings.v1` | 1..50 distinct exchange-qualified symbols; optional `--from` / `--to` |

Dates must be real `YYYY-MM-DD` dates in ascending order. Either endpoint may
be omitted, leaving its default to the provider. The CLI caps metric lists and
earnings symbol lists at 50; these are client bounds, not claims about server
capacity. Metric names accept letters, digits, underscores, dots, hyphens and
period separators (`|`), at most 128 bytes; duplicates are rejected. Omit
`--metric` to ask for the provider's full snapshot. Aliases and period-specific
column rewriting belong to the provider; inspect returned field names rather
than assuming a requested alias proves a particular calculation.

All four contracts retain the request, `source:tradingview_mcp`,
`source_category:desktop_free_read`, `requires_desktop:false`, and
`transport:{status:succeeded,tool_attempts:1}`. `provider_metadata` retains
available symbol/name, reporting period, currency, unit, scale, as-of and window
values. Missing metadata is null. `client_observation.received_at` is the local
receipt time, not a filing/publication/market-data timestamp. Currency is not
inferred from a listing exchange; numeric units and scale are not converted.
Conflicting symbol/period metadata, incompatible known field types, or an
explicit provider failure produce a structured error.

Financial snapshots keep provider column names and scalar JSON values in
`fields`. For example, a synthetic response to a quarterly revenue request can
contain `fields:{total_revenue_fq:null,net_income_fq:0}`: revenue is unknown and
net income is zero. The two remain distinct. `null_fields` and
`returned_field_count` describe the returned object; they do not establish
whether aliases were fully satisfied. `metric_selection_status` stays
`unconfirmed`. In native observation the snapshot returned a ticker name but no
qualified symbol or currency, so `symbol_status` and currency stayed unknown.

History keeps `labels` in provider order and aligns every `series` array to
those labels. Each point retains `value` and `yoy_pct`; null is not zero.
Different array lengths fail with `invalid_response` instead of pairing values
with the wrong period. `capex_latest` is a separate latest-value observation,
not an extra historical row. Returned labels are not converted into fiscal
start/end dates. For example, a request spanning two years may return six
quarterly labels: `returned_period_count:6` is observed, while
`requested_window_coverage:unconfirmed` prevents claiming eight complete quarters.

Forecasts preserve `analyst_rating`, `price_targets`, `estimates` and the current
`price` under `forecast`. A provider recommendation is the provider's opinion,
not a CLI-generated decision. Names such as `eps_next_quarter` and `eps_ttm`
retain their distinct meanings; estimates are not relabelled as actual results.
Missing groups/fields remain null where the supported schema permits them.
No return, surprise or upside calculation is invented by the client.

Earnings preserves event rows in `items`, including provider release-date strings
and next-quarter forecast fields. Multiple rows for one symbol are retained;
`symbol_results` follows requested symbol order and points to matching row
indices. For example, requesting two symbols with only the first returned yields
this abbreviated data (synthetic symbols and values):

```json
{
  "contract_version": "mcp_earnings.v1",
  "request": {"symbols": ["NASDAQ:EXAMPLE", "NYSE:OTHER"]},
  "items": [{"symbol": "NASDAQ:EXAMPLE", "release_date": "2026-08-01", "eps_forecast_next_fq": null}],
  "symbol_results": [
    {"requested_symbol": "NASDAQ:EXAMPLE", "status": "returned", "item_indices": [0]},
    {"requested_symbol": "NYSE:OTHER", "status": "unreported", "item_indices": []}
  ],
  "client_observation": {"returned_count": 1, "completeness": "unconfirmed", "requested_window_coverage": "unconfirmed"}
}
```

An empty list leaves every symbol `unreported`; it does not prove no earnings
occurred. Release-date strings, provider `from`/`to` echoes and request dates are
separate from actual window coverage. No timezone, session, confirmed/estimated
event classification or calendar completeness is inferred. Unexpected symbols
or a contradictory returned count fail before producing a success contract.

A `financial-history --period ttm` request or an invalid date is rejected before
credentials/network access with `code:invalid_request` and `tool_attempts:0`.
Authentication, rate limits, timeouts and response errors use the shared
`mcp_error.v1` contract. A 401 may refresh credentials for the next explicit
invocation but never repeats the data call. A 429 retains the shared cooldown;
a timeout is `deadline_exceeded`; malformed/unaligned data is `invalid_response`.
No fallback or missing-value substitution follows these errors.

Native macOS checks covered a full TTM snapshot, an FQ metric subset, quarterly
and annual history, forecasts, a two-symbol earnings query and an empty past
window through the public service. They establish response handling for those
cases, not general completeness, every metric/market/period, or Windows runtime
qualification. Synthetic JSON/SSE and failure fixtures cover the remaining
contract branches. See the [work record](plans/tradingview-cli-official-mcp-client.md).

## Watchlists and alerts

The independent account reads use the same explicit login and credentials:

```sh
tv mcp watchlist list
tv mcp watchlist get 12
tv mcp alert list --symbol NASDAQ:EXAMPLE --active false
tv mcp alert get 12 10
```

IDs in these examples are synthetic. Use IDs returned by the corresponding
list command. Watchlist IDs are canonical unsigned decimal strings; alert IDs
are positive integers. Alert detail reads accept 1..100 distinct IDs per call
(a client bound), without automatic splitting or replay.

| Command | Contract | Result |
| --- | --- | --- |
| `watchlist list` | `mcp_watchlists.v1` | `items` in provider order |
| `watchlist get <ID>` | `mcp_watchlist.v1` | `watchlist` with matching ID |
| `alert list` | `mcp_alerts.v1` | `items` in provider order; optional symbol/active filters |
| `alert get <ID>...` | `mcp_alert_details.v1` | `items` in requested ID order, each with `requested_id/status/alert` |

All use `source:tradingview_mcp`, `source_category:desktop_free_read` and
`requires_desktop:false`. They do not activate a watchlist or change an alert.
The official `get_active_watchlist` operation can activate/create a list and
is deliberately absent from the read path. Desktop watchlist and alert commands
remain available separately.

Watchlist snapshots retain IDs, names, optional description/type/active/shared
metadata, provider timestamps and the original ordered `symbols` strings.
Section labels are preserved; they are not resolved as instruments or flattened
into a deduplicated symbol set. Missing symbols or optional fields remain null.

Alert snapshots retain IDs, optional symbol/name/active state, condition type,
threshold, resolution and provider timestamp strings. Detail reads additionally
retain available auto-deactivation/webhook-presence flags and a supported
`conditions` projection: type, frequency, resolution, cross-interval flag, and
series type/numeric value. Missing fields remain null. No message, webhook URL,
notification address or arbitrary raw payload is exposed. This projection is
not a complete Pine/complex-condition serialization: `condition_completeness`
stays `unconfirmed`, and it must not be used alone to recreate an alert.

For example, a detail request for `[12,10]` with only alert 12 reported yields
the following selected fields (other metadata omitted here):

```json
{
  "items": [
    {"requested_id": 12, "status": "returned", "alert": {"alert_id": 12}},
    {"requested_id": 10, "status": "unreported", "alert": null}
  ],
  "client_observation": {
    "returned_count": 1,
    "unreported_count": 1,
    "ids_status": "partial",
    "completeness": "unconfirmed"
  }
}
```

An empty list is a successful observation with zero returned items, not proof
of account-wide completeness. An omitted ID is unreported, not asserted deleted.
Duplicate/unexpected IDs, a mismatched watchlist ID, contradictory filter echoes
or malformed fields produce `mcp_error.v1`; failed calls never become empty
successful results. Existing authentication/rate-limit/timeout handling applies.
`received_at` is the client receipt time, separate from provider timestamps.
The commands in this section are reads; explicit watchlist changes are described
below, followed by explicit alert changes.

## Explicit watchlist changes

The development CLI also provides these explicit changes. A disposable-list
lifecycle passed native macOS verification through the public service.
Use account-local IDs obtained from `watchlist list`.

```sh
tv mcp watchlist create "Example research" --symbols NASDAQ:EXAMPLE,NYSE:OTHER
tv mcp watchlist update 12 --name "Renamed research"
tv mcp watchlist update 12 --description ""
tv mcp watchlist add 12 NASDAQ:EXAMPLE
tv mcp watchlist remove 12 NYSE:OTHER
tv mcp watchlist delete 12
```

The IDs and symbols above are synthetic. Creation requires an explicit nonblank
name (up to 500 characters). Update requires a name and/or description; omission
keeps a field unchanged, while an empty description requests clearing it.
Descriptions are bounded to 4096 characters by the client. Create accepts up to
100 distinct qualified symbols; add/remove require 1..100. No implicit chunking
occurs. Adding an existing symbol moves it to the end; it is not a no-op.
Deletion names one target and cannot be inferred from a read request.

Each invocation sends at most one mutation and, after a valid response, one
readback using the same credential/admission service and overall deadline.
No mutation is replayed after timeout, authentication failure or response error.
OAuth renewal before dispatch retains the existing behavior; a rejected mutation
does not trigger the read-command refresh-and-repeat path. Scopes are never
expanded automatically. The public OAuth metadata advertises `mcp:read` and
`mcp:tools`, but the watchlist tool definitions do not declare per-tool scopes;
the disposable-list lifecycle succeeded under the existing `mcp:read` grant.
That scope name is not a token-level guarantee against account writes. No
additional scope was needed for the observed workflow.

Results use `mcp_watchlist_mutation.v1`,
`source_category:desktop_free_mutation` and `requires_desktop:false`.
The outer success envelope means a valid mutation response was received.
It does **not** assert that the requested state was confirmed; consumers must
inspect `readback.status`. A successful response with failed readback is not
a reason to repeat the mutation.

Selected fields from a confirmed rename:

```json
{
  "contract_version": "mcp_watchlist_mutation.v1",
  "operation": "update",
  "target_id": "12",
  "mutation": {
    "status": "response_received",
    "tool_attempts": 1,
    "automatic_retry": false
  },
  "readback": {"status": "matched", "tool_attempts": 1}
}
```

| Readback status | Meaning |
| --- | --- |
| `matched` | Requested name/description/symbol postconditions match the ID-specific observation. |
| `mismatch` | A reported field contradicts the requested postcondition. |
| `unconfirmed` | Required observation fields are absent/null. |
| `not_reported` | After deletion, the target is absent from the returned list; account-wide list completeness remains unconfirmed. |
| `still_present` | The deleted target is still reported by the list read. |
| `not_performed` | Creation returned no usable target ID; the client does not guess by name. |
| `failed` | Readback failed; its structured error is retained separately from the received mutation response. |

Ordinary readback includes the normalized target snapshot. Deletion reports
only membership/timestamp/completeness evidence, not unrelated account lists.
A missing description stays unknown, including after a requested update.

A mutation failure uses `mcp_error.v1` with `details.mutation.status` equal to
`not_attempted` or `outcome_unknown`; validation/local preflight errors retain
zero tool attempts. HTTP/MCP rejection does not prove that no change occurred.
For example, a timeout after attempted dispatch has
`code:deadline_exceeded`, `mutation.status:outcome_unknown`, and
`automatic_retry:false`. Inspect the target before considering a new mutation.
For an uncertain create without an ID, inspect the list and resolve ownership;
do not create another list or pick a same-named list automatically.

## Explicit alert changes

`tv mcp alert` provides simple price-alert creation, settings updates and
explicit lifecycle operations. The existing Desktop commands remain separate.
These synthetic examples change the account when run with real symbols/IDs:

```sh
tv mcp alert create NASDAQ:EXAMPLE --price 100 --condition greater --resolution 1D --name "Example threshold"
tv mcp alert update 12 --name "Renamed threshold" --email false
tv mcp alert stop 12 13
tv mcp alert restart 12
tv mcp alert delete 12
```

Create requires a qualified symbol, finite price and a nonblank name of at most
300 characters. Supported conditions are `cross` (default), `cross_up`,
`cross_down`, `greater` and `less`. `--resolution` uses official chart strings:
`1` (default), `5`, `15`, `30`, `60`, `240`, `1D`, `1W`, `1M`.
Creation explicitly sends `email:false`, `mobile_push:false`, `popup:false`,
`auto_deactivate:false` and `monitor:false`. Override the first four using
`--email true|false`, `--mobile-push true|false`, `--popup true|false`, and
`--auto-deactivate true|false`. Notifications default off even where the official
API defaults on. Creation still creates an active alert and consumes account
capacity. Expiration uses the provider default; its observed value is retained
by `alert get`.

Update accepts `--name` and those four boolean settings; at least one is required.
Omitted settings stay unchanged, and explicit `false` is transmitted.
**Updating reactivates the alert**, including a name-only change to a stopped
alert. It cannot change symbol, condition, price or resolution. Stop preserves
settings and history; restart activates the existing conditions and notification
settings. Delete also deletes fire history. Lifecycle commands accept 1..100
distinct positive integer IDs, and never imply all alerts. Obtain IDs from
explicit reads; do not guess them.

This slice omits message/webhook editing, expiration editing, server-side
monitoring, fire-log reads and Pine condition creation/reconstruction. Missing
notification values in reads remain null, not false. No new dependency,
credential store or implicit Desktop fallback is introduced.

The data contract is `mcp_alert_mutation.v1`, with
`source:tradingview_mcp`, `source_category:desktop_free_mutation` and
`requires_desktop:false`. Like watchlist changes, one mutation is followed by
one readback. `mutation.status:response_received` is separate from verified
postconditions. For example, before `stop 12 13` both IDs may be active. A valid
reply followed by ID 12 stopped and ID 13 omitted produces this abbreviated data:

```json
{
  "contract_version": "mcp_alert_mutation.v1",
  "operation": "stop",
  "request": {"alert_ids": [12, 13]},
  "target_ids": [12, 13],
  "effects": {"reactivates": false, "deletes_fire_history": false},
  "mutation": {"status": "response_received", "tool_attempts": 1, "automatic_retry": false},
  "readback": {
    "status": "unconfirmed",
    "tool_attempts": 1,
    "items": [
      {"requested_id": 12, "status": "matched", "alert": {"alert_id": 12, "active": false}},
      {"requested_id": 13, "status": "unreported", "alert": null}
    ]
  }
}
```

For creation, `matched` requires observed symbol, resolution, condition, price,
name, active state and requested notification settings to match; absent evidence
is `unconfirmed`. Updates compare requested settings and active state. Stop and
restart compare active state for every requested ID. A known contradiction is
`mismatch`; a missing row is `unreported`. Batch aggregate is `matched` only when
all targets match; any known contradiction yields `mismatch`, otherwise missing
or unknown evidence yields `unconfirmed`. Matching covers the checked fields,
not lossless alert/Pine reconstruction or notification delivery.

Delete reads the alert list and reports each target as `still_present` or
`not_reported`; absence alone does not establish complete deletion proof.
Unrelated account rows are excluded. Creation without a returned ID uses
`readback.status:not_performed`, never a guessed identity. Readback failure uses
`readback.status:failed` with its structured error; the received mutation reply
is retained. For example, a readback 429 is not a reason to create another alert.
Mutation errors retain `not_attempted` or `outcome_unknown`; an authentication
rejection, timeout or malformed response after dispatch never triggers automatic
refresh-and-replay. Inspect the target before another explicit change.

Input schemas for all five management tools were accepted against the official
catalog without dispatching mutations. Synthetic JSON/SSE and failure fixtures
cover lifecycle behavior. A native macOS disposable-alert lifecycle also passed:
create, stop, rename/reactivate, stop and restart matched readback; after delete,
the list no longer reported the disposable ID. This does not prove notification
delivery, every supported condition, batch mutations or Windows execution.
Do not infer write permission from the OAuth scope name or catalog annotations.

## Read contract

Use an exchange-qualified symbol and one of `1m`, `5m`, `15m`, `30m`, `1h`,
`4h`, `1D`, `1W` or `1M`, with count 1..5000 (default 300). For example:

```sh
tv mcp bars NASDAQ:AAPL --timeframe 5m --count 20
```

`1m` means one minute; `1M` means one month and maps to provider interval `M`.
No local resampling occurs. Numeric interval aliases and `2h` are unsupported.
Date ranges, Desktop targets, batch bars requests and automatic fallback are
unsupported. Invalid requests fail before credential/provider I/O. Intraday
count satisfaction does not establish calendar completeness or regular spacing;
session boundaries, delay, timestamp anchoring and finality remain unconfirmed.

The normal JSON envelope has `command:"mcp"`. Successful reads contain
`data.contract_version:"mcp_bars.v1"`, `source:"tradingview_mcp"`, the request,
provider observations, client observations, transport evidence and normalized
`bars` with `time/open/high/low/close/volume/finality`.

- `success:true` means the tool result was valid, not that history is complete.
  `client_observation.count_status` distinguishes `met`, `short` and `empty`.
- Symbol/interval echoes are checked when present. A mismatch fails; an absent
  echo remains unconfirmed. An echo is not independent listing verification.
- Receipt time is client-measured UTC. Range, count and order are derived from
  returned rows. They do not establish calendar/session coverage or bar finality.
- Missing/null volume stays null with an entry in `missing_fields`; real zero
  volume stays zero. Missing/malformed OHLC or time fails the operation.
- Delay, adjustment, session, market-data as-of time and timestamp anchoring stay
  null/unconfirmed until a supported provider field establishes their meaning.
  Freeform `notice` and summary text are not converted into guarantees.
- The provider's wrapper failure flag, inconsistent count, schema drift,
  descending/duplicate timestamps and invalid high/low bounds fail closed.

Full synthetic before/after examples and the downstream handoff are in the
[contract examples](plans/tradingview-cli-official-mcp-client.md#accepted-cli-and-json-contract).
The downstream must add an explicit MCP reader and isolate caches by provider;
it must not feed this output through a `bars.v1` parser or invent missing evidence.

## Errors and retry

Failures write an error envelope to stderr, leave stdout empty and return a
nonzero exit code. `error.details.contract_version:"mcp_error.v1"` identifies the
MCP-specific code, stage and dispatched tool count. No raw OAuth/server error is
copied into the envelope. General clap syntax errors retain the existing CLI
parser envelope.

Credential failures keep `code:"credential_store_unavailable"` and may include a
closed `reason` identifying worker spawn/input/output/exit, invalid worker reply,
invalid stored record, or native store read/write. These reasons contain no raw
OS error text, credentials, paths or IPC bytes. They identify the failed boundary,
not an instruction to retry or delete credentials. `status` proves only its own
local read; a subsequent command starts a separate credential operation.

Windows state-security errors keep `code:"local_state_unavailable"` and add a
closed `reason`, such as `state_acl_rejected`, `state_owner_rejected` or
`state_security_query_failed`. Native API failures also include `win32_error`
as a number; policy rejections omit it. No path, SID or native message is returned.
For example, an OS security-query failure may return:

```json
{
  "contract_version": "mcp_error.v1",
  "source": "tradingview_mcp",
  "code": "local_state_unavailable",
  "stage": "local_admission",
  "reason": "state_security_query_failed",
  "win32_error": 5,
  "tool_attempts": 0,
  "automatic_retry": false
}
```

An ACL rejection requires reviewing the existing directory/file permissions;
it is not authorization to delete state files or broaden access. The CLI does
not rewrite existing ACLs or substitute another state location automatically.

Within one command's admission lock, validated credentials are reused in memory.
A successful durable update replaces that snapshot; a failed update invalidates
it. A new command reads the OS store again. There is no cross-process token cache.

Reads have one 30-second deadline spanning lock wait, credential work, discovery,
pacing and response. Login has a five-minute interaction deadline. Processes
share an operation lock and persisted one-second spacing/cooldown. Server limits
and Retry-After take precedence. A missing Retry-After stays unconfirmed in the
contract; a conservative 60-second local cooldown is a client policy, not a
claimed provider reset. There is no arbitrary aggregate proof-count limit.

An expired token may refresh before a tool call. If a tool request gets 401,
credentials may refresh for the next explicit invocation, but that request is
not replayed. `auth_refreshed_retry_required` asks the caller to repeat the
explicit read. There is no automatic fallback, session reinitialization, SSE
reconnect or retry of a rejected/failed tool call.
