# Drawing domain boundary

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md` in this repository. It is self-contained and describes a behavior-preserving refactor that introduces a fourth small domain/service boundary inside the CLI package.

## Purpose / Big Picture

The `tv draw` commands already work, but their command request types and validation still live in the operation adapter. An operation adapter is the part of the CLI package that talks to TradingView through Chrome DevTools Protocol, page-session JavaScript, or DOM-like UI actions. A domain module is different: it owns reusable command meaning, such as request interpretation, validation, and public payload shaping, without depending on command-line parser types or a live TradingView page.

After this change, Drawing becomes the fourth proof of this boundary after Watchlist, Alert, and Replay. `domain::drawing` owns drawing request structs, direction parsing, JSON override parsing, and position price validation. `ops/drawing` remains the adapter that creates drawings through the TradingView chart API, reads chart drawings, removes drawings, and verifies mutations. Users should see no command behavior change.

## Progress

- [x] (2026-04-29) Inspected `ops/drawing` and confirmed request structs, direction parsing, override parsing, and position validation are CDP-free.
- [x] (2026-04-29) Archived the completed Replay domain-boundary plan and created this plan.
- [x] (2026-04-29) Added `domain::drawing` and moved Drawing request/validation implementation into it.
- [x] (2026-04-29) Kept Drawing chart API execution, finite point checks, entity post-checks, reads, and lifecycle cleanup in `ops/drawing`.
- [x] (2026-04-29) Updated stable docs and continuity ledger.
- [x] (2026-04-29) Ran validation, behavior smoke, and hygiene checks.
- [ ] Commit the related changes as one refactor.

## Surprises & Discoveries

- Observation: Drawing is a good fourth domain-boundary example, but it is not as pure as Replay at the command level.
  Evidence: `DrawingPositionRequest` validation is fully CDP-free, while `drawing_shape` still performs point finite checks next to chart API execution. This plan only moves the existing pure validation module and does not change the user-visible validation order.

## Decision Log

- Decision: Add `crates/cli/src/domain/drawing.rs` inside the existing CLI package rather than creating a new workspace crate.
  Rationale: The in-package domain layer is still a design proof. Keeping Drawing inside `tradingview-cli` avoids prematurely stabilizing a Rust API while still separating request interpretation from TradingView execution.
  Date/Author: 2026-04-29 / Codex.
- Decision: Leave `crates/cli/src/ops/drawing/validation.rs` as a thin re-export module.
  Rationale: Application dispatch and drawing adapter modules already import through `ops::DrawingPoint`, `ops::parse_drawing_overrides`, and `super::validation`. Re-exporting preserves those paths and keeps the refactor behavior-preserving.
  Date/Author: 2026-04-29 / Codex.
- Decision: Keep chart API JavaScript, entity creation post-checks, and drawing reads/lifecycle operations in `ops/drawing`.
  Rationale: Those functions require `RuntimeEvaluator` and live TradingView page state. They are adapter responsibilities, not reusable domain logic.
  Date/Author: 2026-04-29 / Codex.

## Outcomes & Retrospective

At completion, `domain::drawing` should be the fourth example of the in-package domain/service layer. The four examples should support a stable guideline: domain modules own CDP-free validation, request interpretation, aggregation, fallback policy, and public-safe payload normalization; operation adapters own TradingView execution, DOM or page-session fallbacks, and post-checks.

The implementation now matches that shape. `domain::drawing` owns the Drawing request boundary and focused unit tests, while `ops/drawing` still owns chart API execution and post-checks. Stable docs now describe Watchlist, Alert, Replay, and Drawing as the current proof set, and the roadmap says future domain extraction should be justified by a clear CDP-free logic boundary rather than performed mechanically.

## Context and Orientation

The repository is a Rust workspace. The `tv` binary and command adapters live in `crates/cli`. The reusable core error and JSON envelope types live in `crates/core`. Desktop-free support crates already exist for market, scanner, and Pine static/check logic.

The relevant Drawing files are:

- `crates/cli/src/domain.rs`, the facade for in-package domain modules.
- `crates/cli/src/domain/drawing.rs`, the new Drawing domain module.
- `crates/cli/src/ops/drawing.rs`, the Drawing operation adapter facade.
- `crates/cli/src/ops/drawing/validation.rs`, which remains as an adapter-facing re-export module.
- `crates/cli/src/ops/drawing/create.rs`, which creates shapes and position drawings through the TradingView chart API.
- `crates/cli/src/ops/drawing/read.rs` and `crates/cli/src/ops/drawing/lifecycle.rs`, which read and remove chart drawing entities.

Chrome DevTools Protocol, abbreviated CDP, is the browser automation protocol used to evaluate JavaScript in TradingView Desktop. Any function that needs `RuntimeEvaluator`, chart API JavaScript, DOM interaction, or a post-check against live chart state must remain in `ops`. Any function that can run from ordinary Rust values without a live page is a domain candidate.

## Plan of Work

Move the implementation from `crates/cli/src/ops/drawing/validation.rs` into a new `crates/cli/src/domain/drawing.rs`. The moved module defines `DrawingPoint`, `DrawingShapeRequest`, `PositionDirection`, `DrawingPositionRequest`, `parse_drawing_overrides`, and `validate_position_request`. Because `domain` must not depend on `ops::common`, copy the small finite-number validation helper into the domain module with the same error message.

Update `crates/cli/src/domain.rs` to expose `pub mod drawing;`.

Replace `crates/cli/src/ops/drawing/validation.rs` with a thin re-export from `crate::domain::drawing`. Keep the existing `ops/drawing.rs` public re-exports unchanged so the application dispatch layer and command contract do not change.

Update stable docs to record the fourth domain-boundary example and to describe this as the point where the domain-layer proof is good enough to stabilize rather than blindly moving every remaining operation. Archive the completed Replay plan and update the plan index. Update `CONTINUITY.md` as the local ledger, but do not include it in the commit.

## Concrete Steps

Run focused tests:

    cargo test -p tradingview-cli domain::drawing -- --nocapture
    cargo test -p tradingview-cli drawing -- --nocapture
    cargo test -p tradingview-cli --test cli_contract draw -- --nocapture

Run full validation:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Run behavior smoke:

    target/debug/tv draw --help
    target/debug/tv draw shape --type "" --time 1 --price 1
    target/debug/tv draw position long --entry-price NaN --stop-loss 90 --take-profit 120
    target/debug/tv draw position long --entry-price 100 --stop-loss 101 --take-profit 120
    target/debug/tv draw position --direction short --entry-price 100 --stop-loss 90 --take-profit 80
    TV_CDP_PORT=9 target/debug/tv draw list

If a smoke command in this plan does not match the current CLI help, first run the relevant `--help`, use the correct command shape, and record that correction in `Artifacts and Notes`.

Run hygiene:

    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true

Commit:

    git add CHANGELOG.md crates/cli/src/domain.rs crates/cli/src/domain/drawing.rs crates/cli/src/ops/drawing/validation.rs docs
    git commit -m "refactor(domain): Introduce drawing request boundary"

## Validation and Acceptance

The change is accepted when all tests pass and behavior remains unchanged:

- Drawing request and position validation tests pass in `domain::drawing`.
- `tv draw position` invalid direction, non-finite price, and invalid long/short price ordering still fail before CDP connection.
- Drawing reads and mutations that require CDP still return structured connection errors when `TV_CDP_PORT=9`.
- Existing CLI contract tests for `draw` continue to pass.
- No Drawing chart API JavaScript or `RuntimeEvaluator` dependency is introduced into `domain::drawing`.

## Idempotence and Recovery

This is a behavior-preserving refactor. If a test fails, compare the old and new error messages before changing behavior. If a helper unexpectedly needs `RuntimeEvaluator` or page-session JavaScript, keep it in `ops/drawing` and record the reason in this plan. Re-running `cargo fmt` and the test commands is safe.

## Artifacts and Notes

Initial structural evidence:

    rg -n "DrawingPoint|DrawingShapeRequest|PositionDirection|validate_position_request|parse_drawing_overrides" crates/cli/src/ops/drawing crates/cli/src/app
    result: Drawing request structs and validation were implemented in ops/drawing/validation.rs and re-exported through the drawing facade for application dispatch.

Validation evidence:

    cargo test -p tradingview-cli domain::drawing -- --nocapture
    result: 5 passed; 0 failed

    cargo test -p tradingview-cli drawing -- --nocapture
    result: 26 passed; 0 failed

    cargo test -p tradingview-cli --test cli_contract draw -- --nocapture
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

    target/debug/tv draw --help
    result: exit 0

    target/debug/tv draw shape --help
    result: exit 0; confirmed `--type <SHAPE_TYPE>` is the current command shape.

    target/debug/tv draw position --help
    result: exit 0; confirmed both positional `DIRECTION` and `--direction <DIRECTION>` are documented.

    target/debug/tv draw shape --type "" --time 1 --price 1
    result: validation error before CDP connection, exit 1

    target/debug/tv draw position long --entry-price NaN --stop-loss 90 --take-profit 120
    result: validation error before CDP connection, exit 1

    target/debug/tv draw position long --entry-price 100 --stop-loss 101 --take-profit 120
    result: validation error before CDP connection, exit 1

    target/debug/tv draw position --direction short --entry-price 100 --stop-loss 90 --take-profit 80
    result: validation error before CDP connection, exit 1

    TV_CDP_PORT=9 target/debug/tv draw list
    result: structured connection error, exit 2

Hygiene evidence:

    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true
    result: existing policy text and validation-command examples only; no new live local path, account id, credential, or raw payload was introduced.
