# `v0.17.0` pre-release audit

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It records
the completion / refactor audit before `v0.17.0` release readiness.

## Purpose / Big Picture

`v0.17.0` focused on making stable browserless `tv bars <EXCHANGE:SYMBOL>`
easier for agents and downstream tools to consume safely. The release now has
additive `summary` / `range` readback plus `source_availability` and
public-safe `wait_summary` diagnostics for bounded historical source behavior.

This audit checks that the `bars.v1` contract, docs, runtime skills, help, and
tests agree before release readiness. It does not add new features, options,
data sources, payload semantics, or dependencies.

## Progress

- [x] Create this ExecPlan and archive the completed bars availability plan.
- [x] Update `docs/plans/README.md` and `docs/v0.17-roadmap.md` so the current
  plan is this pre-release audit.
- [x] Confirm `tv bars` success and failure contracts against docs, tests, and
  runtime skills.
- [x] Confirm deferred work remains deferred and not mixed into `bars.v1`.
- [x] Run focused contract tests, full Rust baseline, docs checks, and hygiene
  scans.
- [x] Record release-readiness recommendation.

## Surprises & Discoveries

- No release blocker was found.
- `tv bars --help` describes the command as bounded Desktop-free historical
  OHLCV with `bars.v1` and `tradingview_bars_ws`, and does not mention the old
  lab gate.
- The broad hygiene scans still report existing policy language, archived
  plan validation commands, and source-boundary docs. No new raw WebSocket
  frame, raw live payload, raw target id, credential, account-local metadata,
  or local validation path was added by this audit.

## Decision Log

- Decision: Treat `tv bars` summary / range evidence maturity as complete for
  `v0.17.0`.
  Rationale: Success payloads expose the count and time-span readback needed
  for downstream first-pass parsing without changing raw `bars[]`.
  Date/Author: 2026-05-14 / Codex.

- Decision: Treat bars availability / failure readback as complete for
  `v0.17.0`.
  Rationale: Success and structured failure payloads now expose
  `source_availability` and public-safe `wait_summary` diagnostics while
  keeping no-bars as failure.
  Date/Author: 2026-05-14 / Codex.

- Decision: Defer larger workflow surfaces until after `v0.17.0`.
  Rationale: Realtime multi-symbol feeds, watch / JSONL compare,
  chart-backed compare, source mixing, and event commands are separate
  product slices and should not be mixed into release readiness.
  Date/Author: 2026-05-14 / Codex.

## Outcomes & Retrospective

The audit confirms `v0.17.0` is ready to move to release readiness.

`tv bars` remains a Desktop-free bounded historical OHLCV read with
`contract_version: "bars.v1"`, `source: "tradingview_bars_ws"`,
`source_category: "desktop_free_read"`, `requires_desktop: false`, and
`non_mutating: true`. The payload keeps raw `bars[]` and adds
machine-readable `summary`, `range`, `data_quality.partial_result`,
`source_availability`, and public-safe `wait_summary`.

No-bars, timeout, WebSocket close/read failure, and protocol error paths
remain structured failures with source diagnostics. They are not documented as
price absence, ranking, recommendation, or trading-action evidence.

## Plan of Work

Audit the completed v0.17 bars work:

1. Confirm source contract fields in implementation, CLI contract tests, docs,
   runtime skills, and packaged runtime guide.
2. Confirm `tv bars --help` avoids old experimental/lab-gated wording and
   realtime guarantees.
3. Confirm failure details preserve source metadata and public-safe
   availability diagnostics.
4. Confirm source boundaries remain separate from `tv ohlcv`, scanner quote,
   chart quote, quote-data, and stream commands.
5. Run focused tests, full Rust baseline, and public-doc hygiene checks.

## Acceptance Criteria

- `tv bars` success payloads are documented and tested with `summary`,
  `range`, `data_quality.partial_result`, `source_availability`, and
  `wait_summary`.
- Structured failure paths are documented as bounded historical source
  diagnostics.
- No raw WebSocket frames, raw payloads, session ids, credentials,
  account-local metadata, target ids, or local paths are added to public docs
  or packaged assets.
- No release-blocking clippy warning, dead code, test mismatch, docs mismatch,
  or private-information leak is found.
- Roadmap lanes are marked complete or deferred.

## Validation

Run:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|account-local|target id|downstream-private)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true
    rg -n "TODO|FIXME|panic!|unimplemented!|todo!" crates docs README.md AGENTS.md CLAUDE.md packaging/agent/AGENTS.md
    rg -n "bars\\.v1|tradingview_bars_ws|source_availability|wait_summary|timeout_no_bars|websocket_closed|websocket_read_failed|protocol_error|summary|range|historical bars|realtime|watch|JSONL|chart-backed compare|auto fallback|binary split|MCP|daemon" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    cargo test -p tradingview-cli market::bars -- --nocapture
    cargo test -p tradingview-cli --test cli_contract bars -- --nocapture
    cargo test -p tradingview-cli --test live_bars

Optional read-only smoke:

    target/debug/tv bars NASDAQ:AAPL --timeframe 1D --count 5
    target/debug/tv bars NASDAQ:RKLB --timeframe 1 --count 10

Live output must not be pasted into tracked docs.

## Interfaces and Dependencies

This audit does not change public interfaces. No new command, option,
dependency, source, version bump, realtime feed, automatic fallback, source
mixing, ranking, scoring, recommendation, or trading action is planned.
