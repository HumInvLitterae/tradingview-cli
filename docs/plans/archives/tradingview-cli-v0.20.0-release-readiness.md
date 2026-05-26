# v0.20.0 release readiness

This ExecPlan is a living document. The sections `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md` from the repository root. Keep this
document self-contained so a contributor can finish the work from this file
alone.

## Purpose / Big Picture

Prepare the `v0.20.0` release state after weekly/monthly `tv bars`
date-range readback, `range_alignment`, and user-facing getting-started docs
are complete. This slice stops feature work and aligns version, changelog,
GitHub Release notes, release package contents, and validation evidence.

After this change, the repository is ready for the user to create the tag,
push it, and publish the GitHub Release. This plan does not create the tag,
push to a remote, or create the GitHub Release.

## Progress

- [x] (2026-05-26T00:00Z) Archive the completed user getting-started docs
  plan and make this release-readiness plan current.
- [x] (2026-05-26T00:05Z) Update Cargo workspace version to `0.20.0`, sync
  `Cargo.lock`, cut the changelog section, and add curated release notes.
- [x] (2026-05-26T00:35Z) Run release package validation, Rust baseline, diff check, and public
  hygiene scans.
- [ ] Commit release preparation as `chore(release): Prepare v0.20.0`.

## Surprises & Discoveries

- Observation: The dependency maintenance commit immediately before this
  slice already updated `reqwest` to `0.13.4`.
  Evidence: recent history includes `build(deps): Update dependencies`, and
  `Cargo.toml` / `Cargo.lock` already reference `reqwest 0.13.4`.

## Decision Log

- Decision: Keep this slice limited to release preparation.
  Rationale: The `v0.20.0` feature surface is complete: daily/weekly/monthly
  date-range `tv bars`, range-alignment readback, and user-facing setup docs.
  Adding intraday ranges, batching, export helpers, or source mixing here
  would blur the release boundary.
  Date/Author: 2026-05-26 / Codex.

- Decision: Do not record the broader remaining-task inventory in tracked
  docs as part of this release readiness.
  Rationale: The user explicitly asked not to preserve that conversational
  explanation in repository documentation.
  Date/Author: 2026-05-26 / Codex.

- Decision: Respect the current dependency-update commit and avoid additional
  dependency changes.
  Rationale: Release prep should not mix feature, CI, or dependency work
  unless required for the release state.
  Date/Author: 2026-05-26 / Codex.

## Outcomes & Retrospective

Release readiness is complete. The workspace version and lockfile now report
`0.20.0`, the changelog has a dated `v0.20.0` section, and
`docs/releases/v0.20.0.md` provides curated GitHub Release notes without a
top-level tag heading.

Validation passed:

- `cargo metadata --no-deps --format-version 1`
- `bash -n scripts/stage-release-package-files.sh`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo build --release --locked`
- release package smoke staging with the two getting-started docs included
- `target/release/tv --version` returned `tv 0.20.0`
- `git diff --check`
- old-version grep, with only historical `CHANGELOG.md` entries remaining
- public hygiene grep, with existing policy / archive / test-example matches
  and no newly introduced private data in the changed release files

## Context and Orientation

`v0.20.0` focuses on historical bars range maturity and first-run usability:

- `tv bars --from --to --timeframe 1D|1W|1M`;
- additive `range_alignment` for period-start timestamp semantics;
- `range_coverage_status` as the primary date-range coverage readback;
- English and Japanese getting-started docs staged into release archives.

The source boundary remains unchanged. `tv bars` is the Desktop-free
historical bars source. `tv range`, `tv ohlcv`, Replay, observe/stream,
scanner quote, chart quote, and quote-data are not hidden fallbacks for this
release.

## Plan of Work

Archive the completed user getting-started docs plan. Update the current-plan
index and `docs/v0.20-roadmap.md` so they point at release readiness.

Set the workspace version to `0.20.0`, sync `Cargo.lock`, cut
`CHANGELOG.md` from `Unreleased` into `v0.20.0 - 2026-05-26`, and add
`docs/releases/v0.20.0.md`. The release note must be user-facing and must not
include raw bars, raw WebSocket frames, raw JSONL output, target ids,
account-local identifiers, local absolute paths, or private validation
details.

Validate that the release package still contains the binary, README,
CHANGELOG, LICENSE, user-facing agent guides, runtime skills, and only the two
getting-started docs from `docs/`.

## Concrete Steps

From the repository root:

    cargo metadata --no-deps --format-version 1
    bash -n scripts/stage-release-package-files.sh
    cargo build --release --locked
    rm -rf target/release-package-smoke
    scripts/stage-release-package-files.sh target/release-package-smoke target/release/tv
    find target/release-package-smoke -maxdepth 4 -print | sort
    target/release/tv --version

Run the Rust baseline:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Run release safety scans:

    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|raw JSONL|raw bars|account-local|target id|downstream-private)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true
    rg -n "v0\\.19\\.0|0\\.19\\.0" README.md docs/releases/v0.20.0.md docs/getting-started.md docs/ja/getting-started.md packaging/agent/AGENTS.md CHANGELOG.md

The old-version grep may find historical `CHANGELOG.md` entries only. It
must not find stale current asset examples or user setup instructions.

Optionally inspect recent CI state:

    gh run list --limit 5

If CI shows a failure, record it separately and do not mix a CI fix into this
release-prep commit without a separate plan.

## Validation and Acceptance

Acceptance is met when:

- `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, and `docs/releases/v0.20.0.md`
  reflect `v0.20.0`.
- `target/release/tv --version` reports `tv 0.20.0`.
- The staged release package includes the binary, README, changelog, license,
  user-facing agent guides, runtime skills, and
  `docs/getting-started.md` / `docs/ja/getting-started.md`.
- Development-only skills and broad repository docs are not staged.
- README and packaged user guides remain consistent with daily/weekly/monthly
  date-range bars, `range_alignment`, and agent-assisted first-run guidance.
- Rust baseline, packaging checks, diff check, and public hygiene scans pass.
- No tag, push, or GitHub Release is created by this slice.

## Idempotence and Recovery

The release package smoke removes and recreates `target/release-package-smoke`,
so it is safe to repeat. If `cargo build --release --locked` fails after the
version bump, inspect whether `Cargo.lock` needs to be regenerated by
`cargo metadata --no-deps --format-version 1`.

If this slice needs to be reverted, restore the workspace version and
`Cargo.lock` to `0.19.0`, move this plan out of current status, remove
`docs/releases/v0.20.0.md`, and restore the `Unreleased` changelog entries.

## Artifacts and Notes

Do not paste raw live output, raw bars, raw WebSocket frames, raw JSONL output,
target ids, account-local identifiers, credentials, or local absolute paths
into this plan or the release notes.
