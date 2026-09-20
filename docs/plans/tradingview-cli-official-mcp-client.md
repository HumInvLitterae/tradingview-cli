# Official TradingView MCP client: bounded historical reads

Status: **active v0.32.0 candidate; independent CLI implemented, qualification open**,
2026-09-21. [v0.31.4 is released](archives/tradingview-cli-v0.31.4-release-readiness.md).
The approved dependencies are settled: no direct async-trait; http 1.5.0 and
sse-stream 0.3.0 accompany the existing SDK/store selections. No Codex MCP setup
is needed. Live proof now covers OAuth, protected storage, cross-process reuse,
refresh/save, and twenty daily/weekly/monthly bars for the selected symbol.
The actual wire tool is `mcp-tv-get-ohlcv`; both that exact name and the public
`get_ohlcv` spelling are supported, without arbitrary tool forwarding.

The owner explicitly withdrew the assistant-imposed total request/time limits
and asked to finish the same verification without repeated count approvals.
Those numerical limits and the pending extra-window question are superseded.
Continue respecting real rate limits, Retry-After, per-operation deadlines,
body bounds, single dispatch/no replay, and the same account/scope/data targets.
Keep request counters and their original start time as evidence; never reset
history to disguise requests. Windows remains a required delivery platform;
its full build/native-runtime acceptance remains open. The independent CLI is
implemented locally; command acceptance is recorded in the latest checkpoint.
Broader actual consent, changed scope or new dependencies remain specific
decision points.
No additional agent/session, downstream write or remote publication is authorized.
The owner now authorizes implementation commits after minimal readability
corrections. Windows runtime verification is deferred until after this local
checkpoint, and remains required for release qualification. Follow [PLANS.md](../../.agents/PLANS.md);
this is the single feature work record.

## Current next slice and expansion decision (2026-09-21)

The initial OHLCV implementation and credential correction are committed.
The correction passed a subsequent macOS smoke: existing credentials reused,
new-process status succeeded, and daily/weekly/monthly reads each returned twenty
rows with one tool attempt and matching identity/interval. Unknown conditions
remained unknown. This does not establish the cause of the earlier transient
credential failures or long-running reliability. Downstream reports observation
storage/readback and optional error-reason compatibility complete; analysis and
scheduled collection adoption remain separate. Windows runtime remains deferred.

The owner agreed to the expansion sequence in the
[roadmap](../next-version-roadmap.md#agreed-expansion-order), specifically promoting
watchlists and alerts immediately after intraday bars and before financial data.
The earlier exclusions below describe the first delivered slice, not a permanent
ban on this agreed follow-up. Symbol discovery, single/batch symbol reads and
screener queries are implemented using the shared authenticated service. The next
slice is recent-count intraday OHLCV. Existing commands remain intact.

Source inspection supports the watchlist/alert substitution rationale:

- `crates/cli/src/ops/layout/watchlist.rs` reads visible panel DOM and uses
  internal API/UI paths for changes. Official list and ID-specific reads could
  remove that visible-panel dependency for the MCP path.
- `crates/cli/src/ops/alert/create.rs` uses logged-in internal endpoints and chart
  metadata. Official explicit-symbol price alerts are a candidate alternative.
- `crates/cli/src/ops/alert/indicator.rs` also supports saved Pine
  `alertcondition()` workflows. Do not claim that simple official price alerts
  replace this capability or remove the existing route without equivalence.

The [official tool documentation](https://www.tradingview.com/mcp/docs) also
requires operation-level side-effect review: `get_active_watchlist` is labelled
read-only but can activate a list or create a default list; it must not be an
implicit read fallback. Adding an existing symbol can move it to the end.
Alert settings updates cannot change conditions/symbol/timeframe; deletion also
removes fire history. Management contracts must expose these effects, avoid
blind replay after an uncertain response, and verify the requested after-state.
Agree on actual OAuth scope and disposable test targets before live mutations;
the priority decision does not authorize changes to existing account objects.
No additional dependency, executor, publication or default-backend change is
implied. Keep this as the single MCP work record.

## Symbol discovery and single-symbol slice (2026-09-21)

The owner approved implementing the first expansion: `tv mcp search`,
`tv mcp columns` and `tv mcp symbol`. These preserve existing commands and add
`mcp_search.v1`, `mcp_columns.v1` and `mcp_symbol.v1` to the existing MCP envelope.
The concrete usage, synthetic before/after example, missing-field behavior and
error handling are in [the public contract](../official-mcp.md#symbol-discovery-and-data).
This agreement concerns the standalone commands, not changing legacy defaults.

The owner separately authorized same-account read verification for Apple symbol
search, volume-column discovery and three selected fields of the existing OHLCV
proof symbol, including necessary verification of those three functions. No
account mutation, new registration, wider OAuth scope or dependency was needed.
Existing authenticated transport, credential snapshots and admission policy are
shared. A closed tool enum owns exact public/observed names and arguments; it
exposes no generic proxy. `model::mcp_data` owns request validation and shaping.

Observed provider shapes inform the decoders:

- Search wraps `symbols` and `count` under `data`. Preserve all candidates and
  nullable descriptive fields; do not automatically select a candidate.
- Filtered column discovery returns `columns` with names, descriptions, groups,
  markets and optional variants. Unfiltered discovery returns `groups`, with
  column names and reported counts. Output mode and measured counts distinguish
  these forms without asserting complete market coverage.
- Single-symbol results wrap selected values under `data`; the observed response
  had no symbol echo or market-data timestamp. Identity, delay, session and as-of
  remain unconfirmed. Requested keys bound the output; zero, null and absence
  are distinct. Column values retain JSON types without guessed units.
- The actual symbol input schema uses `type:["null","array"]` for columns,
  with string items. The first probe stopped before tool dispatch because the
  previous validator only understood scalar type declarations and `anyOf`.
  The validator now accepts this standard union syntax and still rejects wrong
  item types, missing properties and unsatisfied/new required fields.

Deterministic checks cover candidate ambiguity/empty results, both catalog modes,
variant preservation, zero/null/absence, unknown identity, mismatching echoes,
invalid wrappers/counts and invalid requests before I/O. Synthetic SDK service
checks exercise each new command through JSON/SSE, authentication refresh without
replay, 429, malformed output, tool errors and schema drift before dispatch.
The unchanged OHLCV path remains under the workspace baseline. No arbitrary
forwarding, automatic source fallback, new platform dependency or Windows run.

Validation checkpoint: workspace tests passed (975 passed, 27 ignored, zero
failures); strict workspace Clippy passed. The fixed macOS CLI then passed
same-account credential reuse, new-process status and each public command:
search returned ten candidates; filtered columns returned twenty-three entries;
overview returned seventeen groups; symbol returned all three requested fields.
Each data invocation dispatched one tool call and emitted no stderr with
RUST_LOG=trace. Symbol identity/freshness remained unconfirmed as required.
These are dated observations, not fixed future result counts. No live response
values or machine paths are tracked. Runtime package validation passed with six
skills per root; public/diff hygiene passed. Windows runtime remains deferred
and required for release; downstream consumption of these new data contracts is
not claimed by the earlier OHLCV observation acceptance.

## Multi-symbol slice (2026-09-21)

The owner requested the next ordered slice. The public entry is
`tv mcp symbols <EXCHANGE:SYMBOL>... --columns <FIELDS>`, with `mcp_symbols.v1`.
The [concrete contract](../official-mcp.md#multi-symbol-data) distinguishes returned,
explicitly missing and unreported symbols, retaining input order and per-field
null/absence. The provider cap is 50 symbols; this CLI requires distinct inputs
and makes one batch request without implicit splitting or individual fallback.
Shared column defaults, credentials, deadlines, cooldown and no-replay semantics
remain unchanged. Batch row identity must come from the response, never its order.

The owner authorized proceeding with the same-account batch verification and
asked not to repeat confirmations within this work. The observed response has
`data` keyed by qualified symbol, with a field object per returned symbol;
`missing` is an array of objects with symbol and freeform reason. Reported
returned/missing counts are checked against the corresponding collections.
Missing declarations stay separate from unreported symbols; freeform reasons
are not promoted into classified causes. Request order is reconstructed from
explicit identities, never response position. Malformed field maps, unexpected
symbols, contradictory declarations and inconsistent counts fail closed.

Fixtures cover reordered data, partial/all-missing/unreported results, zero/null/
absence, duplicate and excessive input, malformed provider shapes and shared
transport/authentication failures without replay. Native public-service validation
uses the existing authorized immutable credential worker, avoiding another
executable-consent cycle for unchanged credential code. This is separate from
CLI parser tests and is not a new signed/installed binary acceptance claim.
No account writes, scope upgrades, dependency additions or new executors are
needed. Windows qualification remains deferred and required before release.

Native service verification returned two requested symbols with present selected
fields and one provider-declared missing symbol, in original request order.
The result was `mcp_symbols.v1`, `symbols_status:partial`, `tool_attempts:1`, with
missing fields represented by null rather than fabricated data. This verifies
the public service with real transport/native credentials and the completed
normalizer; it does not claim a new executable's OS-consent verification.
The previous credential worker and installed executable were not replaced.

Final baseline: 981 workspace tests passed, 27 ignored, zero failures; strict
workspace Clippy and formatting passed. Runtime staging retained 48 files and
six skills per root, with reference/parity and public/diff hygiene checks passing.
CLI validation covers excessive and duplicate inputs before credential access;
real native evidence uses the shared public service as described above.

## Official screener slice (2026-09-21)

The owner requested the next ordered feature, carrying forward the instruction
to complete necessary same-account read verification without repeating routine
confirmation questions. Implement `tv mcp screener` with explicit market, JSON
filters, sort field/direction, limit, columns, symbol types, preset and symbolset.
The before/after use and partial/empty/error outcomes are in the
[public contract](../official-mcp.md#screener-queries). No legacy command changes.

The observed wire uses `data.rows` with flat symbol/field objects and
`data.totalCount`. Preserve provider row order and selected JSON field values;
reuse the same field absence/null rules as symbol reads. Additional unrequested
provider fields are omitted. Report measured rows separately from the observed
total: limited/all_reported/unconfirmed does not assert independent market
coverage, timestamp meaning or filter verification. Duplicates, bad identities,
excess rows and totals below the observed rows fail closed.

The closed allowlist adds only `run_screener` / `mcp-tv-run-screener`.
Credentials, shared admission/deadlines and no-replay/fallback behavior are reused.
No dependency addition or account mutation. The native verification uses the
already-authorized worker and the public service, with a three-row market query
and a high minimum-price query expected to return no matches. Windows runtime remains deferred.

Validation: 986 workspace tests passed, 27 ignored, zero failures; strict
workspace Clippy and formatting passed. Runtime package reference/parity checks
passed with six skills per root (48 files), as did public/diff hygiene. Native
public-service reads returned three rows against a reported 15,861 matches and
zero rows against zero matches, respectively, with one tool attempt each and only
requested fields. These are dated observations, not expected future counts.
No new executable credential consent, installed-binary replacement or Windows
runtime validation is claimed. Preset/selection options are fixture-validated;
the live checks cover the numeric-filter limited and empty result paths.

## First executable slice from v0.31.4

Start with an internal service and opt-in development harness before exposing
new public commands. This is the already-approved connection-proof stage, not
a separate prototype product or background service.

| Area | First-slice change | Acceptance |
| --- | --- | --- |
| Cargo/service boundary | Add the internal `crates/mcp` member and the approved dependencies with target-specific minimal features. Keep common request/data interpretation I/O-free and reuse core errors. | Resolved versions/features/license/MSRV report; no unintended dependency refresh, server features or runtime proxy. |
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
| Command entry | Independent `tv mcp login/status/bars/logout`, approved by the owner after connection proof. Existing `tv bars` remains unchanged. | A future `tv bars --backend tradingview-mcp` may share the service, but is not part of this implementation or its completion criteria. |
| Output | New `mcp_bars.v1` for the new backend, existing envelope/error kinds unchanged. | Reuse `bars.v1` only after a reviewed optional-evidence evolution; current consumers can mistake absent legacy checks for completeness. Global `bars.v2` would impose unnecessary migration on unchanged consumers. |
| Authenticated service | One internal `tradingview-mcp` workspace crate (`crates/mcp`), with private auth, credentials, transport, and TradingView tool modules. | CLI-only service modules reduce manifest changes but mix reusable authenticated service ownership with command adaptation. Putting OAuth in market/scanner would broaden their credential-free responsibility. |
| Protocol | Prefer official `rmcp` client/HTTP/auth facilities behind the service's small private boundary. | Handwritten MCP/OAuth increases protocol/security maintenance; a permanent Node/Python proxy adds packaging/process ownership. Neither is preferred. |
| Pure interpretation | `crates/model/src/mcp_bars.rs` validates typed request and normalized observations; the service maps actual TradingView wire results into those types. | No generalized multi-provider registry or abstract backend framework until a second real consumer needs it. |
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

The approved live effects cover the selected account's official OAuth flow and
normal credential renewal, using the endpoints and registration fields below.
Earlier one-registration/one-refresh/count/time budgets were verification choices,
not provider restrictions; the owner later withdrew the numerical stop gates.
No additional registration is needed for the completed proof. The implementation
uses client name `tv`, `token_endpoint_auth_method:none`, `response_types:[code]`,
grants `[authorization_code,refresh_token]`, and the running listener's exact
loopback redirect `http://127.0.0.1:<ephemeral-port>/callback`. Use `mcp:read`.
The owner chooses the account and handles browser/OS consent. Explain the
required operation before opening a dialog, and wait for explicit completion
before the next dependent step. Broader scopes or a different account remain
specific decision points, not inferred from tool names or cached credentials.

Use one dedicated credential record, service `tradingview-cli.mcp`, local profile
`default`, only in the current user's OS store. Creation, read, refresh update,
and deletion of that record for verification are the proposed store effects;
never enumerate or modify other clients' entries. Store no token in ordinary
files. Local coordination state and a private, sanitized observation summary
use a newly created task-specific ignored directory under `target/`; raw payloads
and credentials must not be put in the summary. Proof uses NASDAQ:AAPL, 1D/1W/1M, count 20. Necessary verification retries and
credential refresh stay within this task's approved purpose; do not treat the
retired proof counters as reasons to ask again. No remote revocation,
subscription change, watchlist/alert mutation or Desktop operation is included.
No automatic future wake-up or continued collection is implied.

A real consent screen or a server rejection can still reveal an uncovered
requirement. All fixture/local work continues independently; do not treat a
metadata field as completed browser consent.

## Accepted CLI and JSON contract

### Invocation and compatibility

Existing invocation, unchanged:

```sh
tv bars NASDAQ:AAPL --timeframe 1D --from 2024-01-01 --to 2024-03-31 --count 500
```

Existing output projection (other fields omitted here):

```json
{"success":true,"command":"bars","data":{"contract_version":"bars.v1","source":"tradingview_bars_ws","request_mode":"date_range"}}
```

Independent explicit path:

```sh
tv mcp login
tv mcp status
tv mcp bars NASDAQ:AAPL --timeframe 1D --count 300
tv mcp bars NASDAQ:AAPL --timeframe 1W --count 100
tv mcp bars NASDAQ:AAPL --timeframe 1M --count 60
tv mcp logout
```

One profile per OS user/service in the first slice. Require qualified symbols,
`1D|1W|1M`, count 1..5000 (default 300), and send `summary:false`. Map `1M` to
provider `M`; do not resample daily bars locally. Reject date-range options,
unsupported timeframes and out-of-bounds count before credential access/network;
never clamp them. Reject an explicit Desktop `--target-id` for this backend to
avoid a misleading selection claim. No default changes, fallback, or `auto`.

### Successful read with count met

This full synthetic result illustrates the **normalized CLI contract**, not a sample
of the actual MCP wrapper. The fixture assumes provider identity/interval fields
exist; when absent, their values/evidence must be null/unconfirmed. Public
examples use synthetic prices and identity.

```json
{
  "success": true,
  "command": "mcp",
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
      "received_at":"2024-01-04T12:00:00.000Z",
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
{"success":true,"command":"mcp","data":{"contract_version":"mcp_bars.v1","source":"tradingview_mcp","transport":{"status":"succeeded","tool_attempts":1},"client_observation":{"bar_count":1,"count_status":"short","calendar_coverage":"unconfirmed"}}}
```

### Error contract and examples

`error.details.contract_version:mcp_error.v1`, scoped to MCP operations;
no new common ErrorKind or generalized recovery/timing contract. The following
complete error examples use existing kinds/codes. `tool_attempts` counts every
dispatched `get_ohlcv`, including rejected calls; it excludes OAuth/discovery.

```json
{"success":false,"command":"mcp","error":{"kind":"validation","message":"Date ranges are unsupported by this backend","details":{"contract_version":"mcp_error.v1","source":"tradingview_mcp","code":"unsupported_capability","feature":"date_range","tool_attempts":0}}}
```

```json
{"success":false,"command":"mcp","error":{"kind":"internal_api_unavailable","message":"TradingView authorization is required","details":{"contract_version":"mcp_error.v1","source":"tradingview_mcp","code":"auth_required","tool_attempts":0,"next_action":"tv mcp login"}}}
```

```json
{"success":false,"command":"mcp","error":{"kind":"internal_api_unavailable","message":"TradingView request limit reached","details":{"contract_version":"mcp_error.v1","source":"tradingview_mcp","code":"rate_limited","tool_attempts":1,"retry_after_seconds":60,"retry_after_evidence":"http_header"}}}
```

```json
{"success":false,"command":"mcp","error":{"kind":"timeout","message":"TradingView MCP request deadline exceeded","details":{"contract_version":"mcp_error.v1","source":"tradingview_mcp","code":"deadline_exceeded","stage":"tool_response","tool_attempts":1}}}
```

```json
{"success":false,"command":"mcp","error":{"kind":"internal_api_unavailable","message":"TradingView MCP response is invalid","details":{"contract_version":"mcp_error.v1","source":"tradingview_mcp","code":"invalid_response","reason":"invalid_bar_shape","tool_attempts":1}}}
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

### Complete-slice verification (remaining acceptance matrix)

- CLI snapshots: old invocations/`bars.v1` unchanged; independent MCP dispatch;
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

The original numerical proof budgets are historical and were explicitly
withdrawn by the owner. Keep the same selected account, symbol, timeframes and
read-only purpose. Count setup, discovery, cleanup and tool traffic for evidence,
while applying actual server limits, cooldowns, deadlines and no automatic
replay. Do not ask again merely because a local proof counter exceeds a former
threshold. New scopes, accounts, purchases or provider mutations still require
specific authorization.
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
- Implemented: bounded HTTP/MCP transport, OAuth/credential integration,
  macOS/Windows native adapters, an opt-in proof harness, and independent
  `tv mcp login/status/bars/logout` with `mcp_bars.v1`; evidence below.
- Live progress: approved retry completed OAuth exchange, native save and
  cross-process MCP discovery and token refresh/save. After explicit Always
  Allow, background Keychain access works with the fixed helper. OHLCV remains
  observed for all three requested timeframes. Windows
  runtime qualification is mandatory and remains open.
- Downstream modifications and publication remain outside this execution.
  Local implementation commits are now authorized after readability corrections.
- Accepted: recommended release separation, independent subcommands/new contract,
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
- Current checkpoint: implementation and readability review ready for the
  authorized local commit. Subsequent work is downstream acceptance feedback
  and deferred Windows runtime qualification. The downstream PM was contacted
  by the owner. The macOS public-command smoke passed. Reuse the
  settled dependency and same-target live authority; numerical caps are retired.
- Authentication settings, schema/semantics and live results may require a
  revised proposal. Keep those findings here rather than creating a second plan.


### 2026-09-21 local implementation checkpoint

Started from clean main at 96c8d81, two local documentation commits above the
published baseline. Added the internal `tradingview-mcp` member at workspace
version 0.31.4 and resolved the five approved exact dependencies. Cargo.lock
retains every prior package version: 81 new external package versions and one
workspace package, 341 total packages / eight workspace members. reqwest stays
0.13.5. rmcp enables client/auth/HTTP only, with no server/macros feature;
Windows search remains disabled. security-framework's resolved default/ALPN
features are unified with the existing TLS dependency path, not introduced by
our minimal direct declaration. This is resolved-source evidence on macOS,
not native Windows/Linux build or credential-store acceptance. Windows directory
ACL enforcement/qualification remains open before native use.

The first implemented property is local admission: a Unix user-private directory,
OS-released file lock, one-second dispatch spacing, timestamp-only atomic state
replacement, persisted cooldown, bounded lock wait and rejection of corrupt
state/clock rollback. Synthetic tests cover contention, process kill/reacquire,
state reopening and error cases. The opt-in development example currently
supports only `--local-admission`; it explicitly reports connection proof as
pending and performs no network or credential operations. It is not an OAuth
or OHLCV harness yet. The follow-up below adds a bounded SSE decoder check. Existing `tv` behavior and public contracts are unchanged.

The initial request for three direct references was corrected following owner
review. Calling async-trait a required direct dependency was too strong, and the
SDK-resolved sse-stream 0.2.6 was not the current release. On 2026-09-21 the
[crates.io registry](https://crates.io/api/v1/crates/sse-stream) reported
sse-stream 0.3.0 (published 2026-09-18, not yanked); the
[http registry](https://crates.io/api/v1/crates/http) reported 1.5.0 (published
2026-07-29, not yanked), and the [rmcp registry](https://crates.io/api/v1/crates/rmcp)
reported 3.4.0 as the latest stable SDK. Released source archives, rather than
search-engine snippets, were inspected for compatibility.

| Direct reference | Selected version / license | Decision and concrete evidence |
| --- | --- | --- |
| `http` | 1.5.0 / MIT OR Apache-2.0; Rust 1.57 | Added. A fake HTTP client returns `http::Response` through SDK `OAuthHttpClient` and exercises a synthetic registration without network. |
| `sse-stream` | 0.3.0 / MIT OR Apache-2.0; no declared MSRV | Added with optional features disabled; the Rust 1.98.1 build and decoder fixtures pass. Decode using 0.3, then transfer the four event fields into the SDK stream item. |
| `async-trait` | No direct dependency | SDK `CredentialStore` uses an erased future signature. Explicit `Pin<Box<dyn Future<Output = Result<...>> + Send + 'a>>` methods implement it without a macro; dynamic load/save/clear/default refresh-guard calls and the manager setter compile and pass. |

The SDK still depends on async-trait internally; removing that transitive
package would require changing the SDK. Native async code is suitable for our
internal methods, with boxed-future adaptation confined to this SDK boundary.
This is a signature compatibility requirement, not a need to support an older
Rust compiler. See the [macro's documented expansion](https://docs.rs/async-trait/0.1.92/async_trait/#explanation).

rmcp 3.4.0 requires sse-stream `^0.2.4`, so Cargo cannot unify it with 0.3.0.
The SDK's 0.2.6 remains transitive; our decoder uses 0.3.0 only. The small private
adapter names the SDK's public `BoxedSseResponse::Item` through `Stream::Item`
and moves event/data/id/retry fields without serialization or reparsing. There
is no direct 0.2 dependency, fork, patch override or copied parser. When the SDK
updates to 0.3, remove the conversion if its public types unify.

The published 0.3 changelog changes unknown-field and repeated-metadata parsing,
terminal error behavior, encoder APIs and incomplete-EOF handling. Fixtures
cover fragmented CRLF, repeated metadata/unknown extensions, metadata-only versus
empty-data blocks, malformed UTF-8, terminal errors, and cumulative byte limits
including comments/completed events. Incomplete final events are discarded by
the parser; the future MCP response lifecycle must reject a missing final RPC
result, not infer a successful empty response. Full HTTP/lifecycle testing remains
open. The local harness now exercises both admission and the bounded decoder.

The resolved graph now has 342 packages, adding only sse-stream 0.3.0 to the
previous checkpoint; no prior package version was replaced. http stays at the
already resolved 1.5.0, and our direct dependency list contains no async-trait.

Native store source review also found `secret-service::Item::delete` can invoke
an unlock/delete prompt. A prior unlocked check cannot remove the race. Keep
Linux noninteractive deletion unqualified until a no-prompt path is implemented
and tested; do not claim full Linux parity from macOS compilation. A short-lived
same-binary credential worker is the proposed way to kill/reap stalled native
calls at the deadline. No native worker implementation or store operation is
included in this checkpoint.

Validation results for the initial partial checkpoint are recorded below; the complete
acceptance matrix above remains open. No credentials, public CLI, provider calls,
Codex settings, downstream files or commits were changed.

Checkpoint validation: workspace strict Clippy and `cargo test --workspace`
passed (927 passed, 27 ignored). Two later admission fixtures and the narrowed
local-error enum were checked with a final crate Clippy/test run (six unit tests
passed plus doctests); unchanged workspace suites were not repeated. `cargo fmt
--check`, locked/offline metadata, the local-only example, tracked public hygiene
plus a separate scan of all five new MCP files, 24 local document links, eight
JSON examples and `git diff --check` passed. No JavaScript source changed; the
separate JS gates and release builds were not run. Work remains uncommitted.

Dependency-review validation: `cargo test --workspace --locked` passed with
936 passed / 27 ignored, including 11 MCP unit tests and two SDK interface
integration tests. Workspace strict Clippy, formatting, locked/offline metadata,
the updated local example (admission and SSE passed, zero provider requests),
public hygiene with a separate scan of all eight new files, 24 document links,
eight JSON examples and diff hygiene passed. No native store, real OAuth or
MCP request was made. The full connection-proof slice remains incomplete; this
follow-up resolves dependency selection and decoder/SDK-boundary compatibility.


### Integrated macOS proof harness and local verification (2026-09-21)

The example now supports explicit `login`, `status`, `daily`, `weekly`,
`monthly`, `refresh`, and `logout` operations with a proof-directory argument.
`--local-admission` remains entirely local. These are development operations;
there are still no new public `tv` commands, stable Rust APIs, version bump or
normalized `mcp_bars.v1` output.

- HTTP fixes the approved resource/issuer/endpoints, disables redirects and
  reqwest retries, bounds every body/stream to cumulative 8 MiB, and uses one
  absolute deadline. SDK reinitialization/reconnect and automatic tool follow-up
  rounds are disabled; the adapter also refuses a second tool dispatch. Protocol
  setup/GET/cleanup are counted, not just OHLCV calls.
- OAuth uses the SDK's PKCE/state machinery and our HTTP boundary. Resource,
  issuer, endpoint, grant and scope checks happen before tokens can reach the
  wrong target. Broader scopes, foreign redirects, malformed callbacks and
  unsupported tool schemas fail closed. Only login opens a browser; `status`
  and `logout` are local. OAuth/SDK body-bearing errors are not forwarded.
- The credential record binds schema version, resource, issuer, registered
  loopback redirect and SDK credential state in one OS record. Explicit boxed
  futures implement the SDK trait without a direct async-trait dependency.
  The operation lock is held through refresh and durable save. A token request
  without its matching successful save marker prevents stale-credential reuse
  on the next invocation; it does not trigger another uncertain refresh.
- macOS uses only `tradingview-cli.mcp` / `default` via Keychain with interaction
  disabled. Native calls run in a short-lived same-binary worker with piped IPC;
  timeout kills and reaps the worker. No plaintext credential file, browser or
  Codex credential import, or native-store access occurs in fixture tests.
  Windows size checks are synthetic; Windows/Linux native operations remain
  explicitly unavailable until their implementation/qualification gates pass.
- Timestamp/counter state is private and atomically replaced under the operation
  lock. A proof directory cannot reset an existing budget. The 30-minute budget
  allows one registration, exchange and refresh, eight tool dispatches and
  twelve authenticated protocol requests. Public metadata GETs have a separate
  twelve-attempt cap. A server-requested session/cleanup can spend the protocol
  budget before every desired observation; stop partial rather than expand it.
- Delta-seconds and IMF-fixdate Retry-After are supported. An absent hint uses
  the declared 60-second client policy. An unrecognized nonempty server hint
  prevents more reads within this proof, avoiding an earlier-than-permitted
  retry. The full CLI can add broader HTTP-date compatibility when needed.
- Proof observations expose transport success, requested conditions and safe
  shape/count/range summaries only. Known row candidates are not a frozen wire
  contract; unknown wrappers/identity/delay/adjustment/session/finality remain
  unconfirmed. No raw response or token is written to the observation file.

Local evidence: workspace tests passed with 948 passed / 27 ignored. A final
401-refresh/no-replay extraction and fixture then passed the affected 24-unit
suite; the two SDK-interface tests were already covered by the workspace run.
Workspace strict Clippy and final crate Clippy, formatting, public hygiene and
diff checks passed. The rebuilt example passes local admission/SSE checks with
`RUST_LOG=trace`, emits no SDK logs and makes zero provider requests.

Fixtures exercise JSON/SSE success and short/null-volume observations, 401/429/
404/5xx, redirects, malformed/oversized responses, stalls, schema drift, session
setup/cleanup counting, PKCE/state/issuer rejection, broader grants, token
rotation and fresh-manager reuse, successful server rotation with failed local
save, uncertain-refresh rejection, persisted budget exhaustion, cross-process
admission release and a killed/reaped stalled credential worker. This is not
proof of a native record surviving a real application restart or server refresh.

### Live checkpoint: 2026-09-21

The initial automatic execution review rejected login before process creation.
The owner then explicitly approved the concrete registration, browser, dedicated
Keychain and bounded read effects. Retrying through the normal mechanism was
allowed; that review rejection is resolved and is not a new approval gate.

The first actual login read the dedicated native store and found no existing
record, then stopped at HTTP 403 from authorization-server metadata. It did not
reach registration or browser launch. Adding the honest `tv/<version>` User-Agent
and `Accept: application/json` for metadata permitted both discovery documents
to validate. This observation does not establish which header or external
condition caused the earlier denial. No challenge/cookie bypass was attempted.
The read-only `discover` harness operation and sanitized stage/status/challenge
booleans make this distinction observable without logging response bodies.

Login resumed with the original persisted start time and counters, without a
new budget. One client registration succeeded and the browser handoff command
returned success. The user reported HTTP 403 with a CloudFront-generated
"Request blocked" page at browser authorization. This is user-observed browser
evidence, separate from the earlier adapter-observed metadata 403. It does not
establish a plan/scope rejection or distinguish edge access controls, origin
configuration and service availability. The authorization wait was terminated;
no callback, token exchange, token save, refresh or OHLCV call occurred. Resuming
login preserves even exhausted counters, as verified by the affected 24-unit
suite. The dedicated proof directory and exact timestamps stay in the ignored
local ledger; raw credentials and provider payloads are never tracked.

Final dispatched counts: six public metadata GETs, one client registration,
zero token exchanges, zero refreshes, zero authenticated MCP requests and zero
OHLCV calls. The initial dedicated Keychain reads found no existing record; no
record was created, updated or deleted. The browser error request identifier is
not copied into tracked evidence. The 30-minute window was not extended.

The current proof harness retains registration/PKCE state only in memory until
exchange, so terminating this login does not provide a resumable authorization
flow. No registration deletion/revocation was attempted, and no remote cleanup
is claimed. Do not rerun login with a fresh directory to bypass the spent
registration allowance. A later attempt that registers again requires a
concrete renewal of that one-registration/time-window scope. Before that attempt,
review whether interrupted-login recovery belongs in the continuing client and
retain the no-token/no-market-data outcome. Public fixtures cannot establish
that the external browser 403 has been resolved.

Remaining gates: native macOS login/restart/refresh/OHLCV evidence; provider wire
mapping and data semantics; Windows/Linux store/build/runtime acceptance; complete
CLI/JSON integration and downstream acceptance. macOS proof-worker boundaries
must also be preserved when the public CLI installs its normal tracing subscriber.

### Sign-in recovery and Windows follow-up: 2026-09-21

The owner clarified that the browser 403 URL was `/accounts/signin/`, after
OAuth redirected to sign-in. Normal homepage sign-in worked. With explicit
approval for one additional client registration and a new 30-minute proof
window, the next OAuth attempt completed. One token exchange and native macOS
record save succeeded; the encoded record is 1,288 bytes, below the Windows
2,560-byte limit. A new process loaded the record and completed MCP initialization
and tool discovery, then stopped before a tool call at the input-schema gate.
This establishes recovery for this attempt, not the exact CloudFront rule or
general service availability. Client registration and account authorization are
distinct; earlier wording that only said "registration" was misleading.

After rebuilding the development executable for schema diagnostics, ordinary
Keychain reads required OS interaction. A dedicated `authorize-store` harness
action permits only an explicit user-mediated read of the same item; it changes
no ACL programmatically and performs no OAuth/MCP request. Normal reads keep
interaction disabled and return `storage_interaction_required`. Preserve this
distinction for unsigned/development updates and qualify signed release upgrades.

Windows now uses the already-approved native store, exact dedicated target,
local-machine persistence, byte-preserving atomic record updates, typed size
failures, and missing-record handling. The Windows-only synthetic test uses a
unique disposable target and verifies create/read/update/delete plus preservation
of the previous value after an oversize save. The existing Windows CI test job
will run this test; no workflow has been dispatched by this session. Browser
launch uses PowerShell's normal default-browser action with the URL passed on
stdin, never interpolated into script text or logs, and the launcher is bounded
by the operation deadline on both macOS and Windows.

Local Windows compilation of the actual store/browser modules passed using an
isolated, ignored check manifest with the approved versions. Full
`cargo check --target x86_64-pc-windows-msvc --all-targets` is blocked on this
Mac by aws-lc's C build lacking Windows SDK headers (`windows.h`). Module
compilation is not a substitute for the required full Windows build/tests,
native store/process-restart/browser verification, coordination-directory ACL
qualification and packaged execution. Windows completion remains mandatory.

The workspace baseline passed with 949 tests / 27 ignored and strict workspace
Clippy. The subsequent shared-connection harness change passed 25 unit tests
and both SDK tests, plus strict crate Clippy and formatting. Its `all` operation
uses one MCP setup for the three approved timeframes, spaces their dispatches,
and rejects a duplicate interval rather than replaying a request. Partial
observations survive a later failure. It remains a private proof operation,
not a new public batch API. JSON Schema `number` accepts the integer count 20;
ordinary nullable field variants are handled without inventing provider schema.

A workspace test rebuilt the example while its OS permission check was pending.
The proof procedure now freezes the credential worker outside Cargo outputs and
never overwrites it during the attempt. The user clarified that they had selected
Allow before the instructions explained Always Allow. On the next attempt,
instructions were delivered before opening the OS dialog and execution waited
for the user's explicit completion reply. A new noninteractive process then
read the same item successfully. Token refresh and durable save also succeeded,
and a later MCP connection used the updated credentials. The harness permits an
explicit absolute `--credential-worker-path` for this known immutable executable;
it never discovers or executes a helper from provider data or state files. A
rebuilt driver reused that worker successfully without another OS prompt.

The real catalog contains 35 tools. The wire identifier is
`mcp-tv-get-ohlcv`, whereas the public documentation uses `get_ohlcv`. This was
the remaining dispatch blocker, not an empty catalog or demonstrated scope
failure. The client now accepts those two exact identifiers, uses the advertised
one, and rejects ambiguous matches and arbitrary tool names. The fixture checks
actual dispatch of the observed wire name. No broader OAuth scope was requested.

The owner subsequently withdrew the assistant's artificial count/time gates.
The private `read-proof` mode and its extra-window restriction were removed.
Counters remain durable; normal refresh is allowed again, and uncertain refresh
or failed credential persistence still prevents unsafe continuation. Fixed-worker
reuse, actual cooldowns, deadlines and one-dispatch behavior remain enforced.

### Completed macOS acquisition evidence

The corrected harness obtained 20 rows each for 1D, 1W, M. A second explicit read
confirmed matching provider symbol/interval echoes in all three responses. Both
runs completed; all rows had finite OHLC values with valid high/low bounds,
strictly increasing integer timestamps, and non-null numeric volume. This is
count satisfaction for a recent sample, not date-range completeness or proof
against omitted sessions. Token renewal and protected save succeeded, and the
next process reused the updated record. No additional account registration was
needed. The original retry-directory cumulative totals are now registration 1,
exchange 1, refresh 2, public metadata 20, authenticated MCP requests 24 and tool
calls 6. The earlier abandoned pre-login registration is separate historical
activity; it is not hidden by these totals.

The actual result uses structuredContent with root fields `bars`, `count`,
`interval`, `notice`, `success`, `summary`, `symbol`; rows use `t,o,h,l,c,v`.
The proof validates row values and records the symbol/interval echoes without
claiming independently resolved listing identity. `notice` and `summary` need
interpretation during public-contract mapping; their presence is not evidence
of delay, adjustment, session or finality semantics. Those conditions and period
anchoring remain unconfirmed. No raw bars, token, account-local identifiers or
machine paths are tracked. This proof output is not released `mcp_bars.v1`.

The credentials remain in the dedicated OS item for continuing development;
local deletion/revocation was not needed or claimed. Do not discard working
credentials merely to restart a verification window. Windows full build, native
store/restart/refresh/browser checks and directory ACL qualification remain
mandatory. Complete the public CLI/JSON slice and downstream acceptance before
calling the feature or release ready.

Final validation after retiring the artificial caps: 950 workspace tests passed,
27 ignored, zero failures; strict workspace Clippy and formatting passed. Public
hygiene passed for tracked files and all 16 new MCP files; 24 local Markdown
links and diff hygiene passed. Windows native execution remains unverified.
### Independent command implementation (owner approved)

Expose `tv mcp login`, `status`, `bars SYMBOL --timeframe 1D|1W|1M --count N`,
and `logout`. The envelope command is `mcp`, matching other command groups.
Auth/status results identify their operation; bars use `mcp_bars.v1` above.
Do not modify `tv bars`, add a backend flag, or make later unification a release
criterion. Reuse the service/transport proven by the development harness.
The CLI must handle credential-worker IPC before installing any logger and
suppress SDK/wire tracing for MCP commands regardless of RUST_LOG. Normal reads
validate before any local/provider I/O and remain noninteractive. Windows native
storage/browser support, private coordination state, and clear platform limits
are part of this slice. Runtime Windows acceptance remains an explicit gate.


### Independent CLI verification checkpoint (2026-09-21)

Implemented the four standalone subcommands, shared service transport and pure
`model::mcp_bars` normalization. Existing `tv bars` arguments, defaults and
`bars.v1` remain unchanged. No backend flag or automatic fallback was added.
Validation precedes CDP configuration, credentials and provider I/O. Normal MCP
commands and the same-binary credential worker install no tracing subscriber.

The observed wrapper supports `bars`, `count`, `interval`, `symbol` and `success`.
False success, malformed rows, count disagreement, symbol/interval mismatch and
nonascending timestamps fail closed. Freeform notice/summary text supplies no
machine-readable guarantee, so adjustment/session/delay/finality/anchoring remain
unconfirmed. Missing/null volume remains null; short and empty results retain
successful transport with separate count status.

Checks on this uncommitted implementation:

- Locked workspace tests: 958 passed, 27 ignored, zero failures. The new CLI tests
  cover independent help, pre-I/O validation, CDP independence and silent invalid
  credential IPC. Public-service fixtures cover JSON/SSE, short/null data,
  401 without replay, 429 evidence, malformed responses, tool errors and schema
  changes; existing transport fixtures cover deadlines and body bounds.
- Strict workspace Clippy (all targets/features) and formatting passed.
- Windows MSVC compilation passed for the actual native credential, browser and
  coordination-directory ACL modules in an isolated development check. This is
  not a full Windows build or native execution. Full Windows SDK/build, browser,
  credential lifecycle, restart/refresh and ACL runtime acceptance remain open.
- Public hygiene passed for tracked files and 23 new files. Runtime package
  self-tests, staged references/parity and diff hygiene passed; staging retains
  48 files and six skills in each root. Package docs reference bundled material.
- No existing Cargo.lock package version was removed/replaced. This feature's
  approved dependency graph adds 83 package identities compared with HEAD; it
  is not a new general dependency refresh.

A fixed development `tv` binary was prepared outside Cargo's output location.
Its initial `mcp status` returned the safe credential-interaction error with no
provider calls. After advance instructions and the user's explicit Always Allow
completion reply, `mcp login` reused the existing credentials. A new process
successfully ran local `mcp status`; three further processes each ran
`tv mcp bars NASDAQ:AAPL --timeframe 1D|1W|1M --count 20` with `RUST_LOG=trace`.
All three emitted valid `mcp_bars.v1`, twenty rows, `count_status:met`, matched
symbol/interval observations and exactly one tool attempt. Unknown data
conditions remained unconfirmed; stderr stayed empty. No raw responses, prices,
credentials or account-local IDs were saved in the verification summary.

This verifies the public command on macOS, in addition to the earlier actual
OAuth refresh proof through the shared service. Logout is implemented and
synthetic store deletion is tested; the live working credential was retained,
so public-command native deletion is not claimed. The previous trusted proof
executable was preserved. Required Windows native/runtime and downstream
acceptance remain open; Linux credentials remain unimplemented. The CLI is not
yet a qualified cross-platform release. Local document targets (77) and eight
JSON examples also passed checks. No implementation commit, push, tag, workflow
or release was performed.


### Readability review and local commit authority (2026-09-21)

The owner requested conventional source layout beyond rustfmt and authorized
local implementation commits after minimal corrections. Expanded compressed
OAuth control flow, the proof harness, HTTP fixtures and nested JSON objects;
separated functions and logical phases; wrapped long messages without changing
their text; and expanded the embedded Windows ACL script into statements.
No formatter settings, dependency versions or public contracts changed during
this correction. The trusted live-test executables remain untouched.

The owner has contacted the downstream PM. Windows runtime verification is
explicitly deferred, not waived or required before this local implementation
commit. Push, workflow execution and release are still outside this authority.
Record the resulting commit hash in the local ledger/report, not in a separate
tracked evidence-only commit. Preserve the earlier live evidence and its
platform limits; no new provider/credential operation is needed for layout edits.

Validation after the layout review: 958 workspace tests passed, 27 ignored,
zero failures; strict workspace Clippy and formatting passed. Source/literal
comparison and manual review confirmed unchanged logic and wire/message text;
the embedded ACL script changed only whitespace and statement separators.
Public hygiene passed for all 677 staged repository files, and 77 local document
targets, eight JSON examples and diff hygiene passed. Existing runtime-package
and macOS live evidence remains applicable. No new live or Windows execution ran.


### Credential failure investigation and correction (2026-09-21)

A downstream consumer reported two `credential_store_unavailable` failures at
`stage:authentication`, `tool_attempts:0`, after local status had succeeded.
Later explicit reads succeeded with the same executable. Read-only inspection
confirmed the error envelopes, unchanged executable identity, and subsequent
successful count results. The historical cause remains **UNCONFIRMED**: those
errors discarded the native/worker boundary, and available OS logs did not
supply a decisive error code. Successful later root-directory and piped-process
runs do not establish a persistent cwd, environment-file or Keychain-permission
cause. No downstream artifact or installed/trusted executable was modified.

Source inspection found three native record loads in a normal unexpired-token
read: CLI preflight, authentication restore, and SDK token retrieval. Token
refresh can add another load. Repeated loads create avoidable failure points
inside an operation already protected by the admission lock. The correction
keeps one validated snapshot per Store/operation. Durable save precedes snapshot
replacement; failed writes invalidate it; successful deletion records absence;
new operations read storage afresh. Invalid or failed reads are never cached,
and the original deadline still applies. No retry or source fallback was added.

The credential worker now flushes its reply on the same output handle before
reporting success. Parent-side spawn, input, output, exit and decode failures,
stored-record decoding, and native read/write failures retain distinct internal
causes. The public kind, message, code, stage and exit status remain unchanged;
the existing optional `details.reason` carries a closed, public-safe explanation.
The downstream PM confirmed that its current parser accepts optional reason and
retains stderr bytes; execution against the new fixtures remains downstream work.

Concrete example: an authentication-stage invalid worker reply still produces
exit 1 and `code:credential_store_unavailable`, with `tool_attempts:0`; it now
also reports `reason:credential_worker_reply_invalid`. A native read-path error
instead uses `reason:credential_store_read_failed`. Raw worker bytes, system
error text, account values and paths never enter these fields. Public synthetic
fixtures are in `crates/mcp/tests/fixtures/credential-worker-reply-error.json`
and `crates/mcp/tests/fixtures/credential-store-read-error.json`.

Synthetic tests cover a preflight read followed by a failing hypothetical second
storage access: the SDK read now succeeds with one storage load and one tool call.
Other tests cover new-operation rereads, rotation/delete/failed-write state,
invalid-record rejection, cached-operation deadlines, buffered reply delivery,
worker exit versus malformed reply, and exact downstream error envelopes. These
are fixture results, not a reproduction or resolution claim for the two past
macOS failures. No real credential read/update or provider call was performed.
Windows runtime validation remains deferred.


The first full-suite run exposed an existing timing-dependent fixture: its
one-second deadline could expire before the tool was dispatched, while the test
asserted that it had reached a stalled response. The fixture now waits for the
server to receive the tool call before advancing Tokio's test clock past the
response deadline. This retains the exact one-dispatch assertion and avoids
weakening the property to accept a setup timeout. Only the existing Tokio
dev-dependency enables `test-util`; production versions and Cargo.lock are unchanged.


Final validation for this correction: 965 workspace tests passed, 27 ignored,
zero failures; strict workspace Clippy (all targets/features), formatting and
public/diff hygiene passed. The two public error fixtures match the emitted
ErrorEnvelope exactly. No live call, credential mutation, installed executable
replacement, Windows execution, push or release was performed. Native recurrence
and its historical root cause are not claimed resolved by these fixture results.
