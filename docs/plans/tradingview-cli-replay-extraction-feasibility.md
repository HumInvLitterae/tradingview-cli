# Replay-based extraction feasibility

This ExecPlan is a living document. Keep `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes` updated while working.

## Purpose

`v0.22.0` is maturing observation and export workflows without adding hidden
source mixing. `tv watch compare` added bounded scanner-backed JSONL
observation, and the selected-chart export slice added evidence readback to
`tv range` / `tv ohlcv`. The next lane is Replay-based extraction
feasibility.

This slice does not add a stable Replay export command. It adds public-safe
Replay state and operation readback to existing `tv replay ...` commands so
users and agents can see what Replay state was observed after status, start,
step, stop, autoplay, and trade operations.

## Progress

- [x] Create this ExecPlan.
- [x] Archive the completed selected-chart export feasibility plan.
- [x] Update the plan index, `v0.22.0` roadmap, and changelog for the new
  current slice.
- [x] Add Replay read / operation metadata and `replay_context` readback.
- [x] Add selected-chart `chart_context` where the Replay command can read it.
- [x] Sync Replay extraction feasibility guidance in docs and runtime skills.
- [x] Run focused tests, baseline validation, docs validation, and runtime
  skill validation.

## Work Items

1. Keep Replay extraction feasibility separate from export.
   - `tv replay status` is a Desktop-backed read.
   - `tv replay start`, `step`, `stop`, `autoplay`, and `trade` mutate Replay
     state.
   - Replay is not a fallback for Desktop-free `tv bars --from/--to`.
2. Add additive readback to existing Replay payloads.
   - Status payloads report `source_category: "desktop_backed_read"`,
     `requires_desktop: true`, `non_mutating: true`, `replay_context`, and
     optional `chart_context`.
   - Operation payloads report `source_category: "desktop_backed_operation"`,
     `requires_desktop: true`, `non_mutating: false`, `operation`,
     `replay_context`, and optional `chart_context`.
   - Existing practical fields such as `action`, `previous_date`,
     `current_date`, `position`, and `realized_pnl` remain unchanged.
3. Keep diagnostics public-safe.
   - Replay unavailable, Replay not started, missing method, and start/step
     failures remain source diagnostics.
   - Details must not include raw DOM, raw payload, raw target id,
     account-local metadata, credentials, or local absolute paths.
4. Keep later lanes separate.
   - Stable Replay export, automatic historical export, chart-backed compare,
     source mixing, ranking, and recommendation remain out of this slice.

## Decision Log

- Decision: Add readback to existing Replay commands instead of creating
  `tv replay export`.
  Rationale: the current need is to inspect feasibility and state transitions,
  not to promise a stable export artifact.
- Decision: Mark Replay status as a Desktop-backed read and Replay controls as
  Desktop-backed operations.
  Rationale: status is observational, while start/step/stop/autoplay/trade
  change selected-chart Replay state or Replay trade state.
- Decision: Use `replay_context` as the common readback container.
  Rationale: existing payloads expose related fields at top level; the
  additive container gives downstream tools a stable first-pass location
  without removing old fields.
- Decision: Preserve `tv bars --from/--to` as the reproducible historical bars
  entry point.
  Rationale: Replay depends on selected Desktop chart state and should not
  become an implicit historical bars fallback.

## Outcomes

Implemented additive Replay feasibility readback without adding a stable Replay
export command.

- `tv replay status` now reports public-safe read metadata, `replay_context`,
  and optional selected-chart `chart_context`.
- `tv replay start`, `step`, `stop`, `autoplay`, and `trade` now report
  operation metadata, `non_mutating: false`, `replay_context`, and optional
  `chart_context` while preserving existing practical fields.
- Replay docs and runtime skills now describe Replay as Desktop-backed selected
  chart state readback / mutation, not as a hidden fallback for Desktop-free
  historical bars.
- Focused tests, workspace baseline, docs checks, packaging script syntax, and
  runtime skill validation passed. Optional live Replay smoke was not run.

## Validation

Focused tests:

- [x] `cargo test -p tradingview-cli ops::replay -- --nocapture`
- [x] `cargo test -p tradingview-cli --test cli_contract_desktop replay -- --nocapture`
- [x] `cargo test -p tradingview-cli ops::chart -- --nocapture`
- [x] `cargo test -p tradingview-cli ops::market::ohlcv -- --nocapture`

Rust baseline:

- [x] `cargo fmt --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace`
- [x] `cargo metadata --no-deps --format-version 1`
- [x] `git diff --check`
- [x] `bash -n scripts/stage-release-package-files.sh`

Docs and skills:

- [x] `rg -n "v0\\.22|Replay|replay_context|chart_context|desktop_backed_operation|desktop_backed_read|tv replay|tv bars|source mixing|ranking|recommendation" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md`
- [x] `rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|raw JSONL|raw bars|account-local|target id|downstream-private)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true`
- [x] `uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/replay-practice`
- [x] `uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/chart-analysis`
- [x] `uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/market-data-interpretation`
