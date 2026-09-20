# Official TradingView MCP commands

The development checkout provides a separate `tv mcp` command group. It does
not replace `tv bars`, add a backend switch to it, or change `bars.v1`. One `tv`
binary contains both paths. See the [work record](plans/tradingview-cli-official-mcp-client.md)
for remaining platform/release qualification; these commands are not in v0.31.4.

## Use

```sh
tv mcp login
tv mcp status
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

## Read contract

Use an exchange-qualified symbol, `1D`, `1W` or `1M`, and count 1..5000 (default
300). Monthly requests map to provider interval `M`; no local resampling occurs.
Date ranges, intraday requests, Desktop targets, batch requests and automatic
fallback are unsupported. Invalid requests fail before credential/provider I/O.

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
