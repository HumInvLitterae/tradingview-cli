# User getting-started docs and release package guidance

This ExecPlan is a living document. The sections `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md` from the repository root. Keep this
document self-contained so a contributor can finish the work from this file
alone.

## Purpose / Big Picture

Before `v0.20.0` release readiness, the project needs a clearer user-facing
path from downloading a release archive to running first checks and using `tv`
with an AI agent. The existing README explains the CLI and source boundaries,
but it is not enough as a first-run guide for non-developer users.

After this change, a user can open the release archive, read README or the
bundled getting-started docs, run `tv --version`, perform a Desktop-free smoke
test, start TradingView Desktop for Desktop-backed commands, and understand how
to hand the tool to an AI agent without mixing data sources or exposing local
session metadata.

## Progress

- [x] (2026-05-25T18:18Z) Inspect current README, release packaging script,
  packaged agent guide, release packaging docs, roadmap, and current plan
  state.
- [x] (2026-05-25T18:25Z) Archive the completed v0.20 pre-release audit and
  make this user onboarding docs polish the current plan.
- [x] (2026-05-25T18:35Z) Add English and Japanese getting-started docs for
  release users and AI-agent use.
- [x] (2026-05-25T18:45Z) Update README, release packaging docs, packaged
  agent guide, package staging, roadmap, and changelog.
- [x] (2026-05-25T18:55Z) Run package staging, content validation, and hygiene
  checks.
- [x] (2026-05-25T19:20Z) Polish the English and Japanese getting-started
  docs so AI-agent usage comes before CLI examples and Japanese wording avoids
  unexplained jargon.

## Surprises & Discoveries

- Observation: Release archives previously copied only README, CHANGELOG,
  LICENSE, agent guides, and runtime skills, not repository docs.
  Evidence: `scripts/stage-release-package-files.sh` had no `docs/` copy step
  before this slice.

- Observation: README still pointed normal users at detailed command docs but
  did not provide a dedicated first-run path for archive users.
  Evidence: the Installation and Quick Start sections covered command examples
  but not a user-first download, unpack, PATH, Desktop readiness, and AI-agent
  handoff sequence.

## Decision Log

- Decision: Add both English and Japanese getting-started docs instead of
  expanding README into a long manual.
  Rationale: README should remain a concise project overview, while release
  users need a step-by-step guide. A Japanese guide addresses the user-facing
  documentation gap without translating the entire repository.
  Date/Author: 2026-05-25 / Codex.

- Decision: Include only `docs/getting-started.md` and
  `docs/ja/getting-started.md` in release archives.
  Rationale: Release archives should include user guidance that works after
  download, but development-only docs and plans should stay out of the package.
  Date/Author: 2026-05-25 / Codex.

- Decision: Keep package-manager installers, code signing, notarization, and
  broad Japanese documentation expansion deferred.
  Rationale: The release-blocking gap is first-run usability, not a new
  distribution channel or complete documentation translation.
  Date/Author: 2026-05-25 / Codex.

## Outcomes & Retrospective

The docs polish is complete. README now puts Installation and Quick Start
before deeper command taxonomy, points non-developer users to English and
Japanese getting-started docs, and the release staging script includes only
those user docs from `docs/`. The getting-started docs put AI-agent usage
before the detailed CLI smoke checks and explain local executable / current
working directory setup without assuming a specific agent application. The
Japanese guide avoids unexplained English jargon where ordinary Japanese
wording is clearer. No CLI behavior, JSON payload, Rust API, source contract,
or version number changed.

Validation passed:

- `git diff --check`
- `bash -n scripts/stage-release-package-files.sh`
- `cargo build --release --locked`
- release package smoke staging with `docs/getting-started.md` and
  `docs/ja/getting-started.md` included
- content `rg` for first-run and agent guidance
- old-version `rg`, with only historical `CHANGELOG.md` entries remaining
- hygiene `rg`, with existing policy / archived-plan matches and no newly
  introduced private data in the changed user docs

The next step returns to `v0.20.0 release readiness`.

## Context and Orientation

`tv` is released as native archives through GitHub Releases. Release archives
are staged by `scripts/stage-release-package-files.sh`. That script is an
explicit allowlist: it copies the binary, top-level public files, packaged
agent guides, and runtime skills. It should not copy all repository docs.

`README.md` is the public overview. `packaging/agent/AGENTS.md` is copied into
release archives as both `AGENTS.md` and `CLAUDE.md` for runtime users and
their agents. Repository docs under `docs/` usually stay in the repository, so
the getting-started docs added by this plan must be explicitly staged if they
are meant to be available inside release archives.

## Plan of Work

Add `docs/getting-started.md` as the English user-first guide and
`docs/ja/getting-started.md` as the Japanese user-first guide. Each guide
explains release archive download, unpacking, local execution or PATH
installation, `tv --version`, Desktop-free smoke tests, TradingView Desktop
readiness checks, multiple-target handling with `--target-id`, read-only-first
safety, and AI-agent use.

Update README to keep a short Installation / Quick Start path while linking to
the two user guides. Replace stale release-asset examples with `<tag>` style
examples so release readiness does not have to chase old version strings in
the user docs. Fix the stale roadmap link so normal readers land on the
current v0.20 roadmap.

Update `scripts/stage-release-package-files.sh` to create `docs/` and
`docs/ja/` under the package directory and copy only the two getting-started
docs. Update `docs/release-packaging.md` to document those contents. Update
`packaging/agent/AGENTS.md` so users and agents know the archive includes the
getting-started docs.

Update `docs/plans/README.md`, `docs/v0.20-roadmap.md`, and `CHANGELOG.md` to
record this slice as the current pre-release docs polish and to keep deferred
work visible.

## Concrete Steps

From the repository root, inspect the relevant docs and packaging script:

    sed -n '1,260p' README.md
    sed -n '1,240p' scripts/stage-release-package-files.sh
    sed -n '1,220p' packaging/agent/AGENTS.md
    sed -n '1,220p' docs/release-packaging.md

Archive the completed v0.20 pre-release audit:

    mkdir -p docs/plans/archives docs/ja
    mv docs/plans/tradingview-cli-v0.20-pre-release-audit.md docs/plans/archives/

Create and edit the user-facing docs, README links, packaging allowlist, and
current-plan docs. Do not change Rust code or command behavior.

Then validate:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    cargo build --release --locked
    rm -rf target/release-package-smoke
    scripts/stage-release-package-files.sh target/release-package-smoke target/release/tv
    find target/release-package-smoke -maxdepth 4 -print | sort

Expected staged package contents include:

    target/release-package-smoke/tv
    target/release-package-smoke/README.md
    target/release-package-smoke/CHANGELOG.md
    target/release-package-smoke/LICENSE
    target/release-package-smoke/AGENTS.md
    target/release-package-smoke/CLAUDE.md
    target/release-package-smoke/docs/getting-started.md
    target/release-package-smoke/docs/ja/getting-started.md
    target/release-package-smoke/.agents/skills/...
    target/release-package-smoke/.claude/skills/...

Run content and hygiene checks:

    rg -n "getting started|インストール|エージェント|tv launch|tv readiness|SHA256SUMS|--target-id|source boundary" README.md docs packaging/agent/AGENTS.md docs/release-packaging.md
    rg -n "v0\\.19\\.0|0\\.19\\.0" README.md docs/getting-started.md docs/ja/getting-started.md packaging/agent/AGENTS.md CHANGELOG.md
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|raw JSONL|account-local|target id|downstream-private)' README.md CHANGELOG.md docs packaging .agents/skills scripts crates || true

The old-version grep should not find release asset examples in the updated
user-facing docs. The hygiene grep may find historical policy text, but it
must not find newly introduced private data, raw payloads, or local paths.

## Validation and Acceptance

Acceptance is met when:

- README has a concise non-developer setup link to English and Japanese
  getting-started docs.
- `docs/getting-started.md` and `docs/ja/getting-started.md` explain download,
  unpacking, PATH or local execution, `tv --version`, Desktop-free smoke,
  TradingView Desktop readiness, multiple target handling, and AI-agent use.
- Release staging includes only those two user docs from `docs/`, not the
  whole documentation tree.
- `docs/release-packaging.md` and `packaging/agent/AGENTS.md` match the new
  archive contents.
- No CLI behavior, JSON payload, Rust API, or version number changes.
- Packaging and hygiene checks pass.
- The next recorded step is `v0.20.0 release readiness`.

## Idempotence and Recovery

The staging script removes and recreates the package directory, so repeating
the package smoke is safe. If the package smoke fails because the release
binary is missing, rerun `cargo build --release --locked` and then rerun the
staging command. If the docs copy step fails, check that both getting-started
docs exist and that the package script creates `docs/ja` before copying.

If this slice needs to be reverted, remove the two getting-started docs,
remove the docs copy step from the staging script, restore the current plan in
`docs/plans/README.md`, and move the archived audit back from
`docs/plans/archives/`.

## Artifacts and Notes

No raw live output, raw WebSocket frames, raw JSONL output, target ids,
account-local metadata, credentials, or local absolute paths should be added
to tracked docs.

## Interfaces and Dependencies

This slice adds no Rust API, CLI option, command, dependency, or JSON
contract. The only package interface change is that release archives now
include:

    docs/getting-started.md
    docs/ja/getting-started.md

The staging script remains the source of truth for release archive contents.

## Open Questions

None. Full Japanese documentation, installer channels, code signing,
notarization, and package-manager distribution are intentionally deferred.
