# Prepare the v0.30.0 release

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while work proceeds.
Maintain it according to `.agents/PLANS.md`.

## Purpose / Big Picture

This slice prepares `v0.30.0` from the independently reviewed frozen candidate.
It aligns workspace versions, the changelog, curated GitHub Release notes, the
README download example, packaged runtime guidance, and the staged archive.

The promoted user-facing feature is explicit chart-screenshot attachment for
bounded `tv replay log` steps. Users opt in with
`--attach-chart-screenshot --screenshot-output-dir <DIR>`. Successful Replay
steps attempt one deterministic no-overwrite PNG, screenshot failure remains
separate from Replay step failure, and optional OHLCV and screenshot
attachments compose. The release also corrects the existing
`replay_left_running` summary by retaining the post-step Replay state.

Chart-read latency attribution and renderer foreground feasibility are release
evidence, not runtime features. This release does not add public timing,
timeout changes, retry, reconnect, foreground activation, indicator search,
shared sessions, or brokers.

After implementation, a human can build the locked release binary, stage the
explicit 46-file package, run both binaries' version commands, and verify the
Rust baseline plus four pinned JavaScript contracts. This plan stops before tag
creation, branch or tag push, workflow mutation, or GitHub Release publication.

## Progress

- [x] (2026-07-20) Completed and archived the v0.30 completion and
  architecture audit after independent review with no release blocker.
- [x] (2026-07-20) Created this release-readiness ExecPlan and synchronized
  current planning state without changing versioned artifacts.
- [x] (2026-07-20) Classified the post-audit `6c0a1bd` lockfile-only dependency
  update and reran the full deterministic baseline on the dependency-bearing
  candidate; all gates are green.
- [x] (2026-07-20) Completed focused independent plan review; the dependency
  refresh and release artifact plan are green with no finding.
- [x] (2026-07-20) Grounded the exact release candidate and verified version, dependency,
  workflow, package, and stash state.
- [x] (2026-07-20) Bumped the workspace and seven local lockfile package versions to `0.30.0`.
- [x] (2026-07-20) Cut the changelog and added curated `docs/releases/v0.30.0.md` notes.
- [x] (2026-07-20) Updated the README release asset example.
- [x] (2026-07-20) Ran four pinned JavaScript gates, the complete Rust baseline, metadata,
  and a locked release build.
- [x] (2026-07-20) Staged and inspected the 46-file package; both source and staged binaries report `tv 0.30.0`.
- [x] (2026-07-20) Ran hygiene, workflow, stale-version, package, parity, and diff checks.
- [x] (2026-07-20) Obtained focused release-readiness review with no finding or
  local blocker and stopped before remote release operations.
- [x] (2026-07-20) Released `v0.30.0` from `afd1f9a`; branch, tag, workflow, and
  GitHub Release publication were completed outside this local plan.

## Milestones

### Milestone: align public release artifacts

Record the candidate HEAD, change only the workspace version and seven local
lockfile entries, cut the changelog, create curated release notes, and update
the README asset example. Completion means every current public version surface
states `0.30.0` and no historical release record is rewritten.

### Milestone: prove the locked package

Run production JavaScript contracts and the Rust baseline, build with
`--release --locked`, stage the package allowlist, and inspect every path.
Completion means both binaries report `tv 0.30.0`, both package skill roots
contain exactly eight runtime skills, and development material is absent.

### Milestone: close local readiness

Run all deterministic release gates and obtain independent review. Completion
means no stale version, unsupported release claim, package drift, dependency
drift, or local release blocker remains. Publication stays owner-controlled.

## Surprises & Discoveries

- `6c0a1bd` landed after the independently reviewed completion audit and before
  this plan was committed. It changes only `Cargo.lock`: `bytemuck` `1.25.1` to
  `1.25.2` through `image`, `fastrand` `2.4.1` to `2.5.0` through the existing
  `tempfile` development dependency, and `syn` `3.0.0` to `3.0.2` through
  existing procedural macros. No manifest, feature, production source,
  workflow, or `mise.toml` change accompanied it.
- The dependency-bearing candidate passed formatting, strict workspace Clippy,
  the full workspace suite and doctests, Cargo metadata, public-hygiene
  self-test and the 624-file tracked scan, release-package syntax, contributor-
  guide parity, workflow YAML parsing, and diff hygiene. This is a narrow audit
  refresh for the new lockfile state, not a substitute for release-readiness
  review.

## Decision Log

- Decision: describe Replay screenshot attachment as the sole promoted v0.30
  product feature.
  Rationale: latency and foreground plans were test-only investigations and
  explicitly promoted no production behavior.
  Date/Author: 2026-07-20 / Codex

- Decision: mention the post-step Replay-running correction as a fix, not a new
  control surface.
  Rationale: the smoke exposed an existing summary path that omitted a value
  already available from the Replay API.
  Date/Author: 2026-07-20 / Codex

- Decision: do not rerun any live matrix or Replay smoke during release prep.
  Rationale: the independently reviewed completion audit confirmed no code
  drift from the evidence points and no additional release property would be
  established by mutation.
  Date/Author: 2026-07-20 / Codex

- Decision: preserve the explicit runtime-skill package allowlist.
  Rationale: development-only skills, plans, notes, and the local ledger are not
  runtime-user artifacts.
  Date/Author: 2026-07-20 / Codex

- Decision: retain the reviewed compatible dependency patches and require the
  release-readiness review to cover the dependency-bearing candidate.
  Rationale: silently treating a post-audit lockfile update as part of the
  earlier frozen candidate would make that audit evidence stale. The update is
  narrow enough to refresh through exact lockfile classification plus the full
  deterministic baseline; reopening feature implementation or live evidence
  would not prove an additional property.
  Date/Author: 2026-07-20 / Codex

## Outcomes & Retrospective

Release artifacts are prepared locally. The workspace and all seven local
packages report `0.30.0`; the reviewed third-party dependency patches remain
unchanged. Four pinned Node.js contracts passed. Strict workspace Clippy, the
full workspace suite and doctests, Cargo metadata, and the locked release build
are green. The CLI unit suite recorded 465 passed and 5 ignored, Desktop CLI
contracts recorded 100 passed, and CDP recorded 45 passed and 1 ignored.

Both source and staged binaries report `tv 0.30.0`. The staged package contains
exactly 46 files and eight runtime skills under each of `.agents/skills` and
`.claude/skills`, with plans, notes, the continuity ledger, and development-only
skills absent. Public hygiene passed its self-test and scanned 624 tracked
files; package syntax, contributor-guide parity, workflow YAML, stale-version,
and diff checks are green.

Focused release-readiness review completed without a finding or local blocker.
The prepared commit became the released `v0.30.0` tag. No live Desktop operation
or stash mutation occurred during release preparation.

## Context and Orientation

The latest public release is `v0.29.0` at commit `a774142`. The reviewed v0.30
completion audit is archived at
`docs/plans/archives/tradingview-cli-v0.30-pre-release-audit.md`. Record the
exact release-prep HEAD before mutating artifacts. The current candidate also
contains the classified post-audit lockfile-only dependency update `6c0a1bd`;
the full deterministic baseline is green on that exact dependency state.

Root `Cargo.toml` defines version `0.29.0` under `[workspace.package]`. Seven
workspace crates inherit it, and `Cargo.lock` records seven corresponding local
package versions. Preserve the reviewed third-party versions from `6c0a1bd`;
release preparation may change only the seven local workspace package versions.

`CHANGELOG.md` has an `Unreleased` section containing v0.30 changes. Move those
entries under `## v0.30.0 - 2026-07-20` and retain a fresh empty `Unreleased`
section. Add `docs/releases/v0.30.0.md` as a GitHub Release body without a
redundant top-level version heading. Follow the prose-first format of
`docs/releases/v0.29.0.md`.

README currently uses `v0.29.0` in native archive examples. Update current
download examples to `v0.30.0`; do not rewrite historical release notes or
changelog sections.

Four Node.js `24.18.0` gates in `mise.toml`, CI, and the release workflow execute
generated production JavaScript contracts for study values, Pine open/save,
indicator insertion, and three-point drawing. Ordinary Cargo tests remain
Node-independent.

`scripts/stage-release-package-files.sh` owns the package allowlist: binary,
README, changelog, license, packaged agent guides, English and Japanese getting
started docs, and eight runtime skills under each agent root. Plans, notes,
`CONTINUITY.md`, and development-only skills must remain absent.

Preserve both local stashes. Do not apply, drop, rewrite, or package them.

## Plan of Work

First, inspect the worktree, exact HEAD, `v0.29.0..HEAD` inventory, root
manifest, lockfile, workflows, package script, changelog, README, prior release
notes, and stashes. Confirm the completion audit is the final feature boundary.
Stop if production, dependency, or workflow changes appeared after the archived
audit other than the classified `6c0a1bd` lockfile update, or if any further
candidate drift appears before focused plan review.

Second, change `[workspace.package].version` from `0.29.0` to `0.30.0` and use
Cargo tooling to synchronize the seven workspace package entries in
`Cargo.lock`. Inspect the release-prep diff against its grounded HEAD: preserve
the reviewed third-party versions and reject any additional dependency drift.

Third, move all current Unreleased entries into the dated v0.30.0 section and
leave a fresh empty Unreleased section. Write curated release notes covering
the opt-in Replay screenshot workflow, deterministic no-overwrite artifacts,
independent attachment failures, Replay-running correction, package guidance,
and deterministic validation. State that investigations were evidence and do
not claim reliability, public timing, foreground behavior, indicator search,
retry, reconnect, session, broker, source mixing, ranking, recommendations, or
trading judgment.

Fourth, update current README archive examples, run all deterministic gates,
build the locked release binary, stage a clean package, and inspect its paths.
Verify source and staged binaries report `tv 0.30.0` and the package remains 46
files with eight runtime skills under each root.

Finally, record aggregate evidence and obtain focused independent review before
committing prepared artifacts. Do not tag, push, trigger or rerun workflows, or
publish a GitHub Release without a separate current-turn owner instruction.

## Concrete Steps

Run from the repository root to ground the candidate:

    git status --short --branch
    git rev-parse HEAD
    git log --oneline v0.29.0..HEAD
    git diff --name-status v0.29.0..HEAD
    git stash list
    sed -n '1,80p' Cargo.toml
    sed -n '1,120p' CHANGELOG.md
    sed -n '1,120p' docs/releases/v0.29.0.md
    sed -n '1,220p' .github/workflows/release.yml
    sed -n '1,220p' scripts/stage-release-package-files.sh

After version edits, inspect only version metadata:

    git diff -- Cargo.toml Cargo.lock
    cargo metadata --no-deps --format-version 1

Only `[workspace.package].version` and the seven local `tradingview-*` package
version lines may change. No third-party package or checksum may drift.

Run the four pinned JavaScript gates:

    mise run check:study-values-js
    mise run check:pine-open-js
    mise run check:indicator-insertion-js
    mise run check:three-point-drawing-js

Run the Rust and release baseline:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    cargo build --release --locked
    target/release/tv --version

Stage and inspect the release package:

    scripts/stage-release-package-files.sh target/release-package-smoke target/release/tv
    find target/release-package-smoke -type f | sort
    target/release-package-smoke/tv --version

Verify both skill roots contain exactly the explicit eight runtime skills and
the package contains no `plans`, `notes`, `CONTINUITY`, `continuity`,
`conventional-commits`, `discovering-skills`, or `release-prep` path.

Run public and workflow checks:

    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    ruby -e 'require "yaml"; Dir[".github/workflows/*.yml"].each { |f| YAML.load_file(f) }'
    rg -n "v0\.29\.0|0\.29\.0" README.md Cargo.toml Cargo.lock CHANGELOG.md docs/releases/v0.30.0.md
    git diff --check

Historical `v0.29.0` changelog and release-note references are expected. Current
workspace, README asset, and v0.30 release surfaces must not be stale.

## Validation and Acceptance

Local release readiness is green only when all seven workspace packages and
both binaries report `0.30.0`; the lockfile has no dependency drift; changelog,
notes, and README agree; all four JavaScript gates and the complete Rust
baseline pass; the locked build succeeds; the package allowlist is exact;
public hygiene and workflow checks pass; and independent review finds no local
blocker or unsupported claim.

Release notes must distinguish shipped behavior from test-only evidence. Tag,
push, workflow execution, and GitHub Release publication remain outside local
acceptance and require explicit owner direction.

## Idempotence and Recovery

Artifact edits and validation are repeatable. Recreate only the named staging
directory. If Cargo changes a third-party lockfile entry, stop and restore only
that unintended lockfile drift before continuing. If any gate fails, fix the
narrow owning contract and rerun its focused gate plus the full affected
baseline. Do not compensate by weakening tests, omitting package files, or
rewriting historical release records.

Do not run ignored live tests or mutate TradingView. Do not apply or drop
stashes. Do not tag or push.

## Artifacts and Notes

Keep release evidence aggregate and repository-relative. Do not retain raw
target IDs, symbols, Runtime payloads, local paths, account metadata, or live
screenshots in tracked files. One-off reviewer prompts are not release
artifacts.

## Interfaces and Dependencies

No new production interface or dependency belongs in release prep. The stable
surface being packaged is the reviewed Replay screenshot attachment and
post-step running-state correction. Workspace package versions change together
from `0.29.0` to `0.30.0`; third-party dependency state remains fixed.

## Revision Note

2026-07-20: Initial release-readiness plan created after the independently
reviewed v0.30 completion audit was archived with no release blocker.

2026-07-20: Focused plan review accepted the narrow dependency refresh without
reopening the completion audit. Release artifacts, the locked build, the
46-file package, and all local validation gates are now complete; focused
release-readiness review remains pending.
