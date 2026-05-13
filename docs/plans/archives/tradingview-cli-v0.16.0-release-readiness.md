# v0.16.0 release readiness

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It prepares
the `v0.16.0` release without adding features, refactors, dependency updates,
or CI workflow changes.

## Purpose / Big Picture

The `v0.16.0` quote-data regular-session readback and stable browserless bars
work is complete. The pre-release audit found no blocker, and release-before
refactoring was judged unnecessary.

This release-readiness slice fixes the repository state for tag creation by
updating the Cargo package version, changelog, GitHub Release notes, README
release asset examples, current plan index, and release archive validation
evidence.

Tag creation, push, and GitHub Release publication are out of scope and remain
manual user actions after this commit.

## Progress

- [x] Created this release-readiness ExecPlan and archived the completed v0.16
  pre-release audit.
- [x] Bump workspace package version to `0.16.0` and synchronize
  `Cargo.lock`.
- [x] Cut `CHANGELOG.md` `Unreleased` entries into
  `v0.16.0 - 2026-05-13`.
- [x] Add `docs/releases/v0.16.0.md`.
- [x] Update README release asset examples and current roadmap / plan index.
- [x] Run release package validation and Rust baseline.
- [x] Run release safety / hygiene checks and optional recent CI check.
- [x] Prepare the release-prep changes for one local commit.

## Surprises & Discoveries

- `cargo metadata --no-deps --format-version 1` synchronized all workspace
  package entries in `Cargo.lock` to `0.16.0`.
- Release archive staging produced the expected binary, public docs, license,
  user-facing agent guides, and the runtime skill allowlist only. Development
  skills were not staged.
- `target/release/tv --version` printed `tv 0.16.0`.
- The release safety grep found existing policy, archived-plan, and test
  references to restricted terms, but no new release-note or package-guidance
  private data.
- The stale version scan found only historical `v0.15.0` changelog entries;
  README examples and the v0.16 release notes were updated.
- The optional `gh run list --limit 5` check could not be run in this
  environment because the `gh` executable was unavailable.

## Decision Log

- Decision: Keep this slice limited to release preparation.
  Rationale: `v0.16.0` feature work and the pre-release audit are complete;
  adding fixes here would mix release prep with implementation.
  Date/Author: 2026-05-13 / Codex.

- Decision: Do not refactor before `v0.16.0`.
  Rationale: The audited code is passing and contract-sensitive. Refactoring
  quote-data or bars immediately before release would raise wire-shape risk
  without a release blocker.
  Date/Author: 2026-05-13 / Codex.

- Decision: Keep `docs/releases/v0.16.0.md` free of a top-level version
  heading.
  Rationale: GitHub Release titles already include the tag, and prior release
  notes avoid duplicating it in the body.
  Date/Author: 2026-05-13 / Codex.

## Outcomes & Retrospective

The `v0.16.0` release-prep state is complete locally. The workspace version and
lockfile now use `0.16.0`; changelog, README release asset examples,
`docs/releases/v0.16.0.md`, the current plan index, and the v0.16 roadmap are
aligned with the shipped quote-data regular-session readback and stable
browserless `tv bars` work.

Release package validation, the Rust baseline, diff checks, and packaging
script syntax validation passed. No tag, push, GitHub Release publication,
feature work, refactor, dependency update, or CI workflow change was performed.

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

Run release safety checks:

    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|account-local|target id)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true
    rg -n "v0\\.15\\.0|0\\.15\\.0" README.md docs/releases/v0.16.0.md packaging/agent/AGENTS.md CHANGELOG.md

Optional:

    gh run list --limit 5

## Acceptance Criteria

- Workspace package version and `Cargo.lock` package entries are `0.16.0`.
- `CHANGELOG.md` contains `v0.16.0 - 2026-05-13` with user-facing release
  notes.
- `docs/releases/v0.16.0.md` exists and has no top-level `# v0.16.0` heading.
- README release asset examples point to `v0.16.0`.
- Release archive staging contains the binary, public docs, license,
  user-facing `AGENTS.md` / `CLAUDE.md`, and runtime skills only.
- Development-only skills are not staged.
- `target/release/tv --version` prints `tv 0.16.0`.
- No new raw live payloads, raw WebSocket frames, live target ids,
  account-local metadata, credentials, or local absolute paths are added to
  public docs or packaged assets.

## Interfaces and Dependencies

No public interface or dependency change is planned in this release-prep
slice. `quote_data.v1` and `bars.v1` remain command-local contract markers.

## Open Questions

None.
