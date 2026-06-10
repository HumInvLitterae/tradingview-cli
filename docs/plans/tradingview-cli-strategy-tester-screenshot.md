# Strategy Tester screenshot evidence

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`. It is self-contained so a future contributor can continue from this file without needing prior conversation history.

## Purpose / Big Picture

Strategy reports sometimes need visual evidence of the TradingView Strategy Tester panel, not only structured strategy metrics or a chart screenshot.

This slice adds `tv screenshot --region strategy --output <PATH>` as a narrow Desktop-backed visual evidence read. It clips to the visible Strategy Tester panel when the panel can be detected, writes the requested file, and returns source-labeled screenshot metadata. It does not open the panel, run a strategy, export bars, infer strategy metrics, or change chart state.

## Progress

- [x] (2026-06-11) Add Strategy Tester panel clipping to `tv screenshot`.
- [x] (2026-06-11) Add CLI validation and help for `--region strategy`.
- [x] (2026-06-11) Add focused screenshot and CLI contract tests.
- [x] (2026-06-11) Update roadmap, changelog, docs, packaged agent guide, and runtime skills.

## Surprises & Discoveries

- Observation: The existing chart screenshot path already had the right shape for bounded visual evidence: evaluate bounds, attempt CDP clipping, and fall back to local crop when clipped capture is unavailable.
  Evidence: `crates/cli/src/ops/screenshot.rs` already provided full and chart screenshot metadata with `desktop_screenshot` source labels.

## Decision Log

- Decision: Add `strategy` as a third `--region` value instead of a new command.
  Rationale: The behavior is still a screenshot read that writes a local file. Keeping it under `tv screenshot` avoids a broader strategy-report command surface.
  Date/Author: 2026-06-11 / Codex

- Decision: Mark the payload with `evidence_role: "strategy_tester_panel"`.
  Rationale: Downstream agents should not have to infer from the region alone that the image is intended as Strategy Tester panel evidence.
  Date/Author: 2026-06-11 / Codex

- Decision: Do not open or manipulate the Strategy Tester panel automatically.
  Rationale: The command is a non-mutating Desktop-backed read. If the panel is not visible, failure details should explain the next action instead of changing UI state.
  Date/Author: 2026-06-11 / Codex

## Outcomes & Retrospective

Implementation completed the narrow screenshot read. `tv screenshot --region strategy --output <PATH>` now attempts to locate a visible Strategy Tester / backtesting panel, captures that clipped region when possible, and writes the requested image file.

The payload keeps the existing `desktop_screenshot` metadata and adds `region: "strategy"` plus `evidence_role: "strategy_tester_panel"`. Failure details remain public-safe and direct users to open the Strategy Tester panel before retrying.

## Context and Orientation

`tv data strategy`, `tv data trades`, and `tv data equity` remain the structured Strategy Tester evidence commands. `tv screenshot --region strategy` is visual evidence only.

Use this command when a report needs an image of the Strategy Tester panel as displayed in TradingView Desktop. Use `tv screenshot --region chart` for chart visual context and `tv screenshot --region full` when the entire selected target viewport matters.

## Plan of Work

First, refactor the existing chart screenshot code into a reusable clipped screenshot helper.

Second, add a Strategy Tester bounds expression that looks for visible backtesting / Strategy Tester panel elements and returns viewport-bounded clipping coordinates.

Third, wire `strategy` through CLI validation, dispatch, and help text.

Fourth, update docs and runtime skills so strategy-report workflows can request Strategy Tester panel visual evidence without implying strategy metrics, export, ranking, or recommendation semantics.

## Validation and Acceptance

The implementation is acceptable when `full` and `chart` screenshot behavior is unchanged, `strategy` is accepted and attempts CDP connection, unsupported regions still fail before connection, the payload is source-labeled and public-safe, and focused tests plus baseline checks pass.

## Artifacts and Notes

Do not paste screenshot images, raw DOM, raw payloads, target ids, account-local metadata, credentials, or local absolute paths into tracked docs. Optional live smoke evidence may be summarized with command name, region, source marker, file existence, and image dimensions only.

## Interfaces and Dependencies

This plan adds no dependency and no version bump. It adds one supported value to an existing CLI option and additive screenshot metadata.

## Open Questions

Future strategy-report work may still need richer Strategy Tester table extraction or equity-curve evidence. Those should be handled as separate source-boundary plans, not folded into screenshot capture.
