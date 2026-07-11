# Remove machine-specific usernames from tracked documentation

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while working. Maintain
this document in accordance with `.agents/PLANS.md`.

## Purpose / Big Picture

This slice removes a real local username from tracked repository documents
before v0.26 release readiness. Historical validation commands attempted to
detect private paths with an alternation containing both a specific username
and the generic `/Users/` prefix. The specific branch was redundant and itself
violated the documentation policy it was intended to enforce.

After this work, tracked files contain no occurrence of that machine-specific
username. The validation commands retain the generic `/Users/` detector, so
their ability to find absolute macOS user paths is unchanged.

## Progress

- [x] (2026-07-11) Stopped release-readiness progression after the policy contradiction was reported.
- [x] (2026-07-11) Searched all tracked files for macOS, Linux, and Windows user-path forms.
- [x] (2026-07-11) Confirmed 79 occurrences across 61 files, all in the same redundant validation-regex alternation.
- [x] (2026-07-11) Replaced each username-specific branch with the existing generic `/Users/` detector.
- [x] (2026-07-11) Confirmed no other use of the username existed in tracked files.
- [x] (2026-07-11) Kept synthetic `/Users/example` Rust fixtures unchanged because they are explicit portable test placeholders.
- [x] (2026-07-11) Made this cleanup the current plan in the plan index, roadmap, work inventory, and continuity ledger.
- [x] (2026-07-11) Recorded the cleanup in the Unreleased changelog.
- [x] (2026-07-11) Verified by byte comparison that all 61 files match the base content after exactly one mechanical substitution.
- [x] (2026-07-11) Verified zero tracked or untracked documentation occurrences of the machine-specific username.
- [x] (2026-07-11) Verified the mechanical diff contains 79 additions and 79 deletions and preserves generic `/Users/` detection.
- [x] (2026-07-11) Ran documentation whitespace, packaging-script, and contributor-guide checks.
- [x] (2026-07-11) Confirmed Rust, Cargo manifests, and Cargo lockfile have no diff from audit commit `51600f8`.
- [x] (2026-07-11) Created a focused read-only review prompt.
- [x] (2026-07-11) Recorded outcomes as `implemented and validated; independent review pending` and stopped uncommitted.
- [x] (2026-07-11) Independent review reproduced the 61-file byte comparison and reported no findings.

## Surprises & Discoveries

- Observation: the issue was broader than the three executable validator paths
  corrected during the first audit re-review.
  Evidence: 79 tracked occurrences remained across 61 files, including
  `docs/development.md` and 60 archived plans.

- Observation: all 79 occurrences had one mechanically identical role.
  Evidence: every match was the redundant `/Users/<local-user>|/Users/`
  alternation inside a hygiene regex; no prose, executable file path, source
  code, or payload contained the local username.

## Decision Log

- Decision: remove the username-specific alternation everywhere rather than
  narrow the audit acceptance condition.
  Rationale: the generic `/Users/` branch already detects every path matched by
  the username-specific branch, so removal improves privacy and clarity with no
  loss of validation behavior.
  Date/Author: 2026-07-11 / Codex

- Decision: preserve `/Users/example` in two Rust fixtures.
  Rationale: those values are deliberate non-user-specific placeholders used
  to test sanitization of TradingView application URLs; changing them does not
  improve privacy and would mix production-test edits into a docs-only slice.
  Date/Author: 2026-07-11 / Codex

## Outcomes & Retrospective

Implementation and local validation are complete. The cleanup removed 79
redundant username-specific regex branches across 61 files. A byte-for-byte
comparison against `51600f8` after applying the intended substitution reported
`checked=61 failures=0`; generic `/Users/` detection remains in place.

Repository-wide username and concrete-username-alternation scans return no
matches. Documentation whitespace, packaging-script syntax, contributor-guide
identity, and the no-Rust/Cargo-diff check are green.

Independent review reproduced the inventory, 79-addition / 79-deletion diff,
byte comparison, privacy scans, and no-Rust/Cargo-diff check and reported no
findings. The current-tree cleanup is complete and ready to commit. Historical
Git refs still require a separate sanitation decision and plan before release
readiness.

## Context and Orientation

The v0.26 pre-release completion and architecture audit was committed as
`51600f8`. It concluded that no production refactor is required before release
readiness. A follow-up observation found that many historical hygiene commands
still embedded the actual local username in patterns such as
`/Users/<local-user>|/Users/`. Because the second branch subsumes the first,
these commands can use `/Users/` alone.

This cleanup edits documentation only. It does not change Rust code, CLI
behavior, release packaging, runtime skills, dependencies, or version.

## Plan of Work

Use `git grep` to inventory the actual username and nearby user-path forms.
Confirm each match is the same regex alternation. Apply one mechanical
replacement from the username-specific plus generic alternation to the generic
branch alone.

Create this ExecPlan and update current planning sources so release readiness
cannot begin before cleanup review. Then verify zero username matches, inspect
the complete diff for mechanical consistency, and create a focused reviewer
handoff. Do not run the Rust workspace suite because no Rust or Cargo file is
changed; prove that fact directly with Git.

## Concrete Steps

Run from the repository root:

    git grep -nE '/Users/[[:alnum:]_.-]+\|/Users/' -- .
    git diff --check
    git diff --quiet 51600f8 -- crates Cargo.toml Cargo.lock
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md

The concrete-username alternation search must return no matches. Review all
changed validation lines and confirm they still contain `/Users/` in their
private-path detector.

## Validation and Acceptance

Acceptance requires zero tracked occurrences of the machine-specific username,
no Rust or Cargo diff from `51600f8`, no whitespace errors, unchanged packaging
script syntax, and identical contributor guides. The changed lines must differ
only by removal of the redundant username-specific regex branch, apart from
the new planning, changelog, continuity, and review-handoff documents.

Synthetic `/Users/example` test fixtures and generic `/Users/` policy examples
are allowed because they contain no real local username.

## Idempotence and Recovery

The replacement is idempotent because the removed alternation no longer
matches after the first pass. If validation finds a non-regex use, stop and
inspect it individually instead of applying another broad substitution. Do not
reset unrelated user changes. Do not commit until independent review is green.

## Artifacts and Notes

The completed v0.26 audit plan is archived at
`docs/plans/archives/tradingview-cli-v0.26-pre-release-audit.md`. The focused
review prompt will live at
`docs/notes/v0.26-machine-specific-path-cleanup-review-prompt.md`.

## Interfaces and Dependencies

No public interface, command, payload, Rust API, dependency, timeout, retry,
source boundary, or package version changes in this slice.

## Open Questions

None for the current tree. The next slice is canonical Git-history sanitation;
v0.26 release readiness remains paused until that destructive migration is
separately approved and completed.

Revision note (2026-07-11): created after the committed pre-release audit when
a broader repository-wide username scan exposed redundant historical regex
branches that had not been included in the first focused review.

Revision note (2026-07-11): completed the 61-file mechanical cleanup and local
validation, added a focused reviewer handoff, and left the work uncommitted
pending independent review.

Revision note (2026-07-11): independent review reported no findings. The
current-tree cleanup is approved for commit; historical refs remain a separate
pre-release concern.
