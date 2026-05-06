# v0.7.0 release readiness

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is self-contained and describes how to prepare the `v0.7.0` release without adding features, refactors, dependency updates, or CI workflow changes.

## Purpose / Big Picture

The `v0.7.0` implementation and pre-release audit slices are complete. This release-readiness slice fixes the repository state for tag creation by updating the Cargo package version, changelog, GitHub Release notes, README release asset examples, current plan index, and release archive validation evidence.

Tag creation, push, and GitHub Release publication are out of scope and remain manual user actions after this commit.

## Progress

- [x] (2026-05-06T00:00Z) Audited release state, current version strings, README asset examples, existing release-note shape, plan index, roadmap, and release packaging script.
- [x] (2026-05-06T00:00Z) Archived the completed pre-release completion and refactor audit plan and created this release-readiness ExecPlan.
- [x] (2026-05-06T00:00Z) Bumped workspace package version to `0.7.0`.
- [x] (2026-05-06T00:00Z) Cut `CHANGELOG.md` `Unreleased` entries into `v0.7.0 - 2026-05-06`.
- [x] (2026-05-06T00:00Z) Added `docs/releases/v0.7.0.md`.
- [x] (2026-05-06T00:00Z) Updated README release asset examples and current roadmap / plan index.
- [x] (2026-05-06T00:00Z) Synchronized `Cargo.lock`.
- [x] (2026-05-06T00:00Z) Ran release package validation and confirmed `target/release/tv --version` prints `tv 0.7.0`.
- [x] (2026-05-06T00:00Z) Ran Rust baseline validation.
- [x] (2026-05-06T00:00Z) Ran release safety / hygiene checks and optional recent CI check.
- [x] (2026-05-06T00:00Z) Committed the release-prep changes.

## Surprises & Discoveries

- Observation: the broad hygiene grep still reports existing safety policy text, archived validation-command examples, and this plan's validation wording.
  Evidence: no new machine-local path, raw target id, account-local value, cookie, token, authorization value, or raw live payload was added by this release-prep slice.

- Observation: recent remote CI / release workflows are green.
  Evidence: `gh run list --limit 5` reported success for the latest five runs.

## Decision Log

- Decision: Keep this slice limited to release preparation.
  Rationale: `v0.7.0` feature work and pre-release audit are complete; adding fixes here would make the release-prep commit harder to review and tag.
  Date/Author: 2026-05-06 / Codex.

- Decision: Keep `docs/releases/v0.7.0.md` free of a top-level version heading.
  Rationale: GitHub Release titles already include the tag, and prior release notes avoid duplicating it in the body.
  Date/Author: 2026-05-06 / Codex.

## Outcomes & Retrospective

Release readiness is complete. The workspace version and lockfile are at `0.7.0`; `CHANGELOG.md`, README release asset examples, `docs/releases/v0.7.0.md`, roadmap, and plans index all reflect the release-prep state. Release archive staging includes the expected runtime files and excludes development-only skills. No feature, refactor, dependency, or CI workflow change was mixed into this slice.

## Context and Orientation

`v0.7.0` adds `tv observe chart`, opt-in live smoke evidence for observe and lab bars contracts, an observation workflow guide, and scanner-backed fundamentals field enrichment. It does not stabilize browserless bars, add a daemon/MCP server, split the binary, or add standalone event/calendar commands.

The release archive should continue to contain the binary, README, changelog, license, user-facing `AGENTS.md` and `CLAUDE.md`, and runtime-oriented skills only. Development-only skills remain excluded.

## Plan of Work

Prepare the versioned release state, validate package staging, run the standard Rust baseline, run public-doc hygiene checks, and commit the release-prep changes. Do not tag, push, create a GitHub Release, add commands, refactor code, update dependencies, or change CI workflows in this slice.

## Concrete Steps

From the repository root, run:

    cargo metadata --no-deps --format-version 1
    bash -n scripts/stage-release-package-files.sh
    cargo build --release --locked
    rm -rf target/release-package-smoke
    scripts/stage-release-package-files.sh target/release-package-smoke target/release/tv
    find target/release-package-smoke -maxdepth 4 -print | sort
    target/release/tv --version
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Also run hygiene checks over public docs and packaged assets for local paths, credentials, raw target ids, account-local metadata, and stale release asset examples. Optionally inspect recent CI with `gh run list --limit 5`, recording any failures separately from release-prep changes.

## Validation and Acceptance

Acceptance is reached when package version is `0.7.0`, `Cargo.lock` is synchronized, release notes and README asset examples reference `v0.7.0`, release archive staging contains expected runtime files and excludes development-only skills, validation commands pass, and no public docs contain new machine-local or private operational details.

## Idempotence and Recovery

If release build fails because the lockfile is stale, synchronize `Cargo.lock` after the version bump and rerun the release package validation. If CI status shows unrelated failures, do not fix them in this release-prep slice; report them separately.

## Artifacts and Notes

Validation evidence:

    cargo metadata --no-deps --format-version 1
    bash -n scripts/stage-release-package-files.sh
    cargo build --release --locked
    rm -rf target/release-package-smoke
    scripts/stage-release-package-files.sh target/release-package-smoke target/release/tv
    find target/release-package-smoke -maxdepth 4 -print | sort
    target/release/tv --version
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|account-local)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true
    rg -n 'v0\.6\.0|0\.6\.0' README.md docs/releases/v0.7.0.md packaging/agent/AGENTS.md CHANGELOG.md docs/plans/README.md docs/v0.7-roadmap.md || true
    gh run list --limit 5

All validation passed. The staged package includes `tv`, `README.md`, `CHANGELOG.md`, `LICENSE`, user-facing `AGENTS.md`, user-facing `CLAUDE.md`, and the runtime skill allowlist under both `.agents/skills/` and `.claude/skills/`. Development-only skills remain excluded. `target/release/tv --version` prints `tv 0.7.0`. The focused stale-version grep reported only historical `v0.6.0` references in changelog, roadmap, and the archived plans index.

## Interfaces and Dependencies

No CLI command behavior, JSON payload, Rust API, dependency, or CI workflow changes are introduced beyond the workspace package version.

## Open Questions

No open questions.
