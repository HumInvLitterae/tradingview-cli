# Quote-data availability diagnostics

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is
self-contained and describes how to add additive availability diagnostics to
`tv quote <SYMBOL> --source quote-data`.

## Purpose / Big Picture

`tv quote <SYMBOL> --source quote-data` now has `contract_version:
"quote_data.v1"` and `source_availability`, but an unavailable result still
needs more machine-readable detail. After this change, agents should be able
to distinguish no WebSocket activity, no quote-data messages, symbol mismatch,
and matching quote-data messages without `rtc`, without inspecting raw
WebSocket frames or guessing that the symbol has no price.

This slice does not add a new command, option, source, dependency, or version
bump. It keeps `quote-data` explicit and outside `--source auto`.

## Progress

- [x] (2026-05-10T00:15Z) Created this ExecPlan and archived the completed
  quote-data session contract plan.
- [x] (2026-05-10T00:35Z) Added availability reason, timing, and
  next-action readback.
- [x] (2026-05-10T00:35Z) Added public-safe wait-summary counters.
- [x] (2026-05-10T00:35Z) Added quote-data session readback normalization.
- [x] (2026-05-10T00:45Z) Updated tests, docs, and runtime skills.
- [x] (2026-05-10T01:20Z) Ran validation and recorded outcomes.

## Surprises & Discoveries

- Observation: the current observer already has enough structure to classify
  unavailable states without storing raw frames.
  Evidence: it counts WebSocket events/frames and parses quote-data messages
  in one bounded pass.

- Observation: symbol matching must be counted before checking whether a qsd
  payload has `rtc`.
  Evidence: otherwise a requested-symbol qsd without `rtc` can look like a
  generic `no_matching_symbol` result instead of the more useful `no_rtc`
  source diagnostic.

## Decision Log

- Decision: keep `contract_version` at `quote_data.v1`.
  Rationale: this is an additive extension of the v0.14 quote-data contract,
  not a breaking schema replacement.
  Date/Author: 2026-05-10 / Codex.

- Decision: use `unavailable_reason` as source diagnostics only.
  Rationale: unavailable quote-data means the bounded Desktop-backed source did
  not produce usable `qsd.rtc`; it does not mean the market lacks a price.
  Date/Author: 2026-05-10 / Codex.

## Outcomes & Retrospective

Implemented. `tv quote <SYMBOL> --source quote-data` now keeps
`contract_version: "quote_data.v1"` and adds source diagnostics without
changing source behavior. Success payloads include `source_availability` with
`unavailable_reason: null`, `timed_out: false`, `next_action: null`, expanded
public-safe wait-summary counters, and `quote_data.session_readback` with
spelling-only normalized session fields. Structured unavailable details now
classify bounded-read failures as `no_websocket_events`,
`no_websocket_frames`, `no_qsd_messages`, `no_matching_symbol`, or `no_rtc`,
with a source-diagnostic `next_action`.

The implementation does not include raw WebSocket frames, raw payloads, target
ids, account-local metadata, scanner-style `extended_hours`, chart main-series
fields, chart-source `session_boundary`, automatic fallback, or `--source
auto` behavior.

Validation passed:

- `cargo test -p tradingview-cli market::quote_data -- --nocapture`
- `cargo test -p tradingview-cli --test cli_contract quote -- --nocapture`
- `cargo test -p tradingview-cli --test live_quote_data_source`
- runtime skill validation for `market-data-interpretation`
- runtime skill validation for `chart-analysis`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo metadata --no-deps --format-version 1`
- `git diff --check`
- `bash -n scripts/stage-release-package-files.sh`

The hygiene scan reported existing safety policy wording, archived validation
examples, test fixtures, and this plan's own safety wording. It did not
identify a newly introduced raw WebSocket frame, raw live payload, target id,
account-local metadata, credential, local absolute path, or downstream-private
path in the changed public docs or runtime skills.

## Context and Orientation

The implementation is in `crates/cli/src/ops/market/quote_data.rs`. The
current observer receives CDP Network WebSocket events, parses TradingView
socket packets, tracks quote-session symbol mappings, and returns a success
payload when a matching `qsd.rtc` appears.

The live smoke is `crates/cli/tests/live_quote_data_source.rs`. It is ignored
by default and should continue to accept either a valid success payload or a
structured unavailable payload.

## Plan of Work

Extend the observer with public-safe counters:

- `qsd_with_rtc_seen`
- `matching_symbol_qsd_seen`
- `matching_symbol_without_rtc_seen`
- `quote_session_symbol_mappings_seen`

Use those counters to derive `source_availability.unavailable_reason` for
structured unavailable details:

- `no_websocket_events`
- `no_websocket_frames`
- `no_qsd_messages`
- `no_matching_symbol`
- `no_rtc`

Add `timed_out` and `next_action` to `source_availability`. For success,
`unavailable_reason` is `null`, `timed_out` is `false`, and `next_action` is
`null`. For unavailable details, `timed_out` is `true` and `next_action` is
one of `retry_quote_data`, `check_desktop_streaming_symbol`, or
`use_scanner_if_delayed_rest_ok`.

Add `quote_data.session_readback` to success payloads. It should repeat the
source session fields and include normalized spelling variants only:
`market_phase`, `market_phase_normalized`, `current_session`,
`current_session_normalized`, `session_source:
"tradingview_quote_data_fields"`, and `session_inferred: false`.

Do not add raw frames, raw payloads, target ids, account-local metadata,
scanner-style `extended_hours`, chart main-series OHLCV, chart-source
`session_boundary`, automatic fallback, or `--source auto` behavior.

## Concrete Steps

From the repository root, edit quote-data implementation, tests, docs, and
runtime skills. Then run:

    cargo test -p tradingview-cli market::quote_data -- --nocapture
    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture
    cargo test -p tradingview-cli --test live_quote_data_source
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

If runtime skills are changed, validate them with the existing local skill
validator when available. Do not record local validator paths in tracked docs.

## Validation and Acceptance

Acceptance is reached when success payloads preserve the existing
`quote_data.rtc` shape and include additive `session_readback`. Unavailable
details must include stable `unavailable_reason`, `timed_out`, and
`next_action` without raw WebSocket frames or raw payloads.

Focused tests must cover all unavailable reasons. Existing scanner, chart,
snapshot, compare, and `--source auto` contracts must remain unchanged.

## Idempotence and Recovery

The implementation is additive and safe to rerun. If live quote-data frames do
not arrive, treat that as a structured source unavailable result. If validation
fails outside quote-data, inspect the failure before broadening the slice.

## Interfaces and Dependencies

No new dependency is required. The public CLI surface remains:

    tv quote <SYMBOL> --source quote-data

The new additive fields are:

    source_availability.unavailable_reason
    source_availability.timed_out
    source_availability.next_action
    wait_summary.qsd_with_rtc_seen
    wait_summary.matching_symbol_qsd_seen
    wait_summary.matching_symbol_without_rtc_seen
    wait_summary.quote_session_symbol_mappings_seen
    quote_data.session_readback

## Open Questions

Premarket-specific behavior remains unconfirmed. Run the existing opt-in live
smoke during a real premarket window before deciding whether to add
premarket-specific docs or fields.

## Revision Note

Created after `quote_data.v1` source availability landed, because v0.14 should
include as much quote-data diagnostics polish as possible before release
readiness.
