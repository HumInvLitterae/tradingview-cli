# Screener filter storage mutation audit

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md` in this repository. It is self-contained and describes a bounded feasibility slice for replacing Screener filter add/modify UI operations with saved-screen storage mutations.

## Purpose / Big Picture

`tv screener filters add` and `tv screener filters modify` still use visible Screener UI elements and popovers. That is more fragile than storage-backed commands such as `filters remove/clear` and `columns add/remove/reorder`. The goal of this slice is to check whether the saved Screener storage payload contains enough public-safe, testable structure to update filter add/modify directly. If it does, the CLI should prefer storage-backed add/modify and verify by re-fetching saved storage. If it does not, the repository should record the boundary and return to `v0.3.0` release readiness without adding another UI retry layer.

The observable outcome is either a safer implementation for the existing `tv screener filters add/modify` commands, or a durable note that explains why they remain UI-backed for now.

## Progress

- [x] (2026-04-29) Created this ExecPlan and archived the completed operation adapter boundary audit plan.
- [x] (2026-04-29) Inspected existing filter add/modify/remove/clear implementation and model helpers.
- [x] (2026-04-29) Ran bounded read-only live evidence against the available chart target.
- [x] (2026-04-29) Determined that the initial live environment did not expose a full-page Screener target or active saved-screen filter storage payload for add/modify schema derivation.
- [x] (2026-04-29) Re-ran evidence after the user opened a full-page Screener target.
- [x] (2026-04-29) Implemented storage-backed range modification for simple `Condition` filters selected by index.
- [x] (2026-04-29) Confirmed focused model and Screener modify tests.
- [x] (2026-04-29) Ran live dry-run and normal smoke on the test screen, then restored the modified range to its original value.
- [x] (2026-04-29) Ran full validation baseline and hygiene checks.
- [ ] Commit the implementation and docs update.

## Surprises & Discoveries

- Observation: The current live environment did not expose a full-page Screener target.
  Evidence: `tv tab list` returned `screener_target_count: 0`. The available chart target could open the Screener drawer and report the active screen as `米国株（テスト用）`, but it was not a full-page Screener target.

- Observation: Visible filters were available, but active saved-screen filter storage was not available through `window.initData.screen_data` on the chart target.
  Evidence: `tv --target-id <chart-target> screener filters list` returned 15 visible filters. A read-only `ui eval` summary returned `has_storage_url: true`, `has_version: true`, and `active_filter_count: 0` from `window.initData.screen_data`.

- Observation: Existing implementation already has a clear split: `filters remove/clear` edit saved storage, while `filters add/modify` still use UI add search, range preset clicks, and option popovers.
  Evidence: `crates/cli/src/ops/screener/filters.rs` saves storage for remove/clear through `save_screener_storage_filters`, while add/modify call UI helpers such as filter add search, range option selection, and option popover selection.

- Observation: A full-page Screener target exposes active saved-screen filters with enough structure for simple range modification.
  Evidence: a read-only `ui eval` summary reported 15 active filters. Simple numeric filters used `type: Condition`, `operation.type` values such as `above` or `between`, and `right.value` or `right.left/right.right` fields.

- Observation: Add and option modify still do not have enough safe storage evidence.
  Evidence: `window.initData` exposed the active saved screen and its current filters, but no filter catalog or option-value mapping that would allow constructing a new filter entry or translating a localized option label into a raw storage value.

## Decision Log

- Decision: Implement only storage-backed `filters modify --min/--max` for simple `Condition` filters selected by index.
  Rationale: That case can be built by cloning an existing raw filter entry, replacing only its `operation` and `right` range fields, saving the active screen, and re-fetching storage for post-check. Text selectors, add, option modify, and complex operations still lack enough safe schema.
  Date/Author: 2026-04-29 / Codex.

- Decision: Keep existing UI-backed paths as pre-save fallback for unsupported or unavailable storage cases.
  Rationale: Chart-side Screener targets and non-index selectors still need the existing visible UI path. After a storage save attempt, the command must not fall back to UI because the saved state may already have changed.
  Date/Author: 2026-04-29 / Codex.

- Decision: Treat `filters add` and `filters modify --option` storage replacement as release-after candidates, not `v0.3.0` blockers.
  Rationale: Their existing UI-backed commands still work with post-checks, and raw storage construction remains unproven.
  Date/Author: 2026-04-29 / Codex.

## Outcomes & Retrospective

This slice stayed intentionally narrow. It adds a storage-backed path for `filters modify --min/--max` when the target is an index-selected simple saved-storage `Condition` filter. `filters add` and `filters modify --option` remain UI-backed. The implementation preserves the existing CLI surface and uses storage only when preflight succeeds before any mutation; post-save failures return `internal_api_unavailable` rather than falling back to UI.

## Context and Orientation

The Screener command implementation lives under `crates/cli/src/ops/screener/`. The `filters.rs` adapter contains the executable operations for `tv screener filters list/actions/add/modify/remove/clear`. The `tradingview-model` crate contains CDP-free Screener helper logic under `crates/model/src/screener/`.

In this repository, a storage-backed mutation means editing a saved screen payload fetched from TradingView's logged-in page session and saving it back through the same session. It is not an official TradingView API. It must only be used inside the user's logged-in TradingView Desktop session, and success must be verified by re-fetching the saved storage. The CLI already uses this style for Screener column storage operations and for `filters remove/clear`.

The existing proven filter storage helper is `save_screener_storage_filters` in `crates/cli/src/ops/screener/filters.rs`. It clones the active saved screen payload, replaces the `filters` array with a sanitized update payload, saves it, and requires the request to report success. `filters remove/clear` then re-fetch storage and compare the resulting filter order. That behavior is safe because it starts from existing raw filter entries and only removes entries. `filters add/modify` would need to create or edit raw filter entries, which requires schema evidence that was not available in this slice.

## Plan of Work

First, inspect the existing implementation to confirm which filter commands use storage and which still use UI. Then run read-only live commands against the available TradingView target: `tv tab list`, `tv screener screens active`, `tv screener filters list`, `tv screener filters actions`, and a gated read-only `tv ui eval` that summarizes only storage key names and filter counts. Do not paste raw filter payloads, screen ids, target ids, or local paths into tracked docs.

If the evidence shows a safe raw filter schema for numeric range add/modify or single-option modify, implement only that narrow path. The implementation must keep the existing CLI surface and prefer storage only when preflight succeeds. It must limit normal mutation to test screen names, save storage, re-fetch storage, compare expected payload/order/count, and avoid UI fallback after any storage save attempt.

If the evidence does not show a safe schema, do not change Rust code. Update `docs/internal-tradingview-apis.md`, `docs/operation-adapter-boundaries.md`, `docs/v0.3-roadmap.md`, `CHANGELOG.md`, and `CONTINUITY.md` to record that filter add/modify remain UI-backed because raw add/modify payload construction is still unconfirmed.

The current execution followed a partial implementation path: simple range modify is now storage-backed, while add and option modify remain UI-backed.

## Concrete Steps

The read-only evidence commands used in this slice were:

    tv tab list
    tv --target-id <chart-target> screener screens active
    tv --target-id <chart-target> screener filters list
    tv --target-id <chart-target> screener filters actions
    TV_ALLOW_UNSAFE_UI_EVAL=1 tv --target-id <chart-target> ui eval "<read-only storage summary expression>"

The important observed facts were:

    screener_target_count: 0
    active screen title from Screener drawer: 米国株（テスト用）
    visible filter count: 15
    storage URL/version flags were present
    active saved-screen filter count from window.initData.screen_data: 0

The full-page Screener follow-up used the same evidence pattern and then ran focused tests and live smoke. The normal smoke temporarily changed filter index 2 from a `0%` to `10%` range and then restored it to `0%` to `5%`.

Finish this slice with the implementation validation baseline:

    cargo test -p tradingview-cli screener -- --nocapture
    cargo test -p tradingview-model screener::filters -- --nocapture
    cargo test -p tradingview-cli --test cli_contract screener -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true
    git add CHANGELOG.md docs crates/cli/src/ops/screener/engine.rs crates/cli/src/ops/screener/filters.rs crates/model/src/screener/filters.rs
    git commit -m "fix(screener): Use storage-backed range filter edits"

`CONTINUITY.md` is a local ledger and is intentionally not staged. Do not push.

## Validation and Acceptance

This slice is accepted when:

- `tv screener filters modify --index <N> --min/--max` can use `scope: "screen_storage_api"` for simple saved-storage `Condition` filters;
- storage-backed modify succeeds only after a re-fetch post-check;
- unsupported storage schema, text selectors, and unavailable storage preflight fall back to the existing UI path before save;
- post-save post-check failure does not fall back to UI;
- `filters add` and `filters modify --option` remain UI-backed and documented as future candidates;
- `CHANGELOG.md` records the documentation boundary update;
- `git diff --check` passes;
- the hygiene grep reports only existing policy language or validation-command examples, not a new machine-specific path, cookie, token, account-local id, or raw live payload.

## Idempotence and Recovery

The evidence commands are read-only except that `screener screens active` and `filters list/actions` may open the Screener drawer and then restore or leave UI state according to existing command behavior. The normal smoke mutates only the test screen and restores the changed range in the same run. If the restore fails, record the remaining visible filter text in `CONTINUITY.md` and do not write account-local ids or raw payloads into tracked docs.

## Artifacts and Notes

Short read-only evidence summary:

    tv tab list
    initial result: chart targets were available, but no full-page Screener target was reported.
    follow-up result: one full-page Screener target was reported for the test screen.

    tv --target-id <chart-target> screener filters list
    result: succeeded and returned 15 visible filters on 米国株（テスト用）.

    TV_ALLOW_UNSAFE_UI_EVAL=1 tv --target-id <chart-target> ui eval "<summary>"
    result: storage URL/version flags were present, but active screen_data filters were empty on the chart target.

    TV_ALLOW_UNSAFE_UI_EVAL=1 tv --target-id <screener-target> ui eval "<summary>"
    result: active screen_data filters were available, including simple Condition filters with above/between operations.

    cargo run -p tradingview-cli -- --target-id <screener-target> screener filters modify --index 2 --min 0 --max 5
    result: success with scope screen_storage_api; a follow-up read-only eval confirmed index 2 was between 0 and 5.

The tracked docs intentionally do not include the live CDP target id, raw storage payload, screen id, or account-local metadata.

Validation evidence:

    cargo test -p tradingview-model screener::filters -- --nocapture
    result: passed for the storage range helper tests.

    cargo test -p tradingview-cli screener_filters_modify -- --nocapture
    result: passed for existing UI modify tests and new storage modify tests.

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    result: passed.

    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true
    result: reported only existing policy language, archived validation-command examples, and this plan's validation command. No new machine-specific path, account-local id, cookie, token, or authorization value was added.

## Interfaces and Dependencies

No public CLI interface changes in this slice. `tv screener filters modify --min/--max` now tries storage-backed mutation first only for index-selected simple `Condition` filters. It rewrites `operation` and `right` fields in a cloned saved filter entry. The storage path uses `fetch_current_screener_storage_config`, `replace_storage_filter_range`, `save_screener_storage_filters`, and a storage re-fetch post-check. `tv screener filters add` and `tv screener filters modify --option` continue to use the existing UI-backed implementation.

## Open Questions

- UNCONFIRMED: the raw storage payload shape required to add a numeric range filter.
- UNCONFIRMED: the raw storage payload fields required to change a single option filter.
- UNCONFIRMED: whether a full-page Screener target exposes a safe catalog or option mapping source for the remaining add/option cases.
