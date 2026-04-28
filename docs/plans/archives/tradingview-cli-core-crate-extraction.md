# Core contract crate extraction

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md` from the repository root. It is self-contained so a future contributor can continue from this file without relying on chat history.

## Purpose / Big Picture

The `tv` binary now has a library crate boundary, but all shared contracts still live inside the root `tradingview_cli` crate. That makes later reuse awkward because every future internal crate would need to depend on the full CLI implementation to use the typed error and JSON envelope types. After this change, the package will be a Cargo workspace with a small internal crate named `tradingview-core` at `crates/core/`. That crate owns only the error and envelope contract shared by the CLI and future modules.

Users should see no command behavior change. The observable proof is that `tv --help`, Desktop-free `tv info NYSE:IONQ`, and structured error envelopes still work, while `cargo metadata` shows both the existing `tv` binary target and the new `tradingview-core` package.

## Progress

- [x] (2026-04-28T02:19Z) Confirmed the working tree was clean and inspected the existing `src/error.rs`, `src/output.rs`, and import references.
- [x] (2026-04-28T02:19Z) Archived the completed first library-boundary plan and created this ExecPlan.
- [x] (2026-04-28T02:24Z) Added the Cargo workspace, `crates/core/` package, and root path dependency.
- [x] (2026-04-28T02:24Z) Moved the error and JSON envelope contract into `tradingview_core`.
- [x] (2026-04-28T02:24Z) Updated root crate imports, removed the old root `error` and `output` modules, and confirmed `cargo check` passes.
- [x] (2026-04-28T02:24Z) Updated architecture, development, roadmap, changelog, and plan index docs.
- [x] (2026-04-28T02:26Z) Ran validation, cargo metadata, smoke checks, and hygiene checks.
- [ ] Update continuity docs.
- [ ] Commit the related changes.

## Surprises & Discoveries

- Observation: Several operation modules used grouped imports such as `use crate::{ cdp::..., error::{...} };`, so a simple line-based replacement did not catch every old `crate::error` path.
  Evidence: the first `cargo check` failed on unresolved `crate::error` imports in `src/cdp.rs`, `src/ops/alert.rs`, and similar modules. After updating grouped imports, `cargo check` passed.

## Decision Log

- Decision: Create the new crate in `crates/core/`, with package name `tradingview-core` and crate name `tradingview_core`.
  Rationale: The shorter directory keeps the workspace tidy while the package and crate names remain explicit enough for Rust imports and Cargo metadata.
  Date/Author: 2026-04-28 / Codex.

- Decision: Move only `AppError`, `ErrorKind`, `SuccessEnvelope`, `ErrorEnvelope`, and `ErrorBody` into the core crate.
  Rationale: These are pure contract types used across most modules. CDP, transport, market, scanner, screener, and operation logic still depend on the full CLI shape and are intentionally left in the root crate for later slices.
  Date/Author: 2026-04-28 / Codex.

## Outcomes & Retrospective

This section will be completed after implementation and validation.

Validation passed for the workspace split. `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace`, `cargo test --test cli_contract -- --nocapture`, `cargo build`, and `git diff --check` all passed. `cargo metadata --no-deps --format-version 1` showed package `tradingview-core` with target `tradingview_core`, the root library target `tradingview_cli`, and the existing `tv` binary target.

Smoke checks passed. `target/debug/tv --help` printed the normal CLI help, `target/debug/tv info NYSE:IONQ` returned a success envelope with `data.source` equal to `symbol_search_rest`, and `TV_CDP_PORT=9 target/debug/tv status` exited with code 2 and returned a structured `connection` error envelope. The tracked-doc hygiene grep returned only existing policy text and validation-command examples.

## Context and Orientation

The repository is currently a Rust package named `tradingview-cli`. Its binary is named `tv`, and its first library boundary is the inferred library crate `tradingview_cli` rooted at `src/lib.rs`. The root library currently exposes modules such as `cdp`, `cli`, `ops`, and `transport`.

The contract types targeted by this plan are currently:

- `src/error.rs`, which defines `ErrorKind`, `AppError`, and `AppError::exit_code()`.
- `src/output.rs`, which defines `SuccessEnvelope`, `ErrorEnvelope`, `ErrorBody`, and the conversion from `AppError` into `ErrorBody`.

A Cargo workspace is a collection of related Rust packages that share one lockfile and can be tested together with commands such as `cargo test --workspace`. This plan keeps the root package and binary intact, then adds one new workspace member at `crates/core/`.

The new `tradingview-core` crate is internal. It is reusable by other crates in this repository, but it is not a stable crates.io public API.

## Plan of Work

First, introduce a workspace in `Cargo.toml` with members `.` and `crates/core`, preserving the root package `tradingview-cli` and the `tv` binary. Add a path dependency from the root package to `tradingview-core`.

Second, create `crates/core/Cargo.toml` and `crates/core/src/lib.rs`. The core crate should depend only on `serde`, `serde_json`, and `thiserror`. Its root module should expose `AppError`, `ErrorKind`, `SuccessEnvelope`, `ErrorEnvelope`, and `ErrorBody` directly as `tradingview_core::{...}`.

Third, remove `src/error.rs` and `src/output.rs`, remove their declarations from `src/lib.rs`, and update all imports in `src/main.rs`, `src/transport.rs`, and `src/ops/**` from `crate::error` or `tradingview_cli::error` to `tradingview_core`. No operation logic or JSON field names should change.

Fourth, update durable docs. `docs/architecture.md` should explain the workspace and crate boundaries. `docs/development.md` should define when a type belongs in a cross-crate package. `docs/v0.3-roadmap.md` and `CHANGELOG.md` should record the internal refactor. `docs/plans/README.md` should show this plan as active and the previous library-boundary plan as archived. `CONTINUITY.md` should be updated as the local ledger but not committed.

Finally, run the workspace validation baseline and CLI smoke checks, then commit the related tracked changes.

## Concrete Steps

Work from the repository root.

1. Add the workspace and path dependency in `Cargo.toml`.

2. Add `crates/core/Cargo.toml` and `crates/core/src/lib.rs`.

3. Delete the old root contract modules:

       src/error.rs
       src/output.rs

4. Update imports with repository-wide search for:

       crate::error
       crate::output
       tradingview_cli::error
       tradingview_cli::output

5. Update docs and continuity.

6. Run validation:

       cargo fmt --check
       cargo clippy --workspace --all-targets --all-features -- -D warnings
       cargo test --workspace
       cargo test --test cli_contract -- --nocapture
       cargo metadata --no-deps --format-version 1
       cargo build
       target/debug/tv --help
       target/debug/tv info NYSE:IONQ
       TV_CDP_PORT=9 target/debug/tv status
       git diff --check
       git grep -nE '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' -- README.md CHANGELOG.md docs .agents/skills packaging scripts || true

7. Commit with:

       refactor(core): Extract core contract crate

## Validation and Acceptance

The change is accepted when the workspace validation commands pass, `cargo metadata --no-deps --format-version 1` shows package `tradingview-core` and the existing binary target `tv`, and the CLI smoke checks prove that normal help, Desktop-free info reads, and structured connection-error envelopes still work.

The JSON envelope field names must remain unchanged. Success output must still use `success`, `command`, and `data`. Error output must still use `success`, `command`, and `error`, with `kind`, `message`, and optional `details`.

## Idempotence and Recovery

This plan is safe to repeat. If `crates/core/` already exists, inspect it and edit in place rather than creating a second crate. If imports fail to compile, search for remaining old paths and update them to `tradingview_core`. If Cargo workspace metadata looks wrong, check that `[workspace]` includes both `.` and `crates/core`.

If validation fails because of formatting, run `cargo fmt` once and repeat the check. Do not change CLI behavior to satisfy tests; the purpose is a behavior-preserving extraction.

## Artifacts and Notes

Expected `cargo metadata --no-deps --format-version 1` should include a package named `tradingview-core` and the existing `tv` binary target. Do not paste machine-specific metadata paths into repository docs.

Expected `TV_CDP_PORT=9 target/debug/tv status` should fail with a structured connection error envelope rather than panicking, proving that the moved `AppError` and envelope types still drive process output.

## Interfaces and Dependencies

The new crate must expose these types at the crate root:

    pub enum ErrorKind
    pub struct AppError
    pub struct SuccessEnvelope
    pub struct ErrorEnvelope
    pub struct ErrorBody

`AppError` must keep:

    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self
    pub fn with_details(mut self, details: serde_json::Value) -> Self
    pub fn exit_code(&self) -> u8

The root package must depend on the core crate with:

    tradingview-core = { path = "crates/core" }

The core crate's dependencies are limited to:

    serde
    serde_json
    thiserror

## Open Questions

No critical open questions block this slice. After this extraction is stable, a later plan can decide whether `crates/cdp/` or `crates/market/` is the next useful module boundary.
