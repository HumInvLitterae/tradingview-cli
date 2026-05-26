# Plan intraday date-range feasibility for `tv bars`

This ExecPlan is a living document. Keep `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes` updated while working.

## Purpose

`tv bars --from/--to` is now mature for daily, weekly, and monthly historical
ranges, including `range_alignment`, `range_fetch_summary`, and a 5000-bar
date-range safety cap. The remaining `v0.21.0` question is whether intraday
timeframes can safely join the same date-range surface.

This slice does not unlock intraday date-range behavior. It records the
feasibility and contract work needed before `--timeframe 1|5|15|60...` can be
accepted with `--from/--to`.

## Progress

- [x] (2026-05-26T09:05Z) Create this ExecPlan and archive the completed
  large-range pagination plan.
- [x] (2026-05-26T09:10Z) Confirm current behavior with public-safe smoke:
  count-only intraday `15` and `60` reads return 500 bars, while intraday
  date-range remains a validation error before network access.
- [x] (2026-05-26T09:15Z) Update the `v0.21.0` roadmap, plan index, and
  changelog to make intraday date-range feasibility the current slice.

## Surprises & Discoveries

- Count-only intraday continues to work through the existing `bars.v1`
  Desktop-free source boundary. Public-safe smoke with `NASDAQ:AAPL`
  returned 500 bars for both `15` and `60` timeframes.
- The current date-range guard is still clean and early: `--timeframe 15
  --from ... --to ...` fails validation with supported timeframes `1D`,
  `1W`, and `1M`.

## Decision Log

- Decision: Keep intraday date-range guarded in this slice.
  Rationale: count-only intraday availability does not prove stable historical
  range retention, entitlement, or coverage semantics.
- Decision: Treat intraday range as the same `bars.v1` source family, not a
  new source.
  Rationale: future implementation should reuse `tradingview_bars_ws`,
  `range_alignment`, `range_fetch_summary`, `source_availability`, and
  `wait_summary`.
- Decision: Do not use hidden fallbacks to prove intraday feasibility.
  Rationale: `tv range`, `tv ohlcv`, Replay, observe / stream, scanner, chart
  quote, and quote-data have different source meanings and would make results
  harder to reproduce.
- Decision: The next implementation slice, if feasible, should start with a
  narrow stable set rather than all intraday timeframes.
  Rationale: retention and entitlement risk is likely higher for older or
  smaller intraday timeframes.

## Outcomes

This plan establishes the acceptance criteria for a future intraday
date-range implementation. A later slice may unlock a narrow set of intraday
timeframes only if it can show that:

- unsupported, unavailable, partial, timeout, and count-cap cases remain
  structured source diagnostics;
- `bars.v1` remains additive and keeps `range_alignment`,
  `range_fetch_summary`, `range_coverage_status`, `source_availability`, and
  `wait_summary`;
- count-only intraday and daily / weekly / monthly date-range behavior remain
  unchanged;
- no hidden source fallback is introduced.

If live evidence remains inconsistent, intraday date-range should stay
guarded and the next work should improve structured unavailable wording and
docs rather than exposing a broad stable CLI surface.

## Context

Current behavior:

- `tv bars <SYMBOL> --timeframe 1|3|5|15|30|45|60|120|180|240 --count N`
  is a count-only Desktop-free read with maximum count 500.
- `tv bars <SYMBOL> --timeframe 1D|1W|1M --from YYYY-MM-DD --to YYYY-MM-DD`
  is a date-range Desktop-free read with default cap 500 and maximum cap
  5000.
- Intraday date-range currently fails validation before network access.

Future intraday work should evaluate TradingView retention, entitlement,
symbol differences, and source exhaustion without writing raw bars, raw
WebSocket frames, raw payloads, session ids, credentials, target ids, or
account-local metadata into tracked files.

## Work Items

1. Keep current CLI behavior unchanged: no stable intraday date-range unlock
   in this slice.
2. Decide the first implementation candidate for the follow-up slice:
   - candidate timeframes: `1`, `5`, `15`, and `60`;
   - recommended default: unlock only the smallest set with stable evidence;
   - if evidence is mixed, keep all intraday date-range requests guarded.
3. Define the future payload contract:
   - `range_alignment` remains timeframe-specific timestamp readback;
   - `range_coverage_status` remains the primary date-range coverage field;
   - `range_fetch_summary` explains fetch windows, count caps, and
     truncation reasons;
   - `source_availability` and `wait_summary` explain unavailable or partial
     source behavior.
4. Document that `tv bars` remains the only source path for this work.
5. Run docs validation and, if any runtime skills change, skill validation.

## Validation

Completed validation for this planning slice:

- `git diff --check`
- `bash -n scripts/stage-release-package-files.sh`
- Public-safe smoke:
  - count-only `NASDAQ:AAPL` `60` returned 500 bars with complete coverage;
  - count-only `NASDAQ:AAPL` `15` returned 500 bars with complete coverage;
  - intraday date-range `15` remained a validation error with supported
    timeframes `1D`, `1W`, and `1M`.

Recommended validation commands:

```bash
git diff --check
bash -n scripts/stage-release-package-files.sh
rg -n "v0\\.21|intraday|date-range|bars\\.v1|range_fetch_summary|range_alignment|range_coverage_status|source_availability|retention|entitlement|tv range|tv ohlcv|Replay|watch|JSONL|source mixing" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md
```

If later code changes are added, run:

```bash
cargo test -p tradingview-market bars -- --nocapture
cargo test -p tradingview-cli --test live_bars
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo metadata --no-deps --format-version 1
```

Runtime skills only need validation if they are edited in this slice.
