# Agent Operating Guide

This file is the root operating guide for coding agents in this repository.

## Mission

Build the groundwork for a Rust-native TradingView CLI that replaces the currently used TradingView bridge path in sibling trading-analysis projects.

The repository is intentionally in docs-seed mode. The immediate goal is not implementation. The immediate goal is to understand the current bridge surface, identify the narrowest useful replacement boundary, and produce the next implementation-ready ExecPlan without dragging planning context back into another repository.

## Sources of Truth

Read these in order before making major decisions:

1. `CONTINUITY.md`
2. `README.md`
3. `docs/plans/tradingview-cli-bootstrap-and-bridge-replacement.md`
4. `docs/notes/tradingview-mcp-investigation-2026-04-24.md`
5. `docs/plans/tradingview-cli-rust-v1.md`
6. `docs/notes/next-agent-handoff-prompt-2026-04-21.md`
7. `.agents/PLANS.md`
8. `.agents/skills/continuity/SKILL.md` when the continuity skill is active

If these sources disagree, preserve the higher-level user and system instructions, then update repository docs so the durable project state is clear again.

## Current Status

The repository currently exists to hold planning, investigation results, and successor execution plans.

What is true right now:

- there is no Rust implementation yet
- v1 is expected to be CLI-first
- MCP server implementation is not planned
- the first capability and boundary research milestone is complete
- this repository should stay narrower than a full reimplementation of the old bridge

## Near-Term Deliverables

Until Rust v1 is implemented, prefer work that moves one of these forward:

1. implementing `docs/plans/tradingview-cli-rust-v1.md`
2. keeping `docs/notes/tradingview-mcp-investigation-2026-04-24.md` accurate when new facts are discovered
3. preserving the v1 CLI boundary unless a decision is recorded in the ExecPlan
4. validating the Rust CLI with the commands required by the ExecPlan

Supporting notes are welcome when they reduce ambiguity, but avoid speculative design sprawl.

## Execution Rules

1. Use ExecPlans for complex features or significant refactors, and maintain them exactly as required by `.agents/PLANS.md`.
2. Treat tracked repository docs as the durable memory for this project. If a decision matters later, record it in `docs/` or `CONTINUITY.md`.
3. Do not start implementation just because a capability exists in the old bridge. First justify why it belongs in the new CLI.
4. When inspecting external or sibling repositories, summarize the relevant findings in this repository. Do not depend on private local memory.
5. Never write machine-specific absolute filesystem paths into tracked repository files.
6. Mark uncertainty as `UNCONFIRMED` instead of guessing.
7. Keep the repo boundary clean. Downstream workflow helpers, skills, and adapters should stay outside the core CLI unless the investigation proves they belong in v1.
8. Commit related changes in sensible batches when files are changed. Do not accumulate a large mixed set of unrelated edits.
9. Never push to a remote unless the user explicitly asks in the current turn.

## Documentation Policy

1. Keep planning documents under `docs/plans/`.
2. Keep research notes, inventories, and handoff material under `docs/notes/`.
3. Keep agent-only workflow standards under `.agents/`.
4. Prefer English for agent-facing repository documents unless an existing file is intentionally Japanese.
5. User-facing responses should remain concise Japanese unless the user asks otherwise.

## Change Strategy

When new work starts, ask:

1. Is this investigation, boundary definition, or implementation?
2. What observable outcome should exist after this step?
3. Which facts are known, and which are only inferred?
4. What is the smallest durable doc update that keeps the next contributor unblocked?

If the answer still depends on unresolved bridge facts, investigate first and write the evidence down before designing further.

## Repository Structure

- `README.md`: project overview and current status
- `CONTINUITY.md`: compaction-safe continuity ledger for current durable state
- `docs/plans/`: bootstrap and successor ExecPlans
- `docs/notes/`: handoff notes, research notes, inventories
- `.agents/PLANS.md`: ExecPlan standard used by this repository
- `.agents/skills/`: repo-local skills and workflow helpers
