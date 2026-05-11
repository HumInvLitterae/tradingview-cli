# Compare regular change evidence

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is
self-contained and describes how to add additive regular-session movement
readback to `tv compare <SYMBOL>...` without changing existing raw evidence
sections or source boundaries.

## Purpose / Big Picture

Downstream tools use `tv compare` as a Desktop-free evidence packet. A
downstream session-mode analyzer reported that `compare.v1` had complete
coverage but still looked unavailable because it could not find a stable
percent-change field. The raw scanner quote section already includes regular
session percentage change at `items[].sections.quote.data.change`, but
downstream tools should not have to hard-code source-specific raw paths for
this common readback.

After this change, each compare item has an additive `movement` object. A
consumer can read `items[].movement.regular_change_percent` as the stable
regular-session percent change readback, and can still inspect the raw quote
section as the source of evidence.

## Progress

- [x] (2026-05-11T12:35Z) Confirmed the current quote contract: scanner quote
  `change` is regular-session percentage change, while extended-hours groups
  use `change_percent` and `change_abs`.
- [x] (2026-05-11T12:45Z) Added typed compare movement readback and movement
  coverage counts.
- [x] (2026-05-11T12:50Z) Added focused market tests for movement extraction
  and movement coverage.
- [x] (2026-05-11T13:00Z) Update CLI live smoke, docs, and runtime skills.
- [x] (2026-05-11T13:25Z) Ran full validation; no release blocker found.

## Surprises & Discoveries

- Observation: The existing typed `Quote` struct documents `change` as
  regular-session percentage change.
  Evidence: `crates/market/src/types.rs` has the field comment
  "Regular-session percentage change" for `Quote::change`.

- Observation: `compare.v1` preserves the scanner quote payload inside
  `items[].sections.quote.data`, but it did not expose a compare-specific
  movement readback.
  Evidence: `crates/market/src/compare.rs` builds each compare item from the
  quote, info, and fundamentals sections, then derives summary and missing
  evidence metadata from those sections.

## Decision Log

- Decision: Add `items[].movement.regular_change_percent` as the stable
  downstream readback instead of renaming or duplicating the raw quote field.
  Rationale: Existing consumers can keep using raw sections, while downstream
  session-mode tools get a source-independent path for regular percent change.
  Date/Author: 2026-05-11 / Codex.

- Decision: Keep `regular_change_abs` as `null` in this slice.
  Rationale: The current scanner quote payload does not expose a normalized
  regular-session absolute change field. Deriving it from last and close would
  be a new calculation policy and is not needed to unblock percent-change
  evidence.
  Date/Author: 2026-05-11 / Codex.

- Decision: Keep the command-local contract marker as `compare.v1`.
  Rationale: The change is additive and does not remove, rename, or reinterpret
  existing fields.
  Date/Author: 2026-05-11 / Codex.

## Outcomes & Retrospective

The implementation now adds `movement` to each compare item and
`summary.movement_coverage` to the compare summary. `tv compare` continues to
report raw quote/info/fundamentals sections, while downstream tools can read a
stable `movement` object for regular-session percent change without treating
complete coverage as unavailable.

Validation passed with focused compare tests, CLI compare contract tests,
compile-only live compare smoke, formatting, clippy, the full workspace test
suite, metadata generation, diff whitespace checks, release packaging script
syntax checks, runtime skill validation, and the public-doc hygiene scan. The
hygiene scan reported existing policy text, example paths, and historical
validation commands only; no new raw live payload, target id, account-local
metadata, credential, local absolute path, or downstream-private path was
added by this slice.

## Context and Orientation

`tv compare <SYMBOL>...` is implemented in `crates/market/src/compare.rs` and
uses types from `crates/market/src/types.rs`. It is a Desktop-free command:
it calls the scanner-backed quote read, symbol info read, and scanner-backed
fundamentals read for each requested symbol. It does not connect to
TradingView Desktop and does not mutate chart state.

The successful quote section preserves the typed scanner quote payload. In
that payload, `change` is the regular-session percentage change. Extended
hours are separate nested groups under `extended_hours.premarket` and
`extended_hours.postmarket`, and those groups use fields such as
`change_percent` and `change_abs`.

## Plan of Work

Add a `CompareMovement` struct to `crates/market/src/types.rs` and add it to
each `CompareItem` as `movement`. Add a `CompareMovementCoverage` struct and
add it to `CompareSummary` as `movement_coverage`.

In `crates/market/src/compare.rs`, derive `CompareMovement` from the finalized
quote section. The stable source path for regular-session percent change is
`sections.quote.data.change`. Copy `sections.quote.data.last` and
`sections.quote.data.close` into `regular_last` and `regular_close` when
present. Keep `regular_change_abs` as `null`, because the source does not
currently expose a normalized regular absolute-change field.

For a successful quote section with numeric `change`, set
`movement.available` to true and `missing_reason` to null. If the quote
section failed, set `available` to false with `missing_reason:
"quote_section_unavailable"`. If the quote section exists but `change` is not
numeric, set `available` to false with `missing_reason:
"regular_change_percent_missing"`.

Add `summary.movement_coverage` counts over the validated requested items.
These counts are readback only and must not alter `summary.coverage_status`.

Update docs and runtime skills so consumers know that
`items[].movement.regular_change_percent` is the stable first-pass path, while
raw `items[].sections.quote.data.change` remains the evidence source.

## Concrete Steps

From the repository root, run focused tests after the type and implementation
edits:

    cargo test -p tradingview-market compare -- --nocapture

Then update CLI smoke assertions and docs. Run:

    cargo test -p tradingview-cli --test cli_contract compare -- --nocapture
    cargo test -p tradingview-cli --test live_compare

Run the full validation set before committing:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

Optional read-only smoke after building:

    target/debug/tv compare SPY QQQ IWM NASDAQ:RKLB

Do not paste raw live output into tracked docs.

## Validation and Acceptance

Acceptance is reached when successful compare items include `movement` with
`source_section: "quote"`, `source_path: "sections.quote.data.change"`, and
`regular_change_percent` matching the raw quote section's `change` value.

`summary.movement_coverage` must report available and missing movement counts
without changing existing `summary.coverage_status` semantics. Existing
`items[]`, raw `sections`, `summary.field_coverage`, `missing_evidence`,
`follow_up_hints`, source metadata, and `compare.v1` must remain present.

The change must not alter `tv quote`, `tv quotes`, `tv snapshot`,
chart-source quote, quote-data, or `--source auto`.

## Idempotence and Recovery

The implementation is additive and safe to rerun. If a test exposes that
`change` is sometimes not numeric in scanner quote data, preserve the raw
section and mark only `movement.available` false rather than failing the whole
compare item.

If downstream later needs absolute regular-session change, add a separate
plan. Do not derive it silently in this slice.

## Interfaces and Dependencies

The public JSON additions are:

    items[].movement.regular_change_percent
    items[].movement.regular_change_abs
    items[].movement.regular_last
    items[].movement.regular_close
    items[].movement.source_section
    items[].movement.source_path
    items[].movement.available
    items[].movement.missing_reason
    summary.movement_coverage.regular_change_percent_available_count
    summary.movement_coverage.regular_change_percent_missing_count
    summary.movement_coverage.regular_change_abs_available_count
    summary.movement_coverage.regular_change_abs_missing_count

No new dependency, source, command, option, or version bump is introduced.

## Open Questions

None. Absolute regular-session change remains intentionally unavailable until
a separate plan defines a safe source or derivation policy.
