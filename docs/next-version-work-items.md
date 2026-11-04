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
| 1 | Screener mutation outcomes and cleanup | Fixture-tested cleanup and outcome reporting are implemented but unreleased. Actual Desktop close behavior is `UNCONFIRMED`, and a save request that times out is not yet classified as uncertain. | Follow the [outcome reliability plan](plans/tradingview-cli-screener-outcome-reliability.md). Observe the listed Desktop cases when a live check is authorized. |
| 2 | Unreachable Screener JavaScript | Implemented but unreleased. The eight storage and reload expressions no longer embed the helpers, the seven uncalled helpers are gone (53 to 46 functions, 37,979 to 33,713 bytes) and the active and current storage GET share one body. Speed was not measured. | The account JavaScript gate executes all eight expressions with DOM access rejected. No further work before release. |
| 3 | `watch compare` connection reuse | Implemented but unreleased. `watch compare` keeps one `tradingview_market::QuoteClient` for the run instead of building a client per poll. How long the provider keeps idle connections is `UNCONFIRMED`, so reuse at the default 5-second interval is not observed. | Loopback tests count one accepted connection over three polls with a shared client, against one per poll without it, and reconnect after the server closes an idle connection. JSON, order, partial failure, the 25-symbol limit and 4-request concurrency are unchanged. |
| 4 | Implementation documentation corrections | `architecture.md` implies one ten-second bars deadline, while connect, setup, read and cleanup each start their own, for about 40 seconds in the worst case before any symbol search. `operation-adapter-boundaries.md` still calls bars CLI-owned and names the v0.5 roadmap as current. | Correct the current statements to match the code. Keep bars behavior unchanged. |

## Later candidates

| Candidate | Current limitation | Condition or required investigation |
| --- | --- | --- |
| Desktop price-alert response-ID attribution | Confirmation selects a newly listed alert by content; concurrent matching writes can be ambiguous. | Establish the authoritative response ID and saved-row identity before changing the success contract. |
| Broader indicator-input qualification | A non-default integer case does not establish every Pine input type's provider representation. | Qualify additional types when a concrete supported workflow requires them; preserve typed comparisons. |
| Standalone Screener JavaScript | The remaining helpers live in a Rust raw string and no contract gate executes them. | After order 2, move them to `.js` files only with a fixture and `mise` gate like the account contracts, so that behavior gains a test rather than a new location. The 22 expressions that still embed helpers need 5.1 KB of them at the median and 12.5 KB at most; split by those dependencies. |
| Whole-operation bars deadline | The total wait is not bounded by one deadline, but no CLI option or public contract promises one. | Reconsider when a consumer needs a total bound or provider latency data shows the effect. A shared deadline can turn slow successes into partial or timed-out results. |
| Snapshot and compare evidence helpers | Seven identical helpers (82 lines) are duplicated, typed results pass through `Value`, and `SnapshotSection` allows inconsistent field combinations. No incorrect output has been observed. | Consolidate the helpers privately when either copy next needs a change. Use an internal `Result` only if it removes key lookups, and keep public types and JSON unchanged. |

These candidates do not change existing behavior or authorize account mutations.
Release procedures are maintained in [release packaging](release-packaging.md).
