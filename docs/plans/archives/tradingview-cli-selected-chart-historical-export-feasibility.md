# Selected-chart historical export feasibility

This ExecPlan is a living document. Keep `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes` updated while working.

## Purpose

`tv watch compare` established the first `v0.22.0` bounded observation
workflow. The next roadmap lane is selected-chart historical export, but it
should not be exposed as a stable export command until its source boundary is
clear.

This plan adds selected-chart export evidence readback to existing
selected-chart commands so users can check whether a workflow based on
`tv range` plus selected-chart reads can safely prove that it read the
requested visible chart range. It keeps this source separate from Desktop-free
`tv bars --from/--to`.

## Progress

- [x] Create this ExecPlan.
- [x] Archive the completed watch / JSONL compare plan.
- [x] Update the plan index, `v0.22.0` roadmap, and changelog for the new
  current slice.
- [x] Record selected-chart export as feasibility / contract work rather than
  a stable command implementation.
- [x] Sync source-boundary guidance in docs and runtime skills.
- [x] Run docs validation, hygiene checks, and runtime skill validation.
- [x] Decide that the first implementation is additive `tv ohlcv` /
  `tv range` readback, not a stable export command.
- [x] Add selected-chart context, returned-bars range, and conservative range
  match readback to `tv ohlcv`.
- [x] Add selected-chart viewport operation metadata to `tv range`.
- [x] Run focused tests, baseline validation, docs validation, and runtime
  skill validation.

## Work Items

1. Define the boundary.
   - `tv range` is selected Desktop chart viewport movement.
   - `tv ohlcv` is a selected-chart CDP bars read.
   - `tv range` followed by `tv ohlcv` is not yet a proven historical export
     contract.
   - `tv bars --from/--to` remains the Desktop-free historical bars entry
     point.
2. Implement feasibility evidence needed for a future explicit export
   workflow.
   - Read selected-chart symbol, timeframe, visible range, target readiness,
     and bars availability in a public-safe shape.
   - Check whether the chart reports the requested visible range after
     `tv range --from --to`.
   - Check whether returned `tv ohlcv` bars can be matched to that observed
     selected-chart range without assuming viewport behavior.
3. Define candidate diagnostics before implementation.
   - Ambiguous targets, stale chart state, symbol/timeframe mismatch, bars
     unavailable, and range mismatch should be source diagnostics.
   - Details should include requested range, observed chart state, readiness,
     source metadata, and a next-action hint where safe.
   - Do not expose raw target ids, raw DOM, raw payloads, credentials,
     account-local metadata, or local absolute paths.
4. Keep later lanes separate.
   - Replay-based extraction remains a later feasibility lane.
   - Automatic export, chart-backed compare, source mixing, ranking, and
     recommendation remain out of this slice.

## Decision Log

- Decision: Treat selected-chart historical export as feasibility / contract
  work first.
  Rationale: it depends on Desktop state, selected target, visible range,
  symbol, timeframe, and CDP chart readiness; those must be observable before a
  stable export command is safe.
- Decision: Do not use selected-chart export as a fallback for `tv bars`.
  Rationale: `tv bars --from/--to` is Desktop-free historical evidence, while
  selected-chart export would be Desktop-backed and state-dependent.
- Decision: Do not assume `tv range` changes what `tv ohlcv --count ...`
  returns.
  Rationale: downstream already reported that moving the visible period does
  not prove `tv ohlcv` will return bars from that period.
- Decision: Add readback to existing selected-chart commands instead of adding
  `tv export`.
  Rationale: `tv ohlcv` and `tv range` already own the selected-chart source.
  Adding context and range diagnostics there makes the feasibility visible
  without prematurely creating an export workflow.
- Decision: Use `selected_chart_range_match` as conservative diagnostic
  vocabulary.
  Rationale: overlap can show whether returned bars and visible range intersect,
  but it is not a guarantee that a backtest-ready export was produced.

## Outcomes

This slice adds public-safe selected-chart export evidence readback without a
new command, new option, or source fallback. `tv ohlcv` success payloads now
report chart context, returned bars range, and conservative selected-chart
range-match diagnostics. `tv range` reports that it is a selected-chart
visible-range operation.

Focused tests, full Rust baseline, docs validation, hygiene scans, and runtime
skill validation passed. The hygiene scan reported existing safety-policy and
archive references; no raw target id, raw DOM, raw payload, credential,
account-local metadata, or local absolute path was added by this slice.

## Validation

Docs validation:

- [x] `git diff --check`
- [x] `bash -n scripts/stage-release-package-files.sh`
- [x] `rg -n "v0\\.22|selected-chart export|historical export|tv range|tv ohlcv|visible range|Desktop-backed|tv bars|bars\\.v1|Replay|source mixing|watch_compare\\.v1" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md`

Hygiene:

- [x] `rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|raw JSONL|raw bars|account-local|target id|downstream-private)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true`

Runtime skill validation:

- [x] `uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/market-data-interpretation`
- [x] `uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/multi-symbol-scan`
- [x] `uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/chart-analysis`

Focused tests:

- [x] `cargo test -p tradingview-cli ops::market::ohlcv -- --nocapture`
- [x] `cargo test -p tradingview-cli ops::chart -- --nocapture`
- [x] `cargo test -p tradingview-cli --test cli_contract_desktop -- --nocapture`

Rust baseline:

- [x] `cargo fmt --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace`
- [x] `cargo metadata --no-deps --format-version 1`
