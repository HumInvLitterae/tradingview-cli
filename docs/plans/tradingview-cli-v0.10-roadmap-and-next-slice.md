# v0.10.0 roadmap and next-slice planning

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is self-contained and records the planning step after the `v0.9.0` release.

## Purpose / Big Picture

`v0.9.0` added `tv compare <SYMBOL>...`, a Desktop-free multi-symbol evidence packet. Before adding more features, the project needs a `v0.10.0` roadmap that explains what should be polished next and what should remain deferred.

After this slice, contributors should have a durable roadmap and a short list of next implementation candidates that can be shared with downstream development agents for feedback before implementation starts.

## Progress

- [x] (2026-05-08T00:00Z) Confirmed `v0.9.0` release readiness is complete and archived the completed release-readiness plan.
- [x] (2026-05-08T00:00Z) Added the `v0.10.0` roadmap draft.
- [x] (2026-05-08T00:00Z) Updated the current plan index and `v0.9.0` roadmap handoff state.
- [x] (2026-05-08T00:00Z) Recorded that this planning slice should not be committed yet because downstream development-agent feedback is requested first.
- [x] (2026-05-08T00:00Z) Incorporated downstream feedback that `compare` summary polish is more valuable than new market-data surface work.
- [ ] Wait for final user confirmation before committing this planning slice or creating the next implementation ExecPlan.

## Surprises & Discoveries

- The current docs and runtime skills already mention `compare` in the main Desktop-free workflow.
- Downstream tools can preserve the raw `compare` shape today, but they would benefit from additive machine-readable readback fields that avoid re-parsing the raw item list for common counts and resolved symbol mappings.

## Decision Log

- Decision: Treat `v0.10.0` as comparison workflow polish and evidence follow-up rather than a broad realtime or ranking release.
  Rationale: `compare` is new in `v0.9.0`; expanding it before downstream feedback would risk adding surface that agents do not need.
  Date/Author: 2026-05-08 / Codex.

- Decision: Make additive `compare` summary polish the leading first implementation candidate.
  Rationale: Downstream feedback says the highest-leverage upstream change is not ranking, recommendation, or realtime data, but machine-readable scanability such as resolution summaries, per-section counts, missing-value counts, and resolved symbol lists.
  Date/Author: 2026-05-08 / Codex.

- Decision: Do not commit this planning slice immediately.
  Rationale: The user wants downstream development agents to review the roadmap before it becomes a committed project decision.
  Date/Author: 2026-05-08 / Codex.

## Outcomes & Retrospective

The initial roadmap draft was updated after downstream review. The direction
remains `comparison workflow polish and evidence follow-up`, but the first
candidate moved from docs-only workflow alignment to additive `compare`
summary polish, with docs/skills decision-table cleanup as the second
candidate.

## Context and Orientation

The current CLI has three relevant Desktop-free evidence surfaces:

- `tv quotes <SYMBOL>...` for ordered quote-only reads;
- `tv snapshot <SYMBOL>` for one-symbol quote, info, and fundamentals evidence;
- `tv compare <SYMBOL>...` for multi-symbol quote, info, and default fundamentals evidence.

Desktop-backed follow-up remains explicit through `tv readiness`, `tv observe chart`, `tv quote --source chart`, and `tv screenshot`. Chart-source quote is correctness-first and single-symbol; it should not be treated as a multi-symbol realtime comparison source.

`tv bars` remains lab-gated. Stable browserless bars, browserless streaming, `tv diagnose`, binary split, MCP server work, daemon behavior, and standalone `tv events` remain deferred unless a separate plan changes those boundaries.

## Plan of Work

Add `docs/v0.10-roadmap.md` with the theme `comparison workflow polish and evidence follow-up`. Record lanes for compare workflow polish, finalist follow-up workflow, realtime/browserless data strategy, fundamentals/events follow-up, and deferred infrastructure.

Update `docs/plans/README.md` so this plan is the current plan. Archive the completed `docs/plans/tradingview-cli-v0.9.0-release-readiness.md` plan. Update `docs/v0.9-roadmap.md` so it says `v0.9.0` is released and points to the `v0.10.0` roadmap.

Add a short `CHANGELOG.md` Unreleased documentation entry. Do not update Cargo version, CLI behavior, JSON payloads, Rust code, dependencies, release notes, or packaging scripts in this slice.

## Candidate Next Slices

Recommended first candidate:

- Compare payload summary polish. Add an additive top-level `summary` or `resolution_summary` that helps agents read counts and resolved symbol mappings without changing `items`.

  The summary should preserve the existing raw item shape and avoid ranking,
  scoring, recommendations, or trading-action inference. Good initial fields
  include `requested_count`, `resolved_count`, `error_count`,
  `quote_ok_count`, `fundamentals_ok_count`, `missing_total_count`, and a
  stable `resolved_symbols` list that maps `requested_symbol` to
  `observed_symbol` / `symbol` where available.

Second candidate:

- Compare workflow docs and runtime skill alignment. This should add a small
  decision table for `quotes`, `snapshot`, `compare`, and `observe chart`
  rather than only more prose.

Research candidate:

- Browserless realtime or bars strategy evidence. Keep this separate from `compare` unless a reliable source and workflow need are established.

## Validation and Acceptance

Run:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg -n "v0\\.10|compare|snapshot|observe chart|tv bars|tv events|diagnose|binary split|MCP|daemon|realtime" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|account-local|target id)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts || true

Acceptance is met when the roadmap clearly presents `v0.10.0` as a planning phase after the `v0.9.0` release, current plan references are updated, no Rust code changes are made, and the working tree remains uncommitted for downstream review.

## Idempotence and Recovery

This slice is docs-only. If downstream feedback changes the roadmap, replace the roadmap and this ExecPlan before committing. If feedback says implementation should proceed immediately, create a new implementation ExecPlan rather than broadening this planning slice.

## Interfaces and Dependencies

No public interface, JSON payload, Rust API, dependency, version, release package, or CI workflow changes are introduced.

## Open Questions

- What do downstream development agents find hardest to consume in `tv compare` today?
- Do they need docs/skills guidance first, or an additive payload summary field?
- Is there concrete evidence that realtime or browserless data work should outrank compare workflow polish?
