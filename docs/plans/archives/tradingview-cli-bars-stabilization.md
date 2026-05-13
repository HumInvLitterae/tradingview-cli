# `tv bars` stable browserless bars contract plan

This ExecPlan is a living document. Keep `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` updated as work
continues.

## Purpose

Stabilize `tv bars <EXCHANGE:SYMBOL>` as a Desktop-free historical bars read.
The command already existed behind a lab gate; this slice removes that gate and
turns its bounded WebSocket read into a source-labeled `bars.v1` contract.

This is not a realtime feed, watch command, JSONL compare, chart-backed
compare, or recommendation surface.

## Background

`tv bars` was introduced as a lab-gated browserless TradingView WebSocket
prototype. It proved useful enough to keep, but the `TV_EXPERIMENTAL_BARS=1`
gate and `experimental_tradingview_ws` source made it awkward for ordinary
agent workflows.

The stabilized command remains explicit and bounded. It still uses an
undocumented TradingView WebSocket chart-session path, reports no realtime or
entitlement guarantee, and stays separate from selected-chart `tv ohlcv`.

## Progress

- [x] Archived the completed quote-data regular-session semantics plan.
- [x] Created this ExecPlan and made it the current v0.16 plan.
- [x] Removed the `TV_EXPERIMENTAL_BARS` runtime gate.
- [x] Added `contract_version: "bars.v1"` and stable source taxonomy metadata.
- [x] Updated CLI help, contract tests, README, docs, runtime skills, and
  packaged agent guidance.
- [x] Ran focused bars and CLI contract tests.
- [x] Ran full release-slice validation.
- [x] Ran optional read-only live smoke for one daily request and one
  one-minute request; both returned `bars.v1` payloads with the stable source
  metadata and requested bar counts.

## Surprises & Discoveries

- The existing parser and request validation were already narrow enough for a
  stable bounded read: exchange-qualified symbol, supported timeframe, and
  count range are validated before the WebSocket read.
- The important stabilization step was not a new source. It was removing the
  lab gate while keeping the source boundary visible and conservative.
- Read-only live smoke confirmed the stabilized command works without the
  experimental environment gate for public exchange-qualified symbols.

## Decision Log

- Use `contract_version: "bars.v1"` as a command-local marker.
- Use `source: "tradingview_bars_ws"` and
  `source_category: "desktop_free_read"`.
- Keep `requires_desktop: false` and `non_mutating: true`.
- Keep warnings that the WebSocket path is undocumented and does not guarantee
  realtime or entitlement status.
- Do not change `tv ohlcv`; it remains the selected-chart/CDP bars read.
- Do not add browserless streaming, watch/JSONL compare, automatic fallback,
  ranking, scoring, or recommendations.
- Do not expose raw WebSocket frames, raw payloads, account-local metadata,
  credentials, or local validation details.

## Plan Of Work

1. Stabilize the CLI and payload.
   - Remove the `TV_EXPERIMENTAL_BARS` validation gate.
   - Rename the source from `experimental_tradingview_ws` to
     `tradingview_bars_ws`.
   - Add stable source metadata and `bars.v1` to success and structured
     unavailable details.

2. Preserve conservative behavior.
   - Keep exchange-qualified symbols required.
   - Keep bounded timeframe and count validation.
   - Keep `data_quality.realtime_guarantee: false` and
     `entitlement_checked: false`.
   - Keep no-bars, timeout, and WebSocket errors as structured failures.

3. Sync public docs and runtime skills.
   - Remove user-facing lab-gate examples.
   - Explain `tv bars` as Desktop-free historical bars evidence, not
     realtime or chart-selected evidence.
   - Keep `tv ohlcv`, scanner quote, chart quote, and quote-data boundaries
     separate.

## Validation

- `cargo test -p tradingview-cli market::bars -- --nocapture`
- `cargo test -p tradingview-cli --test cli_contract bars -- --nocapture`
- `cargo test -p tradingview-cli --test live_bars`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo metadata --no-deps --format-version 1`
- `git diff --check`
- `bash -n scripts/stage-release-package-files.sh`

Optional live smoke:

- `target/debug/tv bars NASDAQ:AAPL --timeframe 1D --count 5`
- `target/debug/tv bars NASDAQ:RKLB --timeframe 1 --count 10`

If live smoke is run, record only public-safe summary. Do not paste raw live
payloads into tracked docs.

## Acceptance Criteria

- `tv bars` no longer requires `TV_EXPERIMENTAL_BARS=1`.
- Success payloads include `contract_version: "bars.v1"`,
  `source: "tradingview_bars_ws"`, `source_category: "desktop_free_read"`,
  `requires_desktop: false`, and `non_mutating: true`.
- Structured failures include the same stable source metadata.
- Docs and skills no longer tell runtime users to treat `tv bars` as a
  lab-gated command.
- `tv bars` remains separate from `tv ohlcv`, `tv stream bars`, scanner quote,
  chart-source quote, and quote-data.

## Outcomes & Retrospective

The slice stabilized `tv bars` without adding realtime streaming or source
mixing. The command now reports `contract_version: "bars.v1"`,
`source: "tradingview_bars_ws"`, `source_category: "desktop_free_read"`,
`requires_desktop: false`, and `non_mutating: true` on success. It still
reports `data_quality.realtime_guarantee: false` and
`entitlement_checked: false`, so agents should treat it as bounded historical
bars evidence rather than realtime feed evidence.
