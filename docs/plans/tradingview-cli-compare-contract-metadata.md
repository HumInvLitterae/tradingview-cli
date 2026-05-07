# `tv compare` contract metadata

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is
self-contained and describes the first implementation slice after the
`v0.11.0` roadmap.

## Purpose / Big Picture

`tv compare <SYMBOL>...` already returns raw per-symbol evidence and an
additive `summary` readback. Downstream wrappers can use that summary, but they
would be safer if the payload also carried a command-local contract marker,
stable requested-order indexes, machine-readable follow-up hints, and more
explicit missing-field coverage.

After this slice, downstream tools should be able to join `compare` output back
to their original symbol lists, guard fixtures against expected contract shape,
and identify evidence gaps without treating `compare` as ranking, scoring,
recommendation, or realtime chart data.

## Progress

- [x] (2026-05-08T00:00Z) Created this ExecPlan and archived the completed
  v0.10.0 release-readiness ExecPlan.
- [ ] Add additive contract metadata to typed `compare` results.
- [ ] Update compare unit, CLI contract, and live-smoke tests.
- [ ] Update docs and runtime skills to describe the new metadata as readback,
  not judgment.
- [ ] Run validation and record outcomes.

## Surprises & Discoveries

- Observation: Downstream feedback after `v0.10.0` says the highest-leverage
  next improvement is not a new market-data source. It is safer, more
  machine-readable `compare` metadata for wrappers that preserve raw `items[]`.
  Evidence: The current `compare.summary` helps with coverage and missing
  evidence, but downstream still wants stable ordering, follow-up enums, field
  coverage categories, and a contract guard.

## Decision Log

- Decision: Make this slice additive to `tv compare` only.
  Rationale: Existing downstream parsers may already preserve raw `items[]`,
  section errors, top-level counts, `summary`, and `next_action_hints`. The new
  metadata must be easy to ignore.
  Date/Author: 2026-05-08 / Codex.

- Decision: Use a command-local contract marker instead of a global JSON
  envelope version.
  Rationale: Only `compare` needs this downstream guard right now. A global
  envelope version would imply broader stability and migration policy that this
  slice does not need.
  Date/Author: 2026-05-08 / Codex.

- Decision: Treat follow-up hints as available next surfaces, not
  recommendations.
  Rationale: `compare` should not rank or advise trades. It should only tell
  agents which source-specific commands can gather more evidence.
  Date/Author: 2026-05-08 / Codex.

## Outcomes & Retrospective

Not started. Fill this section after implementation and validation. The
expected outcome is an additive `compare` payload that remains Desktop-free and
keeps raw evidence intact while becoming easier for downstream wrappers to
consume.

## Context and Orientation

`tv compare <SYMBOL>...` is a Desktop-free command. Desktop-free means it does
not require TradingView Desktop, Chrome DevTools Protocol, a chart tab, or a
local screenshot. It uses scanner-backed quote, symbol info, and fundamentals
reads from the market crate.

The main implementation lives in `crates/market/src/compare.rs` and the typed
payload structs live in `crates/market/src/types.rs`. The CLI serializes those
typed results into the shared JSON envelope without adding chart reads or
desktop fallback. The existing payload includes source metadata, top-level
counts, `summary`, ordered `items[]`, top-level `errors[]`, and
`next_action_hints`.

The raw `items[]` array must remain the evidence source. `summary` and the new
metadata are readback helpers only.

## Plan of Work

Add `contract_version` to the typed `Compare` payload and serialize it as
`"compare.v1"`. This field is command-local and describes the compare payload
shape, not the global CLI envelope.

Add `requested_index` to each `CompareItem` and each
`CompareResolvedSymbol`. The index is zero-based and reflects the original
argument order after validation. It must not be sorted by resolved symbol,
success state, or section coverage.

Add typed per-item `follow_up_hints[]`. Each hint should include:

- `kind`, one of `snapshot`, `observe_chart`, `chart_quote`, or `screenshot`;
- `command`, a human/executor-readable command string;
- `reason`, a stable readback reason such as `one_symbol_detail`,
  `selected_chart_observation`, `single_symbol_chart_quote`, or
  `visual_evidence`.

The hints describe possible follow-up surfaces. They do not rank candidates,
recommend a trade, or imply that a chart read is required.

Add `summary.field_coverage`. Initial fields should cover counts that can be
derived from existing compare evidence:

- `quote_ok_count` and `quote_missing_count`;
- `info_ok_count` and `info_missing_count`;
- `fundamentals_ok_count` and `fundamentals_missing_count`;
- `earnings_missing_count`;
- `dividends_missing_count`;
- `total_missing_count`.

For earnings and dividends, count only missing fundamentals fields that belong
to the existing earnings or dividends field groups. Do not infer event
meaning, timing meaning, or investment quality from the count.

Keep existing `items[]`, section-level errors, top-level counts,
`summary.resolved_symbols[]`, `errors[]`, source metadata, and
`next_action_hints` compatible. Do not add CLI options. Do not include
TradingView Desktop, chart-source quote loops, screenshots, lab `tv bars`,
ranking, scoring, recommendation, or realtime multi-symbol feed behavior.

Update public docs and runtime skills only enough to explain the new fields:
they are downstream readback helpers, while raw `items[]` remains the evidence
source.

## Concrete Steps

From the repository root, inspect the current compare implementation:

    rg -n "struct Compare|CompareSummary|CompareItem|finalize_compare_items|compare_summary" crates/market/src

Edit `crates/market/src/types.rs` to add the new typed fields and structs.
Edit `crates/market/src/compare.rs` so `finalize_compare_items` derives
contract metadata from already finalized items. Do not perform additional
network reads.

Update compare-focused tests and live smoke contract checks:

    cargo test -p tradingview-market compare -- --nocapture
    cargo test -p tradingview-cli --test cli_contract compare -- --nocapture
    cargo test -p tradingview-cli --test live_compare

Update docs after the payload is stable. Keep repository docs portable: do not
write local absolute paths, raw live payloads, account-local identifiers, or
target ids.

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

Acceptance is met when successful `tv compare` payloads include
`contract_version: "compare.v1"`, stable zero-based `requested_index` values,
per-item `follow_up_hints[]`, and `summary.field_coverage`, while all existing
raw evidence fields remain present and compatible.

Partial section failures must still produce summary and contract metadata when
at least one evidence section succeeds. If all sections fail, the structured
failure details should still include the compare payload and the same metadata.

## Idempotence and Recovery

This slice is additive. If tests fail, preserve the current raw compare shape
and fix only the metadata derivation. If field coverage categorization proves
ambiguous, keep the unambiguous quote/info/fundamentals coverage and record
earnings/dividends categorization as a follow-up instead of guessing.

## Interfaces and Dependencies

No new dependency, command, option, release package behavior, or Desktop-backed
source is introduced. The public JSON shape changes only by adding fields to
the existing `compare` payload.

At the end of the implementation, `Compare` should serialize a
`contract_version` string, each compare item and resolved-symbol summary entry
should serialize a `requested_index`, each item should serialize
`follow_up_hints`, and the summary should serialize `field_coverage`.

## Open Questions

None for the planning slice. If downstream later needs stronger schema
versioning across commands, create a separate cross-command contract plan
instead of broadening this compare-only slice.
