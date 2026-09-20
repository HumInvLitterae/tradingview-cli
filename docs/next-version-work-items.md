# v0.32.0 candidate ordered work inventory

Direction: [roadmap](next-version-roadmap.md). Contracts, implementation details
and acceptance: [one MCP ExecPlan](plans/tradingview-cli-official-mcp-client.md).
This inventory records order and ownership, not a second implementation plan.

## Current baseline

On 2026-09-20, main, recorded origin/main and the local/remote v0.31.4 tag all
resolved to `48e500b208e67e6d13825efb7de6615a490ae4eb`; the starting tree was
clean. Publication and successful native release jobs are recorded in the
[closed release record](plans/archives/tradingview-cli-v0.31.4-release-readiness.md).
There were no post-tag commits before this closeout/planning work. At that
baseline the workspace
had seven crates at 0.31.4. The 2026-09-21 local checkpoint adds the eighth
internal MCP member and the five approved dependencies. The follow-up adds
http 1.5.0 and sse-stream 0.3.0, without a direct async-trait dependency;
resolved evidence and compatibility tests are in the MCP plan.

The earlier code inspection at 7a7b883 still applies to the same Rust sources:
`git diff 7a7b883..v0.31.4 -- crates` is empty. At the initial inspection, the downstream was at
`cee267cb5ecd48c5eb164b5b1f8b1966d4f67167` with a clean checkout and unchanged
named consumers. Its files were read only. Historical maintenance dependency
classification belongs to the archived release record, not this backlog.

## Ordered work

| Order | Work and owner | Status / completion evidence |
| --- | --- | --- |
| 1 | v0.31.4 closeout — upstream PM | Complete: release/tag/workflow verified and record archived. |
| 2 | Contract/dependency/live-scope preparation — upstream PM | Complete: prior approval retained; HTTP/SSE release review and macro-free SDK credential compatibility verified. |
| 3 | Internal MCP service and deterministic proof harness — current upstream executor | Local macOS harness implemented and fixture-verified, including HTTP/OAuth/credential failure paths. Native and cross-platform acceptance remain open. The existing `tv bars` path remains unchanged. |
| 4 | Bounded real connection proof — upstream executor with user browser consent | Approved retry passed OAuth exchange, native save and cross-process MCP discovery after homepage sign-in. macOS proof passed: fixed-worker reuse, refresh and 20 OHLCV rows per timeframe with matching symbol/interval echoes. Windows qualification remains mandatory and open. |
| 5 | Complete first CLI read path — upstream executor | Independent `tv mcp login/status/bars/logout` implemented with `mcp_bars.v1`, typed errors and usage docs. Synthetic service/CLI checks and macOS public-command smoke pass, including three fresh-process timeframe reads. The initial implementation is committed. The credential-read/worker-diagnostics follow-up passed fixtures and a subsequent macOS smoke; historical transient failure cause remains unconfirmed. Windows runtime qualification is deferred by the owner and remains required before release. |
| 6 | Artifact/cache/analysis acceptance — downstream owner | Downstream reports initial observation intake, storage/readback and optional error-reason compatibility complete. Analysis/backtest and scheduled collection adoption remain separate downstream work. No downstream write by this session. |
| 7 | v0.32.0 qualification and release preparation — upstream PM | Conditional. Initial workflow and downstream acceptance, relevant platform checks, then version/release artifacts. Publication is not authorized. |

Dependency/local-proof and the same-target live work are already approved.
The owner explicitly withdrew the assistant-imposed total count/time gates.
Do not insert another generic approval gate before orders 3 or 4. An actual
browser consent requiring broader scopes, an incompatible storage format, a new
dependency outside the approved list, or a different account/data target is a
specific decision point. No additional agent or executor session is authorized.

## Current return point

The local harness and macOS connection proof are complete. The independent
CLI is implemented; deterministic checks and the macOS public-command smoke
pass, and the initial implementation is committed. The credential-read and
worker-diagnostics follow-up preserves the public error code and adds optional
reason detail; downstream fixture acceptance and the corrected binary macOS
smoke have passed. Symbol search, column discovery and single-symbol data are
implemented and passed deterministic checks and macOS public CLI smoke.
Multi-symbol data follows this slice in the agreed expansion order below.
The owner has handed the separate `mcp_bars.v1` path to the downstream PM. Windows
qualification is deferred to a later stage and remains a release requirement. A successful read
still leaves delay/adjustment/session/finality/anchoring unconfirmed. No new
plan, shared backend switch, or release version bump is needed at this stage.

## Expansion work order

The owner agreed to this sequence; the [roadmap](next-version-roadmap.md#agreed-expansion-order)
owns its rationale. Each slice uses the existing MCP work record for concrete
contracts and acceptance. These are not all mandatory v0.32.0 release contents.

| Priority | Slice | Next completion condition |
| --- | --- | --- |
| 1 | Symbol search, column discovery, single-symbol data | Implemented with separate contracts, preserved candidates/missing fields and explicit unknown identity/freshness. Complete: workspace tests, Clippy and macOS public CLI smoke passed. Windows remains a release gate. |
| 2 | Multi-symbol data | Per-symbol results and missing symbols remain distinct, with bounded dispatch. |
| 3 | Screener | Filters/columns and result limits are explicit; returned rows do not imply complete coverage. |
| 4 | Recent-count intraday OHLCV | Supported intervals are validated without adding date-range or completeness claims. |
| 5 | Watchlists and alerts | Read/list foundation, then explicit management with readback; compare existing behavior and retain unsupported Pine/Desktop capabilities. |
| 6 | Earnings and financial data | Periods, units, dates and missing values retain provider meaning. |
| 7 | News, documents and economic data | Pagination, references, timestamps and access limits remain explicit. |

Windows runtime qualification remains deferred by the owner and required before
release. Downstream analysis adoption does not block independent upstream slices.
Implementation order does not grant account-mutation or wider OAuth authority.

## Downstream handoff obligations

The existing consumer inspection remains in the MCP plan. The downstream owner
must add an explicit contract/backend reader, prevent cross-provider cache reuse,
replace fixed WebSocket/zero-latency assumptions only in approved policies, and
decide how to retain null volume and unknown period anchoring/finality/delay.
Acquisition success is not prepared-bars or backtest acceptance. Desktop provider
paths are independently owned and are not migrated by the independent MCP command.

## Verification state

The original plan preparation passed document checks. The integrated macOS development
harness passed the Rust baseline and local failure fixtures; results and
remaining proof gates are recorded in the MCP plan. Real OAuth exchange, native save/read and cross-process MCP discovery have run;
OHLCV reads passed for all three timeframes. Windows is mandatory and not yet
qualified. The independent CLI/contract and downstream observation intake are implemented;
Windows qualification and downstream analysis adoption remain open.
