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
- [x] (2026-04-29) Determined that the current live environment did not expose a full-page Screener target or active saved-screen filter storage payload for add/modify schema derivation.
- [x] (2026-04-29) Recorded the storage add/modify boundary in durable docs.
- [x] (2026-04-29) Ran docs-only validation and hygiene checks.
- [ ] Commit the docs-only boundary update.

## Surprises & Discoveries

- Observation: The current live environment did not expose a full-page Screener target.
  Evidence: `tv tab list` returned `screener_target_count: 0`. The available chart target could open the Screener drawer and report the active screen as `米国株（テスト用）`, but it was not a full-page Screener target.

- Observation: Visible filters were available, but active saved-screen filter storage was not available through `window.initData.screen_data` on the chart target.
  Evidence: `tv --target-id <chart-target> screener filters list` returned 15 visible filters. A read-only `ui eval` summary returned `has_storage_url: true`, `has_version: true`, and `active_filter_count: 0` from `window.initData.screen_data`.

- Observation: Existing implementation already has a clear split: `filters remove/clear` edit saved storage, while `filters add/modify` still use UI add search, range preset clicks, and option popovers.
  Evidence: `crates/cli/src/ops/screener/filters.rs` saves storage for remove/clear through `save_screener_storage_filters`, while add/modify call UI helpers such as filter add search, range option selection, and option popover selection.

## Decision Log

- Decision: Do not implement storage-backed filter add/modify in this slice.
  Rationale: The required storage schema evidence was not available from the current live target. Implementing by guessing raw filter payload shapes would risk corrupting saved screen state and would violate the existing post-check rule.
  Date/Author: 2026-04-29 / Codex.

- Decision: Keep the existing UI-backed add/modify paths unchanged.
  Rationale: They already have validation and visible post-check boundaries. The planned replacement should only happen when a full-page Screener target or another safe read source exposes enough schema to build expected storage payloads.
  Date/Author: 2026-04-29 / Codex.

- Decision: Treat future filter storage add/modify as a release-after candidate, not a `v0.3.0` release blocker.
  Rationale: The CLI already has working commands, and the bounded audit found no safe immediate storage path. The next high-value step is release readiness unless a full-page Screener target is available for a focused follow-up.
  Date/Author: 2026-04-29 / Codex.

## Outcomes & Retrospective

This slice stayed intentionally small. It confirmed that storage-backed `filters remove/clear` and column operations remain the proven storage boundary, while `filters add/modify` still need more schema evidence before replacement. No Rust code changed, so CLI behavior is unchanged. The durable docs now make the boundary explicit and point future work toward full-page Screener storage evidence rather than more DOM retries.

## Context and Orientation

The Screener command implementation lives under `crates/cli/src/ops/screener/`. The `filters.rs` adapter contains the executable operations for `tv screener filters list/actions/add/modify/remove/clear`. The `tradingview-model` crate contains CDP-free Screener helper logic under `crates/model/src/screener/`.

In this repository, a storage-backed mutation means editing a saved screen payload fetched from TradingView's logged-in page session and saving it back through the same session. It is not an official TradingView API. It must only be used inside the user's logged-in TradingView Desktop session, and success must be verified by re-fetching the saved storage. The CLI already uses this style for Screener column storage operations and for `filters remove/clear`.

The existing proven filter storage helper is `save_screener_storage_filters` in `crates/cli/src/ops/screener/filters.rs`. It clones the active saved screen payload, replaces the `filters` array with a sanitized update payload, saves it, and requires the request to report success. `filters remove/clear` then re-fetch storage and compare the resulting filter order. That behavior is safe because it starts from existing raw filter entries and only removes entries. `filters add/modify` would need to create or edit raw filter entries, which requires schema evidence that was not available in this slice.

## Plan of Work

First, inspect the existing implementation to confirm which filter commands use storage and which still use UI. Then run read-only live commands against the available TradingView target: `tv tab list`, `tv screener screens active`, `tv screener filters list`, `tv screener filters actions`, and a gated read-only `tv ui eval` that summarizes only storage key names and filter counts. Do not paste raw filter payloads, screen ids, target ids, or local paths into tracked docs.

If the evidence shows a safe raw filter schema for numeric range add/modify or single-option modify, implement only that narrow path. The implementation must keep the existing CLI surface and prefer storage only when preflight succeeds. It must limit normal mutation to test screen names, save storage, re-fetch storage, compare expected payload/order/count, and avoid UI fallback after any storage save attempt.

If the evidence does not show a safe schema, do not change Rust code. Update `docs/internal-tradingview-apis.md`, `docs/operation-adapter-boundaries.md`, `docs/v0.3-roadmap.md`, `CHANGELOG.md`, and `CONTINUITY.md` to record that filter add/modify remain UI-backed because raw add/modify payload construction is still unconfirmed.

The current execution followed the second path.

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

Because the storage payload shape for add/modify was not available, finish this slice as docs-only:

    git diff --check
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true
    git add CHANGELOG.md docs
    git commit -m "docs(screener): Record filter storage mutation boundary"

`CONTINUITY.md` is a local ledger and is intentionally not staged. Do not push.

## Validation and Acceptance

This docs-only slice is accepted when:

- `docs/internal-tradingview-apis.md` records that filter add/modify storage replacement remains unconfirmed after bounded evidence;
- `docs/operation-adapter-boundaries.md` no longer presents Screener filter add/modify as an immediate pre-release implementation candidate;
- `docs/v0.3-roadmap.md` says release readiness may proceed with filter add/modify still UI-backed;
- `CHANGELOG.md` records the documentation boundary update;
- `git diff --check` passes;
- the hygiene grep reports only existing policy language or validation-command examples, not a new machine-specific path, cookie, token, account-local id, or raw live payload.

If any Rust code is changed unexpectedly, run:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace

No Rust code changed in this execution.

## Idempotence and Recovery

The evidence commands are read-only except that `screener screens active` and `filters list/actions` may open the Screener drawer and then restore or leave UI state according to existing command behavior. They do not mutate saved screen data. The docs update is safe to repeat. If a future run has a full-page Screener target, repeat this plan from the evidence step and update the Decision Log before implementing any storage write path.

## Artifacts and Notes

Short read-only evidence summary:

    tv tab list
    result: chart targets were available, but no full-page Screener target was reported.

    tv --target-id <chart-target> screener filters list
    result: succeeded and returned 15 visible filters on 米国株（テスト用）.

    TV_ALLOW_UNSAFE_UI_EVAL=1 tv --target-id <chart-target> ui eval "<summary>"
    result: storage URL/version flags were present, but active screen_data filters were empty on the chart target.

The tracked docs intentionally do not include the live CDP target id, raw storage payload, screen id, or account-local metadata.

Validation evidence:

    git diff --check
    result: passed.

    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true
    result: only existing policy language, archived validation-command examples, and secret-safety wording were reported. No new machine-specific path, account-local id, cookie, token, authorization value, or raw live payload was added.

## Interfaces and Dependencies

No public CLI interface changes in this slice. `tv screener filters add` and `tv screener filters modify` continue to use the existing request models and existing UI-backed implementation. Future storage-backed implementation must use the existing storage-save pattern from `filters remove/clear` and the model helpers in `tradingview-model`, but only after a future plan proves the raw add/modify schema.

## Open Questions

- UNCONFIRMED: the raw storage payload shape required to add a numeric range filter.
- UNCONFIRMED: the raw storage payload fields required to modify an existing numeric range filter.
- UNCONFIRMED: the raw storage payload fields required to change a single option filter.
- UNCONFIRMED: whether a full-page Screener target exposes enough saved-screen filter schema to support the above safely.
