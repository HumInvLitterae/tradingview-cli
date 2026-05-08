# `tv snapshot` contract metadata

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is
self-contained and describes the `v0.13.0` implementation slice that can move
while Desktop quote-session postmarket and premarket evidence is still waiting
for the right market phase.

## Purpose / Big Picture

`tv snapshot <SYMBOL>` is the one-symbol Desktop-free evidence packet. It
already returns quote, symbol info, and fundamentals sections, but downstream
agents must inspect each section manually to know whether evidence is complete
or what to read next. This slice adds additive contract metadata so an agent can
quickly read coverage status, missing evidence, and follow-up surfaces without
losing the raw `sections` evidence.

After this change, `tv snapshot NASDAQ:AAPL` still returns the existing
`sections`, `errors`, and `next_action_hints`, and additionally returns
`contract_version: "snapshot.v1"`, `summary`, `missing_evidence[]`, and
machine-readable `follow_up_hints[]`.

## Progress

- [x] (2026-05-08T17:05Z) Created this ExecPlan while keeping the Desktop
  quote-session evidence plan active but blocked on postmarket or premarket
  timing.
- [x] (2026-05-08T17:05Z) Added typed snapshot metadata structs and additive
  payload construction.
- [x] (2026-05-08T17:05Z) Added focused market crate tests for coverage status,
  missing evidence, follow-up hints, and blocked details.
- [x] (2026-05-08T17:05Z) Updated docs, runtime skills, changelog, and the
  ignored live snapshot contract smoke expectations.
- [x] (2026-05-08T17:05Z) Validation passed: focused snapshot tests, ignored
  live snapshot smoke compile, skill validation, formatting, clippy, full
  workspace tests, metadata, diff check, and packaging script syntax.
- [ ] Commit the completed slice.

## Surprises & Discoveries

- Observation: `snapshot` and `compare` already share `SnapshotSections`, so
  most coverage and missing-evidence readback can be derived without new
  network reads.
  Evidence: `crates/market/src/snapshot.rs` builds quote, info, and
  fundamentals sections before shaping the payload.

- Observation: `snapshot` only exposes known missing fields through the
  fundamentals section today.
  Evidence: quote and info sections either succeed with their existing section
  payloads or return section errors; there is no stable quote/info missing-field
  list to count without inventing new semantics.

## Decision Log

- Decision: Keep this slice independent from Desktop quote-session
  postmarket/premarket evidence.
  Rationale: `snapshot` is Desktop-free and can be improved while chart quote
  session support waits for phase-specific live evidence.
  Date/Author: 2026-05-08 / Codex.

- Decision: Add `snapshot.v1` as a command-local contract marker.
  Rationale: Downstream tools need a schema guard for snapshot payloads, but
  this does not imply a global envelope version or cross-command schema
  migration.
  Date/Author: 2026-05-08 / Codex.

- Decision: Keep raw `sections` as the evidence source and treat new metadata
  as readback only.
  Rationale: `summary`, `missing_evidence`, and `follow_up_hints` should help
  route follow-up work, not rank symbols or replace section data.
  Date/Author: 2026-05-08 / Codex.

- Decision: Count `field_coverage` missing values only from known section
  missing fields, with fundamentals as the current source.
  Rationale: This keeps snapshot metadata conservative and avoids treating a
  section error as an invented field-level list.
  Date/Author: 2026-05-08 / Codex.

## Outcomes & Retrospective

Implemented. `tv snapshot <SYMBOL>` now returns additive contract metadata:
`contract_version: "snapshot.v1"`, `summary`, `missing_evidence[]`, and
machine-readable `follow_up_hints[]`. The existing `sections`, `errors`,
`next_action_hints`, source metadata, `requested_symbol`, `symbol`, and
`observed_symbol` remain present.

`summary.coverage_status` is conservative: `complete` requires all three
sections to succeed with no known missing fields, `partial` means at least one
section produced evidence but gaps remain, and `blocked` is used for no-section
evidence in structured failure details. `missing_evidence[]` is routing
readback only; it points quote section failures to `chart_quote` and
info/fundamentals gaps to `snapshot` without executing those follow-up reads.

Validation passed with:

- `cargo test -p tradingview-market snapshot -- --nocapture`
- `cargo test -p tradingview-cli --test cli_contract snapshot -- --nocapture`
- `cargo test -p tradingview-cli --test live_snapshot`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo metadata --no-deps --format-version 1`
- `git diff --check`
- `bash -n scripts/stage-release-package-files.sh`
- skill validation for the changed runtime skills

The Desktop quote-session postmarket/premarket evidence plan remains active
and waiting for the relevant market phase.

## Context and Orientation

`tv snapshot <SYMBOL>` is implemented in `crates/market/src/snapshot.rs` and
serialized through the existing CLI JSON envelope. The typed payload structs
live in `crates/market/src/types.rs`. The command is Desktop-free: it does not
connect to TradingView Desktop, does not read a chart, and does not capture
screenshots.

The snapshot command currently reads three independent sections: scanner-backed
quote, Desktop-free symbol info, and scanner-backed fundamentals. Each section
is represented by `SnapshotSection`, which has `ok`, optional `data`, and
optional public-safe `error`.

## Plan of Work

Add typed snapshot metadata in `crates/market/src/types.rs`: a command-local
contract marker on `Snapshot`, a `SnapshotSummary`, a `SnapshotFieldCoverage`,
`SnapshotMissingEvidence`, and `SnapshotFollowUpHint`.

Update `crates/market/src/snapshot.rs` so the new fields are derived from the
already-built sections. Do not perform any additional network reads. The
coverage status must be `complete` only when quote, info, and fundamentals all
succeed and there are no known missing fields; `partial` when at least one
section succeeds but some section fails or known fields are missing; and
`blocked` when no section succeeds.

Keep `sections`, `errors`, `next_action_hints`, source metadata,
`requested_symbol`, `symbol`, and `observed_symbol` intact. Add
machine-readable `follow_up_hints[]` for `chart_quote`, `observe_chart`, and
`screenshot`. Add `missing_evidence[]` entries for section errors and
fundamentals `missing_fields`; route quote section errors to `chart_quote` and
info/fundamentals gaps to `snapshot`.

Update docs and runtime skills to explain that snapshot metadata mirrors
compare-style readback for one symbol. Raw `sections` remain the evidence
source.

## Concrete Steps

From the repository root, run:

    cargo test -p tradingview-market snapshot -- --nocapture

Then update docs and skills, and run the full validation listed below.

## Validation and Acceptance

Run:

    cargo test -p tradingview-market snapshot -- --nocapture
    cargo test -p tradingview-cli --test cli_contract snapshot -- --nocapture
    cargo test -p tradingview-cli --test live_snapshot
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

Acceptance is met when successful snapshot payloads include
`contract_version: "snapshot.v1"`, `summary.coverage_status`,
`summary.field_coverage`, `missing_evidence[]`, and `follow_up_hints[]`, while
existing `sections`, `errors`, `next_action_hints`, and source metadata remain
present. Total failure details must include the same snapshot metadata with
`coverage_status: "blocked"`.

## Idempotence and Recovery

This slice is additive. If tests fail, revert only the snapshot metadata edits
and leave the quote-session evidence plan untouched. The optional live smoke is
ignored by default and must not become a CI requirement.

## Artifacts and Notes

Do not paste raw live payloads, target ids, account-local metadata, local
absolute paths, credentials, or downstream-private paths into tracked files.

## Interfaces and Dependencies

No new CLI command, option, dependency, data source, version bump, chart-backed
snapshot, screenshot automation, watch/JSONL behavior, ranking, scoring, or
recommendation is introduced. The new public JSON fields are additive and are
produced by the existing `tradingview-market` typed snapshot API.

## Open Questions

None blocking this slice. Desktop quote-session postmarket and premarket
evidence remains open in
`docs/plans/tradingview-cli-desktop-quote-session-live-evidence.md`.
