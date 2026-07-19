# Prepare the v0.29.0 release

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while work proceeds.
Maintain it according to `.agents/PLANS.md`.

## Purpose / Big Picture

This slice prepares `v0.29.0` from the independently reviewed candidate after
the completion audit and two bounded investigations closed without promoting
another production feature. It freezes feature work and aligns workspace
version metadata, the changelog, curated GitHub Release notes, the README
download example, packaged guidance, and the staged archive.

The shipped production change is intentionally narrow: Desktop CDP transport
errors may include an additive public-safe `failure_stage` detail naming target
listing, target selection, WebSocket connection, method call, event wait, or an
unknown transport boundary. Existing error kinds, messages, exit codes, and
transport behavior remain unchanged. There is no retry, reconnect, shared
session, broker, generalized wait, or timing field. Source inventories,
ignored measurement harnesses, and bounded live evidence are release evidence,
not additional runtime capabilities.

After implementation, a human can build the locked release binary, stage the
explicit package allowlist, run the staged binary's version command, and verify
that the Rust baseline and four pinned JavaScript contracts are green. This
plan stops before tag creation, branch or tag push, workflow mutation, or
GitHub Release creation. Those actions remain separately owner-controlled.

## Progress

- [x] (2026-07-19) Closed the completion audit, indicator-search reassessment,
  and consecutive-invocation investigation after focused review.
- [x] (2026-07-19) Created this release-readiness ExecPlan and synchronized the
  plan index, roadmap, work inventory, changelog, and local ledger.
- [x] (2026-07-19) Obtained focused independent plan review with no finding.
- [x] (2026-07-19) Grounded the frozen candidate at `7956726` from
  `v0.28.0..HEAD`, manifests, lockfile,
  workflows, package allowlist, and current worktree.
- [x] (2026-07-19) Bumped `[workspace.package].version` from `0.28.0` to `0.29.0` and
  synchronize only the seven workspace package versions in `Cargo.lock`.
- [x] (2026-07-19) Moved `Unreleased` entries into a dated `v0.29.0` section while retaining
  a fresh empty `Unreleased` section.
- [x] (2026-07-19) Added curated notes at `docs/releases/v0.29.0.md` without a redundant
  top-level version heading, and update the README asset example.
- [x] (2026-07-19) Ran four pinned JavaScript gates, the complete Rust baseline, metadata,
  and the locked release build.
- [x] (2026-07-19) Staged and inspected the explicit package; verified source and staged binary
  versions and both eight-skill roots.
- [x] (2026-07-19) Ran hygiene, workflow parsing, package syntax, guide parity,
  stale-version, and diff checks.
- [x] (2026-07-19) Obtained focused release-readiness review with no finding or
  remaining local blocker.
- [x] (2026-07-19) Stopped before tag, push, workflow, or GitHub Release mutation.

## Milestones

### Milestone: freeze and describe the candidate

Reproduce the `v0.28.0..HEAD` candidate and confirm that no dependency or
workflow change escaped the completed audit. Set all workspace packages to
`0.29.0`, cut the changelog, write curated release notes, and update versioned
download examples. Public artifacts must describe shipped behavior and keep
diagnostics distinct from research evidence.

### Milestone: prove the staged release package

Run executable JavaScript contracts and the Rust baseline, build with
`--release --locked`, stage the explicit archive allowlist, and inspect it.
Required public files and eight runtime skills must exist under both packaged
skill roots, development-only material must be absent, and source and staged
binaries must both report `tv 0.29.0`.

### Milestone: close local release readiness

Run every deterministic release gate, record concise evidence, and obtain
independent review. No release blocker, version drift, package drift, or
unsupported claim may remain. Remote publication is outside this plan.

## Surprises & Discoveries

- Observation: the version bump changed only the root workspace version and
  seven local package entries in `Cargo.lock`.
  Evidence: the manifest/lockfile diff contains no third-party package change.

- Observation: the package allowlist remains identical in size to v0.28.
  Evidence: staging produced 46 files and eight runtime skills under each of
  `.agents/skills` and `.claude/skills`; development-only material was absent.

- Observation: all deterministic gates remain green at workspace version
  `0.29.0`.
  Evidence: four pinned JavaScript contracts, formatting, strict Clippy,
  workspace tests, metadata, and the locked release build returned success.

## Decision Log

- Decision: release additive `failure_stage` diagnostics without retry or a
  public timing contract.
  Rationale: deterministic tests support stage attribution, while two reviewed
  runs found no transient transport failure justifying retry. Slow chart-read
  tails require a later operation-level measurement plan.
  Date/Author: 2026-07-19 / Codex

- Decision: treat inventories, ignored harnesses, and bounded live runs as
  evidence rather than user-facing features.
  Rationale: they changed no ordinary command surface or production retry
  behavior. Release notes must not claim indicator search, a reliability
  guarantee, or autonomous operation.
  Date/Author: 2026-07-19 / Codex

- Decision: do not refresh the completion audit.
  Rationale: both post-audit investigations were test-and-documentation-only
  and promoted no production implementation. A promoted production change was
  the reviewed refresh trigger.
  Date/Author: 2026-07-19 / Codex

## Outcomes & Retrospective

Local release preparation is complete pending independent review. Candidate
commit `7956726` is 18 commits and 31 paths after `v0.28.0`; completion-audit
follow-up added no production Rust beyond two ignored live harnesses. All seven
workspace packages and source and staged binaries report `0.29.0`.

The changelog, curated notes, README example, and packaged guidance describe
additive `failure_stage` diagnostics without claiming retry, timing, indicator
search, or autonomous operation. Four pinned JavaScript gates passed. Formatting,
strict Clippy, the workspace suite, metadata, locked build, hygiene, workflow
parsing, package syntax, guide parity, stale-version, and diff checks are green.
CLI unit tests reported 442 passed and 3 ignored; Desktop contracts reported 99
passed; CDP reported 45 passed and 1 ignored.

The staged package contains 46 files and eight runtime skills under each skill
root. No tag, push, workflow mutation, or GitHub Release publication occurred.
Independent release-readiness review found no finding. Local release readiness
is complete and ready to commit; publication remains owner-controlled.

## Context and Orientation

The latest public release is `v0.28.0` at commit `e47ba44`. Root `Cargo.toml`
defines one version inherited by seven workspace crates; `Cargo.lock` records
those seven local package versions. Record the exact reviewed `HEAD` before
changing release artifacts.

Production changes since `v0.28.0` live under `crates/cdp/` with small CLI
contract coverage. `crates/cdp/src/diagnostics.rs` defines internal stage
observations and public-safe mapping. `crates/cdp/src/transport.rs` and
`crates/cdp/src/client.rs` observe bounded stages. Ordinary clients gain no
retry, reconnect, or background work.

Test-only measurement code and ignored live harnesses live in
`crates/cdp/src/measurement.rs` and `crates/cli/tests/`. They established
evidence but are not commands. Indicator reassessment added no search command.
Consecutive-read evidence instead supports guidance to resolve a target once,
reuse its `target_cli_args`, and rediscover only after selection failure,
target-set change, or intentional chart change.

Four Node.js `24.18.0` gates in `mise.toml`, CI, and the release workflow run
generated JavaScript contracts for study values, Pine open/save, indicator
insertion, and three-point drawing. Ordinary Cargo tests remain Node-free.

`scripts/stage-release-package-files.sh` owns the package allowlist: binary,
README, changelog, license, packaged agent guides, English and Japanese
getting-started docs, and eight runtime skills under each of `.agents/skills`
and `.claude/skills`. Plans, notes, the local ledger, and development-only
skills must remain absent.

The local stashes `fable-plan` and
`recovered-indicator-search-prototype-2026-07-12` are unrelated preserved work.
Do not apply, drop, rewrite, or package them.

## Plan of Work

Inspect the worktree, exact candidate commit, `v0.28.0..HEAD` inventory,
manifests, lockfile, workflows, package script, changelog, README, and prior
release notes. Confirm that the only ordinary runtime contract to describe is
additive transport failure-stage diagnostics and that dependencies, feature
flags, and workflow semantics are unchanged. Stop and revise this plan if not.

Change only the workspace version to `0.29.0`, then use Cargo tooling to
synchronize the seven local package entries in `Cargo.lock`. Reject any
third-party dependency drift.

Move current `Unreleased` entries into a dated `v0.29.0` section and leave an
empty `Unreleased` section. Create `docs/releases/v0.29.0.md` without a
top-level version heading. Explain `failure_stage`, preserved error behavior,
target-handoff guidance, and deterministic validation. Mention inventories and
live probes only as evidence. Do not claim retry, reconnect, shared connection,
broker, timing metadata, indicator search, reliability guarantees, automatic
source mixing, ranking, recommendation, or trading judgment.

Update README asset examples to `v0.29.0`. Preserve the explicit runtime-skill
allowlist. Run complete deterministic gates, build the locked binary, stage a
fresh smoke package, inspect every path, and record aggregate public-safe
evidence only.

Obtain focused independent review before committing the prepared candidate.
Do not tag, push, trigger or rerun workflows, or create a GitHub Release without
a separate current-turn owner instruction.

## Concrete Steps

Run from the repository root. Ground the candidate:

    git status --short --branch
    git rev-parse HEAD
    git log --oneline v0.28.0..HEAD
    git diff --name-status v0.28.0..HEAD
    git stash list
    cargo metadata --no-deps --format-version 1
    bash -n scripts/stage-release-package-files.sh

Inspect release inputs:

    sed -n '1,100p' Cargo.toml
    sed -n '1,220p' CHANGELOG.md
    sed -n '1,220p' .github/workflows/release.yml
    sed -n '1,240p' scripts/stage-release-package-files.sh
    rg -n "v0\.28\.0|0\.28\.0|v0\.29\.0|0\.29\.0" README.md docs/getting-started.md docs/ja/getting-started.md docs/release-packaging.md packaging/agent/AGENTS.md CHANGELOG.md

After the version edit, inspect `git diff -- Cargo.toml Cargo.lock` and run
`cargo metadata --no-deps --format-version 1`. Only workspace package versions
may change.

Run all pinned JavaScript gates:

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

Recreate only the named package directory, then stage and inspect it:

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

Remote CI may be read with `gh run list` or `gh run view`, but this plan never
triggers, reruns, cancels, or mutates a workflow.

## Validation and Acceptance

All seven workspace packages and source and staged binaries must report
`0.29.0`. Changelog, curated notes, README example, package contents, and
guidance must agree with the reviewed candidate.

Manifest and lockfile changes may alter the root workspace version and seven
local package versions only. No third-party dependency, feature flag, or
workflow semantic may change.

All four pinned JavaScript gates, formatting, strict Clippy, workspace tests,
metadata, locked build, public hygiene, package syntax, workflow parsing,
guide parity, stale-version scan, and diff checks must pass. Ordinary Cargo
tests must remain Node-independent.

The package must contain required public files and exactly eight runtime skills
under each skill root. It must exclude plans, notes, ledgers, development-only
skills, raw evidence, identifiers, credentials, and machine paths.

`failure_stage` is additive diagnostic context, not a reliability guarantee or
retry authorization. Investigations and inventories remain evidence.
Independent review must find no release blocker before the prepared candidate
is committed. Remote publication is not acceptance.

## Idempotence and Recovery

Inspection and validation are repeatable. Package staging may recreate only
`target/release-package-smoke`. Never reset, clean, stash, apply, or drop
unrelated work.

If Cargo synchronization changes a third-party dependency, stop and correct
only the version edit through a reviewed patch. If a deterministic gate fails,
preserve it, fix only the owning artifact or contract, and rerun focused checks
before the complete baseline. If publication moves to another date, update the
changelog and notes before tagging.

## Artifacts and Notes

Record commit and path counts, version diff, gate and test counts, binary
versions, package file count, and skill counts. Keep evidence aggregate and
public-safe. Do not retain a one-off reviewer prompt in the repository.

Local evidence:

- Frozen candidate: `7956726`; 18 commits and 31 changed paths after `v0.28.0`.
- Workspace packages: seven at `0.29.0`; no third-party lockfile drift.
- Binary readback: source and staged binaries both `tv 0.29.0`.
- Package: 46 files, eight `.agents` skills, eight `.claude` skills.
- JavaScript contracts: four passed with pinned Node.js `24.18.0`.
- Rust baseline: CDP 45 passed/1 ignored, CLI 442 passed/3 ignored,
  Desktop contracts 99 passed, and all workspace/doc tests green.
- Public hygiene: self-test and 613 tracked files passed; the untracked release
  note was scanned directly before review.
- Remote and live mutations: none.

Revision note (2026-07-19): focused plan review was green. Prepared versioned
artifacts, ran the complete local release baseline, inspected the staged
package, and stopped for focused release-readiness review.

Revision note (2026-07-19): focused release-readiness review independently
confirmed the version-only Cargo diff, release claims, binary and package
contents, validation evidence, public hygiene, and owner-controlled publication
boundary. No local blocker remains; the plan is archived with no remote
mutation performed.

## Interfaces and Dependencies

This plan changes workspace version metadata and release documentation only.
It adds no command, option, payload field, source, fallback, feature flag,
dependency, or workflow semantic.

The additive `failure_stage` detail and its six-value public mapping are the
only new ordinary runtime contract since `v0.28.0`. Existing error kinds,
messages, exit codes, envelope fields, transport timeouts, FIFO limits, source
selection, and command semantics remain unchanged.

## Open Questions

- UNCONFIRMED: final publication date. Use `2026-07-19` during local preparation
  and revise it before tagging if publication occurs later.
- UNCONFIRMED: whether the owner requires fresh remote CI before tagging. Local
  validation does not substitute for a requested remote gate.

Revision note (2026-07-19): created after the reviewed completion audit and two
bounded investigations closed without promoted production implementation. It
freezes the additive transport-diagnostic candidate and stops before all remote
mutation.
