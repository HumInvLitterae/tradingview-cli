# Audit the frozen v0.30 candidate before release preparation

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while work proceeds.
Maintain it according to `.agents/PLANS.md`.

## Purpose / Big Picture

This slice freezes v0.30 feature work and determines whether the final
candidate is coherent, correctly documented, fully tested, and ready for a
separate release-readiness plan. A contributor should be able to reproduce the
exact `v0.29.0..HEAD` inventory, distinguish shipped Replay behavior from
test-only investigation evidence, trace the new artifact and summary contracts
end to end, run the complete non-live baseline, and state whether any
release-blocking defect or architecture correction remains.

The only promoted user-facing feature is explicit chart-screenshot attachment
for bounded `tv replay log` steps. The candidate also corrects the existing
`replay_left_running` summary by carrying the post-step Replay state through the
existing expression and model normalizer. Chart-read latency attribution and
renderer foreground feasibility are reviewed test-only investigations, not
runtime capabilities. Public timing, timeout changes, retry, reconnect,
foreground transitions, indicator search, shared sessions, and brokers remain
deferred.

No new feature, dependency, source, fallback, live operation, version bump, or
release operation belongs in this audit. Small contract-preserving corrections
may be made here. A larger behavior change or refactor stops the audit and
requires its own ExecPlan.

## Progress

- [x] (2026-07-20) Closed and archived chart-read latency attribution,
  renderer foreground feasibility, and Replay screenshot attachment after
  their focused reviews.
- [x] (2026-07-20) Created this completion and architecture audit ExecPlan and
  synchronized current planning state.
- [x] (2026-07-20) Obtained focused independent plan review with no blocking
  finding. The execution-time candidate counts must be refreshed from HEAD.
- [x] (2026-07-20) Froze candidate `62ea01c`: 20 commits, 26 changed
  paths, no manifest, lockfile, workflow, or `mise.toml` change.
- [x] (2026-07-20) Audited Replay screenshot attachment and post-step
  running-state contracts
  end to end.
- [x] (2026-07-20) Proved test-only investigation boundaries and maintained
  defers without rerunning ignored live tests.
- [x] (2026-07-20) Audited public docs, packaged guidance, release-package inclusion, and
  architecture ownership.
- [x] (2026-07-20) Ran focused tests and the complete non-live validation
  baseline; every gate was green.
- [ ] Obtain focused independent audit review, record the final outcome, and
  archive before release readiness.

## Milestones

### Milestone: freeze and classify the candidate

Record the reviewed HEAD, every commit, every changed path, Cargo state, and
the production/test/docs classification. Completion means another contributor
can reproduce the inventory from the tag and no changed production path is
unexplained.

### Milestone: audit shipped behavior and non-shipped evidence

Trace Replay options, validation, step ordering, screenshot persistence,
attachment output, counters, broken-pipe behavior, and post-step running-state
normalization. Separately prove that both measurement modules are test-only and
that their live evidence has not become production policy. Completion means
the candidate's observable behavior and exclusions are exact.

### Milestone: validate and obtain independent review

Run focused contract tests, the full workspace baseline, package and hygiene
checks, then prepare a read-only reviewer request. Completion means review is
green, any narrow correction is revalidated, and release readiness is the only
remaining local work item.

## Surprises & Discoveries

- Observation: v0.30 contains two large Rust measurement modules but neither is
  shipped in an ordinary build.
  Evidence: `crates/cli/src/ops.rs` includes both modules only under
  `#[cfg(test)]`; their ignored live tests are evidence, not public commands.

- Observation: the Replay smoke found a summary defect outside the new
  screenshot attachment itself.
  Evidence: `replay_step` checked Replay state before mutation but omitted the
  post-step boolean from its returned payload. The correction adds that field
  to the same expression and adds no evaluation, retry, or step.

- Observation: screenshot persistence precedes JSONL emission.
  Evidence: `run_replay_log_command` captures and writes the attachment before
  emitting the step event. On a broken pipe, that step's PNG may remain even
  though its JSONL event was not delivered; no subsequent step runs.

- Observation: the final candidate contains 20 commits and 26 changed paths,
  one commit and one path more than the plan-creation snapshot.
  Evidence: `git rev-list --count v0.29.0..HEAD` returned 20 and the refreshed
  name-only inventory contained 26 paths at reviewed HEAD `62ea01c`.

## Decision Log

- Decision: freeze the v0.30 candidate after one product slice and two reviewed
  investigations.
  Rationale: the roadmap's product-value requirement is satisfied, both
  observability lanes reached explicit defer/no-go outcomes, and adding another
  feature would invalidate the candidate boundary.
  Date/Author: 2026-07-20 / Codex

- Decision: treat latency and renderer harnesses as release evidence only.
  Rationale: both are test-gated, expose no ordinary command, and intentionally
  did not promote timing, timeout, retry, foreground, session, or broker
  behavior.
  Date/Author: 2026-07-20 / Codex

- Decision: do not repeat any ignored live matrix or Replay smoke during the
  audit.
  Rationale: owner-approved runs already supplied bounded evidence. The audit
  verifies deterministic contracts and recorded aggregate evidence; repetition
  would mutate Desktop state without proving a new release property.
  Date/Author: 2026-07-20 / Codex

- Decision: preserve standalone screenshot overwrite behavior.
  Rationale: Replay attachment owns a separate no-overwrite artifact contract;
  changing the existing standalone command would be unrelated public behavior.
  Date/Author: 2026-07-20 / Codex

## Outcomes & Retrospective

The frozen candidate at `62ea01c` contains 20 commits and 26 changed paths.
Production behavior changes are limited to Replay screenshot attachment and
the post-step running-state correction. Cargo manifests, `Cargo.lock`, GitHub
workflows, and `mise.toml` are unchanged. The two large measurement modules are
included only under `#[cfg(test)]` and add no ordinary command or runtime
policy.

Replay controls were traced from CLI parsing through pre-connect validation,
deterministic path planning, successful step handling, one capture attempt,
create-new persistence, attachment composition, counters, JSONL emission, and
summary. Existing destinations remain untouched, partial writes remove only
their newly created file, screenshot errors are sanitized and independent from
Replay failure, and standalone screenshot overwrite behavior is unchanged.
The `replay_left_running` correction reaches the summary through the existing
normalizer without adding evaluate, polling, retry, or timeout behavior.

Focused tests passed: Replay log 15, Replay operations 19, screenshot 24,
Desktop Replay contracts 7, chart-read latency fixtures 8 with one ignored live
test, and renderer foreground fixtures 8 with one ignored live test. The full
workspace baseline passed with CDP 45 passed and one ignored, CLI unit 465
passed and five ignored, Desktop CLI contracts 100 passed, and all remaining
suites and doc tests green. Formatting, strict workspace Clippy, metadata,
public hygiene over 623 tracked files, package-script syntax, contributor-guide
parity, workflow YAML parsing, and diff hygiene passed.

Staging the current debug binary produced 46 files with exactly eight runtime
skills under each agent skill root. Plans, notes, the local ledger,
development-only skills, and live artifacts were absent. No release-blocking
architecture refactor or contract defect was found locally. Focused independent
audit review is the remaining gate before archive and release readiness.

## Context and Orientation

The latest release is `v0.29.0`, tagged at commit `a774142`. The workspace
version remains `0.29.0` until release readiness. The frozen audit candidate is
`62ea01c` and `v0.29.0..62ea01c` contains 20 commits and 26 changed paths.

`crates/cli/src/cli.rs` defines Replay CLI options. `crates/cli/src/app/runner.rs`
forwards them to `crates/cli/src/app/replay_log.rs`, which validates artifact
controls before connecting, advances Replay, composes attachments, emits JSONL,
and produces counters. `crates/cli/src/ops/screenshot.rs` owns chart capture and
the Replay-specific create-new write. `crates/cli/src/ops/replay/control.rs`
owns the step expression. `crates/model/src/replay.rs` normalizes Replay context.

The option pair is `--attach-chart-screenshot` and
`--screenshot-output-dir <DIR>`. Both are required together. Planned files use
deterministic names such as `replay-step-0001.png`; existing paths are rejected
before Desktop connection and capture-time create-new writing closes the race.
A screenshot error is an attachment error, not a Replay step failure. It does
not retry or change the log end reason.

`crates/cli/src/ops/latency_measurement.rs` and
`crates/cli/src/ops/renderer_foreground_measurement.rs` are test-only. Their
reviewed evidence is recorded in archived plans. No live execution is part of
this audit.

## Plan of Work

First, record the exact HEAD and classify every path changed from `v0.29.0`.
Separate production Rust, test-only Rust, integration tests, public docs,
runtime skills, planning notes, and archived plans. Confirm manifests,
`Cargo.lock`, workflows, and `mise.toml` did not change and that the workspace
version remains `0.29.0`.

Second, trace screenshot controls from parsing through pre-dispatch validation,
path planning, successful Replay step, one capture attempt, create-new write,
attachment envelope, counters, JSONL emission, and summary. Verify both options
are required together before connection; all requested paths are preflighted;
existing files are untouched; partial writes remove only the newly created
file; screenshot failure is sanitized and independent; OHLCV and screenshot
attachments compose; and standalone screenshot replacement is unchanged.

Third, audit failure and termination boundaries. A failed Replay step performs
no capture. A screenshot failure does not retry the step or capture and does
not change Replay failure count or end reason. Broken pipe stops before another
step while allowing the just-written PNG to remain. Counters must satisfy
`requested = ok + error` and never exceed successful steps.

Fourth, trace `is_replay_started` from the post-step expression through
`replay_context_from`, `last_context`, and `replay_left_running`. Confirm the
correction adds no evaluate, polling cycle, retry, timeout extension, or
malformed-success relaxation. Audit start, stop, autoplay, trade, and status
for unrelated regressions.

Fifth, prove the measurement modules disappear from ordinary builds and add no
public command, production observer, dependency, or policy. Check archived
evidence wording: 40/40 in-process chart reads do not establish a repository-
wide latency distribution; HTTP activation and `Page.bringToFront` are only
current-build no-go candidates; renderer lifecycle remains deferred. No docs
may claim automatic foregrounding, indicator search, public timing, retry,
session, broker, or reliability guarantees.

Sixth, compare README, changelog, architecture/development guidance, packaged
agent guidance, Replay runtime skill, roadmap, work inventory, plan index, and
archived plans. Verify release packaging includes changed runtime guidance but
excludes plans, notes, the local ledger, development-only skills, and live
artifacts. No target ID, symbol, raw payload, local path, or account metadata
may appear in tracked evidence.

Finally, inspect changed production modules for coherent ownership, duplicated
policy, dead paths, test-only production APIs, and untestable boundaries. File
size alone is not a refactor trigger. Run focused and full validation and obtain
read-only independent review. A green review permits archive and creation of a
separate v0.30.0 release-readiness plan.

## Concrete Steps

Run from the repository root:

    git rev-parse HEAD
    git log --oneline v0.29.0..HEAD
    git diff --stat v0.29.0..HEAD
    git diff --name-status v0.29.0..HEAD
    git diff v0.29.0..HEAD -- Cargo.toml Cargo.lock .github mise.toml crates
    rg -n "attach_chart_screenshot|screenshot_output_dir|chart_screenshot|replay_left_running|is_replay_started" crates/cli crates/model
    rg -n "latency_measurement|renderer_foreground_measurement|cfg\(test\)" crates/cli/src
    rg -n "retry|reconnect|broker|shared session|foreground|indicator search|timing" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md

Run focused tests:

    cargo test -p tradingview-cli app::replay_log -- --nocapture
    cargo test -p tradingview-cli ops::replay -- --nocapture
    cargo test -p tradingview-cli ops::screenshot -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop replay -- --nocapture
    cargo test -p tradingview-cli latency_measurement -- --nocapture
    cargo test -p tradingview-cli renderer_foreground_measurement -- --nocapture

Every filter must execute at least one test. Ignored live tests must remain
ignored and must not be run with `--ignored`.

Run the complete non-live baseline:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    ruby -e 'require "yaml"; Dir[".github/workflows/*.yml"].each { |f| YAML.load_file(f) }'
    git diff --check

Record test counts, ignored counts, candidate commit/path counts, manifest and
dependency state, package boundaries, and architecture verdict in this plan.
Do not record raw live output or one-off reviewer prompts.

## Validation and Acceptance

The audit is complete only when the exact candidate is reproducible; every
production path is classified; Replay screenshot behavior and summary
correction are traced end to end; test-only investigations remain absent from
ordinary behavior; focused and full validation are green; public/package docs
match implementation; no private data or live artifact is tracked; and
independent review reports no unresolved release blocker.

No row of evidence may promote public timing, timeout changes, retry,
reconnect, foreground behavior, indicator search, session, or broker work.
Release readiness remains a separate plan and stops before owner-controlled
tag, push, workflow mutation, or GitHub Release publication.

## Idempotence and Recovery

All audit commands are read-only or repeatable validation. Do not run ignored
live tests, start or stop Replay, create screenshots, manipulate targets, apply
or drop either preserved stash, or push. If a focused filter runs zero tests,
correct this living plan before relying on it. If validation reveals a narrow
contract defect, fix and rerun its focused gate plus the full affected
baseline. If the correction changes feature scope or architecture, stop and
create a separate ExecPlan.

## Artifacts and Notes

Keep durable evidence aggregate and repository-relative. The prior Replay smoke
proved two successful steps and two valid PNG attachments, then explicitly
stopped Replay. It must not be rerun here. Temporary PNGs and target identity
are not audit artifacts. Preserve both local stashes unchanged.

## Interfaces and Dependencies

No new interface or dependency is planned. The audit preserves the existing
Replay JSONL envelope plus additive `attachments.chart_screenshot` and summary
counters, the existing Replay context normalizer, and the standalone screenshot
contract. Workspace version and third-party dependency state remain unchanged
until release readiness.

## Revision Note

2026-07-20: Initial plan created after Replay screenshot attachment focused
correction/evidence review completed green and all promoted v0.30 work closed.

2026-07-20: Focused plan review was green. Audit execution refreshed the
candidate to `62ea01c`, 20 commits, and 26 paths; all focused, full non-live,
package, hygiene, and architecture gates completed without a local blocker.
