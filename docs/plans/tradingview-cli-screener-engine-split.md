# Split remaining Screener engine modules

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md`.

## Purpose / Big Picture

This refactor completes the current Screener adapter decomposition phase without changing any user-visible `tv screener` behavior. Validation and columns already live in their own submodules. This slice moves the remaining operation bodies from `engine.rs` into `state.rs`, `screens.rs`, and `filters.rs`, leaving `engine.rs` as the shared Screener runtime/helper module.

The visible result should be no behavior change. Users should see the same Screener commands, JSON payloads, validation errors, and exit codes. The maintainability result is a cut point where Screener is organized as `validation / state / screens / filters / columns / engine(common)`.

## Progress

- [x] (2026-04-28T09:55Z) Confirmed `filters.rs`, `screens.rs`, and `state.rs` were still re-export-only modules and `engine.rs` still owned their operation bodies.
- [x] (2026-04-28T10:05Z) Archived the completed columns split ExecPlan.
- [x] (2026-04-28T10:20Z) Moved state, screen, and filter operation bodies plus their focused tests into their respective modules.
- [x] (2026-04-28T10:20Z) Kept shared open/read/restore, storage config fetch, click dispatch, column normalization, and JavaScript helper expansion in `engine.rs`.
- [x] (2026-04-28T10:35Z) Ran full validation, focused Screener tests, CLI contract tests, behavior smoke, and tracked-doc hygiene grep.

## Surprises & Discoveries

- Observation: Screen and filter code both need the same click-point dispatch helpers, so those helpers belong in the shared engine for this slice.
  Evidence: `filters.rs` uses click points for option clear/select and add-filter choices; `screens.rs` uses the same dispatch path for menu/catalog/dialog clicks.

- Observation: Active screen storage config fetch remains shared for columns and filters.
  Evidence: `columns.rs` uses it for column storage mutations and `filters.rs` uses it for filter remove/clear storage mutations.

## Decision Log

- Decision: Move `state`, `screens`, and `filters` in one slice.
  Rationale: A one-module slice was too fine-grained after validation and columns had already established the pattern. Moving the remaining sub-surfaces together leaves a cleaner and more useful Screener boundary.
  Date/Author: 2026-04-28 / Codex.

- Decision: Keep `engine.rs` instead of deleting it.
  Rationale: Shared runtime and page-session helpers are still genuinely common. Deleting the module would force premature duplication or a poorly named storage/runtime module.
  Date/Author: 2026-04-28 / Codex.

## Outcomes & Retrospective

Implemented and validated. `state.rs`, `screens.rs`, and `filters.rs` now own their operation bodies and focused tests. `engine.rs` is reduced to shared Screener runtime and page-session helpers, including read/open-restore sessions, active storage config fetch, click dispatch, column normalization, and common JavaScript helper expansion.

Validation passed:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo test -p tradingview-cli screener -- --nocapture`
- `cargo test -p tradingview-cli --test cli_contract screener -- --nocapture`
- `cargo test -p tradingview-cli screener::filters -- --nocapture`
- `cargo test -p tradingview-cli screener::screens -- --nocapture`
- `cargo test -p tradingview-cli screener::state -- --nocapture`
- `cargo metadata --no-deps --format-version 1`
- `git diff --check`

Behavior smoke passed for Screener help, filter help, screen help, filter add validation, screen delete confirmation validation, and structured connection errors for Screener status and filter list with an unavailable CDP port. The tracked-doc hygiene grep returned existing policy text and validation-command examples in archived plans; no new live account identifiers, credentials, or machine-specific operational values were added.

## Context and Orientation

The `tradingview-cli` package lives under `crates/cli/`. Screener operations are exposed through `crates/cli/src/ops/screener.rs`, which declares submodules under `crates/cli/src/ops/screener/`.

Before this slice, `validation.rs` and `columns.rs` owned implementation bodies. `filters.rs`, `screens.rs`, and `state.rs` were still thin re-export modules. `engine.rs` still contained most Screener behavior.

## Plan of Work

Move state operations into `state.rs`: `screener_status`, `screener_open`, `screener_get`, and `screener_close`, plus their state-specific helpers, open/close JavaScript expressions, and state tests.

Move screen operations into `screens.rs`: `screener_screens_active`, `actions`, `list`, `switch`, `save`, `create`, `rename`, `save_as`, and `delete`, plus screen target/action helpers, menu/catalog/dialog/storage helpers, screen JavaScript expressions, and screen tests.

Move filter operations into `filters.rs`: `screener_filters_list`, `actions`, `add`, `modify`, `remove`, and `clear`, plus filter target/storage helpers, filter action/add/modify helpers, filter transient popup helpers, and filter tests.

Keep `engine.rs` as the shared Screener helper module. It should own the source constant, read/open-restore session helpers, active screen title lookup, active storage config fetch, common click-point dispatch, column normalization used by columns, base Screener read expressions, and the common JavaScript helper body.

Update docs to record that this is the current cut point, not a new workspace crate boundary.

## Concrete Steps

Run commands from the repository root.

1. Archive the completed columns split plan:

        git mv docs/plans/tradingview-cli-screener-columns-split.md docs/plans/archives/tradingview-cli-screener-columns-split.md

2. Move operation bodies into:

        crates/cli/src/ops/screener/state.rs
        crates/cli/src/ops/screener/screens.rs
        crates/cli/src/ops/screener/filters.rs

3. Keep shared helpers in:

        crates/cli/src/ops/screener/engine.rs

4. Update docs:

        docs/architecture.md
        docs/development.md
        docs/v0.3-roadmap.md
        CHANGELOG.md
        docs/plans/README.md
        CONTINUITY.md

5. Validate:

        cargo fmt --check
        cargo clippy --workspace --all-targets --all-features -- -D warnings
        cargo test --workspace
        cargo test -p tradingview-cli screener -- --nocapture
        cargo test -p tradingview-cli --test cli_contract screener -- --nocapture
        cargo metadata --no-deps --format-version 1
        git diff --check

6. Run focused tests:

        cargo test -p tradingview-cli screener::filters -- --nocapture
        cargo test -p tradingview-cli screener::screens -- --nocapture
        cargo test -p tradingview-cli screener::state -- --nocapture

7. Run behavior smoke:

        target/debug/tv screener --help
        target/debug/tv screener filters --help
        target/debug/tv screener screens --help
        target/debug/tv screener filters add --name "" --min 1 --dry-run
        target/debug/tv screener screens delete --name Main
        TV_CDP_PORT=9 target/debug/tv screener status
        TV_CDP_PORT=9 target/debug/tv screener filters list

## Validation and Acceptance

Acceptance requires all workspace tests and Screener contract tests to pass. Focused module tests should run for `filters`, `screens`, and `state`; if an exact module filter changes, record the actual command in this plan.

Behavior smoke should prove that help still renders, validation failures still happen before CDP connection, and CDP-dependent reads still return structured connection errors when pointed at an unavailable port. JSON envelope and field names must not change.

## Idempotence and Recovery

This split is mechanical. If compilation fails because a moved module needs a helper from `engine.rs`, prefer making that helper `pub(super)` in `engine.rs` over duplicating logic. If a helper is clearly specific to one sub-surface, move it with that sub-surface. If behavior output changes, restore the previous payload shape rather than updating tests.

## Artifacts and Notes

This slice should not require live TradingView mutation smoke. Do not record live screen names, raw saved-screen storage payloads, account-local ids, cookies, tokens, or local absolute paths in tracked docs.

## Interfaces and Dependencies

At completion, `crates/cli/src/ops/screener.rs` continues to expose the same public adapter functions. The implementation modules become:

- `validation.rs`: request and selector validation
- `state.rs`: top-level Screener open/status/read/close operations
- `screens.rs`: saved screen lifecycle and switching operations
- `filters.rs`: filter list/action/add/modify/remove/clear operations
- `columns.rs`: column list/config/action/add/remove/reorder operations
- `engine.rs`: common Screener runtime and page-session helpers

## Open Questions

No blocking questions. After this split, inspect whether the remaining `engine.rs` helper set should stay as-is or become a smaller `runtime.rs` / `storage.rs` pair in a later slice.
