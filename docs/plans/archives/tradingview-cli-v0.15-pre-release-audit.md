# v0.15 pre-release audit

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It records
the completion and refactor audit before `v0.15.0` release readiness. It does
not add new command behavior, JSON payload semantics, dependencies, or version
changes.

## Purpose / Big Picture

`v0.15.0` has two main completed slices:

- additive `compare.v1` movement readback for regular-session percent-change
  evidence;
- narrow `tv diagnose quote-data <SYMBOL>` source diagnostics for the explicit
  quote-data source.

Before release readiness, verify that these contracts, docs, runtime skills,
and tests agree. If no release blocker is found, the next step is
`v0.15.0 release readiness`.

## Progress

- [x] (2026-05-11T15:05Z) Created this audit ExecPlan and archived the
  completed quote-data diagnostics implementation plan.
- [x] (2026-05-11T15:10Z) Updated current plan pointers for the v0.15
  pre-release audit.
- [x] (2026-05-11T12:52Z) Ran focused contract confirmation, docs hygiene
  checks, and the full Rust baseline. No release blocker was found.
- [x] (2026-05-11T12:52Z) Recorded final audit outcome and prepared the
  related docs for one local commit.

## Surprises & Discoveries

- Observation: The v0.15 implementation work stayed additive.
  Evidence: `compare.v1` kept raw sections and added `movement`, while
  `quote-data` diagnostics added a new troubleshooting command without
  changing existing quote source meanings.

- Observation: The public-doc hygiene checks returned only existing policy
  text, validation-command examples, ignored live-smoke assertions, and
  sanitized fixture/example strings.
  Evidence: The scan did not identify a new raw WebSocket frame, raw live
  payload, live target id, account-local value, credential, or local absolute
  path introduced by this audit slice.

- Observation: Remaining TODO / panic audit hits are not release blockers.
  Evidence: Hits are the known assertion-style `panic!` calls in ignored live
  smoke tests, one Pine template TODO string, archived validation examples, and
  this plan's validation command.

## Decision Log

- Decision: Treat premarket success evidence as deferred after `v0.15.0`.
  Rationale: `quote_data.v1` source availability and `tv diagnose quote-data`
  make unavailable states machine-readable; phase-specific live success
  evidence is useful but not required to release this diagnostic slice.
  Date/Author: 2026-05-11 / Codex.

- Decision: Do not add more quote-data fallback or source mixing in this
  audit.
  Rationale: The release boundary is source clarity. Automatic mixing and
  quote-data auto fallback remain deferred and require separate plans.
  Date/Author: 2026-05-11 / Codex.

## Outcomes & Retrospective

Audit complete. No release blocker was found.

`compare.v1` movement readback remains additive and aligned with tests and
docs: `items[].movement.regular_change_percent` is derived from raw
`items[].sections.quote.data.change`, and `summary.movement_coverage` counts
validated requested items without changing `coverage_status`,
`field_coverage`, `missing_evidence`, `follow_up_hints`, or raw `sections`.

`tv diagnose quote-data <SYMBOL>` remains a diagnostic command for the
explicit Desktop-backed quote-data source. It does not add quote-data to
`--source auto`, chart-source quote, scanner quote, or scanner
`extended_hours`, and its target / WebSocket / scanner-reference summaries stay
public-safe.

The next step is `v0.15.0 release readiness`.

## Audit Checklist

Confirm the following:

- `compare.v1` exposes `items[].movement.regular_change_percent` as additive
  regular-session percent-change readback derived from raw
  `items[].sections.quote.data.change`.
- `summary.movement_coverage` counts requested items without changing
  `summary.coverage_status`, `summary.field_coverage`, `missing_evidence`,
  `follow_up_hints`, or raw sections.
- `tv diagnose quote-data <SYMBOL>` is a diagnostic command, not a new price
  source.
- `diagnostic_status`, `desktop_target`, `quote_data`, `scanner_reference`,
  and `next_action_hints` stay public-safe and do not include raw target ids,
  raw WebSocket frames, raw live payloads, account-local metadata, or
  credentials.
- `quote-data` remains explicit and is not added to `--source auto`, chart
  quote, scanner quote, or scanner `extended_hours`.

## Validation Plan

Run audit and docs validation:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|account-local|target id|downstream-private)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true
    rg -n "TODO|FIXME|panic!|unimplemented!|todo!" crates docs README.md AGENTS.md CLAUDE.md packaging/agent/AGENTS.md
    rg -n "diagnose quote-data|quote_data_diagnostics|movement|regular_change_percent|source_availability|scanner_reference|extended_hours|auto fallback|ranking|recommendation|realtime|binary split|MCP|daemon" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md

Run Rust baseline:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1

Run focused contract confirmation:

    cargo test -p tradingview-market compare -- --nocapture
    cargo test -p tradingview-cli --test cli_contract compare -- --nocapture
    cargo test -p tradingview-cli --test cli_contract diagnose -- --nocapture
    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture
    cargo test -p tradingview-cli market::quote_data -- --nocapture
    cargo test -p tradingview-cli diagnostics -- --nocapture
    cargo test -p tradingview-cli --test live_quote_data_source

Optional read-only live smokes may be run, but raw output must not be pasted
into tracked docs:

    target/debug/tv compare SPY QQQ IWM NASDAQ:RKLB
    target/debug/tv diagnose quote-data NASDAQ:RKLB
    target/debug/tv --target-id <ID> diagnose quote-data NASDAQ:RKLB

## Idempotence and Recovery

This audit should be safe to rerun. If validation reveals only docs wording
drift, update docs and record the result here. If validation reveals an
implementation bug or contract mismatch, keep the fix small and scoped to the
release blocker. Defer broad refactors and new surfaces after `v0.15.0`.

## Interfaces and Dependencies

No new interface, option, dependency, or version change is planned in this
audit. The next expected ExecPlan, if no blocker is found, is
`tradingview-cli-v0.15.0-release-readiness.md`.

## Open Questions

None.
