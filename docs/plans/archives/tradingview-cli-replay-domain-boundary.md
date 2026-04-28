# Replay domain boundary

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md` in this repository. It is self-contained and describes a behavior-preserving refactor that introduces a third small domain/service boundary inside the CLI package.

## Purpose / Big Picture

`domain::watchlist` and `domain::alert` proved that CDP-free validation and payload normalization can move out of operation adapters without changing CLI behavior. This slice applies the same pattern to Replay.

After completion, `ops/replay` remains the adapter that executes TradingView Replay page-session API calls through CDP. `domain::replay` owns date/speed/action validation, replay timestamp conversion, and replay action/status payload normalization. Users should see no command behavior change.

## Progress

- [x] (2026-04-29) Inspected `ops/replay` and confirmed validation plus payload normalization are CDP-free.
- [x] (2026-04-29) Archived the completed Alert domain-boundary plan and created this plan.
- [x] (2026-04-29) Added `domain::replay` for Replay validation, timestamp parsing, and payload normalization.
- [x] (2026-04-29) Kept Replay API availability checks, page-session JavaScript, runtime evaluation, and post-check execution in `ops/replay`.
- [x] (2026-04-29) Updated stable docs and continuity ledger.
- [x] (2026-04-29) Ran validation, behavior smoke, and hygiene checks.
- [ ] Commit the related changes as one refactor.

## Surprises & Discoveries

- Observation: Replay is an even cleaner domain-boundary sample than Drawing.
  Evidence: `ops/replay/validation.rs` and `ops/replay/payload.rs` already contained no `RuntimeEvaluator`, DOM, or page-session JavaScript dependencies. Drawing validation is also pure, but its request types are more tightly shaped around drawing command construction.

## Decision Log

- Decision: Add `crates/cli/src/domain/replay.rs` inside the existing CLI package rather than creating a new workspace crate.
  Rationale: The domain layer is still being proven. Keeping Replay inside the CLI package avoids prematurely publishing a stable Rust API while still separating pure logic from operation adapters.
  Date/Author: 2026-04-29 / Codex.
- Decision: Leave `ops/replay/validation.rs` and `ops/replay/payload.rs` as thin re-export modules for now.
  Rationale: Existing Replay submodules already import through those modules. Thin re-exports keep the behavior-preserving diff small while making the implementation owner `domain::replay`.
  Date/Author: 2026-04-29 / Codex.

## Outcomes & Retrospective

At completion, `domain::replay` should be the third example of the in-package domain/service layer. It should be testable without a fake CDP runtime and should contain no clap command enum, `RuntimeEvaluator`, DOM, or page-session JavaScript dependency.

## Context and Orientation

The relevant files are:

- `crates/cli/src/domain.rs` and `crates/cli/src/domain/replay.rs`
- `crates/cli/src/ops/replay.rs` and `crates/cli/src/ops/replay/`
- `docs/architecture.md`, `docs/development.md`, `docs/v0.3-roadmap.md`, and `CHANGELOG.md`

Rust 2024 is used in this repository. Do not introduce `mod.rs`.

## Plan of Work

Move Replay date validation, autoplay speed validation, trade action validation, date-to-millisecond conversion, replay action normalization, and replay status normalization into `domain::replay`.

Keep Replay operations in `ops/replay`: start, step, stop, status, autoplay, trade, Replay API availability checks, page-session JavaScript, runtime evaluation, and post-check behavior.

Update durable docs to describe `domain::replay` as the third proof of the boundary. Update `CONTINUITY.md` locally but do not include it in the commit.

## Concrete Steps

Run focused tests:

    cargo test -p tradingview-cli domain::replay -- --nocapture
    cargo test -p tradingview-cli replay -- --nocapture
    cargo test -p tradingview-cli --test cli_contract replay -- --nocapture

Run full validation:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Run behavior smoke:

    target/debug/tv replay --help
    target/debug/tv replay start --date 2026-02-31
    target/debug/tv replay autoplay --speed 123
    target/debug/tv replay trade hold
    TV_CDP_PORT=9 target/debug/tv replay status
    TV_CDP_PORT=9 target/debug/tv replay start --date 2026-04-01

Run hygiene:

    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true

Commit:

    git add crates/cli/src/domain.rs crates/cli/src/domain/replay.rs crates/cli/src/ops/replay.rs crates/cli/src/ops/replay docs CHANGELOG.md
    git commit -m "refactor(domain): Introduce replay service boundary"

## Validation and Acceptance

The change is accepted when all tests pass and the smoke checks preserve existing behavior:

- invalid Replay date, autoplay speed, and trade action fail before CDP connection;
- bad CDP port Replay reads and mutations return structured connection errors;
- Replay action and status payloads keep their public JSON shape;
- `ops/replay` no longer owns the implementation of CDP-free validation and payload normalization.

## Idempotence and Recovery

This is a behavior-preserving refactor. If a test fails, compare JSON field names and error messages against the pre-refactor tests before changing behavior. Keep page-session JavaScript and Replay API method calls in the adapter unless a helper can be tested without `RuntimeEvaluator`. If a moved helper needs CDP runtime objects, move it back to the adapter and record the reason here.

## Artifacts and Notes

Initial structural evidence:

    rg -n "validate_replay_|parse_replay_date_ms|normalize_replay_" crates/cli/src/ops/replay crates/cli/src/app crates/cli/src/ops.rs
    result: Replay validation lived in validation.rs; action/status payload normalization lived in payload.rs; runtime execution lived in control/autoplay/trade/status modules.

Validation evidence:

    cargo test -p tradingview-cli domain::replay -- --nocapture
    result: 10 passed; 0 failed

    cargo test -p tradingview-cli replay -- --nocapture
    result: 29 passed; 0 failed

    cargo test -p tradingview-cli --test cli_contract replay -- --nocapture
    result: 4 passed; 0 failed

    cargo fmt --check
    result: passed

    cargo clippy --workspace --all-targets --all-features -- -D warnings
    result: passed

    cargo test --workspace
    result: passed

    cargo metadata --no-deps --format-version 1
    result: passed

    git diff --check
    result: passed

Behavior smoke evidence:

    target/debug/tv replay --help
    result: exit 0

    target/debug/tv replay start --date 2026-02-31
    result: validation error before CDP connection, exit 1

    target/debug/tv replay autoplay --speed 123
    result: validation error before CDP connection, exit 1

    target/debug/tv replay trade hold
    result: validation error before CDP connection, exit 1

    TV_CDP_PORT=9 target/debug/tv replay status
    result: structured connection error, exit 2

    TV_CDP_PORT=9 target/debug/tv replay start --date 2026-04-01
    result: structured connection error, exit 2

Hygiene evidence:

    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true
    result: existing policy text and validation-command examples only; no new live local path, account id, credential, or raw payload was introduced.
