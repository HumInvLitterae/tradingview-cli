# Harden watchlist DOM clicks

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root.

## Purpose / Big Picture

The Rust CLI already supports `tv watchlist add` and `tv watchlist remove`, but upstream PR #65 reports that recent TradingView Desktop / Electron builds can ignore plain DOM `.click()` calls for watchlist controls. After this change, existing watchlist commands should use browser `MouseEvent` dispatch for the relevant DOM controls and `tv watchlist add` should verify that the requested symbol is present afterward instead of returning success immediately after keyboard input.

## Progress

- [x] (2026-04-25T11:25:12Z) Confirmed the working tree was clean and compared current Rust watchlist code with upstream PR #65.
- [x] (2026-04-25T11:43:02Z) Hardened watchlist panel/add/remove DOM clicks with coordinate-based `MouseEvent` dispatch and added post-add verification.
- [x] (2026-04-25T11:43:02Z) Updated automated tests and durable docs.
- [x] (2026-04-25T11:52:41Z) Ran validation and bounded live smoke.
- [ ] Commit the completed work.

## Surprises & Discoveries

- Live smoke against one chart target showed an empty watchlist surface where `tv watchlist get` returned success but the add button was not present. A different chart target exposed the expected watchlist rows and successfully exercised add/remove.
- During live smoke, adding `NYSE:IBM` reported `before_count: 31` and `after_count: 33`, then removing `NYSE:IBM` reported `after_count: 32`. A follow-up `watchlist get` did not show `NYSE:IBM`, so the temporary symbol was cleaned up, but TradingView appeared to change or refresh the active visible watchlist contents during the test.

## Decision Log

- Decision: Do not add `watchlist add-bulk` in this slice.
  Rationale: Bulk add expands account/watchlist mutation and is operator convenience rather than a reliability fix for existing commands.
  Date/Author: 2026-04-25 / Codex

- Decision: Keep `watchlist remove` exact-match only.
  Rationale: The current Rust cleanup command intentionally avoids fuzzy deletion such as bare ticker matching because row-scoped exact deletion is safer.
  Date/Author: 2026-04-25 / Codex

- Decision: Use DOM `MouseEvent` dispatch only inside watchlist commands for now.
  Rationale: Upstream evidence is specific to watchlist controls; broad UI, alert, Pine, and tab click behavior should not be changed without separate smoke evidence.
  Date/Author: 2026-04-25 / Codex

## Outcomes & Retrospective

Implemented the watchlist reliability slice without adding new operator convenience commands. `tv watchlist add` now returns `already_present` without typing when the exact row already exists, verifies that a newly added symbol is visible before returning success, and reports the click method and before/after counts. `tv watchlist remove` remains exact-match and row-scoped, but now uses the same coordinate-based mouse event dispatch for the row remove control.

Automated validation passed:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -n 'USER'';' -- README.md docs .agents/skills || true
    git grep -nE '(/U[s]ers/|[A-Z]:\\)' -- README.md docs .agents/skills || true

Live smoke used `TV_CDP_TARGET_ID=A80F6F4622DF34104163A605015B059C` and temporary symbol `NYSE:IBM`. Add succeeded with `click_method: "mouse_event"` and `matched_after: true`; remove succeeded with `click_method: "mouse_event"` and `matched_after: false`; follow-up watchlist output did not contain `NYSE:IBM`.

## Context and Orientation

Watchlist operations live in `src/ops/layout.rs` because they share the same TradingView layout/panel surface as pane operations. `watchlist_add` opens the watchlist panel, clicks the add-symbol control, types the symbol through CDP text input, presses Enter, and returns a structured payload. `watchlist_remove` opens the panel, finds an exact `data-symbol-full` row, reveals row controls, clicks the row remove control, and verifies the row is gone.

This plan uses "DOM click" to mean JavaScript executed inside the TradingView page. A plain `element.click()` can be ignored by some React/Electron event paths; the hardened path dispatches `mousedown`, `mouseup`, and `click` mouse events at the element center.

## Plan of Work

In `src/ops/layout.rs`, replace watchlist control `.click()` calls with a local JavaScript helper that computes an element's bounding rectangle and dispatches `MouseEvent` events with `clientX` and `clientY`. Apply this to the watchlist panel button, add-symbol button, and row remove button.

Enhance `watchlist_add` so it reads existing rows before clicking the add button. If the exact requested symbol is already present, return success with `action: "already_present"` and do not send CDP text or keyboard events. Otherwise click the add control with the hardened helper, type the symbol, press Enter, dismiss the overlay with Escape, then read rows again and require the requested symbol to be present. Preserve current fields such as `symbol`, `requested_symbol`, `action`, `source`, `opened_panel`, and `add_button`, and add `before_count`, `after_count`, `matched_before`, `matched_after`, and `click_method`.

Keep `watchlist_remove` behavior exact and row-scoped. Change only the remove-button activation to use the hardened helper and report `click_method: "mouse_event"` on success.

Update docs to mark upstream PR #65 as partially addressed by click hardening while leaving bulk add deferred.

## Concrete Steps

From the repository root, implement the code and docs, then run:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -n 'USER'';' -- README.md docs .agents/skills || true
    git grep -nE '(/U[s]ers/|[A-Z]:\\)' -- README.md docs .agents/skills || true

If TradingView Desktop is available, run bounded smoke:

    target/debug/tv watchlist get
    target/debug/tv watchlist add <SYMBOL>
    target/debug/tv watchlist remove <SYMBOL>

Use a temporary symbol that is not already in the active watchlist. If every safe symbol is already present, run only the already-present add smoke and do not remove an existing user symbol.

## Validation and Acceptance

Automated acceptance is that tests prove the watchlist add and remove expressions use `MouseEvent` dispatch for watchlist controls, `watchlist_add` short-circuits already-present symbols without CDP input, `watchlist_add` rejects missing post-add confirmation, and existing CLI contract tests still pass.

Behavioral acceptance is that live smoke can add one temporary symbol and remove that same symbol afterward, or can safely prove the already-present path without deleting pre-existing watchlist data.

## Idempotence and Recovery

The implementation is additive and scoped to existing watchlist commands. Live smoke must record the temporary symbol used. If remove fails after a successful add, record the symbol and command output so the user can remove only that single leftover row.

## Artifacts and Notes

Relevant upstream evidence:

    #65 reports that TradingView Desktop / Electron can ignore synthetic `.click()` calls for watchlist controls and proposes dispatching real mouse events with coordinates.

## Interfaces and Dependencies

No new crate dependency is required. No new public command is introduced. Existing commands keep their names:

    tv watchlist add <SYMBOL>
    tv watchlist remove <SYMBOL>

## Open Questions

No critical questions are open.
