# Development work inventory

This inventory tracks unresolved product work in priority order. Command behavior
and supported interfaces are documented in the [CLI specification](cli-spec.md),
[Rust API reference](rust-api.md) and [internal API reference](internal-tradingview-apis.md).

## Next release

The Screener cleanup and outcome work, the removal of unreachable Screener
JavaScript and `watch compare` connection reuse are implemented and listed in the
[changelog](../CHANGELOG.md) under Unreleased. Their evidence comes from source
inspection and local fixtures. A Desktop observation on 2026-11-05 covered screen
create and delete, filter add and option modify, unsaved-change prompts and the
panel close; the defects it found in filter popups and screen changes are fixed.
No other item is required for the release.

## Later candidates

| Candidate | Current limitation | Condition or required investigation |
| --- | --- | --- |
| Broader indicator-input qualification | A non-default integer case does not establish every Pine input type's provider representation. | Qualify additional types when a concrete supported workflow requires them; preserve typed comparisons. |
| Screener numeric filter modify | In Desktop 3.4.0 a filter's manual setting shows a condition and a value field instead of percent presets, so numeric `filters modify` through the UI fails with "manual settings button not found" before writing. The storage path is unaffected. | Drive the condition and value field, or drop the UI path, when a workflow needs numeric UI edits. Accept with a Desktop run on a `CLI-Test` screen. |
| Screener save-as from unsaved changes | From a screen with unsaved changes, `screens save-as` created the copy but left the original screen active, so the command failed after its write without `write_outcome`. Observed once. | Find out whether the copy keeps the unsaved changes and why the title stays, then report the created copy instead of a plain title error. |
| Screener screen list entries | With seven saved screens, `screens list` returned one extra entry whose name joined several screen names. Observed once in the title menu. | Restrict the menu reader to item elements, with a fixture from the current menu. |
| Screener panel drawing | A window that is not in front does not draw a newly opened panel's filters until a frame is produced. `filters add` now reports this with a hint; filter commands that fail to find pills do not. | Extend the hint only if other filter failures are seen in practice. Bringing the window forward stays the user's choice. |
| Tab indexes | `tab switch <N>` counts CDP chart targets, while `tab close <N>` and `app_tabs` count the app's tab strip, and the two orders differ. `tab switch` activates the target without selecting the tab in the strip. | Choose one index and selection behavior, agree on the change with examples, and update both commands and the specification. |
| Standalone Screener JavaScript | The remaining helpers live in a Rust raw string. Eight filter expressions run them in a contract gate, against fake pages. | Move them to `.js` files only with a fixture and `mise` gate like the account contracts, so that behavior gains a test rather than a new location. The 23 expressions that still embed helpers need 4.3 KB of them at the median and 12.5 KB at most; split by those dependencies. |
| Whole-operation bars deadline | The total wait is not bounded by one deadline, but no CLI option or public contract promises one. | Reconsider when a consumer needs a total bound or provider latency data shows the effect. A shared deadline can turn slow successes into partial or timed-out results. |
| Snapshot and compare evidence helpers | Seven identical helpers (82 lines) are duplicated, typed results pass through `Value`, and `SnapshotSection` allows inconsistent field combinations. No incorrect output has been observed. | Consolidate the helpers privately when either copy next needs a change. Use an internal `Result` only if it removes key lookups, and keep public types and JSON unchanged. |

These candidates do not change existing behavior or authorize account mutations.
Release procedures are maintained in [release packaging](release-packaging.md).
