# v0.4.0 release readiness plan

This ExecPlan is a living document. Keep `Progress`, `Discoveries`, `Decisions`,
and `Validation` current while preparing the release.

## Purpose

Prepare `v0.4.0` for release without adding new features or broad
refactoring. The release packages the post-`v0.3.0` market-data read lane:
extended-hours scanner reads, scanner metainfo, ordered batch quotes, explicit
quote source selection, typed market/scanner Rust APIs, and runtime skills for
market data interpretation.

Do not tag, push, or create a GitHub Release in this plan.

## Progress

- [x] Archived the completed runtime market-analysis skills plan.
- [x] Bump the workspace package version to `0.4.0` and refresh `Cargo.lock`.
- [x] Cut `CHANGELOG.md` `Unreleased` into `v0.4.0 - 2026-04-30`.
- [x] Add curated GitHub Release notes at `docs/releases/v0.4.0.md`.
- [x] Refresh README release asset examples and release archive wording.
- [x] Update roadmap and plan index to mark v0.4.0 release readiness.
- [x] Validate release package staging, Rust baseline, and public-doc hygiene.
- [x] Commit release readiness changes.

## Scope

In scope:

- Cargo workspace package version and lockfile synchronization.
- `CHANGELOG.md` and `docs/releases/v0.4.0.md`.
- README release archive examples and user-facing release packaging wording.
- `docs/v0.4-roadmap.md`, `docs/plans/README.md`, and release packaging docs if
  they need small alignment edits.
- Release package staging smoke against a local release binary.

Out of scope:

- New CLI commands or options.
- Further market-data implementation work.
- More skill creation.
- Git tag creation, pushing, or GitHub Release creation.

## Concrete Steps

1. Update version metadata.
   - Change the workspace package version from `0.3.0` to `0.4.0`.
   - Refresh `Cargo.lock` through Cargo so local workspace package entries
     match.

2. Prepare release notes.
   - Move the current `CHANGELOG.md` `Unreleased` bullets under
     `v0.4.0 - 2026-04-30`.
   - Leave an empty `Unreleased` section for future work.
   - Add `docs/releases/v0.4.0.md` without a top-level version heading.

3. Align release-facing docs.
   - Update README asset examples from `v0.3.0` to `v0.4.0`.
   - Make sure README mentions the runtime market-data interpretation skills in
     the archive description.
   - Mark the v0.4 market-data lane as release-ready in
     `docs/v0.4-roadmap.md`.
   - Update `docs/plans/README.md` so this plan is current and the runtime
     skill plan is archived.

4. Validate.
   - Run release package staging with `target/release/tv`.
   - Run Rust baseline and doc hygiene checks.
   - Inspect staged runtime skills to ensure development-only skills are not
     included.

## Validation

Completed commands:

```bash
cargo metadata --no-deps --format-version 1
cargo build --release --locked
bash -n scripts/stage-release-package-files.sh
rm -rf target/release-package-smoke
scripts/stage-release-package-files.sh target/release-package-smoke target/release/tv
find target/release-package-smoke -maxdepth 4 -print | sort
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
git diff --check
rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true
```

Results:

- `cargo metadata --no-deps --format-version 1`: passed.
- `cargo build --release --locked`: passed with workspace packages at `0.4.0`.
- Release package staging: passed. The staged package includes `tv`, README,
  changelog, license, user-facing agent guides, and runtime skills under both
  `.agents/skills` and `.claude/skills`.
- `cargo fmt --check`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`:
  passed.
- `cargo test --workspace`: passed.
- `git diff --check`: passed.
- Public-doc hygiene grep reported only policy text, archived validation
  commands, and secret-safety wording; no new machine-specific path, account
  identifier, cookie, token, authorization value, or raw live payload was added.

## Risks

- Release notes may accidentally become too implementation-heavy. Keep
  `docs/releases/v0.4.0.md` user-facing and concise.
- Runtime skill packaging can regress if broad copies are introduced. Keep the
  explicit allowlist and verify staged contents.
- Hygiene scans can match intentional safety wording. Treat matches as review
  prompts rather than automatic failures.

## Rollback

Before commit, revert the version, changelog, release note, README, roadmap,
plan index, and plan archive move. No remote state is changed by this plan.
