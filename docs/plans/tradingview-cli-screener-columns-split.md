# Move Screener columns into its module

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md`.

## Purpose / Big Picture

This refactor continues the Screener adapter decomposition without changing any user-visible `tv screener` behavior. The previous slice moved pure validation into `crates/cli/src/ops/screener/validation.rs`. This slice moves the column operation implementation into `crates/cli/src/ops/screener/columns.rs`.

The visible result should be no behavior change. Users should see the same column commands, JSON payloads, validation errors, and exit codes. The maintainability result is that storage-backed column operations become their own sub-surface implementation, while the remaining `engine.rs` keeps shared Screener runtime and other sub-surfaces until later splits.

## Progress

- [x] (2026-04-28T09:10Z) Confirmed `columns.rs` was still a re-export-only module and `engine.rs` still owned all column operations, helpers, and tests.
- [x] (2026-04-28T09:10Z) Archived the completed validation split ExecPlan.
- [x] (2026-04-28T09:20Z) Moved column operations, column-specific helpers, and column tests into `columns.rs`.
- [x] (2026-04-28T09:20Z) Made only required shared Screener engine helpers visible to sibling modules with `pub(super)`.
- [x] (2026-04-28T09:25Z) Updated architecture, development, roadmap, changelog, plan index, and continuity docs.
- [x] (2026-04-28T09:35Z) Ran validation and behavior smoke.
- [x] (2026-04-28T09:35Z) Prepared the completed refactor for commit.

## Surprises & Discoveries

- Observation: Column operations are a good next extraction boundary because they cluster around saved-screen storage and do not require moving filter or screen lifecycle logic.
  Evidence: `rg` showed `screener_columns_*`, column target resolution, storage column transforms, `save_screener_storage_columns`, and column tests grouped in `engine.rs`.

- Observation: The extracted columns module compiles cleanly while leaving shared open-state and storage-fetch helpers in `engine.rs`.
  Evidence: `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed after making the required shared helpers `pub(super)`.

## Decision Log

- Decision: Move `columns` implementation before a generic storage module.
  Rationale: Column commands can be separated while still calling the existing shared storage fetch/runtime helpers. A generic storage module would require deciding the filter/screen storage boundary too early.
  Date/Author: 2026-04-28 / Codex.

- Decision: Keep `engine.rs` as the temporary shared Screener runtime/helper module.
  Rationale: The goal is a low-risk behavior-preserving split. Shared helpers such as state reads, open/restore sessions, active screen title lookup, and storage fetch can be made `pub(super)` now and reorganized later when more sub-surfaces have moved.
  Date/Author: 2026-04-28 / Codex.

## Outcomes & Retrospective

Implemented and validated. `crates/cli/src/ops/screener/columns.rs` now owns the Screener column operations, visible column target types, storage column target types, storage column payload helpers, storage column add/remove/reorder helpers, column action discovery, and column operation unit tests. `engine.rs` remains the shared runtime/helper module for the rest of Screener.

Validation passed:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo test -p tradingview-cli screener::columns -- --nocapture`
- `cargo test -p tradingview-cli screener -- --nocapture`
- `cargo test -p tradingview-cli --test cli_contract screener -- --nocapture`
- `cargo metadata --no-deps --format-version 1`
- `git diff --check`

Behavior smoke passed for `tv screener columns --help`, column add validation, column reorder validation, structured connection failure for column remove with an unavailable CDP port, and structured connection failure for Screener status with an unavailable CDP port. The tracked-doc hygiene grep returned only existing policy text and validation-command examples, including archived plans; no new live account identifiers, credentials, or machine-specific operational values were added.

## Context and Orientation

The `tradingview-cli` package lives under `crates/cli/`. Screener operations are exposed through `crates/cli/src/ops/screener.rs`, which declares submodules under `crates/cli/src/ops/screener/`. The current `engine.rs` still contains most Screener implementation. `validation.rs` already owns input validation. `columns.rs` currently only re-exports column functions from `engine.rs`.

Column operations are the commands under `tv screener columns`: `list`, `config`, `actions`, `add`, `remove`, and `reorder`. Some are read-only and some mutate saved test screens through TradingView's saved-screen storage API. This plan does not add or remove commands and does not implement `columns reset`.

## Plan of Work

Move the public column operation functions from `engine.rs` into `columns.rs`: `screener_columns_list`, `screener_columns_config`, `screener_columns_actions`, `screener_columns_add`, `screener_columns_remove`, and `screener_columns_reorder`. Move the column-only target structs, target resolution functions, storage column normalization and payload builders, add/remove/reorder helpers, post-check comparison helper, column test-screen guard, `read_column_actions`, and column operation unit tests with them.

Leave shared Screener runtime in `engine.rs`, but make the specific functions and types used by `columns.rs` visible as `pub(super)`. This includes the source constant, state reads, open/restore session helper, active screen title helper, storage fetch helper, open-state checks, visible column normalization, boolean value helper, and expression expansion helper. Do not move filter, screen, row-read, generic UI, JavaScript helper body, or non-column tests in this slice.

Update stable docs to record that `columns.rs` now owns the column sub-surface implementation. Keep the plan and docs public-safe: no live target ids, account-local ids, raw storage payloads, cookies, tokens, or local absolute paths.

## Concrete Steps

Run commands from the repository root.

1. Archive the completed validation split plan:

        git mv docs/plans/tradingview-cli-screener-validation-split.md docs/plans/archives/tradingview-cli-screener-validation-split.md

2. Move column implementation into:

        crates/cli/src/ops/screener/columns.rs

   Keep shared runtime helpers in:

        crates/cli/src/ops/screener/engine.rs

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
        cargo test -p tradingview-cli screener::columns -- --nocapture
        cargo test -p tradingview-cli screener -- --nocapture
        cargo test -p tradingview-cli --test cli_contract screener -- --nocapture
        cargo metadata --no-deps --format-version 1
        git diff --check

5. Run behavior smoke:

        target/debug/tv screener columns --help
        target/debug/tv screener columns add --id "" --dry-run
        TV_CDP_PORT=9 target/debug/tv screener columns remove --index 0 --dry-run
        target/debug/tv screener columns reorder --from-index 0 --to-index 0 --dry-run
        TV_CDP_PORT=9 target/debug/tv screener status

## Validation and Acceptance

Acceptance requires all workspace tests and Screener contract tests to pass. The focused `screener::columns` test filter should run column operation tests from the CLI package. If the exact filter changes after the move, use the nearest column-specific filter and record the actual command in this plan.

Behavior smoke should prove that column help still renders, validation failures still happen before CDP connection, and CDP-dependent reads still return structured connection errors when pointed at an unavailable port. JSON envelope and field names must not change.

## Idempotence and Recovery

This split is mechanical. If compilation fails because `columns.rs` needs a helper from `engine.rs`, prefer making that helper `pub(super)` in `engine.rs` over duplicating logic. If a moved test starts depending on a non-column fixture that is better left in `engine.rs`, keep only that fixture shared or local to the moved tests. If behavior output changes, restore the previous payload shape rather than updating tests.

## Artifacts and Notes

This slice should not require live TradingView mutation smoke. Do not record live screen names, raw saved-screen storage payloads, account-local ids, cookies, tokens, or local absolute paths in tracked docs.

## Interfaces and Dependencies

At completion, `crates/cli/src/ops/screener/columns.rs` owns the column operation API used by the Screener facade:

    pub async fn screener_columns_list(...)
    pub async fn screener_columns_config(...)
    pub async fn screener_columns_actions(...)
    pub async fn screener_columns_remove(...)
    pub async fn screener_columns_add(...)
    pub async fn screener_columns_reorder(...)

`engine.rs` remains a sibling module and may expose shared Screener helper interfaces with `pub(super)` until a later split creates a cleaner shared module.

## Open Questions

No blocking questions. After this split, inspect the remaining `engine.rs` dependency shape to choose between moving `filters`, moving `screens`, or extracting shared storage/runtime helpers.
