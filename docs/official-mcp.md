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
fail rather than truncate. Its native runtime acceptance remains a release gate.
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
Linux. Windows validates the directory's owner/access rules and rejects reparse
points; it does not weaken an existing ACL. SDK/wire tracing is suppressed for
MCP commands even when `RUST_LOG=trace` is set.

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
Commands added here are reads; account management is a subsequent slice.

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
