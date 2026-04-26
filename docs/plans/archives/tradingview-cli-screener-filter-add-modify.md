# Add Screener filter add and modify evidence slice

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, an operator can inspect the Stock Screener filter editing UI and, when the current TradingView Desktop UI exposes safe controls, update an existing numeric range filter through the Rust `tv` CLI. This extends the current Screener support beyond read, screen selection/save, filter remove/clear, and column action discovery without jumping into high-risk saved-screen deletion or generic UI automation.

The visible proof is `tv screener filters actions` returning the active test screen's filter-editing capabilities and `tv screener filters modify --text <FILTER> --min <VALUE> --dry-run` resolving a single visible filter without mutation. Normal modification is allowed only for an existing numeric range filter on a prepared test screen and only when the visible UI exposes stable inputs and an apply action.

## Progress

- [x] (2026-04-26 12:28Z) Read current repo status, Screener CLI definitions, dispatch, operation implementation, ExecPlan requirements, and Conventional Commits rules.
- [x] (2026-04-26 12:45Z) Took live DOM evidence on the active `米国株（テスト用）` screen: 17 visible filters, add-filter button present, `EMA (21)` range filter edit UI present, and visible preset option discovery present.
- [x] (2026-04-26 13:12Z) Implemented `tv screener filters actions` and `tv screener filters modify` for existing visible numeric range presets; generic filter add remains deferred.
- [x] (2026-04-26 13:18Z) Added focused Screener operation tests and CLI contract tests.
- [x] (2026-04-26 13:30Z) Updated README, CHANGELOG, contract notes, Screener feasibility notes, upstream triage, and handoff docs.
- [x] (2026-04-26 13:44Z) Ran focused tests, full validation, safe live smoke, and tracked-doc hygiene checks.
- [x] (2026-04-26 13:50Z) Commit the completed slice.

## Surprises & Discoveries

- `tv screener filters actions` initially chose a simple `変動` filter. Opening that UI caused TradingView to normalize the visible pill from `5%` to `0% 〜 5%` during discovery, so action discovery now prefers explicit range filters such as `EMA (21)`.
- The current TradingView UI exposed a visible `0% 〜 5%` range preset for `EMA (21)`, and dry-run target resolution worked. A normal mutation attempt against `EMA (21)` did not change the visible filter text; the command failed with `internal_api_unavailable` after the post-check rather than reporting a blind success.
- The add-filter button is visible, but the add/search catalog flow was not verified in this slice.

## Decision Log

- Decision: Prioritize existing numeric filter modification before generic add or non-numeric editing.
  Rationale: Existing visible filter pills already have list/remove/clear support, and numeric range filters can be bounded by finite input validation and before/after text checks. Generic add, dropdown, and multi-select filters need broader UI evidence and are more likely to be brittle.
  Date/Author: 2026-04-26 / Codex.

- Decision: Treat normal live mutation as test-screen-only.
  Rationale: Filter changes can alter the active TradingView Screener screen. The user has provided a prepared test screen, and this plan should not mutate non-test screens.
  Date/Author: 2026-04-26 / Codex.

- Decision: Implement only preset-backed existing filter modification, not filter add.
  Rationale: The current UI evidence supports add button discovery and visible preset discovery, but not the full add/search catalog flow. Adding a new filter would require broader modal and field-type evidence.
  Date/Author: 2026-04-26 / Codex.

- Decision: Treat already-matching requested presets as a no-op instead of clicking the filter UI.
  Rationale: This avoids accidental UI normalization and keeps repeated smoke runs idempotent when the target filter already shows the requested range.
  Date/Author: 2026-04-26 / Codex.

## Outcomes & Retrospective

Implementation is complete pending final validation and commit. The slice added read-oriented filter action discovery and guarded preset-backed filter modification with dry-run target reporting, finite input validation, and visible-text post-checks. Generic filter add and non-numeric editing remain deferred.

Validation passed:

- `cargo test screener -- --nocapture`
- `cargo test --test cli_contract screener -- --nocapture`
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `git diff --check`
- tracked-doc grep for local absolute paths and `USER;`, with only existing validation-command examples in plan documents

Live smoke with an explicit TradingView target on `米国株（テスト用）` passed for `filters actions` and `filters modify --dry-run`. The normal EMA mutation did not change the visible filter and correctly failed with `internal_api_unavailable` after post-checks.

## Context and Orientation

The Rust CLI command definitions live in `src/cli.rs`. Command dispatch lives in `src/main.rs`. Screener UI automation lives in `src/ops/screener.rs`. Screener commands use Chrome DevTools Protocol, abbreviated CDP, to evaluate small JavaScript snippets inside the running TradingView Desktop page and dispatch mouse and keyboard events.

Current Screener support includes dialog reads, screen actions/list/switch/save, filter list/remove/clear, column list/actions/remove dry-run, and close. `tv screener filters list` reads visible filter pills. `tv screener filters remove` clicks one visible filter pill and its visible remove button. This plan adds only the next safe filter-management surface.

## Plan of Work

First, use the running TradingView Desktop session to inspect the current test screen. Record only high-level evidence: active screen title, filter names/counts, whether clicking a filter pill opens numeric inputs, whether a visible apply/update action exists, and whether an add/search/catalog control is discoverable. Do not record raw Screener row data, account-linked identifiers, or local absolute filesystem paths.

If the DOM evidence shows safe controls for an existing numeric range filter, add `Actions` and `Modify` variants under `ScreenerFiltersCommand` in `src/cli.rs`, dispatch them from `src/main.rs`, and implement the logic in `src/ops/screener.rs`. The selector should accept exactly one of `--index <N>` or `--text <TEXT>`. The value update should accept at least one of `--min <NUMBER>` or `--max <NUMBER>`, reject non-finite numbers before CDP connection, and report the target in dry-run mode. Normal mode should click only the resolved filter, set only the requested numeric input values, click a visible apply/update action if one is required, and verify that the filter remains visible with changed text or changed parsed bounds.

If filter add/search UI is discoverable and stable, add `Add { name, min, max, dry_run }` in the same style. If it is not stable, leave add explicitly deferred with evidence.

## Concrete Steps

Run initial evidence commands from the repository root:

    target/debug/tv screener screens active
    target/debug/tv screener filters list

If multiple TradingView chart targets are open, use `target/debug/tv tab list` and rerun the commands with `TV_CDP_TARGET_ID=<target id>`.

Run implementation-focused tests:

    cargo test screener -- --nocapture
    cargo test --test cli_contract screener -- --nocapture

Run the full baseline:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check

## Validation and Acceptance

The implementation is accepted when `tv screener filters actions` returns `source: "ui_screener_dialog"`, the active screen title, visible filter count, and detected edit/add capabilities or a clear unavailable reason. `tv screener filters modify --index <N>|--text <TEXT> --min <VALUE>|--max <VALUE> --dry-run` must resolve exactly one target filter and avoid mutation. Normal modify must only run when a safe numeric UI was detected, must verify the post-change visible filter state, and must not claim success from a blind click.

The CLI must reject missing selectors, conflicting `--index` plus `--text`, missing `--min` and `--max`, and non-finite numbers before CDP connection. If filter add is implemented, it must also reject blank names and missing numeric values before CDP connection.

## Idempotence and Recovery

Actions and modify dry-run are read-oriented except for transient menu opening and closing. Normal modify changes the active Screener screen and should only be smoked on a prepared test screen. If a mutation succeeds and should be persisted, use `tv screener screens save`; if the dialog or menu remains open unexpectedly, run `tv screener close` or press Escape in TradingView Desktop.

## Artifacts and Notes

Record command summaries, filter names/counts, action names, and high-level result fields only. Do not paste raw Screener row payloads, account-linked identifiers, or local absolute filesystem paths into tracked docs.

## Interfaces and Dependencies

At completion, if evidence supports modify, `src/ops/screener.rs` exposes:

    pub struct ScreenerFilterModifyRequest { ... }
    pub fn validate_screener_filter_modify_request(...) -> Result<ScreenerFilterModifyRequest, AppError>;
    pub async fn screener_filters_actions(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>;
    pub async fn screener_filters_modify(runtime: &mut impl RuntimeEvaluator, request: ScreenerFilterModifyRequest) -> Result<Value, AppError>;

The CLI exposes:

    tv screener filters actions
    tv screener filters modify --index <N>|--text <TEXT> --min <NUMBER>|--max <NUMBER> [--dry-run]

If evidence supports add, the CLI also exposes:

    tv screener filters add --name <TEXT> --min <NUMBER>|--max <NUMBER> [--dry-run]

No new crate dependencies are expected.

## Open Questions

CONFIRMED: The current TradingView Desktop DOM exposes preset-style numeric range controls for at least one existing `EMA (21)` filter pill, but a normal mutation attempt did not update the visible filter text in live smoke.

UNCONFIRMED: Whether filter add/search UI is stable enough for a future slice.
