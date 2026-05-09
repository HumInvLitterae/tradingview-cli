# v0.13.0 pre-release audit update

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root.

## Purpose / Big Picture

The earlier v0.13 pre-release audit passed before the explicit `quote-data`
source was added. Since then, `tv quote <SYMBOL> --source quote-data` and its
opt-in live contract smoke were implemented. This update audit confirms that
the new source fits the v0.13 source/session-boundary story before release
readiness starts.

This slice does not add new CLI behavior. It records whether the current
implementation, docs, tests, and runtime skills agree on the boundary between
scanner quotes, chart-source quotes, quote-data WebSocket readback, snapshot
metadata, and follow-up vocabulary.

## Progress

- [x] (2026-05-10T02:10Z) Created this audit update plan and archived the
  completed quote-data live smoke plan.
- [x] (2026-05-10T02:15Z) Reviewed current quote-data implementation,
  dispatch, contract tests, docs, runtime skills, and roadmap references.
- [x] (2026-05-10T02:45Z) Ran focused audit validation and Rust baseline.
- [x] (2026-05-10T02:50Z) Recorded outcomes and updated current docs for the
  release-readiness handoff.

## Surprises & Discoveries

- Observation: `quote-data` remains explicitly outside `--source auto`.
  Evidence: dispatch routes `QuoteSource::QuoteData` separately before the
  `QuoteSource::Auto` branch, so auto still follows the chart/scanner behavior
  rather than silently observing WebSocket quote-data frames.

- Observation: `quote-data` is not documented as scanner extended-hours or
  chart main-series data.
  Evidence: README, source taxonomy, observation workflow docs, and runtime
  skills describe it as `desktop_quote_data_ws`, separate from chart-source
  quote and scanner REST `extended_hours`.

- Observation: the live smoke is opt-in only and permits structured
  unavailable by default.
  Evidence: `live_quote_data_source` is ignored, requires
  `TV_LIVE_QUOTE_DATA_SMOKE=1`, and validates unavailable details with
  `raw_frame_included: false`.

- Observation: audit greps found only expected existing policy text, fake
  example paths, validation command text, archived plan examples, and ignored
  smoke/test assertions.
  Evidence: the hygiene scans did not identify new raw WebSocket frames, raw
  live payloads, target ids, account-local metadata, credentials, or
  machine-local validation details introduced by this slice.

- Observation: source-boundary references are consistent after the quote-data
  additions.
  Evidence: README, command source taxonomy, observation workflow docs,
  internal API notes, runtime skills, and tests describe `quote-data` as an
  explicit `desktop_quote_data_ws` source; they do not describe it as chart
  main-series data, scanner REST `extended_hours`, or an `--source auto`
  fallback.

## Decision Log

- Decision: Treat this as an audit update, not a new feature slice.
  Rationale: quote-data implementation and live smoke are already committed.
  Release readiness needs a durable check that the new source did not reopen
  source-boundary or contract blockers.
  Date/Author: 2026-05-10 / Codex.

- Decision: Do not require opt-in live `qsd.rtc` success evidence before
  release readiness.
  Rationale: after-hours has ended, and the source contract already handles
  bounded no-frame results as structured unavailable. A live success can be
  collected later during postmarket or premarket without changing the v0.13
  release boundary.
  Date/Author: 2026-05-10 / Codex.

## Outcomes & Retrospective

The updated pre-release audit found no v0.13.0 release blocker.

The explicit `tv quote <SYMBOL> --source quote-data` source remains separate
from chart-source quote, scanner-backed `extended_hours`, and `--source auto`.
The normal test path confirms the ignored live smoke compiles and keeps
phase/source checks public-safe; opt-in live `qsd.rtc` success evidence was
not required or run in this slice.

Focused quote, quote-data, snapshot, and compare contract tests passed, as did
the workspace Rust baseline and docs/packaging validation. The next step is
`v0.13.0 release readiness`.

## Plan of Work

Audit the current code and docs for source-boundary consistency:

- chart-source quote reports `session_boundary` and does not expose scanner
  extended-hours;
- quote-data source reports `desktop_quote_data_ws`, is Desktop-backed, and
  does not merge chart main-series or scanner `extended_hours`;
- quote-data is not part of `--source auto`;
- no-frame/no-matching `qsd.rtc` returns public-safe structured unavailable
  details without raw frames;
- `live_quote_data_source` stays ignored and opt-in;
- snapshot metadata and compare/snapshot follow-up vocabulary remain aligned
  after quote-data docs additions.

Update `docs/plans/README.md` and `docs/v0.13-roadmap.md` to make this audit
the current plan and to mark quote-data live smoke as complete. Add a minimal
changelog docs entry only if useful for release notes.

## Concrete Steps

Run from the repository root:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|account-local|target id|downstream-private)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true
    rg -n "TODO|FIXME|panic!|unimplemented!|todo!" crates docs README.md AGENTS.md CLAUDE.md packaging/agent/AGENTS.md
    rg -n "quote-data|desktop_quote_data_ws|qsd\\.rtc|session_boundary|extended_hours|premarket|postmarket|quote --source chart|snapshot.v1|missing_evidence|follow_up_hints|ranking|recommendation|realtime|diagnose|binary split|MCP|daemon" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    cargo test -p tradingview-cli market::quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture
    cargo test -p tradingview-cli --test live_quote_data_source
    cargo test -p tradingview-cli --test live_after_hours_ws_correlation -- --nocapture
    cargo test -p tradingview-market snapshot -- --nocapture
    cargo test -p tradingview-market compare -- --nocapture

Do not run the ignored quote-data live smoke unless explicitly requested.

## Validation and Acceptance

Acceptance is met when the audit records no release blocker, the validation
commands pass or any non-blocking grep output is classified, and roadmap lanes
are marked complete or deferred for v0.13.0. The next plan should be
`v0.13.0 release readiness`.

## Idempotence and Recovery

This slice changes docs only. If a validation command reveals a code or
contract blocker, stop and create a focused fix plan instead of mixing a
behavior change into the audit.

## Open Questions

Premarket-specific quote-data evidence remains uncollected. That does not
block v0.13.0 as long as the source is documented as bounded WebSocket
readback rather than scanner-style extended-hours semantics.
