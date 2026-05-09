# v0.13.0 release readiness

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is
self-contained and describes how to prepare the `v0.13.0` release without
adding features, refactors, dependency updates, or CI workflow changes.

## Purpose / Big Picture

The `v0.13.0` source/session boundary work is complete. The pre-release audit
and quote help wording polish found no blocker. This release-readiness slice
fixes the repository state for tag creation by updating the Cargo package
version, changelog, GitHub Release notes, README release asset examples,
current plan index, and release archive validation evidence.

Tag creation, push, and GitHub Release publication are out of scope and remain
manual user actions after this commit.

## Progress

- [x] (2026-05-09T18:10Z) Audited release state, current version strings,
  README asset examples, existing release-note shape, plan index, and release
  packaging script.
- [x] (2026-05-09T18:10Z) Archived the completed quote help wording plan and
  created this release-readiness ExecPlan.
- [x] (2026-05-09T18:35Z) Bump workspace package version to `0.13.0` and synchronize
  `Cargo.lock`.
- [x] (2026-05-09T18:35Z) Cut `CHANGELOG.md` `Unreleased` entries into
  `v0.13.0 - 2026-05-09`.
- [x] (2026-05-09T18:35Z) Add `docs/releases/v0.13.0.md`.
- [x] (2026-05-09T18:35Z) Update README release asset examples and current
  roadmap / plan index.
- [x] (2026-05-09T18:50Z) Run release package validation and Rust baseline.
- [x] (2026-05-09T18:50Z) Run release safety / hygiene checks and optional
  recent CI check.
- [x] (2026-05-09T18:55Z) Prepare the release-prep changes for one local
  commit.

## Surprises & Discoveries

- Observation: release archive packaging still uses an explicit runtime skill
  allowlist.
  Evidence: `scripts/stage-release-package-files.sh` copies named runtime
  skills and excludes development-only skills such as `continuity`,
  `conventional-commits`, `discovering-skills`, and `release-prep`.

- Observation: `Cargo.lock` synchronized every workspace crate package entry
  to `0.13.0` after the workspace package version bump.
  Evidence: `cargo metadata --no-deps --format-version 1` completed
  successfully after the version change.

- Observation: release archive staging produced the expected runtime package.
  Evidence: `scripts/stage-release-package-files.sh
  target/release-package-smoke target/release/tv` staged `tv`, `README.md`,
  `CHANGELOG.md`, `LICENSE`, runtime `AGENTS.md` / `CLAUDE.md`, and runtime
  skills while excluding development-only skills.

- Observation: safety greps reported existing policy language, archived
  validation examples, release-note safety wording, and historical changelog
  references only.
  Evidence: no new raw live payload, raw WebSocket frame, credential, live
  target id, account-local metadata, local absolute path, or stale README /
  release-note asset example was introduced.

- Observation: recent remote CI was green before the local release-prep commit.
  Evidence: `gh run list --limit 5` showed the latest Release and CI runs as
  `completed success`.

## Decision Log

- Decision: Keep this slice limited to release preparation.
  Rationale: `v0.13.0` feature work, pre-release audit, and quote help polish
  are complete; adding fixes here would make the release-prep commit harder to
  review and tag.
  Date/Author: 2026-05-09 / Codex.

- Decision: Keep `docs/releases/v0.13.0.md` free of a top-level version
  heading.
  Rationale: GitHub Release titles already include the tag, and prior release
  notes avoid duplicating it in the body.
  Date/Author: 2026-05-09 / Codex.

## Outcomes & Retrospective

Release readiness is complete. The workspace version and lockfile are at
`0.13.0`, `CHANGELOG.md` now contains `v0.13.0 - 2026-05-09`,
`docs/releases/v0.13.0.md` contains GitHub Release body text without a
duplicate top-level version heading, and README release asset examples point
to `v0.13.0`.

Validation passed with release package staging, `target/release/tv --version`
returning `tv 0.13.0`, formatting, clippy, full workspace tests, metadata,
diff check, packaging script syntax check, public-doc hygiene scans, stale
asset-version scan, and recent CI inspection.

No tag, push, GitHub Release publication, feature addition, refactor,
dependency update, CI workflow change, or runtime skill creation was performed
in this slice.

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
credentials, raw target ids, account-local metadata, raw live payloads, raw
WebSocket frames, and stale release asset examples. Optionally inspect recent
CI with `gh run list --limit 5`, recording any failures separately from
release-prep changes.

## Validation and Acceptance

Acceptance is reached when package version is `0.13.0`, `Cargo.lock` is
synchronized, release notes and README asset examples reference `v0.13.0`,
release archive staging contains expected runtime files and excludes
development-only skills, validation commands pass, and no public docs contain
new machine-local or private operational details.

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
