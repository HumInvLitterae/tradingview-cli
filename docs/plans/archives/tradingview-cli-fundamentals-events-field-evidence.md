# Fundamentals / events scanner field evidence

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is self-contained and describes the next `v0.7.0` research/docs slice: recording scanner metainfo evidence for fundamentals, earnings, dividends, and event-like fields before adding any new public surface.

## Purpose / Big Picture

`tv fundamentals` already supports curated `earnings`, `valuation`, `dividends`, and `financials` groups. The v0.7 roadmap also names future events/calendar reads as useful, especially around earnings and dividends. This slice checks what scanner metainfo actually exposes and records the boundary before adding commands such as `tv events` or new field groups.

After this change, contributors should know that current event-like evidence is scanner field metadata, not a full TradingView event calendar, news feed, or financial statement API.

## Progress

- [x] (2026-05-06T00:00Z) Created this ExecPlan and archived the completed lab bars live-smoke plan.
- [x] (2026-05-06T00:00Z) Gathered scanner metainfo evidence for earnings and dividend fields.
- [x] (2026-05-06T00:00Z) Added public-safe field evidence note.
- [x] (2026-05-06T00:00Z) Updated stable docs and roadmap.
- [x] (2026-05-06T00:00Z) Validated the slice.
- [x] (2026-05-06T00:00Z) Committed the slice.

## Surprises & Discoveries

- Scanner metainfo exposes many date/time fields, including additional earnings and dividend candidates beyond the current field groups.
- The evidence still looks like scanner field bundles rather than a complete standalone event calendar.

## Decision Log

- Decision: Do not add `tv events` or a new `events` group in this slice.
  Rationale: The available evidence is useful for enriching existing fundamentals groups, but it does not justify a broader event/calendar surface yet.
  Date/Author: 2026-05-06 / Codex.

- Decision: If follow-up implementation happens, prefer small additions to existing `earnings` and `dividends` groups before creating a new command.
  Rationale: The strongest evidence is additional field names adjacent to existing groups.
  Date/Author: 2026-05-06 / Codex.

## Outcomes & Retrospective

Added `docs/notes/fundamentals-events-field-evidence-2026-05-06.md` with
public-safe scanner metainfo evidence for current fundamentals groups and
additional earnings/dividend candidates.

No Rust code, CLI surface, JSON payload, or runtime skill change was needed.
The evidence supports keeping `tv events` deferred and, if implementation is
needed later, starting with small additions to the existing `earnings` and
`dividends` field groups.

## Evidence Summary

Read-only evidence commands used normalized `tv scanner metainfo` and `tv fundamentals` output. The tracked note records field names, categories, and types only. It does not include raw endpoint payloads or live response bodies.

Observed confirmed field categories:

- existing `earnings` group fields are still visible through scanner metainfo;
- existing `dividends` group fields are still visible through scanner metainfo;
- additional earnings-adjacent candidates include fiscal-quarter trading-date fields and current-quarter publication/time fields;
- additional dividend-adjacent candidates include amount, frequency, next dividend date, and expected annual dividends fields.

## Plan of Work

1. Add `docs/notes/fundamentals-events-field-evidence-2026-05-06.md` with a public-safe field evidence summary.
2. Update `docs/internal-tradingview-apis.md` to clarify that fundamentals/events remain scanner field bundles, not a complete event calendar.
3. Update `docs/v0.7-roadmap.md` and `docs/plans/README.md` so this evidence slice is the current plan and the lab bars live smoke is archived.
4. Update `CHANGELOG.md` as a docs/research change.
5. Do not change Rust code, CLI options, JSON payloads, or runtime skills unless validation reveals stale wording.

## Validation

Run:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|account-local)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true

Rust tests are not required because this slice is docs/research-only. If Rust code is touched unexpectedly, run the normal Rust baseline.

Result: passed. The hygiene grep reported existing policy wording, archived
validation-command examples, and the new public-safe safety wording only.

## Interfaces and Dependencies

No new CLI interface, Rust API, or dependency is planned.

## Open Questions

- Should the next implementation add small missing fields to existing `earnings` / `dividends` groups? This plan records it as the preferred implementation shape if downstream need appears.
- Should `tv events` exist later? This plan keeps it deferred until source evidence shows a more complete event/calendar surface.
