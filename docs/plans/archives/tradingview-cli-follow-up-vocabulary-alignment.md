# Follow-up vocabulary alignment

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is
self-contained and describes the `v0.13.0` contract-hardening slice that can
move while Desktop quote-session postmarket and premarket evidence is still
waiting for the right market phase.

## Purpose / Big Picture

`tv compare <SYMBOL>...` and `tv snapshot <SYMBOL>` now return machine-readable
follow-up and missing-evidence readback. This slice fixes the shared vocabulary
as a stable contract so agents can route follow-up work without guessing or
renaming values.

After this change, docs, runtime skills, and focused tests agree that the
stable follow-up kinds are `snapshot`, `chart_quote`, `observe_chart`, and
`screenshot`. These values are evidence-surface hints, not ranking, scoring,
recommendation, or automatic execution.

## Progress

- [x] (2026-05-09T00:00Z) Created this ExecPlan and archived the completed
  snapshot contract metadata plan.
- [x] (2026-05-09T00:00Z) Added focused market crate tests that lock stable
  follow-up kind values and reject the unshipped `quote_chart` alias.
- [x] (2026-05-09T00:00Z) Updated docs, runtime skills, roadmap, and
  changelog.
- [x] (2026-05-09T00:00Z) Ran validation for tests, formatting, clippy,
  workspace tests, metadata, diff hygiene, packaging script syntax, skill
  validation, and public-safe hygiene.

## Surprises & Discoveries

- Observation: `compare` and `snapshot` intentionally differ in hint shape.
  `compare.items[].follow_up_hints[]` does not include `requires_desktop`,
  while `snapshot.follow_up_hints[]` does.
  Evidence: the typed structs in `crates/market/src/types.rs` already expose
  those command-specific shapes.

## Decision Log

- Decision: Do not add a `quote_chart` alias.
  Rationale: `chart_quote` has already shipped as the stable value in compare
  and snapshot readback. Adding an alias would create two names for the same
  source and make downstream joins less deterministic.
  Date/Author: 2026-05-09 / Codex.

- Decision: Keep this slice to contract hardening only.
  Rationale: The goal is stable interpretation of existing metadata. New JSON
  fields, command options, or data sources would broaden the contract instead
  of stabilizing it.
  Date/Author: 2026-05-09 / Codex.

## Outcomes & Retrospective

Completed. The stable follow-up vocabulary is now documented and tested across
compare and snapshot:

- `snapshot`: one-symbol Desktop-free detail or retry surface;
- `chart_quote`: selected-chart single-symbol chart-feed quote follow-up, not
  scanner extended-hours evidence;
- `observe_chart`: selected-chart time-window observation;
- `screenshot`: visual evidence for visible state that structured reads cannot
  explain.

Focused tests now assert the exact shipped values and explicitly reject the
unshipped `quote_chart` alias. Runtime skills and stable docs use the same
meaning for `compare.items[].follow_up_hints[]`,
`compare.items[].missing_evidence[].suggested_follow_up`,
`snapshot.follow_up_hints[]`, and
`snapshot.missing_evidence[].suggested_follow_up`.

Validation passed:

    cargo test -p tradingview-market compare -- --nocapture
    cargo test -p tradingview-market snapshot -- --nocapture
    cargo test -p tradingview-cli --test live_compare
    cargo test -p tradingview-cli --test live_snapshot
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

Runtime skill validation also passed for `market-data-interpretation`,
`multi-symbol-scan`, `screener-result-analysis`, and `chart-analysis`. The
public-safe hygiene scan only reported existing policy/archive references and
validation-command examples, not newly introduced raw live payloads, target ids,
account-local metadata, credentials, or downstream-private paths.

## Context and Orientation

`compare` follow-up readback lives in `crates/market/src/compare.rs`.
`snapshot` follow-up readback lives in `crates/market/src/snapshot.rs`.
Both commands are Desktop-free market reads; the follow-up hints can point to
Desktop-backed surfaces, but neither command executes those surfaces.

The Desktop quote-session live-evidence plan remains active and waiting for a
real postmarket or premarket phase. This slice must not advance that evidence
decision.

## Plan of Work

Add focused tests that assert:

- compare follow-up hints use exactly `snapshot`, `observe_chart`,
  `chart_quote`, and `screenshot`;
- compare missing-evidence suggested follow-ups use only `snapshot` or
  `chart_quote`;
- snapshot follow-up hints use exactly `chart_quote`, `observe_chart`, and
  `screenshot`;
- snapshot missing-evidence suggested follow-ups use only `snapshot` or
  `chart_quote`.

Update docs and runtime skills with a small shared vocabulary table. The table
must state that `chart_quote` is not scanner extended-hours evidence and that
follow-up hints are not recommendations or automatic reads.

## Concrete Steps

From the repository root, update the market crate tests and docs, then run:

    cargo test -p tradingview-market compare -- --nocapture
    cargo test -p tradingview-market snapshot -- --nocapture

Then run the full validation listed below.

## Validation and Acceptance

Run:

    cargo test -p tradingview-market compare -- --nocapture
    cargo test -p tradingview-market snapshot -- --nocapture
    cargo test -p tradingview-cli --test live_compare
    cargo test -p tradingview-cli --test live_snapshot
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

Acceptance is met when tests and docs agree on the stable follow-up vocabulary,
no public JSON shape is changed, and the quote-session evidence plan remains
active but blocked on market phase.

## Idempotence and Recovery

This slice is safe to rerun. If test changes fail, revert only the vocabulary
assertions and docs edits. Do not archive or modify the Desktop quote-session
live-evidence plan beyond listing it as phase-waiting.

## Artifacts and Notes

Do not paste raw live payloads, target ids, account-local metadata, local
absolute paths, credentials, or downstream-private paths into tracked files.

## Interfaces and Dependencies

No new CLI command, option, dependency, data source, version bump, JSON field,
ranking, scoring, recommendation, chart-backed compare, automatic screenshot,
or watch/JSONL behavior is introduced.

## Open Questions

None blocking this slice. Desktop quote-session postmarket and premarket
evidence remains open in
`docs/plans/tradingview-cli-desktop-quote-session-live-evidence.md`.
