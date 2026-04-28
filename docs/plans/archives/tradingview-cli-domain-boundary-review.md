# Domain boundary review

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md` in this repository. It is self-contained and describes a behavior-preserving refactor that clarifies how the CLI application layer depends on the in-package domain/service layer.

## Purpose / Big Picture

The CLI package now has an in-package `domain` layer with Watchlist, Alert, Replay, Drawing, and Screener examples. The useful boundary has been proven, but `app/dispatch.rs` still calls some CDP-free validation and request construction helpers through `ops::*`. That makes `ops` look like both an execution adapter and a domain facade.

After this change, application dispatch depends directly on `domain::*` for pure validation, request interpretation, and request types. `ops` remains the executable TradingView adapter layer: CDP/runtime access, page-session API calls, DOM/UI fallback, and post-checks. Users should see no command behavior change.

## Progress

- [x] (2026-04-29) Inspected current `domain`, `ops`, and `app/dispatch` references.
- [x] (2026-04-29) Archived the completed Screener domain-boundary plan and created this plan.
- [x] (2026-04-29) Updated `app/dispatch` to import Watchlist, Alert, Replay, Drawing, and Screener domain helpers directly.
- [x] (2026-04-29) Removed pure domain helper re-exports from the top-level `ops` facade and operation facades where dispatch no longer needs them.
- [x] (2026-04-29) Updated stable docs and continuity ledger.
- [x] (2026-04-29) Ran validation, behavior smoke, and hygiene checks.
- [ ] Commit the related changes as one refactor.

## Surprises & Discoveries

- Observation: The existing domain modules are still best kept inside `crates/cli`.
  Evidence: They are mostly command validation, request interpretation, payload shaping, and policy helpers. They do not yet define a stable Rust API suitable for a standalone workspace crate.

- Observation: `ops` remains necessary even after the domain layer exists.
  Evidence: Watchlist, Alert, Replay, Drawing, and Screener all still execute TradingView runtime calls, page-session APIs, DOM fallback, or live post-checks through adapter modules.

## Decision Log

- Decision: Keep `domain` inside the CLI package for now.
  Rationale: The domain layer is useful as an internal service boundary, but its types remain close to CLI command semantics and internal TradingView payload decisions.
  Date/Author: 2026-04-29 / Codex.

- Decision: Let `app/dispatch` depend on `domain::*` directly for pure validation and request construction.
  Rationale: Dispatch is the CLI-to-operation translation layer. Calling pure domain helpers directly makes dependency direction clearer than routing through `ops`.
  Date/Author: 2026-04-29 / Codex.

- Decision: Keep `ops` as the executable adapter facade.
  Rationale: `ops` owns runtime-backed command execution and remains the right boundary for CDP/page-session/DOM operations.
  Date/Author: 2026-04-29 / Codex.

## Outcomes & Retrospective

The dependency direction is now clear:

    cli command enum -> app dispatch -> domain for pure command interpretation
    cli command enum -> app dispatch -> ops for executable TradingView operations
    ops -> domain for adapter-internal payload normalization when needed

This ends the current round of mechanical domain-boundary refactoring. Future domain extraction should happen only when a concrete adapter exposes another CDP-free, reusable logic boundary.

## Context and Orientation

The repository is a Rust workspace. The `tv` binary and CLI package live under `crates/cli`. The relevant layers are:

- `crates/cli/src/cli.rs`: clap command surface.
- `crates/cli/src/app/dispatch.rs`: converts CLI commands into domain requests and operation calls.
- `crates/cli/src/domain.rs` and `crates/cli/src/domain/`: CDP-free domain/service helpers.
- `crates/cli/src/ops.rs` and `crates/cli/src/ops/`: executable TradingView operation adapters.

Existing domain examples are Watchlist, Alert, Replay, Drawing, and Screener. Existing internal workspace crates remain unchanged.

## Plan of Work

First, import domain helpers directly into `app/dispatch.rs` and replace `ops::validate_*`, `ops::Drawing*`, and similar pure helper references with `domain::*` references. Keep dispatch responsible for converting clap command variants into primitive values or domain request types.

Second, remove pure domain re-exports from `ops.rs` and operation facade files where they are no longer used by dispatch. Do not remove adapter-internal compatibility modules, such as `ops/screener/validation.rs`, if sibling operation modules still import through them.

Third, update docs to record the dependency direction and the decision not to extract `domain` into a standalone crate yet.

## Concrete Steps

Run focused tests:

    cargo test -p tradingview-cli domain -- --nocapture
    cargo test -p tradingview-cli domain::watchlist -- --nocapture
    cargo test -p tradingview-cli domain::alert -- --nocapture
    cargo test -p tradingview-cli domain::replay -- --nocapture
    cargo test -p tradingview-cli domain::drawing -- --nocapture
    cargo test -p tradingview-cli domain::screener -- --nocapture

Run full validation:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo test -p tradingview-cli --test cli_contract -- --nocapture
    cargo metadata --no-deps --format-version 1
    git diff --check

Run behavior smoke:

    target/debug/tv watchlist add-bulk --symbols ""
    target/debug/tv alert create --price 100 --condition banana
    target/debug/tv replay start --date 2026-02-31
    target/debug/tv draw position long --entry-price NaN --stop-loss 90 --take-profit 120
    target/debug/tv screener filters add --name "" --min 1 --dry-run
    TV_CDP_PORT=9 target/debug/tv screener status

Run hygiene:

    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true

Commit:

    git add CHANGELOG.md crates/cli/src/app/dispatch.rs crates/cli/src/ops.rs crates/cli/src/ops docs/architecture.md docs/development.md docs/v0.3-roadmap.md docs/plans/README.md docs/plans/tradingview-cli-domain-boundary-review.md docs/plans/archives/tradingview-cli-screener-domain-boundary.md
    git commit -m "refactor(app): Clarify domain boundary dependencies"

## Validation and Acceptance

The change is accepted when:

- `app/dispatch.rs` calls pure domain helpers directly instead of through `ops::*`;
- `ops.rs` no longer re-exports pure domain validation/request helpers solely for dispatch;
- command behavior, JSON envelopes, and exit codes remain unchanged;
- invalid CLI inputs still fail before CDP connection;
- runtime-backed commands still produce structured connection errors when CDP is unavailable;
- docs state that `domain` stays in-package for now and that `ops` remains the executable adapter layer.

## Idempotence and Recovery

This is a behavior-preserving refactor. If a helper is needed by sibling operation modules, keep the local operation-module re-export and record why. If a contract test changes, compare the JSON payload and exit code before editing implementation. Re-running formatting and tests is safe.

## Artifacts and Notes

Initial evidence:

    rg -n "ops::validate|ops::Drawing|ops::parse_drawing|ops::PositionDirection" crates/cli/src/app/dispatch.rs
    result: dispatch still routed several pure domain helpers through `ops`.

    cargo check --workspace
    result before cleanup: compiled with unused re-export warnings from operation facades.

Validation evidence:

    cargo test -p tradingview-cli domain -- --nocapture
    result: passed, 40 domain tests.

    cargo test -p tradingview-cli domain::watchlist -- --nocapture
    cargo test -p tradingview-cli domain::alert -- --nocapture
    cargo test -p tradingview-cli domain::replay -- --nocapture
    cargo test -p tradingview-cli domain::drawing -- --nocapture
    cargo test -p tradingview-cli domain::screener -- --nocapture
    result: passed.

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo test -p tradingview-cli --test cli_contract -- --nocapture
    cargo metadata --no-deps --format-version 1
    result: passed.

Behavior smoke:

    target/debug/tv watchlist add-bulk --symbols ""
    target/debug/tv alert create --price 100 --condition banana
    target/debug/tv replay start --date 2026-02-31
    target/debug/tv draw position long --entry-price NaN --stop-loss 90 --take-profit 120
    target/debug/tv screener filters add --name "" --min 1 --dry-run
    result: validation failures before CDP, exit 1.

    TV_CDP_PORT=9 target/debug/tv screener status
    result: structured connection error, exit 2.
