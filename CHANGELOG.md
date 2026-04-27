# Changelog

All notable changes to this project are recorded here.

This project uses Git tags such as `v0.2.0` for public releases. The Cargo
package version omits the leading `v`.

## Unreleased

### Added

- Added a `v0.3.0` roadmap that prioritizes upstream PR re-checks,
  non-public API stabilization, direct HTTP feasibility, and a later
  binary/library crate split.
- Started a refreshed upstream pull-request re-check note after the `v0.2.0`
  release.
- Added global `tv --target-id <CDP_TARGET_ID>` target selection, structured
  target handoff hints, and non-mutating scanner-backed symbol quote reads with
  chart-switch freshness checks.

## v0.2.0 - 2026-04-27

This feature release expands market discovery, Stock Screener operation, and
API-backed account mutations after `v0.1.1`. It also refreshes release
packaging and project documentation before the next investigation phase.

### Added

- Added a durable Screener completion and stabilization note that records the
  implemented Screener surface, deferred boundaries, and live-smoke priorities.
- Added a public-safe internal TradingView API reference and Screener
  storage/API stabilization audit for future reliability work.
- Added a cross-command internal API replacement audit that classifies DOM
  dependencies, identifies watchlist and alert-create replacement candidates,
  and records which DOM boundaries should remain intentional.
- Added a runtime `screener-workflow` skill and refreshed packaged agent
  guidance for current scanner, Screener, watchlist, and alert operation.
- Added a post-next-release direct HTTP feasibility plan and refreshed the
  internal API replacement notes after watchlist and alert API-backed work.
- Archived completed planning documents so `docs/plans/` now keeps only
  active/future plans and the plan index at the root.
- Added stable architecture, development, and release packaging guides under
  `docs/`, and archived the older development guideline note.
- Added read-only `tv scanner scan` for basic TradingView scanner REST reads
  with exchange, column, sort, limit, and numeric filter options.
- Added practical `tv scanner scan` filters for stock type, subtype, sector,
  industry, price change, relative volume, and price-to-earnings bounds.
- Added technical `tv scanner scan` filters for average volume, weekly/monthly/
  quarterly performance, RSI, and TradingView recommendation score, with signed
  daily-change and performance bounds.
- Added `tv watchlist add-bulk` for bounded, verified batch watchlist additions
  with duplicate reporting and optional partial-success output.
- Added API-backed `tv watchlist add` and `tv watchlist remove` mutation paths
  for the active custom watchlist, with readback post-checks and DOM fallback
  when the symbols-list API cannot be used before mutation.
- Added API-backed `tv alert create` mutation with alert-list readback
  verification and visible-dialog fallback only before the create request is
  sent.
- Added menu-visible `tv screener screens list` and
  `tv screener screens switch --name <NAME> [--dry-run]` commands for prepared
  Screener screen operation.
- Added catalog-backed `tv screener screens list --catalog` and
  `tv screener screens switch --name <NAME> --catalog [--dry-run]` for exact
  saved-screen targeting.
- Added `tv screener screens actions` and
  `tv screener screens save [--dry-run]` for guarded active-screen menu
  inspection and exact save-action clicking.
- Added guarded Stock Screener screen lifecycle commands:
  `tv screener screens create`, `rename`, `save-as`, and `delete --dry-run`.
  Normal create, rename, and save-as are limited to test/disposable screen
  names and require active-title post-checks; normal delete now uses exact
  saved-screen storage API targeting, requires `--confirm-delete`, refuses
  active screens, and verifies post-delete absence.
- Added guarded `tv screener filters remove` and `tv screener filters clear`
  commands, including dry-run target reporting and clear-all confirmation.
  Normal remove and clear now use the saved-screen storage API on test or
  disposable screens, require storage re-fetch post-checks, and request a
  full-page Screener refresh when available.
- Added `tv screener filters actions` and preset-backed
  `tv screener filters modify --index <N>|--text <TEXT> --min <N>|--max <N>`
  for visible numeric range filters. Modify supports dry-run target reporting,
  finite input validation before CDP connection, and post-mutation visible-text
  verification.
- Extended `tv screener filters modify` with
  `--option <TEXT>` for single visible option selection on existing filter
  pills. The option path rejects mixed range/option input, reports the matched
  option in dry-run mode, clears other selected options when the UI exposes
  them, and still requires a visible-text post-check before success.
- Added guarded `tv screener filters add --name <TEXT> --min <N>|--max <N>
  [--dry-run]` for visible add-filter catalog numeric presets. The command
  validates finite input before CDP connection and reports success only after a
  new visible filter pill appears.
- Hardened Screener open-state detection so off-screen tables, toolbar buttons,
  and unrelated right-panel content do not count as an open Screener panel.
- Hardened Screener option-filter editing by closing stale transient popups
  before opening the target filter option popover.
- Added storage-backed `tv screener columns config`,
  `tv screener columns add`, normal `tv screener columns remove`, and
  `tv screener columns reorder` for active saved test screens. Normal column
  mutations use TradingView's saved-screen storage payload, are limited to
  test/disposable screen names, and require a post-save storage order check
  before success.
- Added `tv quote [SYMBOL]` for symbol-targeted quote reads with temporary chart
  switching and verified restoration.

### Fixed

- Fixed `tv alert delete --id` and `tv alert delete --all` to use the alert
  delete endpoint shape verified by live cleanup while preserving absence
  post-checks.

## v0.1.1 - 2026-04-25

This is a stability and compatibility release after the first public `v0.1.0`
release. It keeps the existing Rust-native `tv` CLI direction while folding in
the first narrow upstream pull-request follow-ups.

### Added

- Added `tv data shapes` for visible Pine `plotshape()` / `plotchar()` signal
  reads, including bar context when TradingView exposes it.
- Added `tv draw position` for native Long/Short position drawings with an
  `entity_id` cleanup path through `tv draw remove`.
- Added `tv data labels` truncation metadata: `available_labels`, `limit`, and
  `truncated`.

### Changed

- Increased the default `tv data labels` cap from 50 to 500 while keeping
  `--max <N>` as the explicit override.
- Improved `tv launch` compatibility for Windows Microsoft Store/MSIX-style
  install discovery and newer macOS TradingView Desktop launch behavior.
- Improved strategy data reads with `StrategyScript` detection, `_reportData`
  paths, and visible Strategy Tester DOM fallbacks.
- Improved Pine Editor detection and compile button matching, including
  transition-state polling and Korean Add/Update-on-chart labels.
- Improved `tv tab switch` output with explicit target handoff metadata for
  follow-up commands.
- Hardened watchlist add/remove DOM click handling and post-action verification.

### Security

- Disabled `tv ui eval` by default. It now requires
  `TV_ALLOW_UNSAFE_UI_EVAL=1` before connecting to the authenticated TradingView
  page context.

### Tests and docs

- Added screenshot output contract tests for explicit `--output` handling and
  parent directory creation.
- Audited original upstream pull requests and recorded remaining candidates as
  evidence-gated follow-up rather than automatic core CLI scope.
- Refreshed release and migration notes around the completed upstream follow-up
  slices.

## v0.1.0 - 2026-04-25

Initial public release of the Rust-native `tv` CLI.

### Included

- CLI-first `tv` binary for TradingView Desktop over Chrome DevTools Protocol.
- Known old JavaScript CLI command migration coverage for the Rust replacement
  surface, with Rust JSON envelopes that place successful payloads under
  `data`.
- Read, chart-control, watchlist, alert, layout, indicator, drawing, Pine, tab,
  replay, stream, launch, screenshot, and compatibility UI command groups.
- GitHub Actions CI for formatting, linting, and tests.
- Tag-triggered GitHub Release builds for Linux, macOS, and Windows archives
  with checksums.
- User-facing agent guides and runtime TradingView CLI skills in release
  archives.
