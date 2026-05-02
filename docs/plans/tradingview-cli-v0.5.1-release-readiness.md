# v0.5.1 release readiness

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is self-contained so a new contributor can finish the `v0.5.1` release preparation without prior chat context.

## Purpose / Big Picture

Prepare `v0.5.1` as a focused patch release after `v0.5.0`. The release contains chart-source quote correctness hardening and an opt-in live endurance smoke for maintainers. It must not add new features, refactors, packaging policy changes, tags, pushes, or GitHub Releases.

After this change, the repository has versioned release notes, changelog, README asset examples, package staging evidence, and a release-prep commit ready for the user to tag and publish.

## Progress

- [x] (2026-05-02) Archived the completed chart quote live smoke ExecPlan.
- [x] (2026-05-02) Added this release readiness ExecPlan.
- [x] (2026-05-02) Bumped workspace package version and synchronized `Cargo.lock`.
- [x] (2026-05-02) Cut `CHANGELOG.md` `Unreleased` entries into `v0.5.1 - 2026-05-02`.
- [x] (2026-05-02) Added `docs/releases/v0.5.1.md`.
- [x] (2026-05-02) Updated README release asset examples and roadmap references.
- [x] (2026-05-02) Ran release package validation and Rust baseline.
- [x] (2026-05-02) Checked recent CI status with `gh run list --limit 5`.
- [x] (2026-05-02) Commit the release prep changes.

## Surprises & Discoveries

- Observation: the broad hygiene grep continues to report existing policy text,
  archived validation-command examples, and secret-safety wording.
  Evidence: the scan reported no new raw live payload, target id, account-local
  value, cookie, token, authorization value, or local machine path in the
  release notes or changed public docs.

- Observation: the release package staging allowlist still copies only runtime
  skills.
  Evidence: staged package contained `chart-analysis`, `market-data-interpretation`,
  `multi-symbol-scan`, `pine-develop`, `replay-practice`,
  `screener-result-analysis`, `screener-workflow`, and `strategy-report` under
  both `.agents/skills` and `.claude/skills`; development-only skills were not
  staged.

## Decision Log

- Decision: Treat this as a patch release, not a `v0.6.0` milestone.
  Rationale: the included changes fix and verify chart-source quote readiness after a reported mismatch, while the broader source taxonomy and observation roadmap remain future-facing docs.
  Date/Author: 2026-05-02 / Codex

- Decision: Do not add or remove runtime skills for this patch.
  Rationale: release package contents already match the current runtime skill policy; this release only needs versioned docs and validation.
  Date/Author: 2026-05-02 / Codex

## Outcomes & Retrospective

Prepared `v0.5.1` as a focused patch release. Workspace package version and
lockfile now report `0.5.1`, `CHANGELOG.md` has a dated `v0.5.1` section, and
`docs/releases/v0.5.1.md` is ready for the GitHub Release body without a
top-level version heading. README release asset examples now point to
`v0.5.1`.

Release package staging and the Rust baseline passed. Recent GitHub Actions
runs also showed success. No tag, push, or GitHub Release was created.

## Context and Orientation

Current release-relevant commits after `v0.5.0`:

- chart-source quote stable readiness hardening;
- v0.6 roadmap and command source taxonomy docs;
- runtime skill maintenance;
- opt-in ignored Rust live smoke for chart-source quote endurance.

The release notes should keep the user-facing emphasis on the chart-source quote fix. The taxonomy and smoke are useful supporting docs/tooling, but this patch release should not be framed as a new feature milestone.

Relevant files:

- `Cargo.toml`
- `Cargo.lock`
- `CHANGELOG.md`
- `README.md`
- `docs/releases/v0.5.1.md`
- `docs/v0.6-roadmap.md`
- `docs/plans/README.md`
- `scripts/stage-release-package-files.sh`

## Plan of Work

Bump `[workspace.package].version` from `0.5.0` to `0.5.1` and synchronize the lockfile. Cut the current changelog `Unreleased` entries into a dated `v0.5.1` section, leaving an empty `Unreleased` placeholder for future work. Add a concise GitHub Release body under `docs/releases/v0.5.1.md` without a top-level version heading.

Update README release asset examples from `v0.5.0` to `v0.5.1` and make sure the validation section still points to the opt-in chart-source quote endurance smoke. Update `docs/v0.6-roadmap.md` so Lane 0 records that the patch candidate has reached release readiness. Update `docs/plans/README.md` so this release readiness plan is current and the live smoke plan is archived.

Do not change CLI behavior, JSON payloads, packaging allowlists, CI workflows, or release automation unless validation proves a release-blocking mismatch.

## Validation and Acceptance

Run release package validation:

    cargo metadata --no-deps --format-version 1
    bash -n scripts/stage-release-package-files.sh
    cargo build --release --locked
    rm -rf target/release-package-smoke
    scripts/stage-release-package-files.sh target/release-package-smoke target/release/tv
    find target/release-package-smoke -maxdepth 4 -print | sort

Run the Rust baseline:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Run release hygiene:

    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true

Optional final check:

    gh run list --limit 5

Acceptance is met when the version is `0.5.1`, release notes are present, package staging succeeds, baseline validation passes, tracked docs contain no live target ids or secret material, and the work is committed without tagging or pushing.

## Idempotence and Recovery

Release prep is safe to rerun. If package staging fails, fix only the release-prep mismatch and rerun the staging commands. If Rust baseline fails because of unrelated code, stop and split that fix into a separate plan or commit instead of hiding it inside release prep.

## Interfaces and Dependencies

No new CLI surface, Rust API, or dependency changes. The release archive allowlist remains explicit and excludes development-only skills.

## Open Questions

None.
