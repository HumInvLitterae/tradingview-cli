# `tv compare` summary polish

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is
self-contained and describes the first implementation slice after the
`v0.10.0` roadmap.

## Purpose / Big Picture

`tv compare <SYMBOL>...` already returns the raw per-symbol evidence that
downstream wrappers should preserve. The next improvement is an additive
machine-readable readback summary so agents can see resolution, section
success counts, and missing counts without reinterpreting the whole `items`
array.

After this slice, `compare` should be easier to consume while remaining a
Desktop-free evidence packet, not a ranking, scoring, recommendation, or
realtime-feed command.

## Progress

- [x] (2026-05-08T00:00Z) Created this ExecPlan and archived the completed
  v0.10 roadmap planning ExecPlan.
- [x] (2026-05-08T00:00Z) Added typed compare summary structs and additive
  `data.summary` serialization.
- [x] (2026-05-08T00:00Z) Updated compare unit/live-smoke contract checks for
  summary counts and resolved symbol order.
- [x] (2026-05-08T00:00Z) Updated docs and runtime skills to treat summary as
  readback, not a raw evidence replacement.
- [x] (2026-05-08T00:00Z) Ran focused tests, full Rust baseline, skill
  validation, packaging syntax check, hygiene grep, and Desktop-free read-only
  smokes.

## Surprises & Discoveries

- `CompareItem` already stores best `symbol`, best `observed_symbol`, section
  success state, and item-level `missing_summary`, so summary construction can
  be derived entirely from existing evidence without another network read.

## Decision Log

- Decision: Add `data.summary` rather than replacing top-level count fields.
  Rationale: Existing downstream users may already read `requested_count`,
  `resolved_count`, `error_count`, and `items`. The new field is additive and
  easier to ignore.
  Date/Author: 2026-05-08 / Codex.

- Decision: Build `resolved_symbols[]` from existing per-item fields only.
  Rationale: Summary is readback, not a new symbol-resolution source. It must
  not infer better symbols or hide section-level evidence.
  Date/Author: 2026-05-08 / Codex.

## Outcomes & Retrospective

Implemented additive `data.summary` for `tv compare <SYMBOL>...`. Existing
raw `items`, top-level counts, errors, source metadata, and next-action hints
remain in place. The summary reports count readbacks and ordered resolved
symbol mappings so downstream tools can build follow-up inputs without
re-parsing the entire item array.

Validation passed. Read-only smoke confirmed that `compare` returns summary
data without requiring CDP/Desktop access.

## Context and Orientation

`tv compare` is a Desktop-free command implemented in `tradingview-market` and
exposed through the CLI JSON envelope. It gathers scanner quote, symbol info,
and default scanner-backed fundamentals evidence for each requested symbol.

The current top-level payload already includes `requested_count`,
`resolved_count`, `error_count`, ordered `items`, top-level `errors`, source
metadata, and next-action hints. The summary added in this slice must not
remove or rename any of those fields.

## Plan of Work

Add typed `CompareSummary` and `CompareResolvedSymbol` structs. Add a
`summary` field to `Compare`. In `finalize_compare_items`, compute summary
from the finalized `items` before building the `Compare` value.

The initial summary shape is:

- `requested_count`
- `resolved_count`
- `error_count`
- `quote_ok_count`
- `info_ok_count`
- `fundamentals_ok_count`
- `missing_total_count`
- `resolved_symbols[]`

Each `resolved_symbols[]` item preserves input order and includes
`requested_symbol`, `ok`, `symbol`, `observed_symbol`, `quote_ok`, `info_ok`,
`fundamentals_ok`, and `missing_total_count`.

Update docs and runtime skills to explain that `summary` is a scanability
helper. Agents may use it for first-pass readback, but should inspect raw
`items[]` before making any substantive comparison or follow-up decision.

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

Acceptance is met when `tv compare` success payloads contain `summary`, the
summary counts match the top-level counts and section states, resolved symbol
readback preserves input order, all existing `items[]` / `errors[]` /
`next_action_hints` fields remain compatible, and validation passes.

## Idempotence and Recovery

This slice is additive. If tests fail, preserve the existing raw compare
payload shape and fix only the summary derivation. Do not add CLI options,
ranking, scoring, chart reads, screenshot reads, or lab `tv bars` integration.

## Interfaces and Dependencies

No new CLI option, dependency, release package behavior, or Desktop-backed
source is introduced. The public JSON shape changes only by adding
`data.summary`.

## Open Questions

None for this slice. If downstream later needs richer summaries, add them as a
separate additive plan after this first contract has shipped.
