# Standalone events feasibility

This ExecPlan is a living document. The sections `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up
to date as work proceeds. This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

`tv fundamentals --group earnings|dividends` and scanner fields can already
return event-like values, such as earnings dates and dividend dates. Those are
scanner-backed field reads. They are useful, but they are not a standalone
earnings, dividends, or calendar event surface.

This slice records whether a future `tv events` command is worth building and
what source boundary it would need. It adds no command, option, source,
dependency, payload semantics, version bump, ranking, recommendation, or
automatic source mixing.

## Progress

- [x] (2026-05-28) Create this ExecPlan.
- [x] (2026-05-28) Archive the completed chart-backed compare contract plan.
- [x] (2026-05-28) Update the v0.23 roadmap, plan index, changelog, docs, and
  runtime skills for event-surface feasibility planning.
- [x] (2026-05-28) Run docs validation, hygiene checks, runtime skill
  validation, and commit the planning slice.

## Surprises & Discoveries

- Observation: current event-like data is available through scanner-backed
  fundamentals and scanner field names.
  Evidence: `docs/internal-tradingview-apis.md` documents
  `fundamentals <SYMBOL> --group earnings|dividends` and scanner columns such
  as earnings release and dividend fields as scanner values.

- Observation: existing workflow docs already warn that earnings and dividend
  groups are not a complete TradingView event calendar or news feed.
  Evidence: `docs/observation-workflows.md` has a Fundamentals And Event-Like
  Fields section with that boundary.

## Decision Log

- Decision: Do not add `tv events` in this slice.
  Rationale: the project first needs to separate scanner field evidence from a
  possible independent event source and contract.
  Date/Author: 2026-05-28 / Codex.

- Decision: Keep `tv fundamentals --group earnings|dividends` as
  scanner-backed field reads.
  Rationale: changing those commands into event-calendar reads would hide a
  source change and break current source-boundary expectations.
  Date/Author: 2026-05-28 / Codex.

- Decision: Treat future `tv events` as a standalone event surface candidate,
  not as a fallback for quote, bars, compare, fundamentals, or chart reads.
  Rationale: event data has different freshness, calendar-range, missing-data,
  and source-availability semantics from quotes or OHLCV.
  Date/Author: 2026-05-28 / Codex.

- Decision: Make symbol-scoped earnings and dividends the first candidate if
  implementation proceeds.
  Rationale: those are closest to current user value and scanner evidence.
  Broader economic calendar or mixed calendar surfaces should wait until the
  source and event-type model are clearer.
  Date/Author: 2026-05-28 / Codex.

## Outcomes & Retrospective

This planning slice records that standalone `tv events` remains feasibility
work. It keeps current earnings and dividend reads under scanner-backed
fundamentals, defines future `tv events` as an independent event-source
candidate, and documents the first likely scope as symbol-scoped earnings and
dividend readback.

Validation passed with diff hygiene, packaging script syntax check, docs grep,
hygiene grep, and runtime skill validation. The hygiene grep reported existing
policy text, archived validation commands, and this plan's safety wording; no
new private data or raw live output was added.

## Context and Orientation

The project currently has several market evidence families:

- price and symbol evidence, such as quote, quotes, compare, snapshot, and
  scanner reads;
- historical OHLCV evidence, such as Desktop-free `tv bars` and explicit
  Desktop-backed `tv export chart-bars`;
- selected-chart evidence, such as chart quote, `tv ohlcv`, screenshots, and
  observations;
- scanner fundamentals fields, including earnings and dividend-adjacent
  values.

Standalone event evidence would be a separate family. It may reuse scanner
field inventory during feasibility, but it should not pretend that scanner
fields are a complete event calendar.

## Plan of Work

Record the source boundary for event-like information and update docs and
runtime skills so agents can describe current evidence accurately:

- current earnings and dividend data is scanner field evidence;
- future `tv events` would need its own source metadata and failure details;
- event evidence is not ranking, recommendation, or trading judgment;
- no source should be substituted automatically.

## Concrete Steps

Run all commands from the repository root.

First inventory current event-like references:

    rg -n "earnings|dividends|calendar|fundamentals|tv events" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md crates

Then update:

- the active plan index and v0.23 roadmap;
- `CHANGELOG.md`;
- source taxonomy, observation workflows, internal API notes, and development
  docs;
- `market-data-interpretation`, `multi-symbol-scan`, and `chart-analysis`;
- packaged agent guidance if needed.

Validate with:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg -n "v0\\.23|tv events|events feasibility|earnings|dividends|calendar|fundamentals|scanner fields|source boundary|ranking|recommendation|source mixing" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|raw JSONL|raw bars|account-local|target id|downstream-private)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true
    uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/market-data-interpretation
    uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/multi-symbol-scan
    uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/chart-analysis

## Validation and Acceptance

This slice is accepted when docs and runtime skills clearly state that:

- current earnings and dividends reads are scanner-backed fundamentals fields;
- standalone `tv events` is not currently a stable command;
- future `tv events` is a separate event-source candidate, not a fallback;
- the first likely implementation candidate is symbol-scoped earnings and
  dividends;
- source metadata, source availability, missing/unavailable reasons, and
  date/time wording must be explicit before implementation;
- no ranking, recommendation, automatic source mixing, or hidden fallback is
  added.

## Idempotence and Recovery

This slice is docs-only and safe to repeat. If the plan index or roadmap
already points here, update the wording rather than adding a duplicate entry.
If a future implementation plan exists, keep this plan as the decision record
and archive it only after the implementation slice starts.

## Artifacts and Notes

Do not paste raw event payloads, account-local metadata, credentials, session
ids, local absolute paths, raw WebSocket frames, raw bars, or raw JSONL output
into tracked docs. Event feasibility notes should record public-safe field
names, source categories, availability states, and failure reasons only.

## Interfaces and Dependencies

No interface is added in this slice.

Future contract candidates for standalone `tv events` include:

- `contract_version`;
- requested and resolved symbol;
- event type;
- event date, time, and session wording;
- source metadata;
- source availability;
- missing or unavailable reason;
- freshness or calendar-range boundary when applicable.

## Open Questions

- Which source, if any, can provide credential-free, public-safe, read-only,
  bounded symbol-scoped event records beyond scanner field values?
- Should the first stable scope be earnings only, dividends only, or both?
- How should before-market, after-market, estimated, confirmed, and unknown
  event timing be represented without over-interpreting source fields?

## Change Note

No runtime behavior changes in this slice.
