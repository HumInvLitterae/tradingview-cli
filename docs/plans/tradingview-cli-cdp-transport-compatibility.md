# Improve CDP transport compatibility

This ExecPlan is a living document. The sections `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to
date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is
self-contained so a reader can understand and continue the work without chat
history.

## Purpose / Big Picture

TradingView Desktop 3.1.0 / Electron 38 can expose CDP differently from older
builds. Upstream PR #108 reports three compatibility risks: `localhost` can fail
where `127.0.0.1` works, initial `Runtime.enable` / `Page.enable` / `DOM.enable`
calls can hang, and desktop file URL targets can be missed by target discovery.
After this change, the Rust CLI defaults to `127.0.0.1`, connects without
domain-enable bootstrap calls, and makes app-window targets more visible without
pretending they are chart API targets.

## Progress

- [x] (2026-04-28 03:25Z) Re-read upstream PR #108 current body and compared it
  with `src/cdp.rs`, `src/transport.rs`, and `src/ops/tab.rs`.
- [x] (2026-04-28 03:45Z) Implemented transport and CDP compatibility changes.
- [x] (2026-04-28 03:55Z) Updated docs, notes, release package guide, and
  `CONTINUITY.md`.
- [x] (2026-04-28 04:15Z) Ran automated validation and live smoke.
- [ ] Commit the completed slice.

## Surprises & Discoveries

- Observation: Rust still defaults to `localhost`, and `CdpClient::connect`
  still sends `Runtime.enable`, `Page.enable`, and `DOM.enable` before any
  operation.
  Evidence: `src/transport.rs` default host is `localhost`; `src/cdp.rs`
  connects then calls the three enable methods.
- Observation: Rust already uses the app-window target for app-tab operations,
  but chart target selection intentionally does not select the file URL app
  window.
  Evidence: `src/ops/tab.rs` searches `/app/window/index.html` for app tab
  reads; `src/transport.rs` selects `tradingview.com/chart` first, then other
  TradingView pages, and returns none for file-only app-window targets.
- Observation: Live app-window target metadata can include machine-local file
  paths and renderer initialization details.
  Evidence: `tv tab list` exposed an app-window file URL before sanitization.
  The final implementation reports app-window title and URL as public-safe
  placeholders while preserving `target_cli_args`.

## Decision Log

- Decision: Default to `127.0.0.1:9222`, while keeping `TV_CDP_HOST` and
  `TV_CDP_PORT` overrides.
  Rationale: Upstream evidence says `127.0.0.1` avoids localhost resolution
  issues on some Macs, and explicit overrides preserve user control.
  Date/Author: 2026-04-28 / Codex.
- Decision: Remove initial CDP domain enable calls.
  Rationale: The CLI's current operations use direct CDP methods such as
  `Runtime.evaluate`, `Page.captureScreenshot`, and `Input.*`; upstream evidence
  says those work in TradingView Desktop without pre-enabling domains, while
  enable calls can hang on Electron 38.
  Date/Author: 2026-04-28 / Codex.
- Decision: Do not select the file URL app-window target as an automatic chart
  command target.
  Rationale: The app window is useful for app-tab UI operations, but
  `TradingViewApi` chart internals may live in chart webviews rather than the
  outer app window.
  Date/Author: 2026-04-28 / Codex.
- Decision: Keep Windows MSIX / AUMID launch policy out of this slice.
  Rationale: It is related upstream compatibility evidence, but it affects
  launch policy rather than CDP connection behavior.
  Date/Author: 2026-04-28 / Codex.

## Outcomes & Retrospective

Implemented. The CLI now defaults to `127.0.0.1:9222`, keeps
`TV_CDP_HOST` / `TV_CDP_PORT` endpoint overrides, and no longer sends
`Runtime.enable`, `Page.enable`, or `DOM.enable` during WebSocket connection.

`tv tab list` now reports app-window target diagnostics with sanitized title
and URL fields plus `target_cli_args`. Chart command auto-selection still
prefers `tradingview.com/chart` targets and does not treat file URL app-window
targets as chart API targets.

Validation passed:

    cargo test cdp -- --nocapture
    cargo test transport -- --nocapture
    cargo test tab -- --nocapture
    cargo test --test cli_contract status -- --nocapture
    cargo test --test cli_contract tab -- --nocapture
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check

Live smoke passed against a running TradingView Desktop session:

    tv status
    tv tab list
    tv --target-id <target> quote
    tv --target-id <target> screenshot --region chart --output target/cdp-compat-smoke.png
    TV_CDP_HOST=localhost tv status

The smoke created `target/cdp-compat-smoke.png`. No live target id, local path,
or account-local value was written into tracked docs.

## Context and Orientation

`src/transport.rs` owns the CDP HTTP endpoint and target discovery.
`src/cdp.rs` owns WebSocket connection and CDP method calls. `src/ops/tab.rs`
uses a TradingView app-window target for Desktop tab-strip operations.

## Plan of Work

First, change the default host from `localhost` to `127.0.0.1` and update
tests/docs. `TV_CDP_HOST` and `TV_CDP_PORT` remain supported.

Second, remove the three initial domain-enable calls in `CdpClient::connect`.
Keep method-level timeout and error handling unchanged.

Third, surface app-window target information without widening chart selection.
`tab list` should expose app-window target metadata for diagnostics, while
chart command selection should continue to require chart or TradingView web
targets. `No TradingView chart target found` should include a clear retry hint.

Fourth, refresh roadmap, upstream PR notes, README, architecture, release
package guide, and `CHANGELOG.md`.

## Concrete Steps

Run commands from the repository root.

1. Update `src/transport.rs`, `src/cdp.rs`, and `src/ops/tab.rs`.
2. Update unit and CLI contract tests.
3. Update docs and `CONTINUITY.md`.
4. Run:

       cargo test cdp -- --nocapture
       cargo test transport -- --nocapture
       cargo test tab -- --nocapture
       cargo test --test cli_contract status -- --nocapture
       cargo test --test cli_contract tab -- --nocapture
       cargo fmt --check
       cargo clippy --all-targets --all-features -- -D warnings
       cargo test
       git diff --check

5. If TradingView Desktop is available, run bounded live smoke with
   placeholders in docs:

       tv status
       tv tab list
       tv --target-id <target> quote
       tv --target-id <target> screenshot --region chart --output target/cdp-compat-smoke.png

## Validation and Acceptance

The change is accepted when the CLI defaults to `127.0.0.1:9222`, explicit host
overrides still work, `CdpClient::connect` no longer sends domain-enable
methods, app-window targets remain visible for tab diagnostics, and chart
commands do not automatically select file URL app-window targets as chart API
targets.

## Idempotence and Recovery

The code changes are connection-boundary changes and do not mutate TradingView
state. Live smoke creates only a screenshot file under `target/`.

## Artifacts and Notes

Do not copy live target ids, account ids, cookies, or machine-specific local
paths into tracked docs.

## Interfaces and Dependencies

The public CLI surface does not change. `--target-id`, `TV_CDP_HOST`, and
`TV_CDP_PORT` remain available.

## Open Questions

None for this slice. Windows MSIX / AUMID launch compatibility remains a later
plan.
