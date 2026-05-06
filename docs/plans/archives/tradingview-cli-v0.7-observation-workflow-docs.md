# v0.7 observation workflow docs and skills cleanup

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is self-contained and describes a docs-and-skills slice for the `v0.7.0` observation workflow lane.

## Purpose / Big Picture

`tv observe chart`, bounded `tv stream ...`, `tv readiness`, `tv screenshot`, lab-gated `tv bars`, and scanner-backed `tv fundamentals` now exist, but users and agents still need a concise path for choosing among them. This slice adds one stable workflow guide and lightly synchronizes runtime skills so they point to the same source taxonomy and evidence boundaries.

After this change, a reader can start at `README.md`, open `docs/observation-workflows.md`, and see when to use Desktop-free screening, Desktop-backed chart observation, screenshots, experimental bars, and fundamentals/event-like fields without reading archived implementation plans.

## Progress

- [x] (2026-05-06T00:00Z) Created this ExecPlan and archived the completed fundamentals/events evidence plan.
- [x] (2026-05-06T00:00Z) Added `docs/observation-workflows.md`.
- [x] (2026-05-06T00:00Z) Updated README, taxonomy, roadmap, and plan index links.
- [x] (2026-05-06T00:00Z) Synchronized runtime skills with the observation workflow guide.
- [x] (2026-05-06T00:00Z) Validated docs, skills, packaging script syntax, and hygiene.
- [x] (2026-05-06T00:00Z) Committed the slice.

## Surprises & Discoveries

- The runtime skills already mentioned `tv observe chart`, `tv readiness`, and lab-gated `tv bars`. The useful cleanup was not to add more procedure, but to give those scattered references a single stable guide.

## Decision Log

- Decision: Add one stable guide at `docs/observation-workflows.md` instead of creating new commands, new runtime skills, or longer skill bodies.
  Rationale: The current gap is operator clarity, not CLI capability. A shared guide reduces duplication while preserving the existing command surface.
  Date/Author: 2026-05-06 / Codex.

- Decision: Keep `tv events`, stable browserless bars, binary split, daemon behavior, MCP server work, and Computer Use-specific skills deferred.
  Rationale: The current roadmap treats these as evidence-gated future choices, and this slice is docs/skills only.
  Date/Author: 2026-05-06 / Codex.

## Outcomes & Retrospective

Added a compact observation workflow guide and linked it from README, source taxonomy, operation boundary docs, and runtime skills. The guide describes practical sequences for Desktop-free screening, Desktop-backed chart observation, screenshot evidence, experimental bars, and fundamentals/event-like fields.

No Rust code, CLI behavior, JSON payload, dependency, or release version changed.

## Context and Orientation

`tv` is a single Rust CLI binary. It uses command source taxonomy rather than separate binaries to distinguish Desktop-free reads, Desktop-backed reads, Desktop-backed operations, hybrid commands, and experimental commands. The current `v0.7.0` roadmap focuses on agent-ready observation workflows.

The most relevant current commands are:

- `tv quotes`, `tv scanner scan`, and `tv fundamentals` for Desktop-free screening;
- `tv readiness` for Desktop-backed chart/session readiness;
- `tv observe chart` for a bounded JSONL workflow that emits readiness first, then selected-chart bar samples and heartbeats;
- `tv stream ...` for lower-level Desktop-backed JSONL samples when a specific sample type is needed;
- `tv screenshot` for portable visual evidence when structured fields are not enough;
- `TV_EXPERIMENTAL_BARS=1 tv bars ...` for lab-gated browserless historical bars evidence.

## Plan of Work

Add `docs/observation-workflows.md` as the durable workflow guide. Keep it concise and public-facing. It must not contain raw live payloads, target ids, local absolute paths, private environment instructions, or claims that experimental bars are stable.

Update `README.md` to link to the guide from Quick Start and Documentation without turning README into an agent handoff. Update `docs/command-source-taxonomy.md`, `docs/operation-adapter-boundaries.md`, and `docs/internal-tradingview-apis.md` only enough to keep the guide and existing source taxonomy aligned.

Update `docs/v0.7-roadmap.md` and `docs/plans/README.md` so this plan is current. Add a concise `CHANGELOG.md` documentation entry.

Update runtime skills `chart-analysis`, `market-data-interpretation`, `multi-symbol-scan`, and, where helpful, `screener-result-analysis` and `screener-workflow`. The edits should reduce scattered explanation by pointing to the guide and preserving the important command choice rules.

## Concrete Steps

From the repository root, inspect the planned files:

    git status --short
    rg -n "observe chart|tv bars|readiness|fundamentals|events" README.md docs .agents/skills

Make the docs and skill edits described above. Then validate:

    git diff --check
    bash -n scripts/stage-release-package-files.sh

Run the repository's existing skill validator for each changed runtime skill. Use the local validation method appropriate to the environment, but do not record private local validation setup in public docs.

Run hygiene checks:

    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|account-local)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true

Rust tests are not required because this slice does not touch Rust code. If Rust code is touched unexpectedly, run:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1

## Validation and Acceptance

Acceptance is met when:

- README links to the new observation workflow guide;
- `docs/observation-workflows.md` describes the five practical paths: Desktop-free screening, Desktop-backed chart observation, stream-specific observation, screenshot evidence, and experimental/fundamentals boundaries;
- runtime skills point to the same workflow choices without adding long private-environment instructions;
- `tv events`, stable browserless bars, binary split, daemon behavior, MCP server, and Computer Use-specific skills remain deferred;
- docs and packaging script checks pass.

## Idempotence and Recovery

All changes are ordinary Markdown edits and can be repeated safely. If a skill validation fails, fix the affected `SKILL.md` and rerun only that validator before rerunning the final docs checks. If the new guide becomes too long, move details back into the source taxonomy or internal API docs rather than expanding runtime skills.

## Artifacts and Notes

The archived previous plan is `docs/plans/archives/tradingview-cli-fundamentals-events-field-evidence.md`.

## Interfaces and Dependencies

No Rust API, CLI option, JSON payload, command behavior, dependency, or release packaging allowlist changes are part of this plan.

## Open Questions

None. Future surface expansion remains evidence-gated by the roadmap.
