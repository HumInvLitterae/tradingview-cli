# v0.5.0 release readiness

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is self-contained so a new contributor can finish the release-preparation work without prior chat context.

## Purpose / Big Picture

This slice prepares the repository for a `v0.5.0` public release. It does not add features or refactor code. A successful result is a committed release-preparation state: Cargo package versions read `0.5.0`, the changelog and GitHub Release body describe `v0.5.0`, package staging works, and the normal Rust validation baseline passes. The user will still create the tag, push, and publish the release separately.

## Progress

- [x] (2026-05-02 06:55Z) Confirmed the previous commit is `refactor(market): Split fundamentals internals` and the working tree was clean before release edits.
- [x] (2026-05-02 07:00Z) Archived the completed pre-release refactor audit plan and created this release readiness plan.
- [x] (2026-05-02 07:05Z) Bumped workspace version to `0.5.0` and synchronized `Cargo.lock`.
- [x] (2026-05-02 07:12Z) Cut `CHANGELOG.md` `Unreleased` into `v0.5.0 - 2026-05-02` and added curated GitHub Release notes.
- [x] (2026-05-02 07:16Z) Updated README release examples, packaged agent guide, roadmap, and plans index. `docs/release-packaging.md` did not need a content change.
- [x] (2026-05-02 07:36Z) Release package staging, Rust baseline, metadata, diff, hygiene validation, and optional recent-CI check passed.
- [x] (2026-05-02 07:40Z) Committed the related changes as `chore(release): Prepare v0.5.0` (`4be9438` before final plan-status amend).

## Surprises & Discoveries

- `docs/release-packaging.md` already described the current release archive and runtime skill allowlist accurately, so no release-packaging content change was needed.
- The hygiene grep is intentionally broad and still reports existing policy language, archived validation-command examples, and secret-safety wording. No new live payload, local path, account-local id, cookie, token, or authorization value was added by this release-prep slice.

## Decision Log

- Decision: Keep this slice limited to release preparation.
  Rationale: The pre-release refactor audit found no blocker, so mixing new features or CI fixes into release prep would make the release commit harder to review.
  Date/Author: 2026-05-02 / Codex

- Decision: Use `docs/releases/v0.5.0.md` as the curated GitHub Release body and omit a top-level version heading.
  Rationale: The release workflow already sets the GitHub Release title to the tag and strips leading headings when present. Existing release notes follow the no-heading convention.
  Date/Author: 2026-05-02 / Codex

## Outcomes & Retrospective

Release preparation is ready to commit. Workspace package metadata reports all internal crates at `0.5.0`; release archive staging includes the binary, README, changelog, license, packaged agent guides, and runtime skills while excluding development-only skills; the Rust validation baseline passed; and recent CI on `main` was green before this local release-prep commit.

## Context and Orientation

This repository is a Rust workspace. The package version is centralized in the root `Cargo.toml` under `[workspace.package]`, and member crates inherit it with `version.workspace = true`. `Cargo.lock` records the resolved local package versions and must be updated after the version bump.

Public release notes live in `CHANGELOG.md` and `docs/releases/`. `docs/releases/<tag>.md` is used by `.github/workflows/release.yml` as the GitHub Release body when a tag is pushed. Release archives are assembled by `scripts/stage-release-package-files.sh`; the script copies the binary, README, changelog, license, user-facing `AGENTS.md` / `CLAUDE.md`, and an explicit allowlist of runtime skills.

## Plan of Work

Set the workspace package version to `0.5.0` and run a Cargo command that updates the lockfile without changing dependency versions. Convert the existing `CHANGELOG.md` `Unreleased` section into a dated `v0.5.0 - 2026-05-02` section and leave an empty `Unreleased` heading for future work.

Add `docs/releases/v0.5.0.md` with concise user-facing release notes. Mention the major `v0.5.0` additions: Desktop readiness diagnostics, lab-gated `tv bars`, Desktop-free fundamentals, fundamentals groups, Computer Use guidance cleanup, capability gap audit, and the pre-release fundamentals internals cleanup. Keep notes public-safe and do not include raw live payloads or local paths.

Update README release asset examples from `v0.4.1` to `v0.5.0`. Confirm the packaged agent guide and runtime skill allowlist already match the release contents; adjust only if they are stale. Update `docs/v0.5-roadmap.md` and `docs/plans/README.md` so they say the current plan is `v0.5.0` release readiness and that the release-prep state has been reached.

Finally build and stage a release package locally, run the normal Rust baseline, run hygiene checks, and commit.

## Concrete Steps

Work from the repository root.

Synchronize version metadata:

    cargo metadata --no-deps --format-version 1

Validate packaging:

    bash -n scripts/stage-release-package-files.sh
    cargo build --release --locked
    rm -rf target/release-package-smoke
    scripts/stage-release-package-files.sh target/release-package-smoke target/release/tv
    find target/release-package-smoke -maxdepth 4 -print | sort

Run the Rust baseline:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Run release safety checks:

    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true

Optionally check recent CI state:

    gh run list --limit 5

Do not tag, push, or create a GitHub Release in this slice.

## Validation and Acceptance

Acceptance is met when:

- Root `Cargo.toml` and `Cargo.lock` show the workspace packages at `0.5.0`.
- `CHANGELOG.md` has an empty `Unreleased` section and a dated `v0.5.0 - 2026-05-02` section.
- `docs/releases/v0.5.0.md` exists and has no top-level version heading.
- Release staging contains the binary, README, changelog, license, user-facing agent guides, and runtime skills, and excludes development-only skills.
- Rust baseline and diff checks pass.
- Hygiene grep reports only existing policy language, archived validation commands, or safety wording.

## Idempotence and Recovery

The release-prep commands are safe to rerun. `scripts/stage-release-package-files.sh` deletes and recreates the requested staging directory. If `cargo build --release --locked` fails because the lockfile was not synchronized, run `cargo metadata --no-deps --format-version 1` after the version bump and retry. If release notes accidentally include private local evidence, remove it before committing.

## Artifacts and Notes

The release archive skill allowlist is explicit. It must not be replaced with a broad copy of `.agents/skills/`, because development-only skills such as `continuity`, `conventional-commits`, `discovering-skills`, and `release-prep` must not be packaged.

## Interfaces and Dependencies

No public CLI interfaces change in this slice. The release workflow remains tag-triggered through `.github/workflows/release.yml`. The expected tag is `v0.5.0`; the Cargo package version is `0.5.0`.

## Open Questions

None.
