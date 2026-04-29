# TradingView model crate extraction

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md` in this repository. It is self-contained and describes a behavior-preserving refactor that extracts the current in-package domain layer into an internal workspace crate named `tradingview-model`.

## Purpose / Big Picture

The CLI package has proven a useful CDP-free boundary for Watchlist, Alert, Replay, Drawing, and Screener logic. That logic is currently under `crates/cli/src/domain/`, which makes it less reusable than the already extracted `tradingview-market`, `tradingview-scanner`, and `tradingview-pine` crates.

After this change, the shared model and policy logic lives in `crates/model/` as package `tradingview-model` and crate `tradingview_model`. The user-visible `tv` command behavior does not change. The observable result is that the workspace has a new internal crate, all model tests run under `cargo test -p tradingview-model`, and existing CLI contract tests still pass.

## Progress

- [x] (2026-04-29) Inspected the existing `crates/cli/src/domain/` modules and confirmed they do not depend on CDP, clap, reqwest, or tokio.
- [x] (2026-04-29) Chose `tradingview-model` over `tradingview-domain` for the extracted crate name.
- [x] (2026-04-29) Added `crates/model/` and moved the current domain facade and modules into it.
- [x] (2026-04-29) Updated the workspace and CLI package dependencies so `tradingview-cli` imports `tradingview_model`.
- [x] (2026-04-29) Updated architecture, development, roadmap, changelog, plans index, and continuity docs.
- [x] (2026-04-29) Ran focused model tests, full workspace validation, behavior smoke, and hygiene checks.
- [x] (2026-04-29) Committed the related changes as one refactor.

## Surprises & Discoveries

- Observation: The current domain modules are already clean enough for a crate move.
  Evidence: `rg` found no `RuntimeEvaluator`, `reqwest`, `tokio`, `clap`, `tradingview_cdp`, or `evaluate(` references under the domain modules before the move.

- Observation: The name `tradingview-model` better describes the extracted layer than `tradingview-domain`.
  Evidence: The extracted code is mostly request structs, selectors, validation, target resolution, normalization, payload shaping, and fallback policy. Broader behavior such as scanner HTTP calls, Pine facade checks, and CDP execution remains in other crates or adapters.

- Observation: The inherited smoke example `tv watchlist add-bulk --symbols ""` is not valid for the current CLI.
  Evidence: `tv watchlist add-bulk --symbols ""` returns a clap usage error because bulk symbols are positional. The correct validation smoke is `tv watchlist add-bulk ""`, which returns `Symbol must not be empty` before CDP connection.

## Decision Log

- Decision: Extract to `crates/model/` with package name `tradingview-model` and crate name `tradingview_model`.
  Rationale: The repository already uses short directory names such as `crates/core/` and package names with the `tradingview-` prefix. `model` describes the I/O-free shared model/policy layer without claiming ownership of every TradingView domain behavior.
  Date/Author: 2026-04-29 / Codex.

- Decision: Treat `tradingview-model` as an internal unstable crate.
  Rationale: It improves reuse and dependency direction now, but its Rust API is still shaped by current CLI commands and should not be considered stable for crates.io or external downstream callers yet.
  Date/Author: 2026-04-29 / Codex.

- Decision: Keep service/client crates separate from the model crate.
  Rationale: `tradingview-market`, `tradingview-scanner`, and `tradingview-pine` perform HTTP reads or source analysis. `tradingview-model` should stay free of network, CDP, page-session, and UI execution dependencies.
  Date/Author: 2026-04-29 / Codex.

## Outcomes & Retrospective

The model extraction is complete in the working tree. `tradingview-model` is now a workspace member and `tradingview-cli` depends on it. The previous `crates/cli/src/domain/` files moved into `crates/model/src/`, and CLI/application/operation imports now use `tradingview_model`.

The result preserves command behavior while making the I/O-free model/policy layer reusable independently of the CLI package. Validation confirmed that the extracted crate has only `tradingview-core` and `serde_json` dependencies and that existing CLI contract tests still pass.

## Context and Orientation

The repository is a Cargo workspace. The `tv` binary and application layer live in package `tradingview-cli` under `crates/cli/`. Shared contract types such as `AppError` and JSON envelopes live in `crates/core/`. Desktop-free HTTP or analysis crates already exist for market reads, scanner reads, and Pine static/check helpers.

Before this plan, `crates/cli/src/domain.rs` and `crates/cli/src/domain/` held CDP-free logic for:

- Watchlist symbol normalization, bulk-add validation, aggregation, and public payload normalization.
- Alert condition validation, public-safe payload normalization, sanitization, and API fallback policy.
- Replay date/speed/action validation, timestamp conversion, and replay payload normalization.
- Drawing request structs, direction parsing, override parsing, and position validation.
- Screener validation, selector and target resolution, storage payload shaping, and test-screen guards.

In this plan, “model” means I/O-free shared command and domain model logic: request interpretation, validation, selector and target resolution, normalization, public-safe payload shaping, and fallback policy decisions. It does not mean only passive structs. It also does not include external I/O, CDP runtime access, DOM interaction, page-session API execution, or live post-checks.

## Plan of Work

First, add `crates/model/Cargo.toml` as an internal package named `tradingview-model`. Its dependency set should stay small: `tradingview-core` for typed errors and `serde_json` for JSON values. Add `serde` only if a moved type requires derives.

Second, move the existing domain facade and modules from `crates/cli/src/domain.rs` and `crates/cli/src/domain/` into `crates/model/src/lib.rs` and sibling modules. Preserve module names so imports are naturally rewritten from `crate::domain::watchlist` to `tradingview_model::watchlist`.

Third, update `crates/cli/Cargo.toml` to depend on `tradingview-model`, remove `pub mod domain;` from `crates/cli/src/lib.rs`, and update `crates/cli/src/app/dispatch.rs` and operation adapters to import from `tradingview_model`.

Fourth, update stable docs to describe the crate relationship. `tradingview-model` is the I/O-free shared model/policy crate. `tradingview-market`, `tradingview-scanner`, and `tradingview-pine` are service/client or analysis crates. `tradingview-cdp` is the Desktop connection crate. `tradingview-cli` owns CLI surface, application orchestration, and executable operation adapters.

## Concrete Steps

Run focused checks for the new crate:

    cargo test -p tradingview-model -- --nocapture
    cargo test -p tradingview-model watchlist -- --nocapture
    cargo test -p tradingview-model alert -- --nocapture
    cargo test -p tradingview-model replay -- --nocapture
    cargo test -p tradingview-model drawing -- --nocapture
    cargo test -p tradingview-model screener -- --nocapture

Run full validation:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo test -p tradingview-cli --test cli_contract -- --nocapture
    cargo metadata --no-deps --format-version 1
    git diff --check

Run behavior smoke:

    target/debug/tv watchlist add-bulk ""
    target/debug/tv alert create --price 100 --condition banana
    target/debug/tv replay start --date 2026-02-31
    target/debug/tv draw position long --entry-price NaN --stop-loss 90 --take-profit 120
    target/debug/tv screener filters add --name "" --min 1 --dry-run
    TV_CDP_PORT=9 target/debug/tv screener status

Run hygiene:

    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true

Commit:

    git add Cargo.toml crates/model crates/cli/Cargo.toml crates/cli/src docs CHANGELOG.md
    git commit -m "refactor(model): Extract TradingView model crate"

## Validation and Acceptance

The change is accepted when:

- `cargo metadata --no-deps --format-version 1` shows package `tradingview-model`;
- `crates/cli` depends on `tradingview-model`;
- no `crates/cli/src/domain.rs` or `crates/cli/src/domain/` remains;
- `tradingview-model` has no dependency on clap, CDP, reqwest, tokio, or the CLI package;
- existing CLI contract tests pass without JSON or exit-code changes;
- validation failures still occur before CDP connection;
- runtime-backed commands still return structured connection errors when CDP is unavailable.

## Idempotence and Recovery

This is a behavior-preserving refactor. If a moved helper unexpectedly requires a CLI or runtime type, do not add that dependency to `tradingview-model`; keep that helper in the CLI adapter and document the boundary. If tests fail after import rewrites, compare the public JSON payload and exit code before changing logic. Re-running formatting and tests is safe.

## Artifacts and Notes

Initial check:

    cargo check --workspace
    result: passed after adding `tradingview-model` and updating imports.

Focused model checks:

    cargo test -p tradingview-model -- --nocapture
    result: passed, 40 tests.

    cargo test -p tradingview-model watchlist -- --nocapture
    cargo test -p tradingview-model alert -- --nocapture
    cargo test -p tradingview-model replay -- --nocapture
    cargo test -p tradingview-model drawing -- --nocapture
    cargo test -p tradingview-model screener -- --nocapture
    result: passed.

Full validation:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo test -p tradingview-cli --test cli_contract -- --nocapture
    cargo metadata --no-deps --format-version 1
    git diff --check
    result: passed. Metadata shows package `tradingview-model` and crate target `tradingview_model`.

Behavior smoke:

    target/debug/tv watchlist add-bulk ""
    target/debug/tv alert create --price 100 --condition banana
    target/debug/tv replay start --date 2026-02-31
    target/debug/tv draw position long --entry-price NaN --stop-loss 90 --take-profit 120
    target/debug/tv screener filters add --name "" --min 1 --dry-run
    TV_CDP_PORT=9 target/debug/tv screener status
    result: validation failures occurred before CDP where expected, and bad CDP port returned a structured connection error.

Hygiene:

    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true
    result: only existing policy text, archived historical notes, and validation-command examples were reported; no new secret, account id, or local path content was added to tracked docs.

## Interfaces and Dependencies

At completion:

- `crates/model/src/lib.rs` exposes `pub mod alert`, `drawing`, `replay`, `screener`, and `watchlist`.
- `crates/model/Cargo.toml` depends on `tradingview-core` and `serde_json`.
- `crates/cli/Cargo.toml` depends on `tradingview-model`.
- `crates/cli/src/app/dispatch.rs` imports model helpers from `tradingview_model`.
- Operation adapters may import model helpers directly, but they keep all CDP/runtime/page-session/DOM execution.

## Open Questions

None. The chosen name is `tradingview-model`; `tradingview-domain` is not used in this slice.
