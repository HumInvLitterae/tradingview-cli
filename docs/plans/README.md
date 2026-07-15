# Plans

This directory contains active or future ExecPlans for the Rust-native `tv`
CLI. Completed implementation plans are archived so this root stays useful
for release and next-phase planning.

## Current and future plans

The active ExecPlan is
`docs/plans/tradingview-cli-launch-environment-hardening.md`. It covers
`ELECTRON_RUN_AS_NODE` removal from direct spawn and the macOS system launcher,
plus confirmed child-exit classification that cannot override a successful
fallback. The no-kill default and live-but-not-ready warning response remain
unchanged. Implementation and full validation are complete; independent
implementation review is pending.

The ordered `v0.28.0` implementation inventory is maintained in
`docs/v0.28-work-items.md`. Pine and indicator mutation safety, current-build
Desktop compatibility, and bounded chart workflow candidates are promoted into
separate ExecPlans only when the preceding required item is complete and green.

Recently completed:

- `tradingview-cli-current-build-indicator-insertion.md`: replaced the legacy
  indicator add path with exact metainfo resolution, awaited chart-owned
  insertion, strict input and typed-result validation, first-snapshot cleanup,
  public-safe failures, and pinned executable coverage. Two correction waves
  and focused re-review are green; the completed plan now lives under
  `docs/plans/archives/`.

- `tradingview-cli-active-pine-editor-compatibility.md`: established semantic
  Pine Editor ownership, trigger-linked saved-script selection, verified
  Save-bound identity, line-ending-safe source checks, platform-correct save
  shortcuts, and public-safe fail-closed diagnostics. The authorized live
  matrix and focused closeout review are green; the completed plan now lives
  under `docs/plans/archives/`.
- `tradingview-cli-pine-saved-script-binding-safety.md`: replaced source-only
  saved-script opening with verified slot rebinding and completed its bounded
  two-script live matrix without changing the unintended script. The completed
  plan now lives under `docs/plans/archives/`.

- `tradingview-cli-v0.27.0-release-readiness.md`: prepared, validated, and
  released `v0.27.0` with aligned versions, changelog, release notes, package
  contents, Rust baseline, and separately pinned JavaScript contract gate. The
  completed plan now lives under `docs/plans/archives/`.
- `tradingview-cli-v0.27-pre-release-audit.md`: froze the promoted v0.27
  feature scope, audited contracts, architecture, docs, skills, tests, and the
  separate pinned-Node JavaScript contract gate, and found no release blocker
  or required pre-release refactor. Independent review reported no findings;
  the completed plan now lives under `docs/plans/archives/`.
- `tradingview-cli-study-value-identity.md`: added public-safe identity,
  compact inputs, visibility, and conservative study kind to one-shot and
  streaming study-value rows while preserving existing values and study state.
  The completed plan now lives under `docs/plans/archives/`.
- `tradingview-cli-screenshot-render-ready-wait.md`: added an opt-in bounded
  readiness wait before full, chart, or Strategy Tester screenshots while
  preserving immediate capture by default and refusing to capture after a
  readiness timeout. The completed plan now lives under
  `docs/plans/archives/`.
- `tradingview-cli-visible-range-history-paging-implementation.md`: added
  bounded selected-chart history loading, deterministic stopping and clamp
  policy, composed progress/absolute deadlines, additive diagnostics, and
  pre-connection validation for the existing bounded `tv range` setter. The
  completed plan now lives under `docs/plans/archives/`.
- `tradingview-cli-visible-range-history-paging-contract.md`: defined and
  independently reviewed bounded selected-chart history paging, deadline and
  terminal precedence, endpoint coverage, discrete-bar matching, viewport
  application, validation placement, and source boundaries. The completed
  plan now lives under `docs/plans/archives/`.
- `tradingview-cli-indicator-search-positive-results.md`: implemented and
  tested a local positive-result search trial, then removed it when repeated
  live readiness was not reproducible. The completed deferred plan now lives
  under `docs/plans/archives/`.
- `tradingview-cli-indicator-search-parser-feasibility.md`: compared upstream
  quality, proved a strict class-free structural parser across three queries,
  and recorded limited go with fail-closed no-result behavior. The completed
  plan now lives under `docs/plans/archives/`.
- `tradingview-cli-indicator-search-contract.md`: recorded the current
  Indicators dialog observation limit, defined the provisional search and
  failure contracts, and gated implementation behind a reviewed stop/go
  parser-feasibility slice. The completed plan now lives under
  `docs/plans/archives/`.
- `tradingview-cli-strategy-tester-compatibility.md`: hardened current-build
  Strategy Tester detection and report selection across metrics, trades, and
  equity, added public-safe shared context, and completed independent review.
  The completed plan now lives under `docs/plans/archives/`.
- `tradingview-cli-v0.26.0-release-readiness.md`: prepared and validated the
  released `v0.26.0` package, changelog, release notes, README examples, and
  explicit archive contents. The completed plan now lives under
  `docs/plans/archives/`.
- `tradingview-cli-cdp-windows-refusal-fixture.md`: replaced platform-dependent
  released-port and fixed-port assumptions with controlled reconnect-capable
  failure fixtures. Cross-platform CI run `29173925167` and focused review are
  green; the completed plan now lives under `docs/plans/archives/`.
- `tradingview-cli-canonical-history-sanitation.md`: removed the sensitive
  machine-specific path from canonical main and release-tag history, completed
  remote and fresh-clone verification, and realigned the primary clone. The
  completed plan now lives under `docs/plans/archives/`.

- `tradingview-cli-machine-specific-path-cleanup.md`: removed 79 redundant
  username-specific regex branches across 61 tracked documents. Independent
  review reported no findings; the plan now lives under
  `docs/plans/archives/`.
- `tradingview-cli-v0.26-pre-release-audit.md`: audited all six v0.26
  robustness gates, found no release-blocking architecture issue, and completed
  independent review and focused re-review. The plan now lives under
  `docs/plans/archives/`.
- `tradingview-cli-bounded-multi-symbol-concurrency.md`: measured and added
  input-ordered four-wide concurrency to bounded Desktop-free quote, compare,
  and event reads. Independent review and focused re-review are complete; the
  plan now lives under `docs/plans/archives/`.
- `tradingview-cli-http-error-taxonomy.md`: made HTTP transport, timeout,
  remote-status, and payload failures consistent across Market, Scanner, Pine,
  and CDP while keeping diagnostics public-safe. Independent review reported
  no findings; the plan now lives under `docs/plans/archives/`.
- `tradingview-cli-bars-heartbeat-wire-format.md`: corrected browserless bars
  heartbeat pong framing, preserved receive compatibility, and verified the
  canonical form through deterministic tests plus a public-safe live probe.
  Independent re-review reported no remaining findings; the plan now lives
  under `docs/plans/archives/`.
- `tradingview-cli-transport-deadlines-http-reuse.md`: bounded public HTTP,
  CDP HTTP/WebSocket, and bars WebSocket transport operations while reusing
  configured HTTP clients in sequential multi-read workflows. Review
  corrections and focused re-review are complete; the plan now lives under
  `docs/plans/archives/`.
- `tradingview-cli-cdp-event-buffering-deadlines.md`: preserved CDP events that
  arrive during method-response waits, added a FIFO bounded to 1024 events and
  8 MiB of encoded event text, and enforced absolute response/event deadlines.
  The completed plan now lives under `docs/plans/archives/`.
- `tradingview-cli-output-broken-pipe.md`: made one-shot JSON and JSONL output
  stop cleanly when a downstream consumer closes its pipe, without panic,
  duplicate error output, or continued polling. The completed plan now lives
  under `docs/plans/archives/`.
- `tradingview-cli-v0.25.0-release-readiness.md`: prepared and validated the
  released `v0.25.0` package, changelog, release notes, README asset examples,
  and explicit archive contents. The completed plan now lives under
  `docs/plans/archives/`.
- `tradingview-cli-v0.25-pre-release-audit.md`: audited completed `v0.25.0`
  chart-backed workflow maturity before release readiness, including source
  boundaries, docs, runtime skills, tests, and architecture posture.
- `tradingview-cli-strategy-tester-screenshot.md`: added
  `tv screenshot --region strategy --output <PATH>` as non-mutating visual
  evidence for the visible TradingView Strategy Tester panel.
- `tradingview-cli-replay-log-ohlcv-attachment.md`: added opt-in
  selected-chart OHLCV summary attachment to bounded `tv replay log --steps
  <N>` JSONL events while keeping Replay logs separate from export,
  screenshots, and Desktop-free `tv bars`.
- `tradingview-cli-events-compare-readback.md`: added `tv events compare
  <SYMBOL>...` as ordered multi-symbol scanner-backed earnings / dividends
  readback while preserving the single-symbol `events.v1` contract.
- `tradingview-cli-chart-backed-compare-command.md`: added `tv chart compare
  <SYMBOL>...` as a separated Desktop-backed selected-chart evidence workflow
  while keeping `tv compare` and `tv watch compare` scanner-backed.
- `tradingview-cli-v0.24.0-release-readiness.md`: prepared the `v0.24.0`
  release state, including version, changelog, release notes, README asset
  examples, package staging, runtime-skill packaging, and release validation.
- `tradingview-cli-runtime-skill-context-cleanup.md`: reduced runtime skill
  context load by keeping common workflows in `SKILL.md` and moving detailed
  source-boundary notes into skill references before `v0.24.0` release
  readiness.
- `tradingview-cli-v0.24-pre-release-audit.md`: audited completed `v0.24.0`
  launch hardening, `tv bars` symbol resolution, and narrow `tv events`
  readback before release readiness, including docs, skills, tests, source
  boundaries, and architecture posture.
- `tradingview-cli-events-symbol-scoped-readback.md`: added narrow
  `tv events <SYMBOL>` readback for scanner-backed earnings and dividends
  fields, shaped as `events.v1` without adding a full event calendar source.
- `tradingview-cli-bars-symbol-resolution.md`: resolved bare `tv bars`
  symbols through Desktop-free TradingView symbol search when possible, while
  reporting both requested and resolved symbols and keeping `EXCHANGE:SYMBOL`
  as the explicit override.
- `tradingview-cli-launch-process-handling.md`: hardened `tv launch` process
  handling so agent-driven launches do not accidentally terminate TradingView
  Desktop, while keeping `--kill-existing` explicit.
- `tradingview-cli-v0.23.0-release-readiness.md`: prepared the `v0.23.0`
  release state, including version, changelog, release notes, README asset
  examples, package staging, and release validation.
- `tradingview-cli-v0.23-pre-release-audit.md`: audited completed `v0.23.0`
  export / Replay / compare / events workflow maturity, including contracts,
  docs, tests, source boundaries, and architecture posture before release
  readiness.
- `tradingview-cli-events-feasibility.md`: planned standalone `tv events`
  feasibility and source boundaries while keeping current earnings and
  dividends reads as scanner-backed fundamentals fields.
- `tradingview-cli-chart-backed-compare-contract.md`: planned chart-backed
  compare source boundaries before adding any stable Desktop-backed compare
  command.
- `tradingview-cli-replay-step-log-contract.md`: added bounded
  `tv replay log --steps <N>` JSONL workflow evidence without adding stable
  Replay export.
- `tradingview-cli-selected-chart-export-command.md`: added
  `tv export chart-bars`, a narrow Desktop-backed selected-chart export
  workflow that keeps requested visible range, range operation, chart context,
  returned bars range, and range-match diagnostics separate from Desktop-free
  `tv bars`.
- `tradingview-cli-v0.22.0-release-readiness.md`: prepared the `v0.22.0`
  release state, including version, changelog, release notes, README asset
  examples, package staging, and release validation.
- `tradingview-cli-v0.22-pre-release-audit.md`: audited completed `v0.22.0`
  observation / export workflow maturity, including contracts, docs, tests,
  source boundaries, and architecture posture before release readiness.
- `tradingview-cli-evidence-follow-up-workflow.md`: clarified
  `snapshot.v1` / `compare.v1` follow-up hints with source and advisory
  metadata so agents can route next evidence checks without automatic source
  mixing or recommendations.
- `tradingview-cli-replay-extraction-feasibility.md`: added public-safe
  Replay state and operation readback to existing `tv replay ...` commands so
  Replay-based extraction can be evaluated without adding a stable export
  command or hidden fallback.
- `tradingview-cli-selected-chart-historical-export-feasibility.md`: added
  selected-chart export evidence readback to `tv ohlcv` and `tv range`
  without creating a stable export command or fallback for Desktop-free
  `tv bars`.
- `tradingview-cli-watch-jsonl-compare-contract.md`: added bounded
  `tv watch compare <SYMBOL>...` with scanner-backed `watch_compare.v1`
  readiness, sample, heartbeat, and summary JSONL events.
- `tradingview-cli-v0.21.0-release-readiness.md`: prepared the `v0.21.0`
  release state, including version, changelog, release notes, README asset
  examples, package staging, and release validation.
- `tradingview-cli-v0.21-pre-release-audit.md`: audited completed `v0.21.0`
  range-scale readback, 5000-count date-range cap, narrow intraday date-range
  support, docs, skills, tests, source boundaries, and refactor posture before
  release readiness.
- `tradingview-cli-bars-intraday-range-feasibility.md`: expanded intraday
  date-range `tv bars --from/--to` support to `5`, `15`, `30`, and `60`
  while keeping the remaining intraday date ranges guarded.
- `tradingview-cli-bars-large-range-pagination.md`: expanded date-range
  `tv bars --count` as a returned-bar safety cap up to 5000 while keeping
  recent count mode capped at 500 and intraday date-range guarded.
- `tradingview-cli-bars-range-scale-and-intraday.md`: added
  `range_fetch_summary` to date-range `tv bars` success payloads and
  structured failure details so fetch windows, `request_more_data`, observed /
  filtered / returned counts, and truncation reasons are machine-readable.
- `tradingview-cli-v0.20.0-release-readiness.md`: prepared the `v0.20.0`
  release state, including version, changelog, release notes, README asset
  examples, package staging, and release validation.
- `tradingview-cli-user-getting-started-docs.md`: added user-facing English
  and Japanese getting-started docs, README links, and release package staging
  before `v0.20.0` release readiness.
- `tradingview-cli-v0.20-pre-release-audit.md`: audited completed `v0.20.0`
  weekly/monthly bars date-range readback, range alignment, docs, skills,
  tests, source boundaries, and refactor readiness before the final
  user-facing docs polish.
- `tradingview-cli-bars-weekly-monthly-range.md`: extended `tv bars`
  date-range historical readback to weekly and monthly bars while preserving
  the Desktop-free `bars.v1` source boundary.
- `tradingview-cli-v0.19.0-release-readiness.md`: prepared the `v0.19.0`
  release state, including version, changelog, release notes, README asset
  examples, and package validation.
- `tradingview-cli-v0.19-pre-release-audit.md`: audited completed `v0.19.0`
  daily bars date-range readback before release readiness.
- `tradingview-cli-bars-date-range-readback.md`: added the first `v0.19.0`
  implementation slice for reproducible Desktop-free historical bars by date
  range, without relying on selected-chart viewport movement.
- `tradingview-cli-v0.18.0-release-readiness.md`: prepared the `v0.18.0`
  release state, including version, changelog, release notes, README asset
  examples, and package validation.
- `tradingview-cli-v0.18-pre-release-audit-update.md`: refreshed the
  `v0.18.0` pre-release audit after adding final JSONL summary events.
- `tradingview-cli-jsonl-observation-summary-event.md`: added final summary
  events to bounded `tv observe chart` and `tv stream ...` JSONL observations.
- `tradingview-cli-v0.18-pre-release-audit.md`: audited completed
  `v0.18.0` JSONL observation contract metadata, then identified final
  summary events as a useful remaining contract polish before release
  readiness.
- `tradingview-cli-jsonl-observation-contract.md`: matured the existing
  Desktop-backed `tv observe chart` and `tv stream ...` JSONL event contracts
  for `v0.18.0` without adding realtime batching or source mixing.
- `tradingview-cli-v0.17.0-release-readiness.md`: prepared the `v0.17.0`
  release state, including version, changelog, release notes, README asset
  examples, and package validation.
- `tradingview-cli-v0.17-pre-release-audit-update.md`: refreshed the final
  `v0.17.0` pre-release audit after the bars crate-boundary cleanup, bars
  market internal split, and CLI contract test split.
- `tradingview-cli-cli-contract-test-split.md`: split the large CLI contract
  integration test into command-family targets with shared test helpers while
  preserving all existing assertions and behavior.
- `tradingview-cli-bars-market-internal-split.md`: split the market crate
  browserless bars implementation into facade, validation, protocol,
  transport, payload, and type modules while preserving the `bars.v1`
  command contract.
- `tradingview-cli-bars-crate-boundary-refactor.md`: moved browserless
  historical bars implementation from CLI `ops` into the Desktop-free market
  crate while preserving the `bars.v1` command contract.
- `tradingview-cli-v0.17-pre-release-audit.md`: audited completed `v0.17.0`
  bars summary/range and availability readback before release readiness, then
  identified the bars crate-boundary refactor as the last pre-release cleanup.
- `tradingview-cli-bars-availability-readback.md`: added
  `source_availability` and public-safe wait summaries to browserless
  historical bars success and failure payloads.
- `tradingview-cli-bars-summary-readback.md`: added additive `tv bars`
  summary / range / quality readback so browserless historical bars are easier
  for agents and downstream tools to consume safely.
- `tradingview-cli-v0.16.0-release-readiness.md`: prepared the `v0.16.0`
  release state, including version, changelog, release notes, README asset
  examples, and package validation.
- `tradingview-cli-v0.16-pre-release-audit.md`: audited completed v0.16
  quote-data regular-session readback and stable browserless bars contracts
  before release readiness.
- `tradingview-cli-bars-stabilization.md`: stabilized browserless
  `tv bars <EXCHANGE:SYMBOL>` as a Desktop-free historical bars read with a
  `bars.v1` contract.
- `tradingview-cli-quote-data-regular-session-semantics.md`: added
  regular-session `lp` readback to explicit quote-data reads so matching
  quote-data messages do not become unavailable only because `rtc` is absent.
- `tradingview-cli-v0.15.0-release-readiness.md`: prepared the `v0.15.0`
  release state, including version, changelog, release notes, README asset
  examples, and package validation.
- `tradingview-cli-v0.15-pre-release-audit.md`: audited completed v0.15
  compare movement readback and quote-data diagnostics before release
  readiness.
- `tradingview-cli-quote-data-diagnostics.md`: added a narrow quote-data
  diagnostic surface that explains Desktop target selection, WebSocket, `qsd`,
  requested-symbol, and `rtc` availability without mixing scanner, chart, and
  quote-data sources.
- `tradingview-cli-compare-change-evidence.md`: added additive
  `tv compare` movement readback so downstream tools can read stable
  regular-session percent-change evidence without parsing source-specific raw
  quote paths.
- `tradingview-cli-quote-data-live-smoke-target-hardening.md`: hardened the
  opt-in quote-data live smoke so premarket validation can target the intended
  TradingView chart in multi-target Desktop sessions.
- `tradingview-cli-v0.14.0-release-readiness.md`: prepared the `v0.14.0`
  release state, including version, changelog, release notes, README asset
  examples, and package validation.
- `tradingview-cli-v0.14-pre-release-audit.md`: audited quote-data source
  boundaries, availability diagnostics, docs, runtime skills, and deferred
  work before `v0.14.0` release readiness.
- `tradingview-cli-quote-data-availability-diagnostics.md`: added
  machine-readable quote-data unavailable reasons, wait-summary counters, and
  session readback without changing the explicit source boundary.
- `tradingview-cli-quote-data-session-contract.md`: added additive
  `quote-data` contract metadata and source-availability readback for
  `tv quote <SYMBOL> --source quote-data`.
- `tradingview-cli-v0.13.0-release-readiness.md`: prepared the `v0.13.0`
  release state, including version, changelog, release notes, README asset
  examples, and package validation.
- `tradingview-cli-quote-help-source-boundary.md`: clarified `tv quote` and
  `tv quotes` help so scanner-backed reads are not mistaken for guaranteed
  realtime data.
- `tradingview-cli-v0.13-pre-release-audit-update.md`: refreshed the v0.13
  pre-release audit after adding the explicit quote-data source and its
  opt-in live contract smoke.
- `tradingview-cli-quote-data-live-smoke.md`: added an opt-in ignored live
  contract smoke for `tv quote <SYMBOL> --source quote-data`, accepting
  structured unavailable results when no bounded `qsd.rtc` frame arrives.
- `tradingview-cli-quote-data-source.md`: added the explicit
  `tv quote <SYMBOL> --source quote-data` source for bounded Desktop-backed
  TradingView `qsd.rtc` quote-data readback without mixing it into
  chart-source quote or scanner extended-hours payloads.
- `tradingview-cli-desktop-quote-data-rtc-source-design.md`: fixed the source
  boundary and feasibility criteria for treating TradingView `qsd.rtc`
  quote-data WebSocket readback as a possible future `quote-data` source.
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
- lab-gated browserless historical bars prototype and later stable
  browserless bars contract
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
- opt-in `tv bars` JSON contract live smoke
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
- v0.14 release readiness
- v0.14 quote-data source availability diagnostics
- v0.14 quote-data live smoke target hardening

For command contract details, prefer the notes under `docs/notes/` before
reading archived implementation plans.
