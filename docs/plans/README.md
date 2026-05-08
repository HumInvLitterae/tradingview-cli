# Plans

This directory contains active or future ExecPlans for the Rust-native `tv`
CLI. Completed implementation plans are archived so this root stays useful
for release and next-phase planning.

## Current and future plans

Current plan:

- `tradingview-cli-desktop-quote-data-rtc-source-design.md`: fix the source
  boundary and feasibility criteria for treating TradingView `qsd.rtc`
  quote-data WebSocket readback as a possible future `quote-data` source,
  without mixing it into chart-source quote or scanner extended-hours payloads.

Recently completed:

- `tradingview-cli-after-hours-websocket-correlation.md`: correlated compact
  right-side visible after-hours price samples with bounded CDP WebSocket
  quote-data candidates such as `qsd.rtc`, while avoiding quote-session
  subscription and broad DOM probes.
- `tradingview-cli-desktop-after-hours-widget-store-evidence.md`: added an
  opt-in right-panel widget smoke and recorded that scoped React props exposed
  regular quote-like fields, but not the visible after-hours price token.
- `tradingview-cli-desktop-after-hours-network-source-evidence.md`: added an
  opt-in CDP Network/WebSocket smoke and recorded that a bounded RKLB run saw
  symbol-related traffic but not the visible after-hours price token.
- `tradingview-cli-desktop-after-hours-panel-source-evidence.md`: identified
  the visible Desktop right-panel after-hours price as a separate visible UI
  source and narrowed the RKLB value to the right-side detail widget status /
  price nodes.
- `tradingview-cli-desktop-quote-session-live-evidence.md`: recorded
  postmarket Desktop quote-session phase evidence and hardened phase matching
  for TradingView's hyphenated phase names. Premarket evidence remains waiting
  for the relevant market phase.
- `tradingview-cli-v0.13-pre-release-audit.md`: audited completed v0.13
  source/session boundary and contract-hardening work before release
  readiness.

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
- Desktop-free market data read polish, including extended-hours quotes,
  scanner extended-hours columns, scanner metainfo, ordered batch quotes,
  explicit quote source selection, typed market/scanner Rust APIs, and the
  v0.4 market data lane review
- lab-gated browserless historical bars prototype
- lab bars evidence review and v0.5 data-lane boundary
- Computer Use boundary docs and skills cleanup
- Codex app Computer Use visual recovery skill research and deferral
- runtime market-data interpretation and scanner/Screener result-analysis
  skills
- fundamentals read and field-group additions
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
- Desktop-free historical bars feasibility and WebSocket lab boundary research
- full-page Screener target opening through existing-target reuse, CDP target
  creation attempt, and bounded Desktop new-tab tile fallback
- shared TradingView Desktop app-window/new-tab helper extraction before
  release readiness
- workspace package metadata and dependency centralization
- v0.4.0 release readiness
- quote chart-source readiness hardening for stale-bar guard
- v0.4.1 release readiness
- v0.5 pre-release refactor audit
- v0.5.0 release readiness
- quote chart-source stable readiness hardening
- chart-source quote opt-in live endurance smoke
- v0.5.1 release readiness
- stream observation controls for bounded duration, max-event, and heartbeat
  JSONL events
- stream source taxonomy metadata for sample and heartbeat JSONL events
- Desktop readiness integrated read command
- screenshot source taxonomy and visual evidence metadata
- core Desktop-backed read source taxonomy metadata
- Desktop-free market and scanner read source taxonomy metadata
- root `tv --version` / `tv -V` support
- public documentation and agent-guide audience cleanup
- v0.6.0 release readiness
- first `tv observe chart` workflow command
- opt-in `tv observe chart` JSONL contract live smoke
- opt-in lab-gated `tv bars` JSON contract live smoke
- public-safe fundamentals/events scanner field evidence
- v0.7 observation workflow guide and runtime skill alignment
- fundamentals earnings and dividends field enrichment
- v0.7 pre-release completion and refactor audit
- v0.7.0 release readiness
- first Desktop-free `tv snapshot <SYMBOL>` evidence packet
- snapshot workflow documentation and runtime skill alignment
- opt-in `tv snapshot` JSON contract live smoke
- v0.8 pre-release completion and refactor audit
- v0.8.0 release readiness
- chart-source quote concurrency audit and realtime source strategy decision
- v0.9.0 roadmap and Desktop-free compare planning
- first Desktop-free `tv compare <SYMBOL>...` comparison packet
- opt-in `tv compare <SYMBOL>...` JSON contract live smoke
- v0.9 pre-release completion and refactor audit
- v0.9.0 release readiness
- v0.10.0 roadmap and first compare-summary planning
- v0.10 compare summary readback
- v0.10 compare workflow decision table
- v0.10 pre-release completion and refactor audit
- v0.10.0 release readiness
- v0.11 compare contract metadata
- v0.11 pre-release completion and refactor audit
- v0.11.0 release readiness
- v0.12 compare follow-up contract polish
- v0.12 compare missing evidence readback
- v0.12 pre-release completion and refactor audit
- v0.12.0 release readiness
- v0.13 chart-source quote session-boundary metadata
- v0.13 snapshot contract metadata alignment
- v0.13 follow-up vocabulary alignment

For command contract details, prefer the notes under `docs/notes/` before
reading archived implementation plans.
