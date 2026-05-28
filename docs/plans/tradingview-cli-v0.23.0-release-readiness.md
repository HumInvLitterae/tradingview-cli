# v0.23.0 release readiness

This ExecPlan prepares the `v0.23.0` release. It does not add features,
refactor code, update dependencies, push tags, push commits, or create a
GitHub Release.

## Purpose

`v0.23.0` ships explicit export and chart workflow maturity:
`tv export chart-bars`, `tv replay log --steps <N>`, chart-backed compare
contract planning, and standalone `tv events` feasibility planning. The
pre-release architecture audit found no blocker, so this plan finalizes
versioned release materials and package validation.

## Progress

- [x] Create this release readiness plan.
- [x] Archive the completed `v0.23.0` pre-release architecture audit plan.
- [x] Update the plan index and `v0.23.0` roadmap to release readiness.
- [x] Bump the workspace package version to `0.23.0` and sync `Cargo.lock`.
- [x] Cut `CHANGELOG.md` `Unreleased` into `v0.23.0 - 2026-05-28`.
- [x] Add curated GitHub Release notes at `docs/releases/v0.23.0.md`.
- [x] Update README release asset examples to `v0.23.0`.
- [x] Validate release package staging and release archive contents.
- [x] Run Rust baseline, release hygiene checks, and optional CI status check.

## Release Notes Scope

The release notes cover:

- `tv export chart-bars` and its `export_chart_bars.v1` selected-chart export
  diagnostics;
- `tv replay log --steps <N>` and its `replay_step_log.v1` JSONL events;
- chart-backed compare contract planning without adding a stable command;
- standalone `tv events` feasibility planning without adding a stable command;
- source-boundary and architecture-audit conclusions.

They intentionally do not include raw JSONL, raw bars, raw live payloads,
session ids, target ids, account-local ids, local absolute paths, secrets, or
local validation environment details.

## Packaging Scope

Release archive staging continues to include:

- `tv` or `tv.exe`;
- `README.md`, `CHANGELOG.md`, and `LICENSE`;
- user-facing `AGENTS.md` and `CLAUDE.md`;
- runtime skills under `.agents/skills/` and `.claude/skills/`;
- `docs/getting-started.md` and `docs/ja/getting-started.md`.

The staging script remains an explicit allowlist. Development-only skills and
the broader contributor docs are not copied.

## Validation

Release package validation:

- `cargo metadata --no-deps --format-version 1`
- `bash -n scripts/stage-release-package-files.sh`
- `cargo build --release --locked`
- `rm -rf target/release-package-smoke`
- `scripts/stage-release-package-files.sh target/release-package-smoke target/release/tv`
- `find target/release-package-smoke -maxdepth 4 -print | sort`
- `target/release/tv --version`

Rust baseline:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo metadata --no-deps --format-version 1`
- `git diff --check`

Release hygiene:

- public docs and packaged assets scan for local paths, credentials, raw target
  ids, account-local metadata, raw live payloads, raw JSONL output, raw
  WebSocket frames, raw bars, and local validation details
- stale `v0.22.0` / `0.22.0` release asset example scan, allowing historical
  changelog and release-note entries
- `gh run list --limit 5` for recent CI status

## Next Step

After this commit, the user may tag, push, and create the GitHub Release using
the prepared release notes. This plan does not perform those remote actions.
