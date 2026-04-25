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

As of this pass, the upstream repository has 45 open PRs.

## Recommended priority

1. Security review for unrestricted page-context evaluation.
   Upstream `#54` removes `ui_evaluate` because it can run arbitrary JavaScript in
   an authenticated TradingView session. Rust currently exposes `tv ui eval` as
   an old CLI compatibility command. This should be reviewed before adding more
   feature surface.

2. Strategy data compatibility.
   Upstream `#90`, `#96`, and older `#51` report concrete TradingView Desktop
   3.1.0 breakage for strategy, trades, and equity reads. Rust currently still
   finds strategies through `metaInfo().is_price_study === false` and public
   `reportData` / `ordersData` / `equityData`-style accessors, so this evidence
   maps directly to existing Rust commands.

3. Release-user launch compatibility.
   Upstream `#100`, `#93`, `#80`, `#79`, `#76`, `#73`, `#52`, `#27`, and `#18`
   all point at launch fragility around Windows MSIX / WindowsApps installs and
   newer TradingView Desktop / Electron behavior. Rust `tv launch` already uses
   `TV_CDP_HOST` / `TV_CDP_PORT` and is bounded/no-kill by default, but it does
   not yet have a Windows MSIX discovery path or macOS fallback for rejected
   `--remote-debugging-port` direct spawn.

4. Pine Editor robustness.
   Upstream `#97`, `#95`, and `#50` describe Pine Editor detection and
   button-matching problems in recent TradingView Desktop builds and non-English
   locales. Rust already checks `textContent`, `aria-label`, and `title`, and it
   has Japanese button matching. It does not clearly cover the state-transition
   race from `#97` or Korean text from `#50`.

5. Existing command hardening and modest capability additions.
   `#65`, `#40`, `#35`, `#60`, and `#43` have possible Rust value, but they are
   lower priority than the security and known-breakage clusters above. `#65`
   partially overlaps Rust's existing `watchlist add/remove`; `#40` should be
   compared against Rust's stateless target selection before changing transport;
   `#35` and `#60` are feature additions rather than migration blockers; `#43`
   is mostly already covered by explicit screenshot output paths.

6. Keep workflow packs, dashboards, and Node-only maintenance outside the Rust
   CLI unless a separate investigation proves core CLI value. This includes
   strategy/rules JSON packs, custom scanners, MCP/Docker support, npm lockfile
   fixes, ESLint setup, and JavaScript dependency updates.

## Newest-first triage

| PR | Category | Rust disposition |
| --- | --- | --- |
| [#100](https://github.com/tradesdontlie/tradingview-mcp/pull/100) `fix(launch): detect TradingView Microsoft Store install on Windows` | `bugfix` | High-value input for a Rust `tv launch` Windows/MSIX ExecPlan. Combines running-process path detection and `Get-AppxPackage`; compare with older MSIX PRs before implementing. |
| [#98](https://github.com/tradesdontlie/tradingview-mcp/pull/98) `Add crypto swing-trading rules config` | `workflow/helper` | Do not add to core CLI. This is a personal rules/config pack better suited to downstream repos or user-facing examples outside the binary. |
| [#97](https://github.com/tradesdontlie/tradingview-mcp/pull/97) `fix(pine): resilient Pine Editor detection during state transitions` | `bugfix` | Candidate Pine hardening. Rust has explicit Monaco polling, but the reported state-transition failure should be checked against live `pine set/new/compile` smoke before changing logic. |
| [#96](https://github.com/tradesdontlie/tradingview-mcp/pull/96) `fix(data): DOM-scrape fallback for strategy results + trades` | `bugfix` | High-value input for `tv data strategy` and `tv data trades`. Rust currently has no Strategy Tester DOM fallback for these commands. |
| [#95](https://github.com/tradesdontlie/tradingview-mcp/pull/95) `fix(pine): match Add/Update-on-chart buttons by title attr` | `bugfix` | Mostly covered: Rust already includes `aria-label` and `title` in Pine button labels. Keep as evidence for tests and live smoke expectations. |
| [#94](https://github.com/tradesdontlie/tradingview-mcp/pull/94) `chore(cdp): env-var overrides for TV CDP host/port` | `maintenance` | Already covered by Rust `TV_CDP_HOST` and `TV_CDP_PORT`. No action unless future evidence shows a missing target-specific path. |
| [#93](https://github.com/tradesdontlie/tradingview-mcp/pull/93) `fix: detect MSIX/WindowsApps TradingView install using PowerShell` | `bugfix` | Same Windows/MSIX cluster as `#100`; useful implementation evidence but likely superseded by newer PRs. |
| [#92](https://github.com/tradesdontlie/tradingview-mcp/pull/92) `feat: make CDP host/port configurable via environment variables` | `feature` | Already covered by Rust transport config. No action. |
| [#91](https://github.com/tradesdontlie/tradingview-mcp/pull/91) `fix: layout_switch dismisses unsaved-changes dialog in non-English locales` | `bugfix` | Rust deliberately does not auto-dismiss unsaved-change dialogs for `layout switch`. Treat as future policy research, not an immediate bugfix. |
| [#90](https://github.com/tradesdontlie/tradingview-mcp/pull/90) `fix: TV Desktop 3.1.0 compat for data.trades / data.strategy / data.equity` | `bugfix` | High priority. The reported root cause matches Rust's current strategy-source predicate and public accessor assumptions. |
| [#89](https://github.com/tradesdontlie/tradingview-mcp/pull/89) `Add dependency injection to drawing functions and update tests` | `maintenance` | Mostly JavaScript testability/regression structure. Rust already has operation-level fake runtime tests; inspect only for hidden data/watchlist/alert behavior before ignoring. |
| [#86](https://github.com/tradesdontlie/tradingview-mcp/pull/86) `Feat/frankie candles pine scripts` | `workflow/helper` | Do not add to core CLI. Pine script packs belong outside this Rust binary. |
| [#80](https://github.com/tradesdontlie/tradingview-mcp/pull/80) `Fix tv_launch for TradingView v2.14.0+ (Electron 38 / Node 22)` | `bugfix` | Launch cluster. Useful for macOS direct-spawn fallback research; compare with `#18` before implementing. |
| [#79](https://github.com/tradesdontlie/tradingview-mcp/pull/79) `Fix Windows launch script for MSIX / Microsoft Store TradingView installs` | `bugfix` | Launch cluster. Older script-level Windows evidence; likely superseded by `#100`. |
| [#76](https://github.com/tradesdontlie/tradingview-mcp/pull/76) `fix(windows): support MSIX-packaged TradingView Desktop in tv_launch` | `bugfix` | Launch cluster. Contains broader Rust-relevant problem statement and PowerShell helper idea. |
| [#74](https://github.com/tradesdontlie/tradingview-mcp/pull/74) `Add 12hr watchlist scanner workflow and rules config` | `workflow/helper` | Keep outside core CLI. This is downstream workflow material, not a `tv` command surface. |
| [#73](https://github.com/tradesdontlie/tradingview-mcp/pull/73) `Auto-detect TradingView when installed as MSIX (Microsoft Store)` | `bugfix` | Launch cluster. Older `Get-AppxPackage` evidence; likely superseded. |
| [#72](https://github.com/tradesdontlie/tradingview-mcp/pull/72) `Fix symbolInfo() throwing 'evaluate is not defined'` | `bugfix` | JavaScript DI regression. Rust `tv info` uses its own evaluator path, so no direct action unless live `tv info` shows equivalent failure. |
| [#71](https://github.com/tradesdontlie/tradingview-mcp/pull/71) `Bump hono and @hono/node-server to patch moderate CVEs` | `maintenance/node-only` | Not applicable. Rust does not depend on the original MCP Node server packages. |
| [#70](https://github.com/tradesdontlie/tradingview-mcp/pull/70) `Fix Windows libuv assertion on CLI exit after fetch` | `maintenance/node-only` | Not applicable to Rust. Keep only as reminder to run Windows CI for commands that make HTTP requests. |
| [#69](https://github.com/tradesdontlie/tradingview-mcp/pull/69) `Add real-time signal dashboard, price monitor, and Sn1P3r signal evaluator` | `workflow/helper` | Do not add to core CLI. This is a dashboard/scanner product surface, not bridge replacement surface. |
| [#67](https://github.com/tradesdontlie/tradingview-mcp/pull/67) `fix: add missing bin entry in package-lock.json` | `maintenance/node-only` | Not applicable. Rust release archives and Cargo metadata are separate. |
| [#66](https://github.com/tradesdontlie/tradingview-mcp/pull/66) `feat: Stock Screener tools + screen/filter/column management` | `feature` | Potential future research only. It is large UI automation surface, likely outside near-term core unless downstream workflows prove strong value. |
| [#65](https://github.com/tradesdontlie/tradingview-mcp/pull/65) `feat: add watchlist_remove, watchlist_add_bulk, fix click handling` | `feature/bugfix` | Partially covered: Rust has `watchlist add` and `watchlist remove`. Bulk add and Electron click-hardening can be considered later if live smoke shows current Rust click path is fragile. |
| [#64](https://github.com/tradesdontlie/tradingview-mcp/pull/64) `feat: add tv_ensure and tv_reconnect tools` | `feature` | Defer. Rust `tv launch` and `tv status` cover the basic preflight path; reconnect/reload is a stronger side effect and needs separate safety design. |
| [#62](https://github.com/tradesdontlie/tradingview-mcp/pull/62) `fix(drawing): restore DI in listDrawings, getProperties, removeOne, clearAll` | `bugfix` | JavaScript DI regression. Rust drawing commands use a different implementation and tests; no direct action unless smoke shows equivalent failure. |
| [#60](https://github.com/tradesdontlie/tradingview-mcp/pull/60) `feat: add draw_position tool for Long/Short position drawings` | `feature` | Candidate feature only. It may be valuable, but it expands drawing semantics beyond old migration closure and needs a dedicated plan. |
| [#54](https://github.com/tradesdontlie/tradingview-mcp/pull/54) `security: remove ui_evaluate tool` | `security` | Highest-priority safety review. Rust currently exposes `tv ui eval`; decide whether to remove it, gate it, or keep it with stronger public warnings before adding more surface. |
| [#53](https://github.com/tradesdontlie/tradingview-mcp/pull/53) `feat: support running MCP server inside a Docker container` | `feature/node-only` | Mostly not applicable. MCP server and containerized Node connection are outside this Rust CLI. Host-header behavior is only relevant if Rust later supports non-local CDP hosts. |
| [#52](https://github.com/tradesdontlie/tradingview-mcp/pull/52) `Fix Windows MSIX install detection in tv_launch` | `bugfix` | Launch cluster. Older evidence for `Get-AppxPackage`; likely superseded by `#100`. |
| [#51](https://github.com/tradesdontlie/tradingview-mcp/pull/51) `feat: improve strategy detection and add DOM metrics fallback` | `bugfix/feature` | Same data strategy cluster as `#90` and `#96`; useful for implementation comparison. |
| [#50](https://github.com/tradesdontlie/tradingview-mcp/pull/50) `feat: add Korean locale support for Pine compile buttons` | `bugfix` | Candidate Pine locale hardening. Rust currently handles English and Japanese chart text, but not the Korean labels documented here. |
| [#49](https://github.com/tradesdontlie/tradingview-mcp/pull/49) `Fix getChartApi not defined in drawing management functions` | `bugfix` | JavaScript DI regression. Rust drawing implementation is separate; no direct action unless live drawing smoke reveals a similar problem. |
| [#47](https://github.com/tradesdontlie/tradingview-mcp/pull/47) `Add development scripts, MCP config, and .DS_Store to gitignore` | `workflow/helper` | Do not add. It mixes local strategy scripts, MCP config, and repo hygiene for the original Node project. |
| [#46](https://github.com/tradesdontlie/tradingview-mcp/pull/46) `Add Apex Scalp Scanner` | `workflow/helper` | Do not add to core CLI. External APIs, scanners, and strategy packs belong downstream. |
| [#45](https://github.com/tradesdontlie/tradingview-mcp/pull/45) `Init ESLint and debugging capabilities` | `maintenance/node-only` | Not applicable except as historical reminder that JS evaluate helpers were introduced for development, not Rust CLI design. |
| [#43](https://github.com/tradesdontlie/tradingview-mcp/pull/43) `feat: add output_dir parameter to screenshot tools` | `feature` | Mostly covered by Rust `tv screenshot --output <PATH>`. No immediate action. |
| [#40](https://github.com/tradesdontlie/tradingview-mcp/pull/40) `fix: reconnect CDP client after tab switch` | `bugfix` | Compare before changing Rust. Rust commands reconnect per process and support `TV_CDP_TARGET_ID`, so the stale-client bug may not apply directly. |
| [#39](https://github.com/tradesdontlie/tradingview-mcp/pull/39) `fix: default screenshot region to 'full' when unspecified` | `bugfix` | Not applicable. Rust requires explicit screenshot region through clap. |
| [#35](https://github.com/tradesdontlie/tradingview-mcp/pull/35) `feat: add data_get_pine_shapes for reading plotshape/plotchar signals` | `feature` | Candidate data-read feature. It could complement current line/label/table/box reads, but needs evidence that downstream workflows need plotshape/plotchar reads. |
| [#34](https://github.com/tradesdontlie/tradingview-mcp/pull/34) `feat: rename draw_shape to draw, expand to 80+ tools` | `feature` | Defer. Rust already has a narrower drawing lifecycle surface; expanding to many drawing tools risks API sprawl without a concrete workflow. |
| [#33](https://github.com/tradesdontlie/tradingview-mcp/pull/33) `fix: input sanitization and JS injection prevention` | `security` | Mostly already covered by Rust serialization and finite-number helpers. Keep as regression-test inspiration when touching command inputs. |
| [#27](https://github.com/tradesdontlie/tradingview-mcp/pull/27) `Improve Windows detection and runtime validation` | `bugfix/feature` | Split the evidence: Windows detection belongs to the launch cluster; runtime chart-type/layout/replay validation should only be added if Rust's current validation blocks valid TradingView states. |
| [#18](https://github.com/tradesdontlie/tradingview-mcp/pull/18) `Fix tv_launch for TradingView v2.14.0+` | `bugfix` | Launch cluster. Older but useful macOS/Electron fallback evidence; compare with `#80`. |
| [#12](https://github.com/tradesdontlie/tradingview-mcp/pull/12) `Add trading tools and trade journaling documentation` | `feature/workflow` | Defer. Broker account positions/orders and trade journaling are outside the current safe core CLI boundary unless a separate investigation proves user value and safety. |

## Next implementation candidates

1. `tv ui eval` safety decision.
   Create a short ExecPlan to decide whether to remove `ui eval`, hide it behind
   an explicit unsafe flag/environment gate, or keep it with stronger public
   warnings. The upstream security PR is directly relevant because Rust currently
   has the same class of capability.

2. `tv data strategy`, `tv data trades`, and `tv data equity` compatibility.
   Create an ExecPlan that uses the upstream `#90` / `#96` / `#51` evidence to
   improve strategy detection and add fallback paths while preserving Rust's
   `{ success, command, data }` envelope.

3. `tv launch` Windows/MSIX and Electron compatibility.
   Create an ExecPlan that merges the duplicate launch PR evidence into a
   Rust-native design. It should keep the current no-kill default and include
   platform-specific tests for candidate discovery and fallback selection.

4. Pine Editor robustness.
   Create a narrower hardening plan only after a live smoke or code comparison
   shows current Rust behavior still misses a reported state or locale.

## Assumptions

- This note only classifies upstream PRs and proposes Rust follow-up work.
- No upstream PR should be cherry-picked into Rust without a dedicated Rust
  design pass.
- MCP server implementation remains not planned.
- Downstream workflow packs remain outside the Rust core CLI by default.
