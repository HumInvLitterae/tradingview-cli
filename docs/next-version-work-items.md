# v0.33.0 ordered work inventory

Direction: [roadmap](next-version-roadmap.md). Decisions and acceptance:
[one active work record](plans/tradingview-cli-mcp-operational-improvements.md).

| Order | Work | Completion condition |
| --- | --- | --- |
| 1 | Close v0.32.0 and establish baseline | Done: publication/workflow verified; prior record archived with historical evidence preserved and downstream report distinguished from local checks. |
| 2 | Settle new read contracts and live verification scope | Contract proposals and scoped reads approved; actual response shapes and missing-value rules still need verification before finalizing normalization. Both are planned features. |
| 3 | Improve login/upgrade operation | Implemented terminal-only pre-interaction guidance; captured stderr, existing stdout/error JSON and credential reuse verified by focused tests. Fresh live authorization not repeated. |
| 4 | Implement alert history | Implemented and fixture-verified, including nonempty provider shape evidence. Public-service 30-second native acceptance remains open. A long-budget empty-history read completed at 50.8 seconds, including 26.9 seconds before catalog completion; the latency root cause is unconfirmed. |
| 5 | Implement official technical snapshot | Single symbol/timeframe, provider values/ratings without local recomputation, missing/unknown conditions retained; fixtures and scoped real acceptance. |
| 6 | Measure and conditionally optimize transport | Implemented and measured: initialization/catalog reduced from two to one on a single-page catalog; lazy pagination and failed readback retain mutation results. MCP regression suite and scoped Clippy passed; no live latency claim. |
| 7 | Stabilize affected fixtures | Reproduce and correct time/readiness dependencies; normal CI parallel execution passes without production timeout changes. Work can accompany stages 3–6. |
| 8 | Integrate documentation and standalone skills | User guides, command/source mapping and account-management/market-data references reflect implemented behavior; individual skill and archive checks pass. |
| 9 | Qualify and prepare release | Applicable Rust/platform checks, scoped runtime acceptance and honest limits recorded; version/notes in a separate final preparation commit. Publication separately authorized. |

The local resource prerequisite is complete: heavy pre-push checks are disabled
and explicit local baselines default to one build job/test thread. Both new
contracts and scoped reads are approved. Catalog/schema checks passed. History
now has a public command, strict normalization, deterministic tests and updated
standalone guidance. Account-wide shape-only investigation established nonempty
fields; the public CLI still requires a symbol. Its separate normal-deadline
native check timed out after one dispatch, so runtime acceptance remains open.
Technical daily returned a provider application error with a rate-limit textual
clue; further technical probes are stopped until the limit permits them.
Successful indicator fields remain unqualified. No version bump or dependency
addition occurred. The active record separates fixture, investigation-budget
and public-deadline evidence; release readiness is not yet established.

The daily technical recheck again returned a rate-limit textual clue. The
opt-in synthetic before/after measurement and command-local connection reuse
are complete. Successful technical response qualification and normal-deadline
history acceptance remain the next provider-dependent work.

Completed-response diagnostics now distinguish long-budget success from public
readiness. Repeated identical technical checks still return a rate-limit textual
clue; another live attempt should follow new availability evidence or a concrete
diagnostic hypothesis, not an automatic retry on each continuation. The public
30-second deadline and approved feature scope remain unchanged.

Independent anonymous HTTP checks reproduced variable response latency outside
the Rust client; no transport configuration fix is established. The active
record proposes an explicit read-only timeout option with the 30-second default
unchanged. That public CLI extension is pending owner approval, not implemented
or part of the previously approved scope.
