# Prepare the v0.31.2 dependency patch

This ExecPlan is a living document maintained according to `.agents/PLANS.md`.
Keep `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes &
Retrospective` current while work proceeds.

## Purpose / Big Picture

Prepare `v0.31.2` from released `v0.31.1` plus nine committed compatible
dependency updates. Users retain the same commands, options, JSON contracts,
source behavior, timeouts, recovery policy, workflows, runtime skills, and
package layout. This patch does not include a product feature or production
correction because no retained feature or engineering candidate currently
meets its evidence trigger.

The patch updates workspace versions, the changelog, curated release notes,
the README archive example, the locked release binary, and the existing
46-file package. It stops before tag creation, branch or tag push, workflow
execution, or GitHub Release publication.

## Progress

- [x] (2026-08-20) Confirmed `v0.31.1` is released from `720098d` and the
  worktree was clean at dependency candidate freeze.
- [x] (2026-08-20) Classified all nine dependency commits and confirmed
  `v0.31.1..7b33270` changes only `Cargo.toml` and `Cargo.lock`.
- [x] (2026-08-20) Closed and archived the published v0.31.1 plan locally.
- [x] (2026-08-20) Created this release-readiness ExecPlan and synchronized
  durable state without changing production source.
- [x] (2026-08-20) Bumped the workspace and seven local lockfile package
  versions to `0.31.2` without additional third-party drift.
- [x] (2026-08-20) Cut the changelog, added curated release notes, and updated
  the README archive example.
- [x] (2026-08-20) Ran four pinned JavaScript gates and the complete Rust
  release baseline successfully.
- [x] (2026-08-20) Built `--release --locked`, staged and inspected the explicit
  package, and verified source and staged binary versions.
- [x] (2026-08-20) Recorded aggregate local release evidence and stopped before
  remote release operations.
- [x] (2026-08-20) Completed focused release-readiness review and published
  `v0.31.2` from commit `03e9c24` under owner control.

## Surprises & Discoveries

- Observation: the candidate is larger than a lockfile-only refresh because
  three direct compatible constraints changed.
  Evidence: root `Cargo.toml` moves `clap` 4.6.3 to 4.6.6,
  `futures-util` 0.3.33 to 0.3.34, and `thiserror` 2.0.19 to 2.0.20.

- Observation: the candidate includes native TLS/build and ICU minor updates
  despite having no production source changes.
  Evidence: `aws-lc-rs`/`aws-lc-sys` and the ICU4X family change in
  `Cargo.lock`, so native release jobs and the complete baseline remain
  necessary rather than treating this as docs-only maintenance.

## Decision Log

- Decision: release the dependency candidate as `v0.31.2` before opening the
  next feature roadmap.
  Rationale: no retained feature or recovery candidate has met its trigger,
  while nine compatible dependency commits already form a coherent patch.
  Date/Author: 2026-08-20 / Codex

- Decision: include no retry, reconnect, timeout, session, broker, source, or
  command change.
  Rationale: dependency maintenance does not provide evidence for changing
  runtime behavior, and mixing such work would invalidate the narrow patch
  boundary.
  Date/Author: 2026-08-20 / Codex

- Decision: require the full deterministic baseline and native release CI.
  Rationale: the updated graph reaches CLI parsing, HTTP/2, TLS, WebSocket,
  Unicode URL processing, procedural macros, and native cryptographic builds.
  Date/Author: 2026-08-20 / Codex

## Outcomes & Retrospective

Local preparation is complete. All seven workspace packages and both source
and staged binaries report `0.31.2`. Four pinned JavaScript contracts,
formatting, strict workspace Clippy, the complete workspace suite and doctests,
metadata, public hygiene, workflow parsing, package syntax, guide parity, the
locked release build, and diff checks are green.

The staged package contains 46 files and exactly eight runtime skills under
each of `.agents/skills` and `.claude/skills`; plans, notes, the local ledger,
and development-only skills are absent. The release-preparation diff changes
only the root/local package versions and release documents after the reviewed
dependency candidate. Focused release-readiness review remains the local gate.
No tag, push, workflow execution, GitHub Release publication, live network
operation, Desktop mutation, or stash operation occurred.

The owner subsequently published `v0.31.2` from release commit `03e9c24`.
This plan is complete and archived. Later build-provenance and dependency work
is separate `v0.31.3` patch input and does not reopen this plan.

## Context and Orientation

`v0.31.1` is tag `720098d`. The nine later commits are already on `main` and
`origin/main` at `7b33270`. Their exact classification is:

- `7032a17`: `clap`/`clap_builder` and `ipnet` compatible updates.
- `0f66bb3`: `aho-corasick`, `clap`/`clap_builder`, `data-encoding`, and
  `regex-automata` compatible updates.
- `72b7fc2`: `aws-lc-rs`, `aws-lc-sys`, `cc`, `find-msvc-tools`,
  `thiserror`, and Wasm binding family compatible updates.
- `bf89910`: `bstr`, the futures 0.3 family, and `rustls-webpki` compatible
  updates.
- `fe8bd01`: `http-body-util` compatible update.
- `2bbf6df`: `cc`, `find-msvc-tools`, ICU4X 2.3 family, `litemap`,
  `potential_utf`, `tinystr`, `writeable`, `zerotrie`, and `zerovec`
  compatible updates.
- `803113a`: `h2`, `quinn-proto`, and `zerovec-derive` compatible updates.
- `eb495f1`: a later `h2` compatible update.
- `7b33270`: a later `zerovec` compatible update.

The cumulative candidate changes only root dependency constraints and resolved
lock selections. `git diff --quiet v0.31.1..7b33270 -- crates .github
mise.toml scripts` succeeds. There is no production source, workflow,
toolchain-task, or package-script change.

Seven workspace crates inherit `[workspace.package].version` from root
`Cargo.toml`. Release preparation may change only that root version and the
seven local `tradingview-*` entries in `Cargo.lock`; every reviewed third-party
version and checksum at `7b33270` must remain fixed.

The package contract remains owned by
`scripts/stage-release-package-files.sh`. It copies the binary, public docs,
packaged agent guide, and exactly eight runtime skills into both skill roots.
Plans, notes, `CONTINUITY.md`, and development-only skills remain excluded.

## Plan of Work

Freeze the nine commits listed above as the only dependency input. Record the
v0.31.1 closeout and this plan separately from versioned release artifacts.
Change the workspace version from `0.31.1` to `0.31.2` and synchronize the
seven local lock entries without allowing any additional third-party
selection.

Leave a fresh empty Unreleased changelog section, create a dated `v0.31.2`
section, add prose-first `docs/releases/v0.31.2.md`, and update the current
README archive example. Claims must remain limited to compatible dependency
maintenance and preserved behavior.

Run all four pinned JavaScript contracts sequentially, formatting, strict
workspace Clippy, the complete workspace suite and doctests, metadata, a
locked release build, public hygiene, package syntax, guide parity, workflow
YAML parsing, version checks, package inspection, and diff hygiene. Record only
aggregate evidence and stop before remote release operations.

## Concrete Steps

Run from the repository root:

    git status --short --branch
    git log --oneline v0.31.1..7b33270
    git diff --name-status v0.31.1..7b33270
    git diff --quiet v0.31.1..7b33270 -- crates .github mise.toml scripts
    git stash list

After version and release-note edits, run:

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

Inspect the staged package and require 46 files and eight skills per root:

    find target/release-package-smoke -type f | sort
    find target/release-package-smoke/.agents/skills -mindepth 1 -maxdepth 1 -type d | sort
    find target/release-package-smoke/.claude/skills -mindepth 1 -maxdepth 1 -type d | sort
    find target/release-package-smoke -type f | wc -l

## Validation and Acceptance

All seven workspace packages, source and staged binaries, changelog, release
notes, and current README example must agree on `0.31.2`. The release-prep
diff may change the root version, seven local lock versions, and release
documents only. All third-party selections and checksums at `7b33270` remain
unchanged during preparation.

All deterministic gates must pass. The staged tree remains 46 files with
exactly eight runtime skills under each skill root and no plans, notes, local
ledger, or development-only skill. Release notes must not claim a feature,
production fix, performance improvement, reliability improvement, retry, or
recovery behavior.

## Idempotence and Recovery

The deterministic commands and package staging may be rerun. Recreate only the
named staging directory. If Cargo selects another third-party version, stop
and restore only that unintended drift. Do not weaken tests, apply or drop
either stash, run live probes, or compensate for a failed gate with another
dependency update.

## Artifacts and Notes

Keep tracked evidence aggregate and repository-relative. Do not retain raw
payloads, endpoints, target IDs, account metadata, credentials, symbols,
environment values, machine paths, or temporary package contents.

## Interfaces and Dependencies

No public interface is added or changed. Root direct constraints end at
`clap = 4.6.6`, `futures-util = 0.3.34`, and `thiserror = 2.0.20`; all other
candidate updates are resolved transitive dependencies. No source provider,
feature flag, production ownership boundary, or package contract changes.

## Open Questions

There are no unresolved critical questions. Feature and communication-
resilience planning resumes only after this patch is closed.
