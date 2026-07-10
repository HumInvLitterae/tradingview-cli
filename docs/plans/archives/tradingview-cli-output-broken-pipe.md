# Make JSON and JSONL output safe when consumers close the pipe

This ExecPlan is a living document. The sections `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to
date as work proceeds. Maintain this document in accordance with
`.agents/PLANS.md`.

## Purpose / Big Picture

After this change, users and agents can pipe `tv` JSON or JSONL into tools such
as `head` without causing a Rust panic when the consumer exits first. A closed
stdout consumer should end successful one-shot or streaming output quietly,
without a panic message, backtrace, duplicate error envelope, or continued
network polling. Normal JSON bytes, event contracts, and command exit codes
must remain unchanged.

Before this slice, `crates/cli/src/app/output.rs` used `println!` and
`eprintln!`. Rust's standard printing macros panic when the underlying write
fails. The failure was observable with a bounded Desktop-free JSONL command
whose first readiness line was consumed and whose later output was written
after the reader had closed.

## Progress

- [x] (2026-07-10) Reproduced a JSONL stdout broken pipe as exit code 101 with a Rust panic.
- [x] (2026-07-10) Created this self-contained ExecPlan and made it the active `v0.26.0` plan.
- [x] (2026-07-10) Resolved plan review findings by fixing stderr behavior and the deterministic no-more-polls test seam before implementation.
- [x] (2026-07-10) Added writer-level tests for unchanged pretty JSON and compact JSONL bytes, broken pipes, other I/O failures, and serialization failures.
- [x] (2026-07-10) Replaced JSON and JSONL output macros with locked explicit writers and private output result types.
- [x] (2026-07-10) Propagated closed-stdout completion and fixed stderr policies through one-shot and all four JSONL runners.
- [x] (2026-07-10) Added a writer-injected watch runner test proving that no poll occurs after a stdout broken pipe is detected.
- [x] (2026-07-10) Added local subprocess coverage for closed one-shot stdout and unavailable terminal stderr.
- [x] (2026-07-10) Ran focused tests, the workspace baseline, and a public-safe `watch compare | head -1` smoke.
- [x] (2026-07-10) Updated this plan, the changelog, roadmap status, work inventory, and local continuity ledger after implementation.
- [x] (2026-07-10) Completed independent read-only review with no findings and closed Gate 1.

## Surprises & Discoveries

- Observation: the problem is not theoretical; a bounded `tv watch compare`
  invocation piped to `head -1` emitted its readiness event and then exited
  with code 101 when later output hit the closed pipe.
  Evidence: local reproduction on 2026-07-10 showed the standard Rust
  `failed printing to stdout: Broken pipe` panic. Raw JSONL output was not
  copied into tracked files.

- Observation: a one-shot subprocess test needs output substantially larger
  than the pipe buffer to prove that the child observes the closed reader
  rather than finishing before it closes.
  Evidence: the local regression test sends synthetic Pine source that creates
  many deterministic diagnostics, reads only the opening pretty-JSON line,
  closes the reader, and requires a prompt successful exit without panic text.

- Observation: terminal stderr behavior can be tested quickly and
  deterministically on Unix without waiting for a CDP connection failure.
  Evidence: the regression test gives the child the write end of a local
  stream whose reader is already closed, then confirms that a validation error
  retains exit code 1.

- Observation: the real Desktop-free watch pipeline now exits successfully as
  soon as `head` closes the reader.
  Evidence: the public-safe smoke completed in under one second with exit code
  0, no stderr bytes, and no panic or broken-pipe marker. Raw JSONL was not
  retained.

## Decision Log

- Decision: treat stdout `BrokenPipe` as normal consumer completion and return
  success for a command that was otherwise succeeding.
  Rationale: Unix pipeline consumers commonly stop after receiving enough
  data. A closed consumer is not an application or TradingView failure.
  Date/Author: 2026-07-10 / Codex.

- Decision: do not restore Unix `SIGPIPE` default behavior as the primary fix.
  Rationale: signal termination is platform-specific and commonly appears as
  a nonzero pipeline status. Explicit `std::io::Write` handling supports Unix,
  macOS, and Windows and lets JSONL loops stop before more network work.
  Date/Author: 2026-07-10 / Codex.

- Decision: preserve an already-determined command failure if stderr closes.
  Rationale: inability to deliver the error envelope must not turn a genuine
  validation, connection, source, or timeout failure into success. It must
  also not trigger a panic or recursive attempt to report an output error.
  Date/Author: 2026-07-10 / Codex.

- Decision: keep serialization failures distinct from broken pipes.
  Rationale: a serialization failure is an internal application defect;
  `BrokenPipe` is expected consumer behavior. Other write failures should be
  reported as internal output failures when a usable stderr remains.
  Date/Author: 2026-07-10 / Codex.

- Decision: use one fixed stderr failure policy rather than deciding it during
  implementation.
  Rationale: terminal error output and nonterminal JSONL error output have
  different workflow roles, but both must avoid panic and recursive reporting.
  The required behavior is:

  - stdout `BrokenPipe` while writing successful output ends the command
    successfully;
  - any other stdout write or serialization failure attempts one internal
    output-error envelope on stderr and exits with code 1; if that stderr write
    also fails, it exits with code 1 without another attempt;
  - failure to write a terminal error envelope to stderr preserves the
    original command error's exit code, regardless of whether stderr returned
    `BrokenPipe` or another I/O error;
  - stderr `BrokenPipe` while writing a nonterminal JSONL runtime-error event
    suppresses that event and lets the stdout workflow continue;
  - any other stderr write or serialization failure during a nonterminal JSONL
    runtime-error event stops the workflow with exit code 1 and is not reported
    to stderr again.

  Date/Author: 2026-07-10 / Codex.

- Decision: serialize each object before writing it and hold one locked stdout
  or stderr writer for each production JSONL workflow.
  Rationale: this keeps serialization failures distinct from I/O failures,
  preserves exact existing formatting, avoids repeated standard-stream lock
  acquisition, and still allows injected writers in deterministic tests.
  Date/Author: 2026-07-10 / Codex.

- Decision: keep the JSONL stderr-output runner error as a marker without
  retaining or re-reporting the underlying output error.
  Rationale: after a non-broken stderr output failure, the required behavior is
  unconditionally exit 1 with no recursive stderr attempt. Retaining the error
  would add no observable behavior and triggered an otherwise unused-field
  warning under the release clippy policy.
  Date/Author: 2026-07-10 / Codex.

## Outcomes & Retrospective

Implementation, local validation, and independent read-only review are
complete. The review reported no findings. Pretty one-shot JSON and compact JSONL retain their previous bytes
and trailing newline when writes succeed. A closed stdout now ends successful
one-shot and JSONL output with exit code 0. JSONL runners return immediately,
so they do not emit a summary, an extra error, or another poll after detecting
the closed consumer. Terminal stderr failure preserves the command's existing
exit code, while nonterminal JSONL stderr `BrokenPipe` suppresses only that
runtime-error line.

Focused tests, CLI contract tests, clippy with warnings denied, the full
workspace test suite, metadata, formatting, diff checks, package-script syntax,
and the public-safe watch pipeline smoke are green. The subprocess test for a
terminal stderr with no reader is Unix-only because it uses a local Unix
stream; the cross-platform writer-level policy remains covered through
`std::io::Write` test doubles. Clap's plain-text help and version output remain
intentionally outside this JSON/JSONL slice. Gate 1 is complete; the next work
item may now be planned separately without mixing it into this commit.

## Context and Orientation

The binary application layer lives in `crates/cli/src/app/`. The runner in
`crates/cli/src/app/runner.rs` parses the command, dispatches one-shot commands,
and selects special JSONL runners. `crates/cli/src/app/output.rs` serializes and
prints both pretty one-shot JSON and compact newline-delimited JSON. A JSONL
stream is a sequence of complete JSON objects, one per line.

The JSONL command loops live in:

- `crates/cli/src/app/stream.rs` for lower-level selected-chart streams;
- `crates/cli/src/app/observe.rs` for bounded selected-chart observation;
- `crates/cli/src/app/watch.rs` for Desktop-free watch comparison;
- `crates/cli/src/app/replay_log.rs` for bounded Replay step logs.

All four now use the private `JsonlOutput` writer and inspect
`OutputDisposition` for each stdout event; stream, observe, and watch also use
the shared nonterminal stderr policy for runtime errors. The one-shot
dispatcher calls result-returning `print_json_stdout` or `print_json_stderr`,
and `startup_error` uses the same stderr writer while preserving exit code 1.

No JSON envelope or JSONL event field should change. This plan changes only
how serialized bytes reach stdout or stderr and how the application reacts
when that write cannot complete.

## Plan of Work

First, separate serialization from writing in
`crates/cli/src/app/output.rs`. Introduce a small output result that can
distinguish a completed write, `BrokenPipe`, serialization failure, and other
I/O failure. Keep pretty formatting for one-shot JSON and compact formatting
for JSONL. Write through locked stdout or stderr with `writeln!` or
`write_all`, then flush only if required by existing behavior. Do not add a
new public crate dependency.

Make the byte-writing helper accept an injected `std::io::Write`. This allows
unit tests to use a `Vec<u8>` for exact output and a custom writer that returns
`std::io::ErrorKind::BrokenPipe` deterministically. Test pretty JSON, compact
JSONL, a broken pipe, and a non-broken write error. The production stdout and
stderr wrappers should be thin calls into that tested helper.

Next, update `crates/cli/src/app/runner.rs`. A one-shot success whose stdout
consumer closes returns `ExitCode::SUCCESS`. A serialization or non-broken
stdout error is reported once to stderr as an internal output error and exits
with code 1. If that stderr write also fails, return code 1 without another
write attempt. When writing a normal command error envelope, preserve the
original error's exit code if stderr returns `BrokenPipe`, another I/O error,
or a serialization error.

Update the four JSONL runners so every stdout event write observes the output
result. When stdout reports `BrokenPipe`, return `Ok(())` immediately. Do not
poll TradingView again and do not attempt to emit the final summary or an
error envelope to the closed stdout. Propagate serialization and other I/O
failures to the application runner once. A nonterminal JSONL error event whose
stderr returns `BrokenPipe` is suppressed and the stdout workflow continues.
Any other stderr write or serialization failure stops that workflow with exit
code 1 and must be carried to the runner as an output-origin failure that the
runner does not try to report to stderr again.

Use a private JSONL runner error that distinguishes application failure,
stdout output failure, and stderr output failure. The top-level runner may
attempt one terminal error envelope for an application failure or a stdout
output failure. It must not attempt another write for a stderr output failure.
This distinction prevents recursive output errors while preserving existing
application exit codes.

Add a deterministic no-more-polls test in
`crates/cli/src/app/watch.rs`. Extract a private writer- and poller-injected
variant of the watch compare loop. The production wrapper supplies locked
stdout/stderr and the existing `watch_sample` poller. The test supplies a fake
poller that increments a counter and returns an emit-worthy sample, plus a
writer that accepts the readiness line and returns `BrokenPipe` on the sample
line. Run with a long duration and assert that the runner returns success and
the poll counter is exactly one. This proves that the loop does not perform a
second poll after detecting the closed stdout without relying on the public
TradingView endpoint or duration expiry.

Add a subprocess regression test under the existing CLI integration-test
layout. Prefer a command that emits a readiness line before doing further
work. The test should start `tv` with a duration long enough that normal
duration expiry cannot explain prompt termination, read one stdout line, drop
the pipe reader, and require the child to exit within a short test deadline.
Assert that stderr does not contain `panicked`, `Broken pipe`, or a backtrace
marker. Keep platform-specific pipe handling behind an appropriate target
guard if the standard library behavior differs on Windows. This subprocess
test is supporting evidence; the writer-injected watch test is the
deterministic proof that polling stops.

Finally, update `CHANGELOG.md` under `Unreleased`. Before independent review,
record Gate 1 as implemented with review pending in `docs/v0.26-roadmap.md` and
`docs/v0.26-work-items.md`. After the review and any resulting fixes are
complete, mark Gate 1 complete. Only then may the CDP event-buffering item be
promoted into a fresh ExecPlan.

## Concrete Steps

Work from the repository root.

Before editing implementation files, confirm the current state:

    git status --short
    sed -n '1,180p' crates/cli/src/app/output.rs
    rg -n "print_jsonl_stdout|print_jsonl_stderr|print_json_stdout|print_json_stderr" crates/cli/src

Run or retain a concise before-fix reproduction. Do not place the raw JSONL
payload in tracked documentation:

    set -o pipefail
    target/debug/tv watch compare NASDAQ:AAPL NASDAQ:MSFT \
      --duration-ms 30000 --interval 1000 --heartbeat-ms 1000 | head -1

Before the fix, the producer is expected to panic after the reader closes.
After the fix, it should terminate without panic. Because this smoke reaches a
public TradingView endpoint, deterministic writer and subprocess tests are
the acceptance gate if network access is unavailable.

Run focused tests after editing:

    cargo test -p tradingview-cli app::output -- --nocapture
    cargo test -p tradingview-cli watch -- --nocapture
    cargo test -p tradingview-cli replay -- --nocapture

Run CLI contract tests that cover the special runners and ordinary envelopes:

    cargo test -p tradingview-cli --test cli_contract -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop -- --nocapture

Then run the workspace baseline:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

## Validation and Acceptance

Acceptance requires all of the following observable behavior:

- an injected writer returning `BrokenPipe` does not panic;
- pretty JSON and compact JSONL bytes are unchanged for successful writes;
- the writer-injected watch runner accepts readiness, performs one fake poll,
  detects `BrokenPipe` on the sample write, returns success, and leaves the
  poll count at exactly one;
- a JSONL subprocess whose stdout reader closes exits promptly without panic
  text or a backtrace;
- a successful command interrupted only by a closed stdout exits successfully;
- a command that was already failing keeps its original exit code if stderr
  is unavailable;
- serialization and non-broken write failures are not mislabeled as normal
  consumer completion;
- existing JSON envelopes and JSONL readiness, sample, heartbeat, step, and
  summary payloads remain unchanged when the consumer stays open;
- focused tests and the workspace baseline are green.

The live pipeline smoke is supplemental because it depends on the public
scanner endpoint. If skipped, record the environment requirement and rely on
the deterministic subprocess and injected-writer tests.

## Idempotence and Recovery

The code and tests are safe to rerun. Do not change signal disposition, shell
configuration, or external TradingView state. If output-result propagation
causes broad unrelated churn, stop and narrow the helper interface rather
than changing command contracts. If a partially updated runner no longer
compiles, finish one runner family at a time and run its focused tests before
continuing.

Do not use destructive Git commands to recover. Preserve unrelated worktree
changes and adapt the plan if another contributor has modified an overlapping
output call site.

## Artifacts and Notes

The before-fix reproduction is summarized, not copied verbatim, in
`Surprises & Discoveries`. At completion, add only concise public-safe evidence
such as exit code, test name, and whether panic text was absent. Do not add raw
JSONL samples or machine-specific paths.

## Interfaces and Dependencies

Keep the output types private to `tradingview-cli`. A suitable final interface
uses a private `OutputDisposition` with `Written` and `BrokenPipe` variants and
a private `OutputFailure` that distinguishes serialization from other I/O.
The common writer function accepts an injected `std::io::Write` and returns
`Result<OutputDisposition, OutputFailure>`. Map an I/O error whose kind is
`std::io::ErrorKind::BrokenPipe` to the disposition rather than the failure.

Special JSONL runners use a private runner error with distinct application,
stdout-output, and stderr-output variants. This lets
`crates/cli/src/app/runner.rs` attempt stderr exactly once for application and
stdout failures while returning code 1 directly for a stderr-output failure.
Do not parse error strings or convert `BrokenPipe` into a normal
`tradingview_core::ErrorKind`.

The private watch test seam accepts injected stdout and stderr writers plus a
poll closure returning the same sample value used by the production loop. It
must execute the production loop body; do not reproduce the loop in test-only
code. The production entry point supplies the real writers and existing
`watch_sample` function.

Use only `std::io`, existing serde/serde_json dependencies, and current
application types. Do not add a signal-handling crate, platform-specific
dependency, tracing redesign, or new JSON contract.

## Open Questions

There are no unresolved critical or noncritical behavior questions blocking
implementation. The stdout, terminal stderr, and nonterminal JSONL stderr
policies are fixed in the Decision Log. If the exact private type names need
minor adjustment to fit Rust ownership or generic constraints, preserve those
behaviors and record the naming-only adjustment in the Decision Log.

Revision note: created on 2026-07-10 as the first active implementation plan
for the `v0.26.0` robustness roadmap after two read-only architecture reviews.
Revised on 2026-07-10 to fix stderr behavior, deterministic poll-stop testing,
and private output interfaces before implementation. Revised again on
2026-07-10 after implementation and local validation to record green evidence
and leave Gate 1 open for independent review. Finalized on 2026-07-10 after an
independent read-only review reported no findings.
