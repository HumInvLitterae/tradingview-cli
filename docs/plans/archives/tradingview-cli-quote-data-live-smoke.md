# Quote-data live contract smoke

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root.

## Purpose / Big Picture

`tv quote <SYMBOL> --source quote-data` is now an explicit Desktop-backed
WebSocket quote-data readback source. It observes bounded TradingView `qsd`
messages and returns `quote_data.rtc` when the selected page emits a matching
symbol frame.

This slice adds an opt-in ignored live contract smoke so the source can be
checked against a real TradingView Desktop page without making live frame
availability a CI requirement. A no-frame result is a valid availability
outcome when it is returned as structured unavailable details and does not leak
raw frames.

## Progress

- [x] (2026-05-10T01:05Z) Created this ExecPlan and archived the completed
  quote-data source implementation plan.
- [x] (2026-05-10T01:15Z) Added ignored `live_quote_data_source`
  integration test with success and structured unavailable contract checks.
- [x] (2026-05-10T01:20Z) Updated development docs, roadmap, observation
  workflow docs, and changelog for the live smoke.
- [x] (2026-05-10T01:40Z) Ran focused validation and workspace baseline.
  Commit is being prepared.

## Surprises & Discoveries

- Pending.

## Decision Log

- Decision: Allow structured unavailable results by default in the live smoke.
  Rationale: quote-data depends on the selected TradingView page emitting a
  matching WebSocket frame during the bounded window. Outside active
  after-hours movement, no-frame can be expected and should validate the
  failure contract rather than fail the smoke.
  Date/Author: 2026-05-10 / Codex.

- Decision: Do not require scanner or visible panel agreement in this smoke.
  Rationale: this is a public contract smoke for `--source quote-data`, not a
  renewed source-correlation investigation. Scanner delay and UI display timing
  remain separate source concerns.
  Date/Author: 2026-05-10 / Codex.

## Outcomes & Retrospective

Added an opt-in ignored live contract smoke for `tv quote <SYMBOL> --source
quote-data`. The smoke validates success payloads when a matching `qsd.rtc`
readback arrives and validates structured `internal_api_unavailable` details
when the bounded window sees no matching frame. Unavailable is allowed by
default because the current work happened after the after-hours window and
frame availability is part of this source boundary.

The smoke does not require scanner agreement, does not paste raw frames, and
does not make phase matching an acceptance condition. Phase expectation remains
a public-safe reporting hint for future postmarket or premarket runs.

## Plan of Work

Add `crates/cli/tests/live_quote_data_source.rs` with one ignored test gated by
`TV_LIVE_QUOTE_DATA_SMOKE=1`. The test invokes the test-built `tv` binary with
`quote <SYMBOL> --source quote-data`, parses either stdout or stderr as the
JSON envelope, and validates only public-safe contract fields.

Success validation requires `source: "desktop_quote_data_ws"`,
`source_category: "desktop_backed_read"`, `requires_desktop: true`,
`non_mutating: true`, matching `requested_symbol`, a `quote_data.rtc` value,
and no scanner-style `extended_hours` injection.

Unavailable validation accepts `internal_api_unavailable` only when details
preserve the same source metadata and include `wait_summary.raw_frame_included:
false`. Connection failures, malformed JSON, or non-public summaries still
fail the smoke.

The smoke may be made stricter by setting `TV_LIVE_QUOTE_DATA_ALLOW_UNAVAILABLE=0`.
`TV_LIVE_QUOTE_DATA_EXPECT_PHASE` is a reporting hint only; it should print
the observed `market_phase` or `current_session` when present and should not
turn scanner equality into an acceptance criterion.

## Concrete Steps

Run from the repository root:

    cargo test -p tradingview-cli --test live_quote_data_source
    cargo test -p tradingview-cli market::quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture
    cargo test -p tradingview-cli --test live_after_hours_ws_correlation -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

Optional live smoke:

    TV_LIVE_QUOTE_DATA_SMOKE=1 cargo test -p tradingview-cli --test live_quote_data_source -- --ignored --nocapture

For RKLB:

    TV_LIVE_QUOTE_DATA_SMOKE=1 TV_LIVE_QUOTE_DATA_SYMBOL=NASDAQ:RKLB cargo test -p tradingview-cli --test live_quote_data_source -- --ignored --nocapture

Do not paste raw live output into tracked docs.

## Validation and Acceptance

Acceptance is met when the ignored smoke compiles in normal test runs, validates
success and structured unavailable contract fields when run explicitly, and
does not print raw WebSocket frames, raw payloads, target ids, local paths,
credentials, or account-local metadata.

## Idempotence and Recovery

The live smoke is read-only from the user's perspective. It invokes the same
bounded quote-data read command and does not switch symbols, mutate
quote-session fields, call scanner fallback, take screenshots, or extract DOM.

## Open Questions

Premarket evidence remains uncollected. A later run during a real premarket
window can reuse this smoke with `TV_LIVE_QUOTE_DATA_EXPECT_PHASE=premarket`.
