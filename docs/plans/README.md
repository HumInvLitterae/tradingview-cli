# Plans

This directory contains implementation plans for the Rust-native `tv` CLI.

## Current plan

- `tradingview-cli-release-builds.md`: tag-triggered GitHub Release builds for Linux, macOS, and Windows binaries.

## Completed release-readiness plans

- `tradingview-cli-docs-release-readiness-cleanup.md`: public README, plan archive, and MIT license cleanup.

## Archived plans

Completed historical ExecPlans live under `docs/plans/archives/`. These files explain how the current CLI surface was built and why key contract decisions were made.

Older filenames used labels such as `v1` or `v1-33`. Those labels were execution-slice identifiers, not Cargo package versions and not public application versions. Archived filenames now omit those labels to avoid confusion with the package version in `Cargo.toml`.

Important archived plans:

- `archives/tradingview-cli-remaining-migration-closure.md`: closes the known old JavaScript CLI migration surface.
- `archives/tradingview-cli-rust-initial-implementation.md`: first Rust `tv` implementation plan.
- `archives/tradingview-cli-launch.md`: bounded TradingView Desktop launcher.
- `archives/tradingview-cli-stream-read.md`: read-only JSONL stream commands.
- `archives/tradingview-cli-pine-save.md`: explicit Pine save contract.
- `archives/tradingview-cli-layout-list.md`: saved layout list and switch history.
- `archives/tradingview-cli-draw-clear.md`: bulk drawing cleanup safeguards.
- `archives/tradingview-cli-bootstrap-and-bridge-replacement.md`: initial bootstrap and bridge replacement framing.

Archived implementation slices:

- `archives/tradingview-cli-advanced-data-reads.md`
- `archives/tradingview-cli-alert-create.md`
- `archives/tradingview-cli-alert-delete.md`
- `archives/tradingview-cli-alert-list.md`
- `archives/tradingview-cli-chart-region-screenshot.md`
- `archives/tradingview-cli-chart-type.md`
- `archives/tradingview-cli-data-depth.md`
- `archives/tradingview-cli-data-module-refactor.md`
- `archives/tradingview-cli-diagnostic-read-commands.md`
- `archives/tradingview-cli-draw-clear.md`
- `archives/tradingview-cli-drawing-commands.md`
- `archives/tradingview-cli-indicator-commands.md`
- `archives/tradingview-cli-launch.md`
- `archives/tradingview-cli-layout-list.md`
- `archives/tradingview-cli-ops-module-refactor.md`
- `archives/tradingview-cli-pane-mutation.md`
- `archives/tradingview-cli-pine-analyze-check.md`
- `archives/tradingview-cli-pine-compile.md`
- `archives/tradingview-cli-pine-new-open.md`
- `archives/tradingview-cli-pine-read.md`
- `archives/tradingview-cli-pine-save.md`
- `archives/tradingview-cli-pine-set.md`
- `archives/tradingview-cli-read-provider-migration.md`
- `archives/tradingview-cli-read-utilities.md`
- `archives/tradingview-cli-replay-autoplay.md`
- `archives/tradingview-cli-replay-basic-controls.md`
- `archives/tradingview-cli-replay-status.md`
- `archives/tradingview-cli-replay-trade.md`
- `archives/tradingview-cli-stream-read.md`
- `archives/tradingview-cli-tab-list-switch.md`
- `archives/tradingview-cli-tab-new-close.md`
- `archives/tradingview-cli-watchlist-add.md`
- `archives/tradingview-cli-watchlist-remove.md`

For command contract details, prefer the notes under `docs/notes/` before reading archived implementation plans.
