# Move Screener validation into its module

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md`.

## Purpose / Big Picture

This refactor makes the Screener adapter easier to maintain without changing any user-visible command behavior. The previous Screener adapter split created `crates/cli/src/ops/screener/validation.rs`, but that file still only re-exported validation functions from the large `engine.rs` implementation. After this change, validation request types, selector types, validation helpers, and validation unit tests live in the validation module itself.

The visible result is intentionally no behavior change. A user should still see the same `tv screener ...` help, validation errors, JSON envelope shape, and exit codes. The maintainability result is that the pure input-boundary code is separated from CDP, DOM, and saved-screen storage logic, making later `columns` or `storage` extraction safer.

## Progress

- [x] (2026-04-28T08:35Z) Confirmed the completed Screener adapter split plan exists and that `validation.rs` is still a re-export-only module.
- [x] (2026-04-28T08:35Z) Moved Screener validation request types, selectors, helpers, and validation tests from `engine.rs` into `validation.rs`.
- [x] (2026-04-28T08:35Z) Updated `engine.rs` to import validation types and helpers from `super::validation`.
- [x] (2026-04-28T08:35Z) Archived the completed Screener adapter split ExecPlan.
- [x] (2026-04-28T08:45Z) Updated architecture, development, roadmap, changelog, plan index, and continuity docs.
- [x] (2026-04-28T08:55Z) Ran validation and behavior smoke.
- [x] (2026-04-28T08:55Z) Prepared the completed refactor for commit.

## Surprises & Discoveries

- Observation: Validation code is mostly pure and could be moved without changing runtime logic.
  Evidence: `cargo check --workspace` passed after moving validation bodies, with only leftover unused constants in `engine.rs` before cleanup.

- Observation: Some runtime tests in `engine.rs` still create validation request values as setup for storage/UI behavior tests.
  Evidence: `cargo clippy --workspace --all-targets --all-features -- -D warnings` initially failed in test builds until those tests imported `validate_screener_filter_add_request`, `validate_screener_filter_modify_request`, and `validate_screener_column_add_request` from `super::super::validation`.

## Decision Log

- Decision: Move only validation implementation in this slice.
  Rationale: Validation is the least coupled Screener sub-surface because it does not need a CDP runtime, DOM reads, or saved-screen storage payloads. Moving it first proves the new facade/submodule shape while avoiding behavior risk.
  Date/Author: 2026-04-28 / Codex.

- Decision: Keep Screener inside the CLI package for now instead of extracting a Screener crate.
  Rationale: The current Screener implementation still mixes page-session storage APIs, UI actions, JavaScript snippets, and post-check logic. The dependency boundary is not clear enough for a useful reusable domain crate yet.
  Date/Author: 2026-04-28 / Codex.

## Outcomes & Retrospective

Implemented and validated. `crates/cli/src/ops/screener/validation.rs` now owns Screener validation request types, selectors, helpers, test-screen guards, and validation unit tests. `crates/cli/src/ops/screener/engine.rs` imports those validation interfaces and continues to own runtime, storage, UI, JavaScript, and post-check behavior.

Validation passed:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo test -p tradingview-cli screener::validation -- --nocapture`
- `cargo test -p tradingview-cli screener -- --nocapture`
- `cargo test -p tradingview-cli --test cli_contract screener -- --nocapture`
- `cargo metadata --no-deps --format-version 1`
- `git diff --check`

Behavior smoke passed for Screener help, validation failures for empty filter names and column ids, guarded screen deletion without confirmation, and structured `TV_CDP_PORT=9 tv screener status` connection failure with exit code 2. The tracked-doc hygiene grep returned only existing policy text and validation-command examples, including archived plans; no new live account identifiers, credentials, or machine-specific operational values were added.

## Context and Orientation

The repository is a Rust workspace. The `tradingview-cli` package lives under `crates/cli/`, and the user-facing binary is still named `tv`. The CLI package has an operation adapter layer under `crates/cli/src/ops/`. An operation adapter is code that connects command requests to the underlying TradingView page session, storage API, or Desktop interaction. It is not yet a pure domain layer.

Screener is currently split behind `crates/cli/src/ops/screener.rs`, with implementation files under `crates/cli/src/ops/screener/`. The previous adapter split left most implementation in `crates/cli/src/ops/screener/engine.rs` and made `validation.rs` a re-export file. This plan moves the validation implementation into `validation.rs` while leaving runtime, storage, UI, JavaScript, and post-check behavior in `engine.rs`.

Validation means checking CLI inputs before a command talks to TradingView. Examples are ensuring a Screener filter selector uses either `--index` or `--text` but not both, rejecting blank column ids, rejecting non-finite numeric values, and requiring destructive screen names to look like disposable test screens.

## Plan of Work

Create the new ExecPlan and archive the completed adapter split plan. Then move the validation-only code from `engine.rs` into `validation.rs`. The moved code includes request and selector types such as `ScreenerFilterSelector`, `ScreenerFilterModifyRequest`, `ScreenerFilterAddRequest`, `ScreenerColumnAddRequest`, and `ScreenerColumnSelector`; validation functions named `validate_screener_*`; helper functions used only to build validation payloads or match filter presets; and validation unit tests.

Update `engine.rs` so runtime operations import the validation module with `super::validation::{...}`. Keep public operation names and re-exports unchanged. Do not move saved-screen storage functions, visible UI helpers, JavaScript snippets, table reads, post-check functions, or runtime mutation code in this slice.

Update stable docs to record that Screener validation is now a pure adapter boundary. The architecture and development docs should describe this as an example of splitting CDP-free input boundaries before deeper storage/UI extraction. The roadmap and changelog should record this as an internal refactor. Update `CONTINUITY.md` as the local ledger, but do not commit it.

## Concrete Steps

Run commands from the repository root.

1. Archive the completed adapter split plan:

        git mv docs/plans/tradingview-cli-screener-adapter-split.md docs/plans/archives/tradingview-cli-screener-adapter-split.md

2. Move validation implementation into:

        crates/cli/src/ops/screener/validation.rs

   Keep `crates/cli/src/ops/screener/engine.rs` as the caller for runtime operations.

3. Update docs:

        docs/architecture.md
        docs/development.md
        docs/v0.3-roadmap.md
        CHANGELOG.md
        docs/plans/README.md
        CONTINUITY.md

4. Validate:

        cargo fmt --check
        cargo clippy --workspace --all-targets --all-features -- -D warnings
        cargo test --workspace
        cargo test -p tradingview-cli screener::validation -- --nocapture
        cargo test -p tradingview-cli screener -- --nocapture
        cargo test -p tradingview-cli --test cli_contract screener -- --nocapture
        cargo metadata --no-deps --format-version 1
        git diff --check

5. Run behavior smoke:

        target/debug/tv screener --help
        target/debug/tv screener filters add --name "" --min 1 --dry-run
        target/debug/tv screener columns add --id "" --dry-run
        target/debug/tv screener screens delete --name Main
        TV_CDP_PORT=9 target/debug/tv screener status

## Validation and Acceptance

Acceptance requires the Rust workspace checks to pass and the Screener CLI contract tests to remain green. The focused validation tests should run from the `tradingview-cli` package. If the exact module-path filter for `screener::validation` matches no tests after the move, use the closest validation-specific filter and record that command in this plan.

The behavior smoke should show that help still renders normally, validation failures still happen before CDP connection, and `TV_CDP_PORT=9 tv screener status` still returns a structured connection error. Success and error envelopes must keep the existing shape.

## Idempotence and Recovery

This is a behavior-preserving move. If compilation fails because `engine.rs` needs a moved helper, prefer making that helper `pub(super)` inside `validation.rs` over moving runtime behavior back into `engine.rs`. If tests fail because validation error text changed, restore the exact previous message rather than updating tests. If the split becomes too broad, stop after validation and leave columns or storage for a later plan.

## Artifacts and Notes

Do not record live Screener screen names, account-local ids, raw TradingView payloads, cookies, tokens, or local absolute paths in tracked docs. This slice should not require live mutation smoke.

## Interfaces and Dependencies

At completion, `crates/cli/src/ops/screener/validation.rs` owns the validation API used by sibling Screener modules:

    pub enum ScreenerFilterSelector { ... }
    pub struct ScreenerFilterModifyRequest { ... }
    pub struct ScreenerFilterAddRequest { ... }
    pub struct ScreenerColumnAddRequest { ... }
    pub enum ScreenerColumnSelector { ... }
    pub fn validate_screener_limit(...) -> Result<usize, AppError>
    pub fn validate_screener_filter_selector(...) -> Result<ScreenerFilterSelector, AppError>
    pub fn validate_screener_filter_modify_request(...) -> Result<ScreenerFilterModifyRequest, AppError>
    pub fn validate_screener_filter_add_request(...) -> Result<ScreenerFilterAddRequest, AppError>
    pub fn validate_screener_column_selector(...) -> Result<ScreenerColumnSelector, AppError>
    pub fn validate_screener_column_add_request(...) -> Result<ScreenerColumnAddRequest, AppError>
    pub fn validate_screener_column_reorder_request(...) -> Result<(), AppError>
    pub fn validate_screener_filter_clear(...) -> Result<(), AppError>
    pub fn validate_screener_screen_name(...) -> Result<String, AppError>
    pub fn validate_screener_screen_rename_request(...) -> Result<(String, String), AppError>
    pub fn validate_screener_screen_test_mutation_name(...) -> Result<(), AppError>
    pub fn validate_screener_screen_delete_request(...) -> Result<(), AppError>

Some helper functions may be `pub(super)` so `engine.rs` can build dry-run payloads or enforce test-screen guards without duplicating logic.

## Open Questions

The next split candidate is still open. `columns` is attractive because storage-backed mutations already have a clearer boundary than visible UI helpers. `storage` is also attractive if repeated saved-screen payload operations can be separated cleanly. Choose the next slice after this validation split is committed and the remaining `engine.rs` dependencies are visible.
