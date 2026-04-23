# Next agent handoff prompt 2026-04-24

Use this repository as the starting point for a Rust-native TradingView CLI project whose first implementation milestone is complete.

## Mission

Keep the Rust-native `tv` CLI narrow, reliable, and useful as a replacement path for practical TradingView bridge usage in sibling trading-analysis projects. Do not expand the tool just because the old bridge had a command or MCP endpoint. Add new capability only when downstream use proves it belongs in the core CLI.

## What has already been decided

- this work belongs in a separate repository
- v1 is CLI-first
- the Rust v1 `tv` implementation exists
- MCP server implementation is not planned for this project
- downstream integration should start through process invocation and JSON output
- full feature parity with the migration source is not a goal
- chart-region screenshots require a separate stability spike before being advertised

## Current v1 surface

The implemented commands are:

- `tv status`
- `tv state`
- `tv quote`
- `tv ohlcv --summary`
- `tv symbol <SYMBOL>`
- `tv timeframe <RESOLUTION>`
- `tv screenshot --region full --output <PATH>`

The default CDP endpoint is `localhost:9222`. `TV_CDP_HOST` and `TV_CDP_PORT` can override it.

All commands use structured JSON envelopes. Successful commands print `success: true` to stdout. Failed commands print `success: false` to stderr.

## Your first tasks

1. Read `README.md`
2. Read `docs/plans/tradingview-cli-rust-v1.md`
3. Read `docs/notes/tradingview-mcp-investigation-2026-04-24.md`
4. Check `git status --short`
5. Run targeted validation before changing behavior

## Constraints

- do not write machine-specific absolute paths into tracked docs
- do not assume every old capability deserves a replacement
- do not promise release packaging or public API stability yet
- do not bloat the core CLI with downstream workflow helpers that can live in consumer repos
- do not implement an MCP server; this project is planned as a CLI-first replacement, and MCP server implementation is not a planned target
- keep changes committed in related batches when files are changed

## Recommended next work

Focus first on operational readiness rather than feature growth:

- keep README and agent-facing docs aligned with the implemented v1 surface
- smoke-test the CLI against real TradingView Desktop sessions when available
- exercise the CLI from downstream workflows before expanding the command surface
- record evidence before starting any post-v1 ExecPlan

Post-v1 candidates that need investigation before implementation:

- whether `screenshot --region chart` can be made stable enough for repeated use
- whether launch automation belongs in this CLI or should remain external runbook material
- whether downstream workflows need additional read-only commands after v1 is exercised

## Validation baseline

The Rust v1 implementation previously passed:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features
cargo test
git diff --check
```

Manual smoke testing previously passed against a running TradingView Desktop CDP target for every v1 command.
