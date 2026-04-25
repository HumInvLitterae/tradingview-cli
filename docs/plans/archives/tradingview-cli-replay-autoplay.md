# Add replay autoplay command

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root.

## Purpose / Big Picture

The old JavaScript `tv` CLI exposed `tv replay autoplay` to toggle TradingView bar replay autoplay, optionally setting an autoplay delay. After this change, the Rust CLI can toggle replay autoplay with `tv replay autoplay` and can set one of TradingView's known safe delays with `tv replay autoplay --speed <MS>`.

This matters because `replay start`, `replay step`, `replay stop`, and `replay status` are already implemented, so autoplay is the next bounded replay control that helps operators and downstream replay-practice workflows. This slice intentionally excludes `replay trade`, because trade commands create replay position state and need a separate safety plan.

## Progress

- [x] (2026-04-24 00:00Z) Confirmed the worktree was clean before starting.
- [x] (2026-04-24 00:00Z) Inspected the old JavaScript `replay autoplay` CLI, core implementation, unit tests, and live smoke behavior.
- [x] (2026-04-24 00:00Z) Created this ExecPlan and recorded the safety boundary.
- [x] (2026-04-24 00:00Z) Add Rust CLI surface and dispatch for `tv replay autoplay [--speed <MS>]`.
- [x] (2026-04-24 00:00Z) Implement replay autoplay in `src/ops/replay.rs`.
- [x] (2026-04-24 00:00Z) Add unit and CLI contract tests.
- [x] (2026-04-24 00:00Z) Update README, migration inventory, lifecycle audit, contract note, handoff docs, and agent guide.
- [x] (2026-04-24 00:00Z) Run automated validation baseline.
- [x] (2026-04-24 00:00Z) Run live TradingView Desktop smoke if a CDP session is available and record the result.
- [x] (2026-04-24 00:00Z) Commit the completed slice.

## Surprises & Discoveries

- Observation: The live autoplay toggle advanced the replay current date while autoplay was active.
  Evidence: Live smoke showed status moving from `current_date: 1775001599` at start to `current_date: 1775073599` after autoplay before cleanup.

## Decision Log

- Decision: Implement `replay autoplay` as its own bounded replay-control slice and keep `replay trade` deferred.
  Rationale: Autoplay mutates replay session state and may change the replay speed setting, but it is still recoverable by toggling autoplay off and stopping replay. Replay trade creates position state and has a higher blast radius.
  Date/Author: 2026-04-24 / Codex

- Decision: Validate autoplay speed before opening a CDP connection.
  Rationale: The old JavaScript implementation warns that invalid values can corrupt account-level replay speed state. Rust should reject unsafe delay values before any page mutation can happen.
  Date/Author: 2026-04-24 / Codex

- Decision: Treat omitted `--speed` and `--speed 0` as "toggle without changing delay".
  Rationale: This preserves the old CLI's practical behavior while avoiding unnecessary persistent speed changes during live smoke.
  Date/Author: 2026-04-24 / Codex

## Outcomes & Retrospective

Implemented `tv replay autoplay [--speed <MS>]` as the next bounded replay control. Invalid speed values are rejected before connecting to CDP. Omitted speed and `--speed 0` toggle autoplay without changing the current delay. Replay trade remains deferred.

Automated validation passed:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    run the repository's standard tracked-doc local absolute path scan

Live TradingView Desktop smoke passed:

    cargo run --quiet -- replay status
    cargo run --quiet -- replay start --date 2026-04-01
    cargo run --quiet -- replay autoplay
    cargo run --quiet -- replay status
    cargo run --quiet -- replay autoplay
    cargo run --quiet -- replay stop
    cargo run --quiet -- replay status

The live start returned `action: "started"` and `current_date: 1775001599`. The first autoplay command returned `autoplay_active: true`, `delay_ms: 1000`, and `requested_delay_ms: null`. Cleanup autoplay returned `autoplay_active: false`. Stop returned `action: "replay_stopped"`, and final status returned `is_replay_started: false` and `is_autoplay_started: false`.

## Context and Orientation

This repository implements a Rust-native command-line binary named `tv`. The entrypoint in `src/main.rs` parses commands, calls operation functions re-exported from `src/ops.rs`, and prints JSON envelopes. Success payloads live under the top-level `data` field.

Replay operations are implemented in `src/ops/replay.rs`. Existing commands include `tv replay start [--date <YYYY-MM-DD>]`, `tv replay step`, `tv replay stop`, and `tv replay status`. They use TradingView's private page object `window.TradingViewApi._replayApi`, so missing or changed private methods should become `internal_api_unavailable` errors rather than empty successes.

The old JavaScript source in the migration repository implements `autoplay({ speed })` with the valid delay list `100, 143, 200, 300, 1000, 2000, 3000, 5000, 10000`. It rejects invalid positive speeds before any CDP calls, requires replay to be started, optionally calls `changeAutoplayDelay(speed)`, then calls `toggleAutoplay()`, reads `isAutoplayStarted()`, reads `autoplayDelay()`, and returns `autoplay_active` and `delay_ms`.

## Plan of Work

First, extend `src/cli.rs`. Add `Autoplay { speed: Option<u64> }` to `ReplayCommand` with `--speed` and `-s`. Do not add `trade` in this slice.

Next, update `src/main.rs` dispatch. For `ReplayCommand::Autoplay`, validate any provided speed before connecting to CDP, then connect to the current chart runtime and call the replay operation. This keeps invalid speed values from touching the live TradingView page.

Then, extend `src/ops.rs` to re-export the new replay operation and validation helper. Extend `src/ops/replay.rs` with `validate_replay_autoplay_speed(speed: u64)` and `replay_autoplay(runtime, speed)`. The validation helper should accept `0` and the known safe delay values. The operation should also call the helper defensively before evaluating JavaScript.

The evaluated JavaScript should check for `window.TradingViewApi._replayApi` and require `isReplayStarted`, `toggleAutoplay`, `isAutoplayStarted`, and `autoplayDelay`. If a positive speed was requested, it should also require `changeAutoplayDelay`. If replay is not started, return a validation error telling the operator to use replay start first. If a positive speed was requested, call `changeAutoplayDelay(requestedDelay)` before toggling. For omitted speed and speed `0`, do not call `changeAutoplayDelay`. On success, return an action-bearing payload with `action: "autoplay"`, `autoplay_active`, `delay_ms`, `requested_delay_ms`, and `source: "internal_api"`.

Finally, update tests and durable docs. Unit tests should cover all accepted speeds, invalid speed validation before evaluation, omitted and zero speed behavior, not-started replay mapping, and practical success fields. CLI contract tests should cover help output, invalid speed validation before connection, and CDP connection failure for a valid speed.

## Concrete Steps

From the repository root, run the implementation loop:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    run the repository's standard tracked-doc local absolute path scan

If TradingView Desktop is available through CDP, run this bounded smoke:

    cargo run --quiet -- replay status
    cargo run --quiet -- replay start --date 2026-04-01
    cargo run --quiet -- replay autoplay
    cargo run --quiet -- replay status
    cargo run --quiet -- replay autoplay
    cargo run --quiet -- replay stop
    cargo run --quiet -- replay status

The smoke should not set a new autoplay speed. If the first autoplay command turns autoplay on, the second autoplay command should turn it back off before stopping replay. Always attempt `tv replay stop` after a successful start, even if autoplay fails.

## Validation and Acceptance

The change is accepted when `tv replay --help` lists `status`, `start`, `step`, `stop`, and `autoplay`, still does not list `trade`, and the automated validation baseline passes.

The success JSON must use the Rust envelope. `tv replay autoplay` should print `success: true`, `command: "replay"`, and `data` containing at least `action`, `autoplay_active`, `delay_ms`, `requested_delay_ms`, and `source`. Invalid speeds such as `500` must fail with a validation envelope before any CDP connection attempt.

Live smoke is accepted when replay starts, autoplay can be toggled, autoplay is toggled back off if needed, replay stops, and the final status reports `is_replay_started: false`.

## Idempotence and Recovery

Automated tests are safe to rerun and must not require TradingView Desktop.

Live smoke mutates chart session state. The recovery path is to run `tv replay autoplay` again if autoplay is active, then run `tv replay stop`. The smoke intentionally does not pass `--speed`, so it avoids changing the replay delay setting. If a command fails after replay starts, run `cargo run --quiet -- replay stop` as cleanup and record the failure in this plan.

## Artifacts and Notes

Expected changed files are `src/cli.rs`, `src/main.rs`, `src/ops.rs`, `src/ops/replay.rs`, `tests/cli_contract.rs`, `README.md`, `AGENTS.md`, `docs/notes/legacy-cli-command-migration-inventory-2026-04-24.md`, `docs/notes/rust-cli-contract-migration-2026-04-24.md`, `docs/notes/command-lifecycle-balance-audit-2026-04-24.md`, and `docs/notes/next-agent-handoff-prompt-2026-04-24.md`.

## Interfaces and Dependencies

At the end of the implementation, this command must exist:

    tv replay autoplay [--speed <MS>]

The Rust operation interface should be:

    pub fn validate_replay_autoplay_speed(speed: u64) -> Result<(), AppError>
    pub async fn replay_autoplay(runtime: &mut impl RuntimeEvaluator, speed: Option<u64>) -> Result<Value, AppError>

The supported speed values are:

    0, 100, 143, 200, 300, 1000, 2000, 3000, 5000, 10000

`0` means "do not change the current speed"; it is not sent to TradingView as a delay.

## Open Questions

There are no unresolved critical questions. If live TradingView behavior differs from the old JavaScript assumptions, record that discovery here and keep `replay trade` out of this slice.
