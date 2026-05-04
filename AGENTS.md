# Agent Operating Guide

This file is the contributor-facing operating guide for coding agents working
inside this repository. Release archives use `packaging/agent/AGENTS.md`
instead; that file is for runtime users and their agents.

## Mission

Build and maintain `tv`, a Rust-native TradingView CLI for Desktop-backed
TradingView automation and Desktop-free TradingView data reads.

The project is CLI-first. It is not an MCP server, and MCP server
implementation is not planned. The known old JavaScript CLI command migration
is closed; newly discovered old CLI commands are migration backlog unless a
durable project decision explicitly excludes them.

## Sources of Truth

Read current sources before historical notes:

1. `CONTINUITY.md`
2. `README.md`
3. `docs/v0.6-roadmap.md`
4. `docs/command-source-taxonomy.md`
5. `docs/architecture.md`
6. `docs/development.md`
7. `docs/rust-api.md`
8. `docs/release-packaging.md`
9. `docs/breaking-changes-from-js-cli.md`
10. `docs/internal-tradingview-apis.md`
11. `docs/plans/README.md`
12. `.agents/PLANS.md`
13. `.agents/skills/continuity/SKILL.md` when the continuity skill is active

Older roadmap files, migration inventories, upstream PR triage notes, handoff
prompts, and archived ExecPlans are historical context. Read them only when a
current source points there or when you need the rationale for an older slice.

If sources disagree, preserve higher-priority system and user instructions,
then update repository docs so the durable state is clear again.

## Current Status

The repository contains a working `tv` binary in a Cargo workspace.

What is true now:

- `tv` remains a single binary; command behavior is explained by source
  taxonomy rather than by splitting Desktop-free and Desktop-backed binaries.
- Desktop-free reads live primarily in `tradingview-market`,
  `tradingview-scanner`, and `tradingview-pine`.
- Desktop-backed reads and operations use `tradingview-cdp` and operation
  adapters under `crates/cli/src/ops/`.
- Shared JSON envelope and error contracts live in `tradingview-core`.
- I/O-free validation, request interpretation, target resolution, and payload
  shaping live in `tradingview-model`.
- The CLI package lives under `crates/cli/`; the repository root is a virtual
  Cargo workspace.
- Release archives include the binary, public docs, user-facing agent guides,
  and runtime skills. Development-only skills are intentionally excluded.
- The Rust JSON wire shape intentionally differs from the old JavaScript CLI,
  but migrated commands should preserve practical information.
- MCP server implementation, cookie/session import/export, trading bots, and
  broad generic UI expansion are not planned by default.

## Work Rules

1. Use ExecPlans for complex features or significant refactors, following
   `.agents/PLANS.md`.
2. Treat tracked repository docs as durable project memory. If a decision
   matters later, record it in `docs/` or `CONTINUITY.md`.
3. Do not add a command only because it existed in the old bridge. First record
   the workflow value, safety boundary, compatibility expectation, and tests.
4. Preserve information compatibility for migrated commands. Field names and
   envelope shape may differ, but practical information should remain
   available unless a migration note says otherwise.
5. Never write machine-specific absolute paths, live account-local identifiers,
   raw target ids, cookies, tokens, authorization values, or raw live payloads
   into tracked repository files.
6. Mark uncertainty as `UNCONFIRMED` instead of guessing.
7. Keep repo boundaries clean. Downstream workflow helpers should stay outside
   the core CLI unless investigation proves they belong here.
8. Commit related changes in sensible batches. Do not accumulate unrelated
   edits into one large mixed commit.
9. Never push to a remote unless the user explicitly asks in the current turn.

## Documentation Policy

- `README.md` is a human-facing public overview.
- This file and `CLAUDE.md` are contributor-facing agent guides and should stay
  identical.
- `packaging/agent/AGENTS.md` is the runtime guide copied into release
  archives as both `AGENTS.md` and `CLAUDE.md`.
- Stable architecture, development, release packaging, migration, API, and
  internal dependency references live directly under `docs/`.
- Current and future plans live under `docs/plans/`; completed plans live under
  `docs/plans/archives/`.
- Research notes, inventories, and handoff material live under `docs/notes/`.
- Prefer English for repository docs unless an existing file is intentionally
  Japanese.

## Change Strategy

When new work starts, ask:

1. Is this investigation, boundary definition, implementation, or release prep?
2. What observable outcome should exist after this step?
3. Which facts are known, and which are inferred?
4. What is the smallest durable doc update that keeps the next contributor
   unblocked?

For release work, stop feature changes first. Version bumps, changelog edits,
release notes, packaging checks, and CI triage should not be mixed with new
feature implementation.

## Repository Structure

- `crates/cli/`: `tradingview-cli` package and `tv` binary.
- `crates/core/`: shared errors, envelopes, and exit-code mapping.
- `crates/model/`: I/O-free validation, request models, target resolution, and
  payload shaping.
- `crates/market/`: Desktop-free market reads such as search, info, quote,
  batch quotes, and fundamentals.
- `crates/scanner/`: Desktop-free scanner reads such as scan, hotlist, and
  metainfo.
- `crates/pine/`: Desktop-free Pine static analysis and facade checks.
- `crates/cdp/`: TradingView Desktop CDP transport, target discovery, runtime
  evaluation, screenshots, and input primitives.
- `docs/`: stable docs, roadmaps, notes, release notes, and ExecPlans.
- `.agents/skills/`: repo-local development and runtime skills.
- `packaging/agent/`: release-archive agent guide source.
- `scripts/`: release packaging and optional local helper scripts.
