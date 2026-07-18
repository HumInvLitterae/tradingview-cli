# v0.29.0 pre-release completion and architecture audit

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while work proceeds.
Maintain it according to `.agents/PLANS.md`.

## Purpose / Big Picture

This slice freezes v0.29 feature work and determines whether the completed CDP
transport-diagnostics candidate is coherent and ready to enter release
preparation. A contributor should be able to inspect the exact
`v0.28.0..HEAD` candidate, verify every changed production boundary and public
contract, run the complete deterministic baseline, and state whether a
release-blocking defect or architecture refactor remains.

The promoted v0.29 behavior is intentionally narrow: public-safe
`failure_stage` details on existing Desktop CDP transport errors, typed internal
stage observations, deterministic deadline/classification coverage, and a
bounded ignored measurement probe. The connection/evaluation topology and
recovery-semantics work produced reviewed inventories, not production
optimizations. Retry, reconnect, public recovery metadata, shared sessions,
brokers, generalized waits, and method-post-dispatch restart remain deferred.

No new command, option, payload field beyond the existing additive
`failure_stage`, dependency, source, fallback, live probe, version bump, or
release operation belongs in this audit. Small contract-preserving fixes may be
made here. A larger behavior change or refactor must stop the audit and receive
its own ExecPlan.

## Progress

- [x] (2026-07-18) Completed and archived transport measurement, topology
  audit, and recovery-semantics inventory with focused reviews green.
- [x] (2026-07-18) Created this self-contained completion and architecture
  audit ExecPlan and synchronized current planning state.
- [x] (2026-07-18) Obtained focused independent plan review with no blocking
  finding. Added the direct `failure_stage` CLI contract filter requested as a
  low-priority execution-time clarification.
- [x] (2026-07-18) Froze and classified the exact `v0.28.0..HEAD` candidate:
  five commits, 23 changed paths, and one production diagnostics slice.
- [x] (2026-07-18) Audited transport stage ownership, absolute deadlines, error mapping, and
  public-safety boundaries end to end.
- [x] (2026-07-18) Audited ordinary command behavior, source ownership, documentation,
  packaged guidance, tests, and deferred-candidate state.
- [x] (2026-07-18) Inspected changed module responsibilities and recorded the architecture
  verdict without using file size alone as a refactor trigger.
- [x] (2026-07-18) Ran focused tests and the complete non-live validation baseline.
- [x] (2026-07-18) Prepared a public-safe read-only reviewer prompt and
  obtained focused audit review with no finding or release blocker.
- [x] (2026-07-18) Recorded the final outcome and archived this plan. No
  correction or validation rerun was required after independent review.

## Surprises & Discoveries

- Observation: the v0.29 candidate contains only one production behavior
  slice even though the documentation program has three completed plans.
  Evidence: `v0.28.0..HEAD` changes production Rust in `tradingview-cdp` and one
  CLI contract test; the topology and recovery plans added reviewed source
  inventories without changing production behavior.

- Observation: the bounded measurement implementation is intentionally
  test-only except for the shared typed observation hooks and public-safe error
  mapping.
  Evidence: `crates/cdp/src/measurement.rs` and observer injection are guarded
  for tests, while ordinary command construction leaves the observer absent.

- Observation: one Desktop CLI contract fixture remains deliberately slow in
  the full workspace suite, but completed successfully without changing the
  audit verdict.
  Evidence: `read_utilities_attempt_connection_when_cdp_is_unavailable`
  completed in approximately 300 seconds during the non-live baseline.

## Decision Log

- Decision: freeze the v0.29 candidate before this audit.
  Rationale: measurement and classification are complete, while every proposed
  behavior expansion lacks either observed need, safely derivable state, or an
  explicit policy decision. Adding one during the audit would invalidate the
  candidate boundary.
  Date/Author: 2026-07-18 / Codex

- Decision: treat the topology and recovery inventories as release evidence,
  not shipped capabilities.
  Rationale: they document ownership and explain why optimization or public
  recovery metadata was deferred. Neither authorizes runtime changes.
  Date/Author: 2026-07-18 / Codex

- Decision: do not repeat the ignored live measurement during the audit.
  Rationale: the owner-approved run already supplied bounded aggregate
  non-regression evidence. Deterministic fixtures and ordinary contract tests
  are the release gate; another quiet live run cannot prove reliability.
  Date/Author: 2026-07-18 / Codex

- Decision: put two bounded investigations between the initial audit and
  release readiness.
  Rationale: the prior indicator-search no-go was a current-build readiness
  result rather than a durable impossibility proof, and the explicit-target
  transport run did not represent repeated heuristic or mixed CLI use. Each
  investigation remains separate from this audit and adds no production
  behavior by itself. If either promotes implementation, refresh this audit
  against the changed final candidate before release readiness.
  Date/Author: 2026-07-18 / Codex

## Outcomes & Retrospective

The initial audit found no release blocker or architecture refactor prerequisite
in the frozen five-commit candidate. Shipped behavior remains limited to typed
internal transport observations and additive allowlisted `failure_stage`
details; measurement infrastructure and stale-target diagnosis remain
test-only. Ordinary commands gain no retry, reconnect, diagnostic
re-discovery, background work, or success-envelope timing metadata. Existing
error kind, message, exit-code, deadline, queue-limit, broken-pipe, and JSONL
termination contracts remain intact.

Focused CDP diagnostics, client, transport, and measurement tests passed; the
two connection-failure CLI contracts and the CLI help contract passed. The
complete non-live workspace baseline, strict Clippy, formatting, metadata,
workflow parsing, public hygiene, release-script syntax, guide parity, and diff
hygiene also passed. Independent focused audit review reproduced the candidate
classification and validation evidence, reported no finding, and agreed that
no release-blocking architecture refactor is required. The indicator-search
reassessment may now begin. Any live matrix still requires separate owner
approval, and any promoted production implementation still requires a new
ExecPlan and refreshed audit.

## Context and Orientation

The latest release is `v0.28.0`, tagged at commit `e47ba44`. The workspace
version remains `0.28.0` until release readiness. The candidate begins after
that tag and currently consists of five commits: strategy and plan setup,
transport diagnostics implementation, topology-audit closeout,
recovery-inventory planning, and recovery-inventory closeout.

`crates/cdp/src/transport.rs` owns target-list HTTP and WebSocket connection.
`crates/cdp/src/client.rs` owns CDP method send/response and event waiting.
`crates/cdp/src/diagnostics.rs` defines the internal stage vocabulary and the
small public-safe `failure_stage` mapping. `crates/cdp/src/measurement.rs` owns
test-only deterministic and ignored live measurement support.
`crates/cli/tests/cli_contract.rs` verifies the additive error detail while
preserving the existing error kind, message, and exit code.

The stable architecture and development explanations live in
`docs/architecture.md` and `docs/development.md`. Runtime package guidance is
generated from `packaging/agent/AGENTS.md`. The longer strategy is
`docs/notes/cdp-stability-and-autonomous-operation-strategy.md`; the two source
inventories are `docs/notes/cdp-connection-evaluation-topology-audit.md` and
`docs/notes/cdp-recovery-semantics-inventory.md`.

An absolute deadline is one fixed end time shared by all iterations of a wait;
unrelated traffic must not reset it. A pre-dispatch failure occurs before a CDP
method could have been sent. A post-dispatch unknown outcome means the method
may have executed even though the caller did not receive a usable result.
These terms classify evidence only; they do not authorize retry.

## Plan of Work

First, record the exact candidate commit and classify every path changed from
`v0.28.0`. Separate production Rust, tests, public docs, planning notes, and
archived plans. Confirm Cargo manifests and the lockfile did not change, the
workspace version is still `0.28.0`, and no unrelated feature entered the
candidate.

Second, trace target listing, target selection, WebSocket connection, method
calls, and event waits through their observer boundaries and public error
mapping. Verify one deadline per operation, unchanged timeout values, FIFO
limits, no raw endpoint or target data in diagnostics, and no observer on
ordinary production construction. Confirm stale-target re-discovery exists
only in the ignored probe and never becomes retry or success.

Third, audit the additive CLI contract. Existing `ErrorKind`, message, details,
and exit-code behavior must be preserved except for the allowlisted
`failure_stage`. Unknown internal stages must map conservatively without raw
values. Success envelopes must not gain timing metadata. Broken-pipe and JSONL
termination behavior must remain unrelated and unchanged.

Fourth, compare implementation with README, architecture, development docs,
packaged guidance, runtime skills under `.agents/skills/`, changelog, roadmap,
work inventory, plan index, and both source inventories. Confirm these sources
do not claim retry, reliability, shared-session performance, generalized
recovery, or live representativeness. The explicit-target live measurement
must remain qualified as one bounded run that does not exercise heuristic
target selection.

Fifth, inspect changed production modules for cohesive ownership, duplicated
policy, dead paths, test-only production APIs, and avoidable coupling. Do not
request a split based on line count. A refactor is release-blocking only when
current ownership causes incorrect behavior, contract drift, unsafe state,
duplicated authoritative policy, or an untestable boundary.

Finally, run focused and full validation, record concise evidence, and prepare
a read-only reviewer prompt. Apply only narrow corrections. After independent
audit review is green, execute the separate indicator-search current-build
reassessment and consecutive-invocation resilience plans. If they add no
production behavior, perform a narrow final candidate/state check and archive
this plan. If either promotes implementation, refresh the relevant candidate,
architecture, docs, and full validation sections before archive. Only then
create the separate v0.29.0 release-readiness ExecPlan.

## Concrete Steps

Run from the repository root:

    git rev-parse HEAD
    git log --oneline v0.28.0..HEAD
    git diff --stat v0.28.0..HEAD
    git diff --name-status v0.28.0..HEAD
    git diff v0.28.0..HEAD -- Cargo.toml Cargo.lock crates .github mise.toml
    rg -n "failure_stage|TransportStage|TransportObserver|with_observer|deadline|timeout_at|pending_event" crates/cdp crates/cli/tests
    rg -n "retry|reconnect|broker|shared connection|recovery_action|failure_stage|explicit-target|heuristic" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md
    rg -n "TODO|FIXME|todo!|unimplemented!|panic!" crates/cdp crates/cli/tests docs/architecture.md docs/development.md

Inspect ordinary construction and test-only boundaries directly. Search output
is an inventory aid, not proof by itself; every match in changed production
files must be classified in the audit evidence.

Run focused tests:

    cargo test -p tradingview-cdp diagnostics -- --nocapture
    cargo test -p tradingview-cdp client -- --nocapture
    cargo test -p tradingview-cdp transport -- --nocapture
    cargo test -p tradingview-cdp measurement -- --nocapture
    cargo test -p tradingview-cli --test cli_contract connection_failure -- --nocapture
    cargo test -p tradingview-cli --test cli_contract help_lists_v1_commands -- --nocapture

Each command must execute at least one relevant test. The ignored live
measurement must remain ignored; do not set its gate variables.

Run the complete non-live baseline:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    ruby -e 'require "yaml"; YAML.load_file(".github/workflows/ci.yml"); YAML.load_file(".github/workflows/release.yml"); puts "workflow YAML parsed"'
    git diff --check

If a focused filter runs zero tests, correct the exact command in this living
plan before accepting the result. Do not substitute remote CI for the local
candidate baseline.

## Validation and Acceptance

Acceptance requires a complete classified `v0.28.0..HEAD` diff and an
end-to-end account of each changed production boundary. Existing CDP timeout
and queue contracts must remain finite. Public errors may add only a recognized
`failure_stage`; they must preserve prior classification and omit raw targets,
URLs, payloads, environment values, local paths, and account metadata.

Ordinary commands must perform no new retry, reconnect, diagnostic
re-discovery, source fallback, or background work. The ignored probe must be
bounded, explicitly gated, read-only, and public-safe. Its previous aggregate
result may support non-regression only.

Docs and packaged guidance must distinguish implemented diagnostics from the
two documentation-only inventories and every deferred candidate. No source may
claim that retries, public recovery metadata, shared sessions, brokers, or
generalized waits ship in v0.29.

All focused commands must run relevant tests and all baseline commands must
pass. The final architecture verdict must be exactly one of: no release-
blocking architecture issue; small corrections applied with no dedicated
refactor required; or release readiness blocked by a named defect/refactor.
Independent review must be green before the two queued investigations start.
Release-readiness planning begins only after both investigations close and any
promoted implementation receives a refreshed completion-audit verdict.

## Idempotence and Recovery

All inspection and non-live validation is repeatable. Do not run the ignored
live measurement, connect to TradingView Desktop for this audit, mutate remote
state, alter environment credentials, or touch preserved stashes. Do not reset,
stash, apply, drop, push, tag, or create a GitHub Release.

If a deterministic check fails, retain the evidence, fix only the owning
contract, update this living plan, and rerun focused validation before the full
baseline. If a finding requires new behavior or broad restructuring, mark the
audit blocked and create a separate ExecPlan instead of expanding this slice.

## Artifacts and Notes

Record commit counts, changed-path classifications, named module boundaries,
test counts, and concise pass/fail summaries. Do not retain raw JSON, target or
session identifiers, endpoint URLs, Runtime payloads, account-local metadata,
credentials, machine-specific paths, or one-off reviewer prompts.

Initial audit evidence: `v0.28.0..HEAD` contains five commits, 23 changed
paths, 3,331 insertions, and 29 deletions. Production Rust changes are confined
to five `tradingview-cdp` files plus the additive CLI contract test. Cargo
manifests, the lockfile, workflows, and `mise.toml` are unchanged. The public
hygiene scan inspected 607 tracked files. No live probe was rerun.

The final reviewer prompt must name the candidate range and request findings in
severity order, verification of deferred boundaries, and an explicit release-
readiness go/no-go. It must tell the reviewer not to run live probes or edit,
stage, commit, push, or touch stashes.

## Interfaces and Dependencies

This audit adds no production interface or dependency. The authoritative
implemented interfaces remain the internal transport observation vocabulary in
`crates/cdp/src/diagnostics.rs`, its allowlisted public `failure_stage`
mapping, and existing `tradingview-core` error envelopes. Timing remains
internal/test-only. The common success envelope has no new metadata layer.

Any retry, reconnect, public recovery enum, shared-session mode, broker,
generalized wait, new source, or production dependency requires a separate
reviewed ExecPlan and, where applicable, owner approval.

## Open Questions

No critical planning question is open. The audit must determine whether the
frozen candidate has a release blocker, not reopen deferred candidates.
Transport failure frequency, a possible conservative `transport_unknown`
mapping, pre-dispatch retry need, and shared-connection value remain
`UNCONFIRMED` and are not release prerequisites.

Revision note (2026-07-18): created the audit after all ordered v0.29
measurement and inventory slices completed focused review. The plan freezes the
candidate, separates implemented diagnostics from documentation-only evidence,
and makes non-promotion of retry and broker work part of acceptance.

Revision note (2026-07-18): focused plan review was green and recommended
directly filtering the two `connection_failure` CLI contract tests. Added that
command and inserted two separately bounded investigations before release
readiness; neither investigation is part of this audit or authorizes production
behavior.

Revision note (2026-07-18): executed the initial audit against the frozen
five-commit candidate. Focused and full non-live validation were green, no
release-blocking architecture issue was found, and the project state now awaits
focused independent audit review before either queued investigation starts.

Revision note (2026-07-18): focused audit review was green with no finding.
Archived the completed initial audit and advanced the ordered current plan to
the indicator-search current-build reassessment without authorizing its live
matrix or any production implementation.
