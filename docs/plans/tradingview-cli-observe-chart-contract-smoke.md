# Observe chart JSONL contract and live smoke

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is self-contained and describes the next `v0.7.0` implementation slice after the first `tv observe chart` command.

## Purpose / Big Picture

`tv observe chart` now exists as a workflow-level JSONL command that emits readiness first and then selected-chart last-bar samples or heartbeats. The next step is not to add more observe modes immediately. First, confirm that the new workflow is operationally easy for agents to consume and that its event contract can be checked without relying on an always-on live TradingView Desktop session.

After this change, contributors should have an opt-in live smoke that can run a short `tv observe chart` window and assert the public-safe JSONL sequence:

    tv observe chart --duration-ms 3000 --heartbeat-ms 1000

The smoke should stay out of normal CI. It should help catch regressions in event ordering, command names, bounded exit behavior, and readiness/sample/heartbeat metadata when a live Desktop session is available.

## Progress

- [x] (2026-05-06T00:20Z) Created this ExecPlan and archived the completed first `tv observe chart` implementation plan.
- [x] (2026-05-06T00:45Z) Added opt-in ignored Rust integration smoke for `tv observe chart`.
- [x] (2026-05-06T00:45Z) Confirmed no helper extraction was needed; no additional focused contract tests were added.
- [x] (2026-05-06T00:45Z) Updated docs with the optional smoke workflow.
- [x] (2026-05-06T01:05Z) Validated the slice.
- [ ] Commit the slice.

## Surprises & Discoveries

- The smoke can validate the JSONL event sequence without changing the
  `observe` implementation. The existing event metadata is sufficient for
  public-safe assertions.

## Decision Log

- Decision: Add an opt-in ignored Rust smoke instead of a standalone script.
  Rationale: The repository already uses Rust integration tests for opt-in live chart quote smoke. Keeping this as Rust avoids new runtime dependencies and keeps command parsing close to the CLI contract tests.
  Date/Author: 2026-05-06 / Codex.

- Decision: Do not add new `observe` subcommands in this slice.
  Rationale: The first observe command needs live-use evidence before the surface expands. Additional modes such as quote, values, all, Screener, or browserless observation should be separate plans.
  Date/Author: 2026-05-06 / Codex.

- Decision: Do not add a final summary event yet.
  Rationale: `tv stream ...` deliberately uses process exit code for bounded completion. `observe chart` should stay compatible with that model until downstream usage shows a final event is useful.
  Date/Author: 2026-05-06 / Codex.

## Outcomes & Retrospective

Implemented `crates/cli/tests/live_observe_chart.rs` as an ignored,
environment-gated integration smoke. It runs `tv observe chart` with a short
bounded window by default, parses stdout/stderr as JSONL, verifies the first
readiness event and later sample/heartbeat metadata, and keeps failure output
to public-safe summaries.

No new CLI surface, dependencies, or runtime skill workflow expansion was
needed.

## Context and Orientation

`tv observe chart` is implemented under `crates/cli/src/app/observe.rs` and uses `ops::readiness`, `StreamKind::Bars`, `StreamRequest`, `StreamDedupe`, `stream_sample`, and `stream_heartbeat`. Its command surface is defined in `crates/cli/src/cli.rs` as:

    tv observe chart [--duration-ms <MS>] [--max-events <N>] [--heartbeat-ms <MS>] [--interval <MS>]

The command emits newline-delimited JSON envelopes with `command: "observe"`. The first event is readiness (`data._event: "readiness"`). Later stdout events are sample or heartbeat events. Polling errors are written to stderr as observe error envelopes and the bounded observation loop continues.

The existing `crates/cli/tests/live_chart_quote.rs` file is the model for an opt-in live integration smoke. It is ignored by default, requires an explicit environment gate, accepts optional target selection, and keeps panic output public-safe.

## Plan of Work

Add an ignored Rust integration test, likely `crates/cli/tests/live_observe_chart.rs`.

The test should:

- use `CARGO_BIN_EXE_tv` to run the test-built binary;
- require `TV_LIVE_OBSERVE_CHART_SMOKE=1` and fail with a clear message when the ignored test is explicitly run without the gate;
- accept optional `TV_LIVE_OBSERVE_CHART_TARGET_ID` and pass `--target-id <ID>` when present;
- accept optional `TV_LIVE_OBSERVE_CHART_DURATION_MS`, `TV_LIVE_OBSERVE_CHART_HEARTBEAT_MS`, and `TV_LIVE_OBSERVE_CHART_MAX_EVENTS`;
- default to a short bounded window such as `--duration-ms 3000 --heartbeat-ms 1000`;
- parse stdout as JSONL with `serde_json`;
- parse stderr JSONL only for public-safe error summaries if the process fails.

The public-safe assertions should be:

- process exits with code 0;
- stdout has at least one JSON object;
- first stdout line has `success: true`, `command: "observe"`, and `data._event: "readiness"`;
- every later stdout line has `command: "observe"`;
- sample events, when present, have `data._event: "sample"`, `data._stream: "bars"`, `source: "desktop_chart_stream"`, `source_category: "desktop_backed_read"`, `requires_desktop: true`, and `non_mutating: true`;
- heartbeat events, when present, have `data._event: "heartbeat"`, `sample_count`, `elapsed_ms`, and the same Desktop-backed source metadata;
- `max-events`, when set, counts sample events rather than heartbeat events.

Failure messages must not include raw JSON payloads, target ids, account-local metadata, local paths, or screenshots. Summaries may include event counts, event types, exit code, and public error kind/message.

If this smoke exposes an implementation bug, fix only the bug needed for `observe chart` contract correctness. Do not broaden observe modes, add `tv diagnose`, stabilize browserless streaming, or change `tv stream ...` behavior in this slice.

Update documentation after the test exists:

- `docs/development.md`: add the opt-in smoke invocation in the live smoke section or testing guidance;
- `docs/v0.7-roadmap.md`: record that the next `observe chart` step is contract smoke / evidence, not surface expansion;
- `docs/plans/README.md`: point current plan to this file and list the previous observe implementation plan as archived;
- runtime skills only if needed, keeping guidance short and focused on `tv observe chart` as a bounded workflow.

## Concrete Steps

From the repository root:

1. Inspect the existing live quote smoke:

       sed -n '1,240p' crates/cli/tests/live_chart_quote.rs

2. Add `crates/cli/tests/live_observe_chart.rs` using the same opt-in and public-safe style.

3. Add focused contract tests only if helper changes are needed. The current command behavior should otherwise remain unchanged.

4. Update docs and skills.

5. Run validation:

       cargo test -p tradingview-cli --test live_observe_chart
       cargo test -p tradingview-cli observe -- --nocapture
       cargo test -p tradingview-cli --test cli_contract observe -- --nocapture
       cargo fmt --check
       cargo clippy --workspace --all-targets --all-features -- -D warnings
       cargo test --workspace
       cargo metadata --no-deps --format-version 1
       git diff --check
       bash -n scripts/stage-release-package-files.sh

Optional live smoke, when TradingView Desktop is running with CDP enabled:

       TV_LIVE_OBSERVE_CHART_SMOKE=1 cargo test -p tradingview-cli --test live_observe_chart -- --ignored --nocapture

When multiple targets are open, first use `tv readiness` or `tv tab list`, then pass the chosen target through the test environment gate. Do not record live target ids in tracked docs.

## Validation and Acceptance

Acceptance is reached when the ignored live smoke is available, normal test runs skip it, and the explicit live invocation verifies the public JSONL sequence without leaking raw payloads.

The slice is also accepted when the full validation list passes and docs explain that this is an optional live smoke, not a CI guarantee and not a new command surface.

## Idempotence and Recovery

This change should be additive. If live smoke is flaky because of Desktop state, keep it ignored and opt-in; do not weaken normal CI. If a failure reveals a real contract issue, fix the contract issue and keep the smoke assertions narrow.

If implementation becomes larger than expected, stop at the ignored smoke and docs. Do not refactor the observe runner unless the test exposes a clear need.

## Artifacts and Notes

Implementation touched:

- `crates/cli/tests/live_observe_chart.rs`
- `docs/development.md`
- `docs/v0.7-roadmap.md`
- `CHANGELOG.md`

Validation evidence will be recorded after the final command pass.

Final validation:

    cargo test -p tradingview-cli --test live_observe_chart
    cargo test -p tradingview-cli observe -- --nocapture
    cargo test -p tradingview-cli --test cli_contract observe -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

Result: passed. The changed-file hygiene grep reported only existing policy,
validation-command, and source-boundary wording in durable docs.

## Interfaces and Dependencies

No new CLI command or option is planned. The new interface is an ignored Rust integration test controlled by environment variables:

    TV_LIVE_OBSERVE_CHART_SMOKE=1
    TV_LIVE_OBSERVE_CHART_TARGET_ID=<optional target id>
    TV_LIVE_OBSERVE_CHART_DURATION_MS=<optional milliseconds>
    TV_LIVE_OBSERVE_CHART_HEARTBEAT_MS=<optional milliseconds>
    TV_LIVE_OBSERVE_CHART_MAX_EVENTS=<optional sample event count>

No new dependencies should be added.

## Open Questions

- If downstream agents need a machine-readable final summary event, should `observe chart` add one in a future slice? This plan keeps that deferred.
- If live usage shows agents need quote and values in the same observe window, should the next command be `tv observe chart --include ...` or separate observe subcommands? This plan keeps that deferred.
