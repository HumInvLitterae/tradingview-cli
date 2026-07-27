# Prepare the v0.31.0 release

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while work proceeds.
Maintain it according to `.agents/PLANS.md`.

## Purpose / Big Picture

This slice prepares `v0.31.0` from the completed and archived v0.31 candidate
audit. It aligns workspace versions, the changelog, curated GitHub Release
notes, the README download example, and the staged native archive.

The release has two promoted user-visible improvements. Bounded Desktop-free
`tv bars --from/--to` accepts normalized one-minute timeframe `1` and its
existing `1m` alias while preserving `bars.v1`, period-start timestamps, the
5,000 returned-bar cap, and existing coverage and truncation fields.
Desktop-free bars failures may also include additive public-safe
`source_failure_stage` details that identify the failing lifecycle boundary
without changing existing error kinds, messages, details, or exit codes.

This release does not add retry, reconnect, timeout changes, fallback, source
substitution, shared sessions, a broker, additional date-range timeframes,
ranges above 5,000 returned bars, or a new event/calendar source. The retained
candidate comparison is release evidence rather than runtime functionality.

After preparation, a human can build the locked release binary, stage the
explicit package, run both binaries' version commands, and verify the Rust
baseline plus four pinned JavaScript contracts. This plan stops before tag
creation, branch or tag push, workflow execution, or GitHub Release
publication.

## Progress

- [x] (2026-07-28) Completed and archived the v0.31 completion and
  architecture audit with no local release blocker or required refactor.
- [x] (2026-07-28) Created this release-readiness ExecPlan and synchronized
  current planning state without changing versioned artifacts.
- [ ] Ground the exact release candidate, package contract, dependency state,
  and local stashes.
- [ ] Bump the workspace and seven local lockfile package versions to `0.31.0`.
- [ ] Cut the changelog, add curated `docs/releases/v0.31.0.md`, and update the
  README release asset example.
- [ ] Run four pinned JavaScript gates and the complete Rust release baseline.
- [ ] Build `--release --locked`, stage and inspect the package, and verify both
  binaries report `tv 0.31.0`.
- [ ] Run hygiene, workflow, stale-version, package, parity, and diff checks.
- [ ] Record release-readiness evidence and stop before owner-controlled remote
  release operations.

## Surprises & Discoveries

- Observation: the first one-minute range smoke failed on a common transport
  connection before range classification, while a later bounded comparison
  succeeded for the existing five-minute path and all one-minute scenarios.
  Evidence: the implementation plan and completion audit record this sequence
  without treating one transient connection failure as a timeframe defect.

- Observation: the failure exposed a diagnostic gap independently of the
  one-minute feature.
  Evidence: the separately reviewed `source_failure_stage` contract now
  distinguishes symbol search, request preparation, WebSocket connection,
  setup, response, protocol, heartbeat, pagination, and empty-result stages.

## Decision Log

- Decision: describe one-minute bounded date ranges as the product feature and
  Desktop-free bars source stages as an additive diagnostic improvement.
  Rationale: these are the only two promoted runtime changes in the audited
  candidate.
  Date/Author: 2026-07-28 / Codex

- Decision: retain the 5,000 returned-bar cap and the existing downstream
  non-overlapping-window workflow.
  Rationale: no reviewed workload proves that explicit bounded windows are
  insufficient, and raising the cap would be a separate scale contract.
  Date/Author: 2026-07-28 / Codex

- Decision: do not rerun live network evidence during release preparation.
  Rationale: the owner-approved bounded comparison already proved the
  production scenarios, and release preparation changes only versioned
  artifacts.
  Date/Author: 2026-07-28 / Codex

- Decision: preserve the explicit runtime-skill package allowlist.
  Rationale: plans, notes, the continuity ledger, and development-only skills
  are not runtime-user artifacts.
  Date/Author: 2026-07-28 / Codex

## Outcomes & Retrospective

Release artifacts are not prepared yet. Completion requires aligned `0.31.0`
version surfaces, curated release notes, four pinned JavaScript gates, the full
Rust baseline, locked release build, explicit package inspection, public
hygiene, and clean local state. Tag, push, workflow, and publication operations
remain owner-controlled.

## Context and Orientation

The latest public release is `v0.30.2` at commit `e8e480d`. The completed v0.31
audit is archived at
`docs/plans/archives/tradingview-cli-v0.31-pre-release-audit.md`. Its frozen
candidate contained 16 commits and 29 paths at `336d229`; the docs-only audit
record `1abf506` adds no production, Cargo, workflow, or toolchain drift.

Root `Cargo.toml` defines version `0.30.2` under `[workspace.package]`. Seven
workspace crates inherit it, and `Cargo.lock` records seven corresponding local
package versions. Release preparation may change only that root version and the
seven local `tradingview-*` lock entries. Third-party package versions and
checksums must remain unchanged.

`CHANGELOG.md` has an `Unreleased` section containing all v0.31 changes. Move
those entries under `## v0.31.0 - 2026-07-28` and retain a fresh empty
`Unreleased` section. Add `docs/releases/v0.31.0.md` as a GitHub Release body
without a redundant top-level version heading. Follow the prose-first format
of `docs/releases/v0.30.0.md`.

README currently uses `v0.30.2` in native archive examples. Update only current
download examples to `v0.31.0`; do not rewrite historical release notes or
changelog sections.

Four Node.js gates in `mise.toml`, CI, and the release workflow execute
generated production JavaScript contracts for study values, Pine open/save,
indicator insertion, and three-point drawing. Ordinary Cargo tests remain
Node-independent.

`scripts/stage-release-package-files.sh` owns the package allowlist: binary,
README, changelog, license, packaged agent guides, English and Japanese getting
started docs, and eight runtime skills under each agent root. Plans, notes,
`CONTINUITY.md`, and development-only skills must remain absent.

Preserve both local stashes. Do not apply, drop, rewrite, or package them.

## Plan of Work

First inspect the worktree, exact HEAD, `v0.30.2..HEAD` inventory, root
manifest, lockfile, workflows, package script, changelog, README, prior release
notes, and stashes. Confirm the audit record commit is docs-only and no
production, dependency, or workflow change appeared after the frozen audit.

Second change `[workspace.package].version` from `0.30.2` to `0.31.0` and use
Cargo tooling to synchronize the seven workspace package entries in
`Cargo.lock`. Inspect the release-prep diff against its grounded HEAD and reject
any third-party dependency or checksum drift.

Third move all current Unreleased entries into the dated v0.31.0 section and
leave a fresh empty Unreleased section. Write curated release notes covering
bounded one-minute date ranges, preserved range semantics, additive bars source
stages, stable downstream invocation, and deterministic validation. State that
the release does not add recovery behavior, broader timeframes, larger
single-request history, private downstream packaging, ranking,
recommendations, or trading judgment.

Fourth update current README archive examples, run all deterministic gates,
build the locked release binary, stage a clean package, and inspect its paths.
Verify source and staged binaries report `tv 0.31.0` and the package retains the
reviewed explicit runtime-skill allowlist.

Finally record aggregate evidence and stop before remote release operations.
Do not tag, push, execute workflows, or publish a GitHub Release without a
separate current-turn owner instruction.

## Concrete Steps

Run from the repository root to ground the candidate:

    git status --short --branch
    git rev-parse HEAD
    git log --oneline v0.30.2..HEAD
    git diff --name-status v0.30.2..HEAD
    git diff --quiet 336d229..HEAD -- crates Cargo.toml Cargo.lock .github mise.toml
    git stash list
    sed -n '1,80p' Cargo.toml
    sed -n '1,100p' CHANGELOG.md
    sed -n '1,160p' docs/releases/v0.30.0.md
    sed -n '1,240p' .github/workflows/release.yml
    sed -n '1,240p' scripts/stage-release-package-files.sh

After version edits:

    git diff -- Cargo.toml Cargo.lock
    cargo metadata --no-deps --format-version 1

Only `[workspace.package].version` and the seven local `tradingview-*` package
version lines may change. No third-party package or checksum may drift.

Run the four pinned JavaScript contracts:

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

The package must contain no plan, note, `CONTINUITY.md`, or development-only
skill such as `continuity`, `conventional-commits`, `discovering-skills`, or
`release-prep`.

## Validation and Acceptance

The workspace version, seven local lock entries, README current asset example,
changelog, release notes, source binary, and staged binary all state `0.31.0`.
Historical release records remain unchanged, and no third-party dependency or
checksum drifts during release prep.

Release notes accurately describe normalized `1`/`1m` bounded date ranges,
`bars.v1`, period-start timestamps, the 5,000 returned-bar cap, coverage and
truncation fields, and additive `source_failure_stage`. They must not promise
retry, automatic recovery, complete exchange-calendar knowledge, additional
timeframes, unlimited history, or guaranteed source availability.

Four pinned JavaScript gates, formatting, strict workspace Clippy, all workspace
tests and doctests, metadata, locked release build, public hygiene, package
syntax, contributor-guide parity, workflow YAML parsing, stale-version scans,
and diff hygiene are green.

The staged package uses the explicit allowlist, contains the expected runtime
skills under both roots, and excludes plans, notes, the local ledger, and
development-only skills. No live network or Desktop operation runs.

## Idempotence and Recovery

Rerun failed deterministic gates after fixing only their owning boundary. Do
not update dependencies, weaken tests, run live probes, apply or drop either
stash, or compensate with package drift. Recreate only the named staging tree.
If the dependency graph changes unexpectedly, stop and restore only that
unintended release-prep drift before continuing.

Do not tag, push, execute workflows, or publish a release. Those actions need a
separate explicit owner instruction after local readiness is complete.

## Artifacts and Notes

Keep evidence aggregate and repository-relative. Do not retain symbols, bars,
prices, date ranges, WebSocket payloads, endpoints, credentials, environment
values, account-local metadata, machine paths, or temporary package contents in
tracked files.

The owner-approved one-minute live evidence is already reviewed and is not
rerun here. Release evidence consists of deterministic tests, locked build,
version readback, and package inspection.

## Interfaces and Dependencies

This plan introduces no interface or dependency. It verifies the reviewed
`bars.v1` success payload, additive Desktop-free bars
`source_failure_stage` error detail, workspace version inheritance, locked
dependency graph, release workflows, and package allowlist.

No new crate, feature flag, command, source provider, recovery behavior, or
output envelope may be introduced during release preparation.

## Open Questions

There are no unresolved critical questions. Remote publication remains
owner-controlled.

Revision note (2026-07-28): Created the v0.31.0 release-readiness plan after
closing the audited two-slice candidate. The plan preserves the reviewed
dependency graph and package allowlist, distinguishes product and diagnostics
claims, and stops before remote release operations.
