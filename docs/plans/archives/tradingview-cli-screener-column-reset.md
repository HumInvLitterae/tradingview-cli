# Screener column reset boundary

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md`. It is self-contained so a new contributor can understand and continue the work from this file alone.

## Purpose / Big Picture

Users can already inspect, add, remove, and reorder saved Stock Screener columns on prepared test screens. The remaining column lifecycle question is whether the CLI can safely reset a saved Screener screen back to TradingView's default column set. This slice investigates that question with bounded read-only evidence and implements `tv screener columns reset [--dry-run] --confirm-reset` only if a trustworthy default source is visible.

After this slice, a future contributor should not need to guess whether reset was skipped by accident. The repository should clearly state that column reset was checked against the current full-page Screener target and remains deferred because no reliable default column source or visible reset action was found.

## Progress

- [x] (2026-04-27 15:05Z) Confirmed the working tree was clean before the slice.
- [x] (2026-04-27 15:10Z) Re-read the existing storage-backed column implementation and current Screener docs.
- [x] (2026-04-27 15:15Z) Ran read-only live evidence on the full-page Screener target for `screens active`, `columns config`, `columns actions`, and page-session storage shape.
- [x] (2026-04-27 15:20Z) Decided not to expose `columns reset` because no safe default source or visible reset action was found.
- [x] (2026-04-27 15:30Z) Updated notes, prior column-storage plan, and `CONTINUITY.md` where stale combined column-add-and-reset language remained.
- [x] (2026-04-27 15:40Z) Docs-only validation passed: `git diff --check` and tracked-doc local-path / `USER;` grep with only existing validation-command examples.
- [ ] Commit the related evidence docs.

## Surprises & Discoveries

- Observation: The active saved screen storage payload exposes only the current custom column set for column configuration.
  Evidence: The fetched active screen body had a `default_custom_column_set` array, but no separate default column set, preset column set, or reset target.
- Observation: The page-level `window.initData` shape also did not expose a default column source.
  Evidence: Relevant keys were limited to storage URL/version, settings, and `screen_data`; `screen_data` carried `active_column_set`, `default_custom_column_set`, screen id/title/version, and view/market settings.
- Observation: The visible column-management UI still does not expose a safe reset action.
  Evidence: `tv screener columns actions` returned `reset_supported: false`, and a read-only DOM text scan found column settings labels but no reset/default action text.

## Decision Log

- Decision: Do not add `tv screener columns reset` in this slice.
  Rationale: Reset would require a trustworthy default column order. Hard-coding a guessed default or copying from another screen could silently damage saved cloud state.
  Date/Author: 2026-04-27 / Codex
- Decision: Keep future reset implementation evidence-gated.
  Rationale: If TradingView later exposes a stable default source or visible reset action, the command can be revisited with the same test-screen guard and post-check rules used by add/remove/reorder.
  Date/Author: 2026-04-27 / Codex

## Outcomes & Retrospective

The slice closes as an evidence boundary rather than a code feature. `columns reset` remains deferred. The safe implemented column surface remains `columns config`, low-level storage id `columns add`, storage-backed `columns remove`, and storage-backed `columns reorder`. The next planned Screener feature surface should move to generic non-numeric filter editing, unless a later TradingView UI/storage change exposes a reliable reset source.

## Context and Orientation

The Rust CLI is implemented as the `tv` binary. The command parser lives in `src/cli.rs`, dispatch lives in `src/main.rs`, operation exports live in `src/ops.rs`, and Screener behavior lives in `src/ops/screener.rs`.

Screener commands talk to TradingView Desktop through the Chrome DevTools Protocol. A "full-page Screener target" is a separate TradingView page target whose URL is a Screener page rather than a chart page. `tv tab list` reports these targets under `screener_targets`, and a user can run follow-up commands against one by setting `TV_CDP_TARGET_ID`.

The existing column commands use TradingView's logged-in page-session saved-screen storage endpoint. `columns config` reads the active saved screen's storage column ids and params. `columns add`, `columns remove`, and `columns reorder` edit `default_custom_column_set` only for active test/disposable screen names containing `CLI-Test` or `テスト`, then re-fetch the storage payload and verify the exact id/params/order before reporting success.

## Plan of Work

Run only read-only live checks first. Use `tv tab list` to identify the full-page Screener target, then run `tv screener screens active`, `tv screener columns config`, and `tv screener columns actions` against it. Use gated `tv ui eval` only to summarize key names and counts from `window.initData` and the active saved screen storage response. Do not record raw storage payloads, account-linked identifiers, or local absolute paths in tracked docs.

If a safe default source exists, add `columns reset [--dry-run] --confirm-reset` with the same storage save and post-check strategy as add/remove/reorder. If no safe default source exists, do not modify CLI code. Instead, record the boundary in this plan, `docs/notes/screener-hotlist-upstream-feasibility-2026-04-25.md`, `docs/notes/ui-screener-read-evidence-2026-04-26.md`, `docs/notes/next-agent-handoff-prompt-2026-04-24.md`, and `CONTINUITY.md`.

## Concrete Steps

From the repository root, the read-only evidence commands are:

    tv tab list
    TV_CDP_TARGET_ID=<screener-target> tv screener screens active
    TV_CDP_TARGET_ID=<screener-target> tv screener columns config
    TV_CDP_TARGET_ID=<screener-target> tv screener columns actions

The read-only page-session summaries checked:

    window.initData keys related to screener/storage/column/default/preset
    window.initData.screen_data keys related to column/default/set/title/version
    active saved screen storage response key names and array counts
    visible DOM labels containing reset/default/column terms

Docs-only validation for this boundary slice is:

    git diff --check
    git grep -nE '(/Users/|C:\\|USER;)' -- README.md CHANGELOG.md docs .agents/skills || true

If code is touched after this point, run the full baseline:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check

## Validation and Acceptance

Acceptance for this slice is a clear durable decision. The CLI must not expose `columns reset` unless a default source is found and post-checkable. Since no source was found, acceptance is that no CLI surface is added, docs record the exact boundary, and validation reports no whitespace errors or new machine-specific paths/account-linked identifiers.

The read-only live evidence showed:

    active screen: 米国株（テスト用）
    active column set: custom
    storage column count: 13
    storage arrays: default_custom_column_set, filters, watchlists
    column-management action discovery: reset_supported false
    visible reset/default labels: none found

## Idempotence and Recovery

All live commands in this slice are read-only. `tv ui eval` was used only with read-only expressions. No storage save, UI click mutation, column reset, column add, column remove, or column reorder is part of this slice.

If a future contributor revisits reset, they should first re-run the same read-only evidence commands against a prepared test screen. Normal reset must remain guarded to `CLI-Test` or `テスト` screen names and must not report success without a post-save storage order match.

## Artifacts and Notes

The important evidence is summarized rather than copied as raw payloads:

    window.initData relevant keys: storage URL/version, screen_data, settings
    screen_data relevant keys: active_column_set, default_custom_column_set, id, title, version, market/view settings
    active saved screen storage arrays: default_custom_column_set count 13, filters count 17, watchlists count 0
    no default column set or preset column set key was visible
    no visible reset/default column-management action was found

## Interfaces and Dependencies

No new Rust interface is added in this slice. Existing column interfaces remain:

    tv screener columns config
    tv screener columns add --id <COLUMN_ID> [--params-json <JSON>] [--after-index <N>] [--dry-run]
    tv screener columns remove --index <N>|--name <TEXT> [--dry-run]
    tv screener columns reorder --from-index <N> --to-index <N> [--dry-run]

## Open Questions

- UNCONFIRMED: Whether TradingView exposes a reliable default column source in another locale, account state, or future UI build.
- UNCONFIRMED: Whether a visible reset action appears only after a specific column settings sub-panel interaction. This slice intentionally did not pursue extended UI clicking.
