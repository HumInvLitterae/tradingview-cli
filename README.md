# TradingView CLI

TradingView CLI is a Rust-native command-line replacement for the current TradingView MCP Bridge usage in sibling trading-analysis projects.

This project is inspired by practical workflows built around [TradingView MCP Bridge](https://github.com/tradesdontlie/tradingview-mcp) by `tradesdontlie`. That project established the useful bridge pattern this repository is now narrowing into a CLI-first tool. This repository is not affiliated with TradingView Inc.

This tool requires the user's own valid TradingView Desktop installation, logged-in session, and data entitlements. It does not bypass TradingView access controls, subscriptions, paywalls, or exchange/data-provider licensing. It controls the locally running desktop app through Chrome DevTools Protocol and undocumented TradingView application interfaces, which may change or break without notice. Market data, Pine scripts, and TradingView account state remain subject to TradingView, exchange, data-provider, and script-author terms.

## Current status

This repository now contains the first Rust-native `tv` CLI implementation.

The first implementation focuses on a narrow CLI surface for connecting to an already-running TradingView Desktop instance through Chrome DevTools Protocol on `127.0.0.1:9222`.

The old TradingView MCP Bridge CLI command migration is now closed for the known JavaScript CLI surface. If new evidence shows a missed old CLI command, treat it as migration backlog unless a repository decision explicitly marks it out of scope. The MCP server is different: implementing an MCP server is not planned for this project.

## Purpose

This is a CLI-first tool that focuses on the practical capabilities currently needed from the existing TradingView bridge:

- bounded TradingView Desktop connectivity
- reliable command-line interaction
- capability surfaces that can later support provider, review, and operator workflows

The replacement is Rust-native, CLI-centered, and narrower than a full MCP-compatible reimplementation.

An MCP server is not planned for this project. Downstream integration should start through ordinary process invocation and JSON CLI output rather than by recreating the original MCP server surface.

## Compatibility policy

This Rust CLI is intended to replace practical usage of the old `tv` CLI over time, but it is not a drop-in JSON wire-format clone.

The Rust CLI intentionally uses stable command envelopes:

```json
{
  "success": true,
  "command": "quote",
  "data": {
    "symbol": "NASDAQ:AAPL"
  }
}
```

Errors use the same envelope shape with structured details:

```json
{
  "success": false,
  "command": "quote",
  "error": {
    "kind": "connection",
    "message": "CDP connection failed",
    "details": null
  }
}
```

The old JavaScript CLI usually returned command fields at the top level, for example `{ "success": true, "symbol": "NASDAQ:AAPL" }`. Downstream adapters must therefore read command payloads from `data` when migrating to this Rust CLI.

The wire shape may differ, but information compatibility is required for migrated commands: information available from the old CLI should remain available from the Rust CLI once the corresponding command is implemented. New fields may be added. Removing old practical information requires an explicit decision and migration note.

For a migration-focused summary, read `docs/breaking-changes-from-js-cli.md`.

## Non-goals

- no copied JavaScript bridge code
- no MCP server implementation
- no claim of JSON wire-format compatibility with the old JavaScript CLI
- no package-manager installer yet

## Validation

GitHub Actions runs the automated Rust baseline on push and pull request: `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test --workspace` across Linux, macOS, and Windows.

Tagged releases matching `v*` build native release archives for Linux, macOS, and Windows and publish them to GitHub Releases with `SHA256SUMS`.
If `docs/releases/<tag>.md` exists, the release workflow uses it as the GitHub Release body; otherwise it falls back to generated notes. The GitHub Release title already contains the tag, so release body files should not need a top-level version heading.

TradingView Desktop live smoke checks are intentionally separate from CI because they require a logged-in desktop session with Chrome DevTools Protocol enabled.

## Release Builds

GitHub Releases are the first supported binary distribution path. Pushing a version tag such as `v0.4.1` creates release assets named:

- `tv-v0.4.1-x86_64-unknown-linux-gnu.tar.gz`
- `tv-v0.4.1-x86_64-apple-darwin.tar.gz`
- `tv-v0.4.1-aarch64-apple-darwin.tar.gz`
- `tv-v0.4.1-x86_64-pc-windows-msvc.zip`
- `SHA256SUMS`

Each archive contains the `tv` or `tv.exe` binary, `README.md`, `CHANGELOG.md`, `LICENSE`, a user-facing `AGENTS.md` and `CLAUDE.md`, and runtime-oriented TradingView CLI skills under `.agents/skills/` and `.claude/skills/`, including chart analysis, market-data interpretation, multi-symbol scanning, Pine, replay, Screener operation, scanner/Screener result analysis, and strategy workflows. Verify the downloaded archive against `SHA256SUMS`, unpack it, and place the executable on your `PATH`.

The repository root `AGENTS.md` and `CLAUDE.md` are contributor-facing development guides. Release archives instead include user-facing agent guides for operating `tv` safely through an AI agent.

Package-manager installers, code signing, notarization, and crates.io publication are not part of the first release workflow.

## Quick Start

Install `tv` from a GitHub Release archive for your OS, or build it from the repository root while developing:

```bash
cargo install --path crates/cli
```

Launch TradingView Desktop with Chrome DevTools Protocol enabled. The bounded launcher searches common install locations and defaults to `127.0.0.1:9222`:

```bash
tv launch
```

On Windows, the standalone Desktop build is the recommended install for `tv launch`. Microsoft Store/MSIX `TradingView.Desktop 3.1.0.7818` has been smoke-tested with the current direct AppX launch path and can open CDP, but Store/MSIX behavior can still change with TradingView Desktop, Electron, or Microsoft Store packaging updates. On macOS, the launcher first tries a direct app spawn and then falls back to `open -a TradingView --args ...` if CDP does not become ready. The launcher does not close an existing TradingView session unless `--kill-existing` is explicit:

```bash
tv launch --kill-existing
```

If the launcher cannot find TradingView Desktop, pass an explicit executable path:

```bash
tv launch --path "/Applications/TradingView.app/Contents/MacOS/TradingView"
```

Common TradingView Desktop executable paths include:

- macOS: `/Applications/TradingView.app/Contents/MacOS/TradingView`
- Windows: `%ProgramFiles%\TradingView\TradingView.exe`, preferably from the standalone Desktop installer. Store/MSIX `TradingView.Desktop 3.1.0.7818` has also been verified through direct AppX launch; if Store/MSIX CDP startup fails on another version, use the standalone build or pass an explicit `--path`.
- Linux: `/opt/TradingView/tradingview`, `/opt/TradingView/TradingView`, or `/snap/tradingview/current/tradingview`

Then run commands against the active TradingView Desktop session:

```bash
tv status
tv tab list
tv --target-id <CDP_TARGET_ID> quote AAPL
tv quotes AAPL MSFT NYSE:IONQ
tv fundamentals NYSE:IONQ
tv fundamentals AAPL --field earnings_release_next_date --field earnings_release_next_time
tv scanner hotlist volume_gainers --limit 10
tv scanner metainfo --market america --field close --field premarket_close --field postmarket_close
tv scanner scan --exchange NASDAQ --exchange NYSE --sort market_cap_basic --desc --limit 10
tv scanner scan --type stock --sector "Technology Services" --min-relative-volume 1.5 --sort relative_volume_10d_calc --desc --limit 10
tv scanner scan --type stock --min-average-volume 1000000 --min-performance-week 5 --max-rsi 70 --sort Perf.W --desc --limit 10
tv scanner scan --type stock --max-change -5 --sort change --asc --columns name,change,volume --limit 10
tv scanner scan --type stock --columns name,close,premarket_close,premarket_volume,postmarket_close,postmarket_volume --limit 10
tv scanner scan --type stock --columns name,earnings_release_next_date,earnings_release_date,price_earnings_ttm --limit 10
TV_EXPERIMENTAL_BARS=1 tv bars NASDAQ:AAPL --timeframe 1D --count 5
tv screener get --limit 10
tv screener screens actions
tv screener screens list
tv screener screens list --catalog
tv screener screens switch --name "米国株（テスト用）" --dry-run
tv screener screens switch --name "CLI-Test2" --catalog --dry-run
tv screener screens save --dry-run
tv screener screens create --name CLI-Test-Codex --dry-run
tv screener screens rename --name CLI-Test-Codex --to CLI-Test-Codex-Renamed --dry-run
tv screener screens save-as --name CLI-Test-Codex-Copy --dry-run
tv screener screens delete --name CLI-Test-Codex --dry-run
tv screener screens delete --name CLI-Test-Codex --confirm-delete
tv screener filters list
tv screener filters actions
tv screener filters add --name RSI --min 70 --dry-run
tv screener filters modify --text "EMA (21)" --min 10 --dry-run
tv screener filters modify --text "アナリストの評価" --option "買い" --dry-run
tv screener filters remove --text PER --dry-run
tv screener columns list
tv screener columns config
tv screener columns actions
tv screener columns add --id TechnicalRating --params-json '{"resolution":"TimeResolution1D"}' --after-index 11 --dry-run
tv screener columns remove --name "EMA (21)" --dry-run
tv screener columns reorder --from-index 12 --to-index 11 --dry-run
tv quote
tv quote AAPL
tv info AAPL
tv ohlcv --summary --count 100
tv watchlist get
tv watchlist add-bulk NASDAQ:AAPL NASDAQ:MSFT --delay-ms 500 --allow-partial
tv layout list
tv alert list
tv alert create-indicator --script "My Indicator" --file my-indicator.pine --condition-title "Long" --dry-run
tv alert create-indicator --script "My Indicator" --file my-indicator.pine --condition-title "Long" --message "tv-cli smoke"
tv pane list
tv data shapes --count 100
tv draw position long --entry-price 100 --stop-loss 95 --take-profit 110
tv pine get
tv pine alertconditions --file my-indicator.pine
tv screenshot --region chart --output target/tv-chart.png
```

Most commands operate on the current chart target. Mutation commands such as `symbol <SYMBOL>`, `watchlist add`, `watchlist add-bulk`, `watchlist remove`, `screener screens switch`, `screener screens save`, `screener screens create`, `screener screens rename`, `screener screens save-as`, `screener screens delete`, `screener filters add`, `screener filters modify`, `screener filters remove`, `screener filters clear`, `screener columns add`, `screener columns remove`, `screener columns reorder`, `alert create`, `alert create-indicator`, `draw position`, `draw clear`, `pine save`, `layout switch`, and generic `ui` automation can change TradingView account, chart, editor, or UI state; prefer their read-only or `--dry-run` forms when available. `tv info <SYMBOL>`, `tv fundamentals <SYMBOL>`, `tv quote <SYMBOL>`, and `tv quotes <SYMBOL>...` are Desktop-free symbol reads by default: `info` uses TradingView symbol search metadata, `fundamentals` uses scanner REST fundamental fields, and `quote` / `quotes` use scanner REST quote data. `tv fundamentals` returns raw TradingView scanner values under `field_values`; earnings date/time fields are not interpreted into a timezone or before/after-market label by the CLI. Use `tv quote <SYMBOL> --source chart` when you explicitly want the selected TradingView Desktop chart feed, or `tv quote <SYMBOL> --source auto` when you want chart-first behavior with scanner fallback only if the chart path is unavailable before mutation. Scanner-backed quote payloads include regular quote fields, `time`, `update_mode`, parsed `delay_seconds` when the mode is clearly delayed, and an additive `extended_hours` object for premarket and postmarket values when TradingView returns them; top-level `last` and `close` are not replaced by extended-hours values. `tv quotes` preserves input order in `items[]`; each successful item contains the same scanner quote payload shape as `tv quote <SYMBOL>`, and each failed item contains a structured error for the requested symbol. Credential-free scanner price reads are useful for screening but are not a realtime entitlement guarantee: freshness can depend on the exchange, TradingView feed, and market-data subscription state. `tv scanner metainfo` is different; it reads scanner field metadata rather than prices. `tv bars <SYMBOL>` is an experimental Desktop-free historical bars prototype that uses an undocumented TradingView WebSocket path, requires `TV_EXPERIMENTAL_BARS=1`, requires an exchange-qualified symbol such as `NASDAQ:AAPL`, and does not replace `tv ohlcv`. `tv info` without a symbol and `tv quote` without a symbol still read current-chart data through CDP. Chart source selection serializes symbol-targeted quote commands with a short process lock, waits for requested-symbol chart bars to become fresh, retries that readiness wait once on timeout, restores the original symbol, and fails instead of reporting success if the observed quote symbol or bar freshness does not match the request. `restored: true` only means the original chart symbol was restored; it is not by itself a quote freshness guarantee. `tv watchlist add` and `tv watchlist remove` prefer TradingView's logged-in symbols-list API for the active custom watchlist and verify the symbol's presence or absence after mutation; if that API cannot be used before mutation, they fall back to the older visible-panel DOM path. `tv alert create` prefers TradingView's logged-in alert endpoint, verifies the created alert through a list readback, and falls back to the visible dialog only when the API path fails before the create request is sent. `tv alert create-indicator` uses the same alert endpoint family for Pine `alertcondition()` alerts: dry-run verifies the local source candidate and saved-script match, while normal mode creates only when required metadata is available and verifies the new alert by list readback. It does not ask users to paste saved-script identifiers or raw endpoint payloads. `tv screener screens switch` targets exact screen names and verifies the active title after clicking; by default it uses the active screen title menu, while `--catalog` uses the saved-screen catalog. If TradingView does not activate the target, it fails instead of reporting success. `tv screener screens save` clicks only the exact visible `Save screen` / `スクリーンを保存` action and supports `--dry-run`; use it only on prepared test or disposable Screener screens. `tv screener screens create`, `rename`, and `save-as` are limited to test or disposable names for normal mutations and report success only after the active screen title changes to the requested name. `tv screener screens delete` resolves exact saved-screen names through the logged-in Screener storage API, requires `--confirm-delete` for normal mutation, refuses non-test names and active screens, and reports success only after the deleted name is absent from the saved-screen list. `tv screener filters add` searches the visible add-filter catalog, selects a numeric range preset, and reports success only after a new visible filter pill appears; use `--dry-run` first on a prepared test screen. `tv screener filters modify` supports visible numeric range presets such as `0% 〜 10%` or `10%以上`, and single visible option selection through `--option`; it validates finite range values before CDP connection, rejects mixed range/option input, and verifies the visible filter text after a normal mutation. Multi-option add/remove semantics and free-text filter editors are not genericized. `tv screener filters remove` and `tv screener filters clear` use the active saved screen's storage filter set for normal mutation, are limited to test or disposable screen names, require storage re-fetch post-checks, and request a best-effort full-page Screener refresh when available; `clear` still requires `--confirm-clear` unless it is a dry run. `tv screener columns add`, `remove`, and `reorder` update the active saved screen's storage column set, are limited to test or disposable screen names for normal mutation, and report success only after the saved storage order matches the requested result. `columns add` is intentionally low-level: it inserts a known storage column id and JSON-object params rather than searching by display name. `tv draw position` returns an `entity_id`; clean up test drawings with `tv draw remove <ENTITY_ID>` rather than `draw clear`. `tv ui eval` is a dangerous old-CLI compatibility command that runs arbitrary JavaScript in the authenticated TradingView page context and is disabled unless `TV_ALLOW_UNSAFE_UI_EVAL=1` is set.

`tv pine alertconditions [--file <PATH>]` is local static discovery for Pine
source. It estimates `alertcondition()` candidate ids such as `plot_1` without
connecting to TradingView or creating alerts. Treat the output as a best-effort
planning aid; TradingView compile and later guarded alert dry-run behavior
remain the authority for account alert creation.

`tv alert create-indicator ... [--dry-run]` creates or previews Pine
`alertcondition()` alerts. It combines a local Pine source candidate with an
exact saved-script name visible to the logged-in TradingView session. Normal
mode uses the alert endpoint and post-create readback; if required saved-script
or input metadata cannot be resolved safely, it fails before sending the create
request. The command does not print saved-script identifiers.

Screenshots require an explicit `--output <PATH>` file path. Parent directories are created automatically, so agent or Claude Desktop workflows should choose a readable output path directly instead of relying on a default screenshots directory.

The default CDP endpoint is `127.0.0.1:9222`. Override it with `TV_CDP_HOST` and `TV_CDP_PORT` when needed, for example if a local setup still requires `localhost`.

### Multiple Chart Targets

If more than one TradingView chart target is open, run `tv tab list`, choose the intended `target_id`, and pass it explicitly:

```bash
tv --target-id <CDP_TARGET_ID> quote
tv --target-id <CDP_TARGET_ID> ohlcv --summary
tv --target-id <CDP_TARGET_ID> symbol NASDAQ:MU
```

`tv tab list` and `tv tab switch <INDEX>` return `target_cli_args`, which can be reused directly for the next command. `--target-id` is the only explicit target handoff; `TV_CDP_TARGET_ID` is no longer part of the public contract.

Use `tv screener open --full-page` to reuse a Stock Screener page tab and get
its `target_cli_args`. The command also attempts the local CDP new-target path,
but current TradingView Desktop builds may reject that endpoint. When that
happens, the command falls back to TradingView Desktop's new-tab Screener tile
and reports `creation_method: "new_tab_tile"` if a full-page target appears.
`tv tab list` also returns `screener_targets` when a Stock Screener page tab is
already open.

If `tv ohlcv` fails while `tv quote` or `tv symbol` still works, do not keep
retrying the same command blindly. Preserve the full JSON error envelope, check
`error.details`, rerun `tv tab list`, choose the active chart target's
`target_cli_args`, then run `tv --target-id <ID> state` and
`tv --target-id <ID> ohlcv --count 1`. For one-off symbol metadata or quotes,
prefer Desktop-free `tv info <SYMBOL>` and `tv quote <SYMBOL>`. Use `tv info`
without a symbol only when you need the current chart's loaded symbol metadata.
The chart timeframe command is `tv timeframe <RESOLUTION>`, not `interval`.

Commands print structured JSON. Most successful commands print one `success: true` envelope to stdout. `tv stream ...` commands are intentionally long-running and print newline-delimited JSON envelopes, one line per changed sample. Failed commands print a `success: false` envelope to stderr.

Exit codes are:

- `0`: success
- `1`: usage, validation, target ambiguity, or unexpected internal failure
- `2`: TradingView or CDP connection failure
- `3`: TradingView internal API unavailable
- `4`: timeout

## Development

During local development, you can run commands without installing the binary:

```bash
cargo run -- status
cargo run -- quote
```

Use `tv --help` or `cargo run -- --help` for the full command list.

Optional local Git guardrails can be installed with Git 2.54 or newer. If you
use `mise`, run:

```bash
mise run hooks:install
```

Without `mise`, run the installer directly:

```bash
scripts/install-config-hooks.sh
```

On Windows, use PowerShell:

```powershell
./scripts/install-config-hooks.ps1
```

These hooks are local development helpers only. The authoritative validation
baseline remains the commands run by CI.

## What is included

- a Rust v1 `tv` CLI implementation
- a GitHub Actions CI baseline for Rust formatting, linting, and tests
- a GitHub Actions release workflow for tag-triggered native binary archives
- optional Git 2.54 config-based local hooks with `mise` task shortcuts
- `CHANGELOG.md` release notes for public versions
- user-facing agent guides and runtime skills in release archives, including a
  dedicated Stock Screener workflow skill
- Desktop-free symbol metadata and quote reads through `tv info <SYMBOL>`,
  `tv quote [SYMBOL]`, and `tv quotes <SYMBOL>...`
- read-only TradingView scanner REST reads through `tv scanner hotlist`,
  `tv scanner scan`, and `tv scanner metainfo`
- TradingView Stock Screener dialog and full-page target handoff, menu-visible and catalog screen switching, exact screen action/save support, guarded test-screen lifecycle commands, storage-backed filter cleanup and range edits, filter add, preset-range filter modification, and storage-backed column inspection/add/remove/reorder through `tv screener status/open/get/screens active/actions/list/switch/save/create/rename/save-as/delete/filters list/actions/add/modify/remove/clear/columns list/config/actions/add/remove/reorder/close`
- old JavaScript CLI command migration coverage for the known CLI surface
- stable architecture, development, packaging, migration, and internal API
  references under `docs/`
- a Rust API reuse guide at `docs/rust-api.md`
- a `v0.3.0` roadmap and release-readiness record at `docs/v0.3-roadmap.md`,
  plus a `v0.4.x` roadmap and release-readiness record at
  `docs/v0.4-roadmap.md`
- a `v0.5.0` roadmap draft at `docs/v0.5-roadmap.md`
- historical command contract, lifecycle, and deferred-surface notes under
  `docs/notes/`
- a public-safe internal TradingView API dependency and DOM replacement
  feasibility reference at `docs/internal-tradingview-apis.md`
- a Screener completion and stabilization boundary note at `docs/notes/screener-surface-completion-and-stabilization.md`
- historical implementation ExecPlans archived under `docs/plans/archives/`
- repo-local CLI skills migrated from the original MCP workflow split, with
  current scanner, Screener, watchlist, and alert operating boundaries reflected

## Where to start

Read these in order:

1. `docs/notes/next-agent-handoff-prompt-2026-04-24.md`
2. `docs/v0.4-roadmap.md`
3. `docs/v0.3-roadmap.md`
4. `docs/architecture.md`
5. `docs/development.md`
6. `docs/rust-api.md`
7. `docs/release-packaging.md`
8. `docs/breaking-changes-from-js-cli.md`
9. `docs/internal-tradingview-apis.md`
10. `docs/notes/rust-cli-contract-migration-2026-04-24.md`
11. `docs/notes/legacy-cli-command-migration-inventory-2026-04-24.md`
12. `docs/notes/remaining-deferred-surface-audit-2026-04-25.md`
13. `docs/notes/upstream-pr-triage-2026-04-25.md`
14. `docs/notes/upstream-pr-recheck-2026-04-27.md`
15. `docs/plans/README.md`
16. `docs/notes/tradingview-mcp-investigation-2026-04-24.md`

The first capability and boundary research milestone, the Rust v1 implementation milestone, the read/provider migration slices, chart/pane/watchlist/alert/layout/indicator/drawing/Pine/tab/replay/stream/launch slices, command lifecycle balance audit, remaining deferred surface audit, operation-layer and data-operation module refactors, development guideline pass, remaining old CLI migration closure slice, and first release readiness pass are complete. Upstream pull request follow-up has addressed the initial narrow Rust fixes and additions, including `tv data shapes`, `tv data labels` truncation metadata, `tv scanner hotlist`, `tv scanner scan`, read-oriented `tv screener` dialog and metadata commands, menu-visible and catalog Screener screen switching, Screener screen action/save support, guarded Screener screen lifecycle, storage-backed Screener filter remove/clear, guarded Screener filter add/modify, storage-backed Screener column config/add/remove/reorder, API-backed `tv watchlist add/remove` mutation with DOM fallback, API-backed `tv alert create` mutation with readback verification, Desktop-free `tv info <SYMBOL>` and `tv quote [SYMBOL]` reads, static Pine `alertcondition()` discovery, and guarded API-backed indicator alertcondition alert creation. The main planned Screener surface is now in stabilization; `columns reset` and broader multi-option/free-text filter editing remain evidence-gated. Future stabilization should check storage/API options before adding more DOM retries.

## License

This project is licensed under the MIT License. See `LICENSE`.
