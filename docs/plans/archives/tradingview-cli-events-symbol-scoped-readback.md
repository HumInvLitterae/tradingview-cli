# `tv events` symbol-scoped readback

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md`. It is self-contained and describes the implementation, validation, and documentation work needed to add a narrow `tv events <SYMBOL>` readback for scanner-backed earnings and dividend fields without adding a full event calendar source.

## Purpose / Big Picture

Users and agents can already inspect event-like fields through `tv fundamentals --group earnings` and `tv fundamentals --group dividends`, but those payloads are scanner field tables rather than an event-shaped surface. This slice adds `tv events <SYMBOL>` as a Desktop-free, scanner-backed readback that turns the same public fields into `events.v1`.

The observable outcome is that `tv events NASDAQ:AAPL` returns earnings and dividend event entries with source metadata, requested / resolved symbol readback, event counts, and field availability. It does not infer timezone, before/after-market, confirmed/estimated, ranking, recommendation, or trading judgment beyond the scanner values TradingView returns.

## Progress

- [x] (2026-06-03 16:10Z) Created this ExecPlan and archived the completed `tv bars` symbol resolution plan.
- [x] (2026-06-03 16:25Z) Added `events.v1` typed payloads and event shaping in the Desktop-free market crate.
- [x] (2026-06-03 16:35Z) Added `tv events <SYMBOL> --event-type <all|earnings|dividends>` CLI wiring.
- [x] (2026-06-03 16:45Z) Added focused market tests and CLI contract tests for event readback and help.
- [x] (2026-06-03 16:55Z) Updated public docs, packaged agent guide, and runtime skills.
- [x] (2026-06-03 17:20Z) Ran focused tests, baseline validation, runtime skill validation, and optional live smoke.
- [x] (2026-06-03 17:25Z) Committed the related implementation and documentation changes as `feat(market): Add symbol events readback`.

## Surprises & Discoveries

- Observation: The existing scanner fundamentals group fields already include enough event-like values for a narrow first slice.
  Evidence: The `earnings` group includes next/latest release date, calendar date, trading date, release time, and publication type fields. The `dividends` group includes upcoming/recent ex-date, payment date, amount, frequency, yield, next dividend date, and expected annual dividends fields.

## Decision Log

- Decision: `tv events` uses `scanner_fundamentals_rest` for this first slice.
  Rationale: This keeps the feature Desktop-free, read-only, and aligned with the already-supported fundamentals field inventory. A standalone event/calendar source remains a later candidate.
  Date/Author: 2026-06-03 / Codex

- Decision: The first command shape is `tv events <SYMBOL> --event-type <all|earnings|dividends>`, with `all` as the default.
  Rationale: This gives users a simple event-shaped readback while avoiding date-range, market-wide calendar, or economic-event scope creep.
  Date/Author: 2026-06-03 / Codex

- Decision: Event entries preserve TradingView scanner values without semantic inference.
  Rationale: Scanner fields may not expose timezone, session, or confirmation semantics consistently. The CLI should report what was read, not invent a richer event calendar contract.
  Date/Author: 2026-06-03 / Codex

## Outcomes & Retrospective

Implemented. `tv events <SYMBOL>` now returns a narrow `events.v1` readback
over scanner-backed earnings and dividend fields. The command supports
`--event-type all`, `--event-type earnings`, and `--event-type dividends`.

Public-safe live smoke for `NASDAQ:AAPL` confirmed:

- `tv events NASDAQ:AAPL`: `event_count` 4, event types earnings and dividends,
  `source` `scanner_fundamentals_rest`, availability `events_present`.
- `tv events NASDAQ:AAPL --event-type earnings`: `event_count` 2, source
  `scanner_fundamentals_rest`.
- `tv events NASDAQ:AAPL --event-type dividends`: `event_count` 2, source
  `scanner_fundamentals_rest`.

Focused tests, workspace baseline, docs checks, package-script syntax check,
and runtime skill validation all passed. No raw scanner output, credentials,
session identifiers, account-local metadata, target identifiers, local paths,
or raw payloads were added to tracked docs.

## Context and Orientation

`tv events` is implemented in the Desktop-free market crate as an event-shaped view over scanner fundamentals fields. It does not connect to TradingView Desktop, CDP, selected-chart state, Replay, chart export, quote-data, or scanner quote fallback paths.

The current source marker is `scanner_fundamentals_rest`. The command is intentionally narrower than a full event calendar: it reads symbol-scoped earnings and dividends evidence only.

## Plan of Work

First, add typed `Events`, `EventEntry`, `EventFieldReadback`, and `EventSourceAvailability` payloads in the market crate. Add `events_symbol(...)` and `events_symbol_typed(...)` public APIs that call `fundamentals_symbol_with_groups_typed(...)` with the earnings and/or dividends groups, then shape non-null scanner values into normalized entries.

Second, add the CLI command `tv events <SYMBOL>` with `--event-type <all|earnings|dividends>`. Empty symbols should fail before network access. Unknown event types should fail through clap value validation or market API validation.

Third, update docs and runtime skills so users and agents understand that `tv events` is an event-shaped scanner fundamentals readback, not a full calendar, chart read, ranking surface, recommendation, or trading signal.

## Validation and Acceptance

Run these focused tests and expect them to pass:

    cargo test -p tradingview-market events -- --nocapture
    cargo test -p tradingview-cli events -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_quote -- --nocapture

Run the baseline checks and expect no failures:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

Validate changed runtime skills:

    uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/market-data-interpretation
    uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/multi-symbol-scan
    uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/chart-analysis

Optionally run live Desktop-free smoke commands. Do not paste raw output into tracked docs; record only a public-safe summary if needed.

    target/debug/tv events NASDAQ:AAPL
    target/debug/tv events NASDAQ:AAPL --event-type earnings
    target/debug/tv events NASDAQ:AAPL --event-type dividends

Acceptance means `tv events` returns `contract_version: "events.v1"`, `source: "scanner_fundamentals_rest"`, `source_category: "desktop_free_read"`, `requires_desktop: false`, `non_mutating: true`, requested / resolved symbol readback, event count, event entries, and field availability readback. Empty event fields should produce a successful payload with `event_count: 0` when the source itself was available.

## Idempotence and Recovery

The implementation is additive. Re-running tests is safe. If live scanner fields are unavailable or null for a symbol, record that as source availability evidence instead of adding fallback reads or semantic inference.

## Interfaces and Dependencies

No new crate dependency is added. The new public market APIs are:

    pub async fn events_symbol(symbol: &str, event_type: &str) -> Result<Value, AppError>
    pub async fn events_symbol_typed(symbol: &str, event_type: &str) -> Result<Events, AppError>

No existing `fundamentals`, `snapshot`, `compare`, `bars`, selected-chart, or Replay contract is changed.

## Open Questions

No blocker remains for the first slice. Later work may investigate a standalone event/calendar source, but that is outside this implementation.
