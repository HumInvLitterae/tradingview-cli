# v0.14.0 pre-release audit

This ExecPlan is a living document. Keep `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` current as work
proceeds.

This document follows `.agents/PLANS.md` from the repository root.

## Purpose / Big Picture

`v0.14.0` added contract and diagnostics polish for explicit quote-data reads:
`tv quote <SYMBOL> --source quote-data`. Before release readiness starts, this
audit confirms that the source boundary, unavailable semantics, docs, runtime
skills, and tests all agree. A passing audit means release prep can proceed
without adding another feature slice.

This slice does not add new CLI behavior. It records whether `quote-data`
remains a separate Desktop-backed readback source, whether unavailable
quote-data is documented as source availability rather than missing market
price, and whether deferred work is still clearly deferred.

## Progress

- [x] (2026-05-09T16:25Z) Created this pre-release audit plan and archived the
  completed quote-data availability diagnostics plan.
- [x] (2026-05-09T16:35Z) Reviewed quote-data source boundary, unavailable
  semantics, docs, runtime skills, and current roadmap state.
- [x] (2026-05-09T17:10Z) Ran audit greps, focused contract tests, and Rust
  baseline.
- [x] (2026-05-09T17:15Z) Recorded outcomes and next-step decision.

## Surprises & Discoveries

- Observation: `quote-data` remains explicit and outside `--source auto`.
  Evidence: CLI help and contract tests still describe `quote-data` as an
  explicit source, while `quote --source auto` keeps the chart/scanner
  boundary and does not mention quote-data fallback.

- Observation: unavailable quote-data is consistently described as source
  availability, not a missing market price.
  Evidence: docs and runtime skills point agents to
  `source_availability.unavailable_reason`, `timed_out`, `next_action`, and
  wait-summary counters for diagnostics instead of ranking, recommendation, or
  price absence.

- Observation: audit greps did not identify a new release blocker.
  Evidence: hygiene hits are existing safety policy text, fake example paths,
  validation-command text, archived plan examples, and this audit plan's own
  commands. TODO/panic hits are the known ignored live smoke assertions, the
  Pine template TODO string, archived validation examples, and this plan's
  validation command.

## Decision Log

- Decision: Treat premarket-specific quote-data evidence as deferred, not as a
  release blocker.
  Rationale: `quote-data` now exposes bounded source availability diagnostics
  and does not claim scanner-style premarket semantics. Premarket evidence
  still requires the matching market phase and can be collected later without
  changing the v0.14 release boundary.
  Date/Author: 2026-05-09 / Codex.

- Decision: Keep this audit docs-only unless validation finds a release
  blocker.
  Rationale: the plan is a completion check before release readiness. New
  behavior, new options, and new payload fields would reopen the feature slice
  instead of preparing release.
  Date/Author: 2026-05-09 / Codex.

## Outcomes & Retrospective

The v0.14 pre-release audit found no release blocker.

`tv quote <SYMBOL> --source quote-data` remains an explicit Desktop-backed
WebSocket quote-data readback source. It is not part of `--source auto`, does
not mix scanner `extended_hours`, and does not mix chart main-series quote
fields or chart-source `session_boundary`. The current `quote_data.v1`
payload and structured unavailable details provide source diagnostics through
`source_availability`, `unavailable_reason`, expanded wait-summary counters,
and `quote_data.session_readback`.

The docs, roadmap, runtime skills, and focused tests agree that unavailable
quote-data means the bounded source did not provide usable `qsd.rtc`; it does
not mean the symbol lacks a market price and it is not a ranking,
recommendation, or trading-action signal.

Validation passed:

- `git diff --check`
- `bash -n scripts/stage-release-package-files.sh`
- public-doc and code hygiene grep, with only expected existing policy text,
  fake examples, archived validation text, ignored live smoke assertions, and
  this plan's validation commands
- quote-data/deferred-surface grep
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo metadata --no-deps --format-version 1`
- `cargo test -p tradingview-cli market::quote_data -- --nocapture`
- `cargo test -p tradingview-cli --test cli_contract quote -- --nocapture`
- `cargo test -p tradingview-cli --test live_quote_data_source`
- `cargo test -p tradingview-cli market::quote -- --nocapture`
- `cargo test -p tradingview-market quote -- --nocapture`

Premarket-specific quote-data live evidence remains deferred until a real
premarket window. That gap does not block `v0.14.0` because the current
contract does not claim premarket semantics. The next step is `v0.14.0 release
readiness`.

## Context and Orientation

The quote-data implementation lives in `crates/cli/src/ops/market/quote_data.rs`.
It observes bounded TradingView Desktop WebSocket quote-data messages and
returns `source: "desktop_quote_data_ws"` when a matching `qsd.rtc` appears.
If no usable quote-data readback appears within the bounded wait, it returns a
structured unavailable failure with `contract_version: "quote_data.v1"` and
`source_availability` diagnostics.

The live smoke is `crates/cli/tests/live_quote_data_source.rs`. It is ignored
by default and is not run in this audit. Normal test runs only compile and
validate the non-live helper path.

The durable docs that explain the user-facing boundary are
`docs/command-source-taxonomy.md`, `docs/observation-workflows.md`,
`docs/internal-tradingview-apis.md`, `README.md`, `CHANGELOG.md`, and runtime
skills under `.agents/skills/`.

## Plan of Work

Audit the current repo state for these facts:

- `quote-data` remains explicit and Desktop-backed, and it is not included in
  `--source auto`.
- `quote-data` does not mix scanner `extended_hours`, chart main-series OHLCV,
  or chart-source `session_boundary` into its payload.
- `source_availability.unavailable_reason`, `timed_out`, `next_action`,
  expanded wait-summary counters, and `quote_data.session_readback` are
  described as source diagnostics and session readback, not market judgment.
- no raw WebSocket frames, raw live payloads, target ids, account-local
  metadata, credentials, or local paths were added to public docs or packaged
  assets.
- deferred items remain deferred after v0.14: premarket evidence,
  quote-data auto fallback, automatic source mixing, chart-backed compare,
  watch/JSONL compare, realtime multi-symbol feed, stable browserless bars,
  standalone `tv events`, `tv diagnose`, binary split, MCP server, and daemon
  work.

Update `docs/plans/README.md` and `docs/v0.14-roadmap.md` so this audit is the
current plan. Archive the completed quote-data diagnostics plan. Do not update
`CHANGELOG.md` unless the audit itself needs a release-note-visible correction.

## Concrete Steps

Run these commands from the repository root:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|account-local|target id|downstream-private)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true
    rg -n "TODO|FIXME|panic!|unimplemented!|todo!" crates docs README.md AGENTS.md CLAUDE.md packaging/agent/AGENTS.md
    rg -n "quote-data|quote_data\\.v1|source_availability|unavailable_reason|session_readback|extended_hours|quote --source chart|auto fallback|ranking|recommendation|realtime|diagnose|binary split|MCP|daemon" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    cargo test -p tradingview-cli market::quote_data -- --nocapture
    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture
    cargo test -p tradingview-cli --test live_quote_data_source
    cargo test -p tradingview-cli market::quote -- --nocapture
    cargo test -p tradingview-market quote -- --nocapture

Do not run the ignored live quote-data smoke in this slice.

## Validation and Acceptance

Acceptance is met when the audit records no release blocker, focused
quote-data and quote tests pass, workspace baseline passes, and roadmap lanes
are marked complete or deferred for `v0.14.0`. The next plan should be
`v0.14.0 release readiness`.

If a validation command finds a real code or contract blocker, stop and create
a focused fix slice instead of mixing behavior changes into this audit.

## Idempotence and Recovery

This audit is docs-only unless validation reveals a release blocker. It is safe
to rerun the grep and test commands. If an ignored live smoke is accidentally
run and produces live data, do not paste raw output into tracked docs; record
only a public-safe summary if it matters.

## Open Questions

Premarket-specific quote-data evidence remains uncollected. It should be
collected only during a real premarket window and does not block `v0.14.0`
because the current contract does not claim premarket semantics.

## Revision Note

Created after quote-data availability diagnostics landed, so release readiness
can start from a durable audit of v0.14 source boundaries and deferred work.
