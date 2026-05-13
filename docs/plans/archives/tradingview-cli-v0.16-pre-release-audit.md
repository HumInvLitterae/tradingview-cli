# v0.16 pre-release audit

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It records
the completion and refactor audit before `v0.16.0` release readiness. It does
not add new command behavior, JSON payload semantics, dependencies, or version
changes.

## Purpose / Big Picture

`v0.16.0` has two main completed slices:

- additive quote-data regular-session readback, where matching non-null
  `qsd.v.lp` can succeed as `price_readback.kind: "regular_last"` when
  `qsd.v.rtc` is absent;
- stable browserless `tv bars <EXCHANGE:SYMBOL>` historical bars readback with
  a `bars.v1` contract and no `TV_EXPERIMENTAL_BARS` gate.

Before release readiness, verify that these contracts, docs, runtime skills,
and tests agree. If no release blocker is found, the next step is
`v0.16.0 release readiness`.

## Progress

- [x] Created this audit ExecPlan and archived the completed browserless bars
  stabilization implementation plan.
- [x] Updated current plan pointers for the v0.16 pre-release audit.
- [x] Run focused contract confirmation, docs hygiene checks, and the full Rust
  baseline.
- [x] Record final audit outcome and prepare the related docs for one local
  commit.

## Surprises & Discoveries

- Observation: The v0.16 implementation work stayed additive and
  source-labeled.
  Evidence: `quote_data.v1` added `price_readback` and source-availability
  counters without adding quote-data to `--source auto`, while `tv bars` now
  reports `bars.v1` and `tradingview_bars_ws` without changing `tv ohlcv`.

- Observation: The public-doc hygiene scan reported expected policy text,
  archived historical plans, sanitized fixtures, and validation-command
  examples rather than a new private-data leak.
  Evidence: No new raw WebSocket frame, raw live payload, live target id,
  account-local value, credential, or local absolute path was identified as a
  release blocker.

- Observation: Remaining TODO / panic audit hits are not release blockers.
  Evidence: Hits are the known assertion-style `panic!` calls in ignored live
  smoke tests, one Pine template TODO string, archived validation examples,
  and this plan's validation command.

## Decision Log

- Decision: Treat premarket-specific quote-data success evidence as deferred
  after `v0.16.0`.
  Rationale: `quote_data.v1` now distinguishes `rtc` and regular `lp`
  readbacks, and structured unavailable states remain source diagnostics rather
  than price absence.
  Date/Author: 2026-05-13 / Codex.

- Decision: Do not add more source mixing, quote-data auto fallback, or bars
  streaming behavior in this audit.
  Rationale: The release boundary is contract clarity for existing explicit
  surfaces. Additional automation or realtime feeds require separate plans.
  Date/Author: 2026-05-13 / Codex.

## Outcomes & Retrospective

Audit complete. No release blocker was found.

`quote_data.v1` regular-session readback remains additive and aligned with
tests and docs. `quote_data.price_readback.kind` distinguishes `rtc` from
`regular_last`, `source_availability.price_readback_observed` reports whether
the bounded source read produced usable price readback, and wait-summary
counters stay public-safe.

`tv diagnose quote-data <SYMBOL>` still wraps quote-data readback as
diagnostics, not as a new blended price source. It does not mix quote-data into
scanner, chart-source quote, or `--source auto`, and it does not expose raw
target ids, raw WebSocket frames, or raw payloads.

`tv bars <EXCHANGE:SYMBOL>` is ready for `v0.16.0` release readiness as a
stable Desktop-free historical bars read. It no longer requires
`TV_EXPERIMENTAL_BARS=1`, and success / structured failure paths carry
`contract_version: "bars.v1"`, `source: "tradingview_bars_ws"`,
`source_category: "desktop_free_read"`, `requires_desktop: false`, and
`non_mutating: true`.

The next step is `v0.16.0 release readiness`.

## Audit Checklist

Confirm the following:

- `quote_data.v1` exposes `quote_data.price_readback` as additive source
  readback, with `kind: "rtc"` for `qsd.v.rtc` and `kind: "regular_last"` for
  `qsd.v.lp`.
- `source_availability.price_readback_observed` and wait-summary counters are
  public-safe and align with docs, tests, and runtime skills.
- `tv diagnose quote-data <SYMBOL>` preserves quote-data readback fields and
  does not expose raw target ids, raw WebSocket frames, raw payloads,
  account-local metadata, or credentials.
- `quote-data` remains explicit and is not added to `--source auto`, chart
  quote, scanner quote, or scanner `extended_hours`.
- `tv bars <EXCHANGE:SYMBOL>` no longer requires `TV_EXPERIMENTAL_BARS=1`.
- `tv bars` reports `contract_version: "bars.v1"`,
  `source: "tradingview_bars_ws"`, `source_category: "desktop_free_read"`,
  `requires_desktop: false`, and `non_mutating: true`.
- `tv bars` remains a bounded historical bars read, not realtime streaming,
  watch/JSONL compare, chart-backed compare, scanner quote, chart-source quote,
  or quote-data.

## Validation Plan

Run audit and docs validation:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|account-local|target id|downstream-private)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true
    rg -n "TODO|FIXME|panic!|unimplemented!|todo!" crates docs README.md AGENTS.md CLAUDE.md packaging/agent/AGENTS.md
    rg -n "quote_data\\.v1|price_readback|regular_last|bars\\.v1|tradingview_bars_ws|source_availability|diagnose quote-data|extended_hours|auto fallback|realtime|watch|JSONL|chart-backed compare|binary split|MCP|daemon" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md

Run Rust baseline:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1

Run focused contract confirmation:

    cargo test -p tradingview-cli market::quote_data -- --nocapture
    cargo test -p tradingview-cli diagnostics -- --nocapture
    cargo test -p tradingview-cli --test cli_contract diagnose -- --nocapture
    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture
    cargo test -p tradingview-cli --test live_quote_data_source
    cargo test -p tradingview-cli market::bars -- --nocapture
    cargo test -p tradingview-cli --test cli_contract bars -- --nocapture
    cargo test -p tradingview-cli --test live_bars

Optional read-only live smokes may be run, but raw output must not be pasted
into tracked docs:

    target/debug/tv quote NASDAQ:RKLB --source quote-data
    target/debug/tv diagnose quote-data NASDAQ:RKLB
    target/debug/tv bars NASDAQ:AAPL --timeframe 1D --count 5
    target/debug/tv bars NASDAQ:RKLB --timeframe 1 --count 10

## Idempotence and Recovery

This audit should be safe to rerun. If validation reveals only docs wording
drift, update docs and record the result here. If validation reveals an
implementation bug or contract mismatch, keep the fix small and scoped to the
release blocker. Defer broad refactors and new surfaces after `v0.16.0`.

## Interfaces and Dependencies

No new interface, option, dependency, or version change is planned in this
audit. The next expected ExecPlan, if no blocker is found, is
`tradingview-cli-v0.16.0-release-readiness.md`.

## Open Questions

None.
