# v0.25.0 release readiness

This ExecPlan prepares the `v0.25.0` release. It does not add features,
refactor code, update dependencies, push tags, push commits, or create a
GitHub Release.

## Purpose

`v0.25.0` ships chart-backed workflow maturity:

- `tv chart compare <SYMBOL>...` for explicit Desktop-backed selected-chart
  finalist comparison with `chart_compare.v1` restore readback;
- `tv events compare <SYMBOL>...` for Desktop-free scanner fundamentals
  earnings / dividends comparison with `events_compare.v1`;
- `tv replay log --attach-ohlcv-summary` for opt-in selected-chart OHLCV
  summary attachments in bounded Replay step logs;
- `tv screenshot --region strategy --output <PATH>` for visible Strategy
  Tester panel visual evidence.

The pre-release architecture audit found no release blocker and no larger
refactor requirement. This plan finalizes versioned release materials and
package validation.

## Progress

- [x] Create this release readiness plan.
- [x] Archive the completed `v0.25.0` pre-release architecture audit plan.
- [x] Update the plan index and `v0.25.0` roadmap to release readiness.
- [x] Bump the workspace package version to `0.25.0` and sync `Cargo.lock`.
- [x] Cut `CHANGELOG.md` `Unreleased` into `v0.25.0 - 2026-06-11`.
- [x] Add curated GitHub Release notes at `docs/releases/v0.25.0.md`.
- [x] Update README release asset examples to `v0.25.0`.
- [x] Validate release package staging and release archive contents.
- [x] Run Rust baseline, release hygiene checks, and optional CI status check.

## Release Notes Scope

The release notes cover:

- Desktop-backed chart compare as a separated selected-chart workflow;
- multi-symbol event readback for scanner-backed earnings / dividends fields;
- explicit Replay OHLCV summary attachment;
- Strategy Tester panel screenshot evidence;
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

Package staging smoke confirmed `tv`, `README.md`, `CHANGELOG.md`, `LICENSE`,
packaged `AGENTS.md` / `CLAUDE.md`, runtime skills, skill references, and the
English / Japanese getting-started docs. Development-only skills were not
staged.

## Validation

Release package validation:

- [x] `cargo metadata --no-deps --format-version 1`
- [x] `bash -n scripts/stage-release-package-files.sh`
- [x] `cargo build --release --locked`
- [x] `rm -rf target/release-package-smoke`
- [x] `scripts/stage-release-package-files.sh target/release-package-smoke target/release/tv`
- [x] `find target/release-package-smoke -maxdepth 4 -print | sort`
- [x] `target/release/tv --version` reported `tv 0.25.0`

Rust baseline:

- [x] `cargo fmt --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace`
- [x] `cargo metadata --no-deps --format-version 1`
- [x] `git diff --check`

Release hygiene:

- [x] public docs and packaged assets scan for local paths, credentials, raw target
  ids, account-local metadata, raw live payloads, raw JSONL output, raw
  WebSocket frames, raw bars, and local validation details
- [x] stale `v0.24.0` / `0.24.0` release asset example scan, allowing historical
  changelog and release-note entries
- [x] `gh run list --limit 5` for recent CI status

The broad hygiene scan reported existing policy language, historical archive
validation examples, and test fixture paths. No new raw payload, credential,
account-local identifier, target id, or local validation path was added to the
new release notes or active release-readiness plan. Recent GitHub Actions runs
listed by `gh run list --limit 5` were checked as release context only; this
local release-prep commit has not been pushed.

## Next Step

After this commit, the user may tag, push, and create the GitHub Release using
the prepared release notes. This plan does not perform those remote actions.
