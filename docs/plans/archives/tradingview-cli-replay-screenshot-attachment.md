# Attach chart screenshots to bounded Replay step logs

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while work proceeds.
Maintain it according to `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, a user running a bounded `tv replay log` can explicitly ask
for one chart screenshot after each successful Replay step. Each JSONL step
reports whether its screenshot succeeded and, on success, its deterministic
file path. The summary counts screenshot results independently from Replay step
failures. The command does not retry steps or captures, overwrite files, start
Replay, or become a general Replay export.

## Progress

- [x] (2026-07-19) Compared retained candidates and selected Replay screenshot
  attachment.
- [x] (2026-07-19) Created this ExecPlan and synchronized durable state.
- [x] (2026-07-19) Completed focused plan verification against current CLI,
  Replay JSONL, screenshot, and test ownership with no blocker.
- [x] (2026-07-19) Implemented controls, deterministic path preflight,
  no-overwrite capture, independent attachment envelopes/counters, and tests.
- [x] (2026-07-19) Completed focused tests, strict Clippy, full workspace
  validation, metadata, hygiene, package syntax, guide parity, and diff checks.
- [x] (2026-07-20) Focused implementation review was green and recommended one
  owner-approved two-step live smoke before archive.
- [x] (2026-07-20) Ran the approved smoke once. Both steps and both PNG
  attachments succeeded, file sizes matched JSONL, and Replay was stopped.
- [x] (2026-07-20) Corrected an existing `replay_left_running` false report
  exposed by the smoke by returning the post-step started boolean from the
  existing Replay expression.
- [x] (2026-07-20) Completed focused correction/evidence review with no
  finding. No additional live run was required, and this plan is archived.

## Milestones

### Milestone: freeze the artifact contract

Add CLI validation and pure path planning before Desktop connection or Replay
mutation. Completion requires proving the exact files the run may create,
rejecting existing destinations, and preserving behavior when the option is off.

### Milestone: attach without changing Replay failure semantics

Capture once only after a successful Replay step. A capture or write failure is
an attachment error and the loop continues; a Replay step failure still stops
the log. Neither boundary retries.

### Milestone: prove the workflow

Add deterministic tests for validation, names, no-overwrite, capture and write
failures, partial-file cleanup, broken stdout, and composition with OHLCV
attachments. Update public and packaged guidance and run the full baseline.

## Surprises & Discoveries

- Observation: Replay log already has a per-step attachment position and
  aggregate counters, but its control type is OHLCV-specific.
  Evidence: `crates/cli/src/app/replay_log.rs` inserts an OHLCV result under
  `attachments` after a successful step and does not turn its failure into a
  `ReplayLogEndReason::StepFailed`.

- Observation: standalone chart screenshot writing permits replacement.
  Evidence: `crates/cli/src/ops/screenshot.rs::write_screenshot` uses
  `fs::write`; Replay needs a separate no-overwrite policy, not a global change.

## Decision Log

- Decision: add `--attach-chart-screenshot` and require
  `--screenshot-output-dir <DIR>` with it.
  Rationale: local side effects must be explicit; the directory option alone is
  also a validation error.
  Date/Author: 2026-07-19 / Codex

- Decision: name files `replay-step-0001.png`, widening naturally above four
  digits.
  Rationale: step index is stable and exposes no target or chart identity.
  Date/Author: 2026-07-19 / Codex

- Decision: preflight every planned path and use create-new writes.
  Rationale: early validation plus atomic no-clobber semantics closes the race;
  this slice has no overwrite option.
  Date/Author: 2026-07-19 / Codex

- Decision: screenshot failure never becomes Replay step failure.
  Rationale: the remote step already succeeded, so retry or reclassification
  would corrupt chronology.
  Date/Author: 2026-07-19 / Codex

## Outcomes & Retrospective

Implementation is complete. Replay log now validates and creates its artifact
directory before Desktop connection, attempts one no-overwrite chart capture
after each successful step, reports screenshot results independently from OHLCV
attachments and Replay failures, and preserves standalone screenshot behavior.
Focused implementation review completed without a blocker and required the
bounded smoke described below before archive.

Focused implementation review was green. The separately approved smoke started
Replay on one explicit target, logged exactly two steps, and wrote exactly two
chart PNG attachments. Both attachment objects reported `status: "ok"`, each
file was a valid PNG, and each on-disk size matched `size_bytes`. Summary counts
were requested 2, ok 2, error 0, with two successful steps and no Replay
failure. Replay was then stopped and read back as stopped.

The smoke also exposed a pre-existing summary defect: `replay_left_running` was
false even though Replay remained active until the explicit stop. The step
expression verified `isReplayStarted()` before mutation but did not include its
post-step boolean in the normalized payload, so the summary treated null as
false. The correction returns that boolean from the same expression without an
extra evaluation. Focused correction/evidence review traced the value through
the model normalizer and summary, confirmed the smoke evidence, and found no
remaining issue. A repeat live run would not establish an additional contract
and was not performed.

One artifact-consumption caveat remains intentional: a step screenshot is
written before its JSONL event is emitted. If the consumer disconnects during
that emit, the PNG for that step may remain and the PNG count may exceed the
emitted step-event count by one. No later step or capture runs after the broken
pipe.

## Context and Orientation

`crates/cli/src/cli.rs::ReplayCommand::Log` defines options.
`crates/cli/src/app/runner.rs` forwards them to
`crates/cli/src/app/replay_log.rs::run_replay_log_command`, which opens one
runtime, verifies Replay, advances a bounded number of steps, emits step JSONL,
and emits a summary. Its optional OHLCV attachment runs after step success and
serializes attachment failure without stopping the loop.

`crates/cli/src/ops/screenshot.rs::screenshot_chart` owns chart-bound lookup,
clipped capture, full-page crop fallback, metadata, and writing. Standalone
screenshot overwrite behavior is out of scope and must remain unchanged.

A planned destination is the deterministic path for one requested step index.
An attachment failure means the step succeeded but capture or persistence did
not. An unknown step outcome retains existing behavior and performs no capture.

## Plan of Work

Add `attach_chart_screenshot: bool` and `screenshot_output_dir: Option<PathBuf>`
to `ReplayCommand::Log` and forward them through `app/runner.rs`. Help must state
the opt-in, deterministic names, no-overwrite rule, and independent failures.

Generalize Replay attachment controls and counters without changing the OHLCV
shape. Before `connect_runtime`, require the directory exactly when attachment
is enabled, compute paths for `1..=steps`, reject duplicates and existing paths,
and create the directory. A filesystem setup error must precede Replay mutation.

After each successful step, run requested OHLCV and chart screenshot attachments
independently. Store them as `attachments.ohlcv_summary` and
`attachments.chart_screenshot`. The screenshot object has a dedicated contract
version, `status: "ok" | "error"`, and `step_index`. Success includes
`output_path`, `region`, `capture_mode`, and `size_bytes`. Failure exposes only
error kind, a fixed message, and an allowlisted `bounds`, `capture`, or `write`
phase, never raw CDP/OS data or target identity.

Add screenshot requested, ok, and error summary counts. Ok plus error equals
requested, and requested never exceeds successful steps. Screenshot errors do
not increment Replay `failure_count` or change `end_reason`.

Add a crate-private Replay capture helper in `ops/screenshot.rs` that accepts a
`Path`, reuses bounds/capture fallback, and writes with
`OpenOptions::create_new(true)`. If writing fails after creating the file,
remove only that incomplete file. Never delete a pre-existing path. Preserve
standalone screenshot behavior.

Update CLI contracts, `docs/development.md`, `packaging/agent/AGENTS.md`, and
the source and packaged `replay-practice` skill. State that screenshots are
local artifacts, Replay remains running, and attachment failure is not step
failure.

## Concrete Steps

Run from the repository root:

    cargo test -p tradingview-cli app::replay_log -- --nocapture
    cargo test -p tradingview-cli ops::screenshot -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop replay -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    git diff --check

Every focused filter must execute at least one test. A live Replay smoke is not
required; it may run only after implementation review and explicit owner
approval because it advances Replay and writes files.

## Validation and Acceptance

Without new flags, `tv replay log --steps 2` keeps its old JSONL and creates no
files. Either screenshot option alone fails before Desktop connection. Existing
planned files remain unchanged and cause pre-dispatch validation failure.

With both options, each successful step attempts one deterministic capture. A
success attachment names an existing file with matching byte count. Capture or
write failure produces a sanitized error attachment and the next step proceeds.
A step failure captures nothing for that index and keeps `step_failed`. OHLCV
and screenshot attachments work together with independent counters.

Tests prove no overwrite, no retry, no capture before step success, cleanup only
of a partial file created by this attempt, broken-pipe termination without extra
steps/captures, filename widening, and no private values in diagnostics.

## Idempotence and Recovery

Tests use temporary directories. A real run is deliberately not repeatable in
the same directory because existing planned files are rejected. Use a new empty
directory. Keep completed JSONL and screenshots after interruption; an absent
screenshot does not prove its Replay step did not occur.

## Artifacts and Notes

Selection evidence is in
`docs/notes/v0.30-retained-backlog-product-selection.md`. Never record live
target IDs, chart titles, symbols, or machine paths in tracked files.

## Interfaces and Dependencies

Use the standard library and existing dependencies only. Add no production
dependency, generic export framework, or common-envelope field. The final CLI is:

    tv replay log --steps <N> --attach-chart-screenshot \
      --screenshot-output-dir <DIR>

Existing `--attach-ohlcv-summary [--ohlcv-count <N>]` remains compatible.

## Open Questions

There are no unresolved implementation blockers. Any live smoke requires
separate owner approval after focused implementation review.

Revision note (2026-07-19): initial plan created from the retained-backlog
comparison. Other candidates remain unpromoted.

Revision note (2026-07-19): implemented the reviewed contract with no new
dependency. Full non-live validation is green; focused implementation review is
the current gate and no live Replay mutation is authorized.

Revision note (2026-07-20): focused implementation review was green. One
owner-approved two-step smoke verified two attachment PNGs and successful
cleanup of Replay state. It also exposed and prompted a narrow correction to
the existing `replay_left_running` summary input. Focused correction/evidence
review is pending; no rerun is authorized.
