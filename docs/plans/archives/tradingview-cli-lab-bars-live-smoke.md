# Lab-gated `tv bars` live smoke

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is self-contained and describes the next `v0.7.0` implementation slice: adding an opt-in live smoke for the lab-gated browserless `tv bars` command.

## Purpose / Big Picture

`tv bars` already exists as a bounded, Desktop-free, experimental historical bars prototype behind `TV_EXPERIMENTAL_BARS=1`. The command uses an undocumented TradingView WebSocket path, so the next step is not to stabilize or broaden it. The next step is to make evidence collection repeatable.

After this change, contributors should have an ignored Rust integration smoke that can run a short set of exchange-qualified `tv bars` requests and verify the public-safe JSON contract without adding a normal CI dependency on TradingView's live WebSocket behavior.

## Progress

- [x] (2026-05-05T17:24Z) Created this ExecPlan and archived the completed `tv observe chart` contract smoke plan.
- [x] (2026-05-05T17:35Z) Added opt-in ignored Rust integration smoke for `tv bars`.
- [x] (2026-05-05T17:35Z) Updated docs with the optional smoke workflow.
- [x] (2026-05-05T17:45Z) Validated the slice.
- [x] (2026-05-05T17:48Z) Committed the slice.

## Surprises & Discoveries

- `tv bars` already returns the data-quality fields needed for a useful
  public-safe smoke. No CLI implementation change was needed.

## Decision Log

- Decision: Add an opt-in ignored Rust smoke instead of a standalone script.
  Rationale: The repository already uses ignored Rust integration tests for live chart quote and observe chart smokes. Keeping this in Rust avoids new runtime dependencies and lets the test use the test-built `tv` binary.
  Date/Author: 2026-05-05 / Codex.

- Decision: Keep `tv bars` lab-gated and do not change its CLI surface in this slice.
  Rationale: The WebSocket protocol is undocumented. This slice is evidence tooling, not stabilization.
  Date/Author: 2026-05-05 / Codex.

## Outcomes & Retrospective

Implemented `crates/cli/tests/live_bars.rs` as an ignored, environment-gated
integration smoke. It runs test-built `tv bars` commands with
`TV_EXPERIMENTAL_BARS=1`, validates only public contract fields, and keeps
failure output to short summaries.

No new CLI surface, dependencies, or `tv bars` behavior changes were needed.

## Context and Orientation

`tv bars <SYMBOL> --timeframe <TIMEFRAME> --count <N>` is implemented as a CLI-owned lab adapter. It requires `TV_EXPERIMENTAL_BARS=1`, rejects bare symbols, keeps `count` bounded to `1..=500`, and returns payloads with:

- `source: "experimental_tradingview_ws"`;
- `experimental: true`;
- `requested_symbol`, `symbol`, `timeframe`, `requested_count`, `bar_count`;
- `bars[]` with `time`, `open`, `high`, `low`, `close`, `volume`;
- `data_quality.realtime_guarantee`, `data_quality.entitlement_checked`, `data_quality.completed`, and `data_quality.elapsed_ms`;
- warnings that keep the feature clearly experimental.

Existing live smoke tests to mirror:

- `crates/cli/tests/live_chart_quote.rs`
- `crates/cli/tests/live_observe_chart.rs`

## Plan of Work

Add `crates/cli/tests/live_bars.rs` as an ignored integration test.

The test should:

- use `CARGO_BIN_EXE_tv` to run the test-built binary;
- require `TV_LIVE_BARS_SMOKE=1` and fail with a clear message when explicitly run without the gate;
- set `TV_EXPERIMENTAL_BARS=1` on the child command;
- accept optional `TV_LIVE_BARS_SYMBOLS`, defaulting to `NASDAQ:AAPL,NYSE:IONQ`;
- accept optional `TV_LIVE_BARS_TIMEFRAME`, defaulting to `1D`;
- accept optional `TV_LIVE_BARS_COUNT`, defaulting to `5`;
- accept optional `TV_LIVE_BARS_RUNS`, defaulting to `1`;
- run each requested symbol sequentially for each run;
- parse stdout or stderr as a single JSON envelope;
- keep failure messages to public-safe summaries.

The public-safe assertions should be:

- process exits with code 0;
- envelope has `success: true` and `command: "bars"`;
- `data.source == "experimental_tradingview_ws"`;
- `data.experimental == true`;
- `data.requested_symbol` and `data.symbol` equal the requested exchange-qualified symbol;
- `data.timeframe` equals the requested timeframe after command normalization;
- `data.requested_count` equals the requested count;
- `data.bar_count > 0`;
- `data.bars` is a non-empty array and each bar has numeric `time`, `open`, `high`, `low`, `close`, and `volume`;
- `data.data_quality.realtime_guarantee == false`;
- `data.data_quality.entitlement_checked == false`;
- `data.data_quality.completed` is present;
- `data.data_quality.elapsed_ms` is present.

Failure messages must not include raw JSON payloads, raw WebSocket frames, target ids, account-local metadata, local paths, or live response bodies. Summaries may include symbol, exit code, public error kind/message, `bar_count`, `completed`, and elapsed time.

Update documentation:

- `docs/development.md`: add the opt-in live bars smoke invocation near the other live smoke checks;
- `docs/v0.7-roadmap.md`: record that the next Browserless Bars step is opt-in evidence tooling, not stabilization;
- `docs/internal-tradingview-apis.md`: confirm the `tv bars` lab boundary and `data_quality` wording still match the implementation;
- `docs/plans/README.md`: point current plan to this file and list the observe chart smoke as archived;
- `CHANGELOG.md`: record a test/tooling improvement.

## Concrete Steps

From the repository root:

1. Inspect the existing live smoke tests:

       sed -n '1,260p' crates/cli/tests/live_observe_chart.rs
       sed -n '1,240p' crates/cli/tests/live_chart_quote.rs

2. Add `crates/cli/tests/live_bars.rs`.

3. Update the current plan index and roadmap.

4. Run validation:

       cargo test -p tradingview-cli --test live_bars
       cargo test -p tradingview-cli --test cli_contract bars -- --nocapture
       cargo test -p tradingview-cli market::bars -- --nocapture
       cargo fmt --check
       cargo clippy --workspace --all-targets --all-features -- -D warnings
       cargo test --workspace
       cargo metadata --no-deps --format-version 1
       git diff --check
       bash -n scripts/stage-release-package-files.sh

Optional live smoke:

       TV_LIVE_BARS_SMOKE=1 cargo test -p tradingview-cli --test live_bars -- --ignored --nocapture

With explicit parameters:

       TV_LIVE_BARS_SMOKE=1 \
       TV_LIVE_BARS_SYMBOLS=NASDAQ:AAPL,NYSE:IONQ \
       TV_LIVE_BARS_TIMEFRAME=1D \
       TV_LIVE_BARS_COUNT=5 \
       cargo test -p tradingview-cli --test live_bars -- --ignored --nocapture

Do not paste live output into tracked docs.

## Validation and Acceptance

Acceptance is reached when the ignored live smoke is available, normal test runs skip it, and explicit live invocation can validate the public `tv bars` JSON contract when TradingView's WebSocket path is available.

The slice is also accepted when the full validation list passes and docs explain that this is optional evidence tooling, not a CI guarantee and not stable browserless bars.

## Idempotence and Recovery

This change should be additive. If live smoke is flaky because of TradingView's undocumented WebSocket behavior, keep it ignored and opt-in; do not weaken normal CI. If a failure exposes a real `tv bars` contract issue, fix only the contract issue needed for public-safe behavior.

If implementation becomes larger than expected, stop at the ignored smoke and docs. Do not move `tv bars` into a reusable crate, add browserless streaming, or change the `tv ohlcv` boundary in this slice.

## Artifacts and Notes

Implementation touched:

- `crates/cli/tests/live_bars.rs`
- `docs/development.md`
- `docs/v0.7-roadmap.md`
- `docs/internal-tradingview-apis.md`
- `docs/plans/README.md`
- `CHANGELOG.md`

Final validation:

    cargo test -p tradingview-cli --test live_bars
    cargo test -p tradingview-cli --test cli_contract bars -- --nocapture
    cargo test -p tradingview-cli market::bars -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

Result: passed. The hygiene grep reported only existing policy, archived
validation-command, and source-boundary wording in durable docs.

## Interfaces and Dependencies

No new CLI command or option is planned. The new interface is an ignored Rust integration test controlled by environment variables:

    TV_LIVE_BARS_SMOKE=1
    TV_LIVE_BARS_SYMBOLS=<optional comma-separated exchange-qualified symbols>
    TV_LIVE_BARS_TIMEFRAME=<optional timeframe>
    TV_LIVE_BARS_COUNT=<optional count>
    TV_LIVE_BARS_RUNS=<optional positive repeat count>

No new dependencies should be added.

## Open Questions

- Should repeated live evidence eventually justify a stable browserless bars surface? This plan keeps that deferred.
- Should browserless bars move from the CLI package into a reusable crate? This plan keeps that deferred.
