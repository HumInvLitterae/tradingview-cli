# Changelog

All notable changes to this project are recorded here.

This project uses Git tags such as `v0.2.0` for public releases. The Cargo
package version omits the leading `v`.

## Unreleased

### Fixed

- Made JSON and JSONL output tolerate closed downstream pipes without a Rust
  panic, backtrace, duplicate error output, or continued JSONL polling after
  the consumer exits, while preserving normal output bytes and command exit
  codes.

### Documentation

- Added the `v0.26.0` robustness and I/O-correctness roadmap, an ordered
  work-item inventory with retained feature follow-ups, and the first
  broken-pipe hardening ExecPlan. Refreshed contributor sources, plan indexing,
  and current-state guidance after the `v0.25.0` release.

## v0.25.0 - 2026-06-11

### Added

- Added `tv chart compare <SYMBOL>...` as a narrow Desktop-backed
  `chart_compare.v1` workflow for checking a small finalist set with
  selected-chart quote evidence, ordered item status, and restore readback.
- Added `tv events compare <SYMBOL>...` as a Desktop-free
  `events_compare.v1` workflow for checking scanner-backed earnings and
  dividend event readback across a small candidate set.
- Added `tv replay log --attach-ohlcv-summary` with optional
  `--ohlcv-count <N>` so bounded Replay step logs can explicitly attach
  selected-chart OHLCV summary evidence with
  `replay_log_ohlcv_summary_attachment.v1` metadata.
- Added `tv screenshot --region strategy --output <PATH>` as
  non-mutating Strategy Tester panel visual evidence with
  `evidence_role: "strategy_tester_panel"`.

### Documentation

- Added the `v0.25.0` roadmap direction and first chart-backed compare
  command plan, keeping future Desktop-backed selected-chart compare separate
  from scanner-backed `tv compare` / `tv watch compare` while carrying
  `tv events` expansion and Replay evidence attachment as adjacent lanes.
- Recorded the `tv events compare` implementation slice for `v0.25.0`,
  keeping it separate from full calendar, ranking, and recommendation
  semantics.
- Recorded the Replay log OHLCV summary attachment slice for `v0.25.0`,
  keeping attachments opt-in and separate from screenshots, Replay export,
  Desktop-free `tv bars`, ranking, and recommendation semantics.
- Recorded the Strategy Tester screenshot evidence slice for `v0.25.0`,
  keeping panel visual evidence separate from structured strategy metrics,
  export, ranking, and recommendation semantics.
- Recorded the `v0.25.0` pre-release architecture audit, confirming
  chart-backed compare, events compare, Replay OHLCV attachment, Strategy
  Tester screenshot evidence, source boundaries, docs, runtime skills, tests,
  and module posture before release readiness.

## v0.24.0 - 2026-06-03

### Added

- Added `tv events <SYMBOL>` as a narrow Desktop-free `events.v1` readback
  for scanner-backed earnings and dividends fields, with `--event-type
  <all|earnings|dividends>` and source metadata that keeps it separate from a
  full event calendar, chart reads, ranking, or recommendations.
- Added Desktop-free bare symbol resolution for `tv bars`, so input such as
  `AAPL` can resolve through TradingView symbol search while the `bars.v1`
  payload reports both the requested and resolved exchange-qualified symbol.

### Fixed

- Hardened `tv launch` on macOS by using the system app launcher for the
  normal no-path launch path, avoiding direct binary spawn as the default while
  keeping explicit `--path` and `--kill-existing` behavior opt-in.

### Documentation

- Reduced runtime skill context load by splitting market data interpretation,
  chart analysis, and multi-symbol scan into short workflow-oriented
  `SKILL.md` files plus detailed source-boundary reference notes.
- Recorded the `v0.24.0` pre-release architecture audit, checking launch
  handling, bars symbol resolution, events readback, docs, skills, source
  boundaries, and module posture before release readiness.
- Added the `v0.24.0` roadmap direction and first launch process-handling
  plan, focusing on operational robustness, `tv bars` symbol usability, and
  high-priority workflow candidates after the `v0.23.0` release.
- Added the `tv events` symbol-scoped readback plan for `v0.24.0`, shaping
  existing scanner fundamentals earnings and dividends fields into a narrow
  event evidence surface.
- Added the `tv bars` symbol resolution plan for `v0.24.0`, keeping
  Desktop-free symbol search separate from selected-chart and quote sources.

## v0.23.0 - 2026-05-28

### Added

- Added `tv export chart-bars --from <UNIX_SECONDS> --to <UNIX_SECONDS>`, an
  explicit Desktop-backed selected-chart bars export workflow with
  `export_chart_bars.v1` diagnostics for requested visible range, range
  operation, chart context, returned bars range, and range-match status.
- Added `tv replay log --steps <N>`, a bounded Desktop-backed Replay JSONL
  workflow that records readiness, per-step state, and final summary with
  `replay_step_log.v1` metadata.

### Documentation

- Recorded the `v0.23.0` pre-release architecture audit, confirming export,
  Replay, compare, and events workflow boundaries before release readiness.
- Added standalone `tv events` feasibility planning for `v0.23.0`, keeping
  existing earnings and dividends reads as scanner-backed fundamentals fields
  while defining any future event command as a separate source surface.
- Added chart-backed compare contract planning for `v0.23.0`, keeping
  Desktop-free `tv compare` separate from selected-chart evidence workflows
  before any stable Desktop-backed compare command is added.
- Added Replay step-log implementation guidance for `v0.23.0`, keeping Replay
  logs as bounded Desktop-backed workflow evidence rather than stable
  historical bars export.
- Added the `v0.23.0` roadmap direction and first selected-chart historical
  export plan, focusing on explicit Desktop-backed export workflow maturity
  without making it a hidden fallback for Desktop-free `tv bars`.

## v0.22.0 - 2026-05-27

### Added

- Added selected-chart export evidence readback to `tv ohlcv` and `tv range`
  so selected-chart context, returned bar ranges, and visible-range diagnostics
  can be inspected without adding a stable export command.
- Added Replay extraction feasibility readback to `tv replay ...` payloads so
  Replay status, state-changing Replay operations, selected-chart context, and
  operation metadata can be inspected without adding a stable export command.
- Added `tv watch compare <SYMBOL>...`, a bounded Desktop-free JSONL workflow
  that polls scanner-backed quote evidence for a known candidate set and emits
  `watch_compare.v1` readiness, sample, heartbeat, and summary events.
- Added advisory source metadata to `snapshot.v1` and `compare.v1`
  `follow_up_hints[]` so possible next evidence checks are easier to route
  without automatic execution, source mixing, ranking, or recommendations.

### Documentation

- Recorded the `v0.22.0` pre-release architecture audit, confirming the
  observation / export workflow contracts, source boundaries, docs, runtime
  skills, tests, and refactor posture before release readiness.
- Added a selected-chart historical export feasibility plan for `v0.22.0`,
  keeping `tv range` / `tv ohlcv` as Desktop-backed selected-chart operations
  rather than hidden fallbacks for Desktop-free `tv bars`.
- Added the `v0.22.0` roadmap direction and first watch / JSONL compare
  contract plan, focusing on observation and export workflow maturity without
  adding daemon behavior, automatic source mixing, or trading recommendations.

## v0.21.0 - 2026-05-26

### Added

- Expanded narrow intraday date-range support in `tv bars --from --to` to
  `5`, `15`, `30`, and `60` timeframes while keeping the remaining intraday
  date ranges guarded.
- Expanded `tv bars --from --to` date-range mode so `--count` remains a
  returned-bar safety cap but may be raised to 5000 for supported date-range
  reads. Recent count mode remains capped at 500.
- Added additive `range_fetch_summary` readback to `tv bars` date-range
  payloads and structured failure details so bounded fetch windows,
  `request_more_data` attempts, returned-count caps, and range truncation
  reasons are machine-readable before intraday date-range support is widened.

### Documentation

- Recorded the `v0.21.0` pre-release audit, confirming range-scale readback,
  5000-count date-range cap, narrow intraday date-range support, docs,
  runtime skills, tests, source boundaries, and refactor posture are ready for
  release readiness.
- Updated the intraday `tv bars --from --to` plan for `v0.21.0` to move from
  feasibility into a narrow `5` / `15` / `30` / `60` implementation while
  keeping the remaining intraday range support deferred.
- Added the `v0.21.0` roadmap direction and first range-scale / intraday
  readiness plan, grouping large-range batching, pagination, and intraday
  date-range work under the existing Desktop-free `bars.v1` historical bars
  source boundary.

## v0.20.0 - 2026-05-26

### Added

- Added weekly and monthly date-range readback to `tv bars --from --to` so
  `1D`, `1W`, and `1M` historical ranges can share the same Desktop-free
  `bars.v1` source boundary.
- Added additive `range_alignment` readback to `tv bars` date-range payloads
  and structured failure details so downstream tools can distinguish
  period-start bar timestamps from requested calendar ranges.

### Documentation

- Added the `v0.20.0` roadmap direction and first weekly/monthly `tv bars`
  date-range plan, focusing on historical bars range maturity without adding
  source mixing, realtime feeds, or trading recommendations.
- Refined the first `v0.20.0` bars plan with downstream feedback on
  weekly/monthly timestamp semantics, range alignment readback, and structured
  date-range failure metadata.
- Recorded the `v0.20.0` pre-release audit, confirming weekly/monthly
  date-range `bars.v1` readback, range-alignment semantics, docs, runtime
  skills, tests, source boundaries, and refactor posture are ready for release
  readiness.
- Added user-facing getting-started docs in English and Japanese, and staged
  them in release archives so non-developer users and agents can follow the
  download, first-check, Desktop readiness, and source-boundary workflow.

## v0.19.0 - 2026-05-21

### Added

- Added daily date-range readback to `tv bars` so older source-guided examples
  can be requested with `--from YYYY-MM-DD --to YYYY-MM-DD` while preserving
  the Desktop-free `bars.v1` source boundary.
- Added additive date-range coverage readback to `tv bars`, including
  `request_mode`, `requested_range`, `returned_range`, `observed_range`, and
  `range_coverage_status`.

### Documentation

- Added the `v0.19.0` roadmap direction and first `tv bars` date-range
  readback plan, focusing on reproducible Desktop-free historical bars input
  for older source-guided examples without treating `tv range` plus
  `tv ohlcv` as a historical export contract.
- Recorded the `v0.19.0` pre-release audit, confirming the daily date-range
  `bars.v1` contract, source boundaries, docs, runtime skills, and tests are
  ready for release readiness.

## v0.18.0 - 2026-05-17

### Added

- Added additive JSONL contract metadata to `tv observe chart` and
  `tv stream ...` events so downstream tools can distinguish readiness,
  sample, and heartbeat events without changing existing event semantics.
- Added final summary JSONL events to bounded `tv observe chart` and
  `tv stream ...` runs so downstream tools can read sample counts, heartbeat
  counts, elapsed time, bounded controls, and end reason without aggregating
  the whole stream themselves.

### Documentation

- Added the `v0.18.0` roadmap direction and first JSONL observation contract
  plan, focusing on existing `tv observe chart` and `tv stream ...` event
  metadata without adding realtime batching, watch loops, or source mixing.
- Recorded the `v0.18.0` pre-release audit for JSONL observation contract
  metadata and source-boundary readiness.
- Added a follow-up JSONL observation summary-event plan before release
  readiness, keeping the scope on existing selected-chart observation
  contracts rather than new realtime feeds or watch loops.
- Refreshed the `v0.18.0` pre-release audit after adding final JSONL summary
  events, so release readiness is based on the completed event contract.

## v0.17.0 - 2026-05-14

### Added

- Added additive `tv bars` summary and range readback so downstream tools can
  inspect requested-vs-returned count, time coverage, ordering, and partial
  historical bars coverage before parsing raw `bars[]`.
- Added `tv bars` source availability and public-safe wait-summary readback
  so no-bars, timeout, WebSocket, and partial completion cases are easier to
  distinguish as bounded historical source diagnostics.

### Documentation

- Added the `v0.17.0` roadmap direction and first bars summary readback plan,
  focusing on historical bars evidence maturity without adding realtime feeds,
  source mixing, or trading recommendations.
- Recorded the updated `v0.17.0` pre-release audit after the bars
  crate-boundary cleanup, bars market internal split, and CLI contract test
  split.

### Internal

- Moved browserless `tv bars` implementation into the Desktop-free market
  crate so CLI `ops` remains a thin command adapter while preserving the
  `bars.v1` contract.
- Split the market crate `tv bars` implementation into facade, validation,
  protocol, transport, payload, and shared type modules without changing the
  `bars.v1` command contract.
- Split the CLI contract integration tests into command-family targets with
  shared helpers, without changing command behavior or JSON contracts.

## v0.16.0 - 2026-05-13

### Added

- Added additive regular-session quote-data readback so matching `qsd.v.lp`
  can return `quote_data.price_readback.kind: "regular_last"` when `qsd.v.rtc`
  is absent, without mixing scanner or chart sources.

### Changed

- Stabilized browserless `tv bars <EXCHANGE:SYMBOL>` as a Desktop-free
  historical bars read with a `bars.v1` contract and no
  `TV_EXPERIMENTAL_BARS` gate.

### Documentation

- Added the `v0.16.0` roadmap direction and quote-data regular-session
  semantics plan, keeping quote-data, scanner, chart, and automatic fallback
  boundaries separate.
- Recorded the `v0.16.0` pre-release audit, confirming quote-data
  regular-session readback and stable browserless bars are ready for release
  readiness.

## v0.15.0 - 2026-05-12

### Added

- Added `tv diagnose quote-data <SYMBOL>` as a narrow source diagnostics
  packet for explicit quote-data reads, keeping scanner, chart, and
  quote-data evidence separate.
- Added additive `tv compare` movement readback so downstream tools can use a
  stable regular-session percent-change path without parsing raw quote
  sections.

### Changed

- Hardened the opt-in `quote-data` live smoke so it can target a specific
  TradingView chart during premarket validation without exposing raw target
  ids or raw WebSocket frames.
- Added the `v0.15.0` roadmap direction and first quote-data diagnostics plan,
  focusing on source availability troubleshooting without mixing scanner,
  chart, and quote-data sources.

## v0.14.0 - 2026-05-10

### Added

- Added `contract_version: "quote_data.v1"` and `source_availability`
  readback to explicit `tv quote <SYMBOL> --source quote-data` success
  payloads and structured unavailable details, without mixing in scanner
  `extended_hours` or chart main-series fields.
- Added quote-data availability diagnostics, including machine-readable
  unavailable reasons, additional public-safe wait-summary counts, and
  session readback normalization for explicit `--source quote-data` reads.

### Documentation

- Added the `v0.14.0` roadmap direction for quote-data maturity and source
  availability clarity, with the first implementation plan focused on
  additive quote-data contract metadata and structured unavailable readback.

## v0.13.0 - 2026-05-09

### Added

- Added additive `session_boundary` readback to chart-source quote payloads so
  agents can see that `tv quote <SYMBOL> --source chart` reads the selected
  chart main-series last bar and does not provide scanner-style extended-hours
  fields.
- Added an opt-in ignored live smoke for TradingView Desktop quote-session
  extended-hours field evidence, so postmarket and premarket behavior can be
  checked without adding new public quote payload fields.
- Clarified the quote-session live smoke timing guard so expected-phase
  mismatches are reported as `not_yet_in_expected_phase` instead of being
  mistaken for postmarket or premarket evidence.
- Hardened the quote-session live smoke so TradingView's `post-market` and
  `pre-market` phase spellings match the `postmarket` / `premarket` expected
  phases, and recorded public-safe postmarket evidence without adding public
  pre/post quote payload fields.
- Added an opt-in ignored live smoke for Desktop visible after-hours panel
  source discovery, comparing scanner REST, chart main-series quote,
  quote-session selected fields, and compact right-panel visible text without
  adding public quote payload fields.
- Extended the Desktop visible after-hours panel source smoke with a compact
  lower-level right-panel detail widget summary, including matched status/price
  node descriptors and React metadata presence without raw DOM or raw props.
- Added an opt-in ignored CDP Network/WebSocket smoke for after-hours source
  discovery, reporting compact communication candidate summaries without raw
  frames, raw response bodies, or public quote payload changes.
- Added an opt-in ignored right-panel widget-store smoke for after-hours
  source discovery, reporting whether scoped React fiber/props/state evidence
  is exposed around the visible price node without raw DOM or raw state.
- Added an opt-in ignored WebSocket correlation smoke for after-hours source
  discovery, comparing compact visible right-panel price samples with CDP
  WebSocket numeric candidates without quote-session subscription.
- Hardened the after-hours WebSocket correlation smoke to summarize public-safe
  TradingView `qsd` quote-data fields such as `rtc`, `rtc_time`, `rch`, and
  `rchp` when investigating visible after-market panel values.
- Added a source-design plan for possible Desktop quote-data `qsd.rtc`
  support, keeping it separate from chart-source quote and scanner
  extended-hours payloads.
- Added `tv quote <SYMBOL> --source quote-data` for explicit bounded
  Desktop-backed WebSocket quote-data readback such as `qsd.rtc`, without
  merging it into chart-source quote or scanner extended-hours payloads.
- Added an opt-in ignored live contract smoke for `tv quote <SYMBOL> --source
  quote-data`, validating both success payloads and structured unavailable
  results without raw WebSocket frames.
- Added additive `tv snapshot <SYMBOL>` contract metadata, including a
  command-local contract marker, coverage summary, missing-evidence readback,
  and machine-readable follow-up hints.
- Clarified the stable follow-up vocabulary shared by `tv compare` and
  `tv snapshot`, keeping `chart_quote` as the canonical chart-feed quote hint.

### Documentation

- Added the `v0.13.0` roadmap direction for source and session boundary
  clarity after `v0.12.0`, with the first implementation plan focused on
  chart-source quote extended-hours feasibility and agent misread prevention.
- Reframed the current v0.13 plan around Desktop quote-session live evidence
  after regular-session probes showed pre/post field names exist but should not
  be treated as scanner-equivalent extended-hours values yet.
- Recorded that the first attempted postmarket Desktop quote-session smoke did
  not observe a postmarket phase, so Desktop quote-session pre/post fields
  remain research evidence rather than public payload values.
- Recorded the pre-`v0.13.0` completion and refactor audit before release
  readiness.
- Recorded screenshot-backed RKLB evidence that visible right-panel
  after-market prices can correlate exactly with received WebSocket numeric
  candidates, while keeping public payload support deferred.
- Recorded HAR and live RKLB evidence that `qsd.rtc` is the strongest current
  backing-source candidate for the visible after-market panel value, while
  keeping it as source-discovery evidence rather than public payload support.
- Refreshed the pre-`v0.13.0` audit after adding the explicit quote-data
  source and its opt-in live contract smoke.
- Clarified `tv quote` and `tv quotes` help so scanner-backed reads are not
  mistaken for guaranteed realtime data and point users to freshness metadata.

## v0.12.0 - 2026-05-08

### Added

- Added additive `summary.coverage_status` readback to `tv compare
  <SYMBOL>...` so downstream tools can distinguish complete, partial, and
  blocked evidence coverage without ranking or recommending symbols.
- Added additive `items[].missing_evidence[]` readback to `tv compare
  <SYMBOL>...` so downstream tools can route quote, info, and fundamentals
  evidence gaps to explicit follow-up surfaces.

### Documentation

- Added the `v0.12.0` roadmap direction for contract-stable evidence
  follow-up after `v0.11.0`, with the first implementation plan focused on
  `tv compare` follow-up hint vocabulary, field coverage semantics, coverage
  status readback, and failure-side contract guards.
- Recorded the pre-`v0.12.0` completion and refactor audit before release
  readiness.

## v0.11.0 - 2026-05-08

### Added

- Added compare contract metadata readback for downstream wrappers, including
  a command-local contract marker, requested-order indexes, per-item follow-up
  hints, and field coverage counts.

### Documentation

- Added the `v0.11.0` roadmap direction for downstream-safe `compare` contract
  metadata and the first implementation plan for additive compare readback
  fields.
- Recorded the pre-`v0.11.0` completion and refactor audit before release
  readiness.

## v0.10.0 - 2026-05-08

### Added

- Added an additive `summary` to `tv compare <SYMBOL>...` so downstream tools
  can read resolution, section-success, and missing-value counts without
  replacing raw per-symbol `items`.

### Documentation

- Added workflow decision-table guidance for choosing between `quotes`,
  `compare`, `snapshot`, chart observation, chart-source quote, and screenshots.
- Recorded the pre-`v0.10.0` completion and refactor audit before release
  readiness.
- Drafted the `v0.10.0` roadmap direction for `compare` summary polish,
  comparison workflow documentation, and evidence follow-up after the
  `v0.9.0` release.

## v0.9.0 - 2026-05-07

### Added

- Added `tv compare <SYMBOL>...`, a Desktop-free multi-symbol comparison
  packet with per-symbol quote, info, and default fundamentals evidence.

### Tests

- Added an opt-in ignored Rust live smoke for near-concurrent
  chart-source quote reads.
- Fixed that smoke harness to pipe child `tv` stdout/stderr before parsing,
  then recorded successful width 2 and width 3 live runs without symbol
  mismatch.
- Added an opt-in ignored Rust live smoke for the Desktop-free `tv compare`
  JSON contract.

### Documentation

- Clarified that chart-source quote is a correctness-first single-symbol read,
  not a multi-symbol realtime batch source, before `v0.9.0` comparison work.
- Recorded the source decision that `v0.9.0` comparison planning should start
  from Desktop-free reads rather than chart-switching realtime loops.
- Added the `v0.9.0` roadmap draft and first `tv compare <SYMBOL>...`
  implementation plan for Desktop-free multi-symbol comparison.

## v0.8.0 - 2026-05-06

### Added

- Added `tv snapshot <SYMBOL>`, a Desktop-free single-symbol evidence packet
  that combines scanner quote, symbol info, and scanner-backed fundamentals
  sections with source metadata and section-level errors.
- Added an opt-in ignored Rust live smoke for the Desktop-free `tv snapshot`
  JSON contract.

### Documentation

- Added the `v0.8.0` roadmap draft and first `tv snapshot <SYMBOL>`
  implementation plan for Desktop-free single-symbol evidence packets.
- Synchronized workflow documentation and runtime skills around `tv snapshot`
  as the one-symbol Desktop-free evidence packet before chart observation.

## v0.7.0 - 2026-05-06

### Added

- Added `tv observe chart`, a Desktop-backed JSONL observation workflow that
  emits readiness first, then selected-chart last-bar samples and heartbeats
  with bounded `--duration-ms`, `--max-events`, and `--heartbeat-ms` controls.
- Added an opt-in ignored Rust live smoke for the `tv observe chart` JSONL
  readiness, sample, and heartbeat contract.
- Added an opt-in ignored Rust live smoke for the lab-gated `tv bars` JSON
  contract and `data_quality` evidence.

### Documentation

- Added the `v0.7.0` roadmap draft for agent-ready observation workflows and
  the first `tv observe chart` implementation plan.
- Added the next `v0.7.0` plan for opt-in live smoke around the
  `tv observe chart` JSONL event contract.
- Added the next `v0.7.0` plan for opt-in live smoke around the lab-gated
  `tv bars` WebSocket read contract.
- Added public-safe scanner field evidence for fundamentals, earnings,
  dividends, and event-like fields before adding new event/calendar surface.
- Added an observation workflow guide and synchronized runtime skills for
  Desktop-free screening, Desktop-backed chart observation, visual evidence,
  experimental bars, and fundamentals/event-like fields.
- Enriched `tv fundamentals --group earnings|dividends` and matching scanner
  scan column validation with additional scanner-metainfo-backed earnings and
  dividend fields.
- Recorded the pre-`v0.7.0` completion and refactor audit before release
  readiness.

## v0.6.0 - 2026-05-05

### Added

- Added root `tv --version` / `tv -V` support for release archive and local
  binary sanity checks.
- Added bounded observation controls to Desktop-backed `tv stream ...`
  commands: `--duration-ms`, `--max-events`, and `--heartbeat-ms`.
- Added additive stream event metadata with `_event: "sample"` for changed
  samples and `_event: "heartbeat"` for heartbeat JSONL envelopes.
- Added source taxonomy metadata to stream sample and heartbeat events:
  `source`, `source_category`, `requires_desktop`, and `non_mutating`.
- Added `tv readiness` as a Desktop-backed, non-mutating read that summarizes
  CDP target selection, chart API readiness, OHLCV bar readiness, and next
  action hints.
- Added source taxonomy and visual evidence metadata to `tv screenshot`
  payloads, including `writes_file` and `visual_evidence`.
- Added source taxonomy metadata to core Desktop-backed read payloads including
  `status`, `tab list`, `state`, `ohlcv`, and chart-source quote.
- Added source taxonomy metadata to Desktop-free market and scanner read
  payloads including `search`, `info`, scanner-source `quote`, `quotes`,
  `fundamentals`, `scanner scan`, `scanner hotlist`, and `scanner metainfo`.

### Fixed

- Stream dedupe now ignores stream metadata such as `_ts` and `_event`, so
  unchanged chart/page samples are not emitted solely because the sample
  timestamp changed.

### Documentation

- Clarified README, contributor agent guides, and packaged runtime agent guide
  roles before the `v0.6.0` release-readiness pass.

## v0.5.1 - 2026-05-02

### Added

- Added a `v0.6.0` roadmap draft for observation-first workflows and
  Desktop-backed readiness/recovery follow-up.
- Added `docs/command-source-taxonomy.md` to classify commands as
  Desktop-free reads, Desktop-backed reads, Desktop-backed operations, hybrid
  commands, or experimental commands while keeping one `tv` binary.
- Added an opt-in ignored Rust integration smoke for chart-source quote
  sequence endurance checks against a live TradingView Desktop session.

### Changed

- Updated docs and runtime skills to use the command source taxonomy and to
  record `quote --source chart` mismatch hardening as a separate patch
  candidate.

### Fixed

- Hardened `tv quote <SYMBOL> --source chart` readiness so chart-backed symbol
  quotes require matching quote/chart symbols and consecutive stable
  requested-symbol bar samples before reporting success.

## v0.5.0 - 2026-05-02

### Added

- Added the `v0.5.0` roadmap draft with separate Desktop-free data and
  Desktop-backed agent operation lanes, plus the first Desktop readiness
  diagnostics ExecPlan.
- Added experimental `tv bars <SYMBOL> --timeframe <TIMEFRAME> --count <N>`
  behind `TV_EXPERIMENTAL_BARS=1` for bounded Desktop-free historical bars
  through TradingView's undocumented WebSocket chart-session path.
- Added Desktop-free `tv fundamentals <SYMBOL>` for scanner-backed fundamental
  fields, including earnings date/time fields when TradingView returns them.
- Added repeatable `tv fundamentals --group earnings|valuation|dividends|financials`
  field bundles for easier Desktop-free fundamentals reads.

### Changed

- Split scanner-backed fundamentals internals into field selection, scanner
  request, and response normalization modules without changing CLI output.
- Improved Desktop readiness diagnostics across `status`, `tab list`, `state`,
  chart-source quote, and OHLCV failure details so agents can inspect endpoint,
  target, and chart-bars readiness before falling back to visual checks.
- Recorded bounded evidence for the lab-gated `tv bars` prototype while keeping
  it CLI-owned and experimental until protocol stability is clearer.
- Scoped Computer Use guidance in docs and runtime skills so CLI-only agents
  use structured `tv` diagnostics and screenshots as the portable fallback.
- Recorded a future Codex app-only Computer Use visual recovery skill as
  deferred until concrete downstream recovery patterns justify it.
- Recorded a TradingView Desktop capability gap audit so future missing-feature
  work is prioritized by operator value rather than broad Desktop parity.

## v0.4.1 - 2026-05-01

### Fixed

- Hardened `tv quote <SYMBOL> --source chart` so chart-backed symbol quotes
  wait for chart bars to reflect the requested symbol, retry once on readiness
  timeout, and fail instead of returning stale previous-symbol data.

## v0.4.0 - 2026-04-30

### Added

- Added `extended_hours` premarket and postmarket fields to Desktop-free
  `tv quote <SYMBOL>` scanner reads.
- Added confirmed premarket and postmarket scanner REST columns to
  `tv scanner scan --columns ...` for Desktop-free extended-hours scans.
- Added Desktop-free `tv scanner metainfo` for scanner field metadata
  discovery.
- Added Desktop-free `tv quotes <SYMBOL>...` for ordered batch quote reads.
- Added explicit `tv quote <SYMBOL> --source scanner|chart|auto` selection and
  scanner quote `time` / `update_mode` / `delay_seconds` metadata.
- Added typed reusable read APIs to the internal `tradingview-market` and
  `tradingview-scanner` crates while preserving CLI JSON payloads.
- Documented the typed market/scanner Rust API boundary with crate-level
  rustdoc and `docs/rust-api.md`.
- Clarified the `v0.4.0` market-data read boundary: scanner REST reads are
  sufficient for current watchlist-style needs, while Desktop-free historical
  bars remain a research/lab candidate rather than a stable command.
- Added runtime skills for market-data source interpretation and scanner /
  Screener result analysis.

## v0.3.0 - 2026-04-29

### Added

- Added a `v0.3.0` roadmap that prioritizes upstream PR re-checks,
  non-public API stabilization, direct HTTP feasibility, and a later
  binary/library crate split.
- Started a refreshed upstream pull-request re-check note after the `v0.2.0`
  release.
- Added global `tv --target-id <CDP_TARGET_ID>` target selection, structured
  target handoff hints, and non-mutating scanner-backed symbol quote reads with
  chart-switch freshness checks.
- Removed the old `TV_CDP_TARGET_ID` explicit target-selection fallback from
  the public contract; use `target_cli_args` / `--target-id` instead.
- Improved TradingView Desktop CDP compatibility by defaulting to
  `127.0.0.1`, skipping initial CDP domain-enable calls, and exposing
  app-window targets in `tv tab list` for diagnostics.
- Recorded Pine `alertcondition()` alert creation feasibility and kept raw
  indicator-alert endpoint primitives out of the public CLI.
- Added `tv pine alertconditions [--file <PATH>]` for local static discovery of
  Pine `alertcondition()` candidates without connecting to TradingView or
  creating account alerts.
- Added
  `tv alert create-indicator --script <NAME> --file <PATH>
  --condition-title <TITLE>|--alert-cond-id <ID> [--dry-run]` for guarded Pine
  `alertcondition()` alert creation. Dry-run verifies a local source candidate
  and a unique saved-script display-name match without creating alerts. Normal
  mode creates through the logged-in alert endpoint only when required metadata
  is available and verifies the new alert by readback before reporting success.
- Fixed alert delete cleanup for numeric alert ids and sanitized returned alert
  condition details so Pine/account metadata is not exposed in alert list,
  create, or delete payloads.
- Improved `tv ohlcv` failures with structured chart-bars readiness details and
  refreshed agent guidance for `--target-id`, valid chart commands, and OHLCV
  recovery.
- Added Desktop-free `tv info <SYMBOL>` symbol metadata reads through
  TradingView symbol search and tightened `tv quote <SYMBOL>` so symbol
  resolution failures return candidates instead of falling back to chart target
  selection.
- Added the first internal library crate boundary so the `tv` binary can share
  modules through the CLI package library root without changing the CLI
  contract.
- Extracted shared typed errors, exit-code mapping, and JSON envelope types
  into the internal workspace crate `tradingview-core` under `crates/core/`.
- Extracted Desktop-free symbol search, symbol info, and symbol quote reads
  into the internal workspace crate `tradingview-market` under
  `crates/market/`.
- Split `tradingview-market` into focused modules and extracted Desktop-free
  scanner reads and Pine static/check helpers into internal workspace crates
  `tradingview-scanner` and `tradingview-pine`.
- Split the `tv` binary entrypoint from the library-owned application runner,
  command dispatch, stream loop, input handling, output envelopes, and safety
  gates.
- Extracted CDP target discovery, runtime evaluation, screenshot capture, and
  input event primitives into the internal workspace crate `tradingview-cdp`.
- Moved the `tradingview-cli` package into `crates/cli/`, leaving the
  repository root as a virtual Cargo workspace while preserving the `tv`
  binary and CLI behavior.
- Split the large Screener operation adapter behind a facade and sub-surface
  modules while preserving the CLI surface and behavior.
- Moved Screener validation request types, helpers, and tests into the
  Screener validation module while preserving command behavior.
- Moved Screener column operations, storage-column helpers, and column tests
  into the Screener columns module while preserving command behavior.
- Moved the remaining Screener state, screen, and filter operation bodies into
  their sub-surface modules, leaving `engine` as the shared runtime/helper
  layer.
- Split the large Alert operation adapter into facade plus list, create,
  indicator, delete, and payload modules while preserving command behavior.
- Split the historical Layout operation adapter into facade plus watchlist and
  pane modules while preserving command behavior.
- Split the CDP-dependent Pine Editor adapter into facade plus runtime, source,
  scripts, and compile modules while preserving command behavior.
- Split the Drawing, Replay, and chart-dependent Market operation adapters
  into facade plus focused submodules while preserving command behavior.
- Split the generic UI automation adapter into facade plus DOM, input,
  selector, and eval modules while preserving command behavior and the
  application-layer unsafe eval gate.
- Introduced an in-package `domain` layer with a first `watchlist` service
  boundary for symbol normalization, bulk aggregation, and payload
  normalization while preserving command behavior.
- Added a second in-package `domain::alert` boundary for alert condition
  validation, public-safe alert payload normalization, sanitization, and API
  fallback policy while preserving command behavior.
- Added a third in-package `domain::replay` boundary for Replay validation,
  timestamp conversion, and action/status payload normalization while
  preserving command behavior.
- Added a fourth in-package `domain::drawing` boundary for Drawing request
  types, direction parsing, override parsing, and position validation while
  preserving command behavior.
- Added a larger in-package `domain::screener` boundary for Screener
  validation, target resolution, and storage payload shaping while preserving
  command behavior.
- Clarified application/domain/operation-adapter dependencies so dispatch
  calls pure domain helpers directly while `ops` stays focused on executable
  TradingView operations.
- Extracted the proven I/O-free domain/model layer into the internal workspace
  crate `tradingview-model` under `crates/model/`, preserving CLI behavior
  while making validation, request models, target resolution, and payload
  shaping reusable across CLI adapters and future crates.
- Added an operation-adapter boundary reference that classifies remaining
  `ops` surfaces as executable adapters, API/storage replacement candidates, or
  intentional UI/DOM boundaries.
- Added a storage-backed path for `tv screener filters modify --min/--max` on
  simple saved-screen `Condition` filters selected by index, with storage
  re-fetch post-checks and UI fallback only before any storage save attempt.
- Added `tv screener open --full-page` to reuse existing full-page Stock
  Screener targets and return `target_cli_args`; when automatic CDP target
  creation is unavailable, the command uses the bounded TradingView Desktop
  new-tab Screener tile fallback and reports `creation_method`.
- Added `--direction <long|short>` as an alias for the positional `DIRECTION`
  argument on `tv draw position`.
- Shared TradingView Desktop app-window/new-tab helper code between `tab`
  operations and `screener open --full-page` while preserving command behavior.

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
