# Add Screener column management

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, an operator can inspect the Stock Screener column-management UI and resolve a visible column removal target through the Rust `tv` CLI. This fills the next practical Screener gap after read, screen switch/save, and filter cleanup support while keeping normal mutation evidence-gated. The command must remain safer than broad UI automation: it should identify a single visible column, support dry-run, avoid claiming unsupported mutation, and avoid save-as, delete, rename, create, filter-add, and column reorder flows.

The visible proof is `tv screener columns actions` returning detected column-management state and `tv screener columns remove --name <COLUMN> --dry-run` resolving one visible column without mutation. A normal remove is allowed only if the running TradingView Desktop UI exposes a safe per-column remove action.

## Progress

- [x] (2026-04-26 10:06Z) Read current repo status, Screener CLI definitions, dispatch, implementation, continuity state, and ExecPlan requirements.
- [x] (2026-04-26) Took live DOM evidence on the active `米国株（テスト用）` screen: 13 visible columns, column settings categories/search/add-column configuration, and header sort/move menu evidence; no safe visible per-column remove action was found.
- [x] (2026-04-26) Implemented `tv screener columns actions` plus `tv screener columns remove --index <N>|--name <TEXT> --dry-run`; normal remove returns `internal_api_unavailable` when the UI exposes no safe remove action.
- [x] (2026-04-26) Added focused Screener operation tests and CLI contract tests.
- [x] (2026-04-26) Updated README, CHANGELOG, contract notes, Screener feasibility notes, upstream triage, and handoff docs.
- [x] (2026-04-26) Ran focused tests, full validation, safe live smoke, and tracked-doc hygiene checks.
- [ ] Commit the completed slice.

## Surprises & Discoveries

- The table's sticky header button opens the column settings UI and exposes categories such as `銘柄情報26`, `マーケットデータ37`, and `テクニカル39`.
- Searching and selecting `EMA (指数移動平均)` opens an add-column configuration surface, not a remove flow for the existing `EMA (21)` visible column.
- The visible header context menu exposes sort and move actions, but no remove/hide action in the current TradingView Desktop session.
- The implementation should not synthesize a delete action from unsupported UI; dry-run target resolution is useful, while normal remove must remain unavailable until a safe action is observed.

## Decision Log

- Decision: Prioritize column actions and remove before add, reorder, or saved-screen management.
  Rationale: Column removal is a common Screener cleanup workflow and can be bounded by visible-column targeting plus post-removal verification. Add/reorder and screen save-as/delete/rename/create need broader modal or drag/drop evidence.
  Date/Author: 2026-04-26 / Codex.

- Decision: Treat normal live mutation as test-screen-only.
  Rationale: Column changes can alter a saved TradingView Screener screen. The user has provided test Screener screens, and this plan should not mutate non-test screens.
  Date/Author: 2026-04-26 / Codex.

- Decision: Ship dry-run target resolution but defer normal column remove.
  Rationale: Live DOM evidence found settings categories, search, add-column configuration, and header sort/move actions, but no safe visible per-column remove action. Returning `remove_supported: false` is safer than attempting a hidden or inferred mutation.
  Date/Author: 2026-04-26 / Codex.

## Outcomes & Retrospective

The CLI now exposes the column action and dry-run remove target surface, with normal remove intentionally unavailable until TradingView exposes a verified safe remove action. Live smoke on `米国株（テスト用）` returned 13 visible columns, eight column-setting categories, `remove_supported: false`, and a successful dry-run target for `EMA (21)` without changing the screen.

Validation passed:

    cargo test screener -- --nocapture
    cargo test --test cli_contract screener -- --nocapture
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check

Tracked-doc grep for local absolute paths and `USER;` produced only existing validation-command examples in plan documents.

## Context and Orientation

The Rust CLI command definitions live in `src/cli.rs`. Command dispatch lives in `src/main.rs`. Screener UI automation lives in `src/ops/screener.rs`. Screener commands use Chrome DevTools Protocol, abbreviated CDP, to evaluate small JavaScript snippets inside the running TradingView Desktop page and dispatch mouse events.

Current Screener support includes dialog reads, screen actions/list/switch/save, filter list/remove/clear, column list, and close. `tv screener columns list` currently reads table header text only; it does not know how to open a column menu or mutate columns. The Rust JSON envelope stores command-specific payload under top-level `data`.

## Plan of Work

First, use the running TradingView Desktop session to inspect the current test screen. Record only high-level evidence: active screen title, column names/counts, and whether a visible column menu or column-management control is discoverable. Do not record raw row data, account-linked identifiers, or local absolute paths.

If the DOM evidence shows a reliable visible-column menu with a remove action, add normal remove. The evidence gathered in this slice did not show that action, so `Actions` and `Remove` variants were added with `--dry-run` target resolution and a normal-mode `internal_api_unavailable` result that includes the target and visible columns.

If a reset/default action is discoverable and can be verified safely, add `Reset { dry_run, confirm_reset }`. If not, leave reset deferred in this plan with the DOM evidence that blocked it.

## Concrete Steps

Run initial evidence commands:

    target/debug/tv screener screens active
    target/debug/tv screener columns list

If multiple TradingView chart targets are open, use `tv tab list` and run the commands with `TV_CDP_TARGET_ID=<target id>`.

Run implementation-focused tests:

    cargo test screener -- --nocapture
    cargo test --test cli_contract screener -- --nocapture

Run the full baseline:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check

## Validation and Acceptance

The implementation is accepted when `tv screener columns actions` returns `source: "ui_screener_dialog"`, the active screen title, visible column count, detected column-management categories, and a clear unavailable reason when remove/reset actions are absent. `tv screener columns remove --index <N>|--name <TEXT> --dry-run` must resolve exactly one target column and avoid mutation. Normal remove must not click unless a visible safe remove action is found; in the current evidence state it reports `internal_api_unavailable` instead of pretending to mutate.

The CLI must reject missing selectors and conflicting `--index` plus `--name` before CDP connection. If reset is implemented, normal reset must require `--confirm-reset`; otherwise reset remains explicitly deferred.

## Idempotence and Recovery

Actions and remove dry-run are read-oriented except for transient menu opening and closing. Normal remove changes the active Screener screen and should only be smoked on a prepared test screen. If a mutation succeeds and should be persisted, use `tv screener screens save`; if the dialog or menu remains open unexpectedly, run `tv screener close` or press Escape in TradingView Desktop.

## Artifacts and Notes

Record command summaries, column names/counts, action names, and high-level result fields only. Do not paste raw Screener row payloads, account-linked identifiers, or local absolute filesystem paths into tracked docs.

## Interfaces and Dependencies

At completion, `src/ops/screener.rs` exposes:

    pub enum ScreenerColumnSelector { Index(usize), Name(String) }
    pub fn validate_screener_column_selector(index: Option<usize>, name: Option<&str>) -> Result<ScreenerColumnSelector, AppError>;
    pub async fn screener_columns_actions(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>;
    pub async fn screener_columns_remove(runtime: &mut impl RuntimeEvaluator, selector: ScreenerColumnSelector, dry_run: bool) -> Result<Value, AppError>;

The CLI exposes:

    tv screener columns actions
    tv screener columns remove --index <N>|--name <TEXT> [--dry-run]

No new crate dependencies are expected.

## Open Questions

Resolved for current evidence: the current TradingView Desktop DOM did not expose a reliable per-column remove action.

Resolved for current evidence: no reset/default action was visible and safe enough for this slice.
