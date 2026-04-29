# Plans

This directory contains active or future ExecPlans for the Rust-native `tv`
CLI. Completed implementation plans are archived so this root stays useful
for release and next-phase planning.

## Current and future plans

- `tradingview-cli-release-builds.md`: tag-triggered GitHub Release builds
  for Linux, macOS, and Windows binaries.
- `tradingview-cli-desktop-app-window-helper-split.md`: share TradingView
  Desktop app-window/new-tab helper code between `tab` operations and
  `screener open --full-page` before release readiness.

## Archived plans

Completed historical ExecPlans live under `docs/plans/archives/`. These files
explain how the current CLI surface was built, why key contract decisions were
made, and which evidence bounded deferred behavior.

Older filenames used labels such as `v1` or `v1-33`. Those labels were
execution-slice identifiers, not Cargo package versions and not public
application versions. Archived filenames omit those labels to avoid confusion
with the package version in `Cargo.toml`.

Important archived plan categories:

- initial Rust CLI bootstrap and old JavaScript CLI migration closure
- release readiness, public documentation, CI/build guardrails, and runtime
  skill packaging
- upstream pull-request follow-up slices for scanner, Screener, watchlist,
  alert, drawing, Pine, tab, quote, screenshot, launch, and internal API
  audits
- Pine `alertcondition()` alert feasibility, static discovery, preview, and
  guarded alert creation
- Screener storage/API research, mutation implementation, and stabilization
  boundaries
- Direct HTTP feasibility, Desktop-free symbol reads, and chart data readiness
  diagnostics
- first behavior-preserving binary/library crate boundary
- core contract crate extraction
- Desktop-free market crate extraction
- Desktop-free scanner and Pine support crate extraction
- application-layer split for the thin `tv` binary and library-owned runner
- CDP client and target-discovery crate extraction
- CLI package relocation under `crates/cli/`
- Screener operation adapter facade split
- Screener validation implementation split
- Screener columns implementation split
- Screener remaining engine split into state, screens, filters, and shared
  helper modules
- Alert operation adapter split into list, create, indicator, delete, and
  payload modules
- Layout operation adapter split into watchlist and pane modules
- Pine Editor operation adapter split into runtime, source, scripts, and
  compile modules
- Drawing, Replay, and chart-dependent Market medium adapter split
- Generic UI automation adapter split into DOM, input, selector, and eval
  modules
- first in-package domain/service boundary for Watchlist validation,
  aggregation, and payload normalization
- second in-package domain/service boundary for Alert validation,
  sanitization, fallback policy, and payload normalization
- third in-package domain/service boundary for Replay validation, timestamp
  conversion, and payload normalization
- fourth in-package domain/service boundary for Drawing request validation
- larger in-package Screener domain/service boundary for validation, target
  resolution, and storage payload shaping
- application/domain/operation-adapter dependency boundary review
- TradingView model crate extraction for I/O-free validation, request models,
  target resolution, and payload shaping
- operation adapter boundary audit after model extraction
- bounded Screener filter storage mutation audit and storage-backed numeric
  range filter modification
- full-page Screener target opening through existing-target reuse, CDP target
  creation attempt, and bounded Desktop new-tab tile fallback

For command contract details, prefer the notes under `docs/notes/` before
reading archived implementation plans.
