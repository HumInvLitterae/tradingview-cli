# Add Pine shape signal reads

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root.

## Purpose / Big Picture

TradingView Pine indicators often use `plotshape()` and `plotchar()` to show buy, sell, warning, and confirmation signals directly on bars. The Rust CLI can already read Pine-created lines, labels, tables, and boxes, but it cannot read those shape signals. After this change, a user can run `tv data shapes` to inspect recent displayed shape signals without changing the chart, account, layout, or Pine source.

## Progress

- [x] (2026-04-25T10:02:47Z) Confirmed the working tree was clean and inspected the current data command structure.
- [x] (2026-04-25T10:08:44Z) Added the `tv data shapes` command, data operation module, and automated tests.
- [x] (2026-04-25T10:08:44Z) Updated README and durable notes to record the new read-only surface and upstream PR #35 status.
- [x] (2026-04-25T10:08:44Z) Ran the full validation baseline and read-only live smoke.
- [ ] Commit the completed work.

## Surprises & Discoveries

- Observation: Current live TradingView targets are ambiguous unless `TV_CDP_TARGET_ID` is set.
  Evidence: `target/debug/tv status` returned three chart targets and `error: "Multiple TradingView chart targets found"`.

- Observation: The selected live chart did not expose any Pine shape plots, but the command returned the intended empty success payload.
  Evidence: `TV_CDP_TARGET_ID=... target/debug/tv data shapes --count 100` returned `success: true`, `scan_count: 100`, `study_count: 0`, and an empty `studies` array.

## Decision Log

- Decision: Implement the upstream #35 idea as `tv data shapes`, not as `tv data pine-shapes`.
  Rationale: Existing Rust data commands use short object names such as `lines`, `labels`, `tables`, and `boxes`; `shapes` fits that command family and remains clear in help text.
  Date/Author: 2026-04-25 / Codex

- Decision: Place the implementation in `src/ops/data/shapes.rs`.
  Rationale: `src/ops/data/drawings.rs` already handles several Pine drawing-derived reads and should not grow into another oversized module.
  Date/Author: 2026-04-25 / Codex

- Decision: Reject `--count 0` before connecting to CDP and clamp values above 500.
  Rationale: Zero bars cannot produce useful signal data, while clamping large reads follows the existing OHLCV/trades bounded-read pattern and avoids excessive page evaluation work.
  Date/Author: 2026-04-25 / Codex

## Outcomes & Retrospective

The read-only `tv data shapes` command is implemented and documented. It scans visible Pine study shape plots, reports signal metadata and OHLC when TradingView exposes them, validates zero count before CDP connection, and clamps large counts to 500. Automated validation passed, and live smoke confirmed the empty-success behavior on a chart without shape plot data.

## Context and Orientation

The Rust CLI command surface is defined in `src/cli.rs`. Command dispatch lives in `src/main.rs`, where parsed commands are routed to operation functions. Operation functions are re-exported through `src/ops.rs`. Advanced data reads are grouped under `src/ops/data.rs`, which is a thin facade over files in `src/ops/data/`.

The term "shape signal" in this plan means a marker produced by Pine functions such as `plotshape()` or `plotchar()`. TradingView stores these as study plot data, not as user-created drawing primitives. That is why the existing `tv data labels` and `tv data boxes` commands cannot see them.

## Plan of Work

Add a `Shapes` variant to `DataCommand` in `src/cli.rs` with `--filter`, `--count`, and `--verbose` flags. In `src/main.rs`, handle this variant inside `Command::Data` and reject `--count 0` before connecting to TradingView. Values above 500 should be accepted but clamped inside the operation.

Create `src/ops/data/shapes.rs` with `pub async fn data_shapes(runtime: &mut impl RuntimeEvaluator, filter: Option<&str>, count: Option<usize>, verbose: bool) -> Result<Value, AppError>`. The JavaScript expression should inspect the active chart's studies, find plots whose metadata type is `shapes`, scan the most recent bounded bar range, and return study names, shape plot metadata, and signal rows. The Rust summarizer should preserve practical fields such as study name, plot title, shape, location, color, bar index, value, time, and OHLC when available. Verbose mode should add plot id, data index, size, and raw plot values useful for debugging.

Update `src/ops/data.rs` and `src/ops.rs` to re-export `data_shapes`. Add tests next to the new operation using `FakeRuntime`, and update CLI contract tests so `data --help` lists `shapes`, `data shapes --help` lists its flags, `data shapes --count 0` fails with a validation envelope, and `data shapes` attempts a CDP connection when CDP is unavailable.

Update `README.md`, `docs/notes/rust-cli-contract-migration-2026-04-24.md`, and `docs/notes/upstream-pr-triage-2026-04-25.md`. The upstream note should mark PR #35 as addressed by the Rust `tv data shapes` read-only command and remove it from the active recommended-candidate list.

## Concrete Steps

From the repository root, implement the code and docs, then run:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -n 'USER'';' -- README.md docs .agents/skills || true
    git grep -nE '(/Users/|C:\\)' -- README.md docs .agents/skills || true

If TradingView Desktop is available, run read-only smoke:

    target/debug/tv data shapes --count 100

If multiple CDP targets are open, first inspect targets with existing CLI commands and use `TV_CDP_TARGET_ID` to point the smoke at the intended chart. Do not add indicators, alter layouts, or mutate Pine source just to create shape data.

Validation transcript from this implementation:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -n 'USER'';' -- README.md docs .agents/skills || true
    git grep -nE '(/Users/|C:\\)' -- README.md docs .agents/skills || true

All commands completed successfully. Read-only smoke used an explicit target because multiple TradingView chart targets were open:

    TV_CDP_TARGET_ID=... target/debug/tv data shapes --count 100

It returned an empty success payload with `study_count: 0`, which is acceptable for a chart with no visible shape plots.

## Validation and Acceptance

Automated acceptance is that the full baseline passes and the new tests prove command help, count validation, JavaScript string serialization, count clamping, and payload summarization. Behavioral acceptance is that `tv data shapes --count 100` returns a successful JSON envelope with `data.study_count` and `data.studies`; charts with no shape plots should return an empty success result rather than an error.

## Idempotence and Recovery

The implementation is additive and read-only. Tests do not require TradingView Desktop. Live smoke does not change account, chart, layout, or Pine state, so it can be repeated safely. If a smoke command fails because the current chart has no internal study shape data, record the failure mode and keep the automated validation as the primary acceptance evidence.

## Artifacts and Notes

Relevant upstream evidence:

    #35 adds data_get_pine_shapes to read plotshape/plotchar markers from study bar data rather than drawing primitives.

## Interfaces and Dependencies

No new crate dependency is required. The new public CLI interface is:

    tv data shapes [--filter <TEXT>] [--count <N>] [--verbose]

The new Rust operation interface is:

    pub async fn data_shapes(
        runtime: &mut impl RuntimeEvaluator,
        filter: Option<&str>,
        count: Option<usize>,
        verbose: bool,
    ) -> Result<Value, AppError>

## Open Questions

No critical questions are open.
