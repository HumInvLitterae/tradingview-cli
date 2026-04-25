# Add replay status command

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root.

## Purpose / Big Picture

The old JavaScript `tv` CLI exposed `tv replay start/step/stop/status/autoplay/trade`. After this change, the Rust CLI can read the current replay state with `tv replay status`.

This is intentionally a read-only migration slice. Replay start, stepping, autoplay, trade execution, and stop commands mutate chart session state and need their own safety plan. `replay status` gives operators and downstream workflows a safe way to inspect whether replay is available or already started before deciding whether those controls belong in the Rust CLI.

## Progress

- [x] (2026-04-24 00:00Z) Confirmed the worktree was clean before starting.
- [x] (2026-04-24 00:00Z) Inspected old JavaScript `replay` CLI and core implementation.
- [x] (2026-04-24 00:00Z) Created this ExecPlan and recorded that replay controls remain deferred.
- [x] (2026-04-24 00:00Z) Add Rust CLI surface and dispatch for `tv replay status`.
- [x] (2026-04-24 00:00Z) Implement replay status in a new capability module under `src/ops/`.
- [x] (2026-04-24 00:00Z) Add unit and CLI contract tests.
- [x] (2026-04-24 00:00Z) Update README, migration inventory, lifecycle audit, contract note, handoff docs, and agent guide.
- [x] (2026-04-24 00:00Z) Run automated validation baseline.
- [x] (2026-04-24 00:00Z) Run live TradingView Desktop smoke if a CDP session is available and record the result.
- [x] (2026-04-24 00:00Z) Commit the completed slice.

## Surprises & Discoveries

No surprises have been discovered yet.

## Decision Log

- Decision: Implement only `replay status` in this slice.
  Rationale: It is read-only and provides the replay state needed before planning higher-risk replay controls.
  Date/Author: 2026-04-24 / Codex

- Decision: Report missing replay API as `internal_api_unavailable`.
  Rationale: A missing or changed private TradingView API must not be represented as an empty successful replay state.
  Date/Author: 2026-04-24 / Codex

## Outcomes & Retrospective

Implemented `tv replay status` as a read-only command that reads `window.TradingViewApi._replayApi` and returns the practical old CLI replay status fields under the Rust `data` envelope. Replay control commands remain deferred.

Automated validation passed:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    run the repository's standard tracked-doc local absolute path scan

Live TradingView Desktop smoke passed:

    cargo run --quiet -- replay status

The live target returned `is_replay_available: true`, `is_replay_started: false`, `is_autoplay_started: false`, `replay_mode: "ActiveChart"`, and `autoplay_delay: 1000`.

## Context and Orientation

This repository implements a Rust-native command-line binary named `tv`. The entrypoint in `src/main.rs` parses commands, calls operation functions re-exported from `src/ops.rs`, and prints JSON envelopes.

The old JavaScript replay status command reads `window.TradingViewApi._replayApi` and returns replay availability, started state, autoplay state, replay mode, current date, autoplay delay, position, and realized P&L. Rust should preserve that practical information under the Rust `data` envelope.

## Plan of Work

First, extend the CLI surface in `src/cli.rs` with a new top-level `Replay` command and a `ReplayCommand` subcommand enum. The only subcommand in this slice is `status`.

Next, update `src/main.rs` dispatch. `replay status` should connect to the current chart runtime and call the replay operation.

Then, create `src/ops/replay.rs`. Implement `replay_status(runtime)` by evaluating a small JavaScript expression that checks `window.TradingViewApi._replayApi`, unwraps TradingView observable values, and returns the old practical fields. If the API or a required method is missing, return an error payload that Rust maps to `internal_api_unavailable`.

Finally, update tests and durable docs. Unit tests should cover successful status mapping and missing API mapping. CLI contract tests should cover help output and structured connection failures.

## Concrete Steps

From the repository root, run the implementation loop:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    run the repository's standard tracked-doc local absolute path scan

If TradingView Desktop is available through CDP, run:

    cargo run --quiet -- replay status

Do not run replay start, stop, step, autoplay, or trade commands in this slice.

## Validation and Acceptance

The change is accepted when `tv replay --help` lists `status`, does not list replay control commands, and `tv replay status` either returns a success envelope with replay state or a structured `internal_api_unavailable` error when the replay API is not exposed.

The success JSON must use the Rust envelope. `tv replay status` should print a success envelope whose `data` includes `is_replay_available`, `is_replay_started`, `is_autoplay_started`, `replay_mode`, `current_date`, `autoplay_delay`, `position`, and `realized_pnl`.

## Idempotence and Recovery

Automated tests are safe to rerun and must not require TradingView Desktop.

Live smoke is read-only. If status fails with `internal_api_unavailable`, record that result and do not try to enter or exit replay mode as recovery.

## Artifacts and Notes

- `src/ops/replay.rs` contains replay status mapping and missing API error handling.
- `tests/cli_contract.rs` covers replay help and connection failure envelope behavior.
- `README.md`, `AGENTS.md`, contract notes, and migration notes now list `replay status` as implemented while keeping replay controls deferred.

## Interfaces and Dependencies

At the end of the implementation, this command must exist:

    tv replay status

The operation facade must expose:

    pub async fn replay_status(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>

No new third-party Rust dependencies are required.

## Open Questions

There are no unresolved critical questions at the plan stage. If live TradingView behavior differs from the old JavaScript assumptions, record the discovery here and keep replay controls out of this slice.
