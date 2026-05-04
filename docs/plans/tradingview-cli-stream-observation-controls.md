# Stream observation controls

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is self-contained so a new contributor can finish the stream observation controls without prior chat context.

## Purpose / Big Picture

Make existing Desktop-backed `tv stream ...` commands useful as bounded JSONL observation producers for agents. The command family already streams current Desktop chart/page state, but it is unbounded by default. This slice adds optional stop conditions and heartbeat events without changing existing no-option infinite stream behavior.

After this change, callers can run commands such as `tv stream quote --duration-ms 10000 --heartbeat-ms 2000` or `tv stream bars --max-events 5` and receive newline-delimited JSON envelopes until a bounded condition is met.

## Progress

- [x] (2026-05-05) Archived the completed `v0.5.1` release readiness ExecPlan.
- [x] (2026-05-05) Added this ExecPlan.
- [x] (2026-05-05) Added stream observation options and bounded runner state.
- [x] (2026-05-05) Added sample/heartbeat event metadata and metadata-insensitive dedupe.
- [x] (2026-05-05) Updated docs and runtime skills for bounded stream usage.
- [x] (2026-05-05) Ran validation.
- [x] (2026-05-05) Commit the related changes.

## Surprises & Discoveries

- Observation: stream dedupe compared samples after `_ts` metadata had been added.
  Evidence: `stream_sample` added `_ts` before `StreamDedupe::should_emit` was called in `run_stream_command`.

- Observation: the local skill validator script could not run in this environment because the active Python lacks `yaml`.
  Evidence: `quick_validate.py` failed with `ModuleNotFoundError: No module named 'yaml'`.

## Decision Log

- Decision: Keep no-option `tv stream ...` behavior as infinite.
  Rationale: existing users may rely on long-running streams; bounded observation is opt-in via `--duration-ms` and/or `--max-events`.
  Date/Author: 2026-05-05 / Codex

- Decision: Emit heartbeat as a normal stdout success envelope.
  Rationale: consumers already read JSONL from stdout; stderr should remain reserved for runtime error envelopes.
  Date/Author: 2026-05-05 / Codex

- Decision: Do not emit a final completion event.
  Rationale: exit code `0` is enough to signal a bounded stream ended normally, and avoiding a final schema keeps the additive event contract smaller.
  Date/Author: 2026-05-05 / Codex

## Outcomes & Retrospective

Implemented bounded observation controls for all existing `tv stream ...` subcommands. The no-option stream behavior remains infinite, while callers can now opt into `--duration-ms`, `--max-events`, and `--heartbeat-ms`.

Stream sample payloads now include `_event: "sample"`, and heartbeat payloads include `_event: "heartbeat"`, `_stream`, `_ts`, `elapsed_ms`, `sample_count`, and `last_sample_ts`. Dedupe now ignores `_ts` and `_event` so unchanged chart/page samples are not emitted solely because metadata changed.

Focused stream tests, CLI contract stream tests, formatting, clippy, workspace tests, metadata, package script syntax, and diff whitespace checks passed. Skill validation was attempted but blocked by the missing local Python `yaml` module; no Python dependency was added for this slice.

## Context and Orientation

`tv stream ...` is a Desktop-backed read surface. It uses CDP and selected TradingView Desktop page state. It is not a Desktop-free market-data command and does not replace scanner REST reads or lab-gated browserless bars.

Relevant files:

- `crates/cli/src/cli.rs` defines stream CLI options.
- `crates/cli/src/app/stream.rs` owns the JSONL loop.
- `crates/cli/src/ops/stream.rs` owns stream request validation, sampling, metadata, heartbeat payloads, and dedupe.

## Plan of Work

Add shared stream options to all stream subcommands: `--duration-ms`, `--max-events`, and `--heartbeat-ms`. Keep existing `--interval` and filter options. Validate zero values before connecting to CDP. Keep the minimum heartbeat interval equal to the existing minimum stream interval, 100ms.

Extend `StreamRequest` with optional bounded controls. In the stream loop, count only emitted sample events for `max_events`; heartbeat events do not consume the limit. If both duration and max events are specified, stop on whichever condition is reached first. During polling errors after startup, keep printing structured error envelopes to stderr and continue until the bounded condition is reached.

Add `_event: "sample"` to changed samples and emit heartbeat payloads with `_event: "heartbeat"`, `_stream`, `_ts`, `elapsed_ms`, `sample_count`, and `last_sample_ts`. Compare stream samples without `_ts` and `_event` so dedupe reflects real chart/page data changes rather than metadata changes.

## Validation and Acceptance

Run:

    cargo test -p tradingview-cli stream -- --nocapture
    cargo test -p tradingview-cli --test cli_contract stream -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Optional live smoke:

    target/debug/tv stream quote --duration-ms 3000 --heartbeat-ms 1000
    target/debug/tv stream bars --max-events 2 --interval 500

Acceptance is met when existing infinite stream behavior remains available by omitting the new options, bounded options validate before connecting, sample and heartbeat JSONL events are additive, tests pass, and docs/skills describe `stream` as Desktop-backed observation.

## Idempotence and Recovery

This slice is safe to rerun. If live smoke is attempted, it reads the selected Desktop chart and does not mutate account state. Do not record raw live payloads, target ids, account-local metadata, cookies, tokens, or local absolute paths in tracked docs.

## Interfaces and Dependencies

No new dependencies. No new top-level command. No JSON envelope shape changes. Stream payload additions are additive under `data`.

## Open Questions

None.
