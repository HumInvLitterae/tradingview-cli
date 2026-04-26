# Harden Screener open-state detection

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, Screener commands should no longer treat the right-toolbar Screener button or unrelated right-panel content as an open Stock Screener dialog. This matters because every UI Screener command starts by reading whether the Screener panel is open. A false open state can make mutation commands skip opening the real panel and then fail later while looking for filter or screen controls.

The visible behavior is simple: `tv screener status` should report `open: false` when only the toolbar Screener button or a watchlist panel is visible, and `open: true` only when the visible Stock Screener panel root is actually present.

## Progress

- [x] (2026-04-26 14:30Z) Confirmed the current implementation can mark Screener open from broad `screener` text, class, or data-name matches.
- [x] (2026-04-26 14:42Z) Hardened `readScreenerState` to require a visible in-viewport Screener panel root.
- [x] (2026-04-26 14:44Z) Added focused tests for toolbar-only false-positive states and open failure behavior.
- [x] (2026-04-26 15:09Z) Ran focused tests, full Rust validation, diff checks, and live open/close smoke.
- [ ] Commit the completed slice.

## Surprises & Discoveries

- Observation: A table far below the viewport was still considered visible
  because the helper only checked element width and height.
  Evidence: live DOM evidence showed a table at y=2031 while the viewport
  height was far smaller, and `tv screener status` still returned `open: true`
  with null titles and zero counts.

- Observation: Adding viewport intersection and style checks to the shared
  `visible` helper made closed Screener status return `open: false`, while
  `tv screener open` still found the real panel and read filters and columns.
  Evidence: live smoke returned `open: false` after close, then `open: true`,
  `screen_title: "米国株（テスト用）"`, 17 filters, and 13 columns after open.

## Decision Log

- Decision: Limit this slice to false-positive open-state hardening.
  Rationale: The user asked to avoid spending too much cost. Broader Screener UI automation stabilization can become open-ended, while the immediate bug is the over-broad open predicate.
  Date/Author: 2026-04-26 / Codex.

- Decision: Treat off-viewport elements as not visible for Screener helpers.
  Rationale: The false positive came from an element with dimensions but no
  viewport intersection. Screener UI automation should operate only on controls
  that are currently visible to the operator.
  Date/Author: 2026-04-26 / Codex.

## Outcomes & Retrospective

The false-positive status path is fixed. `tv screener status` now reports
`open: false` when only off-panel Screener traces are visible, and `tv screener
open` still opens the real Stock Screener panel and reads the active test
screen. The implementation deliberately stops at open-state hardening; broader
Screener click and popover reliability work remains deferred.

## Context and Orientation

The Rust CLI command implementation for TradingView Stock Screener lives in `src/ops/screener.rs`. The helper named `readScreenerState` is JavaScript embedded in a Rust string. It runs inside the TradingView Desktop page through Chrome DevTools Protocol, abbreviated CDP. It returns a JSON object with fields such as `open`, `screen_title`, `filters`, `columns`, and row counts.

The current helper is too broad: it can consider any visible element with a `screener` class or data-name as proof that the Screener dialog is open. The right toolbar always contains a Screener button with data-name `screener-dialog-button`, and other panels can remain visible while the Screener panel itself is closed. The fix is to find a visible Screener panel root first, then read titles, filters, tables, columns, and rows only under that root.

## Plan of Work

In `src/ops/screener.rs`, change the JavaScript helper so `readScreenerState` uses a new `findScreenerPanelRoot` helper. That helper should ignore the toolbar button and prefer a visible container that contains Screener-specific panel content such as the topbar screen title, filter pills, or a Screener table. If no such root is found, `readScreenerState` returns `open: false`, null titles, empty filters and columns, and zero row counts.

Also update `SCREENER_OPEN_EXPRESSION` to click the toolbar Screener button using the existing JavaScript `mouseClick` helper rather than plain `button.click()`. This keeps the change small while matching the event pattern that worked better in recent live Screener evidence.

Add focused Rust unit tests around the returned state behavior that can be exercised without a live TradingView Desktop target. These tests should confirm that a toolbar-only state is not accepted as an open dialog and that `screener_open` fails when the click result still reports `open: false`.

## Concrete Steps

Run these commands from the repository root:

    cargo test screener -- --nocapture
    cargo test --test cli_contract screener -- --nocapture
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -nE '(/Users/|C:\\|USER;)' -- README.md CHANGELOG.md docs .agents/skills || true

For live smoke, use an explicit target from `tv tab list`:

    TV_CDP_TARGET_ID=<target> target/debug/tv screener status
    TV_CDP_TARGET_ID=<target> target/debug/tv screener open
    TV_CDP_TARGET_ID=<target> target/debug/tv screener filters list

Acceptance is met if the commands either read a real Screener panel or fail early with `internal_api_unavailable` when the panel is not actually visible. They must not claim `open: true` from the toolbar button alone.

## Validation and Acceptance

Automated acceptance is the focused Screener test set plus the full Rust baseline passing. Live acceptance is best-effort because it requires a running logged-in TradingView Desktop target. If live smoke still cannot open the panel reliably, record the failure and stop; do not broaden this slice into full right-panel automation.

## Idempotence and Recovery

This slice should not intentionally mutate TradingView account state. It may open or close the visible Screener panel. If a transient popover remains, press Escape or run `tv screener close`. No test filter, column, screen, or saved state should be added or removed in this slice.

## Artifacts and Notes

- Focused test commands passed:
  - `cargo test screener_open -- --nocapture`
  - `cargo test ensure_dialog_open -- --nocapture`
  - `cargo test screener -- --nocapture`
  - `cargo test --test cli_contract screener -- --nocapture`
- Full validation passed:
  - `cargo fmt --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`
  - `git diff --check`
  - tracked-doc local path / `USER;` grep, with only existing validation-command examples found
- Live smoke with an explicit target passed:
  - closed status returned `open: false`
  - `screener open` returned `open: true`, `panel_root_found: true`, 17 filters, and 13 columns
  - `screener filters list` returned 17 visible filters
  - `screener close` returned `closed: true`, followed by status `open: false`

## Interfaces and Dependencies

Do not add crate dependencies. Keep the public CLI interface unchanged. The only behavioral change should be stricter Screener open-state detection and a slightly more realistic open-button click path.

## Open Questions

No critical open questions. If the current TradingView UI no longer exposes a stable Screener panel root, stop after documenting that evidence instead of adding broader automation.
