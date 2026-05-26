# Expand date-range `tv bars` count cap

This ExecPlan is a living document. Keep `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes` updated while working.

## Purpose

`tv bars --from/--to` can now report range-scale diagnostics through
`range_fetch_summary`, but date-range mode still used the same 500-bar cap as
recent count mode. That kept long historical ranges readable as partial or
truncated, but it did not let a caller request larger daily / weekly /
monthly ranges even when the Desktop-free source could return them.

This slice expands date-range mode only: `--count` remains the returned-bar
safety cap, defaults to 500, and may be raised to 5000. Recent count mode
keeps the existing maximum of 500. Intraday date-range remains guarded.

## Progress

- [x] (2026-05-26T07:30Z) Create this ExecPlan and archive the completed
  `range_fetch_summary` plan.
- [x] (2026-05-26T07:35Z) Split the bars count validation limits so recent
  count mode remains capped at 500 while date-range mode allows up to 5000.
- [x] (2026-05-26T07:40Z) Update CLI help, contract tests, README, docs,
  packaged agent guidance, and runtime skills to explain the two count
  limits.
- [x] (2026-05-26T08:05Z) Run focused tests, baseline checks, runtime skill
  validation, and public-safe live smoke summaries.

## Surprises & Discoveries

- The transport already fetches date ranges in 500-bar windows through
  `request_more_data`, so the protocol-side fetch window can stay small while
  the final returned-bar cap grows.
- The existing `range_fetch_summary` fields were sufficient for this slice:
  no new JSON field was needed to explain larger range requests.

## Decision Log

- Decision: Keep count-only mode capped at 500.
  Rationale: It is the longstanding bounded recent sample behavior and is
  independent from date-range sample preparation.
- Decision: Raise only the date-range returned-bar safety cap to 5000.
  Rationale: date-range mode asks for a calendar period; larger returned
  evidence is useful there, while `range_fetch_summary` already reports the
  cap and truncation outcome.
- Decision: Keep the per-fetch window at 500.
  Rationale: it preserves the bounded WebSocket behavior and lets the
  transport page older data without changing the source contract.
- Decision: Do not unlock intraday date-range in this slice.
  Rationale: intraday range still needs retention, entitlement, and coverage
  feasibility work separate from the large-range cap.

## Outcomes

Date-range `tv bars` requests for `1D`, `1W`, and `1M` now accept `--count`
values up to 5000. Count-only mode still rejects values above 500, and
intraday date-range requests still fail validation before network access.

The `bars.v1` contract remains additive and unchanged in shape. Larger
date-range responses continue to use `range_fetch_summary.requested_count_cap`,
`filtered_count`, `returned_count`, `range_truncated`, and
`range_truncation_reason` to describe whether the returned evidence covered
the requested range or hit a cap, source exhaustion, or timeout.

Public-safe live smoke summaries passed:

- `NASDAQ:AAPL`, `1D`, 2010-01-01 through 2020-12-31, requested cap 3000:
  returned 2769 bars, coverage `complete`, fetch windows 9, request-more
  count 8, not truncated.
- `NASDAQ:CRUS`, `1W`, 2009-01-01 through 2015-12-31, requested cap 1000:
  returned 365 bars, coverage `complete`, fetch windows 2, request-more
  count 1, not truncated.

## Context

Previous `v0.21.0` work added `range_fetch_summary` so downstream tools could
read fetch windows, `request_more_data` count, observed / filtered / returned
counts, and range truncation reasons. This slice uses that foundation to
allow larger returned date ranges without adding a new command, source, or
option.

The source boundary remains unchanged:

- `tv bars` is a Desktop-free bounded historical OHLCV read.
- It uses `source: "tradingview_bars_ws"` and
  `source_category: "desktop_free_read"`.
- It does not call `tv range`, `tv ohlcv`, Replay, observe / stream, scanner
  quote, chart quote, or quote-data as hidden fallbacks.
- It does not expose raw WebSocket frames, raw payloads, session ids,
  credentials, target ids, account-local metadata, or raw live bars in
  tracked docs.

## Work Items

1. Add separate validation constants for recent count mode and date-range
   mode.
2. Keep `DATE_RANGE_FETCH_CHUNK` at 500 while allowing date-range requests to
   return up to 5000 bars.
3. Update unit and CLI contract tests:
   - count-only `--count 501` remains invalid;
   - date-range `--count 5001` is invalid with maximum 5000;
   - date-range `--count 501` and `--count 5000` pass market validation;
   - intraday date-range remains invalid.
4. Update help, README, docs, runtime skills, and packaged guidance with the
   two count limits.
5. Run focused tests, baseline checks, runtime skill validation, and optional
   public-safe live smoke.

## Validation

Completed validation:

- `cargo test -p tradingview-market bars -- --nocapture`
- `cargo test -p tradingview-cli market::bars -- --nocapture`
- `cargo test -p tradingview-cli --test cli_contract_bars -- --nocapture`
- `cargo test -p tradingview-cli --test live_bars`
- `uvx --with pyyaml python <skill-validator>/quick_validate.py` for
  `market-data-interpretation`, `chart-analysis`, and `multi-symbol-scan`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo metadata --no-deps --format-version 1`
- `git diff --check`
- `bash -n scripts/stage-release-package-files.sh`

Focused:

```bash
cargo test -p tradingview-market bars -- --nocapture
cargo test -p tradingview-cli market::bars -- --nocapture
cargo test -p tradingview-cli --test cli_contract_bars -- --nocapture
cargo test -p tradingview-cli --test live_bars
```

Baseline:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo metadata --no-deps --format-version 1
git diff --check
bash -n scripts/stage-release-package-files.sh
```

Runtime skills:

```bash
uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/market-data-interpretation
uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/chart-analysis
uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/multi-symbol-scan
```

Optional live smoke should record only public-safe summaries:

```bash
target/debug/tv bars NASDAQ:AAPL --timeframe 1D --from 2010-01-01 --to 2020-12-31 --count 3000
target/debug/tv bars NASDAQ:CRUS --timeframe 1W --from 2009-01-01 --to 2015-12-31 --count 1000
```

Record only symbol, timeframe, requested range, returned count, coverage
status, and range fetch summary. Do not paste raw bars into tracked docs.
