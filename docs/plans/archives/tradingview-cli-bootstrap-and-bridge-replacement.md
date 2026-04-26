# TradingView CLI bootstrap and bridge-replacement plan

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, the repository will have a clean, decision-oriented foundation for building a Rust-native TradingView CLI instead of continuing to rely on an aging JavaScript bridge implementation by inertia. The immediate user-visible outcome is not a working CLI yet. The immediate outcome is a complete, restartable plan that tells the next contributor exactly how to investigate the existing bridge, define the minimum replacement boundary, and choose the first Rust CLI surface without dragging those design decisions back into another project.

## Progress

- [x] (2026-04-24 03:45 JST) Inspect the current upstream bridge repository and write down the capability inventory actually needed for CLI-first use.
- [x] (2026-04-24 03:45 JST) Record the known pain points, bugs, and maintenance risks that justify replacement.
- [x] (2026-04-24 03:45 JST) Define the minimum CLI boundary for v1.
- [x] (2026-04-24 03:45 JST) Decide repository boundaries, release posture, and relationship to sibling consumer projects.
- [x] (2026-04-24 03:45 JST) Produce the first implementation-ready successor plan after the investigation.

## Surprises & Discoveries

- Observation: This repository is intentionally being initialized in docs-seed mode before any implementation starts.
  Evidence: The initial commit contains only `README.md`, this ExecPlan, and a handoff prompt.

- Observation: The migration source exposes both an MCP server and a broad `tv` CLI, but this repository should carry forward only the CLI-first path.
  Evidence: `docs/notes/tradingview-mcp-investigation-2026-04-24.md` records the migration source package structure, registered CLI command groups, and CDP connection model.

- Observation: Full screenshot support is a reasonable v1 candidate, while chart-region screenshot support needs a later stability spike.
  Evidence: The migration source uses CDP `Page.captureScreenshot` directly for full screenshots but uses DOM selectors and clip rectangles for chart-region screenshots.

- Observation: Testability must be a first-class design constraint in the Rust plan.
  Evidence: The local migration source had uncommitted dependency-injection changes for chart operations, and `node --test tests/sanitization.test.js` passed 69 tests including injected-evaluator coverage.

## Decision Log

- Decision: Start as a separate repository rather than keeping this work inside a larger backtesting/tooling repo.
  Rationale: The replacement has its own release surface, lifecycle, and public-facing boundary. Keeping it separate avoids entangling bridge implementation work with trading-analysis roadmap decisions.
  Date/Author: 2026-04-21 / Codex

- Decision: Treat the first milestone as capability and boundary research rather than as immediate reimplementation.
  Rationale: The replacement should not inherit the old surface blindly. The first step must identify which capabilities are actually needed and which ones should be left behind.
  Date/Author: 2026-04-21 / Codex

- Decision: Keep v1 CLI-first and do not promise MCP parity.
  Rationale: The motivating problem is practical command-line use and reliability. Full MCP compatibility would enlarge scope before the narrower operational replacement is proven.
  Date/Author: 2026-04-21 / Codex

- Decision: Do not plan an MCP server implementation for this repository.
  Rationale: The replacement target is a Rust-native CLI. Downstream integration should use ordinary process invocation and JSON CLI output. Recreating the original MCP server would compete with the narrower operational goal and has lower priority than post-v1 CLI capabilities.
  Date/Author: 2026-04-24 / Codex

- Decision: Use `docs/plans/archives/tradingview-cli-rust-initial-implementation.md` as the first implementation-ready successor ExecPlan.
  Rationale: The bootstrap plan should remain the seed and investigation record, while the successor plan should be the place where coding begins.
  Date/Author: 2026-04-24 / Codex

## Outcomes & Retrospective

The bootstrap phase completed on 2026-04-24. The repository now has attribution to the migration source in `README.md`, a migration-source investigation note at `docs/notes/tradingview-mcp-investigation-2026-04-24.md`, and a first implementation-ready successor ExecPlan at `docs/plans/archives/tradingview-cli-rust-initial-implementation.md`.

The main lesson from this bootstrap phase is that the old bridge's breadth is evidence for narrowing, not for feature parity. The Rust v1 should preserve the useful `tv` CLI name and JSON command-line posture, but it should not recreate the MCP server and should not inherit Pine, pane, replay, stream, UI automation, or chart-region screenshot complexity before the core CLI proves reliable.

## Context and Orientation

The motivating background is simple.

A sibling trading-analysis repository currently depends on a TradingView MCP Bridge path that appears to have real maintenance and usability limits. The replacement goal is not “rebuild everything because Rust is nicer.” The goal is to produce a narrower, cleaner CLI that can own the capabilities actually needed by downstream operator and analysis workflows.

In this plan, “bridge replacement” means:

- understanding the current bridge’s useful capability surface
- identifying the painful or unreliable parts that justify replacement
- deciding which minimal subset should become the first Rust CLI

This plan is intentionally self-contained. The next contributor should not need access to any absolute local paths or private notes to understand the mission. If outside repositories are inspected during execution, their relevant findings must be summarized directly inside this document.

## Plan of Work

First, inspect the current upstream bridge implementation and summarize it in plain language. The investigation must answer:

- how it connects to TradingView Desktop
- which command surfaces already exist
- which capabilities are exposed via MCP versus ordinary CLI behavior
- which bugs or operational defects are already known from practical use

Second, separate “needed capabilities” from “historical baggage.” The next contributor should not assume that every old command or MCP endpoint deserves a Rust equivalent. They should identify the smallest operationally useful slice for v1. At minimum, that analysis should distinguish:

- desktop/session connectivity
- chart or tab discovery
- bounded command execution
- provider-style data access needs
- operator workflow helpers that should remain outside the core CLI

Third, define the v1 CLI boundary. This must be concrete enough that a future implementer knows what commands belong in the first executable milestone and what explicitly does not. The desired outcome is a short command surface with clear success criteria, not a speculative kitchen sink.

Fourth, define repository and release posture. The next contributor should decide:

- whether the crate starts as a single binary crate
- whether public release is planned from the start or later
- how downstream repos should consume it during early development

Fifth, convert the investigation into the first implementation-ready ExecPlan. That successor plan should be the point where coding can begin. This bootstrap plan is complete when that handoff exists.

## Concrete Steps

All commands below should run from this repository root unless the next contributor is explicitly inspecting an upstream repository.

Start by reading this file and the handoff note:

    sed -n '1,240p' README.md
    sed -n '1,260p' docs/notes/archives/next-agent-handoff-prompt-2026-04-21.md

Then inspect the current upstream bridge repository and summarize findings directly into this plan before designing any Rust surface. The exact clone location is intentionally not hard-coded here; the contributor should use their local environment and record only the relevant findings, not machine-specific paths.

Expected successful output for this bootstrap phase is not a binary. It is an updated ExecPlan with a concrete capability inventory, a v1 command boundary, and a replacement-ready next milestone.

This bootstrap phase has produced those outputs. Continue with:

    sed -n '1,260p' docs/notes/tradingview-mcp-investigation-2026-04-24.md
    sed -n '1,320p' docs/plans/archives/tradingview-cli-rust-initial-implementation.md

## Validation and Acceptance

Acceptance is document-driven for this bootstrap phase.

This seed is successful when:

1. a novice can open this repository and understand why it exists,
2. the next contributor can start investigation without needing another repository’s planning context,
3. the first follow-up ExecPlan can be written from investigation results without reopening the separate-repo decision,
4. the bootstrap docs do not leak machine-specific local paths or private notes.

## Idempotence and Recovery

This phase is safe to repeat. It is documentation-first and does not yet own any implementation state. If the next contributor changes their mind about command names or crate structure, they should update this plan and the handoff note rather than creating parallel planning files that compete with each other.

## Artifacts and Notes

The first useful artifacts should be:

- a plain-language capability inventory
- a bug/risk inventory for the existing bridge
- a short v1 command list
- a successor ExecPlan that begins implementation

Keep all external findings summarized here or in a sibling repository note. Do not rely on unstated local memory.

The completed artifacts are:

- `docs/notes/tradingview-mcp-investigation-2026-04-24.md`
- `docs/plans/archives/tradingview-cli-rust-initial-implementation.md`
- updated `README.md` attribution to the migration source

## Interfaces and Dependencies

The first implementation-ready plan should strongly prefer:

- Rust stable toolchain
- one binary crate for the initial CLI
- additive downstream integration through ordinary process invocation before any plugin or MCP layering

Do not implement an MCP server for this project unless a future plan explicitly reopens that decision with new evidence. It is not a v1 feature and is not a planned post-v1 target.

## Open Questions

The next contributor must resolve these during investigation:

- Resolved: Which existing bridge capabilities are truly required for v1?
- Resolved: Which known bugs are severe enough to define the replacement boundary?
- Resolved: Should the first Rust CLI own only session/control primitives, or should it also expose provider-friendly data reads in v1?
- Resolved: Which parts should remain in downstream skills or consumer repositories rather than in the core CLI?

Remaining non-blocking questions are captured in `docs/plans/archives/tradingview-cli-rust-initial-implementation.md`.

Revision note: created as the initial seed plan for a separate Rust-native TradingView CLI repository, intentionally before any implementation begins.

Revision note: updated on 2026-04-24 after migration-source investigation. The bootstrap phase now points to the completed investigation note and the first Rust v1 successor ExecPlan.
