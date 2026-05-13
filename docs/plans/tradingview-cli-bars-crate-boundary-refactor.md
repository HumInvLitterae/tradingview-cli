# `tv bars` crate-boundary refactor

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It records
the behavior-preserving refactor that moves browserless historical bars logic
out of the CLI operation adapter and into the Desktop-free market crate.

## Purpose / Big Picture

`tv bars <EXCHANGE:SYMBOL>` is a Desktop-free bounded historical OHLCV read.
The command had stabilized in v0.16 / v0.17, but its implementation still lived
under `crates/cli/src/ops/market/bars.rs` with WebSocket protocol handling,
request validation, payload shaping, source availability, and tests.

That made `ops` start acting like a data-source implementation instead of a
thin command adapter. This refactor restores the intended boundary:
`tradingview-market` owns Desktop-free market reads, while CLI `ops` only
adapts the command to the reusable read.

## Progress

- [x] Add this ExecPlan and switch current planning docs to this refactor.
- [x] Move browserless bars implementation and tests to `crates/market`.
- [x] Keep CLI `ops` as a thin wrapper around `tradingview_market::bars_symbol`.
- [x] Move browserless WebSocket dependencies from CLI runtime dependencies to
  the market crate, leaving CLI integration-test dependencies where needed.
- [x] Update architecture / development / Rust API docs for the boundary.
- [x] Run focused tests, baseline checks, and docs hygiene.
- [x] Commit the refactor.

## Surprises & Discoveries

- `crates/cli` still needs `futures-util` and `tokio-tungstenite` as
  dev-dependencies because opt-in live evidence tests open bounded WebSocket
  streams directly.
- The public `bars.v1` JSON contract did not need to change. Moving the logic
  to `tradingview-market` only changes the internal crate boundary.

## Decision Log

- Decision: Move `tv bars` implementation into `tradingview-market` instead
  of splitting it into more CLI `ops` submodules.
  Rationale: `tv bars` is Desktop-free, credential-free, and does not depend
  on CDP, chart state, UI automation, or account mutation. That matches the
  market crate boundary better than the operation-adapter boundary.
  Date/Author: 2026-05-14 / Codex.

- Decision: Keep only a JSON-returning `bars_symbol` API public for this slice.
  Rationale: v0.17 needs behavior-preserving cleanup before release readiness,
  not a new stable Rust typed API. Typed bars structs can be promoted later if
  downstream Rust callers need them.
  Date/Author: 2026-05-14 / Codex.

## Outcomes & Retrospective

`tv bars` now routes through `tradingview_market::bars_symbol`. The CLI
operation adapter contains no WebSocket protocol logic, payload construction,
or bars availability shaping.

The public command contract remains unchanged: successful payloads and
structured failures still use `contract_version: "bars.v1"`,
`source: "tradingview_bars_ws"`, `source_category: "desktop_free_read"`,
`requires_desktop: false`, `non_mutating: true`, `summary`, `range`,
`data_quality.partial_result`, `source_availability`, and public-safe
`wait_summary`.

## Plan of Work

1. Move `crates/cli/src/ops/market/bars.rs` implementation into
   `crates/market/src/bars.rs` and export `bars_symbol`.
2. Replace the CLI operation adapter with a thin call into
   `tradingview_market::bars_symbol`.
3. Move WebSocket implementation dependencies to `tradingview-market`, keeping
   CLI live-test-only dependencies as dev-dependencies.
4. Update durable docs to record the boundary decision.
5. Run focused tests and full workspace validation.

## Acceptance Criteria

- `tv bars` command behavior and JSON contract are unchanged.
- Bars implementation tests run under `tradingview-market`.
- CLI `ops` no longer owns browserless bars WebSocket protocol or payload
  shaping.
- Docs describe `tv bars` as a Desktop-free market crate read exposed through a
  thin CLI adapter.
- No raw WebSocket frames, raw payloads, session ids, credentials,
  account-local metadata, target ids, or local paths are added to public docs
  or packaged assets.

## Validation

Run:

    cargo test -p tradingview-market bars -- --nocapture
    cargo test -p tradingview-cli market::bars -- --nocapture
    cargo test -p tradingview-cli --test cli_contract bars -- --nocapture
    cargo test -p tradingview-cli --test live_bars
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

Optional read-only smoke:

    target/debug/tv bars NASDAQ:AAPL --timeframe 1D --count 5
    target/debug/tv bars NASDAQ:RKLB --timeframe 1 --count 10

Live output must not be pasted into tracked docs.

## Interfaces and Dependencies

No public CLI interface changes. `tradingview-market` now depends on
`futures-util`, `tokio` time support, and `tokio-tungstenite` for the
browserless historical bars WebSocket read. `crates/cli` keeps those WebSocket
dependencies only for opt-in live integration tests that still use them
directly.
