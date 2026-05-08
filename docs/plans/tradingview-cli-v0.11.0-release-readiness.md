# v0.11.0 release readiness

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is
self-contained and describes how to prepare the `v0.11.0` release without
adding features, refactors, dependency updates, or CI workflow changes.

## Purpose / Big Picture

The `v0.11.0` compare contract metadata slice and pre-release audit are
complete. This release-readiness slice fixes the repository state for tag
creation by updating the Cargo package version, changelog, GitHub Release
notes, README release asset examples, current plan index, and release archive
validation evidence.

Tag creation, push, and GitHub Release publication are out of scope and remain
manual user actions after this commit.

## Progress

- [x] (2026-05-08T05:12Z) Audited release state, current version strings,
  README asset examples, existing release-note shape, plan index, and release
  packaging script.
- [x] (2026-05-08T05:12Z) Archived the completed pre-release completion and
  refactor audit plan and created this release-readiness ExecPlan.
- [x] (2026-05-08T05:12Z) Bumped workspace package version to `0.11.0`.
- [x] (2026-05-08T05:12Z) Cut `CHANGELOG.md` `Unreleased` entries into
  `v0.11.0 - 2026-05-08`.
- [x] (2026-05-08T05:12Z) Added `docs/releases/v0.11.0.md`.
- [x] (2026-05-08T05:12Z) Updated README release asset examples and current
  roadmap / plan index.
- [x] (2026-05-08T05:12Z) Synchronized `Cargo.lock`.
- [x] (2026-05-08T05:12Z) Ran release package validation and confirmed
  `target/release/tv --version`
  prints `tv 0.11.0`.
- [x] (2026-05-08T05:12Z) Ran Rust baseline validation.
- [x] (2026-05-08T05:12Z) Ran release safety / hygiene checks and optional
  recent CI check.

## Surprises & Discoveries

- Observation: release archive packaging still uses an explicit runtime skill
  allowlist.
  Evidence: `scripts/stage-release-package-files.sh` copies runtime skills by
  name and excludes development-only skills such as `continuity`,
  `conventional-commits`, `discovering-skills`, and `release-prep`.

- Observation: `Cargo.lock` workspace package versions synchronized to
  `0.11.0` when the workspace package version was updated.
  Evidence: the TradingView workspace package entries in `Cargo.lock` now
  match `Cargo.toml`.

- Observation: the broad hygiene grep still reports existing safety policy
  text, archived validation-command examples, historical release references,
  and this plan's validation wording.
  Evidence: no new machine-local path, raw target id, account-local value,
  cookie, token, authorization value, or raw live payload was added by this
  release-prep slice.

- Observation: recent remote CI / release workflows are green.
  Evidence: `gh run list --limit 5` showed successful CI or Release workflows
  for the latest `main`, `v0.10.0`, and `v0.9.0` runs.

## Decision Log

- Decision: Keep this slice limited to release preparation.
  Rationale: `v0.11.0` feature work and pre-release audit are complete; adding
  fixes here would make the release-prep commit harder to review and tag.
  Date/Author: 2026-05-08 / Codex.

- Decision: Keep `docs/releases/v0.11.0.md` free of a top-level version
  heading.
  Rationale: GitHub Release titles already include the tag, and prior release
  notes avoid duplicating it in the body.
  Date/Author: 2026-05-08 / Codex.

## Outcomes & Retrospective

Release readiness is complete. The workspace version and lockfile are at
`0.11.0`; `CHANGELOG.md`, README release asset examples,
`docs/releases/v0.11.0.md`, roadmap, and plans index all reflect the
release-prep state. Release archive staging includes the expected runtime files
and excludes development-only skills. No feature, refactor, dependency, or CI
workflow change was mixed into this slice.

## Context and Orientation

`v0.11.0` adds additive contract metadata to `tv compare <SYMBOL>...`.
Successful compare payloads now include `contract_version: "compare.v1"`,
stable requested-order indexes, per-item follow-up hints, and
`summary.field_coverage`. The raw per-symbol `items[]` remains the evidence
source.

The release archive should continue to contain the binary, README, changelog,
license, user-facing `AGENTS.md` and `CLAUDE.md`, and runtime-oriented skills
only. Development-only skills remain excluded.

## Plan of Work

Prepare the versioned release state, validate package staging, run the
standard Rust baseline, run public-doc hygiene checks, and commit the
release-prep changes. Do not tag, push, create a GitHub Release, add commands,
refactor code, update dependencies, or change CI workflows in this slice.

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

Also run hygiene checks over public docs and packaged assets for local paths,
credentials, raw target ids, account-local metadata, and stale release asset
examples. Optionally inspect recent CI with `gh run list --limit 5`, recording
any failures separately from release-prep changes.

## Validation and Acceptance

Acceptance is reached when package version is `0.11.0`, `Cargo.lock` is
synchronized, release notes and README asset examples reference `v0.11.0`,
release archive staging contains expected runtime files and excludes
development-only skills, validation commands pass, and no public docs contain
new machine-local or private operational details.

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
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|account-local|target id)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts target/release-package-smoke || true
    rg -n "v0\\.10\\.0|0\\.10\\.0" README.md docs/releases/v0.11.0.md packaging/agent/AGENTS.md CHANGELOG.md || true
    gh run list --limit 5

The staged package contained `tv`, `README.md`, `CHANGELOG.md`, `LICENSE`,
user-facing `AGENTS.md`, user-facing `CLAUDE.md`, and the expected runtime
skills under `.agents/skills/` and `.claude/skills/`. `target/release/tv
--version` printed `tv 0.11.0`.

## Idempotence and Recovery

This slice is safe to rerun. Re-running `cargo metadata` after the version bump
keeps `Cargo.lock` synchronized. Re-running package staging removes and
recreates `target/release-package-smoke`.

If validation finds a release blocker, fix it in a separate focused commit or
plan unless it is inseparable from release preparation. Do not tag or push from
this slice.

## Interfaces and Dependencies

No CLI behavior, JSON payload, Rust API, dependency, release workflow, or
runtime skill changes are introduced. Only versioned release documentation,
package metadata, and plan pointers change.

## Open Questions

None. Manual tag creation, push, and GitHub Release publication remain outside
this slice.
