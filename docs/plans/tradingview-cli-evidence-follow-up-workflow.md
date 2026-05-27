# Evidence follow-up workflow

This ExecPlan is a living document. Keep `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes` updated while working. Maintain
this document according to `.agents/PLANS.md`.

## Purpose

`tv snapshot` and `tv compare` already return follow-up hints that point to
other evidence surfaces. This slice makes those hints safer for humans and
agents to read by adding source metadata and an explicit `auto_execute: false`
flag. After this change, downstream tools can see which follow-up requires
TradingView Desktop, whether it is a read or operation, and what role the
follow-up evidence can play without treating the hint as a recommendation or
automatic action.

This work does not add commands, sources, source mixing, ranking, or trading
judgment. It only clarifies existing `snapshot.v1` and `compare.v1` readback.

## Progress

- [x] Create this ExecPlan.
- [x] Archive the completed Replay extraction feasibility plan.
- [x] Update the plan index, `v0.22.0` roadmap, and changelog for the current
  slice.
- [x] Add additive follow-up metadata to `snapshot.v1` and `compare.v1`.
- [x] Sync CLI help, docs, release agent guidance, and runtime skills.
- [x] Run focused tests, baseline validation, docs validation, and runtime
  skill validation.

## Work Items

1. Preserve existing follow-up fields.
   - Existing `kind`, `command`, and `reason` fields stay unchanged.
   - Snapshot's existing `requires_desktop` field stays unchanged.
   - Compare gains `requires_desktop` so both surfaces can be read the same
     way.
2. Add advisory metadata to each follow-up hint.
   - `source_category` says whether the follow-up is Desktop-free or
     Desktop-backed.
   - `non_mutating` is true for these follow-up surfaces because they are
     reads or output capture, not chart mutation.
   - `evidence_role` names the kind of evidence the command can add.
   - `auto_execute` is always false.
3. Keep hint vocabulary stable.
   - Stable kinds remain `snapshot`, `chart_quote`, `observe_chart`, and
     `screenshot`.
   - Hints are possible next evidence checks, not rankings,
     recommendations, or instructions to execute automatically.
4. Keep source boundaries separate.
   - `tv bars`, `tv watch compare`, selected-chart reads, Replay, screenshots,
     and scanner-backed reads remain separate sources. This slice does not
     combine them.

## Decision Log

- Decision: Add source and advisory metadata to existing hint objects instead
  of adding a new follow-up command.
  Rationale: downstream tools already consume `follow_up_hints`; additive
  fields make that existing contract clearer without changing command
  behavior.
- Decision: Keep `auto_execute` false for every hint.
  Rationale: hints describe available evidence surfaces, but the CLI must not
  switch charts, take screenshots, start observations, or mix sources on the
  user's behalf.
- Decision: Add `requires_desktop` to compare follow-up hints.
  Rationale: snapshot already exposes this bit; compare should be readable
  with the same shape.

## Outcomes

Implemented additive evidence follow-up metadata without changing command
behavior or adding automatic follow-up execution.

- `snapshot.v1` follow-up hints now include source category, non-mutating
  status, evidence role, and `auto_execute: false` while preserving existing
  fields.
- `compare.v1` follow-up hints now use the same readback shape as snapshot,
  including `requires_desktop`.
- CLI help, public docs, release agent guidance, and runtime skills now
  describe follow-up hints as advisory next evidence checks, not ranking,
  recommendation, source mixing, or automatic command dispatch.
- Focused tests, workspace baseline, docs checks, packaging script syntax, and
  runtime skill validation passed. Optional live smokes remained ignored
  unless explicitly enabled.

## Validation

Focused tests:

- [x] `cargo test -p tradingview-market snapshot -- --nocapture`
- [x] `cargo test -p tradingview-market compare -- --nocapture`
- [x] `cargo test -p tradingview-cli --test cli_contract_quote -- --nocapture`
- [x] `cargo test -p tradingview-cli --test live_snapshot`
- [x] `cargo test -p tradingview-cli --test live_compare`

Rust baseline:

- [x] `cargo fmt --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace`
- [x] `cargo metadata --no-deps --format-version 1`
- [x] `git diff --check`
- [x] `bash -n scripts/stage-release-package-files.sh`

Docs and skills:

- [x] `rg -n "v0\\.22|follow_up_hints|auto_execute|evidence_role|snapshot\\.v1|compare\\.v1|source mixing|ranking|recommendation" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md`
- [x] `rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|raw JSONL|raw bars|account-local|target id|downstream-private)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true`
- [x] `uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/market-data-interpretation`
- [x] `uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/multi-symbol-scan`
- [x] `uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/chart-analysis`
