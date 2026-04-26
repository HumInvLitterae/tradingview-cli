# Add Screener screen actions and save

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, an operator can ask the Rust `tv` CLI which actions TradingView currently exposes in the Stock Screener screen menu, then deliberately save the active test screen through the exact visible `Save screen` / `スクリーンを保存` action. This closes the low-risk upstream PR #66 management gap without adding broader save-as, delete, rename, create, filter-add, or column-mutation flows.

The visible proof is `tv screener screens actions` returning the screen menu actions and `tv screener screens save --dry-run` resolving the exact save action without clicking. A normal `tv screener screens save` is allowed only after the dry run confirms the exact save action and the active Screener screen is a prepared test screen.

## Progress

- [x] (2026-04-26 08:55Z) Read current Screener implementation, CLI dispatch, tests, docs, continuity, and upstream PR #66 notes.
- [x] (2026-04-26 09:05Z) Added CLI and dispatch support for `tv screener screens actions` and `tv screener screens save [--dry-run]`.
- [x] (2026-04-26 09:10Z) Added Screener operation tests and CLI contract tests for actions, dry-run save, exact save clicking, and blocking-dialog failure.
- [x] (2026-04-26 09:22Z) Updated durable docs to include the new commands and remaining Screener backlog.
- [x] (2026-04-26 09:26Z) Safe live smoke passed for active screen read, action list, save dry-run, and normal save on `米国株（テスト用）`.
- [x] (2026-04-26 09:34Z) Full validation passed.
- [x] (2026-04-26 09:40Z) Committed the completed slice as `feat(screener): Add screen actions and save`.

## Surprises & Discoveries

- Observation: The current Rust Screener screen surface already has a reusable mutation session that opens the dialog, restores the original open state, and dispatches coordinate mouse events.
  Evidence: `screener_screens_list`, `screener_screens_switch`, and filter mutation tests all use this pattern successfully.

- Observation: Running multiple Screener UI commands in parallel can compete for the same transient menu and cause one CDP call to time out.
  Evidence: a parallel smoke attempt had `screens save --dry-run` succeed while `screens actions` timed out; rerunning the same commands sequentially succeeded.

- Observation: The visible Japanese screen menu exposes exact action labels for save, share, copy, rename, CSV download, create, recent, and open.
  Evidence: sequential `screens actions` smoke returned `action_count: 8`, `save_available: true`, and `save_enabled: true` on `米国株（テスト用）`.

## Decision Log

- Decision: Add `actions` rather than the upstream name `menu_actions`.
  Rationale: The Rust CLI uses user-facing command names; `actions` is shorter while still mapping to the upstream PR #66 concept in docs.
  Date/Author: 2026-04-26 / Codex.

- Decision: Implement only the exact existing-screen save action in this slice.
  Rationale: Save-as, delete, rename, create, filter-add, and column mutation are modal or catalog workflows with higher account-state risk. The exact save action is a bounded missing MVP gap and helps future Screener mutation testing.
  Date/Author: 2026-04-26 / Codex.

- Decision: Do not claim durable cloud persistence beyond the observable click and post-check.
  Rationale: TradingView may not expose a stable save completion signal. The CLI should report the action clicked and whether a blocking dialog appeared rather than overstate what CDP can prove.
  Date/Author: 2026-04-26 / Codex.

## Outcomes & Retrospective

Implementation is complete. The code, focused tests, docs, live smoke, full validation, and commit are complete.

## Context and Orientation

The Rust CLI command definitions live in `src/cli.rs`. Command dispatch lives in `src/main.rs`. Screener UI automation lives in `src/ops/screener.rs`. Screener commands use Chrome DevTools Protocol, abbreviated CDP, to evaluate small JavaScript snippets inside the running TradingView Desktop page and to dispatch mouse events.

The existing Screener commands can open and close the dialog, read visible rows and metadata, switch exact visible screens, remove filters, and list visible columns. The commands return the Rust JSON envelope, where command-specific fields are under top-level `data`.

## Plan of Work

Add `Actions` and `Save { dry_run }` variants under `ScreenerScreensCommand` in `src/cli.rs`, then dispatch them from `src/main.rs`.

In `src/ops/screener.rs`, add an action reader that opens the active screen title menu, collects visible action labels, identifies the exact save action, closes the transient menu, and restores the original Screener dialog open state. Add a save operation that uses the same menu evidence, returns target action data in dry-run mode, and in normal mode clicks only the exact save action. After clicking, read the Screener state and fail if a save-as, rename, create, copy, or delete dialog appears.

Update README, CHANGELOG, contract notes, upstream triage, Screener feasibility notes, and the next-agent handoff note so the implemented Screener surface and remaining deferred backlog stay clear.

## Concrete Steps

Run focused tests first:

    cargo test screener -- --nocapture
    cargo test --test cli_contract screener -- --nocapture

Then run the full baseline:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check

If TradingView Desktop is available, run safe smoke:

    target/debug/tv screener screens active
    target/debug/tv screener screens actions
    target/debug/tv screener screens save --dry-run

Run normal save only when the active screen title is clearly a prepared test screen and dry-run found the exact save action:

    target/debug/tv screener screens save

## Validation and Acceptance

The implementation is accepted when `tv screener screens actions` returns `source: "ui_screener_dialog"`, `scope: "screen_title_menu"`, `actions`, `save_available`, and `save_enabled`, and `tv screener screens save --dry-run` returns `action: "screen_save"`, `dry_run: true`, `clicked: false`, `save_requested: false`, and an exact `target_action` with `kind: "save"`.

Normal save is accepted when it clicks the exact save action, returns `clicked: true`, `save_requested: true`, and `confirmation: "not_observable"`, does not open a blocking modal, and preserves the active screen title. If the save action is missing or disabled, the command must fail rather than click a neighboring action.

## Idempotence and Recovery

`actions` and `save --dry-run` are read-only except for transient menu opening and closing. Normal `save` can persist the active Screener screen, so live smoke must target only a prepared test screen. If the Screener dialog or menu remains open unexpectedly, run `tv screener close` or press Escape in TradingView Desktop.

## Artifacts and Notes

Record command summaries, action counts, screen titles, and high-level result fields only. Do not paste raw Screener table rows, account-linked identifiers, or local absolute paths into tracked docs.

Safe live smoke on 2026-04-26 used the existing target whose active Screener screen was `米国株（テスト用）`. `screens active` returned that test screen. `screens actions` returned `action_count: 8` with `save_available: true` and `save_enabled: true`. `screens save --dry-run` resolved exact target action `スクリーンを保存` without clicking. Normal `screens save` clicked that exact action, returned `save_requested: true`, `confirmation: "not_observable"`, `blocking_dialog_found: false`, and preserved `after_screen_title: "米国株（テスト用）"`.

Full validation passed:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check

The tracked-doc grep for local absolute paths or `USER;` returned only existing validation-command examples in plan documents.

## Interfaces and Dependencies

At completion, `src/ops/screener.rs` exposes:

    pub async fn screener_screens_actions(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>;
    pub async fn screener_screens_save(runtime: &mut impl RuntimeEvaluator, dry_run: bool) -> Result<Value, AppError>;

The CLI exposes:

    tv screener screens actions
    tv screener screens save [--dry-run]

No new crate dependencies are required.

## Open Questions

No blocking questions remain. Future slices must separately decide whether screen save-as/delete/rename/create, filter add/modify, or column add/remove/reorder/reset belong in the core CLI.
