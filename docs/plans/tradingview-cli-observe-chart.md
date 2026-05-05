# Add `tv observe chart`

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is self-contained and describes the first `v0.7.0` implementation slice: adding `tv observe chart` as a workflow-level JSONL observation command.

## Purpose / Big Picture

After this change, a user or agent can run one bounded command to check whether the current TradingView Desktop chart is ready and then observe chart samples over a short window. Today the caller can run `tv readiness` and then choose a lower-level `tv stream ...` command, but it must manually stitch the two phases together. `tv observe chart` should make that common workflow explicit while keeping `tv stream ...` compatible.

The user-visible proof is a command like:

    tv observe chart --duration-ms 10000 --heartbeat-ms 2000

It should print newline-delimited JSON envelopes. The first envelope should describe readiness. Later envelopes should be sample, heartbeat, or error events. The command should end successfully when its bounded condition is reached.

## Progress

- [x] (2026-05-05T04:05Z) Created the `v0.7.0` roadmap and this initial implementation ExecPlan.
- [x] (2026-05-05T05:45Z) Implemented `tv observe chart` CLI surface.
- [x] (2026-05-05T05:45Z) Implemented JSONL event production using existing readiness and stream helpers where practical.
- [x] (2026-05-05T05:45Z) Added tests and docs for the new observe workflow.
- [x] (2026-05-05T06:10Z) Validated the implementation.
- [ ] Commit the implementation.

## Surprises & Discoveries

- The first implementation can reuse `StreamRequest`, `StreamDedupe`,
  `stream_sample`, and `stream_heartbeat` directly. Keeping the observe loop
  separate from `tv stream ...` avoided changing the lower-level stream
  command contract.

## Decision Log

- Decision: Make the first observe command `tv observe chart`.
  Rationale: The existing v0.6 pieces are strongest around Desktop chart readiness, selected-chart streams, and screenshot hints. Starting with chart observation exercises that work without adding account mutation.
  Date/Author: 2026-05-05 / Codex.

- Decision: Keep `tv stream ...` as the lower-level compatibility surface.
  Rationale: Existing callers may already consume stream subcommands. `observe chart` should combine readiness and stream-style events rather than replace or rename the existing commands.
  Date/Author: 2026-05-05 / Codex.

- Decision: Do not auto-capture screenshots from `observe chart`.
  Rationale: Screenshots write local files and should remain explicit. `observe chart` can include a `screenshot_hint` so agents know what to run next.
  Date/Author: 2026-05-05 / Codex.

## Outcomes & Retrospective

Implemented `tv observe chart` as a top-level workflow command. The command
emits a readiness JSONL envelope with `command: "observe"` and
`data._event: "readiness"`, then uses selected-chart bar stream samples and
heartbeats with the same bounded controls as `tv stream ...`.

The implementation intentionally does not switch symbols, activate tabs,
capture screenshots, or change account/page state. Existing `tv stream ...`
commands remain the lower-level compatibility surface.

## Context and Orientation

The `tv` binary is implemented in the `tradingview-cli` package under `crates/cli`. The CLI surface is defined in `crates/cli/src/cli.rs`; command dispatch lives under `crates/cli/src/app/dispatch.rs`; JSONL stream looping lives under `crates/cli/src/app/stream.rs`; readiness logic lives under `crates/cli/src/ops/readiness.rs`; lower-level stream sampling and heartbeat helpers live under `crates/cli/src/ops/stream.rs`.

In this repository, a JSON envelope is an object with `success`, `command`, and either `data` or `error`. JSONL means one JSON envelope per line. A Desktop-backed read is a read-only command that depends on TradingView Desktop and Chrome DevTools Protocol. A non-mutating command does not change TradingView chart, account, editor, Replay, Screener, alert, drawing, watchlist, or UI state.

`tv stream ...` already supports bounded controls such as `--duration-ms`, `--max-events`, and `--heartbeat-ms`. `tv readiness` already returns a structured readiness payload. `tv observe chart` should compose these ideas into one workflow-level command.

## Plan of Work

Add a new top-level `observe` command with a `chart` subcommand in `crates/cli/src/cli.rs`. The `chart` subcommand should accept `--duration-ms`, `--max-events`, `--heartbeat-ms`, and `--interval` options with the same validation semantics as `tv stream ...`: zero values are rejected before connecting, omitted bounded options mean an infinite observation unless the user provides a bound, and heartbeat events do not count toward `max-events`.

Add a small operation module for observe behavior, preferably `crates/cli/src/ops/observe.rs` or `crates/cli/src/ops/observe/chart.rs` if the implementation would otherwise become large. Reuse `readiness` code to produce the first JSONL event. Reuse stream request validation, sampling, heartbeat, and dedupe helpers when practical; if direct reuse would make the code awkward, extract shared helpers from `ops/stream.rs` without changing existing stream payloads.

The JSONL event contract should be additive and explicit:

- readiness event: `success: true`, `command: "observe"`, and `data._event: "readiness"`, with the readiness payload under `data.readiness` or an equivalently obvious field.
- sample event: `success: true`, `command: "observe"`, and `data._event: "sample"`, using Desktop-backed chart stream source metadata.
- heartbeat event: `success: true`, `command: "observe"`, and `data._event: "heartbeat"`, with elapsed time, sample count, and last sample timestamp.
- polling error: write a structured error envelope to stderr and continue until the bounded condition is reached, matching the existing stream behavior.

The first implementation should observe the current selected chart; it must not switch symbols, activate tabs, mutate account state, or capture screenshots. If readiness reports `ready: false`, still emit the readiness event and continue only when the lower-level stream setup can connect. If CDP connection itself fails before observation starts, use the same structured connection error behavior as other Desktop-backed commands.

Update docs and skills only after the command behavior exists. README should show one short example. `docs/command-source-taxonomy.md` should classify `observe chart` as a Desktop-backed read / JSONL observation workflow. `chart-analysis` and `market-data-interpretation` should tell agents to prefer `tv observe chart` when they need a bounded readiness-plus-stream window.

## Concrete Steps

From the repository root:

1. Inspect the current stream and readiness implementation:

       rg -n "StreamRequest|stream_sample|stream_heartbeat|readiness" crates/cli/src

2. Add the CLI surface and dispatch for `tv observe chart`.

3. Implement the observe runner and shared validation. Keep existing `tv stream ...` tests passing before adding observe-specific tests.

4. Add CLI contract tests for help text and validation behavior.

5. Add unit tests for event shape and bounded behavior.

6. Update docs and runtime skills.

7. Run validation:

       cargo test -p tradingview-cli observe -- --nocapture
       cargo test -p tradingview-cli stream -- --nocapture
       cargo test -p tradingview-cli readiness -- --nocapture
       cargo test -p tradingview-cli --test cli_contract observe -- --nocapture
       cargo fmt --check
       cargo clippy --workspace --all-targets --all-features -- -D warnings
       cargo test --workspace
       cargo metadata --no-deps --format-version 1
       git diff --check
       bash -n scripts/stage-release-package-files.sh

Optional live smoke, when TradingView Desktop is running with CDP enabled:

       target/debug/tv observe chart --duration-ms 3000 --heartbeat-ms 1000
       target/debug/tv observe chart --max-events 2 --interval 500

Do not paste live target ids, account-local metadata, raw payloads, or machine-local paths into tracked docs.

## Validation and Acceptance

Acceptance is reached when `tv observe chart --help` is available, invalid zero bounded options fail before connecting, the first successful observation line is a readiness event, sample and heartbeat events are newline-delimited JSON envelopes, `max-events` counts only sample events, `duration-ms` stops the command with exit code 0, and existing `tv stream ...` behavior remains compatible.

The implementation is also accepted when docs and skills explain `observe chart` as a Desktop-backed, non-mutating workflow command and all listed tests pass.

## Idempotence and Recovery

This change should be additive. If observe event shaping proves difficult, keep `tv stream ...` untouched and implement observe with a thin wrapper around existing public helpers first. If extracting shared helpers breaks stream tests, revert the extraction and duplicate the small amount of observe runner code temporarily; compatibility is more important than premature reuse.

## Artifacts and Notes

Implementation touched:

- `crates/cli/src/cli.rs`
- `crates/cli/src/app/runner.rs`
- `crates/cli/src/app/observe.rs`
- `crates/cli/src/ops/observe.rs`
- `crates/cli/tests/cli_contract.rs`

Initial focused validation:

    cargo test -p tradingview-cli observe -- --nocapture

Result: passed before final full validation.

Final validation:

    cargo test -p tradingview-cli observe -- --nocapture
    cargo test -p tradingview-cli stream -- --nocapture
    cargo test -p tradingview-cli readiness -- --nocapture
    cargo test -p tradingview-cli --test cli_contract observe -- --nocapture
    cargo test -p tradingview-cli --test cli_contract stream -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

Result: passed. Runtime skill validation passed for `chart-analysis`,
`market-data-interpretation`, and `multi-symbol-scan`. The changed-file hygiene
grep reported only existing source-boundary wording in `docs/v0.7-roadmap.md`.

## Interfaces and Dependencies

The final CLI surface should include:

    tv observe chart [--duration-ms <MS>] [--max-events <N>] [--heartbeat-ms <MS>] [--interval <MS>]

The command should be represented as a top-level command named `observe`, with a `chart` subcommand. It should use the existing TradingView Desktop / CDP transport and must not add new dependencies.

The JSONL events should use `command: "observe"` so callers can distinguish this workflow from lower-level `command: "stream"` lines.

## Open Questions

No open questions block the first implementation. If live usage shows that agents need multiple stream kinds in one observe command, add that as a later plan rather than expanding the first `observe chart` slice.
