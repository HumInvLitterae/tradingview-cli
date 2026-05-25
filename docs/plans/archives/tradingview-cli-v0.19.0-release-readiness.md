# `v0.19.0` release readiness

This ExecPlan follows `.agents/PLANS.md` from the repository root. It records
the release-preparation slice for `v0.19.0`.

## Purpose / Big Picture

`v0.19.0` releases daily date-range historical readback for
`tv bars <EXCHANGE:SYMBOL>`. The release makes old daily source-guided samples
reproducible through the Desktop-free `bars.v1` source instead of relying on
selected-chart viewport movement.

This slice stops feature work and prepares versioning, changelog, GitHub
Release notes, README release asset examples, packaged agent guidance, and
release archive staging. It does not create a Git tag, push, or create a
GitHub Release.

## Progress

- [x] (2026-05-21T05:05Z) Create this release-readiness ExecPlan.
- [x] (2026-05-21T05:05Z) Archive the completed `v0.19.0` pre-release audit.
- [x] (2026-05-21T05:05Z) Update version, changelog, release notes, README
  asset examples, plan index, and roadmap for `v0.19.0`.
- [x] (2026-05-21T05:30Z) Run release package validation, Rust baseline,
  release safety scans, and optional CI status check.
- [x] (2026-05-21T05:30Z) Record release-prep outcome.
- [x] (2026-05-21T05:30Z) Commit the release-prep changes.

## Surprises & Discoveries

- Observation: Release validation passed.
  Evidence: `cargo metadata --no-deps --format-version 1`,
  `bash -n scripts/stage-release-package-files.sh`, `cargo build --release
  --locked`, `cargo fmt --check`, `cargo clippy --workspace --all-targets
  --all-features -- -D warnings`, `cargo test --workspace`, and
  `git diff --check` passed.

- Observation: The staged release package contains the expected runtime
  contents and excludes development-only skills.
  Evidence: The staged tree contains `tv`, `README.md`, `CHANGELOG.md`,
  `LICENSE`, user-facing `AGENTS.md`, user-facing `CLAUDE.md`, and runtime
  skills. `release-prep`, `continuity`, `conventional-commits`, and
  `discovering-skills` were absent.

- Observation: Recent remote CI status was clean.
  Evidence: `gh run list --limit 5` showed the latest listed `CI` and
  `Release` runs completed successfully.

## Decision Log

- Decision: Keep `v0.19.0` release prep limited to release artifacts and
  validation.
  Rationale: The pre-release audit found no blocker, and additional feature
  work such as intraday/weekly/monthly range reads, large-range batching, and
  `tv range` plus `tv ohlcv` export are deferred.
  Date/Author: 2026-05-21 / Codex.

## Outcomes & Retrospective

Release prep is complete. `target/release/tv --version` reports `tv 0.19.0`,
the package staging smoke passed, and release notes are ready for the user to
use when creating the GitHub Release. No tag, push, or GitHub Release was
created.

## Plan of Work

Update the workspace version to `0.19.0`, sync `Cargo.lock`, cut the
`CHANGELOG.md` `Unreleased` content into `v0.19.0 - 2026-05-21`, and add
`docs/releases/v0.19.0.md` without a top-level version heading. Update README
release asset examples from `v0.18.0` to `v0.19.0`.

Confirm packaged runtime guidance remains aligned with daily date-range
`tv bars`, `bars.v1`, inclusive `--to`, source availability, and the boundary
between Desktop-free `tv bars`, selected-chart `tv ohlcv`, and viewport-only
`tv range`.

Run release package validation, Rust baseline, release hygiene scans, and an
optional read-only CI status check. If all checks pass, commit with
`chore(release): Prepare v0.19.0`.

## Validation and Acceptance

Acceptance is met when:

- `target/release/tv --version` reports `tv 0.19.0`;
- the staged release package contains the binary, README, changelog, license,
  user-facing `AGENTS.md`, user-facing `CLAUDE.md`, and runtime skills;
- development-only skills remain excluded from the staged release package;
- Rust baseline and release package validation pass;
- public docs and packaged assets contain no raw bars, raw live payload, raw
  WebSocket frame, raw JSONL output, target id, account-local metadata,
  credential, or local absolute path introduced by this slice.

## Interfaces and Dependencies

This release prep does not change public interfaces. The primary released
interface remains:

    tv bars <EXCHANGE:SYMBOL> --timeframe 1D --from YYYY-MM-DD --to YYYY-MM-DD

No new command, option, source, dependency update, CI workflow change, runtime
skill, Git tag, push, or GitHub Release is planned in this slice.
