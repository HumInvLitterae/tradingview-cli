# Quote-data diagnostics

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is
self-contained and describes how to plan the first `v0.15.0` implementation
slice: a narrow quote-data diagnostic surface for the existing explicit
`tv quote <SYMBOL> --source quote-data` source.

## Purpose / Big Picture

`tv quote <SYMBOL> --source quote-data` can now report whether a bounded
TradingView Desktop quote-data WebSocket read was available. When it is not
available, users and agents still need to run several commands to understand
where the read was blocked. After this change, a user should be able to run a
narrow diagnostic command, planned as `tv diagnose quote-data <SYMBOL>`, and
see whether the problem is Desktop target selection, absent WebSocket traffic,
no TradingView `qsd` quote-data messages, no matching requested symbol, or no
`qsd.rtc` for the matching symbol.

The planned command is diagnostic only. It must not add a new market-data
source, merge scanner delayed values with Desktop quote-data values, change
chart-source quote, or become a broad all-purpose health-check command.

## Progress

- [x] (2026-05-11T12:00Z) Created the initial `v0.15.0` roadmap direction and
  this first implementation ExecPlan.
- [ ] Implement the narrow `tv diagnose quote-data <SYMBOL>` command surface.
- [ ] Add focused contract tests for diagnostic success and unavailable paths.
- [ ] Update docs and runtime skills for quote-data diagnostics.
- [ ] Run validation and commit the related changes in one local commit.

## Surprises & Discoveries

- Observation: The post-release quote-data live smoke target hardening showed
  that premarket validation can be blocked before it reaches source
  diagnostics if multiple chart targets are open and the smoke cannot pass an
  explicit target id.
  Evidence: The previous plan added `TV_LIVE_QUOTE_DATA_TARGET_ID` and was
  committed as `a7649f0 test(cli): Harden quote-data live smoke target
  selection`.

- Observation: A bounded quote-data read can be unavailable even when some
  WebSocket or `qsd` activity exists.
  Evidence: The v0.14 diagnostics contract distinguishes
  `no_websocket_events`, `no_websocket_frames`, `no_qsd_messages`,
  `no_matching_symbol`, and `no_rtc`.

## Decision Log

- Decision: Make the first v0.15 implementation candidate a narrow
  `tv diagnose quote-data <SYMBOL>` command rather than a broad
  `tv diagnose all` command.
  Rationale: The repeated manual work is currently specific to quote-data
  target and WebSocket availability. A broad diagnostic command would invite
  unrelated source checks and release risk.
  Date/Author: 2026-05-11 / Codex.

- Decision: Keep scanner, chart, and quote-data values separated in the
  diagnostic output.
  Rationale: Scanner quote can be delayed REST data, chart quote is selected
  chart main-series data, and quote-data is Desktop-backed WebSocket readback.
  The diagnostic can show source status side by side, but it must not
  synthesize a single price.
  Date/Author: 2026-05-11 / Codex.

## Outcomes & Retrospective

This plan starts the `v0.15.0` lane after `v0.14.0` release and post-release
smoke hardening. No implementation has happened yet. The expected outcome is
a narrow diagnostic command that makes quote-data source availability easier
to understand without changing the existing quote source contracts.

## Context and Orientation

The CLI entry point is `crates/cli/src/cli.rs`. It defines the top-level
`Command` enum parsed by `clap`, the Rust command-line parser used by this
project. Dispatch lives in `crates/cli/src/app/dispatch.rs`, where parsed
commands call operation functions from `crates/cli/src/ops/`.

The existing quote-data implementation lives in
`crates/cli/src/ops/market/quote_data.rs`. It observes TradingView Desktop
WebSocket messages for a bounded window. A WebSocket is a browser connection
that can push messages over time. TradingView uses messages containing `qsd`
quote-data updates; `qsd.rtc` is the source-labeled readback currently exposed
by `tv quote <SYMBOL> --source quote-data` when a matching requested symbol
and `rtc` value are observed.

The existing command `tv quote <SYMBOL> --source quote-data` returns
`source: "desktop_quote_data_ws"` and `contract_version: "quote_data.v1"` on
success and in structured unavailable details. Its `source_availability`
object includes `available`, `status`, `rtc_observed`,
`unavailable_reason`, `timed_out`, `next_action`, `raw_frame_included`, and a
public-safe `wait_summary`. Public-safe means the payload avoids raw WebSocket
frames, raw live payloads, account-local metadata, and raw target ids in
tracked documentation.

Target selection uses the global `--target-id <CDP_TARGET_ID>` CLI option.
CDP means Chrome DevTools Protocol, the local debug interface used to read
TradingView Desktop pages. Multiple TradingView chart targets can be open, so
Desktop-backed commands may need an explicit target id.

## Plan of Work

Add a top-level `diagnose` command group in `crates/cli/src/cli.rs` only if no
such command exists yet. The first subcommand should be `quote-data` and it
should require a `SYMBOL` positional argument. The help text should say this
diagnoses the explicit Desktop-backed quote-data source and does not merge
scanner, chart, or quote-data prices.

Dispatch `tv diagnose quote-data <SYMBOL>` from
`crates/cli/src/app/dispatch.rs` to a new operation under `crates/cli/src/ops/`.
The operation should reuse the existing quote-data bounded read path where
possible instead of duplicating WebSocket parsing logic. If helper functions in
`quote_data.rs` need to be made visible within the crate, do that with
crate-private functions rather than new public crate APIs.

The diagnostic payload should include:

- `source: "quote_data_diagnostics"`;
- `source_category: "desktop_backed_read"`;
- `requires_desktop: true`;
- `non_mutating: true`;
- `requested_symbol`;
- a `desktop_target` summary that indicates selected, missing, ambiguous, or
  target-id-not-found without requiring tracked docs to paste raw target ids;
- `quote_data_contract_version: "quote_data.v1"`;
- the quote-data `source_availability` object or structured unavailable
  details from the bounded read;
- a `scanner_reference` section that may call the Desktop-free scanner quote
  path for freshness context, clearly labeled as separate and never merged
  with quote-data;
- `next_action_hints` that explain whether to retry quote-data, specify
  `--target-id`, inspect `tv tab list`, or use scanner data when delayed REST
  is acceptable.

Do not call `tv quote --source chart` with a requested symbol from this
diagnostic, because that path can switch the selected chart symbol. If current
chart context is included later, it must be read-only and must not mutate the
chart.

Update docs to describe the diagnostic as a troubleshooting surface for
source availability, not a price source. Update runtime skills only where they
currently ask agents to manually combine `tv tab list`, quote-data, and
scanner quote for source debugging.

## Concrete Steps

From the repository root, inspect the current command definitions and quote
data helpers:

    rg -n "QuoteSource|quote_data|Command::Quote|readiness|tab list" crates/cli/src

Implement the command and run focused tests:

    cargo test -p tradingview-cli market::quote_data -- --nocapture
    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture

Add or update CLI contract tests so `tv diagnose quote-data --help` and
`tv diagnose quote-data NASDAQ:RKLB` payload behavior are covered without
requiring live Desktop unless the test is explicitly ignored.

After implementation, run the full validation set:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

If running optional live evidence during a relevant market phase, use an
explicit target when multiple chart targets are open:

    tv tab list
    tv --target-id <ID> diagnose quote-data NASDAQ:RKLB

Do not paste the raw target id, raw WebSocket frames, or raw live payloads
into tracked docs.

## Validation and Acceptance

Acceptance is reached when `tv diagnose quote-data <SYMBOL>` returns a JSON
envelope that helps a user tell whether quote-data is blocked at target
selection, WebSocket visibility, `qsd` message observation, requested-symbol
matching, or `qsd.rtc` availability.

The diagnostic must preserve existing quote contracts. `tv quote <SYMBOL>
--source scanner`, `tv quote <SYMBOL> --source chart`, `tv quote <SYMBOL>
--source quote-data`, and `tv quote <SYMBOL> --source auto` must keep their
current meanings. In particular, quote-data must not be added to `--source
auto`, and the diagnostic must not synthesize a single price from scanner and
quote-data.

Structured unavailable quote-data should be acceptable diagnostic output. It
means the source was unavailable during the bounded wait, not that the symbol
has no market price.

## Idempotence and Recovery

The diagnostic command must be read-only and safe to rerun. It should not
switch symbols, activate tabs, capture screenshots, or mutate TradingView
state.

If Desktop target selection is ambiguous, the diagnostic should report that
state and point to `tv tab list` and `tv --target-id <ID> diagnose quote-data
<SYMBOL>`. If scanner reference fails, the diagnostic should keep that failure
inside the scanner section rather than hiding quote-data diagnostics.

## Interfaces and Dependencies

The implementation should use existing workspace crates and dependencies.
Do not add a new dependency.

The new public CLI interface planned by this ExecPlan is:

    tv diagnose quote-data <SYMBOL>

The command should respect the existing global `--target-id` option:

    tv --target-id <ID> diagnose quote-data <SYMBOL>

No new JSON field should be added to `tv quote <SYMBOL> --source quote-data`
as part of this diagnostic slice unless a bug in the existing quote-data
contract blocks the diagnostic. If such a bug is found, record it in
`Surprises & Discoveries` and keep the fix additive.

## Open Questions

None. The first slice is intentionally narrow. Broader diagnostics, automatic
source fallback, and source mixing remain deferred.
