# Split data operation modules

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root.

## Purpose / Big Picture

The Rust `tv` CLI already implements the advanced read-only `data ...` commands, but their implementation is concentrated in one large `src/ops/data.rs` file. This refactor keeps the user-visible CLI and JSON output unchanged while splitting that operation code by capability. After the change, a future contributor can add or maintain indicator reads, strategy reads, and Pine drawing-derived reads without growing a single catch-all module.

The observable behavior is that the existing commands continue to work and the same test suite passes, while the source tree now contains smaller data operation modules.

## Progress

- [x] (2026-04-24 00:00Z) Confirmed the worktree was clean before starting.
- [x] (2026-04-24 00:00Z) Ran `cargo test`; 46 unit tests and 18 CLI contract tests passed before the refactor.
- [x] (2026-04-24 00:00Z) Split `src/ops/data.rs` into a facade plus `src/ops/data/indicator.rs`, `src/ops/data/strategy.rs`, and `src/ops/data/drawings.rs`.
- [x] (2026-04-24 00:00Z) Ran the full validation baseline: `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test`, `git diff --check`, and the tracked-doc local absolute path scan passed.
- [x] (2026-04-24 00:00Z) Updated `README.md`, `AGENTS.md`, and the current handoff note to record the completed data module split.
- [ ] Commit the code refactor and documentation updates in sensible batches.

## Surprises & Discoveries

- Observation: No unexpected behavior has been discovered yet.
  Evidence: Initial `cargo test` passed before any code was moved.

- Observation: Moving the tests into nested modules did not require any behavior changes.
  Evidence: After the split, `cargo test` still passed with 46 unit tests and 18 CLI contract tests.

## Decision Log

- Decision: Split the data operations into three areas: indicator, strategy, and drawings.
  Rationale: These are the three responsibilities currently mixed in `src/ops/data.rs`: current study values and study input reads, strategy report reads, and Pine drawing object extraction/summarization. This is enough structure to prevent further growth without creating one file per command.
  Date/Author: 2026-04-24 / Codex

- Decision: Keep the existing public `ops::data_*` function names and CLI behavior unchanged.
  Rationale: This is an internal module layout refactor, not a command migration or JSON contract change. Downstream callers should not need to adjust.
  Date/Author: 2026-04-24 / Codex

## Outcomes & Retrospective

The source refactor is complete: `src/ops/data.rs` is now a seven-line facade, and the previous implementation is split into indicator, strategy, and drawing-derived read modules. Public CLI behavior and JSON contracts were intentionally left unchanged. Final validation remains to be run before commit.

Final validation passed. The refactor achieved the intended module split without changing public CLI behavior, JSON contract notes, or command dispatch.

## Context and Orientation

The repository is a Rust-native command-line tool named `tv`. The binary entrypoint is `src/main.rs`, which parses the command line, connects to TradingView Desktop through Chrome DevTools Protocol when needed, calls operation functions under `src/ops.rs`, and prints structured JSON envelopes.

The operation layer already uses a facade pattern: `src/ops.rs` declares modules such as `chart`, `market`, `layout`, and `data`, then re-exports the public operation functions used by `src/main.rs`. The repository uses Rust 2024 module style and must not introduce `mod.rs`. When a module needs submodules, use a facade file and a same-named directory, as in `src/ops/data.rs` plus `src/ops/data/`.

Before this refactor, `src/ops/data.rs` contains all non-depth data operations and their tests. The implemented user-facing commands are `tv values`, `tv data indicator`, `tv data strategy`, `tv data trades`, `tv data equity`, `tv data lines`, `tv data labels`, `tv data tables`, and `tv data boxes`. `tv data depth` is already isolated in `src/ops/data_depth.rs` and is not part of this refactor.

## Plan of Work

First, keep `src/ops.rs` as the top-level operation facade and keep its public re-exports unchanged. Then turn `src/ops/data.rs` into a smaller data facade that declares three private submodules and publicly re-exports the existing data operation function names.

Create `src/ops/data/indicator.rs` for `study_values` and `data_indicator`, including their existing unit tests. These operations read current study values and study input information. They should continue to use `crate::cdp::RuntimeEvaluator`, `crate::error::{AppError, ErrorKind}`, and `super::super::common::{js_string, CHART_API}`.

Create `src/ops/data/strategy.rs` for `data_strategy`, `data_trades`, and `data_equity`, including their existing unit tests. These operations read strategy report, order, and equity information. They should continue to use `MAX_TRADES_COUNT` and `CHART_API` from `src/ops/common.rs`.

Create `src/ops/data/drawings.rs` for `data_lines`, `data_labels`, `data_tables`, `data_boxes`, and their private summarization helpers. These operations read Pine drawing primitives and summarize them into practical JSON. Keep helper functions private inside the module.

Do not change command names, clap definitions, public JSON fields, error mapping, or `src/main.rs` dispatch behavior unless compilation requires an import path adjustment that preserves the same public `ops::...` call shape.

## Concrete Steps

From the repository root, run:

    cargo test

This establishes the pre-refactor baseline. It should pass before code is moved.

Edit the operation modules as described in the plan of work. After each substantial move, run:

    cargo test

At completion, run:

    cargo fmt --check
    cargo clippy --all-targets --all-features
    cargo test
    git diff --check
    run the repository's tracked-doc local absolute path scan

If code formatting is needed, run `cargo fmt`, then repeat `cargo fmt --check`.

## Validation and Acceptance

Acceptance is behavior-preserving:

- `cargo test` passes with the same user-facing commands covered by the existing tests.
- `cargo clippy --all-targets --all-features` passes without warnings.
- `src/ops/data.rs` is a facade rather than a large implementation file.
- `src/ops/data/indicator.rs`, `src/ops/data/strategy.rs`, and `src/ops/data/drawings.rs` exist and contain the moved implementation and tests.
- No tracked documentation contains machine-specific absolute paths.

No live TradingView Desktop CDP smoke is required because this refactor does not change runtime behavior or JSON contract. If a live smoke is run anyway, record it here.

## Idempotence and Recovery

The refactor is safe to retry because it only moves Rust code within the repository and keeps public behavior unchanged. If a moved test fails to compile, inspect the relative module path first, especially references from nested modules to `test_support` and `common`. If the refactor becomes too broad, revert only the uncommitted refactor edits and keep this ExecPlan as the guide for a smaller retry.

## Artifacts and Notes

Initial baseline:

    cargo test
    result: ok. 46 unit tests and 18 CLI contract tests passed.

Post-split test check:

    cargo test
    result: ok. 46 unit tests and 18 CLI contract tests passed.

Final validation:

    cargo fmt --check
    result: passed

    cargo clippy --all-targets --all-features
    result: passed

    cargo test
    result: ok. 46 unit tests and 18 CLI contract tests passed.

    git diff --check
    result: passed

    tracked-doc local absolute path scan
    result: no matches

## Interfaces and Dependencies

The following operation functions must remain available through `crate::ops` at the end of the refactor:

    pub async fn study_values(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>
    pub async fn data_indicator(runtime: &mut impl RuntimeEvaluator, entity_id: &str) -> Result<Value, AppError>
    pub async fn data_strategy(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>
    pub async fn data_trades(runtime: &mut impl RuntimeEvaluator, max_trades: Option<usize>) -> Result<Value, AppError>
    pub async fn data_equity(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>
    pub async fn data_lines(runtime: &mut impl RuntimeEvaluator, filter: Option<&str>, verbose: bool) -> Result<Value, AppError>
    pub async fn data_labels(runtime: &mut impl RuntimeEvaluator, filter: Option<&str>, max_labels: Option<usize>, verbose: bool) -> Result<Value, AppError>
    pub async fn data_tables(runtime: &mut impl RuntimeEvaluator, filter: Option<&str>) -> Result<Value, AppError>
    pub async fn data_boxes(runtime: &mut impl RuntimeEvaluator, filter: Option<&str>, verbose: bool) -> Result<Value, AppError>

No new third-party dependencies are required.

## Open Questions

There are no unresolved critical questions. The chosen default is the three-area split: indicator, strategy, and drawings.
