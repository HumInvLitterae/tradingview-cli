# Screener domain boundary

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md` in this repository. It is self-contained and describes a behavior-preserving refactor that introduces a larger in-package Screener domain/service boundary.

## Purpose / Big Picture

The Screener operation adapter is the largest remaining adapter in the CLI package. It already has submodules for state, screens, filters, columns, validation, and shared runtime helpers, but several CDP-free pieces still live inside operation modules: validation, visible target resolution, storage payload shaping, and public payload helpers.

After this change, `domain::screener` owns those pure Rust pieces. `ops/screener` remains responsible for opening or restoring the Screener, evaluating JavaScript in TradingView Desktop, calling logged-in storage endpoints through the page session, clicking visible UI, and verifying post-mutation state. Users should see no command behavior change.

## Progress

- [x] (2026-04-29) Inspected `ops/screener` and identified CDP-free validation, target resolution, and storage payload helpers.
- [x] (2026-04-29) Archived the completed Drawing domain-boundary plan and created this plan.
- [x] (2026-04-29) Added `domain::screener::{validation,columns,filters,screens}`.
- [x] (2026-04-29) Moved Screener validation implementation to the domain module and kept `ops/screener/validation.rs` as a thin re-export.
- [x] (2026-04-29) Moved CDP-free column, filter, and screen target/storage/payload helpers into `domain::screener`.
- [x] (2026-04-29) Kept runtime evaluation, storage fetch/save JavaScript, UI click/popover logic, and post-check behavior in `ops/screener`.
- [x] (2026-04-29) Updated stable docs and continuity ledger.
- [x] (2026-04-29) Ran validation, behavior smoke, and hygiene checks.
- [ ] Commit the related changes as one refactor.

## Surprises & Discoveries

- Observation: Screener is a larger domain-boundary sample than Watchlist, Alert, Replay, or Drawing.
  Evidence: `columns.rs`, `filters.rs`, and `screens.rs` each contained pure target/payload helpers next to runtime execution. Moving all runtime code would be wrong, so this slice deliberately moves only helpers that take and return ordinary Rust or JSON values.

## Decision Log

- Decision: Add `domain::screener` inside the existing CLI package rather than creating `tradingview-screener`.
  Rationale: Screener still depends heavily on TradingView page-session state, saved-screen storage endpoints, UI popovers, and post-checks. A workspace crate would imply a more stable reusable API than the current boundary supports.
  Date/Author: 2026-04-29 / Codex.
- Decision: Keep `ops/screener/validation.rs` as a thin re-export module.
  Rationale: Application dispatch and operation modules already import validation through the Screener adapter. Re-exporting keeps the diff behavior-preserving while making `domain::screener::validation` the implementation owner.
  Date/Author: 2026-04-29 / Codex.
- Decision: Keep storage fetch/save functions in `ops/screener`.
  Rationale: Fetching and saving storage uses page-session JavaScript, logged-in fetch calls, and `RuntimeEvaluator`. The domain module should shape payloads, not execute TradingView API calls.
  Date/Author: 2026-04-29 / Codex.

## Outcomes & Retrospective

`domain::screener` is now the largest proof of the in-package domain/service boundary. Validation, target resolution, storage payload shaping, and public payload helpers moved out of the operation adapter while runtime execution stayed in `ops/screener`.

The domain layer should now be considered a stable pattern, not a mandate to move every operation. Future extraction should start only when there is a clear CDP-free logic boundary.

## Context and Orientation

The repository is a Rust workspace. The `tv` binary and command adapters live in `crates/cli`. The `crates/cli/src/domain.rs` facade exposes reusable command logic that does not depend on clap command enums or CDP runtime objects. The existing domain examples are Watchlist, Alert, Replay, and Drawing.

The relevant Screener files are:

- `crates/cli/src/domain/screener.rs` and `crates/cli/src/domain/screener/`, the new domain boundary.
- `crates/cli/src/ops/screener.rs`, the Screener operation adapter facade.
- `crates/cli/src/ops/screener/validation.rs`, which remains a compatibility re-export.
- `crates/cli/src/ops/screener/columns.rs`, `filters.rs`, and `screens.rs`, which continue to perform runtime work but delegate pure helpers to domain modules.
- `crates/cli/src/ops/screener/engine.rs`, which remains the shared runtime/helper module for open/restore sessions, state reads, active storage fetches, common click dispatch, and shared JavaScript helpers.

CDP means Chrome DevTools Protocol, the browser automation protocol used here to evaluate JavaScript inside TradingView Desktop. Any helper that requires `RuntimeEvaluator`, `fetch` inside the logged-in page, DOM click points, popovers, or post-check reads remains in `ops`.

## Plan of Work

Create `domain::screener` with focused modules. Move request validation and test-screen guards into `validation`. Move column target resolution, column storage target shaping, add/remove/reorder helpers, and order checks into `columns`. Move filter target resolution, filter storage target shaping, alignment/index checks, remove/order helpers, text normalization, and option matching into `filters`. Move screen target/action resolution and payload shaping into `screens`.

Update the operation modules so they import these helpers from `crate::domain::screener`. Do not move runtime evaluation, storage fetch/save JavaScript, popover handling, click dispatch, or post-check loops.

Update stable docs to record `domain::screener` and to mark this as the point where domain-boundary refactoring should stabilize instead of continuing mechanically.

## Concrete Steps

Run focused tests:

    cargo test -p tradingview-cli domain::screener -- --nocapture
    cargo test -p tradingview-cli screener -- --nocapture
    cargo test -p tradingview-cli --test cli_contract screener -- --nocapture

Run full validation:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Run behavior smoke:

    target/debug/tv screener --help
    target/debug/tv screener filters add --name "" --min 1 --dry-run
    target/debug/tv screener filters modify --index 0 --option "" --dry-run
    target/debug/tv screener columns add --id "" --dry-run
    target/debug/tv screener columns reorder --from-index 0 --to-index 0 --dry-run
    target/debug/tv screener screens delete --name Main
    TV_CDP_PORT=9 target/debug/tv screener status
    TV_CDP_PORT=9 target/debug/tv screener columns config

Run hygiene:

    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true

Commit:

    git add CHANGELOG.md crates/cli/src/domain.rs crates/cli/src/domain/screener.rs crates/cli/src/domain/screener crates/cli/src/ops/screener docs
    git commit -m "refactor(domain): Introduce screener service boundary"

## Validation and Acceptance

The change is accepted when tests and smoke prove the same behavior:

- invalid Screener CLI inputs still fail before CDP connection;
- bad CDP port Screener reads return structured connection errors;
- Screener command JSON field names and exit codes are unchanged;
- `domain::screener` contains no `RuntimeEvaluator` dependency and no page-session JavaScript;
- storage fetch/save, UI interaction, popover handling, and post-checks remain in `ops/screener`.

## Idempotence and Recovery

This is a behavior-preserving refactor. If a moved helper needs `RuntimeEvaluator`, move it back to `ops/screener` and record the reason here. If a JSON payload test fails, compare field names and values before changing behavior. Re-running formatting and tests is safe.

## Artifacts and Notes

Initial structural evidence:

    wc -l crates/cli/src/ops/screener/*.rs
    result: filters, screens, and columns remain the largest Screener modules.

    rg -n "RuntimeEvaluator|storage|target|payload|validate_screener" crates/cli/src/ops/screener
    result: validation is CDP-free; column/filter/screen helpers mix pure target/payload logic with runtime execution.

Validation evidence:

    cargo test -p tradingview-cli domain::screener -- --nocapture
    result: passed, 13 tests.

    cargo test -p tradingview-cli screener -- --nocapture
    result: passed, 66 tests.

    cargo test -p tradingview-cli --test cli_contract screener -- --nocapture
    result: passed, 6 tests.

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    result: passed.

Behavior smoke:

    target/debug/tv screener --help
    result: help rendered with Screener subcommands and `--target-id`.

    target/debug/tv screener filters add --name "" --min 1 --dry-run
    target/debug/tv screener filters modify --index 0 --option "" --dry-run
    target/debug/tv screener columns add --id "" --dry-run
    target/debug/tv screener columns reorder --from-index 0 --to-index 0 --dry-run
    target/debug/tv screener screens delete --name Main
    result: validation failures returned before CDP connection, exit 1.

    TV_CDP_PORT=9 target/debug/tv screener status
    TV_CDP_PORT=9 target/debug/tv screener columns config
    result: structured connection errors, exit 2.

Boundary check:

    rg -n 'RuntimeEvaluator|evaluate\(|expanded_expression|js_string|CdpClient|TransportConfig' crates/cli/src/domain/screener || true
    result: no matches.
