# Desktop app-window helper split

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md` in this repository.

## Purpose / Big Picture

`tv tab ...` and `tv screener open --full-page` both need to operate TradingView Desktop's application window: read app tabs, click the create-new-tab button, and wait for the Desktop new-tab page target. Before this change, that logic was split between `crates/cli/src/ops/tab.rs` and `crates/cli/src/ops/screener/state.rs`. After this change, those shared app-window operations live in one internal helper module, while the public CLI behavior remains unchanged.

This is a release-readiness refactor. It reduces duplication around a brittle Desktop UI boundary before `v0.3.0` release preparation.

## Progress

- [x] (2026-04-29) Created this ExecPlan and archived the completed Screener full-page open plan.
- [x] (2026-04-29) Added `crates/cli/src/ops/desktop.rs` for shared app-window tab helpers.
- [x] (2026-04-29) Updated `tab` and `screener open --full-page` code to use the shared helper.
- [x] (2026-04-29) Ran focused `desktop`, `tab`, and `screener::state` tests.
- [x] (2026-04-29) Updated stable docs and local continuity ledger.
- [x] (2026-04-29) Ran full validation and live smoke.
- [ ] Commit the related changes.

## Surprises & Discoveries

- Observation: The overlap is narrower than the full Screener opener. Only app-window tab reading, create-new-tab clicking, and new-tab target waiting should be shared.
  Evidence: Screener still owns the Stock Screener tile click expression because that is product-specific, while `tab` still owns chart-tab payload shaping.

## Decision Log

- Decision: Add an internal `ops::desktop` helper instead of creating a new workspace crate.
  Rationale: The code depends on CDP runtime evaluation and TradingView Desktop app-window DOM, so it is an executable adapter helper rather than a reusable I/O-free model or service crate.
  Date/Author: 2026-04-29 / Codex.

- Decision: Keep Screener tile launch logic in `ops/screener/state.rs`.
  Rationale: The tile selector and post-check are Screener-specific. Moving them into a generic desktop helper would blur the helper boundary.
  Date/Author: 2026-04-29 / Codex.

## Outcomes & Retrospective

Implementation and validation completed. `tab` and Screener full-page fallback now share app-window/new-tab primitives through `ops::desktop` without changing command output. The live smoke created a full-page Screener target through the `new_tab_tile` path and verified `screener status` with the returned `target_cli_args`.

## Context and Orientation

The `tradingview-cli` package lives under `crates/cli/`. Operation adapters live under `crates/cli/src/ops/`. The `tab` adapter owns `tv tab list/switch/new/close`. The Screener state adapter owns `tv screener status/open/get/close`, including `tv screener open --full-page`.

TradingView Desktop exposes an app-window CDP target for the shell UI and separate page targets for chart tabs, Screener tabs, and the Desktop new-tab launcher. The shared helper in this plan is only for the app-window/new-tab launcher boundary. It is not domain logic and it is not a public API.

## Plan of Work

Create `crates/cli/src/ops/desktop.rs` and declare it from `crates/cli/src/ops.rs`. Move the shared app-window tab row model, app-window target handoff model, app-tab reading, create-new-tab clicking, close-tab clicking, new-app-tab diffing, and new-tab target waiting into that module.

Update `crates/cli/src/ops/tab.rs` so it uses `ops::desktop` for app-window operations while keeping chart target classification, source-tab validation, tab payload shaping, and tab-specific tests in `tab.rs`.

Update `crates/cli/src/ops/screener/state.rs` so `screener_open_full_page` uses `ops::desktop` to create or find the Desktop new-tab target. Keep the Stock Screener tile click expression and full-page Screener target post-check in `state.rs`.

Update stable docs to record that app-window/new-tab helper code is shared inside the CLI package and should remain an adapter helper, not a domain/model crate.

## Concrete Steps

Run these commands from the repository root:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo test -p tradingview-cli desktop -- --nocapture
    cargo test -p tradingview-cli tab -- --nocapture
    cargo test -p tradingview-cli screener::state -- --nocapture
    cargo test -p tradingview-cli --test cli_contract tab -- --nocapture
    cargo test -p tradingview-cli --test cli_contract screener -- --nocapture
    cargo metadata --no-deps --format-version 1
    git diff --check

For optional live smoke, use only scrubbed target ids in notes:

    target/debug/tv tab list
    target/debug/tv screener open --full-page
    target/debug/tv --target-id <screener-target> screener status

## Validation and Acceptance

The refactor is accepted when the focused tests for `desktop`, `tab`, and `screener::state` pass, the full workspace baseline passes, and the CLI contract tests for `tab` and `screener` still pass. `tv tab ...` and `tv screener open --full-page` must keep their existing public behavior and JSON fields.

Validation run on 2026-04-29:

    cargo test -p tradingview-cli desktop -- --nocapture
    cargo test -p tradingview-cli tab -- --nocapture
    cargo test -p tradingview-cli screener::state -- --nocapture
    cargo test -p tradingview-cli --test cli_contract tab -- --nocapture
    cargo test -p tradingview-cli --test cli_contract screener -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

All passed. The tracked-doc hygiene grep reported only existing policy language, archived validation-command examples, and secret-safety wording; no new machine-specific path, account-local identifier, cookie, token, or authorization value was added.

## Idempotence and Recovery

The code changes are behavior-preserving. If a test fails, revert only the helper extraction for the failing path and keep the original operation-specific behavior until the shared helper can exactly preserve it. Do not change live TradingView account data for this refactor.

## Artifacts and Notes

Do not record live target ids, account-local values, cookies, tokens, or local absolute paths in this plan.

## Interfaces and Dependencies

The end state should include these internal helpers in `crates/cli/src/ops/desktop.rs`:

    pub(crate) struct AppTab { ... }
    pub(crate) struct AppWindowTarget { ... }
    pub(crate) fn app_window_targets_from_targets(targets: &[Target]) -> Vec<AppWindowTarget>;
    pub(crate) fn app_window_target(targets: &[Target]) -> Result<&Target, AppError>;
    pub(crate) async fn app_tabs_from_targets(targets: &[Target]) -> Vec<AppTab>;
    pub(crate) async fn read_app_tabs(runtime: &mut impl RuntimeEvaluator) -> Result<Vec<AppTab>, AppError>;
    pub(crate) async fn click_create_new_app_tab(runtime: &mut impl RuntimeEvaluator) -> Result<(), AppError>;
    pub(crate) async fn create_new_app_tab(config: &TransportConfig) -> Result<(), AppError>;
    pub(crate) async fn click_close_app_tab(runtime: &mut impl RuntimeEvaluator, index: usize) -> Result<(), AppError>;
    pub(crate) async fn wait_for_app_tab_update(milliseconds: u64);
    pub(crate) fn new_app_tabs(before: &[AppTab], after: &[AppTab]) -> Vec<AppTab>;
    pub(crate) async fn current_new_tab_target(config: &TransportConfig) -> Result<Option<Target>, AppError>;
    pub(crate) async fn wait_for_new_tab_target(config: &TransportConfig, wait_attempts: usize, wait_ms: u64, failure_details: Value) -> Result<Target, AppError>;

These helpers are internal to the CLI package and are not stable public Rust APIs.

## Open Questions

- UNCONFIRMED: Whether a future TradingView Desktop build will make the standard CDP `/json/new` path reliable. This refactor does not depend on that behavior.
