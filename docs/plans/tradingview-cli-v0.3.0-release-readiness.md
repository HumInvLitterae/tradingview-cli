# v0.3.0 release readiness

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository uses `.agents/PLANS.md` as the ExecPlan standard. This document is intentionally self-contained so a future contributor can resume release preparation from this file alone.

## Purpose / Big Picture

Prepare the `v0.3.0` release without adding features or refactoring. The release should publish the current implemented state: Desktop-free market reads, API/storage-backed stability work, full-page Screener target setup, target handoff cleanup, Windows Store/MSIX launch evidence, workspace crate boundaries, and packaged runtime skill guidance. The outcome is a release-prep commit only; tag creation, push, and GitHub Release publication remain manual user actions.

## Progress

- [x] (2026-04-29 13:05Z) Confirmed the working tree was clean after the workspace metadata centralization commit.
- [x] (2026-04-29 13:08Z) Read release-prep skill guidance and grounded current release docs, packaging script, README, changelog, and version metadata.
- [x] (2026-04-29 13:12Z) Bumped workspace package version to `0.3.0` and synchronized `Cargo.lock`.
- [x] (2026-04-29 13:17Z) Cut `CHANGELOG.md` `Unreleased` content into `v0.3.0 - 2026-04-29`.
- [x] (2026-04-29 13:18Z) Added curated GitHub Release body at `docs/releases/v0.3.0.md`.
- [x] (2026-04-29 13:22Z) Refreshed README, packaged agent guide, runtime Screener skill, roadmap, and plans index.
- [x] (2026-04-29 13:37Z) Release packaging and Rust validation passed.
- [x] (2026-04-29 13:40Z) Optional `gh run list --limit 5` check showed the latest CI run on `main` succeeded before this local release-prep change.
- [x] (2026-04-29 13:40Z) Left changes uncommitted; no tag, push, or GitHub Release was created.

## Surprises & Discoveries

- Observation: Shared package metadata, including `authors` and `publish = false`, is now inherited from root `[workspace.package]`.
  Evidence: Member manifests use `authors.workspace = true` and `publish.workspace = true`.
- Observation: The packaged agent guide still described Windows Store/MSIX launch as unverified.
  Evidence: `packaging/agent/AGENTS.md` startup guidance predates the Windows Store/MSIX smoke result.
- Observation: The Screener runtime skill still suggested plain `tv screener open` when no full-page Screener target exists.
  Evidence: `.agents/skills/screener-workflow/SKILL.md` had not yet been updated for `tv screener open --full-page`.

## Decision Log

- Decision: Release target is `v0.3.0` / Cargo version `0.3.0`.
  Rationale: The release contains user-visible feature additions, behavior changes, compatibility work, and major internal crate/workspace refactors after `v0.2.0`.
  Date/Author: 2026-04-29 / Codex
- Decision: Do not tag, push, or create the GitHub Release in this slice.
  Rationale: The user owns final publication actions.
  Date/Author: 2026-04-29 / Codex
- Decision: Keep release notes public-safe and omit raw target ids, account-local identifiers, local paths, cookies, tokens, and raw live payloads.
  Rationale: Release artifacts are public-facing and should not expose operator-specific TradingView or machine data.
  Date/Author: 2026-04-29 / Codex

## Outcomes & Retrospective

Release readiness is prepared but not committed. The workspace package version is `0.3.0`, `Cargo.lock` is synchronized for all internal workspace packages, `CHANGELOG.md` has a dated `v0.3.0` section, and `docs/releases/v0.3.0.md` is ready to use as the curated GitHub Release body. README, packaged agent guidance, the Screener runtime skill, roadmap, and plans index now match the current behavior around `--target-id`, Desktop-free `info` / `quote`, full-page Screener target setup, Windows Store/MSIX smoke evidence, and release packaging.

Validation passed with `bash -n scripts/stage-release-package-files.sh`, `cargo build --release --locked`, release package staging, `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace`, `cargo metadata --no-deps --format-version 1`, and `git diff --check`. The staged package contains the runtime skills and excludes development-only skills. The public-doc hygiene scan reported only policy language, archived validation-command examples, and this plan's validation command; no new secret, local machine path, live target id, or account-local identifier was added. An optional `gh run list --limit 5` check showed the latest CI run on `main` was successful before this local release-prep change.

## Context and Orientation

The previous `v0.2.0` release is documented in `docs/releases/v0.2.0.md` and `CHANGELOG.md`. Since then, the CLI added target-selection cleanup, Desktop-free symbol reads, API-backed alert and watchlist improvements, guarded indicator alert creation, full-page Screener setup, Screener storage stabilization, Windows launch evidence, and a substantial internal workspace split.

The repository root is now a virtual Cargo workspace. Package versioning is controlled by root `[workspace.package]`, and the release version bump updates all workspace packages that inherit that version.

## Plan of Work

Update root `Cargo.toml` from `0.2.0` to `0.3.0` and synchronize `Cargo.lock`.

Move `CHANGELOG.md` content from `Unreleased` into a dated `v0.3.0 - 2026-04-29` section. Keep the notes user-facing and grouped enough for release readers.

Create `docs/releases/v0.3.0.md` as the curated GitHub Release body. Do not include a top-level `# v0.3.0` heading because the GitHub Release title already contains the tag.

Refresh README release asset examples from `v0.2.0` to `v0.3.0`, ensure build instructions reflect the workspace package layout, and verify current guidance for `--target-id`, Desktop-free `info/quote`, Screener full-page target setup, and Windows Store/MSIX launch evidence.

Refresh packaged agent guidance and runtime skills only where they are stale. Keep the packaging script's explicit runtime-skill allowlist; do not include development-only skills.

Update `docs/v0.3-roadmap.md` to show release readiness reached, update `docs/plans/README.md`, and archive completed active plans that are no longer current.

Run release package staging and the Rust baseline before considering the prep done.

## Concrete Steps

Run from the repository root:

    cargo update -p tradingview-cli --precise 0.3.0
    bash -n scripts/stage-release-package-files.sh
    cargo build --release --locked
    rm -rf target/release-package-smoke
    scripts/stage-release-package-files.sh target/release-package-smoke target/release/tv
    find target/release-package-smoke -maxdepth 4 -print | sort
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true

Inspect the staged release package and confirm that it includes runtime skills and excludes development-only skills such as `continuity`, `conventional-commits`, `discovering-skills`, and `release-prep`.

## Validation and Acceptance

The change is accepted when the version bump is reflected in `Cargo.toml` and `Cargo.lock`, release notes exist at `docs/releases/v0.3.0.md`, README and packaged guidance match current behavior, release package staging succeeds, the Rust baseline passes, and hygiene checks do not show secrets, local machine paths, raw target ids, or account-local identifiers in tracked public docs.

## Idempotence and Recovery

Release prep is mostly documentation plus a Cargo version bump. It is safe to rerun validation commands. If release package staging fails, inspect the staged tree and packaging script instead of changing the release notes blindly. If `Cargo.lock` changes beyond workspace package versions, inspect the diff before accepting it.

## Artifacts and Notes

Do not commit automatically. The user requested release preparation but controls the final commit, tag, push, and GitHub Release creation.

## Interfaces and Dependencies

No CLI command, JSON envelope, runtime behavior, or public Rust API should change in this slice. The only intended behavior-facing artifact is documentation and release metadata.

## Open Questions

None.
