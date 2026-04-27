# Next agent handoff prompt 2026-04-24

Use this repository as the starting point for a Rust-native TradingView CLI project whose first implementation milestone is complete.

## Mission

Keep the Rust-native `tv` CLI reliable and useful as a replacement path for practical TradingView bridge usage in sibling trading-analysis projects. The known old JavaScript CLI command migration is closed; if new evidence shows a missed old CLI command, treat it as migration backlog unless a durable decision explicitly excludes it.

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
- stable architecture, development, and release packaging guidance is now summarized under `docs/architecture.md`, `docs/development.md`, and `docs/release-packaging.md`
- development guidelines are recorded in `docs/development.md`
- `data depth` is implemented as a read-only DOM-dependent slice and may require a visible DOM or Depth of Market panel
- `alert list` is implemented as a read-only internal API slice
- `alert create` is implemented as an explicit account mutation; it now prefers the logged-in alert endpoint and verifies creation through alert-list readback before success
- `alert delete --id` is implemented as an explicit account mutation, uses the bare delete endpoint shape verified by live cleanup, and remains the cleanup pair for created alerts
- `watchlist add` is implemented as an explicit operator mutation that prefers TradingView's logged-in symbols-list API for the active custom watchlist, reports `already_present` when applicable, verifies newly added symbols through API readback before returning success, and falls back to the older DOM panel path only when the API cannot be used before mutation
- `watchlist remove <SYMBOL>` is implemented as a Rust-specific cleanup command for `watchlist add`; it prefers the same API-backed active-list path, verifies absence through API readback, and keeps the older exact-match row-scoped DOM path as fallback when the API is unavailable before mutation
- command lifecycle balance has been audited, and no immediate asymmetric lifecycle gap is known in the implemented Rust CLI
- GitHub Actions CI is configured for the automated Rust baseline
- `pane layout`, `pane focus`, and `pane symbol` are implemented as explicit chart mutations using TradingView's chart widget collection
- `layout list` and `layout switch` are implemented; `layout switch` supports `--dry-run`
- `indicator add/remove/toggle/set/get` is implemented as a complete chart-local lifecycle mutation and read surface
- `draw shape/position/list/get/remove/clear` is implemented as a chart-local drawing lifecycle surface; `draw position` returns an `entity_id` for cleanup with `draw remove`, and `draw clear` includes a read-only dry-run and post-clear verification
- `pine get/set/new/open/save/compile/raw-compile/analyze/check/errors/console/list` is implemented as a Pine surface; `set`, `new`, and `open` change only the editor buffer, `save` explicitly persists the current saved script to TradingView cloud state, `compile` compiles the current buffer, may add or update a chart-local study, and refuses save-related buttons, while `raw-compile` preserves the old broad button behavior and may click save-related Pine actions. Named new-save for unsaved scripts is deferred because current live smoke showed the TradingView naming dialog can be outside the CDP page target.
- `tab list/switch/new/close` is implemented as a bounded tab lifecycle surface; `tab list` preserves chart-target fields and adds app-tab fields, while `tab close` requires an explicit app-tab index and refuses to close the final app tab
- `replay start/step/stop/status/autoplay/trade` is implemented as a bounded replay lifecycle surface
- `stream quote/bars/values/lines/labels/tables/all` is implemented as read-only JSONL polling for shell and external monitoring workflows
- `launch` is implemented as a bounded local process-control command; it is no-kill by default, treats an already responding CDP endpoint as success, and requires explicit `--kill-existing` for process termination
- `tv quote [SYMBOL]` supports symbol-targeted quote reads by temporarily switching the chart only when needed, serializing symbol-targeted quote commands with a short process lock, and verifying restoration before success
- `tv screener screens list --catalog` and `tv screener screens switch --name <NAME> --catalog [--dry-run]` support exact saved-screen catalog targeting while preserving the older title-menu default path
- `tv screener screens actions` and `tv screener screens save [--dry-run]` support visible screen-menu inspection and exact save-action clicking for prepared test screens
- `tv screener filters add --name <TEXT> --min <N>|--max <N> [--dry-run]` supports visible add-filter catalog numeric presets with dry-run candidate reporting and visible-pill post-checks
- Screener open-state detection requires a visible, in-viewport Screener panel root; toolbar buttons and unrelated right-panel content should not count as `open: true`
- upstream PR #105 drawing wrapper regression was live-smoked against Rust; `draw shape/list/get/remove/clear` worked on disposable drawings, so no Rust code change was needed
- remaining old CLI migration closure is recorded in `docs/plans/archives/tradingview-cli-remaining-migration-closure.md`; `layout switch`, `alert delete --all`, `pine raw-compile`, and generic `ui` commands are implemented. Alert edit/pause/resume are future feature research, not confirmed old CLI backlog.
- upstream pull requests on the original repository have been triaged in `docs/notes/upstream-pr-triage-2026-04-25.md`; use that note before choosing post-release fixes or additions.
- `tv ui eval` is default-disabled as a dangerous compatibility command; it runs only when `TV_ALLOW_UNSAFE_UI_EVAL=1` is set.
- `tv data strategy`, `tv data trades`, and `tv data equity` use `StrategyScript` marker detection and `_reportData` fallbacks for TradingView Desktop 3.1.0-style strategy data.
- upstream PR #102 guardrail follow-up is addressed through CI permission/concurrency hardening, optional Git 2.54 config-based hooks, and `mise` task shortcuts. The hooks are local helpers, not a replacement for CI or the standard validation baseline.

## Current v1 surface

The implemented commands are:

- `tv status`
- `tv launch [--port <PORT>] [--path <PATH>] [--kill-existing]`
- `tv state`
- `tv info`
- `tv search <QUERY>`
- `tv quote [SYMBOL]`
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
- `tv layout list`
- `tv alert list`
- `tv alert create --price <NUMBER> [--condition <CONDITION>] [--message <TEXT>]`
- `tv alert delete --id <ALERT_ID>`
- `tv indicator add <INDICATOR_NAME...> [--inputs <JSON>]`
- `tv indicator remove <ENTITY_ID>`
- `tv indicator toggle <ENTITY_ID> [--visible | --hidden]`
- `tv indicator set <ENTITY_ID> --inputs <JSON>`
- `tv indicator get <ENTITY_ID>`
- `tv draw shape --type <TYPE> --price <NUMBER> --time <UNIX_SECONDS> [--price2 <NUMBER>] [--time2 <UNIX_SECONDS>] [--text <TEXT>] [--overrides <JSON>]`
- `tv draw position <long|short> --entry-price <NUMBER> --stop-loss <NUMBER> --take-profit <NUMBER> [--entry-time <UNIX_SECONDS>] [--account-size <NUMBER>] [--risk <NUMBER>] [--lot-size <NUMBER>]`
- `tv draw list`
- `tv draw get <ENTITY_ID>`
- `tv draw remove <ENTITY_ID>`
- `tv draw clear [--dry-run]`
- `tv pine get`
- `tv pine set [--file <PATH>]`
- `tv pine new [indicator|strategy|library]`
- `tv pine open <NAME...>`
- `tv pine save`
- `tv pine compile`
- `tv pine analyze [--file <PATH>]`
- `tv pine alertconditions [--file <PATH>]`
- `tv pine check [--file <PATH>]`
- `tv pine errors`
- `tv pine console`
- `tv pine list`
- `tv alert create-indicator --script <NAME> --file <PATH> --condition-title <TITLE>|--alert-cond-id <ID> --dry-run`
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
- `tv stream quote [--interval <MS>]`
- `tv stream bars [--interval <MS>]`
- `tv stream values [--interval <MS>]`
- `tv stream lines [--filter <TEXT>] [--interval <MS>]`
- `tv stream labels [--filter <TEXT>] [--interval <MS>]`
- `tv stream tables [--filter <TEXT>] [--interval <MS>]`
- `tv stream all [--interval <MS>]`
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

The default CDP endpoint is `127.0.0.1:9222`. `TV_CDP_HOST` and `TV_CDP_PORT` can override it.

Commands use structured JSON envelopes. Most successful commands print one `success: true` envelope to stdout. Stream commands print newline-delimited `success: true` envelopes, one line per changed sample. Failed commands print `success: false` to stderr.

The Rust CLI does not preserve the old JavaScript CLI's top-level payload wire shape. Command payloads live under `data`, and errors live under `error.kind` / `error.message` / `error.details`. Read `docs/notes/rust-cli-contract-migration-2026-04-24.md` before changing adapters.

## Your first tasks

1. Read `README.md`
2. Read `docs/architecture.md`
3. Read `docs/development.md`
4. Read `docs/release-packaging.md` if the task touches release archives or distribution
5. Read `docs/notes/rust-cli-contract-migration-2026-04-24.md`
6. Read `docs/notes/legacy-cli-command-migration-inventory-2026-04-24.md`
7. Read `docs/plans/archives/tradingview-cli-rust-initial-implementation.md`
8. Read `docs/notes/tradingview-mcp-investigation-2026-04-24.md`
9. Check `git status --short`
10. Run targeted validation before changing behavior

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
- treat old CLI command coverage as closed unless new evidence shows a missed command; preserve information compatibility for any future compatibility work
- use `docs/notes/command-lifecycle-balance-audit-2026-04-24.md` when evaluating mutation surfaces and cleanup gaps
- use `docs/plans/archives/tradingview-cli-remaining-migration-closure.md` and `docs/notes/remaining-deferred-surface-audit-2026-04-25.md` when checking old CLI migration closure
- use `docs/notes/upstream-pr-triage-2026-04-25.md` before acting on original upstream pull requests
- use `docs/notes/screener-hotlist-upstream-feasibility-2026-04-25.md` before implementing Stock Screener, Hotlist, or scanner-like features
- use `docs/notes/ui-screener-read-evidence-2026-04-26.md` before planning UI Screener commands
- use `docs/notes/screener-surface-completion-and-stabilization.md` before adding more Screener surface; the current main Screener implementation is in stabilization
- use `docs/internal-tradingview-apis.md` before changing code that depends on private TradingView page objects, page-session endpoints, or Screener storage
- use `docs/plans/archives/tradingview-cli-internal-api-command-audit.md` before adding retries to DOM-backed commands; it classifies which commands are API-backed, replacement candidates, research-only, or intentional DOM boundaries
- use `docs/plans/tradingview-cli-direct-http-feasibility.md` after the next release if planning direct HTTP reads that do not require TradingView Desktop page-session context
- record evidence before starting any post-v1 ExecPlan

Old CLI migration is closed except for MCP server implementation, which remains explicitly not planned. Narrow upstream PR follow-up is also partly complete: launch compatibility, strategy/Pine hardening, screenshot contract tests, tab target handoff, watchlist click hardening, API-backed `watchlist add/remove`, API-backed `alert create` with readback verification, `tv data shapes`, `tv data labels` truncation metadata, read-only `tv scanner hotlist`, read-only `tv scanner scan`, `tv screener status/open/get/screens active/actions/list/switch/save/create/rename/save-as/delete/filters list/actions/add/modify/remove/clear/columns list/config/actions/add/remove/reorder/close` including catalog-backed screen list/switch, exact screen save, guarded test-screen create/rename/save-as/delete, evidence-gated column action discovery, storage-backed filter remove/clear, storage-backed column config/add/remove/reorder, target-popover-scoped filter action discovery, guarded add-filter catalog numeric presets, preset-backed numeric filter modify, and single-option `filters modify --option`, `tv watchlist add-bulk`, `tv quote [SYMBOL]`, static `tv pine alertconditions`, dry-run-only `tv alert create-indicator`, PR #105 drawing smoke, and PR #102-style CI/agent guardrails have all been addressed. Remaining upstream-derived candidates are evidence-gated: Windows COM/AUMID launch activation only if Windows smoke proves current launch insufficient, layout-dialog behavior only after separate research shows core CLI value and safe operating policy, indicator alertcondition normal mutation only after readback and cleanup design, broader multi-option/free-text filter editing only after a separate safety plan, and alert edit/pause/resume as future feature research. Screener column reset was checked on 2026-04-27 and remains deferred because no post-checkable default source or visible reset action was found. Treat `filters add` and `filters modify` as guarded visible UI automation. Before adding more DOM retry logic, inspect whether the operation can use page-session storage/API evidence instead; active saved-screen storage currently supports Screener filter remove/clear by deleting existing filter entries, but not yet safe add/modify payload creation. Direct HTTP expansion is deferred until after the next release and should start with credential-safe read-only candidates, not account mutations. `data depth`, chart screenshots, visible strategy-table fallbacks, and generic UI automation remain intentional DOM boundaries unless new evidence appears.

## Validation baseline

The Rust v1 implementation is checked locally and in GitHub Actions with:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
git diff --check
```

Manual smoke testing remains separate from CI because it requires a running, logged-in TradingView Desktop CDP target. `alert create` was live-smoked with an explicit test alert after the user approved account mutation smoke testing; that smoke alert was later deleted while confirming the `alert delete --id` endpoint.
