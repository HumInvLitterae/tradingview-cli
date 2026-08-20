# Prepare the v0.31.1 dependency and documentation patch

This ExecPlan is a living document maintained according to `.agents/PLANS.md`.
Keep `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes &
Retrospective` current while work proceeds.

## Purpose / Big Picture

Prepare `v0.31.1` from released `v0.31.0` plus compatible lockfile refresh
`eceaa18` and post-release documentation closeout `73c79e1`. Users retain the
same commands, options, JSON contracts, source behavior, timeouts, recovery
policy, runtime skills, workflows, and package layout.

The patch updates workspace versions, the changelog, curated release notes,
the README archive example, the locked release binary, and the existing
46-file package. It stops before tag creation, branch or tag push, workflow
execution, or GitHub Release publication.

## Progress

- [x] (2026-08-01) Confirmed `v0.31.0` is tag and `origin/main` commit
  `c76546d`, with a clean published release state.
- [x] (2026-08-01) Classified `eceaa18` as a `Cargo.lock`-only refresh of five
  compatible transitive dependencies and no manifest, source, workflow, or
  package change.
- [x] (2026-08-01) Closed and archived the published v0.31 release state and
  corrected contributor source-of-truth routing in docs-only commit `73c79e1`.
- [x] (2026-08-01) Created this release-readiness ExecPlan and synchronized
  durable state without changing versioned artifacts.
- [x] (2026-08-01) Bumped the workspace and seven local lockfile package
  versions to `0.31.1` without additional third-party drift.
- [x] (2026-08-01) Cut the changelog, added curated release notes, and updated
  the README archive example.
- [x] (2026-08-01) Ran four pinned JavaScript gates and the complete Rust
  release baseline successfully.
- [x] (2026-08-01) Built `--release --locked`, staged and inspected the explicit
  package, and verified source and staged binary versions.
- [x] (2026-08-01) Recorded aggregate local release evidence and stopped before
  remote release operations.
- [x] (2026-08-01) Completed focused release-readiness review and published
  `v0.31.1` from commit `720098d` under owner control.

## Surprises & Discoveries

- Observation: the dependency refresh changes only resolved transitive
  packages, while direct workspace constraints remain fixed.
  Evidence: `git show eceaa18 -- Cargo.toml Cargo.lock` changes only
  `Cargo.lock`, with twelve additions and twelve deletions.

- Observation: the contributor guide still pointed to the v0.28 roadmap even
  after v0.31 publication.
  Evidence: docs-only closeout `73c79e1` now resolves the current roadmap and
  inventory through `docs/plans/README.md` instead of a fixed historical
  version.

- Observation: running all four Cargo-backed JavaScript gates concurrently
  caused build-directory lock contention but no test failure.
  Evidence: sequential TTY-backed reruns completed all four pinned contracts
  successfully; release validation therefore records the sequential results.

## Decision Log

- Decision: prepare a patch release before opening the next feature roadmap.
  Rationale: the committed candidate contains compatible dependency updates
  and documentation corrections only. Mixing a new feature would enlarge an
  otherwise behavior-preserving release.
  Date/Author: 2026-08-01 / Codex

- Decision: rerun the complete deterministic release baseline rather than
  relying only on focused dependency inspection.
  Rationale: `http`, `rustls`, WebSocket digest support, Tokio macros, and ICU
  URL dependencies reach multiple production crates even though the update is
  lockfile-only.
  Date/Author: 2026-08-01 / Codex

- Decision: retain every v0.31 command and recovery boundary unchanged.
  Rationale: no production defect or concrete retry trigger is part of this
  candidate; communication reliability work belongs to the next roadmap.
  Date/Author: 2026-08-01 / Codex

## Outcomes & Retrospective

Local preparation is complete. All seven workspace packages and both source
and staged binaries report `0.31.1`. The four pinned JavaScript contracts,
formatting, strict workspace Clippy, the complete workspace suite and doctests,
metadata, public hygiene, workflow parsing, package syntax, guide parity,
locked build, and diff checks are green.

The staged package contains 46 files and exactly eight runtime skills under
each of `.agents/skills` and `.claude/skills`; plans, notes, the local ledger,
and development-only skills are absent. The release-preparation diff changes
only root/local package versions, changelog, release notes, and the current
README example. Focused release-readiness review remains the only local gate.
No tag, push, workflow, GitHub Release, live network operation, Desktop
mutation, or stash operation occurred.

The owner subsequently published `v0.31.1` from release commit `720098d`.
The release plan is complete and archived. Nine later dependency-update
commits are separate `v0.31.2` patch input and do not reopen this plan.

## Context and Orientation

`v0.31.0` is released from `c76546d`. Commit `eceaa18` updates five transitive
packages in `Cargo.lock`: `displaydoc` 0.2.6 to 0.2.7, `http` 1.4.2 to 1.5.0,
`hybrid-array` 0.4.13 to 0.4.14, `rustls` 0.23.42 to 0.23.43, and
`tokio-macros` 2.7.1 to 2.7.2. The two procedural-macro packages now use the
already selected `syn` 3 graph. No direct dependency constraint changes.

`displaydoc` is reached through the ICU and URL parsing graph used by
`reqwest`. `http` is shared by the HTTP and WebSocket stacks. `hybrid-array`
is used through the WebSocket SHA-1 handshake graph. `rustls` backs existing
HTTPS and secure WebSocket connections. `tokio-macros` backs existing Tokio
runtime and test macros. These are existing ownership paths, not new features.

Commit `73c79e1` changes contributor and planning documents only. It archives
the completed v0.31.0 plan, records publication, and removes a fixed v0.28
source-of-truth reference. It changes no packaged runtime guide or command
contract.

Seven workspace crates inherit `[workspace.package].version` from root
`Cargo.toml`. Release preparation may change only that version and the seven
local `tradingview-*` lock entries. The reviewed third-party selections from
`eceaa18` must remain fixed.

The package contract is owned by `scripts/stage-release-package-files.sh`.
It copies the binary, public docs, packaged agent guide, and exactly eight
runtime skills into each skill root. Plans, notes, `CONTINUITY.md`, and
development-only skills remain excluded.

## Plan of Work

Freeze `v0.31.0..HEAD` as dependency refresh `eceaa18`, documentation closeout
`73c79e1`, this plan, and the later release-preparation commit. Before version
edits, verify that neither committed change modifies production source,
manifests, workflows, `mise.toml`, scripts, or package ownership.

Change the workspace version from `0.31.0` to `0.31.1` and synchronize the
seven local package entries in `Cargo.lock` without selecting any further
third-party update. Move the current Unreleased documentation entry into a
dated `v0.31.1` section, add prose-first `docs/releases/v0.31.1.md`, and update
the current README archive example.

Run all four pinned JavaScript contracts, formatting, strict workspace Clippy,
the complete workspace suite and doctests, metadata, a locked release build,
public hygiene, package syntax, guide parity, workflow YAML parsing, version
checks, and diff hygiene. Stage the explicit package and confirm its file and
skill counts. Record only aggregate evidence.

Do not add a feature, production correction, retry, reconnect, timeout change,
fallback, shared session, broker, dependency, workflow, or package-layout
change. Do not run ignored live tests. Stop before all remote release actions.

## Concrete Steps

Run from the repository root:

    git status --short --branch
    git log --oneline v0.31.0..HEAD
    git diff --name-status v0.31.0..HEAD
    git diff v0.31.0..HEAD -- Cargo.toml Cargo.lock crates .github mise.toml scripts
    git stash list

After version and release-note edits, inspect the candidate and run:

    cargo metadata --no-deps --format-version 1
    mise run check:study-values-js
    mise run check:pine-open-js
    mise run check:indicator-insertion-js
    mise run check:three-point-drawing-js
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo build --release --locked
    target/release/tv --version
    scripts/stage-release-package-files.sh target/release-package-smoke target/release/tv
    target/release-package-smoke/tv --version
    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    ruby -e 'require "yaml"; Dir[".github/workflows/*.{yml,yaml}"].sort.each { |f| YAML.load_file(f); puts "parsed #{f}" }'
    git diff --check

Inspect the staged package:

    find target/release-package-smoke -type f | sort
    find target/release-package-smoke/.agents/skills -mindepth 1 -maxdepth 1 -type d | sort
    find target/release-package-smoke/.claude/skills -mindepth 1 -maxdepth 1 -type d | sort
    find target/release-package-smoke -type f | wc -l

## Validation and Acceptance

All seven workspace packages, source and staged binaries, changelog, release
notes, and current README example must agree on `0.31.1`. The release-prep
diff may change root workspace version, seven local lock versions, and release
documents only. It must preserve all third-party versions and checksums selected
by `eceaa18`.

All deterministic gates must pass. The staged tree must remain 46 files with
exactly eight runtime skills under each skill root and no plans, notes, local
ledger, or development-only skill. Release notes must describe a compatible
dependency and documentation patch and must not claim a command, contract,
performance, reliability, or recovery improvement.

## Idempotence and Recovery

The deterministic commands and package staging may be rerun. Recreate only the
named staging directory. If Cargo selects an additional third-party version,
stop and restore only that unintended lockfile drift before continuing. Do not
weaken tests, apply or drop either stash, run live probes, or compensate for a
failed gate with another dependency update.

## Artifacts and Notes

Keep tracked evidence aggregate and repository-relative. Do not retain raw
payloads, endpoints, target IDs, account metadata, credentials, symbols,
environment values, machine paths, or temporary package contents.

## Interfaces and Dependencies

No public interface is added or changed. The final locked graph keeps the five
reviewed compatible transitive versions from `eceaa18`; no manifest dependency,
feature flag, source provider, or runtime ownership boundary changes.

## Open Questions

There are no unresolved critical questions. Communication resilience and the
next product roadmap begin only after this patch is closed.
