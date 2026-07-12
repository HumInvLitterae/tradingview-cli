# Prepare the v0.26.0 release

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while working. Maintain
this document according to `.agents/PLANS.md`.

## Purpose / Big Picture

This plan prepares `v0.26.0` after the six robustness gates, pre-release
architecture audit, canonical-history sanitation, and Windows fixture
correction have all completed. It does not add another feature or refactor.

After this work, the workspace package version, changelog, curated GitHub
Release notes, README download examples, packaged agent guidance, and staged
release archive will all describe the same `v0.26.0` release. A human can verify
the result by building with the lockfile, staging the package, running the
staged binary's version command, inspecting the allowlisted archive contents,
and confirming the normal Rust and public-hygiene baselines are green.

This plan stops before creating a Git tag, pushing a tag or branch, or creating
a GitHub Release. Those remote actions require explicit project-owner approval
in a later turn.

## Progress

- [x] (2026-07-12) Created this release-readiness ExecPlan after CI run
  `29173925167` and focused review closed the Windows fixture blocker.
- [x] (2026-07-12) Archived the completed Windows CDP transport-failure fixture
  ExecPlan and updated the current plan index, roadmap, work inventory, and
  changelog transition.
- [x] (2026-07-12) Confirmed the release candidate worktree, recent commits,
  dependency state, CI result `29173925167`, release workflow, and package
  allowlist.
- [x] (2026-07-12) Bumped `[workspace.package].version` in `Cargo.toml` from
  `0.25.0` to `0.26.0` and synchronized `Cargo.lock` without updating
  dependencies.
- [x] (2026-07-12) Cut the `CHANGELOG.md` `Unreleased` entries into
  `v0.26.0 - 2026-07-12` and left a fresh empty `Unreleased` section.
- [x] (2026-07-12) Added curated GitHub Release notes at
  `docs/releases/v0.26.0.md` without a redundant top-level version heading.
- [x] (2026-07-12) Updated README release asset examples and verified the
  English/Japanese getting-started and packaged agent guidance against the
  shipped behavior.
- [x] (2026-07-12) Built with `--release --locked`, staged the release package,
  and inspected the explicit allowlist output.
- [x] (2026-07-12) Ran the full Rust baseline, metadata, public-hygiene,
  package-script, stale-version, guide-parity, and diff checks.
- [x] (2026-07-12) Recorded final local evidence. Tag, push, and GitHub Release
  creation remain owner-controlled actions.
- [x] (2026-07-12) Project owner published GitHub Release `v0.26.0` from
  release-preparation commit `5e7f48f`; local `main`, `origin/main`, and the
  release tag agree. Archived this completed plan during the v0.27 transition.

## Surprises & Discoveries

- Observation: release readiness begins from a fully green cross-platform CI
  result rather than from the earlier Windows-blocked state.
  Evidence: run `29173925167` passed Windows, Ubuntu, macOS, Clippy, Format, and
  both operating-system script-check jobs at commit `f06e88f`.

## Decision Log

- Decision: release `v0.26.0` as a robustness and I/O-correctness release
  without promoting another retained feature.
  Rationale: all six hardening gates and the architecture audit are complete;
  adding feature work now would invalidate the audited release candidate and
  delay a release whose main value is reliability.
  Date/Author: 2026-07-12 / Codex

- Decision: retain the private rollback bundles and local `main-backup` branch
  through the release.
  Rationale: canonical-history sanitation is complete, but deletion of those
  recovery artifacts is a separate post-release owner decision.
  Date/Author: 2026-07-12 / Codex

- Decision: use `2026-07-12` as the planned changelog release date.
  Rationale: this is the current project date and the release-readiness slice
  begins after all required gates became green. If the actual release occurs on
  a later date, update the changelog and release notes before the tag is made.
  Date/Author: 2026-07-12 / Codex

## Outcomes & Retrospective

The release-preparation changes completed locally and were released. The
workspace and all
member packages report `0.26.0`; the staged binary reports `tv 0.26.0`; the
package contains the binary, public docs, user-facing guides, eight runtime
skills in both supported skill roots, and no development-only skills. The full
Rust baseline and public hygiene checks are green. This plan itself stopped
before remote mutation as required; the project owner subsequently committed,
pushed, tagged, and published `v0.26.0`. Planning then moved to v0.27 without a
post-release correction.

## Context and Orientation

The repository is a virtual Cargo workspace whose shared package version is in
the root `Cargo.toml`. Member crates inherit that version, and `Cargo.lock` must
be regenerated after the workspace version changes. The shipped executable is
the `tv` binary from the `tradingview-cli` package.

`CHANGELOG.md` currently contains the complete `v0.26.0` work under
`Unreleased`. The user-facing release body belongs in
`docs/releases/v0.26.0.md`; it must summarize behavior rather than reproduce
internal gate chronology. `README.md` contains versioned release archive
examples. `docs/getting-started.md`, `docs/ja/getting-started.md`, and
`packaging/agent/AGENTS.md` are included or represented in release archives and
must not contradict the final release.

`scripts/stage-release-package-files.sh` owns the release archive allowlist.
It must continue to include the binary, `README.md`, `CHANGELOG.md`, `LICENSE`,
runtime `AGENTS.md` / `CLAUDE.md`, allowlisted runtime skills, and English and
Japanese getting-started docs. It must not broadly copy development-only skills
such as `continuity`, `conventional-commits`, `discovering-skills`, or
`release-prep`.

The release candidate includes the six v0.26 hardening outcomes: safe broken
pipe handling; buffered CDP events and absolute deadlines; configured transport
deadlines and HTTP client reuse; corrected bars heartbeat framing; consistent
HTTP failure taxonomy; and measured four-wide bounded concurrency. It also
includes the test-only cross-platform CDP/CLI failure fixtures. Canonical Git
history has been sanitized without changing the release candidate tree or
GitHub Release assets.

## Plan of Work

First inspect `git status`, recent commits, `Cargo.toml`, `Cargo.lock`,
`CHANGELOG.md`, `README.md`, `.github/workflows/release.yml`,
`scripts/stage-release-package-files.sh`, `docs/release-packaging.md`, and the
latest CI run. Do not update dependencies or workflows during this slice. If a
new release blocker appears, stop and record it rather than mixing its fix into
release preparation.

Set the root workspace package version to `0.26.0` and use Cargo metadata or a
locked build to synchronize the lockfile. Confirm that dependency versions do
not change. Move the existing `Unreleased` content into a dated
`v0.26.0 - 2026-07-12` section and create a new empty `Unreleased` heading for
future work.

Create `docs/releases/v0.26.0.md` as a concise GitHub Release body. Cover the
six robustness outcomes, bounded performance improvement, cross-platform test
stabilization, and the architecture-audit conclusion. Mention the canonical
history rewrite only as repository hygiene and downstream clone recovery where
useful; do not expose removed text, private paths, raw history, or rollback
artifact locations. Do not add ranking, recommendation, automatic fallback, or
source-mixing claims.

Update versioned README archive examples to `v0.26.0`. Review the packaged
guides for behavior drift, making only release-critical corrections. Preserve
the package staging script's explicit runtime-skill allowlist.

Build the release binary with the lockfile, stage it into a clean smoke
directory, inspect the file tree, and run the binary's version command. Then
run the complete baseline and public-hygiene checks. Record concise evidence in
this plan without raw payloads, machine-specific paths, credentials, target
identifiers, account-local metadata, or private rollback paths.

## Concrete Steps

Run all commands from the repository root. Ground the state with:

    git status --short --branch
    git log --oneline -10
    gh run view 29173925167 --json conclusion,headSha,jobs,url
    cargo metadata --no-deps --format-version 1
    bash -n scripts/stage-release-package-files.sh

After editing the version and release materials, validate the release package:

    cargo build --release --locked
    rm -rf target/release-package-smoke
    scripts/stage-release-package-files.sh target/release-package-smoke target/release/tv
    find target/release-package-smoke -maxdepth 4 -print | sort
    target/release/tv --version

The version command must report `tv 0.26.0`. The staged tree must include the
binary, public overview and changelog, license, packaged agent guides, runtime
skills and their required references, and both getting-started docs. It must
not contain development-only plans, notes, or skills.

Run the full baseline:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    git diff --check

Search the release-facing documents for stale `v0.25.0` asset examples. A
historical changelog or release-note reference is allowed; a current download
or package example is not.

## Validation and Acceptance

Acceptance requires all of the following. Cargo metadata and
`target/release/tv --version` report `0.26.0`. `Cargo.lock` changes only where
workspace member package versions require it, with no dependency update. The
new changelog section and curated release notes accurately describe the six
hardening outcomes and avoid unsupported product claims.

The release package stages successfully from an explicit allowlist and contains
all required runtime files without development-only skills or docs. Formatting,
strict Clippy, all workspace tests, metadata, public hygiene, package script
syntax, contributor-guide parity, and diff checks pass. Current release-facing
docs contain no machine-specific path, credential, raw payload, session or
target identifier, account-local metadata, or stale `v0.25.0` asset example.

The final worktree contains only release-preparation changes. No tag, push,
GitHub Release, dependency update, CI workflow change, new feature, or unrelated
refactor is included.

## Idempotence and Recovery

All local checks and package staging steps are repeatable. Remove and recreate
only `target/release-package-smoke`; never delete private history-sanitation
artifacts or the local `main-backup` branch. If a release check fails, correct
the release material or stop for a separate blocker plan. Do not weaken tests,
skip a platform, amend shared history, or mix a runtime fix into this plan.

Before a tag exists, the version and documentation edits can be corrected with
ordinary new commits. After a tag or GitHub Release exists, treat corrections
as a separate release decision rather than rewriting public release history.

## Artifacts and Notes

Entry evidence:

    Candidate commit: f06e88f
    CI run: 29173925167
    Windows tests: passed
    Ubuntu tests: passed
    macOS tests: passed
    Clippy and Format: passed
    Ubuntu and Windows script checks: passed
    Workspace version before preparation: 0.25.0

Final local evidence:

    Workspace version: 0.26.0
    Release binary: tv 0.26.0
    Package staging: passed
    Runtime skills: 8 allowlisted skills in .agents/skills and .claude/skills
    Development-only staged skills: none
    Public hygiene: passed, 558 tracked files inspected
    Full workspace tests: passed
    Strict Clippy and Format: passed
    Metadata, packaging syntax, guide parity, diff check: passed
    Stale release-facing v0.25.0 scan: clean
    Cross-platform CI: run 29173925167 passed

## Interfaces and Dependencies

This plan changes no public Rust API, CLI command, JSON/JSONL contract, source
boundary, dependency version, or CI workflow. It changes only package version
metadata and release-facing documentation unless validation reveals a blocker,
in which case implementation stops.

The release continues to use the existing `scripts/stage-release-package-files.sh`
interface:

    scripts/stage-release-package-files.sh <DESTINATION> <BINARY>

The GitHub Release body is the Markdown content of
`docs/releases/v0.26.0.md`; it must not begin with a redundant top-level
`# v0.26.0` heading.

## Open Questions

There is no unresolved release-scope or validation question. Release
`v0.26.0` was published on `2026-07-12`. Rollback-bundle deletion and
`main-backup` deletion remain separate owner decisions and are not implied by
this completed release plan.

Revision note (2026-07-12): created after cross-platform CI and focused review
closed the final Windows fixture blocker. This plan intentionally freezes
feature and refactor work and moves the repository into `v0.26.0` release
preparation.

Revision note (2026-07-12): archived after GitHub Release `v0.26.0` was
published from `5e7f48f`. The release outcome was recorded without changing the
historical boundary that remote release actions required separate owner
authorization.
