# Plans

This directory contains active or future ExecPlans for the Rust-native `tv`
CLI. Completed implementation plans are archived so this root stays useful
for release and next-phase planning.

## Current and future plans

- `tradingview-cli-release-builds.md`: tag-triggered GitHub Release builds
  for Linux, macOS, and Windows binaries.
- `tradingview-cli-layout-adapter-split.md`: split the historical Layout
  operation adapter into facade plus watchlist and pane modules.

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

For command contract details, prefer the notes under `docs/notes/` before
reading archived implementation plans.
