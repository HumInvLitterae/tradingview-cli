# Codex app Computer Use skill research

This ExecPlan is a living document. Keep `Progress`, `Discoveries`,
`Decisions`, and `Validation` updated as work proceeds.

## Purpose

Record the future direction for a Codex app-only TradingView visual recovery
skill that combines the structured `tv` CLI with Computer Use. This slice does
not create a runtime skill. It documents why the idea is promising, why it is
deferred, and what conditions should be met before implementation.

## Progress

- [x] Archived the completed Computer Use boundary cleanup plan.
- [x] Recorded Codex app-only Computer Use skill as a deferred `v0.5.0`
      roadmap candidate.
- [x] Documented candidate workflows and creation conditions.
- [x] Kept runtime skills portable and did not add a new skill or packaging
      allowlist entry.
- [x] Ran docs, packaging, and hygiene validation.
- [x] Committed the docs-only roadmap clarification.

## Decisions

- A Codex app-only Computer Use skill is useful, but should not be created yet.
- The standard runtime skill surface remains CLI-only and screenshot-first.
- A future skill should treat Computer Use as a visual inspection and UI
  recovery aid, not as a replacement for `tv` structured reads.
- Create the skill only after `tv` readiness, screenshot, chart, and Screener
  operation flows are stable enough that the skill is not compensating for
  missing CLI behavior.

## Candidate Workflows

When implemented later, the first skill should probably be small and named
something like `codex-app-tradingview-visual-recovery`. It should cover only:

- chart target recovery when structured target/readiness fields do not explain
  the visible state;
- full-page Screener recovery when target handoff, status, or filters reads do
  not match the visible page;
- visual evidence review when `tv screenshot` is insufficient and the current
  Codex app environment can inspect the visible desktop.

The intended sequence is:

1. Use `tv status`, `tv tab list`, `tv state`, `tv ohlcv`, and
   `tv screenshot` first.
2. Use Computer Use only when the current Codex app environment explicitly
   provides it and the structured CLI evidence is inconclusive.
3. Return to `tv` commands for structured readback or post-check after any
   visual inspection or UI recovery.

## Validation

- `git diff --check`
- `bash -n scripts/stage-release-package-files.sh`
- `rg -n "Computer Use|Codex app|codex-app" docs .agents/skills packaging/agent/AGENTS.md`
- `rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true`

No Rust code changes are expected for this slice.
