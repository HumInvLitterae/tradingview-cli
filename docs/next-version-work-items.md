# Development work inventory

This inventory tracks unresolved product work in priority order. Command behavior
and supported interfaces are documented in the [CLI specification](cli-spec.md),
[Rust API reference](rust-api.md) and [internal API reference](internal-tradingview-apis.md).

## Next release

The Screener cleanup and outcome work, the removal of unreachable Screener
JavaScript and `watch compare` connection reuse are implemented and listed in the
[changelog](../CHANGELOG.md) under Unreleased. Their evidence comes from source
inspection and local fixtures, not from TradingView Desktop or the provider.

| Order | Candidate | Current limitation | Required work and acceptance |
| --- | --- | --- | --- |
| 1 | Screener Desktop observation | Whether Escape closes the panel while a screen name dialog is open, how often a close fails after a write, and what the Escapes left over after a filter popup closes reach are `UNCONFIRMED`. | Observe the cases listed in the [outcome reliability plan](plans/tradingview-cli-screener-outcome-reliability.md) when a live check is authorized. |

## Later candidates

| Candidate | Current limitation | Condition or required investigation |
| --- | --- | --- |
| Broader indicator-input qualification | A non-default integer case does not establish every Pine input type's provider representation. | Qualify additional types when a concrete supported workflow requires them; preserve typed comparisons. |
| Standalone Screener JavaScript | The remaining helpers live in a Rust raw string. Only the add-filter popup close runs them in a contract gate, against a fake DOM. | Move them to `.js` files only with a fixture and `mise` gate like the account contracts, so that behavior gains a test rather than a new location. The 22 expressions that still embed helpers need 5.1 KB of them at the median and 12.5 KB at most; split by those dependencies. |
| Screener popup Escape count | Nine filter expressions close popups by sending Escape three times without checking between keys, so Escapes left over after the popup closes reach whatever lies beneath it. Only the add-filter popup close after an error stops once its popup is gone. | Decide after the Desktop observation shows what the leftover Escapes close. Stopping per key needs each caller to name the popup it closes, so change the helper only with a test per caller. |
| Whole-operation bars deadline | The total wait is not bounded by one deadline, but no CLI option or public contract promises one. | Reconsider when a consumer needs a total bound or provider latency data shows the effect. A shared deadline can turn slow successes into partial or timed-out results. |
| Snapshot and compare evidence helpers | Seven identical helpers (82 lines) are duplicated, typed results pass through `Value`, and `SnapshotSection` allows inconsistent field combinations. No incorrect output has been observed. | Consolidate the helpers privately when either copy next needs a change. Use an internal `Result` only if it removes key lookups, and keep public types and JSON unchanged. |

These candidates do not change existing behavior or authorize account mutations.
Release procedures are maintained in [release packaging](release-packaging.md).
