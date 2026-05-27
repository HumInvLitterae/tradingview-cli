# Selected-chart historical export feasibility

This ExecPlan is a living document. Keep `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes` updated while working.

## Purpose

`tv watch compare` established the first `v0.22.0` bounded observation
workflow. The next roadmap lane is selected-chart historical export, but it
should not be exposed as a stable export command until its source boundary is
clear.

This plan checks whether a workflow based on `tv range` plus selected-chart
reads can safely prove that it read the requested visible chart range. It keeps
this source separate from Desktop-free `tv bars --from/--to`.

## Progress

- [x] Create this ExecPlan.
- [x] Archive the completed watch / JSONL compare plan.
- [x] Update the plan index, `v0.22.0` roadmap, and changelog for the new
  current slice.
- [x] Record selected-chart export as feasibility / contract work rather than
  a stable command implementation.
- [x] Sync source-boundary guidance in docs and runtime skills.
- [x] Run docs validation, hygiene checks, and runtime skill validation.

## Work Items

1. Define the boundary.
   - `tv range` is selected Desktop chart viewport movement.
   - `tv ohlcv` is a selected-chart CDP bars read.
   - `tv range` followed by `tv ohlcv` is not yet a proven historical export
     contract.
   - `tv bars --from/--to` remains the Desktop-free historical bars entry
     point.
2. Identify feasibility evidence needed for a future explicit export workflow.
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

## Outcomes

This slice should leave the repository with a clear selected-chart export
feasibility plan and synchronized guidance. It should not add a command, new
option, payload semantics, or source fallback.

Docs validation, hygiene scans, and runtime skill validation passed. The
hygiene scan reported existing safety-policy and archive references, plus this
plan's public-safe validation command; no raw target id, raw DOM, raw payload,
credential, account-local metadata, or local absolute path was added by this
slice.

## Validation

Docs validation:

- [x] `git diff --check`
- [x] `bash -n scripts/stage-release-package-files.sh`
- [x] `rg -n "v0\\.22|selected-chart export|historical export|tv range|tv ohlcv|visible range|Desktop-backed|tv bars|bars\\.v1|Replay|source mixing|watch_compare\\.v1" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md`

Hygiene:

- [x] `rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|raw JSONL|raw bars|account-local|target id|downstream-private)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true`

Runtime skill validation:

- [x] `uvx --with pyyaml python "${CODEX_HOME:-$HOME/.codex}/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/market-data-interpretation`
- [x] `uvx --with pyyaml python "${CODEX_HOME:-$HOME/.codex}/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/multi-symbol-scan`
- [x] `uvx --with pyyaml python "${CODEX_HOME:-$HOME/.codex}/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/chart-analysis`

Rust tests are not required for this docs / feasibility slice unless Rust code
is changed.
