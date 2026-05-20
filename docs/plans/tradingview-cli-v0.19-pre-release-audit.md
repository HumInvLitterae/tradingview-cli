# `v0.19.0` pre-release audit

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It records
the completion / refactor audit before `v0.19.0` release readiness.

## Purpose / Big Picture

`v0.19.0` adds daily date-range historical readback to
`tv bars <EXCHANGE:SYMBOL>`. A user can now prepare reproducible old-period
OHLCV input with a command such as
`tv bars NASDAQ:CRUS --timeframe 1D --from 2010-01-01 --to 2010-12-31`
without relying on selected-chart viewport movement.

This audit confirms the `bars.v1` date-range contract, source boundaries,
docs, runtime skills, help, tests, and implementation placement are aligned
before release readiness. It does not add new features, new options, new data
sources, large refactors, ranking, scoring, recommendations, or trading
behavior.

## Progress

- [x] (2026-05-21T04:10Z) Create this pre-release audit ExecPlan.
- [x] (2026-05-21T04:10Z) Archive the completed bars date-range readback plan.
- [x] (2026-05-21T04:10Z) Update `docs/plans/README.md` and
  `docs/v0.19-roadmap.md` so the current plan is this audit.
- [x] (2026-05-21T04:35Z) Confirm docs, runtime skills, help, Rust API
  boundaries, and tests align with the daily date-range `bars.v1` contract.
- [x] (2026-05-21T04:35Z) Run focused tests, full Rust baseline, docs checks,
  skill validation, hygiene scans, and optional public-safe live smoke.
- [x] (2026-05-21T04:35Z) Record the release-readiness recommendation.

## Surprises & Discoveries

- Observation: The audit found no release blocker.
  Evidence: Focused bars tests, full workspace tests, clippy, formatting,
  metadata, packaging script syntax, diff checks, and runtime skill validation
  passed.

- Observation: Hygiene and source-boundary scans produced expected existing
  hits from public docs, skills, tests, and archived plans, but no new private
  value, raw frame, raw payload, or machine-local identifier was added by this
  audit.
  Evidence: The tracked changes are limited to this audit plan, roadmap plan
  status, and archiving the completed date-range plan.

- Observation: Optional public-safe live smoke confirmed daily date-range
  readback for both old and recent sample ranges.
  Evidence: `NASDAQ:CRUS` for 2010-01-01 through 2010-12-31 returned 252 daily
  bars with complete coverage; `NASDAQ:AAPL` for 2020-01-01 through
  2020-03-31 returned 62 daily bars with complete coverage. Raw bars were not
  recorded.

## Decision Log

- Decision: Treat the daily date-range `tv bars` implementation as complete
  and move into release-readiness audit.
  Rationale: The implementation plan is complete and has already recorded
  focused tests plus public-safe live smoke for AAPL 2020-Q1 and CRUS 2010.
  Date/Author: 2026-05-21 / Codex.

- Decision: Do not add another feature or refactor in this audit unless a
  release blocker is found.
  Rationale: The audit exists to confirm contract readiness. Intraday,
  weekly, monthly, larger-range batching, Replay extraction, and automatic
  source mixing are separate post-v0.19 candidates.
  Date/Author: 2026-05-21 / Codex.

- Decision: Recommend `v0.19.0 release readiness` as the next step.
  Rationale: The date-range `bars.v1` contract, docs, runtime skills, tests,
  and source boundaries are aligned, and validation found no blocker.
  Date/Author: 2026-05-21 / Codex.

## Outcomes & Retrospective

The audit is complete. `tv bars --from YYYY-MM-DD --to YYYY-MM-DD --timeframe
1D` is ready to enter `v0.19.0` release readiness as the Desktop-free
historical bars path for reproducible old daily samples. No new feature,
refactor, source, option, dependency, or payload semantic change was added in
this audit.

## Context and Orientation

`tv bars` is the Desktop-free historical OHLCV command. Desktop-free means it
does not require TradingView Desktop, Chrome DevTools Protocol, a selected
chart target, or visible chart state. It lives in the market crate and is
exposed through a thin CLI adapter.

The stable command contract is `bars.v1` with
`source: "tradingview_bars_ws"` and `source_category: "desktop_free_read"`.
Count mode still reads recent bounded bars. The new date-range mode is daily
only and is selected by passing both `--from YYYY-MM-DD` and
`--to YYYY-MM-DD` with `--timeframe 1D`. In date-range mode, `--count` is a
safety cap on returned bars and defaults to 500.

The `--to` value is an inclusive calendar date. Because TradingView daily bar
timestamps can occur during the market day rather than exactly at midnight
UTC, the implementation records `requested_range.to_time` as the requested
day's UTC start and `requested_range.to_time_exclusive` as the next UTC day
start used for filtering.

`tv range` is a different command that moves the selected Desktop chart
viewport. `tv ohlcv` is a selected-chart CDP bars read. Neither command is a
hidden fallback for `tv bars`, and neither is a stable historical export
contract for source-guided old samples.

## Plan of Work

Archive the completed date-range implementation plan and update durable
planning docs so this audit is the current plan. Inspect public docs, runtime
skills, CLI help, tests, and market crate APIs for consistency around
`request_mode`, `requested_range`, `returned_range`, `observed_range`,
`range_coverage_status`, `source_availability`, `wait_summary`, and raw
`bars[]`.

Confirm that `tradingview_market::bars_symbol(...)` remains the existing
count-mode API and that the new Rust API remains narrow as
`tradingview_market::bars_symbol_range(...)`. Confirm the implementation stays
within the Desktop-free market crate and does not use `tv range`, `tv ohlcv`,
Replay, observe / stream, scanner quote, chart quote, or quote-data as hidden
fallbacks.

Run the focused bars tests, full Rust baseline, docs checks, runtime skill
validation, and hygiene scans. If all checks pass, record in this ExecPlan and
the roadmap that the next step is `v0.19.0 release readiness`.

## Concrete Steps

From the repository root, run:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|raw JSONL|account-local|target id|downstream-private)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true
    rg -n "TODO|FIXME|panic!|unimplemented!|todo!" crates docs README.md AGENTS.md CLAUDE.md packaging/agent/AGENTS.md
    rg -n "bars\\.v1|date-range|request_mode|requested_range|returned_range|observed_range|range_coverage_status|to_time_exclusive|historical bars|tv range|tv ohlcv|realtime|watch|JSONL|chart-backed compare|auto fallback|binary split|MCP|daemon" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md

Then run:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1

Run focused checks:

    cargo test -p tradingview-market bars -- --nocapture
    cargo test -p tradingview-cli market::bars -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_bars -- --nocapture
    cargo test -p tradingview-cli --test live_bars

Validate changed runtime skills with `uvx` so local Python packages do not
need to be installed globally:

    uvx --with pyyaml python "$HOME/.codex/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/market-data-interpretation
    uvx --with pyyaml python "$HOME/.codex/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/chart-analysis
    uvx --with pyyaml python "$HOME/.codex/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/multi-symbol-scan

Optional public-safe live smoke:

    target/debug/tv bars NASDAQ:CRUS --timeframe 1D --from 2010-01-01 --to 2010-12-31
    target/debug/tv bars NASDAQ:AAPL --timeframe 1D --from 2020-01-01 --to 2020-03-31

Do not paste raw live output into tracked docs. Record only symbol, requested
range, bar count, returned range, and coverage status.

## Validation and Acceptance

Acceptance is met when the focused tests and full baseline pass, docs and
runtime skills consistently describe daily date-range `tv bars`, and no
release blocker is found.

The audit must confirm:

- `tv bars --from YYYY-MM-DD --to YYYY-MM-DD --timeframe 1D` is documented as
  the Desktop-free historical bars source for reproducible old daily samples.
- `--to` is documented and implemented as an inclusive calendar date, with
  `requested_range.to_time_exclusive` available as public-safe readback.
- `bars.v1` date-range payloads preserve existing fields and add
  `request_mode`, `requested_range`, `returned_range`, `observed_range`, and
  `range_coverage_status`.
- `source_availability` and `wait_summary` remain source diagnostics, not
  trading conclusions.
- `tv range`, `tv ohlcv`, Replay, observe / stream, scanner quote, chart
  quote, and quote-data are not hidden fallbacks for `tv bars`.
- no raw WebSocket frame, raw payload, session id, credential, account-local
  metadata, target id, or local absolute path is added to public docs,
  packaged assets, payloads, or error details.

## Idempotence and Recovery

This audit is safe to repeat. If a validation command fails, record the
failure in `Surprises & Discoveries`, fix only release blockers, and rerun the
failed command plus the relevant focused tests. If an unrelated working-tree
change appears, do not revert it; inspect whether it affects this audit before
deciding how to proceed.

## Artifacts and Notes

No raw live payloads or machine-local identifiers should be stored here.
Public-safe live evidence may include only:

- symbol;
- requested timeframe and date range;
- number of returned bars;
- first and last returned timestamps;
- coverage status;
- high-level unavailable reason if a command fails.

## Interfaces and Dependencies

This audit does not change public interfaces. The current interfaces being
audited are:

    tv bars <EXCHANGE:SYMBOL> --timeframe 1D --from YYYY-MM-DD --to YYYY-MM-DD

and the market crate APIs:

    tradingview_market::bars_symbol(symbol, timeframe, count)
    tradingview_market::bars_symbol_range(symbol, timeframe, from, to, count_cap)

No new command, option, dependency, source, version bump, realtime feed,
automatic fallback, source mixing, ranking, scoring, recommendation, or
trading action is planned.

## Open Questions

There are no unresolved critical questions. If validation finds a blocker, the
blocker should be fixed in this slice only when the fix is small and directly
related to release readiness.
