# v0.18.0 release readiness

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It prepares
the `v0.18.0` release without adding features, refactors, dependency updates,
or CI workflow changes.

## Purpose / Big Picture

The `v0.18.0` JSONL observation contract work is complete. The updated
pre-release audit found no blocker after the final summary-event slice.

This release-readiness slice fixes the repository state for tag creation by
updating the Cargo package version, changelog, GitHub Release notes, README
release asset examples, current plan index, and release archive validation
evidence.

Tag creation, push, and GitHub Release publication are out of scope and remain
manual user actions after this commit.

## Progress

- [x] (2026-05-17T04:20Z) Create this release-readiness ExecPlan and archive
  the completed v0.18 pre-release audit update.
- [x] (2026-05-17T04:25Z) Bump workspace package version to `0.18.0` and synchronize
  `Cargo.lock`.
- [x] (2026-05-17T04:30Z) Cut `CHANGELOG.md` `Unreleased` entries into
  `v0.18.0 - 2026-05-17`.
- [x] (2026-05-17T04:20Z) Add `docs/releases/v0.18.0.md`.
- [x] (2026-05-17T04:30Z) Update README release asset examples, packaged agent guide, and current
  roadmap / plan index.
- [x] (2026-05-17T04:55Z) Run release package validation and Rust baseline.
- [x] (2026-05-17T05:00Z) Run release safety / hygiene checks and optional
  recent CI check.
- [x] (2026-05-17T05:05Z) Prepare the release-prep changes for one local
  commit.

## Surprises & Discoveries

- Observation: A dependency update commit already exists before this release
  prep slice.
  Evidence: recent history includes `37ec3b7 build(deps): Update
  dependencies`, touching `Cargo.toml` and `Cargo.lock`.

- Observation: `cargo metadata --no-deps --format-version 1` synchronized the
  workspace lockfile package entries to `0.18.0`.
  Evidence: `Cargo.lock` now lists all internal workspace packages at
  `0.18.0`.

- Observation: Release package staging produced the expected runtime archive
  shape.
  Evidence: `scripts/stage-release-package-files.sh
  target/release-package-smoke target/release/tv` staged `tv`, `README.md`,
  `CHANGELOG.md`, `LICENSE`, user-facing `AGENTS.md` / `CLAUDE.md`, and
  runtime skills.

- Observation: The release binary reports the intended version.
  Evidence: `target/release/tv --version` printed `tv 0.18.0`.

- Observation: Release safety checks did not identify new private live data in
  public docs or packaged assets.
  Evidence: hygiene grep output was limited to existing policy text,
  historical archived-plan text, and test fixtures; the stale-version scan
  found only historical `v0.17.0` changelog entries.

- Observation: Recent GitHub Actions runs were green before this local release
  prep commit.
  Evidence: `gh run list --limit 5` showed the latest five runs as
  `completed` / `success`.

## Decision Log

- Decision: Keep this slice limited to release preparation.
  Rationale: `v0.18.0` implementation and audit work is complete; adding
  fixes here would mix release prep with implementation.
  Date/Author: 2026-05-17 / Codex.

- Decision: Respect the already-committed dependency update as current state
  and do not add another dependency update in this slice.
  Rationale: Release prep should only synchronize package versions and verify
  the locked release build.
  Date/Author: 2026-05-17 / Codex.

- Decision: Do not tag, push, or create the GitHub Release in this slice.
  Rationale: Repository release workflow is tag-triggered, and the user
  explicitly keeps remote publication as a manual follow-up.
  Date/Author: 2026-05-17 / Codex.

- Decision: Keep the GitHub Release note body free of a top-level
  `# v0.18.0` heading.
  Rationale: The GitHub Release title already carries the tag, and prior
  release-note hygiene avoids duplicate version headings in the rendered
  release page.
  Date/Author: 2026-05-17 / Codex.

## Outcomes & Retrospective

Release preparation is complete locally for `v0.18.0`.

Validation completed:

- `cargo metadata --no-deps --format-version 1`
- `bash -n scripts/stage-release-package-files.sh`
- `cargo build --release --locked`
- `scripts/stage-release-package-files.sh target/release-package-smoke target/release/tv`
- `target/release/tv --version`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `git diff --check`
- release safety / hygiene greps
- optional `gh run list --limit 5`

The release archive smoke contained the expected public runtime files and did
not stage development-only skills. Tag creation, push, and GitHub Release
publication were intentionally not performed.

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

    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw JSONL output|raw WebSocket|account-local|target id)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true
    rg -n "v0\\.17\\.0|0\\.17\\.0" README.md docs/releases/v0.18.0.md packaging/agent/AGENTS.md CHANGELOG.md

Optional:

    gh run list --limit 5

## Acceptance Criteria

- Workspace package version and `Cargo.lock` package entries are `0.18.0`.
- `CHANGELOG.md` contains `v0.18.0 - 2026-05-17` with user-facing release
  notes.
- `docs/releases/v0.18.0.md` exists and has no top-level `# v0.18.0`
  heading.
- README release asset examples point to `v0.18.0`.
- Release archive staging contains the binary, public docs, license,
  user-facing `AGENTS.md` / `CLAUDE.md`, and runtime skills only.
- Development-only skills are not staged.
- `target/release/tv --version` prints `tv 0.18.0`.
- No new raw live payloads, raw JSONL output, raw WebSocket frames, live target
  ids, account-local metadata, credentials, or local absolute paths are added
  to public docs or packaged assets.

## Interfaces and Dependencies

No public interface change is planned in this release-prep slice.
`observe_chart.v1` and `stream.v1` remain command-local JSONL event markers.
The final summary event remains an observation-window readback, not a
market-data sample.

## Open Questions

None.
