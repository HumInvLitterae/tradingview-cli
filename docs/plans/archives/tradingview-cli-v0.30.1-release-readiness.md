# Prepare the v0.30.1 dependency-only patch release

This ExecPlan is a living document maintained according to `.agents/PLANS.md`.
Keep Progress, Surprises & Discoveries, Decision Log, and Outcomes current.

## Purpose / Big Picture

Prepare `v0.30.1` from released `v0.30.0` plus the single committed dependency
refresh `1727a1b`. The patch must preserve all production behavior while
shipping compatible `clap`/`clap_derive`, `serde_json`, `hyper`, `libc`, and
`tokio` versions.
It updates workspace versions, changelog, release notes, README, locked build,
and the existing 46-file package. It stops before tag, push, workflow, or GitHub
Release publication.

## Progress

- [x] (2026-07-21) Confirmed `v0.30.0..HEAD` is one dependency commit changing
  only workspace dependency constraints and `Cargo.lock` selections.
- [x] (2026-07-21) Classified all six named dependency components and their
  existing production or build ownership paths.
- [x] (2026-07-21) Prepared `0.30.1` version metadata, changelog, release notes,
  README example, and durable planning state.
- [x] (2026-07-21) Ran four pinned JavaScript gates and the complete Rust baseline.
- [x] (2026-07-21) Built `--release --locked`, staged and inspected the 46-file
  package, and verified source/staged binary versions.
- [x] (2026-07-21) Ran public hygiene, workflow YAML, package syntax, parity, stale-version,
  and diff checks.
- [x] (2026-07-21) Obtained focused release-readiness review with no finding or
  local blocker and stopped before publication.

## Surprises & Discoveries

- The final dependency commit aligns direct `clap`, `serde_json`, and `tokio`
  workspace constraints in addition to refreshing the resolved lock graph. It
  changes no production source or feature.

## Decision Log

- Decision: treat this as a dependency-only patch rather than reopening v0.30
  feature planning or completion audit.
  Rationale: `v0.30.0..1727a1b` changes only compatible workspace dependency
  constraints and their resolved lock graph, and the release gates validate the
  exact graph without introducing behavior or policy work.
  Date/Author: 2026-07-21 / Codex

- Decision: preserve the existing package allowlist and all public claims from
  `v0.30.0`.
  Rationale: no source, command, contract, runtime-skill, workflow, or package
  ownership changed.
  Date/Author: 2026-07-21 / Codex

## Outcomes & Retrospective

Local patch artifacts and validation are complete. All seven workspace packages
and both source and staged binaries report `0.30.1`. Four pinned Node.js
contracts, strict workspace Clippy, the full workspace suite and doctests,
metadata, and the locked release build are green. CLI unit tests recorded 465
passed and 5 ignored; Desktop contracts recorded 100 passed; CDP recorded 45
passed and 1 ignored.

The staged package remains exactly 46 files with eight runtime skills under each
skill root and no development-only material. Public hygiene passed its self-test
and scanned 624 tracked files. Package syntax, guide parity, workflow YAML,
workspace-version, and diff checks are green. The initial stale-version command
matched the unrelated dependency package `tokio-tungstenite 0.30.0`; the
corrected check scopes the assertion to `[workspace.package]`, seven local
`tradingview-*` lock entries, and current public release surfaces.

Focused release-readiness review is green with no local blocker. No tag, push,
workflow execution, GitHub Release publication, live Desktop operation, or
stash mutation occurred.

## Context and Orientation

`v0.30.0` is tag `afd1f9a`. Commit `1727a1b` updates direct workspace
constraints for `clap`, `serde_json`, and `tokio`, plus resolved lock entries:
`clap` `4.6.2` to `4.6.3`, `clap_derive` `4.6.1` to `4.6.3`, `hyper` `1.10.1`
to `1.11.0`, `libc` `0.2.186` to `0.2.187`, and `tokio` `1.53.0` to `1.53.1`.
`clap` owns CLI parsing; `hyper`, `tokio`, and `libc` remain in existing HTTP,
WebSocket, async-runtime, TLS/build, and development paths. No production
source, feature, workflow, script, or documentation changed in that commit.

Seven workspace crates inherit `[workspace.package].version`. Release prep may
change only that root version and the seven local `tradingview-*` lock entries;
the reviewed dependency constraints and third-party selections must remain fixed.

## Plan of Work

Ground `v0.30.0..HEAD`, manifests, lockfile, package script, workflows, and
stashes. Update only workspace/local package versions to `0.30.1`, cut the
changelog, add prose-first release notes, and update the README example. Run the
four pinned JavaScript gates, strict Rust baseline, metadata, locked release
build, package staging, version readback, public hygiene, workflow parsing,
package/parity checks, and diff hygiene. Record aggregate evidence and obtain
focused review. Do not run ignored live tests, mutate Desktop, touch stashes,
tag, push, trigger workflows, or publish a Release.

## Concrete Steps

    git diff --name-status v0.30.0..HEAD
    git diff v0.30.0..HEAD -- Cargo.toml Cargo.lock crates .github mise.toml scripts
    mise run check:study-values-js
    mise run check:pine-open-js
    mise run check:indicator-insertion-js
    mise run check:three-point-drawing-js
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    cargo build --release --locked
    target/release/tv --version
    scripts/stage-release-package-files.sh target/release-package-smoke target/release/tv
    target/release-package-smoke/tv --version
    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    ruby -e 'require "yaml"; Dir[".github/workflows/*.yml"].each { |f| YAML.load_file(f) }'
    git diff --check

## Validation and Acceptance

The exact dependency graph, all seven workspace versions, source and staged
binaries, changelog, notes, and README must agree on `0.30.1`. All deterministic
gates must pass. The staged tree must remain 46 files with exactly eight runtime
skills under each skill root and no development-only material. Public claims
must say dependency-only and retain owner control of remote operations.

## Idempotence and Recovery

Rerun failed deterministic gates after fixing only their owning boundary. Do
not update another dependency, weaken a test, run live probes, apply/drop either
stash, or compensate with package drift. Recreate only the named staging tree.

## Artifacts and Notes

Keep evidence aggregate and repository-relative. Do not retain raw payloads,
target IDs, account metadata, credentials, symbols, machine paths, or temporary
package contents in tracked files.

## Interfaces and Dependencies

No production interface changes. The only dependency changes are the reviewed
compatible workspace constraints and lockfile selections in `1727a1b`;
workspace package versions move together from `0.30.0` to `0.30.1`.
