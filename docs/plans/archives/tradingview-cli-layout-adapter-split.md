# Split Layout operation adapter modules

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md`.

## Purpose / Big Picture

This refactor splits the historical Layout operation adapter without changing user-visible `tv watchlist` or `tv pane` behavior. The current `layout.rs` file mixes watchlist reads, watchlist mutation, pane listing, pane layout/focus/symbol operations, validation, helper JavaScript, and tests. After this change, `layout.rs` remains the facade used by dispatch, while the implementation lives in focused `watchlist` and `pane` modules.

The visible result should be no behavior change. Users should see the same commands, JSON payloads, validation errors, and exit codes. The maintainability result is that Layout follows the same facade-plus-submodule direction as Screener and Alert.

## Progress

- [x] (2026-04-28T12:10Z) Confirmed `layout.rs` is about 2,238 lines and mixes watchlist and pane operations.
- [x] (2026-04-28T12:12Z) Archived the completed Alert adapter split ExecPlan.
- [x] (2026-04-28T12:25Z) Split Layout into facade plus `watchlist` and `pane` modules.
- [x] (2026-04-28T12:25Z) Moved focused tests into the nearest Layout submodule while preserving test behavior.
- [x] (2026-04-28T12:35Z) Updated architecture, development, roadmap, changelog, and plans index docs.
- [x] (2026-04-28T12:55Z) Ran validation, behavior smoke, metadata, whitespace, and hygiene checks.

## Surprises & Discoveries

- Observation: No shared Layout helper was clearly needed before the split.
  Evidence: panel-open and keyboard helpers are watchlist-specific; pane operations use chart-widget and chart-API helpers directly.

- Observation: The planned `watchlist add-bulk --symbols ""` smoke uses a flag that the current CLI does not define.
  Evidence: the command returns a structured clap validation error before CDP connection; the equivalent positional smoke `watchlist add-bulk ""` returns the existing `Symbol must not be empty` validation error.

## Decision Log

- Decision: Keep `layout.rs` as the facade module name.
  Rationale: `crates/cli/src/ops.rs` already re-exports watchlist and pane operations from `layout`, and changing the module name would not improve the public CLI contract.
  Date/Author: 2026-04-28 / Codex.

- Decision: Do not add `shared.rs` unless a helper is genuinely shared.
  Rationale: The current helper set is watchlist-specific or pane-specific. A mostly empty shared module would add indirection without reducing duplication.
  Date/Author: 2026-04-28 / Codex.

## Outcomes & Retrospective

Layout now follows the same adapter split direction as Screener and Alert. The
facade preserves existing operation exports, while watchlist and pane behavior
live in separate modules with their focused tests.

Validation passed with `cargo fmt --check`,
`cargo clippy --workspace --all-targets --all-features -- -D warnings`,
`cargo test --workspace`, `cargo test -p tradingview-cli layout -- --nocapture`,
`cargo test -p tradingview-cli --test cli_contract watchlist -- --nocapture`,
`cargo test -p tradingview-cli --test cli_contract pane -- --nocapture`,
focused `layout::watchlist` and `layout::pane` test filters,
`cargo metadata --no-deps --format-version 1`, `git diff --check`, and the
planned behavior smoke commands. The tracked-doc hygiene grep returned only
existing policy text and archived validation-command examples; no new live ids,
local paths, credentials, or raw payloads were introduced.

## Context and Orientation

The `tradingview-cli` package lives under `crates/cli/`. Operation adapters are exposed through `crates/cli/src/ops.rs`. Layout is currently a single `crates/cli/src/ops/layout.rs` file even though the user-visible commands are `tv watchlist ...` and `tv pane ...`.

Watchlist operations include `watchlist get`, `watchlist add`, `watchlist add-bulk`, and `watchlist remove`. They use a page-session symbols-list API when possible and keep DOM fallback behavior for the visible watchlist panel. Pane operations include `pane list`, `pane layout`, `pane focus`, and `pane symbol`; they operate through chart-widget objects in the active TradingView page.

## Plan of Work

Turn `crates/cli/src/ops/layout.rs` into a facade with submodules under `crates/cli/src/ops/layout/`.

Move watchlist behavior into `watchlist.rs`: watchlist get/add/add-bulk/remove, API-backed mutation helpers, DOM fallback helpers, symbol normalization, keyboard dispatch, and watchlist tests.

Move pane behavior into `pane.rs`: pane layout constants and validation, pane list/layout/focus/symbol operations, and pane tests.

Keep all existing exported function names available from `ops::layout`: `watchlist_get`, `watchlist_add`, `watchlist_add_bulk`, `watchlist_remove`, `validate_watchlist_add_bulk_request`, `pane_list`, `pane_layout`, `pane_focus`, `pane_symbol`, and `validate_pane_layout`.

Do not change CLI dispatch, JSON payload field names, validation error messages, API fallback policy, DOM fallback policy, or pane mutation behavior.

## Concrete Steps

Run commands from the repository root.

1. Archive the completed Alert adapter split plan:

        git mv docs/plans/tradingview-cli-alert-adapter-split.md docs/plans/archives/tradingview-cli-alert-adapter-split.md

2. Split Layout implementation into:

        crates/cli/src/ops/layout.rs
        crates/cli/src/ops/layout/watchlist.rs
        crates/cli/src/ops/layout/pane.rs

   Add `crates/cli/src/ops/layout/shared.rs` only if a helper is used by both watchlist and pane modules.

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
        cargo test -p tradingview-cli layout -- --nocapture
        cargo test -p tradingview-cli --test cli_contract watchlist -- --nocapture
        cargo test -p tradingview-cli --test cli_contract pane -- --nocapture
        cargo metadata --no-deps --format-version 1
        git diff --check

5. Run focused tests:

        cargo test -p tradingview-cli layout::watchlist -- --nocapture
        cargo test -p tradingview-cli layout::pane -- --nocapture

6. Run behavior smoke:

        target/debug/tv watchlist --help
        target/debug/tv pane --help
        target/debug/tv watchlist add ""
        target/debug/tv watchlist add-bulk --symbols ""
        target/debug/tv pane layout banana
        TV_CDP_PORT=9 target/debug/tv watchlist get
        TV_CDP_PORT=9 target/debug/tv pane list

## Validation and Acceptance

Acceptance requires all workspace tests and watchlist/pane contract tests to pass. Focused module tests should run for `layout::watchlist` and `layout::pane`; if an exact module filter changes, record the actual command in this plan.

Behavior smoke should prove that help still renders, validation failures still happen before CDP connection, and CDP-dependent reads still return structured connection errors when pointed at an unavailable port. JSON envelope, field names, and exit codes must not change.

## Idempotence and Recovery

This split is mechanical. If compilation fails because a moved helper is used by both modules, either keep it in the narrower module that actually owns it or create `shared.rs` and keep it `pub(super)`. If behavior output changes, restore the previous payload shape rather than updating tests.

## Artifacts and Notes

This slice should not require live TradingView mutation smoke. Do not record live watchlist names, symbols from the user's private lists, chart target ids, cookies, tokens, or local absolute paths in tracked docs.

## Interfaces and Dependencies

At completion, `crates/cli/src/ops/layout.rs` continues to expose the same adapter functions. The implementation modules become:

- `watchlist.rs`: watchlist reads and mutations
- `pane.rs`: pane listing and mutations

No new workspace crate is introduced. `saved_layout.rs` is already independent and is not part of this split.

## Open Questions

No blocking questions. After this split, inspect whether watchlist API helpers should stay private to Layout or whether a later page-session API helper boundary is useful.
