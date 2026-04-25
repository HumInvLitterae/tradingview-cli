# Add replay basic controls

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root.

## Purpose / Big Picture

The old JavaScript `tv` CLI exposed `tv replay start/step/stop/status/autoplay/trade`. After this change, the Rust CLI can start replay, advance one replay bar, and stop replay with `tv replay start`, `tv replay step`, and `tv replay stop`.

This is the next replay migration slice because `replay status` is already implemented and the basic replay lifecycle is incomplete without a bounded way to start and stop replay. This slice intentionally excludes autoplay and replay trade commands.

## Progress

- [x] (2026-04-24 00:00Z) Confirmed the worktree was clean before starting.
- [x] (2026-04-24 00:00Z) Inspected old JavaScript `replay` CLI and core implementation.
- [x] (2026-04-24 00:00Z) Created this ExecPlan and recorded that autoplay and trade remain deferred.
- [x] (2026-04-24 00:00Z) Add Rust CLI surface and dispatch for `tv replay start/step/stop`.
- [x] (2026-04-24 00:00Z) Implement replay basic controls in `src/ops/replay.rs`.
- [x] (2026-04-24 00:00Z) Add unit and CLI contract tests.
- [x] (2026-04-24 00:00Z) Update README, migration inventory, lifecycle audit, contract note, handoff docs, and agent guide.
- [x] (2026-04-24 00:00Z) Run automated validation baseline.
- [x] (2026-04-24 00:00Z) Run live TradingView Desktop smoke if a CDP session is available and record the result.
- [x] (2026-04-24 00:00Z) Commit the completed slice.

## Surprises & Discoveries

No surprises have been discovered yet.

## Decision Log

- Decision: Implement `replay start`, `replay step`, and `replay stop` together.
  Rationale: They form the smallest useful replay lifecycle: enter replay mode, advance a bounded amount, and return to realtime.
  Date/Author: 2026-04-24 / Codex

- Decision: Keep `replay autoplay` and `replay trade` deferred.
  Rationale: Autoplay can change persistent replay speed settings, and trade creates replay position state. Both need separate safety plans.
  Date/Author: 2026-04-24 / Codex

## Outcomes & Retrospective

Implemented `tv replay start [--date <YYYY-MM-DD>]`, `tv replay step`, and `tv replay stop` as the bounded replay lifecycle slice. `tv replay status` remains available, and replay autoplay/trade remain deferred.

Automated validation passed:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    run the repository's standard tracked-doc local absolute path scan

Live TradingView Desktop smoke passed:

    cargo run --quiet -- replay status
    cargo run --quiet -- replay start --date 2026-04-01
    cargo run --quiet -- replay step
    cargo run --quiet -- replay stop
    cargo run --quiet -- replay status

The live start returned `action: "started"` and `current_date: 1775001599`. The live step advanced from `1775001599` to `1775073599`. The live stop returned `action: "replay_stopped"`, and the final status returned `is_replay_started: false`.

## Context and Orientation

This repository implements a Rust-native command-line binary named `tv`. The entrypoint in `src/main.rs` parses commands, calls operation functions re-exported from `src/ops.rs`, and prints JSON envelopes.

The previous replay slice implemented `tv replay status`. The old JavaScript replay implementation starts replay through `window.TradingViewApi._replayApi`, waits until replay is initialized, steps one bar with `doStep()`, and stops with `stopReplay()`.

## Plan of Work

First, extend the existing `ReplayCommand` enum in `src/cli.rs` with `start`, `step`, and `stop`. `start` accepts optional `--date <YYYY-MM-DD>`.

Next, update `src/main.rs` dispatch. Validate replay start date before connecting to CDP, then connect to the current chart runtime and call the replay operation.

Then, extend `src/ops/replay.rs`. `replay_start` should verify replay availability, show the replay toolbar, select the requested date or first available date, and poll until replay has started and `currentDate` is non-null. `replay_step` should require an active replay session and poll briefly for `currentDate` to change. `replay_stop` should return `already_stopped` if replay was not active, or stop replay and return `replay_stopped`.

Finally, update tests and durable docs. Unit tests should cover date validation, unavailable replay, step without start, step success, and stop success. CLI contract tests should cover help output, invalid date validation, and connection failures.

## Concrete Steps

From the repository root, run the implementation loop:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    run the repository's standard tracked-doc local absolute path scan

If TradingView Desktop is available through CDP, run:

    cargo run --quiet -- replay status
    cargo run --quiet -- replay start --date 2026-04-01
    cargo run --quiet -- replay step
    cargo run --quiet -- replay stop
    cargo run --quiet -- replay status

If start fails because the selected date has no replay data, record the failure and do not search across dates by trial and error.

## Validation and Acceptance

The change is accepted when `tv replay --help` lists `status`, `start`, `step`, and `stop`, does not list `autoplay` or `trade`, and the automated validation baseline passes.

The success JSON must use the Rust envelope. `replay start` should expose `replay_started`, `date`, and `current_date`; `replay step` should expose `action`, `previous_date`, and `current_date`; `replay stop` should expose `action` and `replay_started`.

## Idempotence and Recovery

Automated tests are safe to rerun and must not require TradingView Desktop.

Live smoke mutates chart session state. Always attempt `tv replay stop` after a successful `tv replay start`, even if `step` fails.

## Artifacts and Notes

- `src/ops/replay.rs` contains replay start, step, stop, status, date validation, and replay API error mapping.
- `tests/cli_contract.rs` covers replay help, invalid date validation, and CDP connection failure envelope behavior.
- `README.md`, `AGENTS.md`, contract notes, and migration notes now list replay start/step/stop/status as implemented while keeping replay autoplay/trade deferred.

## Interfaces and Dependencies

At the end of the implementation, these commands must exist:

    tv replay start [--date <YYYY-MM-DD>]
    tv replay step
    tv replay stop
    tv replay status

The operation facade must expose:

    pub async fn replay_start(runtime: &mut impl RuntimeEvaluator, date: Option<&str>) -> Result<Value, AppError>
    pub async fn replay_step(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>
    pub async fn replay_stop(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>

No new third-party Rust dependencies are required.

## Open Questions

There are no unresolved critical questions at the plan stage. If live TradingView behavior differs from the old JavaScript assumptions, record the discovery here and keep autoplay and trade commands out of this slice.
