# v0.32.0 candidate ordered work inventory

Direction: [roadmap](next-version-roadmap.md). Contracts, implementation details
and acceptance: [one MCP ExecPlan](plans/tradingview-cli-official-mcp-client.md).
This inventory records order and ownership, not a second implementation plan.

## Current baseline

On 2026-09-20, main, recorded origin/main and the local/remote v0.31.4 tag all
resolved to `48e500b208e67e6d13825efb7de6615a490ae4eb`; the starting tree was
clean. Publication and successful native release jobs are recorded in the
[closed release record](plans/archives/tradingview-cli-v0.31.4-release-readiness.md).
There were no post-tag commits before this closeout/planning work. The workspace
has seven crates at 0.31.4, without MCP dependencies or implementation.

The earlier code inspection at 7a7b883 still applies to the same Rust sources:
`git diff 7a7b883..v0.31.4 -- crates` is empty. The downstream remains at
`cee267cb5ecd48c5eb164b5b1f8b1966d4f67167` with a clean checkout and unchanged
named consumers. Its files were read only. Historical maintenance dependency
classification belongs to the archived release record, not this backlog.

## Ordered work

| Order | Work and owner | Status / completion evidence |
| --- | --- | --- |
| 1 | v0.31.4 closeout — upstream PM | Complete: release/tag/workflow verified and record archived. |
| 2 | Contract/dependency/live-scope preparation — upstream PM | Ready: prior approval retained; first executable slice specified in the MCP plan. |
| 3 | Internal MCP service and deterministic proof harness — current upstream executor | Next. Resolve the five approved dependencies with minimal target features; prove one-dispatch behavior, bounded responses/deadlines and credential lifecycle using fixtures. Existing CLI remains unchanged. |
| 4 | Bounded real connection proof — upstream executor with user browser consent | Approved scope, not executed. Prove registration/login, restart, actual refresh and one-symbol 1D/1W/1M responses; unknown data semantics remain explicit. |
| 5 | Complete first CLI read path — upstream executor | Follows wire evidence. Implement auth commands, explicit backend, mcp_bars.v1/errors and documentation; pass compatibility and negative-case contracts. |
| 6 | Artifact/cache/analysis acceptance — downstream owner | Handoff after a usable upstream read exists. Add source-aware adapter/cache isolation; preserve or quarantine unsupported prepared_bars.v1 semantics. No downstream write by this session. |
| 7 | v0.32.0 qualification and release preparation — upstream PM | Conditional. Initial workflow and downstream acceptance, relevant platform checks, then version/release artifacts. Publication is not authorized. |

Dependency/local-proof and the exact bounded live work are already approved.
Do not insert another generic approval gate before orders 3 or 4. An actual
browser consent requiring broader scopes, an incompatible storage format, a new
dependency outside the approved list, or a materially different budget is a
specific decision point. No additional agent or executor session is authorized.

## First return point

The local proof slice returns an executable development harness, passing
synthetic fault/credential tests and the resolved dependency/platform report.
Then continue into the approved real proof when the user can complete browser
consent. The proof report must separate actual refresh/read observations from
fixtures and list unresolved semantics; it may conclude no-go without creating
a misleading public CLI contract. Do not add a new plan for each substep.

## Downstream handoff obligations

The existing consumer inspection remains in the MCP plan. The downstream owner
must add an explicit contract/backend reader, prevent cross-provider cache reuse,
replace fixed WebSocket/zero-latency assumptions only in approved policies, and
decide how to retain null volume and unknown period anchoring/finality/delay.
Acquisition success is not prepared-bars or backtest acceptance. Desktop provider
paths are independently owned and are not migrated by the new bars backend.

## Verification for this preparation

Read-only Git/release/workflow/consumer checks and public specification refresh
are complete. Changed document links, JSON examples, public hygiene
and diff hygiene passed. No functional test/build rerun is required for these plan-only
changes; no dependency installation, credential access, live tool call or
production source change is included in this preparation.
