# Upstream pull request triage 2026-04-25

This note triages open pull requests on the original
`tradesdontlie/tradingview-mcp` repository after the Rust `tv` CLI completed the
known old JavaScript CLI migration and first public release.

The goal is not to copy JavaScript changes into Rust. The goal is to identify
bug evidence, compatibility pressure, and feature requests that should influence
the Rust CLI backlog.

## Sources checked

- Repository: https://github.com/tradesdontlie/tradingview-mcp
- Command: `gh pr list -R tradesdontlie/tradingview-mcp --state open --limit 100`
- Follow-up: `gh pr view` for the open PRs listed below
- Rust comparison points: current `src/ops/launch.rs`, `src/ops/data/strategy.rs`,
  `src/ops/pine/editor.rs`, `src/ops/ui.rs`, `src/transport.rs`
- Refresh check after the `tv data labels` hardening slice:
  `gh pr list -R tradesdontlie/tradingview-mcp --state open --limit 20 --json number,title,updatedAt,url`
  still showed `#100` as the newest open PR.

As of this pass, the upstream repository has 45 open PRs.

## Recommended priority

1. Security review for unrestricted page-context evaluation. Completed as a
   default-disabled gate.
   Upstream `#54` removes `ui_evaluate` because it can run arbitrary JavaScript in
   an authenticated TradingView session. Rust keeps `tv ui eval` as an old CLI
   compatibility command, but it now requires `TV_ALLOW_UNSAFE_UI_EVAL=1` before
   connecting to CDP.

2. Strategy data compatibility. Addressed with Rust-side `StrategyScript`
   detection and `_reportData` fallbacks.
   Upstream `#90`, `#96`, and older `#51` report concrete TradingView Desktop
   3.1.0 breakage for strategy, trades, and equity reads. Rust now prefers
   `metaInfo().id` values beginning with `StrategyScript`, reads `_reportData`
   where available, and keeps legacy public accessors plus DOM fallbacks.

3. Release-user launch compatibility. Addressed with bounded Rust-side discovery
   and fallback metadata.
   Upstream `#100`, `#93`, `#80`, `#79`, `#76`, `#73`, `#52`, `#27`, and `#18`
   all point at launch fragility around Windows MSIX / WindowsApps installs and
   newer TradingView Desktop / Electron behavior. Rust `tv launch` now keeps its
   bounded/no-kill default, adds PowerShell-based Windows process/AppX discovery,
   reports `launch_method`, `resolved_by`, and `fallback_used`, and tries a
   macOS `open -a TradingView --args ...` fallback when direct spawn does not
   make CDP ready. Windows COM/AUMID activation remains a future enhancement if
   Windows live smoke proves direct AppX executable launch is still insufficient.

4. Pine Editor robustness. Addressed with Rust-side editor detection and
   compile-label hardening.
   Upstream `#97`, `#95`, and `#50` describe Pine Editor detection and
   button-matching problems in recent TradingView Desktop builds and non-English
   locales. Rust now tries the global Monaco editor API before falling back to
   React fiber traversal, re-runs the Pine panel open trigger during the Monaco
   polling loop, and recognizes Korean Add/Update-on-chart labels while keeping
   `pine compile` non-persistent.

5. Existing command hardening and modest capability additions.
   `#65` is now partially addressed through Rust watchlist click hardening and
   post-add verification, while `watchlist add-bulk` remains deferred. `#40` is
   addressed for Rust as an explicit `tab switch` target handoff instead of
   persistent CDP reconnect logic. `#43` is addressed by the existing explicit
   `tv screenshot --output <PATH>` contract plus tests that parent directories
   are created and `--output` is required before connecting. `#89` has been
   audited as a mixed fork bundle; its only near-term Rust candidate, the
   read-only `tv data labels` default/truncation hardening slice, is addressed.

6. Keep workflow packs, dashboards, and Node-only maintenance outside the Rust
   CLI unless a separate investigation proves core CLI value. This includes
   strategy/rules JSON packs, custom scanners, MCP/Docker support, npm lockfile
   fixes, ESLint setup, and JavaScript dependency updates.

## Newest-first triage

| PR | Category | Rust disposition |
| --- | --- | --- |
| [#100](https://github.com/tradesdontlie/tradingview-mcp/pull/100) `fix(launch): detect TradingView Microsoft Store install on Windows` | `bugfix` | Addressed as Rust launch discovery input. Rust now checks running-process and `Get-AppxPackage` paths without changing the no-kill default. |
| [#98](https://github.com/tradesdontlie/tradingview-mcp/pull/98) `Add crypto swing-trading rules config` | `workflow/helper` | Do not add to core CLI. This is a personal rules/config pack better suited to downstream repos or user-facing examples outside the binary. |
| [#97](https://github.com/tradesdontlie/tradingview-mcp/pull/97) `fix(pine): resilient Pine Editor detection during state transitions` | `bugfix` | Addressed by Rust Pine Editor hardening: direct Monaco fast path plus repeated panel-open trigger during polling. |
| [#96](https://github.com/tradesdontlie/tradingview-mcp/pull/96) `fix(data): DOM-scrape fallback for strategy results + trades` | `bugfix` | Partially addressed. Rust keeps internal strategy data reads as the fast path and adds visible Strategy Tester DOM fallback for strategy metrics and trades. |
| [#95](https://github.com/tradesdontlie/tradingview-mcp/pull/95) `fix(pine): match Add/Update-on-chart buttons by title attr` | `bugfix` | Covered. Rust already reads `textContent`, `aria-label`, and `title` in Pine button labels; the hardening slice kept that behavior under test. |
| [#94](https://github.com/tradesdontlie/tradingview-mcp/pull/94) `chore(cdp): env-var overrides for TV CDP host/port` | `maintenance` | Already covered by Rust `TV_CDP_HOST` and `TV_CDP_PORT`. No action unless future evidence shows a missing target-specific path. |
| [#93](https://github.com/tradesdontlie/tradingview-mcp/pull/93) `fix: detect MSIX/WindowsApps TradingView install using PowerShell` | `bugfix` | Addressed by the same Rust launch discovery slice as `#100`. |
| [#92](https://github.com/tradesdontlie/tradingview-mcp/pull/92) `feat: make CDP host/port configurable via environment variables` | `feature` | Already covered by Rust transport config. No action. |
| [#91](https://github.com/tradesdontlie/tradingview-mcp/pull/91) `fix: layout_switch dismisses unsaved-changes dialog in non-English locales` | `bugfix` | Rust deliberately does not auto-dismiss unsaved-change dialogs for `layout switch`. Treat as future policy research, not an immediate bugfix. |
| [#90](https://github.com/tradesdontlie/tradingview-mcp/pull/90) `fix: TV Desktop 3.1.0 compat for data.trades / data.strategy / data.equity` | `bugfix` | Addressed in Rust by preferring `StrategyScript` source detection and `_reportData.performance`, `_reportData.trades`, and `_reportData.buyHold` when available. |
| [#89](https://github.com/tradesdontlie/tradingview-mcp/pull/89) `Add dependency injection to drawing functions and update tests` | `mixed` | Audited in `docs/notes/upstream-pr-89-hidden-surface-audit-2026-04-25.md` and `docs/notes/screener-hotlist-upstream-feasibility-2026-04-25.md`. Do not cherry-pick the fork bundle. The read-only `tv data labels` default/truncation hardening has been addressed in Rust; Hotlist REST reads are now classified as a separate near-term read-only candidate. |
| [#86](https://github.com/tradesdontlie/tradingview-mcp/pull/86) `Feat/frankie candles pine scripts` | `workflow/helper` | Do not add to core CLI. Pine script packs belong outside this Rust binary. |
| [#80](https://github.com/tradesdontlie/tradingview-mcp/pull/80) `Fix tv_launch for TradingView v2.14.0+ (Electron 38 / Node 22)` | `bugfix` | Addressed as macOS fallback evidence. Rust now tries `open -a TradingView --args ...` after direct spawn does not make CDP ready. |
| [#79](https://github.com/tradesdontlie/tradingview-mcp/pull/79) `Fix Windows launch script for MSIX / Microsoft Store TradingView installs` | `bugfix` | Covered by the Rust launch discovery slice; script-level Chrome fallback remains out of scope. |
| [#76](https://github.com/tradesdontlie/tradingview-mcp/pull/76) `fix(windows): support MSIX-packaged TradingView Desktop in tv_launch` | `bugfix` | Partially addressed. Rust adopted the Windows MSIX discovery evidence but did not add COM/AUMID activation in this slice. |
| [#74](https://github.com/tradesdontlie/tradingview-mcp/pull/74) `Add 12hr watchlist scanner workflow and rules config` | `workflow/helper` | Keep outside core CLI. This is downstream workflow material, not a `tv` command surface. |
| [#73](https://github.com/tradesdontlie/tradingview-mcp/pull/73) `Auto-detect TradingView when installed as MSIX (Microsoft Store)` | `bugfix` | Covered by the Rust launch discovery slice. |
| [#72](https://github.com/tradesdontlie/tradingview-mcp/pull/72) `Fix symbolInfo() throwing 'evaluate is not defined'` | `bugfix` | JavaScript DI regression. Rust `tv info` uses its own evaluator path, so no direct action unless live `tv info` shows equivalent failure. |
| [#71](https://github.com/tradesdontlie/tradingview-mcp/pull/71) `Bump hono and @hono/node-server to patch moderate CVEs` | `maintenance/node-only` | Not applicable. Rust does not depend on the original MCP Node server packages. |
| [#70](https://github.com/tradesdontlie/tradingview-mcp/pull/70) `Fix Windows libuv assertion on CLI exit after fetch` | `maintenance/node-only` | Not applicable to Rust. Keep only as reminder to run Windows CI for commands that make HTTP requests. |
| [#69](https://github.com/tradesdontlie/tradingview-mcp/pull/69) `Add real-time signal dashboard, price monitor, and Sn1P3r signal evaluator` | `workflow/helper` | Do not add to core CLI. This is a dashboard/scanner product surface, not bridge replacement surface. |
| [#67](https://github.com/tradesdontlie/tradingview-mcp/pull/67) `fix: add missing bin entry in package-lock.json` | `maintenance/node-only` | Not applicable. Rust release archives and Cargo metadata are separate. |
| [#66](https://github.com/tradesdontlie/tradingview-mcp/pull/66) `feat: Stock Screener tools + screen/filter/column management` | `feature` | Investigated in `docs/notes/screener-hotlist-upstream-feasibility-2026-04-25.md`. Do not import the whole UI automation bundle. A read-oriented UI Screener slice needs live UI evidence; filter/screen/column mutations remain deferred. |
| [#65](https://github.com/tradesdontlie/tradingview-mcp/pull/65) `feat: add watchlist_remove, watchlist_add_bulk, fix click handling` | `feature/bugfix` | Partially addressed: Rust has `watchlist add/remove`, watchlist controls now use coordinate-based `MouseEvent` dispatch, and `watchlist add` verifies the symbol afterward. Bulk add remains deferred as operator convenience. |
| [#64](https://github.com/tradesdontlie/tradingview-mcp/pull/64) `feat: add tv_ensure and tv_reconnect tools` | `feature` | Defer. Rust `tv launch` and `tv status` cover the basic preflight path; reconnect/reload is a stronger side effect and needs separate safety design. |
| [#62](https://github.com/tradesdontlie/tradingview-mcp/pull/62) `fix(drawing): restore DI in listDrawings, getProperties, removeOne, clearAll` | `bugfix` | JavaScript DI regression. Rust drawing commands use a different implementation and tests; no direct action unless smoke shows equivalent failure. |
| [#60](https://github.com/tradesdontlie/tradingview-mcp/pull/60) `feat: add draw_position tool for Long/Short position drawings` | `feature` | Addressed as Rust `tv draw position`, a chart-local mutation that creates native TradingView Long/Short position drawings from entry, stop, and target price levels and returns an `entity_id` for cleanup with `draw remove`. |
| [#54](https://github.com/tradesdontlie/tradingview-mcp/pull/54) `security: remove ui_evaluate tool` | `security` | Addressed in Rust by default-disabling `tv ui eval` behind `TV_ALLOW_UNSAFE_UI_EVAL=1`, while retaining the old compatibility surface for explicit unsafe use. |
| [#53](https://github.com/tradesdontlie/tradingview-mcp/pull/53) `feat: support running MCP server inside a Docker container` | `feature/node-only` | Mostly not applicable. MCP server and containerized Node connection are outside this Rust CLI. Host-header behavior is only relevant if Rust later supports non-local CDP hosts. |
| [#52](https://github.com/tradesdontlie/tradingview-mcp/pull/52) `Fix Windows MSIX install detection in tv_launch` | `bugfix` | Covered by the Rust launch discovery slice. |
| [#51](https://github.com/tradesdontlie/tradingview-mcp/pull/51) `feat: improve strategy detection and add DOM metrics fallback` | `bugfix/feature` | Partially addressed through improved strategy detection and DOM metric fallback, without adding the upstream debug/evaluate-js surface. |
| [#50](https://github.com/tradesdontlie/tradingview-mcp/pull/50) `feat: add Korean locale support for Pine compile buttons` | `bugfix` | Addressed by adding Korean Add/Update-on-chart labels to safe `pine compile` and broad `pine raw-compile` matching. |
| [#49](https://github.com/tradesdontlie/tradingview-mcp/pull/49) `Fix getChartApi not defined in drawing management functions` | `bugfix` | JavaScript DI regression. Rust drawing implementation is separate; no direct action unless live drawing smoke reveals a similar problem. |
| [#47](https://github.com/tradesdontlie/tradingview-mcp/pull/47) `Add development scripts, MCP config, and .DS_Store to gitignore` | `workflow/helper` | Do not add. It mixes local strategy scripts, MCP config, and repo hygiene for the original Node project. |
| [#46](https://github.com/tradesdontlie/tradingview-mcp/pull/46) `Add Apex Scalp Scanner` | `workflow/helper` | Do not add to core CLI. External APIs, scanners, and strategy packs belong downstream. |
| [#45](https://github.com/tradesdontlie/tradingview-mcp/pull/45) `Init ESLint and debugging capabilities` | `maintenance/node-only` | Not applicable except as historical reminder that JS evaluate helpers were introduced for development, not Rust CLI design. |
| [#43](https://github.com/tradesdontlie/tradingview-mcp/pull/43) `feat: add output_dir parameter to screenshot tools` | `feature` | Addressed for Rust by the existing explicit `tv screenshot --output <PATH>` file path contract. Rust does not add `--output-dir`; tests now lock that parent directories are created and `--output` is required before CDP connection. |
| [#40](https://github.com/tradesdontlie/tradingview-mcp/pull/40) `fix: reconnect CDP client after tab switch` | `bugfix` | Addressed for Rust as explicit target handoff: Rust commands reconnect per process, and `tv tab switch` now returns `target_id`, `target_env.TV_CDP_TARGET_ID`, and `next_command_hint` so follow-up commands can avoid ambiguity without persistent reconnect logic. |
| [#39](https://github.com/tradesdontlie/tradingview-mcp/pull/39) `fix: default screenshot region to 'full' when unspecified` | `bugfix` | Not applicable. Rust requires explicit screenshot region through clap. |
| [#35](https://github.com/tradesdontlie/tradingview-mcp/pull/35) `feat: add data_get_pine_shapes for reading plotshape/plotchar signals` | `feature` | Addressed as Rust `tv data shapes`, a read-only command that complements current line/label/table/box reads by returning visible Pine `plotshape()` / `plotchar()` signal metadata and bar OHLC when available. |
| [#34](https://github.com/tradesdontlie/tradingview-mcp/pull/34) `feat: rename draw_shape to draw, expand to 80+ tools` | `feature` | Defer. Rust already has a narrower drawing lifecycle surface; expanding to many drawing tools risks API sprawl without a concrete workflow. |
| [#33](https://github.com/tradesdontlie/tradingview-mcp/pull/33) `fix: input sanitization and JS injection prevention` | `security` | Mostly already covered by Rust serialization and finite-number helpers. Keep as regression-test inspiration when touching command inputs. |
| [#27](https://github.com/tradesdontlie/tradingview-mcp/pull/27) `Improve Windows detection and runtime validation` | `bugfix/feature` | Windows detection evidence is covered by the Rust launch discovery slice. Runtime chart-type/layout/replay validation remains separate and should only be added if Rust's current validation blocks valid TradingView states. |
| [#18](https://github.com/tradesdontlie/tradingview-mcp/pull/18) `Fix tv_launch for TradingView v2.14.0+` | `bugfix` | Addressed as older macOS/Electron fallback evidence alongside `#80`. |
| [#12](https://github.com/tradesdontlie/tradingview-mcp/pull/12) `Add trading tools and trade journaling documentation` | `feature/workflow` | Defer. Broker account positions/orders and trade journaling are outside the current safe core CLI boundary unless a separate investigation proves user value and safety. |

## Evidence-gated implementation candidates

1. Windows COM/AUMID launch activation, only if needed.
   Rust now discovers Windows AppX/MSIX installs and attempts a direct executable
   launch. If Windows live smoke shows packaged-app direct launch cannot pass
   the debug port, plan a Windows-specific activation slice based on
   `IApplicationActivationManager` evidence from upstream `#76`.

2. Watchlist bulk add, only if a downstream/operator workflow needs batched
   account mutation.
   Revisit the bulk-add part of `#65` only if a downstream workflow needs
   batched account watchlist mutation. It should include duplicate handling,
   per-symbol verification, partial-failure reporting, and a cleanup story
   before implementation.

3. Scanner, hotlist, and layout dialog behavior remain evidence-gated.
   `#66` stock screener and `#89` hotlist reads are separated in
   `docs/notes/screener-hotlist-upstream-feasibility-2026-04-25.md`. The
   strongest near-term candidate is a narrow read-only Hotlist REST command.
   UI Screener dialog reads need live UI evidence, while filter/screen/column
   mutations and workflow scanner packs should stay outside the core CLI unless
   separate evidence proves they belong here. `#91` unsaved-layout dialog
   auto-dismiss should also remain deferred; Rust should not dismiss
   unsaved-change dialogs unless a dedicated safety policy is written. No
   current screenshot-output follow-up remains from `#43`, and the `tv data
   labels` default/truncation follow-up from `#89` has been addressed.

## Assumptions

- This note only classifies upstream PRs and proposes Rust follow-up work.
- No upstream PR should be cherry-picked into Rust without a dedicated Rust
  design pass.
- MCP server implementation remains not planned.
- Downstream workflow packs remain outside the Rust core CLI by default.
