# v0.4.1 release readiness

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds. This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

This patch release makes the chart-backed quote hardening available as `v0.4.1`. After the release prep is complete, a user who installs `v0.4.1` gets the `tv quote <SYMBOL> --source chart` readiness guard that waits for chart bars to reflect the requested symbol, retries once on readiness timeout, and fails instead of returning stale previous-symbol data. This work does not add new features beyond release documentation and package version updates.

## Progress

- [x] (2026-05-01) Created this `v0.4.1` release readiness plan and archived the completed quote chart readiness plan.
- [x] (2026-05-01) Update workspace package version and lockfile to `0.4.1`.
- [x] (2026-05-01) Cut the changelog `Unreleased` fix into a dated `v0.4.1` section.
- [x] (2026-05-01) Add curated GitHub Release notes for `v0.4.1`.
- [x] (2026-05-01) Refresh README release asset examples and release planning docs for the patch release.
- [x] (2026-05-01) Run release packaging, Rust baseline, and hygiene validation.
- [x] (2026-05-01) Commit the release-prep changes without tagging or pushing.

## Surprises & Discoveries

- Observation: `cargo update -w` was required after changing the workspace
  package version; `cargo metadata` alone did not rewrite the lockfile package
  versions.
  Evidence: `cargo update -w` reported the seven internal `tradingview-*`
  packages moving from `0.4.0` to `0.4.1`.

## Decision Log

- Decision: Treat the chart quote readiness guard as a patch release rather than waiting for a `v0.5.0` roadmap.
  Rationale: The change fixes a stale-data correctness issue in an existing command and should reach downstream users without being bundled into a larger feature phase.
  Date/Author: 2026-05-01 / Codex

## Outcomes & Retrospective

The release-prep edits are complete and validated. The workspace package
version and lockfile now report `0.4.1`, `CHANGELOG.md` and
`docs/releases/v0.4.1.md` describe the chart-source quote readiness fix, README
release asset examples point at `v0.4.1`, and release package staging includes
the expected runtime skills while excluding development-only skills. No tag,
push, or GitHub Release was created.

## Context and Orientation

The repository is a virtual Cargo workspace. The shared package version is stored in the root `Cargo.toml` under `[workspace.package]`, and the `tradingview-cli` package in `crates/cli/Cargo.toml` inherits it with `version.workspace = true`. `Cargo.lock` records each internal path package version and must be synchronized after the version bump.

Public release notes live in two places. `CHANGELOG.md` is the long-running project changelog. `docs/releases/<tag>.md` is the curated GitHub Release body consumed by the release workflow. Release body files should not start with a top-level version heading because GitHub Release titles already contain the tag.

Release archive contents are staged by `scripts/stage-release-package-files.sh`. That script copies the binary, README, changelog, license, packaged agent guide, and a fixed allowlist of runtime skills. Development-only skills such as `release-prep` must remain excluded.

The fix being released was committed before this plan. It hardens `tv quote <SYMBOL> --source chart` and `--source auto` by avoiding success on stale chart bars after a symbol switch.

## Plan of Work

First, archive `docs/plans/tradingview-cli-quote-chart-readiness.md` and make this release plan the active patch-release plan in `docs/plans/README.md`.

Next, update the root workspace version from `0.4.0` to `0.4.1` and run Cargo metadata or an equivalent Cargo command to synchronize `Cargo.lock`. Because all internal packages inherit `workspace.package.version`, the internal path package versions in the lockfile should move together.

Then, move the current `CHANGELOG.md` `Unreleased` fix into a new `## v0.4.1 - 2026-05-01` section, leaving an empty `Unreleased` heading for future work. Add `docs/releases/v0.4.1.md` with concise user-facing notes focused on the chart quote stale-data fix, behavior notes, packaging notes, and non-goals.

Finally, update README release asset examples from `v0.4.0` to `v0.4.1`, make any minimal roadmap note needed to say the patch release is prepared, run release packaging validation, run the Rust baseline, run hygiene scans, and commit the release-prep changes. Do not create a tag, push, or create a GitHub Release.

## Concrete Steps

Run commands from the repository root.

1. Edit release files:
   - `Cargo.toml`
   - `Cargo.lock`
   - `CHANGELOG.md`
   - `README.md`
   - `docs/releases/v0.4.1.md`
   - `docs/v0.4-roadmap.md`
   - `docs/plans/README.md`
   - this plan

2. Validate packaging and Rust baseline:
   - `bash -n scripts/stage-release-package-files.sh`
   - `cargo build --release --locked`
   - `rm -rf target/release-package-smoke`
   - `scripts/stage-release-package-files.sh target/release-package-smoke target/release/tv`
   - `find target/release-package-smoke -maxdepth 4 -print | sort`
   - `cargo fmt --check`
   - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
   - `cargo test --workspace`
   - `cargo metadata --no-deps --format-version 1`
   - `git diff --check`

3. Run public-release hygiene scan:
   - `rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true`

4. Commit with:
   - `chore(release): Prepare v0.4.1`

## Validation and Acceptance

The release prep is accepted when `cargo metadata` shows the workspace packages at version `0.4.1`, `CHANGELOG.md` has a dated `v0.4.1` section, `docs/releases/v0.4.1.md` exists without a top-level version heading, release-package staging includes runtime skills and excludes development-only skills, and all validation commands listed above pass.

The manual release steps remain outside this plan: tag creation, pushing the tag, and creating or publishing the GitHub Release are not performed by Codex unless the user explicitly asks.

## Idempotence and Recovery

All edits are additive or version-note updates. If validation fails, fix the smallest relevant file and rerun the failed command. If package staging creates `target/release-package-smoke`, it can be safely deleted and recreated. If the version bump needs to be abandoned, revert this plan, release notes, changelog section, README asset examples, and the `0.4.1` version changes.

## Artifacts and Notes

Do not paste live TradingView payloads, target IDs, account-local values, local absolute paths, cookies, tokens, or raw credentials into this plan or release notes.

Validation completed:

- `bash -n scripts/stage-release-package-files.sh`
- `cargo build --release --locked`
- `scripts/stage-release-package-files.sh target/release-package-smoke target/release/tv`
- `find target/release-package-smoke -maxdepth 4 -print | sort`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo metadata --no-deps --format-version 1`
- `git diff --check`
- tracked-doc hygiene scan for local paths and secret-like strings

The staging tree contained the expected runtime skills:
`chart-analysis`, `market-data-interpretation`, `multi-symbol-scan`,
`pine-develop`, `replay-practice`, `screener-result-analysis`,
`screener-workflow`, and `strategy-report`.

## Interfaces and Dependencies

No command interface changes are introduced by this plan. The existing release workflow reads `docs/releases/v0.4.1.md` if the user later pushes tag `v0.4.1`. The release archive staging script remains the package-content source of truth.

## Open Questions

None.
