# Screener column storage management

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md`. It is self-contained so a new contributor can understand and continue the work from this file alone.

## Purpose / Big Picture

Users can already read Stock Screener rows, switch prepared screens, save screens, create and delete test screens, and manage filters. The remaining practical Screener gap is column management: operators need to inspect the saved column configuration, remove unwanted columns, and reorder columns on disposable test screens without relying on brittle column-settings dialog clicks. This slice adds storage-backed column configuration reads and the smallest safe mutation set: remove and reorder for test/disposable screens only.

The behavior is visible through commands such as `tv screener columns config`, `tv screener columns remove --name "EMA (21)" --dry-run`, and `tv screener columns reorder --from-index 12 --to-index 11 --dry-run`. Normal mutation uses TradingView's saved Screener screen storage API from the logged-in page session, then reads the same storage API again and reports success only if the saved column order matches the requested result.

## Progress

- [x] (2026-04-27 10:10Z) Confirmed the current full-page Screener target exposes the active test screen as a saved screen with `active_column_set: custom` and storage columns in `default_custom_column_set`.
- [x] (2026-04-27 10:25Z) Added CLI surface for `tv screener columns config` and `tv screener columns reorder --from-index <N> --to-index <N> [--dry-run]`.
- [x] (2026-04-27 10:45Z) Reworked `tv screener columns remove` from visible-UI action discovery to saved-screen storage payload mutation with dry-run and post-check behavior.
- [x] (2026-04-27 11:00Z) Added operation tests for config reads, dry-run remove, normal remove, test-screen guard, dry-run reorder, normal reorder, and reorder validation.
- [x] (2026-04-27 11:05Z) Ran focused column tests: `cargo test screener_column -- --nocapture`.
- [x] (2026-04-27 11:25Z) Ran live smoke on the prepared full-page test Screener target: read active screen, read `columns config`, dry-run remove, dry-run reorder, normal reorder, reverse normal reorder, and final `columns config`.
- [x] (2026-04-27 11:45Z) Full validation baseline passed: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `git diff --check`, and tracked-doc local-path / `USER;` grep with only existing validation-command examples.
- [x] (2026-04-27 11:50Z) Updated `CONTINUITY.md` with the column storage management state and validation evidence.
- [ ] Commit the related implementation and docs.

## Surprises & Discoveries

- Observation: The active test screen stores the visible column set under saved screen storage as `default_custom_column_set`, with each entry carrying an `id` and optional `params`.
  Evidence: Read-only page-session inspection showed the active test screen has `active_column_set: custom` and 13 storage columns corresponding to the visible columns.
- Observation: The same read-only inspection did not expose a safe column catalog or canonical default column set.
  Evidence: `window.initData` did not expose useful column catalog keys for add/reset. Therefore `columns add` and `columns reset` remain deferred in this slice.
- Observation: A reversible reorder smoke is safer than a normal remove smoke because this slice intentionally does not expose `columns add`.
  Evidence: Removing a live column would require a separate add/catalog implementation to restore it, while reordering adjacent columns can be immediately reversed with the same command.
- Observation: The initial storage save result returned the whole saved screen response, including filters, which was more data than the column command needs.
  Evidence: Normal reorder smoke succeeded but produced a large nested `save_result.response_screen`. The implementation was tightened to return only save status, screen id, title, and column counts.
- Observation: Visible column labels can lag after a storage-only reorder until the page refreshes or TradingView re-renders the table.
  Evidence: The storage ids and params post-check succeeded during normal reorder, but visible labels were still read by index from the current table. Storage-column payloads now mark names as `name_source: "visible_column_index"` to keep that boundary explicit.

## Decision Log

- Decision: Implement normal `columns remove` and `columns reorder` through the saved-screen storage API rather than the visible column settings dialog.
  Rationale: Previous evidence showed the visible column UI exposes categories, search, and move/sort menus but no stable per-column remove or reset action. The saved-screen storage payload is exact, compact, and post-checkable.
  Date/Author: 2026-04-27 / Codex
- Decision: Limit normal column mutations to active screen names containing `CLI-Test` or `テスト`.
  Rationale: Column storage edits change TradingView cloud state for a saved Screener screen. The repository already uses this guard for test/disposable Screener screen lifecycle commands, and it keeps accidental production screen edits out of the normal path.
  Date/Author: 2026-04-27 / Codex
- Decision: Do not add `columns add` or `columns reset` in this slice.
  Rationale: `columns add` needs a reliable catalog of valid column ids and params, and `columns reset` needs a reliable default source. Neither was exposed by the current read-only evidence, so publishing those commands would invite guessing.
  Date/Author: 2026-04-27 / Codex

## Outcomes & Retrospective

The implementation now lets users inspect storage column ids and params, dry-run a remove or reorder, and normally remove/reorder columns only on prepared test screens. Live smoke used a reversible reorder on `米国株（テスト用）`, then reversed it and confirmed the final storage order returned to the original 13-column order. Full validation passed. Remaining column work after this slice is `columns add`, `columns reset`, and possibly a richer catalog/discovery command if TradingView exposes a stable source.

## Context and Orientation

The Rust CLI is implemented as the `tv` binary. The command parser lives in `src/cli.rs`, dispatch lives in `src/main.rs`, operation exports live in `src/ops.rs`, and Screener behavior lives in `src/ops/screener.rs`.

Screener commands talk to TradingView Desktop through the Chrome DevTools Protocol. A "full-page Screener target" is a separate TradingView page target whose URL is a Screener page rather than a chart page. `tv tab list` reports these targets under `screener_targets`, and a user can run follow-up commands against one by setting `TV_CDP_TARGET_ID`.

The "storage API" in this plan means TradingView's logged-in page-session screen storage endpoint, accessed from inside the authenticated TradingView page with `fetch`. It is not a public stable API. The command must therefore treat any missing storage metadata, failed save request, or failed post-check as `internal_api_unavailable` rather than guessing success.

## Plan of Work

In `src/cli.rs`, add `Config` and `Reorder` variants under `ScreenerColumnsCommand`. In `src/main.rs`, validate reorder requests before opening a CDP connection and dispatch the new commands.

In `src/ops/screener.rs`, add a storage-column target type that preserves `index`, storage `id`, optional visible `name`, and `params`. Add helpers to read the active saved screen storage payload, convert its `default_custom_column_set` into storage-column targets, save an updated custom column set, and compare the re-fetched storage order with the expected result.

Change `screener_columns_remove` so dry-run resolves one visible column, maps it to the same storage index, and reports the post-remove expected column list without saving. Normal mode must first verify that the active screen name is test/disposable, update `default_custom_column_set`, save via storage API, re-fetch the active screen storage payload, and report success only if the saved storage order matches the expected post-remove list.

Add `screener_columns_reorder` with the same storage API and post-check strategy. It rejects equal source/destination indices before connection and rejects out-of-range indices after reading storage columns.

Update README, CHANGELOG, and Screener contract notes so they no longer describe normal column mutation as purely deferred. Keep `columns add/reset` deferred because the current evidence does not expose a reliable column catalog or default source.

## Concrete Steps

From the repository root, run:

    cargo test screener_column -- --nocapture
    cargo test --test cli_contract screener -- --nocapture
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -nE '(/Users/|C:\\|USER;)' -- README.md CHANGELOG.md docs .agents/skills || true

For live smoke, first identify the full-page Screener target:

    tv tab list

Then set `TV_CDP_TARGET_ID` to the Screener target and run read-only and dry-run commands:

    TV_CDP_TARGET_ID=<screener-target> tv screener screens active
    TV_CDP_TARGET_ID=<screener-target> tv screener columns config
    TV_CDP_TARGET_ID=<screener-target> tv screener columns remove --name "テクニカル評価" --dry-run
    TV_CDP_TARGET_ID=<screener-target> tv screener columns reorder --from-index 12 --to-index 11 --dry-run

Normal live smoke should prefer a reversible reorder on the prepared test screen:

    TV_CDP_TARGET_ID=<screener-target> tv screener columns reorder --from-index 12 --to-index 11
    TV_CDP_TARGET_ID=<screener-target> tv screener columns reorder --from-index 11 --to-index 12
    TV_CDP_TARGET_ID=<screener-target> tv screener columns config

Do not run normal `columns remove` during live smoke unless a disposable column can be restored by another verified path.

## Validation and Acceptance

Acceptance requires all focused and full validation commands to pass. `tv screener columns config` must return `scope: "screen_storage_api"`, the active screen title, the active column set, and storage columns with ids and params. `tv screener columns remove --dry-run` must return the target storage column and the expected post-remove list without saving. Normal `columns remove` must refuse non-test screen names and must not report success unless the storage API post-check confirms the exact expected order.

`tv screener columns reorder --from-index <N> --to-index <N> --dry-run` must return the expected post-reorder order without saving. Normal `columns reorder` must be limited to test/disposable screens and must report success only after the saved storage order matches the expected order. `columns add` and `columns reset` remain absent from the CLI in this slice.

## Idempotence and Recovery

Read-only and dry-run commands are safe to repeat. Normal reorder is reversible by running the opposite reorder. Normal remove is not automatically reversible in this slice because `columns add` is not implemented, so live smoke should avoid normal remove unless the target column can be restored manually or by a later verified command.

If a storage save fails, the command returns `internal_api_unavailable` with the save response. If save succeeds but the post-check does not match the expected column order, the command also returns `internal_api_unavailable` rather than assuming success. In either case, re-run `tv screener columns config` against the same `TV_CDP_TARGET_ID` to inspect the saved storage state.

## Artifacts and Notes

Focused test output after the implementation:

    cargo test screener_column -- --nocapture
    running 11 tests
    11 passed; 0 failed

The read-only live evidence intentionally records only summarized shape, not raw storage payload or account-linked identifiers:

    Active test Screener screen: 米国株（テスト用）
    active_column_set: custom
    storage columns: 13 entries with id + params

## Interfaces and Dependencies

At the end of this plan, these interfaces exist:

    pub fn validate_screener_column_reorder_request(from_index: usize, to_index: usize) -> Result<(usize, usize), AppError>
    pub async fn screener_columns_config(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>
    pub async fn screener_columns_reorder(runtime: &mut impl RuntimeEvaluator, from_index: usize, to_index: usize, dry_run: bool) -> Result<Value, AppError>

The existing `screener_columns_remove` interface remains:

    pub async fn screener_columns_remove(runtime: &mut impl RuntimeEvaluator, selector: ScreenerColumnSelector, dry_run: bool) -> Result<Value, AppError>

The payloads keep the Rust envelope convention: the top-level CLI output contains `success`, `command`, and `data`, while the command-specific fields described here live under `data`.

## Open Questions

- UNCONFIRMED: Whether TradingView exposes a reliable column catalog suitable for `columns add --id <COLUMN_ID>`.
- UNCONFIRMED: Whether TradingView exposes a reliable default column set suitable for `columns reset --confirm-reset`.
- UNCONFIRMED: Whether the full-page Screener target refreshes visible columns immediately after a storage API save, or whether a page refresh is needed for the visible table to reflect storage-only changes.
