# `v0.17.0` pre-release audit update

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It records
the final completion / refactor audit before `v0.17.0` release readiness,
after the bars crate-boundary cleanup, bars market internal split, and CLI
contract test split.

## Purpose / Big Picture

`v0.17.0` makes browserless `tv bars <EXCHANGE:SYMBOL>` a more usable
historical evidence surface. It now exposes `summary`, `range`,
`data_quality.partial_result`, `source_availability`, and public-safe
`wait_summary` readback while keeping raw `bars[]` as detailed evidence.

After the first audit, two release-prep cleanups landed: the bars
implementation moved into the Desktop-free `tradingview-market` crate, and
the large CLI contract integration test was split by command family. This
updated audit checks that those refactors did not change the `bars.v1`
contract and that the project is ready to proceed to `v0.17.0` release
readiness.

## Progress

- [x] (2026-05-13T21:09Z) Create this updated pre-release audit ExecPlan.
- [x] (2026-05-13T21:09Z) Archive the completed CLI contract test split plan.
- [x] (2026-05-13T21:09Z) Update `docs/plans/README.md` and
  `docs/v0.17-roadmap.md` so the current
  plan is this updated audit.
- [x] (2026-05-13T21:19Z) Confirm `bars.v1` docs, tests, and skills remain
  aligned after the
  refactors.
- [x] (2026-05-13T21:19Z) Confirm the CLI contract split preserved
  command-family test coverage.
- [x] (2026-05-13T21:19Z) Run focused tests, full Rust baseline, docs checks,
  and hygiene scans.
- [x] (2026-05-13T21:19Z) Record the release-readiness recommendation.

## Surprises & Discoveries

- Observation: No release blocker was found in the refactor-aware audit.
  Evidence: focused bars and split CLI contract targets passed; `cargo
  clippy --workspace --all-targets --all-features -- -D warnings` and `cargo
  test --workspace` passed.

- Observation: The hygiene scans still report existing policy language,
  archived validation commands, fake example paths, and known assertion-style
  `panic!` calls in ignored live smokes.
  Evidence: the scans did not show a newly introduced raw WebSocket frame,
  raw live payload, raw target id, credential, account-local metadata, or
  local validation path from this audit.

## Decision Log

- Decision: Treat the previous `v0.17.0` pre-release audit as historical
  context and run this audit as the final refactor-aware check.
  Rationale: The earlier audit was correct for the bars payload work, but two
  behavior-preserving cleanup slices landed afterward and should be included
  in the release-readiness decision.
  Date/Author: 2026-05-13 / Codex.

- Decision: Do not add another feature or refactor in this slice unless a
  release blocker is found.
  Rationale: `v0.17.0` already contains the bars evidence maturity work and
  the intended cleanup. Additional behavior belongs in a later version.
  Date/Author: 2026-05-13 / Codex.

- Decision: Recommend `v0.17.0 release readiness` as the next step.
  Rationale: The bars payload contract, market crate boundary, private module
  split, and split CLI contract tests all validate without changing public
  behavior.
  Date/Author: 2026-05-13 / Codex.

## Outcomes & Retrospective

This updated audit confirms `v0.17.0` is ready to proceed to release
readiness.

`tv bars` remains a Desktop-free bounded historical OHLCV read with
`contract_version: "bars.v1"` and `source: "tradingview_bars_ws"`. The raw
`bars[]` evidence, `summary`, `range`, `data_quality.partial_result`,
`source_availability`, and public-safe `wait_summary` are preserved. No-bars,
timeout, WebSocket close/read failure, and protocol error remain structured
source diagnostics rather than success payloads.

The post-audit refactors are complete and behavior-preserving:
browserless bars implementation belongs to `tradingview-market`, CLI `ops`
remains a thin adapter, market crate internals are split by responsibility,
and CLI contract tests are split by command family.

## Context and Orientation

`tv bars` is a Desktop-free command that reads bounded historical OHLCV bars
for an exchange-qualified symbol such as `NASDAQ:AAPL`. Desktop-free means it
does not require a running TradingView Desktop chart target. The stable
payload contract is marked by `contract_version: "bars.v1"` and
`source: "tradingview_bars_ws"`.

The implementation now belongs in `crates/market/`, the crate used for
Desktop-free market reads. The CLI operation under `crates/cli/src/ops/` is
only an adapter from command-line arguments to the market crate function. The
market crate keeps `tradingview_market::bars_symbol(symbol, timeframe, count)`
as the public Rust entrypoint and hides validation, protocol, transport,
payload, and internal type modules behind it.

The CLI contract integration tests are split by command family. Root CLI
behavior remains in `crates/cli/tests/cli_contract.rs`; bars coverage is in
`cli_contract_bars.rs`; Desktop-free market evidence is in
`cli_contract_quote.rs`; quote-data diagnostics are in
`cli_contract_diagnose.rs`; CDP-backed command contracts are in
`cli_contract_desktop.rs`; shared helpers live in
`crates/cli/tests/support/mod.rs`.

## Plan of Work

Update the durable planning docs so this audit is the current plan, and archive
the completed CLI contract split plan. Then inspect docs, runtime skills, and
tests for consistency around `bars.v1`, `summary`, `range`,
`source_availability`, `wait_summary`, and the split CLI contract targets.

Run the focused bars and contract test targets first, then the full Rust
baseline. Run repository hygiene scans for private information and stale
release-risk wording. If all checks pass, record in this ExecPlan and the
roadmap that the next step is `v0.17.0 release readiness`.

## Concrete Steps

From the repository root, run:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|account-local|target id|downstream-private)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true
    rg -n "TODO|FIXME|panic!|unimplemented!|todo!" crates docs README.md AGENTS.md CLAUDE.md packaging/agent/AGENTS.md
    rg -n "bars\\.v1|tradingview_bars_ws|source_availability|wait_summary|summary|range|historical bars|cli_contract_bars|cli_contract_quote|cli_contract_diagnose|cli_contract_desktop|realtime|watch|JSONL|chart-backed compare|auto fallback|binary split|MCP|daemon" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md

Then run:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1

Run focused checks:

    cargo test -p tradingview-market bars -- --nocapture
    cargo test -p tradingview-cli market::bars -- --nocapture
    cargo test -p tradingview-cli --test cli_contract -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_bars -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_diagnose -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop -- --nocapture
    cargo test -p tradingview-cli --test live_bars

Optional read-only smoke can be run if desired:

    target/debug/tv bars NASDAQ:AAPL --timeframe 1D --count 5
    target/debug/tv bars NASDAQ:RKLB --timeframe 1 --count 10

Do not paste raw live output into tracked docs. If smoke is recorded, keep only
a public-safe summary.

Validation completed for this audit:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg hygiene scans for private data and deferred-surface wording
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    cargo test -p tradingview-market bars -- --nocapture
    cargo test -p tradingview-cli market::bars -- --nocapture
    cargo test -p tradingview-cli --test cli_contract -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_bars -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_diagnose -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop -- --nocapture
    cargo test -p tradingview-cli --test live_bars

## Validation and Acceptance

Acceptance is met when the focused tests and full baseline pass, the docs and
skills consistently describe `tv bars` as a Desktop-free bounded historical
bars read, and no release blocker is found.

The audit must confirm:

- `bars.v1` success payloads still expose `summary`, `range`,
  `data_quality.partial_result`, `source_availability`, `wait_summary`, and
  raw `bars[]`.
- no-bars, timeout, WebSocket close/read failure, and protocol error remain
  structured source diagnostics rather than success payloads.
- `tv bars` is not described as realtime feed, selected-chart `tv ohlcv`,
  scanner quote, chart quote, quote-data, ranking, recommendation, or trading
  action.
- the market crate refactor preserved `tradingview_market::bars_symbol(...)`
  as the public Rust API and did not expand product behavior.
- split CLI contract tests preserve the previous coverage.

## Idempotence and Recovery

This audit is docs-first and safe to repeat. If a validation command fails,
record the failure in `Surprises & Discoveries`, fix only release blockers,
and rerun the failed command plus the relevant focused tests. If an unrelated
working-tree change appears, do not revert it; inspect whether it affects this
audit before deciding how to proceed.

## Interfaces and Dependencies

This audit does not change public interfaces. No new command, option,
dependency, source, version bump, realtime feed, automatic fallback, source
mixing, ranking, scoring, recommendation, or trading action is planned.

## Open Questions

There are no unresolved critical questions. If validation finds a blocker, the
blocker should be fixed in this slice only when the fix is small and directly
related to release readiness.
