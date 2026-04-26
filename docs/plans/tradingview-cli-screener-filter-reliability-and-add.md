# Stabilize Screener filter modify and evaluate add

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, an operator should know whether `tv screener filters modify` can safely perform a normal visible Screener filter mutation in the current TradingView Desktop UI. If it can, the command should complete a real preset change and verify the visible filter text. If it cannot, the command must continue to fail safely and the repository should clearly record the boundary. This slice also checks whether a minimal `tv screener filters add` command is safe to implement now.

The visible proof is a live run on the prepared `米国株（テスト用）` screen. `tv screener filters actions` and `tv screener filters modify --dry-run` should work. A normal modify should either succeed with a verified visible-text change, or fail with `internal_api_unavailable` and documented evidence showing why. Filter add should only be implemented if the add/search/catalog UI can be driven and verified without guessing.

## Progress

- [x] (2026-04-26 14:05Z) Read the current git state, previous filter modify plan, Screener evidence note, current CLI dispatch, and `src/ops/screener.rs` filter helpers.
- [x] (2026-04-26 14:30Z) Took live evidence for the current `filters modify` normal mutation path and classified the original failure as wrong-scope option discovery plus unverified normal mutation reliability.
- [x] (2026-04-26 14:40Z) Inspected the add-filter UI and decided not to implement `filters add` in this slice.
- [x] (2026-04-26 14:50Z) Applied the smallest supported code and docs changes: action discovery now scopes range options to the target filter popover and normal modify keeps the existing post-check boundary.
- [x] (2026-04-26 15:10Z) Ran focused tests, full validation, live smoke, and tracked-doc hygiene checks.
- [x] (2026-04-26 15:15Z) Commit the completed slice.

## Surprises & Discoveries

- Live `filters actions` initially reported `EMA (21)` as supporting `0% 〜 5%`, but direct DOM evidence showed that option belonged to the separate `変動` filter. The `EMA (21)` popover exposes `0% 〜 10%`, `10%以上`, and `20%以上`.
- The current TradingView UI can accept a manual click from `EMA (21): 0% 〜 10%` to `10%以上`, and the test screen was restored to `0% 〜 10%`. Repeated CLI normal mutation remained flaky, so the command still relies on visible-text post-check and fails safely instead of claiming success when the UI does not reflect the requested preset.
- The add-filter button opens a searchable filter catalog, but the add/search/range/post-add sequence was not verified end to end. `filters add` remains deferred.

## Decision Log

- Decision: Investigate and stabilize `filters modify` before adding new filter mutation surface.
  Rationale: A command that already exists but cannot complete a normal mutation in live smoke is higher priority than adding another mutation command.
  Date/Author: 2026-04-26 / Codex.

- Decision: Treat `filters add` as evidence-gated in this slice.
  Rationale: Adding filters changes the active Screener screen. It should only be implemented if the current UI exposes a stable search, selection, range preset, and post-add verification path.
  Date/Author: 2026-04-26 / Codex.

- Decision: Do not expose `filters add` in this slice.
  Rationale: The UI exposes a searchable add-filter catalog, but this pass did not verify a stable add plus numeric preset plus visible-pill post-check path. Adding CLI surface now would widen mutation risk before the existing modify reliability boundary is settled.
  Date/Author: 2026-04-26 / Codex.

- Decision: Keep normal `filters modify` guarded by post-check rather than claiming full live reliability.
  Rationale: Scoped option discovery is now more accurate, and one live manual mutation was possible, but repeated CLI normal mutation was still not reliable enough to advertise as fully stable. The command must continue to fail safely when the visible pill text does not change.
  Date/Author: 2026-04-26 / Codex.

## Outcomes & Retrospective

Implementation stabilized the evidence path but did not add new CLI surface. `filters actions` now reports range options from the target filter popover instead of accidentally reading another visible filter pill. `filters add` remains deferred. Normal `filters modify` still has a visible-text post-check and must be treated as evidence-gated in live UI; dry-run and action discovery are the reliable operator tools in this slice.

Validation passed with `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `git diff --check`. Focused `cargo test screener_filters -- --nocapture` also passed. The tracked-doc hygiene grep returned only existing validation-command examples in plan documents.

Live smoke on the prepared `米国株（テスト用）` screen showed:

- `tv screener filters actions` now reports `EMA (21)` range options as `0% 〜 10%`, `10%以上`, and `20%以上`.
- `tv screener filters modify --text "EMA (21)" --min 10 --dry-run` resolves the target without mutation.
- One live manual mutation from `0% 〜 10%` to `10%以上` succeeded, and the filter was restored to `0% 〜 10%`.
- Repeated CLI normal mutation remained flaky and failed safely with `internal_api_unavailable`; no new `filters add` surface was exposed.

## Context and Orientation

The Rust CLI command definitions live in `src/cli.rs`. Command dispatch lives in `src/main.rs`. Screener UI automation lives in `src/ops/screener.rs`. Screener commands use Chrome DevTools Protocol, abbreviated CDP, to evaluate small JavaScript snippets inside the running TradingView Desktop page and dispatch mouse and keyboard events.

The current Screener filter surface includes `tv screener filters list`, `tv screener filters actions`, `tv screener filters modify`, `tv screener filters remove`, and `tv screener filters clear`. `filters modify` resolves an existing visible filter by `--index` or unique `--text`, maps finite numeric inputs to visible preset labels such as `0% 〜 5%`, clicks the filter edit UI, and then verifies that the visible filter pill text contains the requested preset. Previous live smoke showed dry-run worked for `EMA (21)`, but normal modify did not change the visible filter text and failed safely with `internal_api_unavailable`.

## Plan of Work

First, run live evidence commands against the prepared test screen. Record only high-level command results, filter labels, action names, and availability fields. Do not write raw Screener rows, account-linked identifiers, or local absolute filesystem paths into tracked docs.

Then inspect the normal modify UI path. The key question is whether selecting a range preset immediately changes the filter pill, whether an Apply or Update button must be clicked, or whether TradingView's current filter edit UI does not accept this change through the DOM path. If an Apply or Update action is visible and stable, update `click_filter_range_preset` in `src/ops/screener.rs` to click it and then keep the existing visible-text post-check. If no stable action exists, keep the normal command failure behavior and improve the evidence notes.

Finally, inspect the add-filter UI. If a stable minimal path exists, add a `ScreenerFiltersCommand::Add` variant with `--name`, `--min`, `--max`, and `--dry-run`, validate finite preset inputs before CDP connection, implement target reporting in dry-run, and verify that a new visible filter pill appears after normal add. If the add UI is unstable or not clearly verifiable, do not add CLI surface in this slice.

## Concrete Steps

Run live evidence commands from the repository root:

    TV_CDP_TARGET_ID=<target> target/debug/tv screener screens active
    TV_CDP_TARGET_ID=<target> target/debug/tv screener filters list
    TV_CDP_TARGET_ID=<target> target/debug/tv screener filters actions
    TV_CDP_TARGET_ID=<target> target/debug/tv screener filters modify --text "EMA (21)" --min 0 --max 5 --dry-run

If multiple TradingView targets are open, use `target/debug/tv tab list` and rerun with the intended target id in `TV_CDP_TARGET_ID`.

Run focused tests:

    cargo test screener -- --nocapture
    cargo test --test cli_contract screener -- --nocapture

Run the full baseline:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check

## Validation and Acceptance

The slice is accepted when the repo clearly records whether normal `filters modify` is supported in the current UI. If supported, `tv screener filters modify --text "EMA (21)" --min 0 --max 5` must return success only after the visible filter text changes, and a restore smoke should move the test filter back to its starting preset when possible. If unsupported, the command must continue to fail safely and docs must explain that dry-run and action discovery are the reliable parts.

If `filters add` is implemented, `tv screener filters add --name <TEXT> --min <N>|--max <N> --dry-run` must resolve the intended add action without mutation, and normal add must verify a visible filter pill after mutation. If not implemented, no new add CLI variant should be exposed.

## Idempotence and Recovery

Use only the prepared `米国株（テスト用）` screen for normal mutation smoke. If a normal mutation changes a filter, restore it when a safe reverse preset is available. If a popover remains open, run `tv screener close` or press Escape in TradingView Desktop. If a test filter remains added, leave enough evidence in this plan to identify what was added.

## Artifacts and Notes

Keep evidence concise. Record command names, success or error kind, filter counts, visible filter labels, and whether apply/add actions were found. Do not paste raw row payloads, account-linked identifiers, or local absolute filesystem paths.

## Interfaces and Dependencies

Keep using the existing `RuntimeEvaluator`, `AppError`, and `serde_json::Value` patterns. Do not add crate dependencies. If implemented, `filters add` must be exposed through `src/cli.rs`, dispatched from `src/main.rs`, re-exported from `src/ops.rs`, and implemented in `src/ops/screener.rs` beside the existing filter helpers.

## Open Questions

UNCONFIRMED: Whether current TradingView Desktop requires an Apply or Update action after selecting a numeric range preset.

UNCONFIRMED: Whether the add-filter search/catalog UI exposes a stable path for adding a numeric range filter.
