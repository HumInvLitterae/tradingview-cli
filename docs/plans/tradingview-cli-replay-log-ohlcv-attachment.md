# `tv replay log` OHLCV summary attachment

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`. It is self-contained so a future contributor can continue from this file without needing prior conversation history.

## Purpose / Big Picture

Replay step logs should be able to carry a small amount of selected-chart evidence without becoming a Replay export command or mixing sources implicitly.

The implemented slice adds opt-in OHLCV summary attachment to `tv replay log --steps <N>`. It preserves the existing `replay_step_log.v1` readiness / step / summary JSONL events and adds selected-chart OHLCV summary evidence only when the caller passes `--attach-ohlcv-summary`.

## Progress

- [x] (2026-06-11) Add `--attach-ohlcv-summary` and `--ohlcv-count <N>` to `tv replay log`.
- [x] (2026-06-11) Add additive attachment controls to readiness events.
- [x] (2026-06-11) Add per-step `attachments.ohlcv_summary` payloads with `replay_log_ohlcv_summary_attachment.v1` metadata.
- [x] (2026-06-11) Add attachment counters to summary events.
- [x] (2026-06-11) Update focused tests, docs, packaged agent guidance, runtime skills, roadmap, and changelog.

## Surprises & Discoveries

- Observation: The existing `ops::ohlcv_summary` helper already returns selected-chart summary evidence with chart context and range readback, so this slice does not need a new data source.
  Evidence: `crates/cli/src/ops/market/ohlcv.rs` builds summary payloads from the selected chart's current bars and preserves Desktop-backed read metadata.

## Decision Log

- Decision: Add only OHLCV summary attachment in this slice.
  Rationale: OHLCV summary is structured, bounded, and already public-safe. Screenshot attachment would require file path, naming, and artifact lifecycle decisions, so it remains a follow-up.
  Date/Author: 2026-06-11 / Codex

- Decision: Require `--attach-ohlcv-summary` when `--ohlcv-count` is provided.
  Rationale: A count option that silently does nothing would be easy for agents to misuse. Validation fails before CDP connection when the attachment flag is absent.
  Date/Author: 2026-06-11 / Codex

- Decision: Treat attachment failures as attachment errors, not Replay step failures.
  Rationale: A successful Replay step should remain represented as a step event. The OHLCV read is separate selected-chart evidence and reports its own `status: "error"` plus summary counters.
  Date/Author: 2026-06-11 / Codex

## Outcomes & Retrospective

Implementation completed the narrow attachment workflow. `tv replay log --steps <N> --attach-ohlcv-summary [--ohlcv-count <N>]` now emits the existing Replay step-log JSONL plus opt-in `attachments.ohlcv_summary` on each successful step.

The attachment payload uses `contract_version: "replay_log_ohlcv_summary_attachment.v1"`, `source: "selected_chart_cdp"`, `source_category: "desktop_backed_read"`, `requires_desktop: true`, and `non_mutating: true`. Attachment failures are public-safe item errors and do not stop the step loop.

## Context and Orientation

`tv replay log` remains a Desktop-backed Replay operation. It advances an already-started Replay session and does not start or stop Replay automatically.

The OHLCV attachment reads the selected chart after each successful Replay step. It is explicit evidence attached to the step log, not a replacement for `tv bars --from/--to`, not a stable Replay export, and not automatic source mixing.

## Plan of Work

First, extend the CLI parser and JSONL runner with explicit attachment controls and validation. Keep `tv replay log --steps <N>` behavior unchanged when no attachment flag is passed.

Second, reuse `ops::ohlcv_summary` after each successful Replay step when `--attach-ohlcv-summary` is enabled. Wrap the result in a small attachment payload with source metadata and sanitized failure details.

Third, update docs and runtime skills so agents know when to request attached OHLCV summary evidence and when to keep Replay, Desktop-free bars, chart export, and screenshots separate.

## Validation and Acceptance

The implementation is acceptable when attachment-free logs preserve the existing contract, attachment-enabled logs include readiness controls, step attachments, and summary counters, and attachment failures do not become Replay step failures.

## Artifacts and Notes

Do not paste raw JSONL output, raw bars, raw DOM, raw payloads, target ids, account-local metadata, credentials, or local absolute paths into tracked docs. Optional live smoke evidence may be summarized with command name, contract marker, step count, attachment counts, and end reason only.

## Interfaces and Dependencies

This plan adds no dependency and no version bump. It adds only CLI options and additive JSONL fields.

## Open Questions

Screenshot attachment remains a future candidate. It needs an explicit file path / artifact naming contract before implementation.
