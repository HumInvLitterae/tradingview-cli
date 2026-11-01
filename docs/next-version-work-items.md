# Development work inventory

This inventory tracks unresolved product work in priority order. Command behavior
and supported interfaces are documented in the [CLI specification](cli-spec.md),
[Rust API reference](rust-api.md) and [internal API reference](internal-tradingview-apis.md).

## Next release

Evidence for the Screener, `watch compare`, bars and snapshot candidates comes
from source inspection and local fixtures, not from TradingView Desktop or the
provider.

| Order | Candidate | Current limitation | Required work and acceptance |
| --- | --- | --- | --- |
| 1 | Screener mutation outcomes and cleanup | Mutation bodies leave through `?` before `restore()`, so an ambiguous selector can leave a panel the command opened. After a confirmed write, a close failure returns only the close error and drops the write result. A failed readback after a write is not separated from other failures. Close failure has no test. | Use one ExecPlan. Hold the operation result, then attempt cleanup once for all 12 session entry points. Keep the existing rule that the primary error wins. Agree before/after JSON for a confirmed write followed by a close failure. Close transient popups before the panel: success paths already do, and whether one Escape closes the panel with a popup open is `UNCONFIRMED`. Inject a failure after each await, with the panel initially open and closed, using the existing fake runtime. Do not use `Drop` for asynchronous cleanup. |
| 2 | Unreachable Screener JavaScript | Every expression embeds all 53 helpers (37,979 bytes). Eight expressions use none of them, and seven helpers have no caller. The two storage GET bodies are near copies. | Remove the expansion from the eight storage and reload expressions, delete the seven helpers and share the GET body while keeping title checks and error details. Execute all eight generated expressions in a Node fixture with DOM access rejected; PUT and DELETE are among them. Report size only, not speed. |
| 3 | `watch compare` connection reuse | Each poll calls `quote_symbols_typed`, which builds a new client, so the pool does not outlive a poll. The default 5-second interval is within reqwest's 90-second idle limit; how long the provider keeps connections is `UNCONFIRMED`. | Agree before/after examples for one additive `tradingview-market` type that hides reqwest and exposes this quote read only. Preserve JSON, order, partial failure, the 25-symbol limit and 4-request concurrency. Compare TCP accepts over several polls on a keep-alive loopback server, and cover a server closing an idle connection. May move to a later release without blocking order 1. |
| 4 | Implementation documentation corrections | `architecture.md` implies one ten-second bars deadline, while connect, setup, read and cleanup each start their own, for about 40 seconds in the worst case before any symbol search. It also omits that `watch compare` shares a client only within a poll. `operation-adapter-boundaries.md` still calls bars CLI-owned and names the v0.5 roadmap as current. | Correct the current statements to match the code. Keep bars behavior unchanged. Update the `watch compare` sentence if order 3 lands. |

## Later candidates

| Candidate | Current limitation | Condition or required investigation |
| --- | --- | --- |
| Desktop price-alert response-ID attribution | Confirmation selects a newly listed alert by content; concurrent matching writes can be ambiguous. | Establish the authoritative response ID and saved-row identity before changing the success contract. |
| Broader indicator-input qualification | A non-default integer case does not establish every Pine input type's provider representation. | Qualify additional types when a concrete supported workflow requires them; preserve typed comparisons. |
| Standalone Screener JavaScript | The remaining helpers live in a Rust raw string and no contract gate executes them. | After order 2, move them to `.js` files only with a fixture and `mise` gate like the account contracts, so that behavior gains a test rather than a new location. Generated expressions need 3.9 KB of helpers at the median and 12.5 KB at most; split by those dependencies. |
| Whole-operation bars deadline | The total wait is not bounded by one deadline, but no CLI option or public contract promises one. | Reconsider when a consumer needs a total bound or provider latency data shows the effect. A shared deadline can turn slow successes into partial or timed-out results. |
| Snapshot and compare evidence helpers | Seven identical helpers (82 lines) are duplicated, typed results pass through `Value`, and `SnapshotSection` allows inconsistent field combinations. No incorrect output has been observed. | Consolidate the helpers privately when either copy next needs a change. Use an internal `Result` only if it removes key lookups, and keep public types and JSON unchanged. |

These candidates do not change existing behavior or authorize account mutations.
Release procedures are maintained in [release packaging](release-packaging.md).
