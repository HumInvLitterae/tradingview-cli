# Official TradingView MCP client: bounded historical reads

Status: **active v0.32.0 candidate plan; first local proof slice ready**,
2026-09-20. [v0.31.4 is released](archives/tradingview-cli-v0.31.4-release-readiness.md),
so the sequencing prerequisite is satisfied. The previously approved exact
dependencies, account/store effects and read budget remain authorized; do not
request that approval again. This request prepares the next work from the
released baseline; no implementation or live operation is performed by the
planning update. Broader actual consent, changed scope or new dependencies
remain specific decision points.
No additional agent/session, downstream write or remote publication is authorized.
Local planning/closeout commits are covered by existing authority; this does not
yet include later MCP implementation commits. Follow [PLANS.md](../../.agents/PLANS.md);
this is the single feature work record.

## First executable slice from v0.31.4

Start with an internal service and opt-in development harness before exposing
new public commands. This is the already-approved connection-proof stage, not
a separate prototype product or background service.

| Area | First-slice change | Acceptance |
| --- | --- | --- |
| Cargo/service boundary | Add the internal `crates/mcp` member and the five approved dependencies with target-specific minimal features. Keep common request/data interpretation I/O-free and reuse core errors. | Resolved versions/features/license/MSRV report; no unintended dependency refresh, server features or runtime proxy. |
| Protocol/HTTP | Use the selected SDK with the private bounded HTTP adapter, no AuthClient replay, no session reinit/SSE retry, and cumulative response limits. | Local JSON/SSE/OAuth fake endpoints prove one tool dispatch, 401/429/timeout/invalid response, origin binding and cancellation. |
| Credential/admission state | Implement the dedicated profile/store and cross-process lock, refresh/save ordering and noninteractive failures. | Synthetic store/concurrent-process tests; no real OS credential access during fixtures. Cover Windows record size and macOS/Linux UI restrictions. |
| Development harness | Add a narrow opt-in example such as `crates/mcp/examples/connection_proof.rs`, using the same service code that the CLI will later call. | A second process can reuse synthetic credentials; startup has no implicit registration/login/tool call. Live commands are explicit and bounded. |
| Proof output | Emit only a private sanitized observation summary and useful typed outcomes; wire decoding follows actual supported schema. | Distinguish requested conditions, returned facts and unknowns; no raw tokens/account IDs/payload dumps or guessed normalized success. |

The first slice does not alter `tv bars`, its parser/defaults, existing public
contracts or the released version. It returns the harness, fixture evidence and
resolved dependency report. Continue into the approved live session after local
checks pass and the user is available for browser consent; do not create a new
plan or generic reapproval step. If real behavior contradicts the contract,
record the concrete difference here before changing the public design.

A first macOS proof does not qualify Windows/Linux credential behavior. Retain
those platform gates for the complete CLI slice, including an explicit result
if the actual OAuth record exceeds Windows storage limits. Do not solve an
unobserved incompatibility by silently changing persisted storage.

## Outcome and scope

A user explicitly authorizes the official service once, then a new `tv` process
can obtain recent daily, weekly, or monthly OHLCV for one exchange-qualified
symbol and emit a source-honest JSON result. A downstream adapter can preserve
that observation and independently decide whether it is usable for analysis.
Expired/revoked authorization, insufficient history, unsupported requirements,
and unknown data semantics remain distinguishable.

Keep one Rust binary. This is an MCP **client**, not a server. Initial tools/calls
are limited to protocol discovery, authorization, and `get_ohlcv`. No arbitrary
tool forwarding, symbol-search fallback, batch API, intraday expansion, scanner,
news, financials, watchlist, alert, Desktop mutation, or custom Pine replacement.
Sequential invocations for 1D/1W/1M are sufficient for initial acceptance.

## Evidence and remaining uncertainty

Public sources and unauthenticated discovery metadata were read on 2026-09-20.
No registration, login, token exchange, MCP tool call or credential access ran.

- [TradingView documentation](https://www.tradingview.com/mcp/docs): endpoint
  `https://mcp.tradingview.com/mcp`, Streamable HTTP, OAuth 2.1, Essential or
  above excluding trials, approximately 100 tool requests/minute/user.
  `get_ohlcv` documents `symbol`, `interval`, `count` (maximum 5,000), and
  `summary`; raw rows use `t` (Unix seconds UTC), `o/h/l/c/v`. Monthly is `M`
  (also documented as `1mo`/`month`), not the CLI's `1M`. No public date bounds
  or older-history cursor are listed. `run_screener` caps at 1,000 and lists no
  offset. These are documented limits, not a live capability audit.
- [TradingView announcement](https://www.tradingview.com/blog/ja/tradingview-mcp-server-public-beta-60864/):
  beta market data is delayed and daily request limits may apply. Exact delay,
  quota accounting/reset, adjustment/session conditions, and response stability
  are not established by this announcement.
- [MCP authorization specification](https://modelcontextprotocol.io/specification/2026-07-28/basic/authorization)
  and [transport specification](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports)
  inform discovery, issuer/resource binding, PKCE, protocol negotiation, and
  bounded JSON/SSE handling. The version actually supported by TradingView is
  **UNCONFIRMED**; do not assume the newest version or lifecycle.
- The [official Rust SDK OAuth guide](https://github.com/modelcontextprotocol/rust-sdk/blob/main/docs/OAUTH_SUPPORT.md)
  offers discovery, authorization and refresh building blocks. Its example
  dependency version is older than the [workspace manifest](https://github.com/modelcontextprotocol/rust-sdk/blob/main/Cargo.toml)
  observed at version 3.4.0 / Rust 1.88. These moving main-branch sources are
  research evidence, not a selected or tested dependency release.

**UNCONFIRMED:** authorized server tool/input/output schema; structuredContent
versus JSON text placement; symbol/interval echo or resolved identity; sorting,
nulls, error payloads, empty-series semantics; bar anchoring/finality; split or
dividend adjustment; regular/extended-session handling; per-series delay and
entitlements; registration/redirect support; granted OAuth scopes, refresh-token
issuance/rotation/expiry/revocation; provider retention rights for the intended
artifacts; process/platform stability. Never derive realtime entitlement from
OAuth success, or historical availability from the bar timestamp alone.

### Current code and consumers

Released upstream baseline: `v0.31.4` / `48e500b`. The Rust-source inspection
originally made at 7a7b883 was revalidated: there is no `crates/` diff. [CLI parsing](../../crates/cli/src/cli.rs) and
[dispatch](../../crates/cli/src/app/dispatch.rs) route `bars` into
[market bars](../../crates/market/src/bars.rs). Its
[transport](../../crates/market/src/bars/transport.rs) sends an unauthenticated
WebSocket token and explicitly requests split adjustment; its
[payload](../../crates/market/src/bars/payload.rs) fixes `bars.v1` and
`tradingview_bars_ws`, with WebSocket wait/pagination diagnostics. Recent mode
caps at 500; date-range mode supports bounded additional windows up to 5,000.
These source-specific facts cannot be fabricated for MCP.

Downstream `t-tools-and-memo` was read only at
`cee267cb5ecd48c5eb164b5b1f8b1966d4f67167`, with a clean starting checkout.
Paths below are relative to that repository, not implementation targets here:

| Consumer | Actual constraint / handoff |
| --- | --- |
| `crates/backtest-cli/src/tradingview_bars_acquisition.rs` | Builds `tv bars`, enforces `bars.v1`, and extracts only a fixed WebSocket error-stage list from stderr. Add explicit backend/contract dispatch and preserve typed MCP errors. |
| `crates/backtest-cli/src/analyze/tradingview_bars_prepare.rs` | Enforces `bars.v1`, derives gaps from legacy optional fields, groups source windows, and refuses duplicate timestamps. A missing MCP field must not bypass a completeness check. |
| `crates/backtest-cli/src/analyze/tradingview_bars_batch_prepare.rs` | Date-window and source/latency policies are validated; its one-attempt rule needs accurate upstream attempt semantics. Do not silently reinterpret a range job as a recent-count job. |
| `crates/toolchain-contracts/src/prepared_bars.rs` | `prepared_bars.v1` requires numeric volume, `PeriodStart` timestamps, and numeric fixed latency in availability. Unknown MCP conditions are not representable by substituting zero. |
| `crates/market-data/src/provider/core.rs` | `FetchRequest` is range-based and its provider returns `Vec<PriceBar>`; MCP recent-count acquisition is not an automatic provider substitution. Desktop paths remain separate. |
| `crates/market-data/src/cache.rs` | Cache key uses symbol/interval/range/timezone but no provider. Separate namespaces and data-semantics identity are prerequisites for mixed-source use. |

A narrow additional source inspection confirmed existing source and zero-latency
constants in `crates/backtest-cli/src/operate/support_research_collect.rs`.
Changing that policy remains entirely downstream; no private artifacts were read.

## Accepted design and considered alternatives

| Decision | Recommendation | Alternative and tradeoff |
| --- | --- | --- |
| Backend choice | `tv bars --backend tradingview-mcp`; omit the flag to retain existing WebSocket behavior. Explicit legacy value: `tradingview-ws`. | `tv mcp bars` separates semantics more visibly but duplicates the bars entrypoint. Alternative not selected; keep one explicit bars entrypoint. |
| Output | New `mcp_bars.v1` for the new backend, existing envelope/error kinds unchanged. | Reuse `bars.v1` only after a reviewed optional-evidence evolution; current consumers can mistake absent legacy checks for completeness. Global `bars.v2` would impose unnecessary migration on unchanged consumers. |
| Authenticated service | One internal `tradingview-mcp` workspace crate (`crates/mcp`), with private auth, credentials, transport, and TradingView tool modules. | CLI-only service modules reduce manifest changes but mix reusable authenticated service ownership with command adaptation. Putting OAuth in market/scanner would broaden their credential-free responsibility. |
| Protocol | Prefer official `rmcp` client/HTTP/auth facilities behind the service's small private boundary. | Handwritten MCP/OAuth increases protocol/security maintenance; a permanent Node/Python proxy adds packaging/process ownership. Neither is preferred. |
| Pure interpretation | Proposed `crates/model/src/mcp_bars.rs` validates typed request and normalized observations; the service maps actual TradingView wire results into those types. | No generalized multi-provider registry or abstract backend framework until a second real consumer needs it. |
| Credentials | OS credential store; macOS Keychain, Windows Credential Manager, and a tested Linux secret-service facility. | Plaintext tokens are easier for headless use but add exposure; encrypted files require separate key ownership. No silent fallback. |

The CLI owns parsing, auth command UX, one-shot JSON/error output and exit status.
The MCP service owns OAuth/resource binding, private credential lifecycle,
protocol requests, cancellation and local admission limits. The model owns
I/O-free interpretation and shaping. Core retains existing lightweight envelopes
and ErrorKind. Downstream consumes the CLI, with no external Rust API guarantee.

### Dependency selection and implementation constraints (2026-09-20)

Downloaded released source archives into a disposable directory for static
inspection only; no Cargo dependency was added, resolved, installed or executed.
The current toolchain is Rust 1.98.1. Pin the initial implementation candidate to
these versions, with defaults disabled unless listed:

| Direct dependency | Target / features | Purpose / published MSRV |
| --- | --- | --- |
| `rmcp = "=3.4.0"` | All; `client`, `auth`, `transport-streamable-http-client-reqwest`, `reqwest` | Released official SDK; Rust 1.88; Apache-2.0. |
| `security-framework = "=3.7.0"` | macOS; no optional features | Keychain operations and explicit suppression of interaction; Rust 1.85. |
| `keyring-core = "=1.0.0"` | Windows only; no optional features | Interface used by the selected native Windows store; Rust 1.85. |
| `windows-native-keyring-store = "=1.1.0"` | Windows only; disable default `search` | Credential Manager access without implementing new Win32 pointer ownership; Rust 1.88. |
| `secret-service = "=5.2.0"` | Linux only; `rt-tokio-crypto-rust` | Async Secret Service access with explicit unlock policy; Rust 1.87. |

The four credential-related crates are MIT OR Apache-2.0. Reuse the workspace's
reqwest 0.13.5, tokio, serde and error infrastructure. SDK reqwest constraint
0.13.2 permits 0.13.5, so a second reqwest major is not required by these
manifests. Resolved features/transitive versions and build/link behavior remain
unverified until the first resolved build; the listed additions are approved.
SDK defaults would enable
server/macros; keep them off. JWT/client-credentials, enterprise auth, subprocess,
server, and general elicitation features are not selected. Linux uses Rust
crypto rather than introducing OpenSSL or native libdbus via this choice; a
running Secret Service and user-session D-Bus are still runtime prerequisites.

This refines the former `keyring` candidate without changing OS-store ownership.
[Keyring 4.2 documentation](https://docs.rs/keyring/4.2.0/keyring/) recommends
linking specific stores for applications needing control. Source inspection of
`zbus-secret-service-keyring-store` 1.0.1 found `unlock_all` during credential
lookup and `ensure_unlocked` before reads. It cannot enforce the agreed
noninteractive policy unchanged. Using `secret-service` directly allows reads
and in-place secret updates without invoking unlock; creation and unlock remain
explicit-login operations. A lock race returns an error, not a prompt.

[SDK 3.4.0 release](https://github.com/modelcontextprotocol/rust-sdk/releases/tag/rmcp-v3.4.0)
and its released source establish the following implementation requirements:

- `transport/common/auth/streamable_http_client.rs` automatically refreshes and
  resends after a 401 in `AuthClient`. Do not use that wrapper for tool traffic.
  Use the SDK authorization manager for explicit pre-dispatch refresh and pass
  the resulting token through a controlled HTTP adapter; a later 401 follows
  the agreed one-dispatch policy.
- `StreamableHttpClientTransportConfig` defaults session reinitialization on and
  uses exponential SSE reconnect. Set `reinit_on_expired_session:false`,
  `retry_config:NeverRetry`, and one concurrent request. Prove no replay through
  a fake server; configuration inspection alone is not acceptance.
- The built-in HTTP adapter bounds an SSE event, but reads a JSON reply with
  `response.bytes()` before parsing. Enforce the proposed cumulative 8 MiB bound
  on JSON and SSE bytes in the application's private HTTP adapter. Reuse SDK
  protocol types/parsing rather than forking the protocol implementation.
- OAuth `with_client` inherits a single redirect policy. Use the explicit
  `OAuthHttpClient` interface to keep token/refresh exchanges nonredirecting,
  bound bodies/deadlines, and validate metadata origins. Refuse the SDK's legacy
  guessed-endpoint fallback; discovered metadata is available below.
- The SDK `CredentialStore` offers refresh-lock hooks. Reuse the existing
  per-operation lock without reacquiring it inside load/save; test rotation and
  concurrent processes. Do not emit SDK Debug/error strings or wire tracing into
  ordinary CLI output, including under user-selected verbose logging.

macOS `SecKeychain::disable_user_interaction` provides a process-wide guard.
Hold it across noninteractive store access and serialize such access; do not
let independent workers re-enable prompts. A synchronous native call is not
cancelled merely by timing out its async waiter, so deadline tests must cover
worker shutdown/process exit rather than claim cancellation from `spawn_blocking`.
Linux uses async store requests inside the operation deadline.

The Windows store enforces a 2,560-byte credential-blob cap. Validate the encoded
credential record size before saving and report a typed local storage failure;
never truncate tokens or split a rotating record into non-atomic fragments.
Actual token/registration record size is a live-proof item. If it exceeds the
native limit, propose a separate persisted-format decision (for example a
DPAPI-protected atomic file) before changing the agreed storage design.

### Approved registration and proof scope

Two public GETs, with no cookies or Authorization header, were performed:

- [Protected resource metadata](https://mcp.tradingview.com/.well-known/oauth-protected-resource/mcp)
  names resource `https://mcp.tradingview.com/mcp`, issuer
  `https://www.tradingview.com`, and bearer headers.
- [Authorization server metadata](https://www.tradingview.com/.well-known/oauth-authorization-server)
  advertises `/mcp/oauth/authorize`, `/mcp/oauth/token`,
  `/mcp/oauth/register`, and `/mcp/oauth/revoke` under that issuer;
  authorization-code and refresh-token grants, S256, public-client token auth
  (`none`), and scopes `mcp:read` / `mcp:tools` are advertised. This is metadata,
  not proof of successful registration, scope meaning, refresh, or OHLCV access.

The owner-approved scope covers the named dependencies, local service /
proof-harness implementation and fake-endpoint tests. Build/test may download and
unpack their Cargo dependencies into the normal Cargo cache, or an explicitly
configured disposable cache, without modifying the protected stashes or downstream.
No library may silently add a runtime proxy or general MCP server.

The approved bounded live scope additionally covers one public-client registration at
`https://www.tradingview.com/mcp/oauth/register`, token exchange and one refresh
at `https://www.tradingview.com/mcp/oauth/token`, and the previously bounded MCP
read budget. Proposed registration fields are client name `tv`,
`token_endpoint_auth_method:none`, `response_types:[code]`, grants
`[authorization_code,refresh_token]`, and one exact loopback redirect
`http://127.0.0.1:<ephemeral-port>/callback` matching the running listener. Request
`mcp:read` initially; inspect registration and actual consent before authorizing
broader scopes. The user selects their paid account and completes browser
consent. Do not select an account from cached credentials or infer what
`mcp:tools` permits from its name. If required scope is broader, return the actual
consent requirement for review before granting it.

Use one dedicated credential record, service `tradingview-cli.mcp`, local profile
`default`, only in the current user's OS store. Creation, read, refresh update,
and deletion of that record for verification are the proposed store effects;
never enumerate or modify other clients' entries. Store no token in ordinary
files. Local coordination state and a private, sanitized observation summary
use a newly created task-specific ignored directory under `target/`; raw payloads
and credentials must not be put in the summary. Proof uses NASDAQ:AAPL,
1D/1W/1M, count 20, at most eight OHLCV dispatches and twelve authenticated MCP
requests in a 30-minute session, one login and one refresh. No remote revocation,
subscription change, watchlist/alert mutation or Desktop operation is proposed.
The 30-minute session is a budget, not authorization to wake later automatically.

A real consent screen or a server rejection can still reveal an uncovered
requirement. All fixture/local work continues independently; do not treat a
metadata field as completed browser consent.

## Accepted contract direction (not implemented)

### Invocation and compatibility

Before, still unchanged after this proposal:

```sh
tv bars NASDAQ:AAPL --timeframe 1D --from 2024-01-01 --to 2024-03-31 --count 500
```

Existing output projection (other fields omitted here):

```json
{"success":true,"command":"bars","data":{"contract_version":"bars.v1","source":"tradingview_bars_ws","request_mode":"date_range"}}
```

Proposed explicit path:

```sh
tv auth tradingview login
tv auth tradingview status
tv bars NASDAQ:AAPL --backend tradingview-mcp --timeframe 1D --count 300
tv bars NASDAQ:AAPL --backend tradingview-mcp --timeframe 1W --count 100
tv bars NASDAQ:AAPL --backend tradingview-mcp --timeframe 1M --count 60
tv auth tradingview logout
```

One profile per OS user/service in the first slice. Require qualified symbols,
`1D|1W|1M`, count 1..5000 (default 300), and send `summary:false`. Map `1M` to
provider `M`; do not resample daily bars locally. Reject date-range options,
unsupported timeframes and out-of-bounds count before credential access/network;
never clamp them. Reject an explicit Desktop `--target-id` for this backend to
avoid a misleading selection claim. No default changes, fallback, or `auto`.

### Successful read with count met

This full synthetic result is a **proposed normalized contract**, not a sample
of the actual MCP wrapper. The fixture assumes provider identity/interval fields
exist; when absent, their values/evidence must be null/unconfirmed. Public
examples use synthetic prices and identity.

```json
{
  "success": true,
  "command": "bars",
  "data": {
    "contract_version": "mcp_bars.v1",
    "source": "tradingview_mcp",
    "source_category": "desktop_free_read",
    "requires_desktop": false,
    "request": {"symbol":"NASDAQ:EXAMPLE","timeframe":"1D","mode":"recent_count","count":2},
    "provider_observation": {
      "symbol": {"value":"NASDAQ:EXAMPLE","evidence":"provider_response"},
      "interval": {"value":"1D","evidence":"provider_response"},
      "data_as_of": {"value":null,"evidence":"unconfirmed"},
      "delay_seconds": {"value":null,"evidence":"unconfirmed"},
      "adjustment": {"value":null,"evidence":"unconfirmed"},
      "session": {"value":null,"evidence":"unconfirmed"},
      "timestamp_semantics": {"value":null,"evidence":"unconfirmed"}
    },
    "client_observation": {
      "received_at":"2024-01-04T12:00:00Z",
      "bar_count":2,
      "returned_range":{"first_time":1704153600,"last_time":1704240000},
      "time_order":"ascending",
      "identity_match":"matched",
      "interval_match":"matched",
      "count_status":"met",
      "calendar_coverage":"unconfirmed",
      "missing_fields":[]
    },
    "transport":{"status":"succeeded","tool_attempts":1},
    "bars":[
      {"time":1704153600,"open":100,"high":103,"low":99,"close":102,"volume":1000,"finality":"unconfirmed"},
      {"time":1704240000,"open":102,"high":104,"low":101,"close":103,"volume":1200,"finality":"unconfirmed"}
    ]
  }
}
```

`success:true` means a valid tool result was received/decoded, not that an
analysis requirement is satisfied. Exit 0 outputs JSON to stdout even for a
valid short/empty result. Failures output the existing error envelope to stderr
and nonzero status; stdout is empty. No raw server error, tokens or OAuth URLs
are copied to diagnostics.

Provider fields contain only actual response values with their evidence origin;
request echoes are not resolved-identity proof. If symbol/interval are missing,
use null/unconfirmed and `identity_match`/`interval_match:unconfirmed`. A reported
mismatch fails `invalid_response`; never relabel another symbol as requested.
Client time is measured after the complete reply, range/count/order are derived
from rows, and server `data_as_of` is separate from all bar timestamps. UTC units
follow the documented field mapping; period-start anchoring still needs evidence.
`count_status` is `met|short|empty`; calendar continuity is never inferred from N.

### Insufficient, missing, or unsupported data

- For the same N=2 request returning one valid bar, retain `success:true`, set
  `bar_count:1`, `count_status:short`, and use that bar for both range endpoints.
  The returned array contains only that row. Do not infer why history is short.
- An empty valid tool result uses `bars:[]`, `bar_count:0`, null range endpoints,
  `count_status:empty`, `time_order:unconfirmed`. A tool error claiming no data
  is a failed operation, not an invented empty successful payload.
- Missing/null volume is retained as null and recorded as, for example,
  `missing_fields:[{"bar_index":0,"field":"volume"}]`. It does not become zero
  or remove a bar. OHLC/time absent or invalid prevents a valid normalized result
  and fails `invalid_response`; zero volume supplied by the provider stays zero.
- A consumer wishing for an older date range must distinguish its own desired
  range from this command's recent-count request. Supplying `--from/--to` fails
  before I/O. No local trimming can establish that the unserved interval exists.
- No assumption about finality or delay is made for a daily, weekly, or monthly
  last bar. The downstream must quarantine unknown conditions or use a reviewed
  representation capable of preserving them.

Concrete short-result projection:

```json
{"success":true,"command":"bars","data":{"contract_version":"mcp_bars.v1","source":"tradingview_mcp","transport":{"status":"succeeded","tool_attempts":1},"client_observation":{"bar_count":1,"count_status":"short","calendar_coverage":"unconfirmed"}}}
```

### Error contract and examples

Proposed `error.details.contract_version:mcp_error.v1`, scoped to MCP operations;
no new common ErrorKind or generalized recovery/timing contract. The following
complete error examples use existing kinds/codes. `tool_attempts` counts every
dispatched `get_ohlcv`, including rejected calls; it excludes OAuth/discovery.

```json
{"success":false,"command":"bars","error":{"kind":"validation","message":"Date ranges are unsupported by this backend","details":{"contract_version":"mcp_error.v1","source":"tradingview_mcp","code":"unsupported_capability","feature":"date_range","tool_attempts":0}}}
```

```json
{"success":false,"command":"bars","error":{"kind":"internal_api_unavailable","message":"TradingView authorization is required","details":{"contract_version":"mcp_error.v1","source":"tradingview_mcp","code":"auth_required","tool_attempts":0,"next_action":"tv auth tradingview login"}}}
```

```json
{"success":false,"command":"bars","error":{"kind":"internal_api_unavailable","message":"TradingView request limit reached","details":{"contract_version":"mcp_error.v1","source":"tradingview_mcp","code":"rate_limited","tool_attempts":1,"retry_after_seconds":60,"retry_after_evidence":"http_header"}}}
```

```json
{"success":false,"command":"bars","error":{"kind":"timeout","message":"TradingView MCP request deadline exceeded","details":{"contract_version":"mcp_error.v1","source":"tradingview_mcp","code":"deadline_exceeded","stage":"tool_response","tool_attempts":1}}}
```

```json
{"success":false,"command":"bars","error":{"kind":"internal_api_unavailable","message":"TradingView MCP response is invalid","details":{"contract_version":"mcp_error.v1","source":"tradingview_mcp","code":"invalid_response","reason":"invalid_bar_shape","tool_attempts":1}}}
```

Validation exits 1; connection failure exits 2; provider/auth/rate/schema failure
exits 3; timeout exits 4; internal/store failure exits 1. Distinguish revoked or
failed-refresh `auth_required`, insufficient scope/plan `access_denied`, absent
tool `unsupported_capability`, schema drift `schema_changed`, JSON-RPC/tool
`provider_error`, network `transport_error`, store `credential_store_unavailable`,
and local admission `local_busy`. Never map arbitrary message substrings into
an entitlement or quota claim. Missing Retry-After is null/unconfirmed; do not
invent the daily reset. Incomplete JSON/SSE at timeout emits no partial bar array.

## Authentication and process lifetime

`login` is explicitly interactive: discover the approved service's resource
metadata and validated issuer, choose supported registration, then browser
authorization-code/PKCE S256 with unpredictable state and a bounded loopback
callback (ephemeral port, exact callback/state/issuer validation). A normal bars
call must never launch a browser, accept a password, or elevate scopes. Reject
unexpected server-requested sampling/elicitation/tool actions.

The callback/registration method and redirect acceptance must be proven first;
do not assume a preconfigured public client ID or embed a client secret. If an
externally hosted client metadata document is required, that is a separate
publication decision with an exact document/URL before work starts. Restrict
credentials to the validated resource/issuer, keep token exchanges off redirects,
and do not forward Authorization headers across origins. Narrow scopes where
offered; if consent is broader than OHLCV, review the actual grant with the owner.
Tool allowlisting still enforces this client's read scope.

Persist the issuer/resource/client registration binding and refresh/access-token
state atomically in an OS-protected credential record. Keep logs and ordinary
config free of credentials, OAuth query strings, account IDs and session IDs.
Test SDK debug/tracing paths with sentinel secrets. No token flag, shell-history
token, environment dump, or imported browser/Codex credential path.

For each process, acquire a per-service/user OS-released lock before reading and
refreshing credentials; reread under lock to avoid refresh-token rotation races.
Commit the rotated record before releasing the lock. A crash between server
rotation and durable save must require reauthorization if continuity is lost;
do not retry an uncertain refresh in a loop. Lock timeout and inaccessible store
produce structured errors. One profile avoids an account ID in public output.

`status` is local-only and reports presence/expiry knowledge and next action,
never claims that local credentials are currently accepted by the provider.
`logout` removes only this client's local record; it does not promise remote
revocation. Documentation gives the user the provider-side revoke route once
verified. Headless reads are possible only with usable stored authorization and
an accessible/unlocked credential store; locked desktop stores or re-consent
need user action. Noninteractive reads/status must disable credential-store UI;
if unlocking or access consent is needed, return a structured user-action error.
Only explicit login may request such interaction. Linux without a supported
store fails explicitly.

## Limits, timeout and retry policy

Proposed initial operational defaults: one in-flight read per local OS user and
service; at least one second between tool dispatches persisted outside the token
record, with a lock spanning the bounded operation. All MCP tool calls from this
client use that admission path and the same lock used for credential rotation.
This bounds repeated processes without a daemon;
it cannot coordinate another host, another OS user or another MCP client sharing
the account. Server limits always dominate.

Use a 30-second absolute noninteractive budget from local admission through
credential refresh, discovery and tool response; connection attempts have a
five-second sublimit. Lock wait, pacing, Retry-After and response-body reading
cannot extend the budget. Interactive login has a separate five-minute bound.
These are client proposals to test, not measured provider response guarantees.

Refresh once before tool dispatch when stored expiry requires it. Default to
one `get_ohlcv` dispatch per invocation: on a later 401, refresh credentials at
most once when safe, but return a structured failure and let the next explicit
invocation read. Successful refresh after rejection reports
`auth_refreshed_retry_required` with a next-action hint to repeat the explicit
read; it must not incorrectly demand login. Failed or unavailable refresh reports
`auth_required`. Do not silently resend after 429, 5xx, timeout, disconnect,
session expiry or parsing failure. Audit/disable SDK automatic tool replay,
scope upgrades, reinitialization loops and reconnect where they exceed this
policy. A received 429 updates a shared local cooldown from a valid Retry-After;
an absent hint imposes a conservative 60-second local cooldown explicitly marked
as client policy, without claiming a server reset. Rate errors return promptly.

Local pacing/cooldown files contain only coordination timestamps, no tokens or
market payloads. Use restricted OS-user permissions, atomic replacement, bounded
clock-skew handling and process-exit lock release. Rate-limit exhaustion tests
use fixtures; never deliberately exhaust the real account quota. No general
CDP/WS retry policy is changed by this design.

## Implementation sequence and acceptance

| Stage | Usable result and work | Acceptance / return condition |
| --- | --- | --- |
| Contract direction review | Source audit, contract direction and concrete dependencies/live effects accepted. | Complete; the published v0.31.4 prerequisite is satisfied. |
| Approved local and real connection proof | A private, opt-in development harness using the intended client path: OAuth, restart, refresh, and one-symbol 1D/1W/1M actual responses. Reuse its service logic in the CLI; no permanent proxy. | Demonstrate fresh-process reuse and an actual refresh followed by a successful read; record safe shape/condition observations, or a precise no-go. Forced local expiry alone is not proof of server refresh. Do not freeze the wire adapter from documentation alone. |
| Complete initial CLI slice | Implement service/model/CLI adapters, protected credential reuse, bounded admission/error paths and proposed output mapping. No released half-finished login-only feature. | All deterministic tests below; actual response shape mapped; a user can authorize once and obtain validated JSON after restart on supported platforms. If proof changes the public proposal materially, return for contract review before implementation. |
| Downstream acceptance | Downstream owner adds explicit MCP reader/adapter and stores a distinct observation, without treating it as legacy bars/provider data. | Matched and unknown evidence survives persistence; short/empty/unsupported/errors are distinct; nullable volume/unknown timestamp/finality/latency cannot enter prepared_bars.v1 as invented values. |
| Minor release qualification | Update public usage, source taxonomy, runtime resources, architecture and release record for v0.32.0 if accepted. | Full baseline, platform/build/package verification and bounded live matrix. Do not publish until separately authorized. |

The proof may close as no-go or a narrower useful observation path. No automatic
promotion of recent samples into date-range/backtest coverage. If semantic
unknowns remain, a downstream observation artifact or quarantine can be useful;
an accepted backtest-ready `prepared_bars.v1` artifact is a separate stronger gate.

### Deterministic verification (proposed, not run)

- CLI snapshots: old invocations/`bars.v1` unchanged; explicit backend dispatch;
  no Desktop connection; invalid inputs rejected before any I/O; stdout/stderr,
  exit codes and broken-pipe behavior follow existing application conventions.
- Local fake OAuth/MCP endpoints with synthetic secrets: metadata/issuer/resource
  mismatch, state/PKCE, malicious redirects, refused registration, expired grant,
  rotation and concurrent-process refresh, crash/store failure, locked store,
  noninteractive behavior, logout, and no secret leakage in all diagnostics.
- Negotiated lifecycle/schema contract tests for the supported SDK version;
  JSON and SSE replies, HTTP 200 with `isError`, JSON-RPC failures, missing tool,
  incompatible required schema change, malformed/truncated/oversized replies,
  and stalled streams. Additive unrelated fields may be ignored; changed required
  types/units fail. No tolerant prose scraping or guessed field mapping. Bound
  response bytes to 8 MiB initially and tool-list pagination to 10 pages under
  the same deadline; revisit only on actual schema-size evidence.
- Pure rows: real zero versus null volume, finite OHLC, valid integer UTC
  timestamps, OHLC ordering, duplicate/conflicting timestamps, unordered rows,
  over-count results, symbol/interval mismatch and missing identity, month mapping,
  count short/empty, finality unknown, and unsupported date ranges. Do not sort,
  deduplicate or trim silently: reject malformed ordering/duplicates/over-count.
- Injected clocks/local multi-process tests: admission lock release, queue time,
  cooldown persistence, clock skew, 401/429/5xx, one tool attempt, bounded refresh,
  no hidden replays, deadline cancellation and no late output.
- Consumer-driven synthetic fixtures for every before/after/error case above;
  downstream tests run by its owner. Required evidence survives any transformation.
- Rust baseline per [development](../development.md#validation-baseline), current
  separately pinned JS gates for release qualification, public hygiene, minimal
  dependency features, package guide/reference checks and source/staged provenance.

### Real connection and platform verification (approved bounded scope)

Approved first budget: one owner-selected paid account, NASDAQ:AAPL,
1D/1W/1M, count 20 each, at most eight
`get_ohlcv` dispatches and twelve authenticated MCP protocol requests in one
30-minute observation session. Discovery/auth flow requests are separately
recorded and bounded by one login attempt plus one refresh; no blind retries.
Track setup/discovery/cleanup traffic as well as tool dispatches so the protocol
budget cannot be bypassed by hidden SDK work. Stop with partial proof if the
remaining budget cannot cover the next operation; do not silently expand it.
The approved budget below and dedicated storage effects above apply. The user
selects the paid account and completes browser consent at execution; any broader
consent or changed targets/effects require a new decision. No subscription purchase or
plan upgrade is proposed. If token lifetime exceeds the observation window,
record refresh as pending and arrange an explicitly approved later check.

Inspect negotiated tool schema, actual wrapper and nulls, identity/timeframe,
OHLCV, returned timestamps/count, observed delay/adjustment/session/finality
fields, empty/error representation if naturally observed, restart, and genuine
token refresh with durable rotation. Preserve unknowns when the provider omits
them. Do not provoke quota exhaustion or change an account to manufacture errors.
Any comparison with legacy WebSocket or Desktop requires separately named
targets and a bounded read budget; equality of a few bars is not semantic parity.

Distribution keeps the four [existing targets](../release-packaging.md#release-channel).
Build and fixture-test all platforms; prove browser callback, credential-store
access, restart and refresh on macOS and Windows and the selected Linux runtime.
An untested target is explicitly unverified and blocks a claim of uniform MCP
support. Inspect TLS/native library link requirements and clean-host behavior;
do not add Node/Python runtime requirements or MSIX activation work.

## Downstream return work

Before accepting MCP data into the normal corpus, the downstream owner must:

1. Add an explicit backend selector and `mcp_bars.v1` reader; preserve typed
   errors instead of discarding them behind the legacy stderr stage allowlist.
2. Define acceptance for unknown identity, anchoring, last-bar finality, adjustment,
   sessions and latency. Reject/quarantine where prepared_bars.v1 cannot express
   these; propose a versioned storage change only with real consumers identified.
3. Partition caches by provider and material semantics; prevent legacy cache hits
   and cross-source overwrites. Preserve source/contract version in manifests.
4. Replace fixed WebSocket/zero-delay assumptions only in separately approved
   collector policies. Keep receipts, sealing and market-session authority local.
5. Demonstrate one accepted or explicitly quarantined daily/weekly/monthly
   observation artifact, negative cases and no unintended Desktop-provider change.

## Guide updates with implementation

Current AGENTS.md and architecture already exclude an MCP **server**, so that
boundary needs no reversal. Update crate ownership for the authenticated client,
source taxonomy/CLI help for explicit backend capabilities, error contract docs,
getting-started and Japanese guidance for OAuth/OS-store/noninteractive use,
runtime market-data selection and package references, and development/CI for
synthetic MCP/OAuth tests. Legacy API research remains relevant to retained
commands. Record the client separately from credential-free market/scanner and
page-session APIs. Do not document draft commands as currently available.

## Progress and review decisions

- Completed: current Git/publication audit, affected implementation and actual
  consumer inspection, public TradingView/MCP/SDK source research, release
  comparison, this single draft and concrete success/failure proposals.
- Not performed: implementation, dependency installation, OAuth, live data,
  platform/runtime acceptance, downstream modifications or MCP implementation
  commits/publication.
- Accepted: recommended release separation, explicit backend/new contract,
  source-evidence/null handling, OS-store direction and one-attempt policy.
- Completed follow-up: released dependency/source inspection and two public
  discovery GETs. No credentials accessed. Exact additions, SDK controls and
  bounded registration/store/data effects are now specified above.
- Approved on 2026-09-20: the exact dependency/local-proof and bounded live
  proposal above. Owner sequence correction: v0.31.4 first.
- 2026-09-20 follow-up: v0.31.4 publication/tag/release jobs verified; baseline
  is now 48e500b. Downstream HEAD/consumer paths remain unchanged. The official
  public tool documentation still has the same bounded get_ohlcv interface.
- Planning validation: four changed plan documents, 24 local links and eight
  JSON examples passed, along with public/diff hygiene. Cargo inputs and Rust
  sources are unchanged; no rebuild or functional-test rerun was performed.
- Next: implement the first local service/proof slice above, then use the
  existing bounded live approval. No dependency installation or tool call ran
  during this plan preparation. Do not reopen settled choices without evidence.
- Authentication settings, schema/semantics and live results may require a
  revised proposal. Keep those findings here rather than creating a second plan.
