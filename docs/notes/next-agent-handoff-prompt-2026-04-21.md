# Next agent handoff prompt 2026-04-21

Use this repository as the starting point for a Rust-native TradingView CLI project.

## Mission

Investigate and define the first practical Rust CLI replacement for the currently used TradingView bridge path. Do not start by reimplementing the old project wholesale. First determine what capabilities are actually needed, what is broken or operationally painful today, and what the narrowest useful v1 CLI surface should be.

## What has already been decided

- this work belongs in a separate repository
- this repository starts in docs-seed mode
- v1 should be CLI-first
- full MCP parity is not a default goal
- the first milestone is investigation and boundary definition, not implementation

## Your first tasks

1. Read `README.md`
2. Read `docs/plans/tradingview-cli-bootstrap-and-bridge-replacement.md`
3. Read `docs/notes/tradingview-mcp-investigation-2026-04-24.md`
4. Read `docs/plans/tradingview-cli-rust-v1.md`
5. Continue from the Rust v1 ExecPlan unless the user explicitly redirects the work

## Constraints

- do not write machine-specific absolute paths into tracked docs
- do not assume every old capability deserves a replacement
- do not promise release packaging or public API stability yet
- do not bloat v1 with downstream workflow helpers that can live in consumer repos
- do not implement an MCP server; this project is planned as a CLI-first replacement, and MCP server implementation is not a planned target

## Questions you should answer

- What is the minimum operational capability set for v1?
- Which known defects or maintenance risks are the real reason to replace the current bridge?
- Which features belong in the core CLI versus in downstream skills or adapter layers?
- What should the first binary command surface look like?

## Desired outputs

- Rust v1 implementation that follows `docs/plans/tradingview-cli-rust-v1.md`
- updates to the ExecPlan as implementation progresses
- focused supporting notes only when new facts change the existing investigation record

## Status update 2026-04-24

The initial investigation and implementation-ready successor plan have been produced. The next agent should treat `docs/plans/tradingview-cli-rust-v1.md` as the active implementation plan.
