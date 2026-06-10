# `tv events compare` readback

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`. It is self-contained so a future contributor can continue from this file without needing prior conversation history.

## Purpose / Big Picture

The user should be able to inspect earnings and dividend event-shaped evidence for a small candidate set without treating the result as a full event calendar, ranking, or trading recommendation.

The implemented command shape is `tv events compare <SYMBOL>...`. It preserves the existing single-symbol `tv events <SYMBOL>` readback and adds a separate `events_compare.v1` payload for ordered multi-symbol event evidence.

## Progress

- [x] (2026-06-11) Add `events_compare.v1` payload shaping in the market crate.
- [x] (2026-06-11) Add `tv events compare <SYMBOL>...` while preserving `tv events <SYMBOL>`.
- [x] (2026-06-11) Add focused tests for help, validation, summary counts, and public-safe failure details.
- [x] (2026-06-11) Update docs, packaged agent guidance, runtime skills, roadmap, and changelog.

## Surprises & Discoveries

- Observation: The existing `tv events <SYMBOL>` implementation already exposes the right scanner-backed `events.v1` source boundary, so the multi-symbol command can reuse it directly.
  Evidence: `crates/market/src/events.rs` shapes earnings and dividends from scanner fundamentals fields without requiring TradingView Desktop.

## Decision Log

- Decision: Use `tv events compare <SYMBOL>...` rather than adding event fields to `tv compare`.
  Rationale: `tv compare` is a scanner quote / info / fundamentals evidence packet, while `events compare` is specifically about event-shaped earnings and dividend readback. A separate subcommand keeps the workflow clear.
  Date/Author: 2026-06-11 / Codex

- Decision: Keep `events_compare.v1` separate from `events.v1`.
  Rationale: The single-symbol payload is already stable and useful. The multi-symbol payload needs ordered item status and summary counts, so it should be additive instead of reshaping `events.v1`.
  Date/Author: 2026-06-11 / Codex

- Decision: Do not add date range filtering or a standalone calendar source in this slice.
  Rationale: The current source is scanner fundamentals fields, not a full event calendar. Range semantics would invite false precision unless a separate calendar source is proven later.
  Date/Author: 2026-06-11 / Codex

## Outcomes & Retrospective

Implementation completed the narrow multi-symbol readback. Users can run `tv events compare <SYMBOL>...` for 2 to 25 symbols and optionally pass `--event-type all|earnings|dividends`. The payload reports `contract_version: "events_compare.v1"`, ordered item status, single-symbol `events.v1` payloads for successful items, public-safe item errors, and summary counts.

## Context and Orientation

`tv events <SYMBOL>` remains a Desktop-free single-symbol readback sourced from scanner fundamentals fields. It is not a full event calendar and does not infer timezone, before/after-market meaning, confirmation status, ranking, recommendations, or trading judgment.

`tv events compare <SYMBOL>...` reuses the same source for several symbols. It does not call `tv fundamentals`, `tv compare`, `tv watch compare`, `tv chart compare`, `tv bars`, chart reads, Replay, or any calendar source as hidden fallback.

## Plan of Work

First, add a market-level multi-symbol wrapper around the existing typed `events_symbol` readback. The wrapper should preserve input order, return item-level success or public-safe error, and summarize event counts across successful items.

Second, update the CLI so `tv events <SYMBOL>` remains valid while `tv events compare <SYMBOL>...` becomes available. Validate 2 to 25 symbols and empty inputs before any network read.

Third, update public docs and runtime skills so agents choose `tv events compare` for candidate-set event evidence, not for ranking or a full calendar.

## Validation and Acceptance

The implementation is acceptable when `tv events compare <SYMBOL>...` returns `events_compare.v1`, keeps `events.v1` single-symbol behavior intact, reports ordered item status and summary counts, and never silently falls back to chart, bars, compare, Replay, or calendar sources.

## Artifacts and Notes

Do not paste raw scanner payloads, credentials, session ids, target ids, account-local metadata, or local absolute paths into tracked docs. Optional live smoke evidence may be summarized with command name, source marker, requested count, ok/error count, and total event count only.

## Interfaces and Dependencies

This plan adds no dependency and no version bump. It uses the existing scanner fundamentals source and adds only additive command / payload behavior.

## Open Questions

No blocker remains for this slice. Future work can investigate calendar-range event reads only if a separate source is identified and its semantics can be represented without pretending scanner fields are a full calendar.
