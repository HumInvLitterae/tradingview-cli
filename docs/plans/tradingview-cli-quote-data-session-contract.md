# Quote-data session contract

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is
self-contained and describes how to add additive contract metadata and source
availability readback to `tv quote <SYMBOL> --source quote-data`.

## Purpose / Big Picture

`tv quote <SYMBOL> --source quote-data` is an explicit Desktop-backed source
for bounded TradingView quote-data WebSocket readback. It was introduced after
v0.13 source discovery found that `qsd.rtc` is the strongest current candidate
for the visible after-hours price in TradingView's right-side detail panel.

After this change, agents should be able to tell whether quote-data produced a
source-labeled readback, or whether the source was unavailable during the
bounded wait. They should not have to guess whether a missing `qsd.rtc` means
"no price" or "no matching frame arrived".

This slice does not add a new source. It does not add quote-data to
`--source auto`, does not merge scanner `extended_hours`, and does not change
chart-source quote behavior.

## Progress

- [x] (2026-05-09T19:20Z) Created this ExecPlan after the `v0.13.0` release.
- [ ] Add quote-data contract metadata and source-availability readback.
- [ ] Update tests for success and structured unavailable details.
- [ ] Synchronize docs and runtime skills.
- [ ] Run validation and record outcomes.

## Surprises & Discoveries

- Observation: `quote-data` already reports public-safe unavailable details.
  Evidence: `crates/cli/src/ops/market/quote_data.rs` includes a
  `wait_summary` with bounded wait, WebSocket event counts, qsd message counts,
  matching counts, and `raw_frame_included: false`.

## Decision Log

- Decision: Add command-local `contract_version: "quote_data.v1"` instead of
  a global envelope version.
  Rationale: the stable contract being matured is specific to the explicit
  `quote-data` source. Other quote sources already have different payload
  shapes and should not inherit this marker.
  Date/Author: 2026-05-09 / Codex.

- Decision: Keep `quote-data` outside `--source auto`.
  Rationale: `quote-data` is a bounded Desktop-backed WebSocket readback that
  can be unavailable when no matching frame arrives. Adding it to automatic
  source selection would blur the scanner, chart, and quote-data boundaries
  that v0.13 clarified.
  Date/Author: 2026-05-09 / Codex.

## Outcomes & Retrospective

In progress.

## Context and Orientation

The CLI command definition is in `crates/cli/src/cli.rs`. It already exposes
`quote-data` as one value of the `--source` option for `tv quote`.

Command dispatch is in `crates/cli/src/app/dispatch.rs`. The `quote-data`
branch requires a symbol, connects to TradingView Desktop through Chrome
DevTools Protocol, and calls `ops::quote_data`.

The implementation lives in `crates/cli/src/ops/market/quote_data.rs`. It
enables CDP Network events, observes bounded WebSocket frame events, parses
TradingView `qsd` quote-data messages, and returns a success payload when a
matching `qsd.rtc` is found. If the bounded wait expires, it returns a
structured `internal_api_unavailable` error with public-safe wait details.

In this document, "source availability" means whether the bounded
Desktop-backed quote-data observation produced a matching quote-data readback.
It is not a statement about whether the symbol has a market price.

## Plan of Work

Update `crates/cli/src/ops/market/quote_data.rs` so both success payloads and
structured unavailable details include `contract_version:
"quote_data.v1"`. Add a small `source_availability` object that can be read
the same way in success and unavailable cases. The object should say whether
`rtc` was observed, whether the source was available during the bounded wait,
and should preserve the existing public-safe bounded wait and message-count
evidence.

Keep the existing `quote_data` object on success with `rtc`, `rtc_time`,
`rch`, `rchp`, `current_session`, `market_phase`, and `update_mode`. Do not add
scanner-style `extended_hours`, chart main-series fields, raw WebSocket
frames, raw DOM, or target identifiers.

Update tests in the quote-data module and CLI contract tests so they assert
the new contract marker and source-availability readback for success and
unavailable details. The tests should continue to prove that `--source auto`
does not include quote-data and that `quote --source quote-data` requires a
symbol.

Synchronize `docs/command-source-taxonomy.md`, `docs/observation-workflows.md`,
`docs/internal-tradingview-apis.md`, `.agents/skills/market-data-interpretation/SKILL.md`,
and `.agents/skills/chart-analysis/SKILL.md` so agents read quote-data
availability as source availability, not as a missing market price.

## Concrete Steps

From the repository root, edit the quote-data implementation and tests, then
run:

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

Acceptance is reached when `tv quote <SYMBOL> --source quote-data` success
payloads contain `contract_version: "quote_data.v1"` and a source-availability
readback without changing the existing `quote_data.rtc` shape. Structured
unavailable details must also contain the same contract marker, source label,
and public-safe availability summary.

Tests must show that scanner-backed quote, chart-source quote, and
`--source auto` behavior are unchanged. The live quote-data smoke remains
ignored by default and accepts either a valid success contract or a structured
unavailable contract.

## Idempotence and Recovery

The implementation is additive and safe to rerun. If quote-data events do not
arrive during live testing, treat that as an unavailable source result rather
than a failed market quote. If validation fails outside quote-data, inspect the
failure before broadening the slice.

## Interfaces and Dependencies

No new dependency is required. Continue using the existing CDP client and
serde JSON payload construction. The public CLI surface remains
`tv quote <SYMBOL> --source quote-data`; no new option or command is added.

The new payload fields are:

    contract_version: "quote_data.v1"
    source_availability: {
      available: boolean,
      status: "available" | "unavailable",
      rtc_observed: boolean,
      raw_frame_included: false,
      wait_summary: object
    }

The exact `wait_summary` fields should remain public-safe and should not
include raw frames, raw payloads, target ids, account-local metadata, cookies,
tokens, or local paths.

## Open Questions

Premarket-specific behavior is still unconfirmed. Run the existing opt-in live
smoke during a real premarket window before deciding whether v0.14 should add
more session-specific docs or fields.

## Revision Note

Created after `v0.13.0` release readiness to make the first v0.14
implementation slice decision-complete while preserving source boundaries.
