# Split the Screener operation adapter

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md`.

## Purpose / Big Picture

This refactor reduces the largest remaining `ops` module without changing CLI behavior. The Screener operation adapter has grown into a single large file that mixes command-facing operations, validation, page-session storage APIs, visible UI helpers, JavaScript snippets, and post-check logic. The first step is to put a facade and sub-surface modules in place so future work can move implementation details incrementally without changing dispatch or public CLI contracts.

The user-visible result should be no behavior change. The maintainability result is that `crates/cli/src/ops/screener.rs` becomes a small facade, with submodules for state, screens, filters, columns, and validation. The existing implementation is initially isolated behind an `engine` module, which keeps this slice behavior-preserving and creates clear next extraction points.

## Progress

- [x] (2026-04-28T10:10Z) Confirmed `crates/cli/src/ops/screener.rs` is the largest operation adapter at roughly 8k lines.
- [x] (2026-04-28T10:15Z) Moved the existing Screener implementation into `crates/cli/src/ops/screener/engine.rs`.
- [x] (2026-04-28T10:15Z) Added a Screener facade plus state, screens, filters, columns, and validation surface modules.
- [x] (2026-04-28T10:20Z) Archived the completed CLI package relocation ExecPlan.
- [x] (2026-04-28T10:25Z) Updated architecture, development, roadmap, changelog, and continuity docs.
- [x] (2026-04-28T10:40Z) Ran validation and smoke checks.
- [x] (2026-04-28T10:45Z) Prepared the behavior-preserving split for commit.

## Surprises & Discoveries

- Observation: The Screener file is much larger than the next operation adapters.
  Evidence: `wc -l` showed `screener.rs` at about 7,900 lines, followed by `alert.rs` at about 3,000 lines and `layout.rs` at about 2,200 lines.

- Observation: Moving the existing implementation under the nested `screener/engine.rs` path required only import-path adjustment, not behavior changes.
  Evidence: `cargo check --workspace` passed after updating the moved file's parent-module imports.

## Decision Log

- Decision: Do not create a generic `tradingview-ops` crate.
  Rationale: Current operation modules are adapters, not pure domain logic. Moving all of them to a crate would preserve the same mixed responsibilities under a different path.
  Date/Author: 2026-04-28 / Codex.

- Decision: Use a facade plus sub-surface modules before moving implementation bodies.
  Rationale: The existing Screener implementation has dense internal coupling between validation, storage, UI click helpers, JavaScript snippets, and post-check logic. A facade-first split is low-risk, preserves behavior, and creates stable destinations for later deeper moves.
  Date/Author: 2026-04-28 / Codex.

## Outcomes & Retrospective

The public `ops::screener_*` and `ops::validate_screener_*` exports remain unchanged. `crates/cli/src/ops/screener.rs` is now a small facade, while the existing implementation is isolated in `crates/cli/src/ops/screener/engine.rs` behind sub-surface re-export modules. This is intentionally a first-stage adapter split: it makes the Screener surface navigable and prepares later movement of implementation bodies without changing CLI behavior.

Validation passed:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo test -p tradingview-cli screener -- --nocapture`
- `cargo test -p tradingview-cli --test cli_contract screener -- --nocapture`
- `cargo metadata --no-deps --format-version 1`
- `git diff --check`

Behavior smoke passed for Screener help, filters help, columns help, screens help, and structured `TV_CDP_PORT=9 tv screener status` connection failure with exit code 2.

The tracked-doc hygiene grep returned only existing policy text and validation-command examples, including archived plans; no new live account identifiers, credentials, or machine-specific operational values were added.

## Context and Orientation

The repository now has a virtual workspace root, a CLI package under `crates/cli/`, and internal crates for core contracts, Desktop-free reads or analysis, and CDP support. The remaining `ops` modules inside the CLI package are operation adapters. Screener is the clearest next target because it contains screens lifecycle, filters, columns, storage API, UI, and expression logic in one file.

## Plan of Work

Keep `crates/cli/src/ops.rs` and application dispatch unchanged. Split only the Screener adapter module path:

- `crates/cli/src/ops/screener.rs` remains the public operation facade.
- `crates/cli/src/ops/screener/engine.rs` initially contains the existing implementation.
- `state`, `screens`, `filters`, `columns`, and `validation` modules re-export their sub-surface functions from `engine`.

Do not change CLI command names, JSON payloads, errors, validation behavior, or runtime behavior in this slice. Deeper movement from `engine` into the submodules is a later mechanical follow-up once this boundary is stable.

## Concrete Steps

Run commands from the repository root.

1. Move the existing implementation:

        mkdir -p crates/cli/src/ops/screener
        git mv crates/cli/src/ops/screener.rs crates/cli/src/ops/screener/engine.rs

2. Add facade and sub-surface files:

        crates/cli/src/ops/screener.rs
        crates/cli/src/ops/screener/state.rs
        crates/cli/src/ops/screener/screens.rs
        crates/cli/src/ops/screener/filters.rs
        crates/cli/src/ops/screener/columns.rs
        crates/cli/src/ops/screener/validation.rs

3. Update docs and continuity notes.

4. Validate:

        cargo fmt --check
        cargo clippy --workspace --all-targets --all-features -- -D warnings
        cargo test --workspace
        cargo test -p tradingview-cli screener -- --nocapture
        cargo test -p tradingview-cli --test cli_contract screener -- --nocapture
        cargo metadata --no-deps --format-version 1
        git diff --check

5. Run behavior smoke:

        target/debug/tv screener --help
        target/debug/tv screener filters --help
        target/debug/tv screener columns --help
        target/debug/tv screener screens --help
        TV_CDP_PORT=9 target/debug/tv screener status

## Validation and Acceptance

Acceptance requires all existing Screener unit tests and CLI contract tests to pass. `cargo metadata` should still show the same workspace members. `TV_CDP_PORT=9 tv screener status` should still return a structured connection error. No command output or payload shape should change.

## Idempotence and Recovery

This split is mechanical. If imports fail, first confirm that `crates/cli/src/ops/screener.rs` declares all submodules and re-exports the same functions that `crates/cli/src/ops.rs` imports. If tests fail because private helper visibility changed, keep the implementation in `engine` for this slice rather than moving helper bodies prematurely.

## Artifacts and Notes

Do not record live Screener screen names, account-local ids, or raw TradingView payloads in tracked docs. This slice should not require live mutation smoke.

## Interfaces and Dependencies

No public CLI interface changes. Internally, the Screener adapter path becomes:

    crates/cli/src/ops/screener.rs
    crates/cli/src/ops/screener/engine.rs
    crates/cli/src/ops/screener/{state,screens,filters,columns,validation}.rs

## Open Questions

No blocking open questions. The next deeper split should move implementation bodies out of `engine` one sub-surface at a time, likely starting with validation or columns because they have clearer boundaries than visible UI helpers.
