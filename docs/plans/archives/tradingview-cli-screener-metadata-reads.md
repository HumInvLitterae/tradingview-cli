# Add Screener metadata read commands

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, a user can inspect the current TradingView Stock Screener setup without reading full Screener rows. The new commands are `tv screener screens active`, `tv screener filters list`, and `tv screener columns list`. They are read-only metadata commands: they may open and close the visible Screener dialog to read state, but they do not save screens, remove filters, change columns, or persist account state.

This builds on the existing `tv screener status/open/get/close` implementation. `get` remains the command for visible row data. The new commands are lighter shortcuts for the active screen title, visible filter pills, and visible table columns.

## Progress

- [x] (2026-04-26 03:05Z) Read the existing UI Screener ExecPlan, upstream PR #66 summary, current `src/ops/screener.rs`, CLI dispatch, and tests.
- [x] (2026-04-25 16:49Z) Added `screens active`, `filters list`, and `columns list` CLI surface and operation functions.
- [x] (2026-04-25 16:49Z) Added unit and CLI contract tests.
- [x] (2026-04-25 16:49Z) Updated README and migration/upstream notes.
- [x] (2026-04-25 16:50Z) Ran automated validation and live smoke.
- [x] (2026-04-25 16:50Z) Recorded outcomes and prepared the completed slice for commit.

## Surprises & Discoveries

- Observation: The existing `tv screener get` implementation already reads `screen_title`, `filters`, and `columns`.
  Evidence: `src/ops/screener.rs` returns these fields from `readScreenerState`.

- Observation: Live smoke found an active Stock Screener screen title and visible columns, while no filters were currently selected.
  Evidence: `tv screener screens active` returned `screen_title: "米国株"`, `tv screener filters list` returned `filter_count: 0`, and `tv screener columns list` returned `column_count: 13`.

## Decision Log

- Decision: Add read-only metadata commands before any Screener mutation commands.
  Rationale: The user wants Screener functionality to become convenient over time, and metadata reads provide useful operator context without risking TradingView screen/filter/column state.
  Date/Author: 2026-04-26 / Codex.

- Decision: Reuse the same temporary open and restore behavior as `tv screener get`.
  Rationale: Users should not need to manually open the dialog for metadata reads, and smoke should leave the dialog in the same open/closed state it started with.
  Date/Author: 2026-04-26 / Codex.

## Outcomes & Retrospective

Implemented `tv screener screens active`, `tv screener filters list`, and
`tv screener columns list` as read-only UI Screener metadata commands. The
shared temporary open/read/restore helper keeps `tv screener get` behavior
intact while allowing lighter metadata reads.

Validation passed:

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `git diff --check`
- `git grep -nE '(/Users/|C:\\|USER;)' -- README.md AGENTS.md docs .agents/skills || true`

The grep command reported only tracked command examples that intentionally show
the grep pattern itself; no live account identifiers or machine-specific paths
were added.

Live smoke with an explicit `TV_CDP_TARGET_ID` passed. The initial Screener
state was closed, each metadata read opened the dialog temporarily with
`opened_for_read: true` and `restored_open_state: false`, and the final status
was closed again.

## Context and Orientation

The command-line parser is in `src/cli.rs`. The current `ScreenerCommand` enum already defines `status`, `open`, `get`, and `close`. Nested command groups can be added with clap's `#[command(subcommand)]` pattern.

The command dispatch is in `src/main.rs`. It connects to TradingView Desktop through CDP for Screener commands, then calls operation functions in `src/ops/screener.rs`.

The operation module `src/ops/screener.rs` reads the visible TradingView Stock Screener dialog through JavaScript evaluated in the page context. The operation functions return only the payload that appears under the top-level `data` envelope.

## Plan of Work

Add three new nested command groups under `tv screener`: `screens active`, `filters list`, and `columns list`.

Refactor `src/ops/screener.rs` so the temporary open / read / close-restore behavior is shared by `get` and the new metadata functions. Keep the existing `get` payload shape intact.

Add `screener_screens_active`, `screener_filters_list`, and `screener_columns_list` operation functions. These functions should return `source: "ui_screener_dialog"`, `opened_for_read`, and `restored_open_state`. `filters list` should normalize filters to include `index`, `text`, `data_name`, and `visible`. `columns list` should normalize columns to include `index` and `name`.

Update docs to mark these metadata read commands as implemented while keeping filter removal, screen save/switch, and column mutation deferred.

## Concrete Steps

Run commands from the repository root.

1. Edit `src/cli.rs`, `src/main.rs`, and `src/ops/screener.rs`.
2. Add focused tests in `src/ops/screener.rs` and `tests/cli_contract.rs`.
3. Update `README.md`, `docs/notes/rust-cli-contract-migration-2026-04-24.md`, `docs/notes/screener-hotlist-upstream-feasibility-2026-04-25.md`, `docs/notes/upstream-pr-triage-2026-04-25.md`, and `docs/notes/next-agent-handoff-prompt-2026-04-24.md`.
4. Run:

        cargo fmt --check
        cargo clippy --all-targets --all-features -- -D warnings
        cargo test
        git diff --check
        git grep -nE '(/Users/|C:\\|USER;)' -- README.md AGENTS.md docs .agents/skills || true

5. If TradingView Desktop is available, run:

        tv tab list
        TV_CDP_TARGET_ID=<target> tv screener status
        TV_CDP_TARGET_ID=<target> tv screener screens active
        TV_CDP_TARGET_ID=<target> tv screener filters list
        TV_CDP_TARGET_ID=<target> tv screener columns list
        TV_CDP_TARGET_ID=<target> tv screener status

The final status should match the initial open/closed state.

## Validation and Acceptance

The change is accepted when `tv screener --help` lists `screens`, `filters`, and `columns`; each group lists the intended read subcommand; and all new commands return successful payloads from a live TradingView Desktop session without modifying filter, screen, or column state.

Automated tests must prove that metadata reads restore a closed dialog, leave an initially open dialog open, and normalize filters and columns. Full baseline must pass.

## Idempotence and Recovery

The commands are designed to be repeatable. If the Screener dialog is closed, metadata reads may open it temporarily and should close it afterward. If it is already open, they should leave it open. If live smoke leaves the dialog open unexpectedly, run `tv screener close` or press Escape in TradingView Desktop.

## Artifacts and Notes

Do not paste raw Screener table rows into tracked docs. Metadata such as filter names, column names, counts, and final open state may be summarized.

## Interfaces and Dependencies

At completion, `src/ops/screener.rs` must additionally expose:

    pub async fn screener_screens_active(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>;
    pub async fn screener_filters_list(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>;
    pub async fn screener_columns_list(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>;

No new crate dependencies are required.

## Open Questions

None.
