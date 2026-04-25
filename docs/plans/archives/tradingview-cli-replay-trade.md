# Add replay trade command

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root.

## Purpose / Big Picture

The old JavaScript `tv` CLI exposed `tv replay trade buy`, `tv replay trade sell`, and `tv replay trade close`. After this change, the Rust CLI can execute the same replay-mode trade actions and report the resulting replay position and realized P&L under the Rust JSON envelope.

This is the next old CLI migration slice because `replay start`, `replay step`, `replay stop`, `replay status`, and `replay autoplay` are already implemented. `replay trade` is the only remaining replay subcommand from the old CLI. It mutates TradingView replay session state, so this slice includes `close` as the cleanup action and intentionally excludes broader surfaces such as `draw clear`, tab creation/closing, Pine, stream, and generic UI automation.

## Progress

- [x] (2026-04-24 00:00Z) Confirmed the worktree was clean before starting.
- [x] (2026-04-24 00:00Z) Inspected the old JavaScript `replay trade` CLI, core implementation, and tests.
- [x] (2026-04-24 00:00Z) Created this ExecPlan and recorded the safety boundary.
- [x] (2026-04-24 00:00Z) Add Rust CLI surface and dispatch for `tv replay trade <buy|sell|close>`.
- [x] (2026-04-24 00:00Z) Implement replay trade in `src/ops/replay.rs`.
- [x] (2026-04-24 00:00Z) Add unit and CLI contract tests.
- [x] (2026-04-24 00:00Z) Update README, migration inventory, lifecycle audit, contract note, handoff docs, and agent guide.
- [x] (2026-04-24 00:00Z) Run automated validation baseline.
- [x] (2026-04-24 00:00Z) Run live TradingView Desktop smoke if a CDP session is available and record the result.
- [x] (2026-04-24 00:00Z) Commit the completed slice.

## Surprises & Discoveries

- Observation: In the live TradingView Desktop smoke, `trade buy` and `trade close` returned success but `position` and `realized_pnl` were both `null`.
  Evidence: The smoke payloads included `action: "buy"` and `action: "close"` with `position: null` and `realized_pnl: null`. Final status also returned `position: null` and `realized_pnl: null`.

## Decision Log

- Decision: Implement `buy`, `sell`, and `close` together.
  Rationale: `buy` and `sell` create replay position state. `close` is the cleanup path and must be available in the same Rust surface.
  Date/Author: 2026-04-24 / Codex

- Decision: Validate trade action before opening a CDP connection.
  Rationale: Invalid user input should fail deterministically without touching the live TradingView page.
  Date/Author: 2026-04-24 / Codex

- Decision: Smoke `buy` followed by `close`, but do not smoke `sell` automatically.
  Rationale: `buy` plus `close` proves the mutation and cleanup path while minimizing replay position state changes. `sell` is covered by unit and command validation tests.
  Date/Author: 2026-04-24 / Codex

## Outcomes & Retrospective

Implemented `tv replay trade <buy|sell|close>` as the final old replay CLI subcommand. Invalid actions are rejected before connecting to CDP. Successful payloads preserve the old CLI's practical fields under the Rust `data` envelope.

Automated validation passed:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    run the repository's standard tracked-doc local absolute path scan

Live TradingView Desktop smoke passed:

    cargo run --quiet -- replay status
    cargo run --quiet -- replay start --date 2026-04-01
    cargo run --quiet -- replay trade buy
    cargo run --quiet -- replay status
    cargo run --quiet -- replay trade close
    cargo run --quiet -- replay stop
    cargo run --quiet -- replay status

The live start returned `action: "started"` and `current_date: 1775001599`. `trade buy` returned `action: "buy"`, `position: null`, and `realized_pnl: null`. Cleanup `trade close` returned `action: "close"`, `position: null`, and `realized_pnl: null`. Stop returned `action: "replay_stopped"`, and final status returned `is_replay_started: false` and `is_autoplay_started: false`.

## Context and Orientation

This repository implements a Rust-native command-line binary named `tv`. Success payloads live under the top-level `data` field and errors live under `error`.

Replay operations are implemented in `src/ops/replay.rs`. Existing commands include `tv replay start [--date <YYYY-MM-DD>]`, `tv replay step`, `tv replay stop`, `tv replay status`, and `tv replay autoplay [--speed <MS>]`. They use TradingView's private page object `window.TradingViewApi._replayApi`, so missing or changed private methods should become `internal_api_unavailable` errors rather than empty successes.

The old JavaScript source implements `trade({ action })` by requiring replay to be started, calling `buy()`, `sell()`, or `closePosition()` on the replay API, then reading `position()` and `realizedPL()`. It returns `action`, `position`, and `realized_pnl` as the practical useful fields.

## Plan of Work

First, extend `src/cli.rs`. Add `Trade { action: String }` to `ReplayCommand`. Do not add other old replay commands because the replay group is otherwise complete.

Next, update `src/main.rs` dispatch. Validate the action with a replay operation helper before connecting to CDP, then connect to the current chart runtime and call the replay trade operation.

Then, extend `src/ops.rs` to re-export the new replay operation and validation helper. Extend `src/ops/replay.rs` with `validate_replay_trade_action(action: &str)` and `replay_trade(runtime, action)`. The helper should accept only `buy`, `sell`, and `close`. The operation should defensively validate again, require the replay API methods `isReplayStarted`, `position`, `realizedPL`, and the selected action method, fail with validation if replay is not started, execute the selected method, then return `action`, `position`, `realized_pnl`, and `source`.

Finally, update tests and durable docs. Unit tests should cover all actions, invalid action validation before evaluation, not-started replay mapping, missing API mapping, and practical success fields. CLI contract tests should cover help output, invalid action validation before connection, and CDP connection failure for a valid action.

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
    cargo run --quiet -- replay trade buy
    cargo run --quiet -- replay status
    cargo run --quiet -- replay trade close
    cargo run --quiet -- replay stop
    cargo run --quiet -- replay status

Always attempt `tv replay trade close` and `tv replay stop` after a successful `replay start`, even if the trade command fails.

## Validation and Acceptance

The change is accepted when `tv replay --help` lists `status`, `start`, `step`, `stop`, `autoplay`, and `trade`, and the automated validation baseline passes.

The success JSON must use the Rust envelope. `tv replay trade buy`, `sell`, or `close` should print `success: true`, `command: "replay"`, and `data` containing at least `action`, `position`, `realized_pnl`, and `source`. Invalid actions such as `hold` must fail with a validation envelope before any CDP connection attempt.

Live smoke is accepted when replay starts, `trade buy` returns a replay trade payload, `trade close` runs as cleanup, replay stops, and final status reports `is_replay_started: false`.

## Idempotence and Recovery

Automated tests are safe to rerun and must not require TradingView Desktop.

Live smoke mutates replay session state. The recovery path is to run `tv replay trade close` and then `tv replay stop`. If a command fails after replay starts, run both cleanup commands as best effort and record the failure in this plan.

## Artifacts and Notes

Expected changed files are `src/cli.rs`, `src/main.rs`, `src/ops.rs`, `src/ops/replay.rs`, `tests/cli_contract.rs`, `README.md`, `AGENTS.md`, `docs/notes/legacy-cli-command-migration-inventory-2026-04-24.md`, `docs/notes/rust-cli-contract-migration-2026-04-24.md`, `docs/notes/command-lifecycle-balance-audit-2026-04-24.md`, and `docs/notes/next-agent-handoff-prompt-2026-04-24.md`.

## Interfaces and Dependencies

At the end of the implementation, this command must exist:

    tv replay trade <buy|sell|close>

The Rust operation interface should be:

    pub fn validate_replay_trade_action(action: &str) -> Result<(), AppError>
    pub async fn replay_trade(runtime: &mut impl RuntimeEvaluator, action: &str) -> Result<Value, AppError>

## Open Questions

There are no unresolved critical questions. If live TradingView behavior differs from the old JavaScript assumptions, record that discovery here and keep larger deferred surfaces out of this slice.
