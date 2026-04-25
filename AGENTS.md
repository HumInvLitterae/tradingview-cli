# Agent Operating Guide

This file is the root operating guide for coding agents in this repository.

## Mission

Build and maintain a Rust-native TradingView CLI that replaces the currently used TradingView bridge path in sibling trading-analysis projects.

The repository now contains the first Rust-native `tv` CLI implementation. The immediate goal is to keep that narrow v1 surface reliable, document the real operating contract, and choose any post-v1 work only after evidence shows it belongs in the core CLI.

The known old JavaScript CLI command migration is now closed. If new evidence shows a missed old CLI command, treat it as migration backlog unless a durable project decision explicitly excludes it. The MCP server remains separate from that backlog and is not planned.

## Sources of Truth

Read these in order before making major decisions:

1. `CONTINUITY.md`
2. `README.md`
3. `docs/notes/development-guidelines-2026-04-24.md`
4. `docs/notes/next-agent-handoff-prompt-2026-04-24.md`
5. `docs/notes/rust-cli-contract-migration-2026-04-24.md`
6. `docs/notes/legacy-cli-command-migration-inventory-2026-04-24.md`
7. `docs/notes/remaining-deferred-surface-audit-2026-04-25.md`
8. `docs/plans/README.md`
9. `docs/notes/tradingview-mcp-investigation-2026-04-24.md`
10. `.agents/PLANS.md`
11. `.agents/skills/continuity/SKILL.md` when the continuity skill is active

Historical ExecPlans live under `docs/plans/archives/`. Read them only when you need slice-level implementation history or rationale that is not already summarized in the current notes.

If these sources disagree, preserve the higher-level user and system instructions, then update repository docs so the durable project state is clear again.

## Current Status

The repository currently contains a working Rust v1 CLI plus the planning and investigation history that explains its boundary.

What is true right now:

- Rust v1 is implemented as a `tv` binary
- v1 is CLI-first
- MCP server implementation is not planned
- the Rust JSON wire shape intentionally differs from the old JavaScript CLI
- migrated commands must preserve the practical information available from the old CLI
- newly discovered old CLI commands remain migration backlog unless explicitly excluded
- the first capability and boundary research milestone is complete
- the first Rust v1 implementation milestone is complete
- the first read/provider migration slice is complete
- the read utilities migration slice is complete
- the chart-region screenshot slice is complete
- the diagnostic read commands slice is complete
- the advanced data reads slice is complete
- the chart type slice is complete
- the DOM-dependent data depth read slice is complete
- the read-only alert list slice is complete
- the watchlist add operator mutation slice is complete
- the watchlist remove operator cleanup slice is complete
- the alert create slice is complete
- the pane mutation slice is complete
- the alert delete slice is complete
- the saved layout list and switch slice is complete; switching supports `--dry-run`
- the indicator command lifecycle slice is complete
- the drawing command lifecycle slice is complete, including bulk `draw clear --dry-run` preflight and post-clear verification
- the Pine read, source set, compile, raw-compile compatibility, analyze, check, new, open, and save slices are complete
- the tab command lifecycle slice is complete with explicit-index app-tab close safety
- the replay command lifecycle slice is complete
- the read-only stream command slice is complete and emits newline-delimited JSON envelopes
- the bounded launch command slice is complete with no-kill default behavior
- the remaining old CLI migration closure slice is complete; `layout switch`, `alert delete --all`, `pine raw-compile`, and generic `ui` commands are implemented
- the operation layer has been split from one oversized `src/ops.rs` into a thin facade plus feature modules under `src/ops/`
- the data operation layer has been split from one large `src/ops/data.rs` into a thin facade plus capability modules under `src/ops/data/`
- repo-local development guidelines now record module layout, style, contract, and validation rules for future work
- the original MCP workflow skills have been migrated into repo-local CLI skills with current capability gaps marked
- this repository should stay narrower than a full reimplementation of the old bridge

## Near-Term Deliverables

Prefer work that moves one of these forward:

1. keeping `README.md`, handoff notes, and ExecPlans aligned with the implemented CLI
2. validating the Rust CLI in real downstream provider, review, and operator workflows
3. release readiness, including packaging, distribution, and public-facing documentation
4. checking original upstream pull requests for fixes or additions after release readiness is in good shape

Supporting notes are welcome when they reduce ambiguity, but avoid speculative design sprawl.

## Execution Rules

1. Use ExecPlans for complex features or significant refactors, and maintain them exactly as required by `.agents/PLANS.md`.
2. Treat tracked repository docs as the durable memory for this project. If a decision matters later, record it in `docs/` or `CONTINUITY.md`.
3. Do not start implementation just because a capability exists in the old bridge. First justify why it belongs in the new CLI.
4. When inspecting external or sibling repositories, summarize the relevant findings in this repository. Do not depend on private local memory.
5. Never write machine-specific absolute filesystem paths into tracked repository files.
6. Mark uncertainty as `UNCONFIRMED` instead of guessing.
7. Keep the repo boundary clean. Downstream workflow helpers, skills, and adapters should stay outside the core CLI unless investigation proves they belong in the Rust CLI migration surface.
8. Do not describe unimplemented old CLI commands as non-goals unless a project decision explicitly excludes them.
9. Preserve information compatibility for migrated commands. Field names and envelope shape may change, but practical information available from the old CLI must remain available in the Rust CLI.
10. Commit related changes in sensible batches when files are changed. Do not accumulate a large mixed set of unrelated edits.
11. Never push to a remote unless the user explicitly asks in the current turn.

## Documentation Policy

1. Keep planning documents under `docs/plans/`.
   Historical completed plans belong under `docs/plans/archives/`.
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
- `docs/notes/development-guidelines-2026-04-24.md`: module layout, coding style, and validation guide
- `src/ops.rs`: thin operation facade that re-exports feature modules
- `src/ops/`: operation implementations grouped by capability
- `src/ops/indicator.rs`: indicator add/remove/toggle/set operation implementation
- `src/ops/drawing.rs`: drawing shape/list/get/remove/clear operation implementation
- `src/ops/pine.rs`: thin Pine operation facade
- `src/ops/pine/`: Pine Editor source/template/open, static analysis, and server-side check implementations
- `src/ops/tab.rs`: tab list/switch/new/close operation implementation
- `src/ops/replay.rs`: replay start/step/stop/status/autoplay/trade operation implementation
- `src/ops/stream.rs`: read-only JSONL stream operation implementation
- `src/ops/launch.rs`: bounded TradingView Desktop launch operation implementation
- `src/ops/saved_layout.rs`: saved chart layout list operation implementation
- `src/ops/ui.rs`: generic UI automation compatibility command implementation
- `src/ops/data.rs`: thin data operation facade
- `src/ops/data/`: data operation implementations grouped by indicator, strategy, and drawing-derived reads
- `docs/plans/`: active plan index and current ExecPlans
- `docs/plans/archives/`: completed historical ExecPlans
- `docs/notes/`: handoff notes, research notes, inventories
- `.agents/PLANS.md`: ExecPlan standard used by this repository
- `.agents/skills/`: repo-local skills and workflow helpers
