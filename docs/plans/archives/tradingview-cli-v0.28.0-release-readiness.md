# Prepare the v0.28.0 release

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while work proceeds.
Maintain it according to `.agents/PLANS.md`.

## Purpose / Big Picture

This slice prepares `v0.28.0` after the independently reviewed completion and
architecture audit found no release-blocking architecture issue or required
pre-release refactor. It freezes feature work and aligns workspace version
metadata, the changelog, curated GitHub Release notes, README download
examples, packaged agent guidance, and the staged release archive with the
reviewed candidate.

The release promotes five user-facing areas: safer Pine saved-script and source
operations, verified current-build indicator insertion, TradingView launch
environment hardening, native three-point `parallel_channel` creation, and
bounded Desktop-free scanner pagination. The dependency update already present
at `b28603a` is part of the reviewed candidate and must not be expanded during
release preparation.

After implementation, a human can build the locked release binary, stage the
explicit package allowlist, run the staged binary's version command, and verify
that the ordinary Rust baseline plus all four pinned JavaScript contracts are
green. This plan stops before creating a Git tag, pushing a branch or tag,
triggering workflows, or creating a GitHub Release. Those remote actions remain
owner-controlled.

## Progress

- [x] (2026-07-16) Closed and archived the independently reviewed v0.28
  completion and architecture audit.
- [x] (2026-07-16) Created this release-readiness ExecPlan and made it the
  current plan in the plan index, roadmap, work inventory, changelog, and local
  continuity ledger.
- [x] (2026-07-16) Grounded the release candidate from the current worktree, recent commits,
  manifests, lockfile, workflows, package allowlist, and relevant CI evidence.
- [x] (2026-07-16) Bumped `[workspace.package].version` from `0.27.0` to `0.28.0` and
  synchronize only workspace package versions in `Cargo.lock`.
- [x] (2026-07-16) Cut `CHANGELOG.md` `Unreleased` content into a dated `v0.28.0` section,
  leaving a fresh empty `Unreleased` section.
- [x] (2026-07-16) Added curated GitHub Release notes at `docs/releases/v0.28.0.md` without a
  redundant top-level version heading.
- [x] (2026-07-16) Updated versioned README release-asset examples and verified packaged
  guidance against reviewed v0.28 behavior.
- [x] (2026-07-16) Built with `--release --locked`, staged the explicit release package, and
  inspect its file and runtime-skill allowlists.
- [x] (2026-07-16) Ran all four pinned JavaScript gates, the complete Rust baseline, locked
  release build, metadata, hygiene, packaging, workflow, parity, and diff
  checks.
- [x] (2026-07-16) Recorded final local evidence and stopped before tag, push, workflow, or
  GitHub Release mutation.
- [x] (2026-07-16) Completed independent review with no findings or release
  blocker. Release preparation is ready to commit.

## Milestones

### Milestone: align versioned release artifacts

Set the workspace and lockfile package versions to `0.28.0`, cut the changelog,
write curated release notes, and update versioned README examples. This
milestone is complete when every public release artifact describes the reviewed
candidate without claiming deferred work.

### Milestone: prove the staged release package

Build the locked release binary, stage the explicit allowlist, inspect the
resulting tree, and run both source and staged binaries with `--version`. This
milestone is complete when required files and eight runtime skills exist under
both packaged skill roots and no development-only skill or planning document is
present.

### Milestone: close local release readiness

Run all deterministic gates, record concise public-safe evidence, and obtain
independent review. This milestone is complete only when no release blocker or
artifact drift remains. Tagging, pushing, and GitHub Release publication remain
outside this plan.

## Surprises & Discoveries

- Observation: the version bump changed only the seven workspace package
  entries in `Cargo.lock`.
  Evidence: `git diff b28603a -- Cargo.toml Cargo.lock` contains the root
  workspace version and seven `0.27.0` to `0.28.0` package changes; no
  third-party package version changed.

- Observation: the explicit release package remains compact despite carrying
  two skill roots.
  Evidence: staging produced 46 files, including eight runtime skills under
  `.agents/skills` and the same eight under `.claude/skills`. Development-only
  skills and `docs/plans` were absent.

- Observation: the Desktop CLI contract suite remains the longest local gate.
  Evidence: all 99 tests passed, with connection-unavailable coverage consuming
  most of the baseline elapsed time. This is not a release blocker.

## Decision Log

- Decision: release the five independently reviewed promoted areas without
  reopening deferred feasibility work.
  Rationale: Pine, indicator insertion, launch hardening, native three-point
  drawing, and bounded scanner pagination form the reviewed v0.28 candidate.
  Windows MSIX launch, finite-`f64` right offset, width-derived geometry, other
  three-point shapes, and indicator search lack the required completed
  evidence or implementation.
  Date/Author: 2026-07-16 / Codex

- Decision: retain dependency commit `b28603a` as part of the frozen release
  candidate and perform no additional dependency update.
  Rationale: the completion audit classified the direct `clap 4.6.2` update and
  compatible lockfile changes, and the full baseline passed. Expanding
  dependency scope during release preparation would invalidate that evidence.
  Date/Author: 2026-07-16 / Codex

- Decision: require all four JavaScript contracts before release builds while
  preserving a Node-free ordinary Cargo baseline.
  Rationale: study-value identity, Pine open/save, indicator insertion, and
  three-point drawing execute generated JavaScript that needs pinned
  execution-level coverage. Ordinary Rust contributors should not acquire an
  implicit Node prerequisite.
  Date/Author: 2026-07-16 / Codex

## Outcomes & Retrospective

Local release preparation is complete. All seven workspace packages and both
the source and staged binaries report `0.28.0`. The changelog, curated release
notes, README asset example, and explicit package agree with the reviewed
candidate. No new dependency, feature flag, command, source, fallback, or
payload semantic was introduced during release preparation.

All four pinned JavaScript gates passed. Formatting, strict Clippy, the complete
workspace suite, Cargo metadata, locked release build, public-hygiene self-test
and 596-file tracked scan, release-note-specific hygiene, workflow YAML parsing,
package-script syntax, contributor-guide parity, stale-version scan, and diff
checks are green. CLI unit tests reported 442 passed and 3 ignored; Desktop CLI
contracts reported 99 passed; scanner reported 36 passed.

The staged package contains 46 files and eight runtime skills under each skill
root. It excludes development-only skills, plans, and local ledgers. No tag,
push, workflow mutation, or GitHub Release publication occurred. Independent
review reported no findings. Local release readiness is complete and ready to
commit.

## Context and Orientation

The latest release is `v0.27.0` from commit `73ee3a2`. The root `Cargo.toml`
contains the shared workspace package version inherited by all seven crates.
`Cargo.lock` records those workspace package versions and the already-reviewed
third-party dependency set. During the version bump, inspect the lockfile diff
and reject any new third-party update.

The reviewed v0.28 candidate promotes:

- Pine saved-script binding and source operations that fail closed, verify
  explicit save/source readback, normalize only line endings, and sanitize
  Runtime diagnostics.
- `tv indicator add` through one current chart-owned inserter signature,
  exactly-once mutation, JavaScript-safe keyed inputs, and immediate same-ID
  readback.
- `tv launch` removal of inherited `ELECTRON_RUN_AS_NODE` from direct spawn and
  normal macOS system launch, with correct fallback and child-state precedence.
- `tv draw shape` explicit third-point support for native
  `parallel_channel`, requiring exactly one entity and verified native identity
  and point readback.
- `tv scanner scan` explicit offset and bounded aggregate pagination with a
  100-row page cap, 10,000-row and 100-request aggregate caps, raw page
  completeness, first-seen dedupe, sequential drift metadata, and no partial
  successful aggregate.

Four pinned Node.js `24.18.0` gates are defined in `mise.toml` and required by
CI and release workflows. Release builds depend on study-value, Pine open/save,
indicator insertion, and three-point drawing jobs. Do not weaken or merge these
gates into ordinary Cargo tests.

`scripts/stage-release-package-files.sh` owns the release archive allowlist. It
must copy the binary, README, changelog, license, packaged `AGENTS.md` and
`CLAUDE.md`, English and Japanese getting-started docs, and eight runtime skills
under both `.agents/skills` and `.claude/skills`. It must not broadly copy
development-only skills, plans, notes, or local ledgers.

Current release guidance lives in `README.md`, `CHANGELOG.md`,
`docs/release-packaging.md`, `docs/getting-started.md`,
`docs/ja/getting-started.md`, `packaging/agent/AGENTS.md`, and
`.github/workflows/release.yml`. Curated release notes belong at
`docs/releases/v0.28.0.md`.

The local stashes `fable-plan` and
`recovered-indicator-search-prototype-2026-07-12` are unrelated preserved work.
Do not apply, drop, rewrite, or package them.

## Plan of Work

First inspect the current worktree, recent commits, root manifest, lockfile,
changelog, README, release workflow, staging script, packaging docs, and recent
read-only CI evidence if available. A failing required gate is a separate
release blocker; record it and stop instead of mixing an unrelated fix into
release preparation.

Change only the workspace package version to `0.28.0` and use Cargo tooling to
synchronize workspace package entries in `Cargo.lock`. Compare the resulting
lockfile against `b28603a` and confirm that third-party package versions do not
change.

Move all current `Unreleased` entries into a dated `v0.28.0` section, preserving
a fresh empty `Unreleased` section. Use the actual publication-preparation date
and revise it if publication happens later. Create concise GitHub Release notes
covering the five promoted areas, public-safe diagnostics, the four executable
JavaScript gates, and the no-refactor audit verdict.

Do not claim Windows MSIX package-identity launch, finite-`f64` right-offset
restoration, width-derived geometry, other three-point shapes, indicator
search, automatic source mixing, ranking, recommendation, or trading
judgment.

Update README archive examples to `v0.28.0`. Review getting-started and packaged
agent guidance for contradictions, but make only release-critical corrections.
Preserve the explicit runtime-skill allowlist.

Build the locked release binary and stage it into a clean smoke directory.
Inspect the full staged tree, verify source and staged binary versions, and
confirm no development-only material is included.

Run all four JavaScript gates and the complete Rust and release baseline. Record
concise counts and pass/fail evidence without raw payloads, account-local
identifiers, target IDs, source text, credentials, or machine-specific paths.
Stop for independent review and do not perform any remote mutation.

## Concrete Steps

Run every command from the repository root. Ground the state with:

    git status --short --branch
    git log --oneline -15
    git stash list
    cargo metadata --no-deps --format-version 1
    bash -n scripts/stage-release-package-files.sh

Inspect release inputs:

    sed -n '1,100p' Cargo.toml
    sed -n '1,220p' CHANGELOG.md
    rg -n "v0\.27\.0|0\.27\.0|v0\.28\.0|0\.28\.0" README.md docs/getting-started.md docs/ja/getting-started.md docs/release-packaging.md packaging/agent/AGENTS.md CHANGELOG.md
    sed -n '1,180p' .github/workflows/release.yml
    sed -n '1,240p' scripts/stage-release-package-files.sh

After editing version metadata, verify that third-party dependency versions did
not change relative to `b28603a`. Workspace package versions are expected to
change from `0.27.0` to `0.28.0`; other package-version changes are not:

    git diff b28603a -- Cargo.toml Cargo.lock
    cargo metadata --no-deps --format-version 1

Run all JavaScript gates:

    mise run check:study-values-js
    mise run check:pine-open-js
    mise run check:indicator-insertion-js
    mise run check:three-point-drawing-js

Run the complete Rust and release baseline:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    cargo build --release --locked
    target/release/tv --version

Stage and inspect the release package:

    rm -rf target/release-package-smoke
    scripts/stage-release-package-files.sh target/release-package-smoke target/release/tv
    find target/release-package-smoke -maxdepth 5 -print | sort
    target/release-package-smoke/tv --version

Run public and workflow checks:

    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    ruby -e 'require "yaml"; YAML.load_file(".github/workflows/ci.yml"); YAML.load_file(".github/workflows/release.yml"); puts "workflow YAML parsed"'
    git diff --check

If current remote CI evidence is consulted, use read-only commands such as
`gh run list` and `gh run view`. Do not trigger, rerun, cancel, or mutate a
workflow.

## Validation and Acceptance

Acceptance requires all seven workspace crates and both source and staged
binaries to report `0.28.0`. The changelog, curated release note, README
examples, packaged guidance, and archive contents must agree with the reviewed
candidate.

The lockfile version-bump diff may change workspace package versions only.
Third-party dependencies must remain at the versions reviewed in `b28603a`.

All four pinned JavaScript gates must pass with Node.js `24.18.0`. The ordinary
Cargo baseline must remain Node-independent. Formatting, strict Clippy,
workspace tests, metadata, locked release build, public hygiene, package
syntax, workflow parsing, guide parity, and diff checks must pass.

The staged package must contain the required public files and exactly the
intended runtime skills under both skill roots. It must not contain plans,
notes, local ledgers, development-only skills, raw evidence, credentials,
account-local identifiers, target IDs, or machine-specific paths.

Independent review must report no unresolved release blocker before release
preparation is committed. Tag creation, push, workflow mutation, and GitHub
Release publication are not acceptance steps in this plan.

## Idempotence and Recovery

Inspection and validation are repeatable. Release-package staging may recreate
only `target/release-package-smoke`. Do not reset, clean, stash, apply, or drop
unrelated work.

If Cargo synchronization changes a third-party dependency, stop and restore
only the release-preparation version edit through a reviewed patch; do not run
an opportunistic dependency update. If a deterministic gate fails, preserve
the failure, correct only its owning release artifact or contract, and rerun
focused validation before the full baseline.

If the planned release date changes before publication, update both changelog
and release notes before tagging. If remote CI is stale or unavailable, record
that local validation is not remote evidence; do not trigger a workflow without
explicit owner authorization.

## Artifacts and Notes

Record package file counts, runtime-skill counts, binary version output, test
counts, and pass/fail summaries only. Do not copy raw JSON, Runtime payloads,
Pine source, scanner rows, drawing IDs, target IDs, account-local IDs, local
paths, credentials, or environment values into tracked files.

Prepare a self-contained read-only reviewer prompt after implementation. Do not
retain a one-off prompt in the tracked tree unless it has reusable project
value.

Local evidence:

- Workspace packages: seven at `0.28.0`.
- Binary readback: source and staged binaries both `tv 0.28.0`.
- Staged package: 46 files, eight `.agents` skills, eight `.claude` skills.
- JavaScript contracts: four passed with pinned Node.js `24.18.0`.
- Rust baseline: CLI 442 passed and 3 ignored; Desktop contracts 99 passed;
  scanner 36 passed; all workspace and doc tests green.
- Public hygiene: 596 tracked files plus the untracked release note checked
  directly.
- No live TradingView operation or remote mutation was performed.

## Interfaces and Dependencies

This plan changes workspace package version metadata and release documentation
only. It introduces no command, option, payload field, source, fallback,
feature flag, production dependency, or workflow semantic.

The reviewed third-party dependency set at `b28603a` is authoritative for
release preparation. Existing JSON envelopes, source metadata, Pine
diagnostics, indicator result typing, launch semantics, drawing payloads,
scanner pagination contracts, and package allowlists remain unchanged.

## Open Questions

- UNCONFIRMED: the final publication date. Use `2026-07-16` during local
  preparation and revise it before tagging if publication occurs later.
- UNCONFIRMED: whether fresh remote CI will be required by the owner before
  tagging. Local validation does not substitute for a requested remote gate.

Revision note (2026-07-16): created after the v0.28 completion and architecture
audit closed with no release-blocking architecture issue. The plan freezes the
reviewed dependency-bearing candidate and stops before every remote mutation.

Revision note (2026-07-16): independent release-readiness review reported no
findings. Versioned artifacts, locked build, staged package, four JavaScript
gates, and the full baseline are green; tag, push, workflow, and GitHub Release
actions remain owner-controlled.
