# v0.17.0 release readiness

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It prepares
the `v0.17.0` release without adding features, refactors, dependency updates,
or CI workflow changes.

## Purpose / Big Picture

The `v0.17.0` browserless historical bars maturity work is complete. The
refactor-aware pre-release audit found no blocker after the bars
crate-boundary cleanup, bars market internal split, and CLI contract test
split.

This release-readiness slice fixes the repository state for tag creation by
updating the Cargo package version, changelog, GitHub Release notes, README
release asset examples, current plan index, and release archive validation
evidence.

Tag creation, push, and GitHub Release publication are out of scope and remain
manual user actions after this commit.

## Progress

- [x] (2026-05-13T21:34Z) Create this release-readiness ExecPlan and archive
  the completed v0.17 pre-release audit update.
- [x] (2026-05-13T21:36Z) Bump workspace package version to `0.17.0` and synchronize
  `Cargo.lock`.
- [x] (2026-05-13T21:39Z) Cut `CHANGELOG.md` `Unreleased` entries into
  `v0.17.0 - 2026-05-14`.
- [x] (2026-05-13T21:40Z) Add `docs/releases/v0.17.0.md`.
- [x] (2026-05-13T21:42Z) Update README release asset examples and current roadmap / plan index.
- [x] (2026-05-13T21:58Z) Run release package validation and Rust baseline.
- [x] (2026-05-13T22:01Z) Run release safety / hygiene checks and optional recent CI check.
- [x] (2026-05-13T22:04Z) Prepare the release-prep changes for one local commit.

## Surprises & Discoveries

- `cargo metadata --no-deps --format-version 1` synchronized the workspace
  lockfile package entries to `0.17.0`; no dependency update was introduced.
- Release package staging contained the expected binary, public docs, license,
  user-facing `AGENTS.md` / `CLAUDE.md`, and runtime skills. Development-only
  skills were not staged.
- `target/release/tv --version` printed `tv 0.17.0`.
- The release safety grep reported existing policy text, archived plan
  examples, test fixtures, and validation-command examples. It did not reveal
  a new raw live payload, raw WebSocket frame, credential, live target id,
  account-local metadata, or local absolute path in the release-prep changes.
- The stale `v0.16.0` scan found only historical `CHANGELOG.md` entries.
- `gh run list --limit 5` showed the latest `main` CI run succeeding; the
  older visible failure was from the already-resolved `v0.16.0` release-prep
  iteration.

## Decision Log

- Decision: Keep this slice limited to release preparation.
  Rationale: `v0.17.0` implementation and refactor-audit work is complete;
  adding fixes here would mix release prep with implementation.
  Date/Author: 2026-05-13 / Codex.

- Decision: Do not tag, push, or create the GitHub Release in this slice.
  Rationale: Repository release workflow is tag-triggered, and the user
  explicitly keeps remote publication as a manual follow-up.
  Date/Author: 2026-05-13 / Codex.

- Decision: Keep the GitHub Release note body free of a top-level
  `# v0.17.0` heading.
  Rationale: The GitHub Release title already carries the tag, and prior
  release-note hygiene avoids duplicate version headings in the rendered
  release page.
  Date/Author: 2026-05-13 / Codex.

## Outcomes & Retrospective

`v0.17.0` release preparation is complete locally. The workspace version and
lockfile are `0.17.0`; `CHANGELOG.md`, `README.md`,
`docs/releases/v0.17.0.md`, the v0.17 roadmap, and plan index are aligned with
the released implementation state.

Validation passed:

- `cargo metadata --no-deps --format-version 1`
- `bash -n scripts/stage-release-package-files.sh`
- `cargo build --release --locked`
- release package staging and file listing
- `target/release/tv --version`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `git diff --check`

Tag creation, push, and GitHub Release publication were intentionally not
performed.

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
    rg -n "v0\\.16\\.0|0\\.16\\.0" README.md docs/releases/v0.17.0.md packaging/agent/AGENTS.md CHANGELOG.md

Optional:

    gh run list --limit 5

## Acceptance Criteria

- Workspace package version and `Cargo.lock` package entries are `0.17.0`.
- `CHANGELOG.md` contains `v0.17.0 - 2026-05-14` with user-facing release
  notes.
- `docs/releases/v0.17.0.md` exists and has no top-level `# v0.17.0`
  heading.
- README release asset examples point to `v0.17.0`.
- Release archive staging contains the binary, public docs, license,
  user-facing `AGENTS.md` / `CLAUDE.md`, and runtime skills only.
- Development-only skills are not staged.
- `target/release/tv --version` prints `tv 0.17.0`.
- No new raw live payloads, raw WebSocket frames, live target ids,
  account-local metadata, credentials, or local absolute paths are added to
  public docs or packaged assets.

## Interfaces and Dependencies

No public interface or dependency change is planned in this release-prep
slice. `bars.v1` remains the command-local contract marker for bounded
Desktop-free historical bars reads.

## Open Questions

None.
