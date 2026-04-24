# Next agent handoff prompt 2026-04-24

Use this repository as the starting point for a Rust-native TradingView CLI project whose first implementation milestone is complete.

## Mission

Keep the Rust-native `tv` CLI reliable and useful as a replacement path for practical TradingView bridge usage in sibling trading-analysis projects. The first implementation is intentionally narrow, but the broader old CLI migration is still in progress. Do not confuse unimplemented commands with rejected commands.

## What has already been decided

- this work belongs in a separate repository
- v1 is CLI-first
- the Rust v1 `tv` implementation exists
- MCP server implementation is not planned for this project
- downstream integration should start through process invocation and JSON output
- the Rust JSON envelope intentionally differs from the old JavaScript CLI
- migrated commands must preserve the practical information available from the old CLI
- missing old CLI commands are migration backlog unless explicitly excluded
- chart-region screenshots have a first Rust implementation, but remain DOM-selector dependent
- the high-priority planned read-only migration backlog is complete
- the operation layer is split into a thin `src/ops.rs` facade plus feature modules under `src/ops/`; do not reintroduce a monolithic ops file or `mod.rs`
- the data operation layer is split into a thin `src/ops/data.rs` facade plus `indicator`, `strategy`, and `drawings` modules under `src/ops/data/`
- development guidelines are recorded in `docs/notes/development-guidelines-2026-04-24.md`
- `data depth` is implemented as a read-only DOM-dependent slice and may require a visible DOM or Depth of Market panel
- `alert list` is implemented as a read-only internal API slice
- `alert create` is implemented as an explicit account mutation; downstream workflow helpers remain outside the core CLI
- `alert delete --id` is implemented as an explicit account mutation and is the cleanup pair for created alerts
- `watchlist add` is implemented as an explicit operator mutation using DOM panel controls plus CDP input events
- `watchlist remove <SYMBOL>` is implemented as a Rust-specific cleanup command for `watchlist add`; it is exact-match and row-scoped
- command lifecycle balance has been audited, and no immediate asymmetric lifecycle gap is known in the implemented Rust CLI
- GitHub Actions CI is configured for the automated Rust baseline
- `pane layout`, `pane focus`, and `pane symbol` are implemented as explicit chart mutations using TradingView's chart widget collection
- `indicator add/remove/toggle/set/get` is implemented as a complete chart-local lifecycle mutation and read surface
- `draw shape/list/get/remove` is implemented as a chart-local drawing lifecycle surface; `draw clear` remains deferred as bulk destructive cleanup
- `pine get/set/errors/console/list` is implemented as a Pine surface; `set` changes only the editor buffer, while save, compile, new/open, analyze, and check remain deferred
- `tab list/switch/new/close` is implemented as a bounded tab lifecycle surface; `tab list` preserves chart-target fields and adds app-tab fields, while `tab close` requires an explicit app-tab index and refuses to close the final app tab
- `replay start/step/stop/status/autoplay/trade` is implemented as a bounded replay lifecycle surface

## Current v1 surface

The implemented commands are:

- `tv status`
- `tv state`
- `tv info`
- `tv search <QUERY>`
- `tv quote`
- `tv values`
- `tv discover`
- `tv ui-state`
- `tv ohlcv --summary`
- `tv ohlcv --count <N>`
- `tv range`
- `tv range --from <UNIX_SECONDS> --to <UNIX_SECONDS>`
- `tv scroll <DATE_OR_UNIX_SECONDS>`
- `tv watchlist get`
- `tv watchlist add <SYMBOL>`
- `tv watchlist remove <SYMBOL>`
- `tv pane list`
- `tv pane layout <LAYOUT>`
- `tv pane focus <INDEX>`
- `tv pane symbol <INDEX> <SYMBOL>`
- `tv alert list`
- `tv alert create --price <NUMBER> [--condition <CONDITION>] [--message <TEXT>]`
- `tv alert delete --id <ALERT_ID>`
- `tv indicator add <INDICATOR_NAME...> [--inputs <JSON>]`
- `tv indicator remove <ENTITY_ID>`
- `tv indicator toggle <ENTITY_ID> [--visible | --hidden]`
- `tv indicator set <ENTITY_ID> --inputs <JSON>`
- `tv indicator get <ENTITY_ID>`
- `tv draw shape --type <TYPE> --price <NUMBER> --time <UNIX_SECONDS> [--price2 <NUMBER>] [--time2 <UNIX_SECONDS>] [--text <TEXT>] [--overrides <JSON>]`
- `tv draw list`
- `tv draw get <ENTITY_ID>`
- `tv draw remove <ENTITY_ID>`
- `tv pine get`
- `tv pine set [--file <PATH>]`
- `tv pine errors`
- `tv pine console`
- `tv pine list`
- `tv tab list`
- `tv tab switch <INDEX>`
- `tv tab new [--from <INDEX>]`
- `tv tab close <INDEX>`
- `tv replay start [--date <YYYY-MM-DD>]`
- `tv replay step`
- `tv replay stop`
- `tv replay status`
- `tv replay autoplay [--speed <MS>]`
- `tv replay trade <buy|sell|close>`
- `tv data indicator <ENTITY_ID>`
- `tv data strategy`
- `tv data trades [--max <N>]`
- `tv data equity`
- `tv data lines [--filter <TEXT>] [--verbose]`
- `tv data labels [--filter <TEXT>] [--max <N>] [--verbose]`
- `tv data tables [--filter <TEXT>]`
- `tv data boxes [--filter <TEXT>] [--verbose]`
- `tv data depth`
- `tv symbol [SYMBOL]`
- `tv timeframe [RESOLUTION]`
- `tv type [CHART_TYPE]`
- `tv screenshot --region full --output <PATH>`
- `tv screenshot --region chart --output <PATH>`

The default CDP endpoint is `localhost:9222`. `TV_CDP_HOST` and `TV_CDP_PORT` can override it.

All commands use structured JSON envelopes. Successful commands print `success: true` to stdout. Failed commands print `success: false` to stderr.

The Rust CLI does not preserve the old JavaScript CLI's top-level payload wire shape. Command payloads live under `data`, and errors live under `error.kind` / `error.message` / `error.details`. Read `docs/notes/rust-cli-contract-migration-2026-04-24.md` before changing adapters.

## Your first tasks

1. Read `README.md`
2. Read `docs/notes/development-guidelines-2026-04-24.md`
3. Read `docs/notes/rust-cli-contract-migration-2026-04-24.md`
4. Read `docs/notes/legacy-cli-command-migration-inventory-2026-04-24.md`
5. Read `docs/plans/tradingview-cli-rust-v1.md`
6. Read `docs/notes/tradingview-mcp-investigation-2026-04-24.md`
7. Check `git status --short`
8. Run targeted validation before changing behavior

## Constraints

- do not write machine-specific absolute paths into tracked docs
- do not assume every old capability deserves a replacement
- do not promise release packaging or public API stability yet
- do not bloat the core CLI with downstream workflow helpers that can live in consumer repos
- do not implement an MCP server; this project is planned as a CLI-first replacement, and MCP server implementation is not a planned target
- do not describe missing old CLI commands as out of scope unless a durable decision excludes them
- do not reduce practical information available from old CLI commands when implementing their Rust equivalents
- keep changes committed in related batches when files are changed

## Recommended next work

Focus first on migration readiness:

- keep README and agent-facing docs aligned with the implemented v1 surface
- smoke-test the CLI against real TradingView Desktop sessions when available
- keep GitHub Actions CI aligned with the local baseline
- exercise the CLI from downstream workflows before deciding the next command slice
- keep new operation code in the relevant `src/ops/` feature module
- expand old CLI command coverage in planned slices, preserving information compatibility
- use `docs/notes/command-lifecycle-balance-audit-2026-04-24.md` when evaluating mutation surfaces and cleanup gaps
- record evidence before starting any post-v1 ExecPlan

Deferred old CLI surfaces that need planned implementation or an explicit exclusion decision:

- whether launch automation belongs in this CLI or should remain external runbook material
- larger old CLI surfaces such as alert bulk deletion/editing, draw clear, Pine save/compile/new/open/analyze/check helpers, stream, and UI automation

## Validation baseline

The Rust v1 implementation is checked locally and in GitHub Actions with:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
git diff --check
```

Manual smoke testing remains separate from CI because it requires a running, logged-in TradingView Desktop CDP target. `alert create` was live-smoked with an explicit test alert after the user approved account mutation smoke testing; that smoke alert was later deleted while confirming the `alert delete --id` endpoint.
