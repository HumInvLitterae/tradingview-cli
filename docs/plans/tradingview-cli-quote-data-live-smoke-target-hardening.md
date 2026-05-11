# Quote-data live smoke target hardening

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is
self-contained and describes how to harden the opt-in `quote-data` live smoke
after the `v0.14.0` release without changing public CLI behavior or JSON
payloads.

## Purpose / Big Picture

`tv quote <SYMBOL> --source quote-data` is an explicit Desktop-backed
WebSocket quote-data readback source. During the first premarket validation
after `v0.14.0`, the command itself could be checked with `--target-id`, but
the ignored live smoke could not pass a target id and therefore failed with
`target_ambiguous` when multiple chart targets were open.

After this change, a user can run the ignored smoke against the intended
TradingView chart target by setting `TV_LIVE_QUOTE_DATA_TARGET_ID`. The smoke
still validates only public-safe contract fields, and it still treats
structured unavailable quote-data as source unavailability rather than as
evidence that a market price does not exist.

## Progress

- [x] (2026-05-11T10:52Z) Confirmed current test shape and development docs.
- [x] (2026-05-11T10:55Z) Added optional target-id support to the ignored
  `live_quote_data_source` smoke.
- [x] (2026-05-11T11:00Z) Update durable docs and changelog for the
  test-only target selection.
- [x] (2026-05-11T11:05Z) Run focused validation.
- [x] (2026-05-11T11:10Z) Run opt-in premarket smoke with an explicit target id while the market
  phase is available.
- [x] (2026-05-11T11:25Z) Run full workspace validation.
- [ ] Commit the related changes in one local commit.

## Surprises & Discoveries

- Observation: The released `v0.14.0` command contract worked with explicit
  `--target-id`, but the ignored live smoke had no target-id environment
  variable.
  Evidence: A premarket smoke run failed with `target_ambiguous` before it
  could validate `quote_data.v1` details, while the manual command with
  `--target-id` returned structured `quote_data.v1` unavailable details.

- Observation: A bounded RKLB premarket run saw WebSocket and qsd activity but
  no matching requested-symbol `rtc`.
  Evidence: The public-safe wait summary included nonzero WebSocket and qsd
  counts, nonzero `qsd_with_rtc_seen`, and matching-symbol qsd without `rtc`,
  with `unavailable_reason: "no_rtc"`.

- Observation: After adding target selection to the smoke, the opt-in
  premarket run no longer failed with `target_ambiguous`.
  Evidence: The smoke printed `target_id=<provided>`, returned
  `success=false` with `contract=quote_data.v1`,
  `source=desktop_quote_data_ws`, `availability=unavailable`, and passed the
  ignored test. In that run the unavailable reason was
  `no_websocket_events`, which is source-availability evidence rather than a
  missing market-price claim.

## Decision Log

- Decision: Add `TV_LIVE_QUOTE_DATA_TARGET_ID` as a test-only environment
  variable rather than changing CLI source behavior.
  Rationale: The public CLI already supports `--target-id`; the bug is only
  that the ignored smoke did not expose that existing knob.
  Date/Author: 2026-05-11 / Codex.

- Decision: Keep unavailable quote-data acceptable by default in the ignored
  smoke.
  Rationale: A bounded no-`rtc` result is useful source-availability evidence
  when it returns `quote_data.v1`, `source_availability`, and public-safe
  wait-summary counters.
  Date/Author: 2026-05-11 / Codex.

## Outcomes & Retrospective

The ignored quote-data smoke now supports explicit chart target selection
through `TV_LIVE_QUOTE_DATA_TARGET_ID` without printing the raw target id. The
premarket smoke can run in a multi-target Desktop session and validate
structured unavailable quote-data as public-safe source diagnostics.

The premarket evidence collected in this slice did not produce a successful
`qsd.rtc` read for `NASDAQ:RKLB`. That does not invalidate the v0.14 contract:
it confirms that unavailable quote-data is distinguishable from scanner
premarket data and chart main-series quote data. Premarket `rtc` success
evidence remains open.

Validation passed with focused `live_quote_data_source`, quote-data unit tests,
quote CLI contract tests, formatting, clippy, full workspace tests, metadata,
diff check, packaging script syntax check, and a hygiene scan that reported
only existing policy language, test fixtures, validation-command examples, and
this plan's public-safe safety wording.

## Context and Orientation

The ignored smoke lives in `crates/cli/tests/live_quote_data_source.rs`. It
executes the test-built `tv` binary through `CARGO_BIN_EXE_tv` and validates
the JSON envelope from `tv quote <SYMBOL> --source quote-data`.

The public command already accepts a global `--target-id <CDP_TARGET_ID>`
option before the subcommand. CDP means Chrome DevTools Protocol, the local
debug interface used to read TradingView Desktop pages. Multiple TradingView
chart pages can be open at the same time, so Desktop-backed commands sometimes
need an explicit target id to avoid ambiguity.

This plan does not change the public payload. The source remains
`desktop_quote_data_ws`, the command-local contract marker remains
`quote_data.v1`, and `quote-data` remains separate from scanner
`extended_hours`, chart main-series quote, and `--source auto`.

## Plan of Work

Update `crates/cli/tests/live_quote_data_source.rs` so the smoke reads
`TV_LIVE_QUOTE_DATA_TARGET_ID`. When present, the test must invoke the binary
as `tv --target-id <ID> quote <SYMBOL> --source quote-data`. When absent, the
test must keep the current invocation so existing single-target environments
continue to work.

The smoke output must not print the raw target id. It should print only whether
a target id was provided or automatic target selection is being used.

Update `docs/development.md` to document the new environment variable and show
a multiple-target premarket example with `<ID>` placeholder text. Update
`docs/plans/README.md`, `docs/v0.14-roadmap.md`, and `CHANGELOG.md` so the
current project state reflects this post-release smoke hardening.

## Concrete Steps

From the repository root, run:

    cargo test -p tradingview-cli --test live_quote_data_source
    cargo test -p tradingview-cli market::quote_data -- --nocapture
    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

For opt-in premarket evidence, run only during the relevant market phase:

    TV_LIVE_QUOTE_DATA_SMOKE=1 \
      TV_LIVE_QUOTE_DATA_TARGET_ID=<ID> \
      TV_LIVE_QUOTE_DATA_EXPECT_PHASE=premarket \
      TV_LIVE_QUOTE_DATA_SYMBOL=NASDAQ:RKLB \
      TV_LIVE_QUOTE_DATA_ALLOW_UNAVAILABLE=1 \
      cargo test -p tradingview-cli --test live_quote_data_source -- --ignored --nocapture

The output should report `target_id=<provided>` rather than the raw target id.
Success should validate `contract_version: "quote_data.v1"` and
`source_availability.available: true`. Unavailable should validate
`contract_version: "quote_data.v1"`,
`source_availability.available: false`, a machine-readable
`unavailable_reason`, and public-safe wait-summary counters.

## Validation and Acceptance

Acceptance is reached when the ignored smoke can be run with an explicit target
id in a multi-target Desktop session without failing at `target_ambiguous`.
The smoke may still return structured unavailable quote-data; that is
acceptable when it includes `quote_data.v1` details and no raw WebSocket frame
or raw live payload.

The focused test `quote_data_args_include_optional_target_before_command`
should prove that the target id, when supplied, is placed before the `quote`
subcommand. Existing quote-data unit tests and CLI contract tests must still
pass.

## Idempotence and Recovery

The change is test-only and docs-only. Re-running the ignored smoke is safe and
non-mutating. If the smoke returns unavailable, retrying later may produce
different source-availability counts because TradingView WebSocket frames are
time-dependent.

If multiple chart targets remain open and the smoke still reports
`target_ambiguous`, verify that `TV_LIVE_QUOTE_DATA_TARGET_ID` is set in the
same command invocation and that the target id comes from `tv tab list`.
Do not paste the target id into tracked docs.

## Interfaces and Dependencies

No new public CLI option, source, JSON field, dependency, or version bump is
introduced. The only new interface is the ignored-test environment variable
`TV_LIVE_QUOTE_DATA_TARGET_ID`.

## Open Questions

None. Premarket `qsd.rtc` success evidence remains dependent on TradingView
Desktop actually emitting a matching `rtc` frame during the bounded wait.
