# Compare workflow decision table

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It describes
a docs-and-skills polish slice after the additive `tv compare` summary work.

## Purpose / Big Picture

`tv compare <SYMBOL>...` now returns both raw per-symbol evidence and additive
summary readback. The remaining v0.10 polish is to make the common workflow
choice obvious: when to use `quotes`, `compare`, `snapshot`, chart observation,
chart-source quote, or screenshots.

After this slice, humans and agents should be able to choose the right read
surface from a short decision table without treating `compare.summary` as a
ranking, recommendation, or raw evidence replacement.

## Progress

- [x] (2026-05-08T00:00Z) Created this ExecPlan and archived the completed
  compare summary polish plan.
- [x] (2026-05-08T00:00Z) Added the workflow decision table to stable docs.
- [x] (2026-05-08T00:00Z) Synchronized README and runtime skills with the same
  command choices.
- [x] (2026-05-08T00:00Z) Ran docs validation, packaging syntax check, skill
  validation, and public-doc hygiene grep.

## Surprises & Discoveries

- Existing docs already described the right boundaries in prose. The useful
  change was to make those boundaries scannable rather than adding more
  explanation.

## Decision Log

- Decision: Put the decision table in `docs/observation-workflows.md`.
  Rationale: That guide already owns practical command sequencing, while
  `docs/command-source-taxonomy.md` owns source categories.
  Date/Author: 2026-05-08 / Codex.

- Decision: Keep README as a pointer, not a full workflow table.
  Rationale: README should stay a human-facing overview and avoid duplicating
  stable workflow docs.
  Date/Author: 2026-05-08 / Codex.

## Outcomes & Retrospective

Implemented. The workflow table now distinguishes quote-only reads,
multi-symbol evidence packets, one-symbol detail, selected-chart observation,
single-symbol chart-feed follow-up, and screenshot evidence.

Validation passed. No Rust code or CLI contract changed.

## Context and Orientation

The current relevant surfaces are:

- `tv quotes <SYMBOL>...` for ordered quote-only reads;
- `tv compare <SYMBOL>...` for Desktop-free multi-symbol evidence;
- `tv snapshot <SYMBOL>` for one-symbol detail;
- `tv observe chart` for selected-chart time-window observation;
- `tv quote <SYMBOL> --source chart` for explicit single-symbol chart-feed
  follow-up;
- `tv screenshot` for visual evidence after structured reads.

`compare.summary` is a readback helper. `items[]` remains the evidence source.

## Plan of Work

Add a compact decision table to `docs/observation-workflows.md`. Keep it close
to the Desktop-free and Desktop-backed workflow sections so readers find it
before lower-level command details.

Update README with a short pointer from the Desktop-free examples to the
workflow guide. Update runtime skills so they use the same command choices and
do not imply that `compare` computes rankings or recommendations.

Update the v0.10 roadmap and plan index so this decision-table slice is the
current completed plan. Record the docs/skills polish in `CHANGELOG.md`.

## Validation and Acceptance

Run:

    git diff --check
    bash -n scripts/stage-release-package-files.sh

Run the existing skill validator against the changed runtime skills:

    .agents/skills/market-data-interpretation
    .agents/skills/multi-symbol-scan
    .agents/skills/screener-result-analysis

Acceptance is met when the decision table is present, README links naturally
to the workflow guide, skills use the same command boundaries, no Rust code is
changed, and public docs contain no new local paths, secrets, raw target ids,
account-local metadata, or raw live payloads.

## Idempotence and Recovery

This slice is docs-only. If the wording becomes too long, preserve the table
and shorten surrounding prose rather than adding more sections.

## Interfaces and Dependencies

No CLI behavior, JSON payload, Rust API, dependency, release package behavior,
or version changes are introduced.

## Open Questions

None. Further compare payload polish should be planned separately.
