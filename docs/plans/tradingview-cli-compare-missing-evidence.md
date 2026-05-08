# Compare missing evidence readback

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is
self-contained and describes how to add item-level missing evidence readback to
`tv compare <SYMBOL>...` without adding new commands, options, data sources, or
recommendations.

## Purpose / Big Picture

`tv compare <SYMBOL>...` already returns a Desktop-free multi-symbol evidence
packet with a summary, ordered items, field coverage, coverage status, and
follow-up hints. Downstream agents can tell that evidence is incomplete, but
they still have to infer from section errors and missing fields which follow-up
surface would help for each item.

After this change, every compare item has an additive `missing_evidence` array.
Each entry names the section with missing evidence, the known missing fields
when available, a simple missing reason, a stable suggested follow-up kind, and
whether that follow-up requires TradingView Desktop. This is readback for
evidence routing only; it does not rank symbols, score securities, or recommend
trades.

## Progress

- [x] (2026-05-08T08:07Z) Created this ExecPlan, archived the completed
  compare follow-up contract plan, and updated current-plan pointers.
- [x] (2026-05-08T08:21Z) Added typed `missing_evidence` fields to the market
  compare payload.
- [x] (2026-05-08T08:21Z) Added unit and live-contract tests for item-level
  missing evidence.
- [x] (2026-05-08T08:21Z) Updated stable docs, runtime skills, roadmap, and
  changelog.
- [x] (2026-05-08T08:21Z) Ran focused compare tests, full Rust baseline, docs
  validation, runtime skill validation, and package script validation.
- [x] (2026-05-08T08:21Z) Prepared the completed slice for a single local
  commit.

## Surprises & Discoveries

- Observation: `missing_summary` currently only has known missing fields from
  successful sections, while section failures live in `errors` and
  `sections.*.error`.
  Evidence: `crates/market/src/compare.rs` builds `missing_summary` from
  `sections.fundamentals.missing_fields`, so the new `missing_evidence`
  readback combines section errors with that existing summary instead of
  changing the summary semantics.

- Observation: the opt-in live compare smoke can validate `missing_evidence`
  without requiring live evidence to be missing.
  Evidence: the test checks that every entry, if present, has a stable section,
  reason, follow-up kind, and Desktop requirement, while allowing an empty
  array for fully covered items.

## Decision Log

- Decision: Keep `missing_summary` unchanged and add `missing_evidence` as a
  separate additive item-level field.
  Rationale: `missing_summary` already feeds summary and field coverage counts.
  A separate field avoids changing existing downstream parsers while exposing
  the routing metadata requested for follow-up workflows.
  Date/Author: 2026-05-08 / Codex.

- Decision: Use existing stable follow-up kinds only: `snapshot` and
  `chart_quote`.
  Rationale: `v0.11.0` and the first `v0.12.0` slice stabilized the follow-up
  vocabulary. This slice should not add a new alias such as `quote_chart` or a
  new `manual_review` kind.
  Date/Author: 2026-05-08 / Codex.

## Outcomes & Retrospective

Implemented. `tv compare <SYMBOL>...` now serializes additive
`items[].missing_evidence[]` entries for section errors and known fundamentals
missing fields. The new field is derived from already collected section
results and `missing_summary`, so it adds no network reads and does not change
source selection. Existing compare fields remain in place.

The mapping is intentionally small: quote section errors point to
`chart_quote` with `requires_desktop: true`; info and fundamentals gaps point
to `snapshot` with `requires_desktop: false`. This keeps the field useful for
downstream routing without creating ranking or recommendation semantics.

## Context and Orientation

The repository is a Cargo workspace for the Rust-native `tv` CLI. The
Desktop-free market evidence code lives under `crates/market/`. The compare
typed payload is defined in `crates/market/src/types.rs`, and the compare
orchestration lives in `crates/market/src/compare.rs`.

`compare` reads quote, symbol info, and default fundamentals for each requested
symbol without using TradingView Desktop or CDP. Each item currently includes
`sections`, `errors`, `missing_summary`, and `follow_up_hints`. `sections`
holds per-section success or error details. `missing_summary` holds known
missing fields from successful sections. `follow_up_hints` lists generally
available follow-up surfaces.

The new `missing_evidence` array is narrower than `follow_up_hints`: it exists
only when an item has missing evidence and names the specific section and
follow-up kind that can help.

## Plan of Work

In `crates/market/src/types.rs`, add a serializable `CompareMissingEvidence`
struct with these fields:

    pub section: String
    pub missing_fields: Vec<String>
    pub missing_reason: String
    pub suggested_follow_up: String
    pub requires_desktop: bool

Add `pub missing_evidence: Vec<CompareMissingEvidence>` to `CompareItem` after
`missing_summary`. Keep all existing fields and names.

In `crates/market/src/compare.rs`, build `missing_evidence` from the already
finalized `SnapshotSections` and `CompareMissingSummary`. Do not perform any
additional network read. The fixed mapping is:

- quote section error: section `quote`, empty `missing_fields`,
  `missing_reason: "section_error"`, `suggested_follow_up: "chart_quote"`,
  `requires_desktop: true`;
- info section error: section `info`, empty `missing_fields`,
  `missing_reason: "section_error"`, `suggested_follow_up: "snapshot"`,
  `requires_desktop: false`;
- fundamentals section error: section `fundamentals`, empty `missing_fields`,
  `missing_reason: "section_error"`, `suggested_follow_up: "snapshot"`,
  `requires_desktop: false`;
- fundamentals missing fields: section `fundamentals`, the known missing field
  names, `missing_reason: "missing_fields"`, `suggested_follow_up:
  "snapshot"`, `requires_desktop: false`.

Add tests proving missing fields, section errors, empty missing evidence, and
total-failure details. Update the opt-in live compare contract smoke so it
checks `missing_evidence` exists and that every entry uses one of the fixed
sections, reasons, and suggested follow-up kinds.

Update `docs/observation-workflows.md`,
`docs/command-source-taxonomy.md`, the runtime skills for market data
interpretation, multi-symbol scans, and screener result analysis, and
`CHANGELOG.md`. The wording must say `missing_evidence` is a readback helper
for missing evidence and follow-up routing, not a recommendation.

## Concrete Steps

From the repository root, inspect the current compare contract with:

    rg -n "CompareItem|missing_summary|follow_up_hints|coverage_status" crates/market crates/cli/tests docs .agents/skills

Edit the compare types and summary construction as described above. Then run
the validation commands in the next section. Update this plan's `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` as
the work completes.

## Validation and Acceptance

Run:

    cargo test -p tradingview-market compare -- --nocapture
    cargo test -p tradingview-cli --test cli_contract compare -- --nocapture
    cargo test -p tradingview-cli --test live_compare
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

Optional read-only smoke:

    target/debug/tv compare NASDAQ:AAPL NYSE:IONQ

Acceptance requires that compare success payloads include `items[].missing_evidence`
without removing any existing compare fields. Items with no missing evidence
must serialize `missing_evidence: []`. Total compare failure details must still
include `contract_version: "compare.v1"`, `summary.coverage_status: "blocked"`,
and the per-item `missing_evidence` arrays.

## Idempotence and Recovery

The implementation is additive and safe to retry. If tests fail, inspect the
compare payload shape and update only the new field construction or tests. Do
not change command-line arguments, network behavior, or source selection to fix
this slice.

If a formatter changes Rust files, review the diff before committing. Do not
write local absolute paths, raw live payloads, target ids, account-local
metadata, or downstream-specific private workflow names into tracked files.

## Artifacts and Notes

Validation evidence will be recorded here after the commands are run.

Validation evidence recorded for this completed slice:

    cargo test -p tradingview-market compare -- --nocapture
    cargo test -p tradingview-cli --test cli_contract compare -- --nocapture
    cargo test -p tradingview-cli --test live_compare
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

The three changed runtime skills were validated with the existing local skill
validator. The exact validator path is local tooling and is intentionally not
recorded in this public plan.

## Interfaces and Dependencies

The new Rust type must be exported through the existing `tradingview-market`
typed API by adding it to `crates/market/src/types.rs` and any existing
re-export list if needed. No new crate dependency is required.

The JSON interface is additive:

    items[].missing_evidence[]:
      section: "quote" | "info" | "fundamentals"
      missing_fields: string[]
      missing_reason: "section_error" | "missing_fields"
      suggested_follow_up: "chart_quote" | "snapshot"
      requires_desktop: boolean

## Open Questions

None. The mapping and vocabulary are fixed by this plan.
