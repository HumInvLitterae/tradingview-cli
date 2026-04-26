# Add storage-backed Screener filter removal and clear

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

The current Screener filter mutation commands work through visible filter pills and popovers. That is useful, but it is fragile because localized labels, stale popovers, and timing can break the click path. Previous Screener work became more reliable when column and screen-delete mutations moved to TradingView's saved Screener storage API, which is the logged-in page-session endpoint discovered from `window.initData`.

After this change, `tv screener filters remove` and `tv screener filters clear` should use the saved-screen storage payload when they can do so safely. A user operating on a prepared test screen such as `米国株（テスト用）` should be able to dry-run the target, run the mutation, and see the command report success only after the saved storage payload no longer contains the removed filters.

## Progress

- [x] (2026-04-26 17:26Z) Reviewed the current filter DOM mutation path and existing storage-backed column helpers.
- [x] (2026-04-26 17:42Z) Implemented storage-backed helpers for saved-screen filters.
- [x] (2026-04-26 17:47Z) Routed `filters remove` and `filters clear` normal modes through storage-backed mutation when the active screen is a test/disposable screen.
- [x] (2026-04-26 18:05Z) Added best-effort full-page Screener refresh after storage filter saves.
- [x] (2026-04-26 18:18Z) Updated focused tests, README, CHANGELOG, contract notes, handoff notes, and internal API reference.
- [x] (2026-04-26 18:28Z) Ran full automated validation.
- [x] (2026-04-26 18:20Z) Ran bounded live smoke on the full-page test Screener target.
- [x] (2026-04-26 18:38Z) Updated `CONTINUITY.md`.
- [ ] Commit the related changes without pushing.

## Surprises & Discoveries

- Observation: Saving the active screen storage `filters` array does not immediately update the already-rendered full-page Screener filter pills.
  Evidence: A live `filters remove --text ベータ` storage save succeeded and the re-fetched storage count changed from 17 to 16, but `filters list` still showed 17 visible filters until the full-page Screener target was reloaded.

- Observation: A full-page Screener reload after storage save makes the visible filter list reflect the saved storage payload.
  Evidence: A read-only reload check followed by `filters list` showed 16 visible filters and no `ベータ` filter.

## Decision Log

- Decision: Start with `filters remove` and `filters clear`, not `filters add` or `filters modify`.
  Rationale: Remove and clear can preserve TradingView's existing filter payloads and only delete array entries. Add and modify require constructing or editing filter payload internals and need more schema evidence.
  Date/Author: 2026-04-26 / Codex

- Decision: Normal storage-backed filter mutations remain limited to test/disposable screen names containing `CLI-Test` or `テスト`.
  Rationale: Saved-screen filters are account state. This mirrors the existing storage-backed column guard and prevents accidental production-screen mutation.
  Date/Author: 2026-04-26 / Codex

- Decision: After storage-backed filter saves, request a best-effort reload only for full-page Screener targets.
  Rationale: The storage payload is authoritative after re-fetch, but the visible UI can stay stale. Reloading a full-page Screener target is bounded and restores operator visibility. Reloading a chart-side drawer target would be too disruptive, so that path reports storage success without forcing a page reload.
  Date/Author: 2026-04-26 / Codex

## Outcomes & Retrospective

The implementation now uses the saved-screen storage API for normal `filters remove` and `filters clear`. Dry-run behavior remains visible-target based and non-mutating. Normal mode is guarded to test/disposable screen names, verifies visible/storage filter count alignment before saving, re-fetches storage after saving, and requests a full-page refresh when available. `filters add` and `filters modify` remain DOM/post-check guarded.

## Context and Orientation

The Rust CLI binary is `tv`. Screener command parsing lives in `src/cli.rs`, command dispatch lives in `src/main.rs`, and Screener operations live in `src/ops/screener.rs`.

A "visible filter" means a filter pill shown in the current TradingView Screener UI. The current `filters remove` and `filters clear` commands resolve visible filter pills, click each pill, find a remove button in a popover, and wait until the visible filter list changes.

A "storage filter" means an entry in the active saved Screener screen's `filters` array returned by TradingView's saved-screen storage endpoint. This is not a public TradingView API. The CLI accesses it only inside the user's logged-in TradingView Desktop page session. The existing function `fetch_active_screener_storage_config` already fetches the active saved screen payload for column operations, and `save_screener_storage_columns` already writes a modified saved screen back through the page session.

The safest first storage-backed filter mutation is to remove existing filter entries by index. The command will still resolve the user target through visible filters, then map that target index to the saved storage filter at the same index only when the visible filter count and storage filter count agree. It will not synthesize new filter payloads.

## Plan of Work

First, extend `src/ops/screener.rs` with a small storage filter target type and helpers beside the existing storage column helpers. Add helpers to extract storage filters from the fetched config, build public-safe output payloads for filters without raw internals, remove one storage filter by index, clear all storage filters, and compare expected versus re-fetched storage filter arrays.

Second, replace the normal-mode internals of `screener_filters_remove` and `screener_filters_clear`. Dry-run behavior should remain visible-target based and non-mutating. Normal mode should read the active screen title, fetch active storage, require a test/disposable screen name, verify visible and storage filter counts match, update the saved screen's `filters` array, save the whole screen payload, re-fetch storage, and report success only if the storage array matches the expected post-mutation state. The output should clearly say `scope: "screen_storage_api"` and keep target filters public-safe. On a full-page Screener target, request a page reload after the storage post-check and report whether visible filter count was confirmed.

Third, keep `filters add` and `filters modify` unchanged in this slice. Those commands require schema-aware creation or editing and remain DOM/post-check guarded until a separate plan proves a safe storage schema.

Fourth, update docs. `README.md` and `CHANGELOG.md` should mention storage-backed filter remove/clear. `docs/internal-tradingview-apis.md` should move `screener filters remove/clear` from audit candidates to storage/API-backed. The prior audit plan should record that the first follow-up adopted remove/clear only, while add/modify remain deferred.

## Concrete Steps

From the repository root, inspect state:

    git status --short

Edit `src/ops/screener.rs`:

- Add `ScreenerStorageFilterTarget`.
- Add `storage_filters_from_config`, `storage_filter_target_payload`, `storage_filter_targets_payload`, `storage_filter_update_payload`, `ensure_storage_filter_index`, `ensure_storage_filter_alignment`, `remove_storage_filter`, `storage_filter_order_matches`, and `save_screener_storage_filters`.
- Update `screener_filters_remove` normal mode to use storage-backed removal.
- Update `screener_filters_clear` normal mode to use storage-backed clear.
- Keep dry-run branches non-mutating.

Update tests in `src/ops/screener.rs` and CLI contract tests only if command help or validation changes. This slice should not require new CLI flags.

Update user-facing docs and notes.

## Validation and Acceptance

Run focused tests first:

    cargo test screener_filter -- --nocapture
    cargo test screener -- --nocapture
    cargo test --test cli_contract screener -- --nocapture

Run the full baseline:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -nE '(/Users/|C:\\|USER;)' -- README.md CHANGELOG.md docs .agents/skills || true

Acceptance is that `filters remove` and `filters clear` still validate selectors and confirmations as before, dry-run remains non-mutating, normal mode on non-test screen names is rejected before saving, and normal mode reports success only after a storage re-fetch confirms the expected filters array.

For live smoke, prefer the full-page Screener target for `米国株（テスト用）`. First run read-only and dry-run commands. If the active screen is the test screen and a disposable filter is present or can be safely removed, run one normal storage-backed remove. Do not run clear-all unless the visible filter set is intentionally disposable.

## Idempotence and Recovery

Dry-run commands are safe to repeat. Normal mutation is limited to test/disposable screen names. If a save succeeds but visible UI does not immediately refresh, use storage re-fetch as the authoritative post-check and record that the UI may need refresh. If post-check fails, return `internal_api_unavailable` and include expected and observed public-safe filter summaries.

If a live smoke leaves unwanted test filters, use `tv screener filters remove --index <N>` or restore the test screen manually. Do not mutate non-test screens.

## Artifacts and Notes

Focused validation:

    cargo test screener_filter -- --nocapture
    # Passed: 21 tests.

    cargo test --test cli_contract screener -- --nocapture
    # Passed: 6 tests.

Full validation:

    cargo test screener -- --nocapture
    # Passed: 66 tests.

    cargo test --test cli_contract screener -- --nocapture
    # Passed: 6 tests.

    cargo fmt --check
    # Passed.

    cargo clippy --all-targets --all-features -- -D warnings
    # Passed.

    cargo test
    # Passed: 318 unit tests and 80 CLI contract tests.

    git diff --check
    # Passed.

    tracked-doc local path / USER marker grep
    # Returned only existing validation-command examples in older plan documents.

Live smoke on full-page `米国株（テスト用）`:

    tv screener filters remove --text ベータ --dry-run
    # Resolved one visible target at index 11 and did not mutate.

    tv screener filters remove --text ベータ
    # Saved storage filters from 17 to 16, re-fetched storage successfully, and removed the target from saved storage.

    tv screener filters list
    # Immediately after storage save, visible UI still showed 17 filters.

    full-page reload followed by tv screener filters list
    # Visible UI showed 16 filters. The removed test-screen filter was ベータ.

    tv screener filters remove --text 相対出来高 --dry-run
    # Resolved one visible target at index 15 and did not mutate.

    tv screener filters remove --text 相対出来高
    # Saved storage filters from 16 to 15 and returned visible_refresh.confirmed: true.

    tv screener filters list
    # Visible UI showed 15 filters. The removed test-screen filter was 相対出来高.

## Interfaces and Dependencies

No new CLI flags are introduced.

The existing public functions remain:

    pub async fn screener_filters_remove(
        runtime: &mut impl RuntimeEvaluator,
        selector: ScreenerFilterSelector,
        dry_run: bool,
    ) -> Result<Value, AppError>

    pub async fn screener_filters_clear(
        runtime: &mut impl RuntimeEvaluator,
        dry_run: bool,
        confirm_clear: bool,
    ) -> Result<Value, AppError>

New private helpers should stay in `src/ops/screener.rs` near the existing storage column helpers.

## Open Questions

- UNCONFIRMED: whether TradingView immediately refreshes visible filter pills after storage-backed filter saves.
- UNCONFIRMED: whether every Screener storage payload keeps visible filter order identical to storage filter order. This slice requires equal counts and index-based alignment before saving.
