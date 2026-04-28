# Alert domain boundary

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md` in this repository. It is self-contained and describes a behavior-preserving refactor that introduces the second small domain/service boundary inside the CLI package.

## Purpose / Big Picture

The first in-package domain boundary, `domain::watchlist`, proved that CDP-free validation, aggregation, and payload normalization can move out of operation adapters without changing CLI behavior. This slice applies the same pattern to Alert.

After completion, `ops/alert` remains the adapter that executes API calls, DOM fallback, indicator metadata resolution, and post-checks. `domain::alert` owns alert condition validation, public-safe payload normalization, sanitization, and API fallback policy. Users should see no command behavior change.

## Progress

- [x] (2026-04-29) Inspected `ops/alert` and confirmed condition validation and payload normalization still lived in the adapter.
- [x] (2026-04-29) Archived the completed operation-domain boundary plan and created this plan.
- [x] (2026-04-29) Added `domain::alert` for condition validation, public-safe alert payload normalization, sanitization, and API fallback policy.
- [x] (2026-04-29) Kept `RuntimeEvaluator`, page-session fetches, DOM fallback, indicator saved-script resolution, and post-check execution in `ops/alert`.
- [x] (2026-04-29) Updated stable docs and continuity ledger.
- [x] (2026-04-29) Ran validation, behavior smoke, and hygiene checks.
- [ ] Commit the related changes as one refactor.

## Surprises & Discoveries

- Observation: Indicator alert candidate and saved-script resolution are not a good fit for this slice.
  Evidence: those helpers sit next to page-session metadata lookup and API create orchestration. Moving them now would make `domain::alert` depend on runtime-adjacent behavior, so this slice only moves the normalized public payload boundary for indicator create.

## Decision Log

- Decision: Add `crates/cli/src/domain/alert.rs` inside the existing CLI package rather than creating a new `tradingview-alert` crate.
  Rationale: The alert boundary is still being proven. Keeping it in the CLI package avoids prematurely publishing a stable Rust API while still separating pure logic from operation adapters.
  Date/Author: 2026-04-29 / Codex.
- Decision: Leave `ops/alert/payload.rs` as a thin adapter-facing re-export for now.
  Rationale: Existing alert submodules already import through `payload`. A thin re-export keeps the behavior-preserving diff small while making the implementation owner `domain::alert`.
  Date/Author: 2026-04-29 / Codex.
- Decision: Keep indicator saved-script resolution and alertcondition candidate selection in `ops/alert/indicator.rs`.
  Rationale: Those paths are coupled to API/page-session orchestration and are not purely validation or payload normalization.
  Date/Author: 2026-04-29 / Codex.

## Outcomes & Retrospective

At completion, `domain::alert` should be the second example of the in-package domain/service layer. It should be testable without a fake CDP runtime and should contain no clap command enum, `RuntimeEvaluator`, DOM, or page-session JavaScript dependency.

## Context and Orientation

The relevant files are:

- `crates/cli/src/domain.rs` and `crates/cli/src/domain/alert.rs`
- `crates/cli/src/ops/alert.rs` and `crates/cli/src/ops/alert/`
- `docs/architecture.md`, `docs/development.md`, `docs/v0.3-roadmap.md`, and `CHANGELOG.md`

Rust 2024 is used in this repository. Do not introduce `mod.rs`.

## Plan of Work

Move alert condition validation, normalization, public-safe alert sanitization, create/list/delete/indicator-create payload normalization, and API fallback flag interpretation into `domain::alert`.

Keep alert operations in `ops/alert`: list endpoint execution, normal alert create API path, DOM fallback create, indicator create metadata resolution, delete API execution, delete-all API execution, and all post-check behavior.

Update durable docs to describe `domain::alert` as a second proof of the boundary. Update `CONTINUITY.md` locally but do not include it in the commit.

## Concrete Steps

Run focused tests:

    cargo test -p tradingview-cli domain::alert -- --nocapture
    cargo test -p tradingview-cli alert -- --nocapture
    cargo test -p tradingview-cli --test cli_contract alert -- --nocapture

Run full validation:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Run behavior smoke:

    target/debug/tv alert --help
    target/debug/tv alert create --price NaN
    target/debug/tv alert create --price 100 --condition banana
    target/debug/tv alert delete --id ""
    TV_CDP_PORT=9 target/debug/tv alert list
    TV_CDP_PORT=9 target/debug/tv alert create --price 100

Run hygiene:

    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|webhook|web_hook)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true

Commit:

    git add crates/cli/src/domain.rs crates/cli/src/domain/alert.rs crates/cli/src/ops/alert.rs crates/cli/src/ops/alert docs CHANGELOG.md
    git commit -m "refactor(domain): Introduce alert payload boundary"

## Validation and Acceptance

The change is accepted when all tests pass and the smoke checks preserve existing behavior:

- invalid alert condition and non-finite price fail before CDP connection;
- bad CDP port alert reads and create attempts return structured connection errors;
- alert list/create/delete/delete-all/indicator-create payloads keep their public JSON shape;
- raw condition series, Pine IDs, inputs, and account-local alert internals are sanitized before reaching public payloads;
- `ops/alert` no longer owns the implementation of CDP-free validation and payload normalization.

## Idempotence and Recovery

This is a behavior-preserving refactor. If a test fails, compare the JSON field names and error messages against the pre-refactor tests before changing behavior. Keep page-session JavaScript and DOM fallback in the adapter unless a helper can be tested without `RuntimeEvaluator`. If a moved helper needs CDP runtime objects, move it back to the adapter and record the reason here.

## Artifacts and Notes

Initial structural evidence:

    rg -n "fn (sanitize|normalize|validate_alert_condition|alert_condition_type|alert_api_error)|ALERT_CONDITIONS|mod payload" crates/cli/src/ops/alert.rs crates/cli/src/ops/alert/*.rs
    result: condition validation lived in create.rs; payload normalization and fallback policy lived in payload.rs.

Validation evidence:

    cargo test -p tradingview-cli domain::alert -- --nocapture
    result: 5 passed; 0 failed

    cargo test -p tradingview-cli alert -- --nocapture
    result: 31 passed; 0 failed

    cargo test -p tradingview-cli --test cli_contract alert -- --nocapture
    result: 13 passed; 0 failed

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

    target/debug/tv alert --help
    result: exit 0

    target/debug/tv alert create --price NaN
    result: validation error before CDP connection, exit 1

    target/debug/tv alert create --price 100 --condition banana
    result: validation error before CDP connection, exit 1

    target/debug/tv alert delete --id ""
    result: validation error before CDP connection, exit 1

    TV_CDP_PORT=9 target/debug/tv alert list
    result: structured connection error, exit 2

    TV_CDP_PORT=9 target/debug/tv alert create --price 100
    result: structured connection error, exit 2

Hygiene evidence:

    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|webhook|web_hook)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true
    result: existing policy text, archived-plan validation commands, and safety references only; no new live local path, account id, credential, webhook URL, or raw payload was introduced.
