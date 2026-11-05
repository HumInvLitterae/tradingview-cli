# Development work inventory

This inventory tracks unresolved product work in priority order. Command behavior
and supported interfaces are documented in the [CLI specification](cli-spec.md),
[Rust API reference](rust-api.md) and [internal API reference](internal-tradingview-apis.md).

## Next release

The Screener cleanup and outcome work, the removal of unreachable Screener
JavaScript and `watch compare` connection reuse are implemented and listed in the
[changelog](../CHANGELOG.md) under Unreleased. Their evidence comes from source
inspection and local fixtures. A Desktop observation on 2026-11-05 covered screen
create and delete and the panel close; it found the defect below.

| Order | Candidate | Current limitation | Required work and acceptance |
| --- | --- | --- | --- |
| 1 | Screener filter Escapes close the panel | In Desktop 3.4.0 one Escape dispatched to the document closes the Screener panel when no popup is open. `filters add` sends three before looking for the add button, so it fails with `add_button_not_found` every time; v0.41.0 has the same code. Four filter expressions send Escape three times at nine points, and keys left over after a popup closes are expected to close the panel too. | Send each Escape only while the popup it closes is visible, and skip it when none is. Each caller names its popup and gets a test. Accept with a Desktop run of `filters add` and `filters modify` on a `CLI-Test` screen. Evidence is in the [outcome reliability plan](plans/tradingview-cli-screener-outcome-reliability.md). |

## Later candidates

| Candidate | Current limitation | Condition or required investigation |
| --- | --- | --- |
| Broader indicator-input qualification | A non-default integer case does not establish every Pine input type's provider representation. | Qualify additional types when a concrete supported workflow requires them; preserve typed comparisons. |
| Standalone Screener JavaScript | The remaining helpers live in a Rust raw string. Only the add-filter popup close runs them in a contract gate, against a fake DOM. | Move them to `.js` files only with a fixture and `mise` gate like the account contracts, so that behavior gains a test rather than a new location. The 22 expressions that still embed helpers need 5.1 KB of them at the median and 12.5 KB at most; split by those dependencies. |
| Whole-operation bars deadline | The total wait is not bounded by one deadline, but no CLI option or public contract promises one. | Reconsider when a consumer needs a total bound or provider latency data shows the effect. A shared deadline can turn slow successes into partial or timed-out results. |
| Snapshot and compare evidence helpers | Seven identical helpers (82 lines) are duplicated, typed results pass through `Value`, and `SnapshotSection` allows inconsistent field combinations. No incorrect output has been observed. | Consolidate the helpers privately when either copy next needs a change. Use an internal `Result` only if it removes key lookups, and keep public types and JSON unchanged. |

These candidates do not change existing behavior or authorize account mutations.
Release procedures are maintained in [release packaging](release-packaging.md).
