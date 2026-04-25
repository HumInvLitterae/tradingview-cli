# Add drawing clear command

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, a user can run `tv draw clear` to remove all chart-local drawings from the current TradingView chart, or `tv draw clear --dry-run` to inspect the targets without deleting them. This closes the remaining old JavaScript drawing lifecycle command while keeping the Rust CLI safer than the original bridge.

The old JavaScript CLI exposed `draw clear` as a direct `removeAllShapes()` call. The Rust CLI should preserve that practical capability, but must add a read-only dry-run, preflight target reporting, and post-delete verification before treating the operation as successful.

## Progress

- [x] (2026-04-25 01:05Z) Read the current drawing CLI dispatch, operation implementation, CLI contract tests, old JavaScript `draw clear` implementation, and deferred surface notes.
- [x] (2026-04-25 01:05Z) Created this ExecPlan.
- [x] (2026-04-25 01:18Z) Add `tv draw clear [--dry-run]` CLI surface and dispatch.
- [x] (2026-04-25 01:18Z) Implement dry-run and clear operations with before/after shape counts.
- [x] (2026-04-25 01:18Z) Add fake-runtime and CLI contract tests.
- [x] (2026-04-25 01:28Z) Update README, AGENTS, migration inventory, lifecycle audit, contract notes, handoff note, and remaining deferred audit.
- [x] (2026-04-25 01:34Z) Run automated validation.
- [x] (2026-04-25 01:36Z) Run conditional live smoke; destructive smoke skipped because dry-run found two existing drawings.
- [x] (2026-04-25 01:50Z) Run approved AAPL destructive live smoke with one identifiable smoke drawing.
- [x] (2026-04-25 01:42Z) Commit the completed slice.

## Surprises & Discoveries

The first live dry-run found two existing drawings in the current TradingView chart. Because they were pre-existing, destructive smoke correctly stopped before calling `tv draw clear`.

After explicit approval to smoke AAPL without preserving leftover drawings, the CLI switched to an AAPL chart, created one identifiable horizontal-line smoke drawing, cleared it with `tv draw clear`, and confirmed `tv draw list` returned zero shapes.

## Decision Log

- Decision: Implement `draw clear` without a `--yes` flag.
  Rationale: A confirmation flag that is validated before connecting to TradingView does not improve safety for this workflow. Safety should come from `--dry-run`, target reporting, and post-delete verification.
  Date/Author: 2026-04-25 / Codex.

- Decision: Make `--dry-run` the read-only inspection path.
  Rationale: Operators and smoke tests need a way to see exactly what would be deleted before running the destructive command.
  Date/Author: 2026-04-25 / Codex.

- Decision: Treat residual drawings after `removeAllShapes()` as `internal_api_unavailable`.
  Rationale: A destructive command must not report success when TradingView did not clear every target the command said it would clear.
  Date/Author: 2026-04-25 / Codex.

## Outcomes & Retrospective

Implemented `tv draw clear [--dry-run]` as a bulk chart-local drawing cleanup command. The dry-run path is read-only and returns the targets that would be cleared. The normal path returns a no-op when no drawings exist, otherwise calls TradingView's all-shapes removal API and requires the post-clear count to be zero.

## Context and Orientation

The Rust binary is named `tv`. Command-line shape is defined in `src/cli.rs`. Runtime dispatch is in `src/main.rs`. Drawing operations are grouped in `src/ops/drawing.rs` and re-exported through `src/ops.rs`.

Current drawing commands already expose `tv draw shape`, `tv draw list`, `tv draw get`, and `tv draw remove`. The old JavaScript implementation also had `draw clear`, implemented as a bulk `removeAllShapes()` call. The Rust implementation should not use the old JSON envelope; successful command-specific fields live under top-level `data`.

## Plan of Work

First add a `Clear { dry_run: bool }` variant to `DrawingCommand` in `src/cli.rs`, then dispatch it from `src/main.rs` to a new `ops::drawing_clear` function.

Then implement `drawing_clear` in `src/ops/drawing.rs`. The JavaScript expression should read `getAllShapes()` before any mutation, map each target to `{ id, name }`, and return immediately for `--dry-run` with `action: "dry_run"`, `dry_run: true`, `before_count`, `would_clear_count`, `cleared_entities`, and `source`.

For normal execution, if there are no drawings, return a successful no-op payload with `action: "noop"`, `cleared: false`, `before_count: 0`, `after_count: 0`, `cleared_entities: []`, and `source`. If there are drawings, call `removeAllShapes()`, read `getAllShapes()` again, and return `action: "cleared"`, `cleared`, `before_count`, `after_count`, `cleared_entities`, and `source`. Rust should reject any non-dry-run result where `after_count` is not zero.

Finally update tests and durable docs. The migration inventory and remaining deferred audit should no longer list `draw clear` as deferred, while still leaving Pine raw compile, bulk alert delete, alert edit/pause/resume, and generic UI automation deferred.

## Concrete Steps

Run commands from the repository root.

Targeted validation while implementing:

    cargo test ops::drawing::tests::drawing_clear -- --nocapture
    cargo test --test cli_contract draw -- --nocapture

Full validation before commit:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    rg -n "(/[U]sers/|[C]:\\\\)" README.md AGENTS.md docs .agents/skills || true

## Validation and Acceptance

Automated acceptance is that tests prove help output, connection-attempt behavior, dry-run payload pass-through, clear success, no-op success, and residual-shape failure.

Live smoke is allowed only with the following guardrail:

1. Run `tv draw clear --dry-run`.
2. If it reports existing drawings, skip destructive smoke and record the blocker.
3. If it reports no existing drawings, create one identifiable smoke drawing, run `tv draw clear`, then confirm `tv draw list` reports zero drawings.
4. If cleanup fails, record the created entity ID and remove it with `tv draw remove <ENTITY_ID>` if possible.

Never delete pre-existing user drawings during live smoke.

## Idempotence and Recovery

Source and docs edits are ordinary additive changes and can be rerun. Automated tests use fake runtime responses and do not require TradingView Desktop.

If live smoke creates a disposable drawing and `draw clear` fails, use the recorded entity ID with `tv draw remove <ENTITY_ID>` to clean up. If pre-existing drawings are present in dry-run output, stop before mutation.

## Artifacts and Notes

- Targeted validation passed: `cargo test ops::drawing::tests::drawing_clear -- --nocapture`.
- Targeted CLI validation passed: `cargo test --test cli_contract draw -- --nocapture`.
- Full validation passed: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `git diff --check`.
- Tracked docs local absolute path scan passed with `rg -n '(/[U]sers/|[C]:\\\\)' README.md AGENTS.md docs .agents/skills || true`.
- Live smoke dry-run passed and reported two existing drawings: `dMlruO` and `vlHUFh`, both named `trend_line`.
- Destructive live smoke was skipped to avoid deleting pre-existing drawings.
- Approved AAPL destructive smoke passed:
  - `tv symbol NASDAQ:AAPL` observed `BATS:AAPL`.
  - `tv draw clear --dry-run` reported `before_count: 0`.
  - `tv draw shape --type horizontal_line --price 271.06 --time 1777037400 --text "Codex draw clear smoke AAPL 20260425"` created entity `dyTHP5`.
  - `tv draw clear` returned `cleared: true`, `before_count: 1`, `after_count: 0`, and `cleared_entities[0].id: "dyTHP5"`.
  - `tv draw list` returned `count: 0`.

## Interfaces and Dependencies

At completion, the CLI exposes:

    tv draw clear
    tv draw clear --dry-run

At completion, `src/ops/drawing.rs` exposes:

    pub async fn drawing_clear(runtime: &mut impl RuntimeEvaluator, dry_run: bool) -> Result<Value, AppError>

No new crates are required.

## Open Questions

No unresolved critical questions remain for this slice.
