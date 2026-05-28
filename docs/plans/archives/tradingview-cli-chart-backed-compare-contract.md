# Chart-backed compare contract

This ExecPlan is a living document. The sections `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up
to date as work proceeds. This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

`tv compare <SYMBOL>...` is a Desktop-free scanner-backed comparison packet.
It is useful for first-pass multi-symbol evidence because it does not depend
on the selected TradingView Desktop chart.

Chart-backed evidence is different. `tv quote <SYMBOL> --source chart`,
selected-chart `tv ohlcv`, screenshots, and `tv export chart-bars` all depend
on TradingView Desktop target state. Before the project adds any chart-backed
compare command, it needs a clear contract that keeps these sources visibly
separate from Desktop-free `tv compare`.

This slice is feasibility and contract planning only. It adds no command,
option, source, dependency, version bump, ranking, recommendation, or automatic
source mixing.

## Progress

- [x] (2026-05-28) Create this ExecPlan.
- [x] (2026-05-28) Archive the completed Replay step-log plan.
- [x] (2026-05-28) Update the v0.23 roadmap, plan index, changelog, docs, and runtime
  skills for chart-backed compare contract planning.
- [x] (2026-05-28) Run docs validation, hygiene checks, runtime skill validation, and commit
  the planning slice.

## Surprises & Discoveries

- Observation: existing `compare` and `snapshot` follow-up hints already use
  `chart_quote` as a stable evidence-surface name.
  Evidence: `docs/command-source-taxonomy.md` and
  `docs/observation-workflows.md` define `chart_quote` as selected-chart
  single-symbol chart-feed quote follow-up, not as a multi-symbol compare
  source.

- Observation: chart-source quote is intentionally single-symbol and selected
  chart dependent.
  Evidence: CLI help for `tv quote --source chart` describes selected
  TradingView Desktop chart feed, while `tv compare` help describes
  Desktop-free multi-symbol evidence.

## Decision Log

- Decision: Keep existing `tv compare <SYMBOL>...` Desktop-free and
  scanner-backed.
  Rationale: adding chart-backed behavior to the existing command would make
  source identity harder for agents and downstream tools to read.
  Date/Author: 2026-05-28 / Codex.

- Decision: Treat chart-backed compare as a separate Desktop-backed workflow
  candidate, not as a hidden fallback.
  Rationale: chart-backed evidence depends on selected chart state, target
  selection, and possibly chart switching. Those are different constraints
  from scanner-backed comparison.
  Date/Author: 2026-05-28 / Codex.

- Decision: Do not add `tv compare --source chart` in the first stable design.
  Rationale: it would put Desktop-free and Desktop-backed compare behind the
  same surface and invite accidental source mixing. If implementation later
  proceeds, prefer a separated surface such as `tv compare chart ...` or
  `tv chart compare ...`.
  Date/Author: 2026-05-28 / Codex.

- Decision: Use documented workflow as the default for this slice.
  Rationale: existing commands can already collect finalist chart evidence.
  Planning should first clarify how to report source boundaries before adding a
  new command.
  Date/Author: 2026-05-28 / Codex.

## Outcomes & Retrospective

This planning slice records chart-backed compare as a separate Desktop-backed
workflow candidate. It keeps existing `tv compare <SYMBOL>...` Desktop-free
and scanner-backed, rejects `tv compare --source chart` as the first stable
design, and makes documented workflow the default until a separated command is
worth implementing.

Validation passed with diff hygiene, packaging script syntax check, docs grep,
hygiene grep, and runtime skill validation. The hygiene grep reported existing
policy text, archived validation commands, and this plan's safety wording; no
new private data or raw live output was added.

## Context and Orientation

The repository has three relevant evidence families:

- Desktop-free multi-symbol evidence: `tv compare`, `tv watch compare`,
  `tv quotes`, scanner reads, and browserless `tv bars`.
- Desktop-backed selected-chart reads: chart quote, current-chart `tv ohlcv`,
  screenshots, `tv observe chart`, and `tv stream ...`.
- Desktop-backed selected-chart operations: symbol/timeframe/range changes,
  selected-chart export, and Replay operations.

Chart-backed compare belongs to the Desktop-backed family. It may eventually
be useful for finalist comparisons, but it cannot be treated as a drop-in
replacement for scanner-backed `tv compare`.

## Plan of Work

Record that chart-backed compare is not yet a stable command. Update docs and
runtime skills so agents use `tv compare` / `tv watch compare` for broad
Desktop-free comparison, then use chart-backed reads only for explicit
finalist follow-up.

The future implementation candidates are:

- documented workflow only, using `tv quote --source chart`, `tv ohlcv`,
  `tv export chart-bars`, and screenshots explicitly;
- separated command such as `tv compare chart ...` or `tv chart compare ...`;
- no `tv compare --source chart` in the first stable design.

Any future chart-backed compare command must report `contract_version`, source
metadata, selected chart context, target ambiguity, symbol/timeframe mismatch,
mutation requirements, readback status, and public-safe failure details.

## Concrete Steps

Run all commands from the repository root.

First confirm the current source boundary:

    rg -n "tv compare|chart_quote|tv quote.*source chart|watch_compare\\.v1|export_chart_bars\\.v1" README.md docs .agents/skills packaging/agent/AGENTS.md crates/cli/src crates/market/src

Then update:

- the active plan index and v0.23 roadmap;
- `CHANGELOG.md`;
- source taxonomy, observation workflows, and development docs;
- `chart-analysis`, `market-data-interpretation`, and `multi-symbol-scan`;
- packaged agent guidance if needed.

Validate with:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg -n "v0\\.23|chart-backed compare|tv compare|tv quote.*source chart|selected-chart|tv ohlcv|tv export chart-bars|watch compare|source mixing|ranking|recommendation" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|raw JSONL|raw bars|account-local|target id|downstream-private)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true
    uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/chart-analysis
    uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/market-data-interpretation
    uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/multi-symbol-scan

## Validation and Acceptance

This slice is accepted when docs and runtime skills clearly state that:

- `tv compare` remains Desktop-free and scanner-backed;
- chart-backed compare is not yet a stable command;
- chart-source quote is selected-chart single-symbol evidence, not
  scanner-style multi-symbol compare;
- future chart-backed compare must be source-separated and public-safe;
- no ranking, recommendation, automatic source mixing, or hidden fallback is
  added.

## Idempotence and Recovery

This slice is docs-only and safe to repeat. If the plan index or roadmap
already points here, update the wording rather than adding a duplicate entry.
If a future implementation plan exists, keep this plan as the decision record
and archive it only after the implementation slice starts.

## Artifacts and Notes

Do not paste raw chart output, raw target ids, raw DOM, raw payloads,
account-local metadata, credentials, or local absolute paths into tracked
docs. Chart-backed compare evidence should use public-safe source diagnostics
only.

## Interfaces and Dependencies

No new interface is added in this slice.

Future command candidates remain:

- `tv compare chart ...`
- `tv chart compare ...`
- documented workflow only

The initial non-candidate is:

- `tv compare --source chart`

## Open Questions

None for this planning slice. The default decision is documented workflow
first; if implementation proceeds later, prefer a separated command surface.

## Change Note

No runtime behavior changes in this slice.
