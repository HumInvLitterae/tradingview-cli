# Centralize workspace dependency versions

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository uses `.agents/PLANS.md` as the ExecPlan standard. This document follows that standard and is intentionally self-contained so a future contributor can resume the dependency cleanup from this file alone.

## Purpose / Big Picture

The repository has been split into several crates under one Cargo workspace. Each crate currently repeats dependency versions and shared package metadata in its own `Cargo.toml`, which makes routine release and dependency updates more error-prone. After this change, the workspace root will own shared package metadata and the no-publish policy in `[workspace.package]` plus dependency versions in `[workspace.dependencies]`, while each crate will declare only its crate-specific name, dependencies, and feature needs. Users should see no CLI behavior change; the proof is that Cargo metadata, build, lint, and tests still pass.

## Progress

- [x] (2026-04-29 12:17Z) Confirmed the working tree is clean after the previous release-prep edits were stashed by the user.
- [x] (2026-04-29 12:17Z) Inspected all workspace crate manifests and confirmed dependency versions are duplicated across crate-local `Cargo.toml` files.
- [x] (2026-04-29 12:22Z) Added `[workspace.dependencies]` to the root `Cargo.toml`.
- [x] (2026-04-29 12:22Z) Converted crate-local normal and dev dependencies to `workspace = true`.
- [x] (2026-04-29 12:23Z) Updated development documentation with the workspace dependency rule.
- [x] (2026-04-29 12:26Z) Ran Cargo validation and confirmed this manifest cleanup did not change `Cargo.lock`.
- [x] (2026-04-29 12:30Z) Centralized shared package metadata with `[workspace.package]`, including `publish = false`.
- [x] (2026-04-29 12:36Z) Re-ran Cargo validation after publish metadata inheritance.
- [ ] Commit the workspace metadata and dependency centralization separately from release preparation if the user approves.

## Surprises & Discoveries

- Observation: Cargo does not use a separate `[workspace.dev-dependencies]` table for this workflow.
  Evidence: Workspace dependency inheritance uses `[workspace.dependencies]`, and individual crates can reference those entries from `[dependencies]`, `[dev-dependencies]`, or `[build-dependencies]` with `workspace = true`.
- Observation: The user's dependency update already moved `reqwest` to `0.13.3` in crate-local manifests.
  Evidence: `crates/cli/Cargo.toml`, `crates/cdp/Cargo.toml`, `crates/market/Cargo.toml`, `crates/pine/Cargo.toml`, and `crates/scanner/Cargo.toml` all show `reqwest = { version = "0.13.3", features = ["json"] }`.

## Decision Log

- Decision: Use `[workspace.dependencies]` for dev-only crates such as `assert_cmd`, `predicates`, and `tempfile` as well as normal dependencies.
  Rationale: Cargo's workspace dependency inheritance is shared across dependency tables. Keeping test-only versions in the root avoids a second source of truth.
  Date/Author: 2026-04-29 / Codex
- Decision: Keep feature choices at the crate edge when feature needs differ.
  Rationale: Dependencies such as `tokio`, `reqwest`, `futures-util`, `image`, and `tracing-subscriber` are used with different feature sets or default-feature choices. The root should own versions and stable path dependencies; each crate should opt into the features it actually needs.
  Date/Author: 2026-04-29 / Codex
- Decision: Centralize `version`, `edition`, `license`, and `publish` in `[workspace.package]`.
  Rationale: These fields are shared package metadata for all workspace crates. Inheriting `publish = false` keeps the no-publish policy centralized while still making each member manifest explicit through `publish.workspace = true`.
  Date/Author: 2026-04-29 / Codex

## Outcomes & Retrospective

Implemented and validated, but not committed. The root workspace now owns shared package metadata in `[workspace.package]`, including the no-publish policy, plus third-party dependency versions and internal crate paths in `[workspace.dependencies]`. Member crates still declare their direct dependencies, but they inherit package metadata, versions, and paths with `workspace = true` and keep only crate-specific feature selections locally. `Cargo.lock` did not change from this manifest cleanup. `cargo metadata`, formatting, clippy, workspace tests, and whitespace checks pass after `publish.workspace = true` was added.

## Context and Orientation

The repository root `Cargo.toml` is a virtual Cargo workspace. It has `[workspace]` members for `crates/cli`, `crates/core`, `crates/model`, `crates/market`, `crates/scanner`, `crates/pine`, and `crates/cdp`. The root previously had no `[workspace.package]` or `[workspace.dependencies]` table.

Each package manifest under `crates/*/Cargo.toml` currently repeats dependency versions. For example, multiple crates repeat `reqwest`, `serde_json`, and `tradingview-core` definitions. This is safe but awkward after the crate split: a future dependency update has to touch several files and can accidentally leave versions inconsistent.

In Cargo, `[workspace.dependencies]` lets the root workspace define a dependency once. Member crates then reference that dependency by writing `serde_json = { workspace = true }`. This does not automatically add the dependency to every crate; each crate must still list what it uses. A crate can also add features at the use site, for example `tokio = { workspace = true, features = ["time", "net"] }`.

## Plan of Work

Edit the root `Cargo.toml` and add `[workspace.package]` plus `[workspace.dependencies]` after the existing `[workspace]` table. Put shared package metadata such as `version`, `edition`, `license`, and `publish` in `[workspace.package]`. Include all third-party dependencies currently used by workspace members and all internal path dependencies between workspace crates in `[workspace.dependencies]`.

Then edit each crate manifest. Replace shared package metadata with `version.workspace = true`, `edition.workspace = true`, `license.workspace = true`, and `publish.workspace = true`. Keep crate-specific `name` and binary target definitions in member manifests. Replace dependency version strings and path definitions with `workspace = true`. Preserve crate-specific feature choices. For example, `crates/cdp/Cargo.toml` should keep `futures-util`'s `sink` feature and `tokio`'s `time` and `net` features, but the version should come from the root. `crates/cli/Cargo.toml` should keep `clap`'s `derive` feature, `image`'s `png`, `tokio`'s runtime features, and `tracing-subscriber`'s `env-filter` feature. Its dev dependencies should become `assert_cmd = { workspace = true }`, `predicates = { workspace = true }`, and `tempfile = { workspace = true }`.

Finally, update `docs/development.md` to say that new dependency versions belong in root `[workspace.dependencies]`; member crates should use `workspace = true` and add only crate-specific feature selections locally.

## Concrete Steps

Run commands from the repository root.

After editing manifests, verify metadata and lockfile behavior:

    cargo metadata --no-deps --format-version 1
    git diff -- Cargo.lock

Then run the normal validation baseline:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    git diff --check

If the release-prep stash remains separate, do not apply it during this slice. This dependency cleanup should stay as its own commit.

Commands already run successfully in this slice:

    cargo metadata --no-deps --format-version 1
    git diff -- Cargo.lock
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    git diff --check

## Validation and Acceptance

The change is accepted when `cargo metadata --no-deps --format-version 1`, `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace`, and `git diff --check` all pass after both package metadata and dependency centralization. `Cargo.lock` should either be unchanged or show only changes that are directly explained by the user's prior dependency update, not broad dependency churn caused by this manifest cleanup.

The repository docs are accepted when `docs/development.md` records the new dependency rule clearly enough that future agents do not add version strings to member crate manifests by default.

## Idempotence and Recovery

This is a manifest-only refactor plus documentation. It is safe to rerun Cargo metadata, lint, and tests. If Cargo reports feature or dependency inheritance errors, keep the root version entry and move feature selections back to the member crate use site. If `Cargo.lock` churns unexpectedly, do not accept it blindly; inspect the diff and restore the lockfile unless the change comes from an explicit dependency version update.

## Artifacts and Notes

At plan creation, the working tree was clean. Release-readiness edits were not present because the user had stashed them to perform dependency work first.

## Interfaces and Dependencies

No Rust API, CLI command, JSON envelope, or runtime behavior should change. The only intended interface change is the Cargo manifest convention: root `[workspace.package]` owns shared package metadata and no-publish policy, root `[workspace.dependencies]` owns dependency versions and internal path dependencies, while member crate manifests inherit them with `workspace = true`.

## Open Questions

There are no unresolved blocking questions. The only nuance is feature placement: root owns versions, while member crates may still specify features where their needs differ.
