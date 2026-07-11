# v0.26.0 pre-release completion and architecture audit

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while working. Maintain
this document in accordance with `.agents/PLANS.md`.

## Purpose / Big Picture

This slice audits the completed `v0.26.0` robustness work before release
readiness. A contributor should be able to confirm that closed-pipe output,
CDP event retention, transport deadlines, bars heartbeat framing, HTTP error
classification, and bounded multi-symbol reads are mutually consistent and do
not require a release-blocking refactor.

No feature is added here. Small documentation, test, naming, or metadata drift
may be corrected. A larger responsibility split or public behavior change must
be reported as a separate refactor plan rather than folded into this audit.

## Progress

- [x] (2026-07-11) Confirmed Gate 6 commit `f23a0d5` and a clean tracked worktree.
- [x] (2026-07-11) Chose the pre-release audit instead of promoting a retained feature.
- [x] (2026-07-11) Archived the completed Gate 6 ExecPlan.
- [x] (2026-07-11) Created this self-contained audit ExecPlan.
- [x] (2026-07-11) Made this plan current in the plan index, roadmap, work inventory, and continuity ledger.
- [x] (2026-07-11) Audited Gate 1 JSON and JSONL broken-pipe behavior and output-policy ownership.
- [x] (2026-07-11) Audited Gate 2 CDP FIFO buffering, queue limits, absolute deadlines, and client ownership.
- [x] (2026-07-11) Audited Gate 3 HTTP and WebSocket deadlines plus configured-client reuse.
- [x] (2026-07-11) Audited Gate 4 bars heartbeat framing, parser compatibility, and accepted live evidence.
- [x] (2026-07-11) Audited Gate 5 HTTP error taxonomy and public-safe diagnostics across owning crates.
- [x] (2026-07-11) Audited Gate 6 concurrency limits, input ordering, first-error determinism, and source boundaries.
- [x] (2026-07-11) Inspected changed module sizes, dependencies, duplicate helpers, and responsibility boundaries.
- [x] (2026-07-11) Scanned public docs, packaged guidance, tests, and errors for stale or private information.
- [x] (2026-07-11) Corrected three small current-source status drifts; production code and public contracts were unchanged.
- [x] (2026-07-11) Recorded that no release-blocking architecture issue or pre-release refactor is required.
- [x] (2026-07-11) Ran focused output and CLI output-contract tests.
- [x] (2026-07-11) Ran focused CDP client and transport tests.
- [x] (2026-07-11) Ran focused bars protocol and transport tests through the Market suite.
- [x] (2026-07-11) Ran focused Market, Scanner, and Pine HTTP/error/concurrency tests.
- [x] (2026-07-11) Ran focused CLI quote, bars, diagnose, and Desktop contract tests.
- [x] (2026-07-11) Ran final formatting, metadata, diff, packaging, and guide-consistency checks after documentation finalization.
- [x] (2026-07-11) Did not modify runtime skills, so no skill validation was required.
- [x] (2026-07-11) Created a self-contained independent-review prompt under `docs/notes/`.
- [x] (2026-07-11) Recorded final outcomes as `implemented and fully validated; independent review pending` and stopped uncommitted.
- [x] (2026-07-11) Independent review approved the implementation and architecture conclusion and identified two documentation findings.
- [x] (2026-07-11) Replaced ten machine-specific validator paths across three archived plans with a portable `CODEX_HOME` form.
- [x] (2026-07-11) Corrected the `quote.rs` production boundary evidence from line 294 to line 481.
- [x] (2026-07-11) Focused re-review approved both documentation corrections with no remaining findings.

## Surprises & Discoveries

- Observation: the largest inspected v0.26-adjacent modules are not all new in
  this release.
  Evidence: `compare.rs`, bars `transport.rs`, `quote.rs`, and `CdpClient` are
  already substantial, so line count is a prompt for responsibility review,
  not by itself a release blocker.

- Observation: test modules account for a large share of the apparent module
  size.
  Evidence: the test module begins at line 482 in `quote.rs`, 586 in
  `compare.rs`, 526 in `events.rs`, 464 in `CdpClient`, and 685 in bars
  `transport.rs`. The line 295 `#[cfg(test)]` in `quote.rs` applies only to one
  helper; production code continues through line 481.

- Observation: current-source planning contained three stale descriptions.
  Evidence: the roadmap introduction described only Gate 1 as complete, R3
  still said independent review remained, and R6 described the pre-change
  sequential behavior in the present tense. All three are corrected without
  changing product behavior.

- Observation: the first hygiene assessment overlooked inherited executable
  paths in archived plans.
  Evidence: independent review found ten user-specific Codex validator paths
  across three archived plans. They were not packaged or runtime data,
  but they violated the repository-wide documentation policy and this audit's
  acceptance condition. All ten now use
  `${CODEX_HOME:-$HOME/.codex}/...`; other scan matches are policy text,
  detection patterns, ignored-live assertions, or Pine-template content.

## Decision Log

- Decision: proceed directly to the v0.26 pre-release audit instead of adding
  event-range or Replay screenshot work.
  Rationale: Gates 1 through 6 form a coherent robustness and efficiency
  release, and the roadmap explicitly says they are sufficient for audit.
  Date/Author: 2026-07-11 / Codex

- Decision: retain previously approved live bars heartbeat evidence and audit
  its deterministic regression coverage rather than requiring another live
  probe.
  Rationale: remote acceptance was already independently reviewed; repeating
  it adds external variability without changing the release decision.
  Date/Author: 2026-07-11 / Codex

- Decision: prepare a tracked independent-review prompt in `docs/notes/`.
  Rationale: the user requested a reusable reviewer handoff after completion,
  and repository guidance assigns handoff material to `docs/notes/`.
  Date/Author: 2026-07-11 / Codex

- Decision: no dedicated architecture refactor is required before v0.26
  release readiness.
  Rationale: output policy is centralized; CDP remains a bounded sequential
  client; reqwest-specific helpers remain with owning crates; bars protocol is
  separated from transport; and bounded concurrency is isolated in a small
  private runner. Large files remain future maintainability candidates, but
  no inspected responsibility drift changes a public contract or creates a
  release risk.
  Date/Author: 2026-07-11 / Codex

## Outcomes & Retrospective

The audit found no release-blocking architecture issue and no need for a
dedicated refactor before v0.26 release readiness. It corrected three small
current-source status drifts and changed no production code, public command,
payload, dependency, or runtime skill.

Independent review confirmed the Gate 1 through 6 implementation and
architecture conclusion. It found two documentation issues: ten inherited
machine-specific validator paths in archived plans and an inaccurate
`quote.rs` production-line boundary. Both are corrected without changing the
architecture decision or production behavior. Focused re-review reported no
remaining findings.

Focused tests, strict clippy, and the full workspace suite are green. The
accepted Gate 4 live evidence was not repeated; deterministic heartbeat tests
remain green and the only residual risk is ordinary external TradingView
service variability. After final non-mutating checks, this audit should be
handed to an independent reviewer and remain uncommitted until review and any
focused re-review are green.

## Context and Orientation

The latest public release is `v0.25.0`; the workspace package version remains
`0.25.0`. The `v0.26.0` work is six completed gates. Gate 1 replaced panic-prone
JSON and JSONL printing with explicit output policy. Gate 2 retained CDP events
that arrive while a method response is pending and bounded the FIFO queue to
1024 events and 8 MiB. Gate 3 added finite HTTP and WebSocket deadlines and
reused configured clients. Gate 4 corrected the browserless bars heartbeat
frame and proved remote acceptance through a public-safe opt-in probe. Gate 5
standardized HTTP failure classification. Gate 6 measured and adopted
maximum-four concurrency for bounded Desktop-free quote, compare, and event
reads while restoring input order.

The principal implementation owners are `crates/cli/src/app/` for process and
JSONL runners, `crates/cdp/src/` for Desktop transport, and
`crates/market/src/` for Desktop-free reads. Scanner and Pine own their HTTP
clients and reqwest-specific error mapping. `tradingview-core` continues to own
shared error kinds and envelopes without depending on reqwest.

## Plan of Work

First, make this audit the current plan and preserve Gate 6 as completed
history. Do not alter the package version or start release notes.

Second, inspect each Gate's implementation, tests, public documentation, and
contract boundaries. Confirm that output failure policies remain consistent
between one-shot and JSONL paths; CDP buffering remains bounded and sequential;
transport timeouts and client reuse remain owned by the appropriate crates;
bars heartbeat parsing keeps legacy receive compatibility; HTTP diagnostics
remain public-safe; and bounded concurrency does not affect chart-backed state
operations.

Third, inspect architecture posture rather than treating test success as the
only criterion. Review line counts, dependencies, repeated HTTP helpers,
payload shaping, event-loop ownership, and concurrency/error-selection logic.
Apply only corrections whose behavior and scope are plainly local. Record any
larger split or API move as a proposed dedicated refactor.

Fourth, run focused validation followed by the full workspace baseline. Then
write a reviewer prompt that tells an independent agent what changed, what to
inspect, what commands to run, what privacy constraints apply, and that the
review must not edit, stage, commit, or push.

Finally, update the living sections and continuity ledger. Stop with all audit
changes uncommitted and independent review pending.

## Concrete Steps

Run all commands from the repository root.

Inspect architecture and documentation with `rg`, `wc -l`, `cargo tree`, and
`git diff`. Record concise findings in this plan; do not paste private or raw
live data.

Run focused tests for output, CDP, bars, HTTP classification, bounded reads,
and public CLI contracts. Then run:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md

No live TradingView request is required. If a deterministic test exposes a
remote-acceptance uncertainty that prior evidence does not cover, stop and
record that uncertainty instead of silently running an account-bearing smoke.

## Validation and Acceptance

The audit is acceptable when all Gate 1 through 6 contracts are represented
consistently in implementation, tests, stable docs, and current planning; no
public document or diagnostic exposes raw frames, raw payloads, raw bars,
credentials, session identifiers, account-local identifiers, raw target ids,
or machine-specific absolute paths; and architecture inspection gives one
explicit release decision.

The focused and full command set must pass, or every failure must be explained
with its release impact. Existing ignored live tests may remain ignored. The
review prompt must be self-contained and usable without conversation history.

## Idempotence and Recovery

All audit searches and tests are non-destructive and repeatable. Documentation
corrections should be small and applied with `apply_patch`. If a test writes
build artifacts, rerunning it is safe. Do not reset or revert unrelated user
changes. Do not commit until independent review and any focused re-review are
green.

## Artifacts and Notes

The completed Gate 6 plan is archived at
`docs/plans/archives/tradingview-cli-bounded-multi-symbol-concurrency.md`.
The independent-review prompt will live at
`docs/notes/v0.26-pre-release-audit-review-prompt.md`.

## Interfaces and Dependencies

This audit adds no command, option, payload field, dependency, retry, timeout,
fallback, source mixing, or version bump. It may update documentation and tests
only when they correct clear drift. Any required production behavior change is
a blocker that must be described before implementation.

## Open Questions

None. The next planned slice is v0.26 release readiness.

Revision note (2026-07-11): created after Gate 6 completed implementation,
validation, independent review, focused re-review, and commit `f23a0d5`.

Revision note (2026-07-11): completed the audit with three current-source
documentation corrections, no production changes, no release-blocking
architecture finding, green focused and full validation, and a self-contained
independent-review prompt. The work remains uncommitted pending review.

Revision note (2026-07-11): corrected the two independent-review findings by
portableizing ten archived validator commands and fixing the `quote.rs`
production boundary evidence. The architecture conclusion remains unchanged;
focused re-review reported no remaining findings.
