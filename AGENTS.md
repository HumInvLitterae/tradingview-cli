# Agent Operating Guide

This file is the root operating guide for coding agents in this repository.

## Mission

Build and maintain a Rust-native TradingView CLI that replaces the currently used TradingView bridge path in sibling trading-analysis projects.

The repository now contains the first Rust-native `tv` CLI implementation. The immediate goal is to keep that narrow v1 surface reliable, document the real operating contract, and choose any post-v1 work only after evidence shows it belongs in the core CLI.

## Sources of Truth

Read these in order before making major decisions:

1. `CONTINUITY.md`
2. `README.md`
3. `docs/notes/next-agent-handoff-prompt-2026-04-24.md`
4. `docs/plans/tradingview-cli-rust-v1.md`
5. `docs/notes/tradingview-mcp-investigation-2026-04-24.md`
6. `docs/plans/tradingview-cli-bootstrap-and-bridge-replacement.md`
7. `docs/notes/next-agent-handoff-prompt-2026-04-21.md`
8. `.agents/PLANS.md`
9. `.agents/skills/continuity/SKILL.md` when the continuity skill is active

If these sources disagree, preserve the higher-level user and system instructions, then update repository docs so the durable project state is clear again.

## Current Status

The repository currently contains a working Rust v1 CLI plus the planning and investigation history that explains its boundary.

What is true right now:

- Rust v1 is implemented as a `tv` binary
- v1 is CLI-first
- MCP server implementation is not planned
- the first capability and boundary research milestone is complete
- the first Rust v1 implementation milestone is complete
- this repository should stay narrower than a full reimplementation of the old bridge

## Near-Term Deliverables

Prefer work that moves one of these forward:

1. keeping `README.md`, handoff notes, and ExecPlans aligned with the implemented CLI
2. validating the Rust CLI in real downstream provider, review, and operator workflows
3. preserving the v1 CLI boundary unless a decision is recorded in a new ExecPlan
4. investigating post-v1 candidates such as chart-region screenshots, launch automation, or additional read-only commands before implementing them

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
