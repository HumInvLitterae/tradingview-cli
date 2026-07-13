# Prepare the v0.27.0 release

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while work proceeds.
Maintain it according to `.agents/PLANS.md`.

## Purpose / Big Picture

This slice prepares `v0.27.0` after the independently reviewed completion and
architecture audit found no release blocker or required pre-release
refactor. It freezes feature work and aligns package version metadata, the
changelog, curated GitHub Release notes, README download examples, packaged
agent guidance, and the staged release archive with the reviewed candidate.

After implementation, a human can build the locked release binary, stage the
explicit package allowlist, run the staged binary's version command, and verify
that both the ordinary Rust baseline and the separately pinned study-value
JavaScript contract gate are green. This plan stops before creating a Git tag,
pushing a branch or tag, or creating a GitHub Release. Those remote actions
remain owner-controlled.

## Progress

- [x] (2026-07-13) Closed and archived the independently reviewed completion
  and architecture audit.
- [x] (2026-07-13) Created this release-readiness ExecPlan and made it the
  current plan in the index, roadmap, work inventory, changelog, and continuity
  ledger.
- [x] (2026-07-13) Grounded the release candidate from the current worktree,
  recent commits, manifests, workflows, package allowlist, and latest relevant
  CI evidence.
- [x] (2026-07-13) Bumped `[workspace.package].version` from `0.26.0` to
  `0.27.0` and synchronized only workspace package entries in `Cargo.lock`.
- [x] (2026-07-13) Cut `CHANGELOG.md` `Unreleased` content into a dated
  `v0.27.0 - 2026-07-13` section, leaving a fresh empty `Unreleased` section.
- [x] (2026-07-13) Added curated GitHub Release notes at
  `docs/releases/v0.27.0.md` without a redundant top-level version heading.
- [x] (2026-07-13) Updated versioned README release-asset examples and verified
  packaged guidance against the reviewed v0.27 behavior.
- [x] (2026-07-13) Built with `--release --locked`, staged the explicit release
  package, and inspected its file and runtime-skill allowlists.
- [x] (2026-07-13) Ran focused release checks, the dedicated pinned-Node
  contract gate, and the complete Rust and public-hygiene baseline.
- [x] (2026-07-13) Recorded final local evidence and stopped before tag, push,
  or GitHub Release creation.
- [x] (2026-07-13) Completed independent review. The only finding was stale
  workspace-version wording in the roadmap and work inventory.
- [x] (2026-07-13) Corrected both current-state documents to `0.27.0` and
  reran the focused stale-version, hygiene, and diff checks.
- [x] (2026-07-13) Replaced informal ordinal work labels in current
  source-of-truth documents with descriptive names and repository paths.

## Surprises & Discoveries

- Observation: v0.27 has two intentionally separate test-tool layers.
  Evidence: ordinary `cargo test --workspace` is Rust-only and passes without
  Node.js on `PATH`, while `scripts/check-study-values-js-contract.py` executes
  the generated JavaScript helper under Node.js `24.18.0`. CI and the release
  workflow install that exact Node version for a dedicated required job.

- Observation: the latest remote CI evidence covers the released v0.26 tree,
  not the 19 local v0.27 commits.
  Evidence: CI and Release runs `29181079491` and `29181079505` are green at
  `5e7f48f`. The complete local release baseline is therefore required before
  the owner decides whether to push the candidate and obtain fresh CI evidence.

- Observation: checking the built binary version is a necessary package gate,
  not a redundant display check.
  Evidence: the first combined build/stage attempt ended before the release
  build completed and exposed a stale `tv 0.26.0` binary. A standalone locked
  build completed successfully; restaging then reported `tv 0.27.0` for both
  the source and staged binaries. No stale package was accepted as evidence.

## Decision Log

- Decision: release the four independently reviewed promoted feature areas
  without reopening deferred indicator search or exact-add work.
  Rationale: current-build Strategy Tester selection, selected-chart history
  paging, screenshot render readiness, and study-value identity form a coherent
  correctness release. Indicator-search readiness was not reproducible and its
  prototype remains deliberately outside the tracked tree.
  Date/Author: 2026-07-13 / Codex

- Decision: keep the JavaScript contract gate separate from ordinary Cargo
  tests and require it before release builds.
  Rationale: execution-level coverage is required for the generated helper,
  but the Rust baseline must not acquire an undeclared Node prerequisite. The
  existing CI and release workflow structure already enforces this boundary.
  Date/Author: 2026-07-13 / Codex

- Decision: use `2026-07-13` as the planned changelog date.
  Rationale: this is the current project date when release readiness starts. If
  publication occurs later, update the date before creating the tag.
  Date/Author: 2026-07-13 / Codex

- Decision: refer to plans and work items by descriptive name and repository
  path rather than by an invented ordinal alias.
  Rationale: a local numbering scheme is context that readers must reconstruct
  outside the named artifact. Descriptive references keep each current source
  self-contained and reduce coordination drift.
  Date/Author: 2026-07-13 / Project owner and Codex

## Outcomes & Retrospective

Local release preparation is complete. All seven workspace packages and the
locked release binary report `0.27.0`. The changelog, curated release notes,
README asset example, and staged package agree. The explicit package contains
the binary, public docs, packaged guides, both getting-started documents, and
eight runtime skills under each supported skill root; no development-only
skill or planning document is staged.

The dedicated JavaScript helper fixture passes under Node.js `24.18.0`.
Formatting, strict Clippy, the complete workspace suite with Node absent from
`PATH`, Cargo metadata, public-hygiene self-test and 579 tracked-file scan,
separate hygiene and whitespace checks for the new release note, workflow YAML
parsing, package-script syntax, contributor-guide parity, stale-version scan,
and diff checks are green. Independent review found only two stale
workspace-version statements; both now distinguish the pre-release-readiness
`0.26.0` state from the current `0.27.0` state. No feature, dependency,
workflow, source,
fallback, or payload semantic changed. No tag, push, workflow mutation, or
GitHub Release publication occurred.

After release preparation, current planning sources were also normalized to
use descriptive work names instead of an informal ordinal scheme. Contributor
guidance now makes that naming rule durable. Archived plans retain their
historical wording and are not current coordination sources.

## Context and Orientation

The latest release is `v0.26.0` from commit `5e7f48f`. The root `Cargo.toml`
contains the shared workspace package version, inherited by each member crate.
`Cargo.lock` records those member package versions and must be synchronized
after the workspace version changes without updating third-party dependencies.
The shipped executable is the `tv` binary from `tradingview-cli`.

The reviewed v0.27 candidate promotes four areas. Strategy Tester metrics,
trades, and equity use current semantic metadata, ambiguity-safe report
selection, shared public-safe `strategy_context`, and no automatic panel or
visibility mutation. The existing `tv range --from/--to` can request older
selected-chart main-series history under finite request and deadline controls;
it does not fall back to Desktop-free bars, OHLCV, export, or Replay.
`tv screenshot --wait-for-render` adds an opt-in bounded stable-observation
wait while immediate capture remains the default and timeout writes no image.
One-shot and streaming study values add same-instance public-safe identity,
compact inputs, visibility, and conservative kind without changing their
existing value-reader semantics.

Indicator search implementation and exact-match search-result add remain
deferred. The recovered search prototype stays in the named local stash and
must not be applied, dropped, or
packaged during release preparation. The audit also recorded the roughly
715-line selected-chart paging adapter as a future internal-split candidate
only if its responsibilities grow; no refactor is required before release.

`CHANGELOG.md` contains the complete candidate under `Unreleased`. Curated
GitHub Release notes belong in `docs/releases/v0.27.0.md` and must describe
user-visible outcomes rather than internal plan chronology. `README.md`
contains versioned release archive examples. `docs/getting-started.md`,
`docs/ja/getting-started.md`, and `packaging/agent/AGENTS.md` are included or
represented in release archives and must not contradict the final behavior.

`scripts/stage-release-package-files.sh` owns the archive allowlist. It copies
the binary, public overview and changelog, license, packaged `AGENTS.md` and
`CLAUDE.md`, English and Japanese getting-started docs, and eight runtime skills
into both supported skill roots. It must not copy development-only skills such
as `continuity`, `conventional-commits`, `discovering-skills`, or
`release-prep`, nor planning and audit documents.

The release workflow in `.github/workflows/release.yml` runs the dedicated
study-value JavaScript contract job with Node.js `24.18.0`; all four target
builds depend on that job. Each target also runs locked Rust tests, builds the
release binary, stages the package allowlist, and produces an archive. Do not
weaken or redesign that workflow in this slice.

## Plan of Work

First inspect the current worktree, recent commits, root manifest, lockfile,
changelog, README, release workflow, staging script, release packaging docs,
and the most recent relevant CI result. A failing required gate is a separate
release blocker: record it and stop instead of mixing its fix into release
preparation.

Change the root workspace package version to `0.27.0` and use Cargo tooling to
synchronize the lockfile. Inspect the lockfile diff to ensure only workspace
member package versions changed. Do not update dependencies, feature flags, or
workflow actions.

Move all current `Unreleased` entries into `v0.27.0 - 2026-07-13`, preserving a
fresh empty `Unreleased` section. Create `docs/releases/v0.27.0.md` as concise
GitHub Release notes covering Strategy Tester correctness, selected-chart
history paging, screenshot readiness, study-value identity, the dedicated
JavaScript contract gate, and the no-refactor audit verdict. Do not claim
indicator search, exact-add, automatic source mixing, ranking, recommendation,
or trading decisions.

Update current README archive examples to `v0.27.0`. Review getting-started and
packaged agent guidance for contradictions, but make only release-critical
corrections. Preserve the explicit runtime-skill allowlist.

Build the locked release binary, stage it into a clean smoke directory, inspect
the tree, and run its version command. Run the dedicated JavaScript gate under
the pinned Node version and run ordinary Cargo tests without making Node a
Cargo prerequisite. Finish with formatting, strict Clippy, metadata, public
hygiene, workflow parsing, package-script syntax, contributor-guide parity,
stale-version scans, and diff checks. Record concise evidence here and stop
before any remote mutation.

## Concrete Steps

Run every command from the repository root. Ground the state with:

    git status --short --branch
    git log --oneline -12
    cargo metadata --no-deps --format-version 1
    bash -n scripts/stage-release-package-files.sh
    git stash list

If current CI evidence matters, inspect it read-only with `gh run list` and
`gh run view`. Do not trigger, cancel, rerun, or otherwise mutate a workflow in
this plan.

After editing release metadata and documents, validate the package:

    cargo build --release --locked
    rm -rf target/release-package-smoke
    scripts/stage-release-package-files.sh target/release-package-smoke target/release/tv
    find target/release-package-smoke -maxdepth 5 -print | sort
    target/release/tv --version

The version command must report `tv 0.27.0`. The staged tree must contain the
binary, README, changelog, license, packaged agent guides, both getting-started
documents, and exactly the eight allowlisted runtime skills under both
`.agents/skills` and `.claude/skills`. It must not contain plans, audit notes,
or development-only skills.

Run the separate JavaScript and ordinary Rust gates:

    mise exec node@24.18.0 -- python3 scripts/check-study-values-js-contract.py
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    git diff --check

Parse `.github/workflows/ci.yml` and `.github/workflows/release.yml` as YAML
using an available local YAML parser. Confirm both pin Node.js `24.18.0` for the
study-value JavaScript contract, and confirm release build jobs depend on that
gate. Search release-facing documents for stale `v0.26.0` asset examples;
historical changelog and release-note references are allowed, but current
download examples are not.

## Validation and Acceptance

Acceptance requires Cargo metadata and `target/release/tv --version` to report
`0.27.0`. `Cargo.lock` may change only for workspace member versions; dependency
updates are outside scope. The dated changelog and curated notes must describe
only the four promoted v0.27 feature areas, their source and mutation
boundaries, the separate JavaScript gate, and the no-refactor audit verdict.

The explicit package staging command must succeed and include all required
runtime files without development-only plans or skills. The dedicated helper
fixture must execute under Node.js `24.18.0`, while ordinary Cargo tests remain
valid without an undeclared Node prerequisite. Formatting, strict Clippy, all
workspace tests, metadata, public hygiene, workflow parsing, package syntax,
guide parity, and diff checks must pass.

Release-facing docs and staged assets must contain no machine-specific path,
credential, raw payload, raw report value, session or target identifier, or
account-local metadata. The final worktree must contain only release-prep
changes. No tag, push, GitHub Release, dependency update, workflow change,
feature, fallback, payload semantic, or refactor belongs in this slice.

## Idempotence and Recovery

All local checks and package staging are repeatable. Remove and recreate only
`target/release-package-smoke`; do not alter the preserved search-prototype
stash or unrelated recovery artifacts. If a required check fails, correct only
release material or stop and create a separate blocker plan. Do not weaken a
test, skip a platform, rewrite history, or mix a runtime fix into this plan.

Before publication, version and documentation mistakes can be corrected with
ordinary commits. After a tag or GitHub Release exists, treat any correction as
a separate release decision rather than rewriting public release history.

## Artifacts and Notes

Entry evidence:

    Latest release: v0.26.0 at 5e7f48f
    Workspace version before preparation: 0.26.0
    Completion and architecture audit: independent review reported no findings
    Architecture blocker: none
    Preserved search prototype: named stash, untouched
    Promoted feature areas: four
    Runtime skills currently allowlisted: eight
    Dedicated JavaScript tool version: Node.js 24.18.0

Record final local evidence here during implementation. Do not paste raw JSON,
bars, report data, DOM, target IDs, account-local values, credentials, or
machine-specific paths.

Final local evidence:

    Workspace packages: seven at 0.27.0
    Release and staged binary: tv 0.27.0
    Cargo.lock: workspace package versions only
    Locked release build: passed
    Runtime skills: eight under each supported skill root
    Development-only staged skills: none
    Study-value JavaScript contract: passed with Node.js 24.18.0
    Rust-only workspace suite: passed with Node absent from PATH
    CLI unit tests: 410 passed, one dedicated JavaScript fixture ignored
    Strict Clippy and formatting: passed
    Public hygiene: passed, 579 tracked files plus new release note inspected
    Metadata, workflow YAML, package syntax, guide parity, diff: passed
    Remote mutation: none

## Interfaces and Dependencies

This plan changes no public Rust API, CLI command, option, JSON/JSONL contract,
source boundary, dependency version, or CI/release workflow. It changes only
workspace package version metadata and release-facing documentation unless a
separate blocker is discovered.

The staging interface remains:

    scripts/stage-release-package-files.sh <DESTINATION> <BINARY>

The release body is the Markdown content of `docs/releases/v0.27.0.md` and must
not begin with a redundant top-level `# v0.27.0` heading.

## Open Questions

No release-scope or architecture question remains. Local preparation,
validation, and focused review correction are complete. The planned date must
be updated if publication occurs after 2026-07-13. Push, CI confirmation, tag
creation, and GitHub Release publication require separate owner action.

Revision note (2026-07-13): Created after completion-audit independent review
reported no findings. The plan freezes the reviewed v0.27 candidate, preserves
the separate pinned-Node contract boundary, and stops before remote release
actions.

Revision note (2026-07-13): Replaced informal work-item numbering in current
planning sources with descriptive names and paths. This avoids importing a
conversation-local numbering scheme into durable project state.

Revision note (2026-07-13): Completed local release preparation and validation.
The release package and both test-tool layers are green; changes remain
uncommitted for review, and no remote action was performed.

Revision note (2026-07-13): Independent review found only stale current-version
wording in the roadmap and work inventory. Both now report `0.27.0`; focused
stale-version, hygiene, and diff checks are green.
