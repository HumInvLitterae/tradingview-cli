# v0.21.0 pre-release completion and refactor audit

This ExecPlan records the release-readiness gate before `v0.21.0`.

## Purpose

`v0.21.0` expands `tv bars --from/--to` from daily / weekly / monthly range
reads into a more scalable historical range surface. The completed work adds
`range_fetch_summary`, raises the date-range returned-bar safety cap to 5000,
and unlocks narrow intraday date-range support for `5`, `15`, `30`, and `60`.

This audit stops feature work before release readiness and checks that the
contract, docs, runtime skills, help, tests, source boundaries, and refactor
posture are aligned.

## Progress

- [x] Create this pre-release audit plan.
- [x] Archive the completed intraday date-range feasibility and implementation
  plan.
- [x] Confirm docs / skills / help describe date-range `tv bars` support for
  `5`, `15`, `30`, `60`, `1D`, `1W`, and `1M`.
- [x] Confirm `1`, `3`, `45`, `120`, `180`, and `240` date-range requests
  remain guarded by validation before network access.
- [x] Confirm count-only mode remains capped at 500 and date-range mode remains
  capped at 5000.
- [x] Run docs validation, hygiene checks, focused bars tests, runtime skill
  validation, and the full Rust baseline.
- [x] Run optional public-safe smoke for `5`, `30`, and `60` intraday
  date-range reads.
- [x] Record the audit result and set the next step to `v0.21.0 release
  readiness`.

## Findings

No release blocker was found.

`tv bars --from/--to --timeframe 5|15|30|60|1D|1W|1M` is the supported
Desktop-free historical range entry point for `v0.21.0`. The remaining
intraday date-range timeframes stay guarded, while count-only intraday reads
keep their existing behavior.

`range_coverage_status` remains the primary date-range coverage readback.
`range_fetch_summary` explains bounded fetch windows, `request_more_data`
attempts, observed / filtered / returned counts, count caps, and truncation
reasons. `range_alignment` continues to explain period-start timestamps and
the `timestamp_within_requested_range` filter policy. `source_availability`
and `wait_summary` remain source diagnostics, not price evaluation or trading
judgment.

No hidden fallback or source mixing was found. `tv bars` remains the
Desktop-free `tradingview_bars_ws` source. `tv range`, `tv ohlcv`, Replay,
observe / stream, scanner, chart quote, and quote-data are not used as
fallbacks for this range work.

Hygiene grep hits were existing policy text, examples, tests, ignored live
smoke guards, Pine template text, or archived validation examples. No new
tracked docs, package assets, or error details were found to contain raw bars,
raw WebSocket frames, raw payloads, session ids, credentials, target ids,
account-local metadata, or local machine paths.

## Roadmap Decision

- Lane 1: range scale foundation is complete for `v0.21.0`.
- Lane 2: narrow intraday date-range support is complete for `v0.21.0`.
- Lane 3: unified range coverage semantics is complete for `v0.21.0`.
- Lane 4: source-guided sample workflow clarity is complete for `v0.21.0`.
- `1`, `3`, `45`, `120`, `180`, and `240` date-range support, automatic
  historical export, Replay extraction, watch / JSONL compare, chart-backed
  compare, source mixing, ranking, and recommendation remain deferred after
  `v0.21.0`.

## Refactor Decision

No release-blocking refactor is needed before `v0.21.0`.

The reviewed surface is limited to bars validation, range filtering, fetch
summary construction, payload shaping, the CLI adapter, help text, docs, and
runtime skills. The current split remains acceptable for release. Larger
refactors, new options, new sources, new payload semantics, dependency
updates, and version bumps should wait until after release readiness.

## Validation

Docs and hygiene:

- `git diff --check`
- `bash -n scripts/stage-release-package-files.sh`
- private / raw-data hygiene grep over public docs, package guides, skills,
  scripts, and crates
- TODO / panic grep over crates, docs, public guides, and package guides
- contract vocabulary grep for `bars.v1`, intraday date-range support,
  range-scale readbacks, source boundaries, and deferred work

Rust baseline:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo metadata --no-deps --format-version 1`

Focused contract confirmation:

- `cargo test -p tradingview-market bars -- --nocapture`
- `cargo test -p tradingview-cli market::bars -- --nocapture`
- `cargo test -p tradingview-cli --test cli_contract_bars -- --nocapture`
- `cargo test -p tradingview-cli --test live_bars`

Runtime skill validation:

- `market-data-interpretation`
- `chart-analysis`
- `multi-symbol-scan`

Public-safe smoke:

- `NASDAQ:AAPL` `5` from 2026-05-20 to 2026-05-22 returned 234 bars,
  `range_coverage_status: "complete"`, `range_truncated: false`, and
  `range_truncation_reason: "none"`.
- `NASDAQ:AAPL` `30` from 2026-05-01 to 2026-05-22 returned 208 bars,
  `range_coverage_status: "complete"`, `range_truncated: false`, and
  `range_truncation_reason: "none"`.
- `NASDAQ:AAPL` `60` from 2026-05-01 to 2026-05-22 returned 112 bars,
  `range_coverage_status: "complete"`, `range_truncated: false`, and
  `range_truncation_reason: "none"`.

Only symbol, timeframe, requested range, returned count, coverage status, and
range-fetch summary-level fields were recorded. Raw bars and raw live payloads
were not copied into tracked docs.

## Next Step

Proceed to `v0.21.0 release readiness`.
