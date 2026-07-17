# Inventory failure-specific recovery semantics

This ExecPlan is a living document. The sections `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to
date as work proceeds. This document must be maintained in accordance with
`.agents/PLANS.md` at the repository root.

## Purpose / Big Picture

The CLI currently reports stable error kinds, exit codes, and, for CDP
transport failures, a public-safe `failure_stage`. Those fields explain what
failed, but they do not tell an agent whether repeating a command is safe.
Safety depends on where failure occurred and what may already have happened. A
chart read that never connected, a mutation whose CDP response timed out, a
screenshot whose file was written, and a temporary chart mutation whose restore
failed cannot share one command-level `idempotent` value.

After this inventory, maintainers can read one durable matrix that classifies
actual failure boundaries and command workflow families by dispatch certainty,
possible side effects, verification state, and the safest operator response.
The inventory decides whether a small future recovery vocabulary is useful and
which meanings need separate contract plans. This slice does not add public
recovery metadata, retry a request, restart a command, introduce a wait command,
or change production behavior.

## Progress

- [x] (2026-07-17) Create this self-contained inventory plan after transport
  measurement and topology audit closeout.
- [ ] Freeze shared error, transport, runner, and output boundaries at the
  current production commit.
- [ ] Define documentation-only dispatch-state, effect-state, and operator-
  response labels.
- [ ] Inventory every shared transport failure boundary before and after method
  dispatch.
- [ ] Map every Desktop-backed command arm to a complete workflow archetype.
- [ ] Inventory local output, process lifecycle, temporary mutation/restore,
  multi-target, streaming, and partial-diagnostic exceptions.
- [ ] Write `docs/notes/cdp-recovery-semantics-inventory.md` with the complete
  matrix and grouping proof.
- [ ] Decide whether any recovery meaning should receive a later public-contract
  ExecPlan; do not implement it here.
- [ ] Synchronize stable planning documents with the reviewed result.
- [ ] Run focused and full validation appropriate to the final diff.
- [ ] Obtain focused independent inventory review and apply corrections.
- [ ] Archive this plan and create only the separately approved follow-up plans.

## Surprises & Discoveries

None yet. Record source-backed facts and concise deterministic evidence. Do not
record raw target IDs, endpoints, Runtime payloads, credentials, account-local
metadata, or machine-specific paths.

## Decision Log

- Decision: recovery is classified from a failure boundary and effect state,
  not from a static command property.
  Rationale: a read can have a local file side effect, and a mutation can fail
  before any CDP method is sent. The command name alone cannot distinguish
  those outcomes.
  Date/Author: 2026-07-17 / planning owner.
- Decision: no automatic method-post-dispatch retry is authorized by this
  inventory.
  Rationale: after a WebSocket send succeeds, timeout or disconnect cannot
  prove whether TradingView executed the method. Repeating even a nominal read
  sequence may repeat temporary mutation, polling setup, or local output.
  Date/Author: 2026-07-17 / planning owner.
- Decision: provisional labels in the inventory are documentation-only.
  Rationale: a public enum would be a stable agent-facing contract. The
  inventory must first prove that each meaning changes operator behavior and
  can be derived safely from existing execution evidence.
  Date/Author: 2026-07-17 / planning owner.

## Outcomes & Retrospective

Not started. At completion, state which failure/workflow families were audited,
which operator responses are distinguishable, and whether a later public
contract is promoted, deferred, or declined. A reviewed conclusion that no
public field is yet justified is a successful outcome.

## Context and Orientation

`crates/core/src/error.rs` defines the stable `ErrorKind` values and their exit
codes. `AppError` contains a kind, message, and optional details. Error kind
describes category, not execution certainty. For example, `Timeout` can occur
before a WebSocket handshake completes or after a CDP method was sent while its
response was pending.

`crates/cdp/src/transport.rs` owns HTTP target listing, target selection, target
creation, and target activation. `crates/cdp/src/client.rs` owns WebSocket
connection, method send/response waits, event waits, screenshot methods, and
input dispatch. The reviewed transport measurement slice adds public-safe
`failure_stage` details with the values `target_list`, `target_select`,
`websocket_connect`, `method_call`, `event_wait`, and `transport_unknown`.
These stages locate a failure but do not themselves authorize recovery.

`crates/cli/src/app/dispatch.rs` owns one-shot command selection.
`app/stream.rs`, `app/observe.rs`, and `app/replay_log.rs` own long-running
JSONL loops. `app/output.rs` and `app/runner.rs` own stdout/stderr behavior;
broken stdout is a successful consumer stop, not a command failure to recover.

Desktop-backed workflows fall into recurring effect families. Pure reads
include chart state, OHLCV, data, and status reads. Verified mutations include
symbol/timeframe changes, alerts, drawings, Pine operations, Replay controls,
Screener changes, and UI input with post-checks. Temporary mutations with
restoration include chart compare and selected-chart quote switching. Local
side effects include screenshots and exports. Process and target lifecycle
operations include launch and tab/Screener target creation. Diagnostics may
return partial structured success after a sub-read fails. The inventory must
cover every command arm through one of these documented archetypes.

The completed topology audit at
`docs/notes/cdp-connection-evaluation-topology-audit.md` accounts for connection
and Runtime-evaluation ownership. Reuse its workflow grouping, but independently
inspect failures, local writes, mutation dispatch, verification, and restore
paths. Do not infer recovery from call count.

## Documentation-only classification model

The inventory uses three independent columns. They are not Rust types or JSON
fields in this slice.

`dispatch_state` describes what the CLI can prove about CDP dispatch:

- `before_transport`: validation or local setup failed before target I/O.
- `before_method_dispatch`: target listing, selection, or WebSocket connection
  failed before an operation method could be sent.
- `method_dispatch_unknown`: a method send began or completed, but timeout,
  disconnect, or missing/malformed CDP response prevents proving the remote
  outcome.
- `method_response_received`: CDP returned a response and operation code can
  inspect its result. A `Runtime.evaluate` response with `exceptionDetails`
  belongs here because the response was received; whether the expression
  partially mutated page state is recorded separately as
  `remote_effect_possible`.
- `postcondition_failed`: the operation returned but its explicit verification
  did not establish the requested state.
- `restoration_failed`: a temporary mutation may remain because restoration or
  restore verification failed.

`effect_state` describes possible externally visible effects:

- `none_observed`: no CDP method or local output was committed.
- `pure_read_only`: only read methods are known to have completed.
- `remote_effect_possible`: a TradingView mutation may have occurred.
- `remote_effect_verified`: the requested mutation and its post-condition were
  verified.
- `temporary_effect_unrestored`: a temporary chart mutation may remain.
- `local_effect_possible`: a file or process effect may have occurred.
- `partial_result_available`: public-safe diagnostic or stream evidence exists
  even though a sub-operation failed.

`operator_response` is a provisional action description:

- `correct_request`: fix validation or configuration before rerunning.
- `rediscover_then_retry_candidate`: no method was dispatched; a later retry
  plan may re-discover under one budget, but this inventory does not do so.
- `reobserve_before_action`: inspect current state before deciding whether to
  repeat or clean up.
- `manual_recovery_required`: automated repetition is unsafe; follow a named
  operation-specific recovery instruction.
- `consume_partial_result`: retain partial evidence and decide whether another
  read is needed.
- `no_recovery_needed`: broken stdout or a completed verified operation needs
  no retry.
- `unclassified`: evidence is insufficient; never convert this to automatic
  retry.

The final note may refine names, but it must preserve these independent axes.
Do not collapse them into `retryable: bool` or `idempotent: bool`.

## Plan of Work

### Milestone 1: inventory shared failure boundaries

Read `crates/core/src/error.rs`, `crates/cdp/src/transport.rs`,
`crates/cdp/src/client.rs`, `crates/cdp/src/diagnostics.rs`, and the CLI app
runner/output modules. Create
`docs/notes/cdp-recovery-semantics-inventory.md` and record the audited commit,
search commands, grouping rules, and classification labels.

The shared-boundary matrix must include validation/setup, target list, target
selection, target creation/activation, WebSocket handshake, method serialization
and send, method response wait, event wait, Runtime exception/malformed result,
stdout/stderr write, and broken-pipe handling. For each boundary record existing
`ErrorKind`, `failure_stage` when present, dispatch state, effect state, and
operator response. Do not claim that `method_call` distinguishes send failure
from response timeout unless the implementation actually provides that fact.

Acceptance is that searches for error construction/mapping in the shared files
produce no unexplained production boundary.

### Milestone 2: map all command arms to workflow archetypes

Inventory Desktop-backed command arms in `crates/cli/src/app/dispatch.rs` plus
stream, observe, Replay log, readiness, status, diagnostics, launch, tab,
Screener full-page, screenshot, export, and chart-backed compare paths. Map each
arm to exactly one primary archetype and any applicable exception:

- `pure_read_one_shot`
- `pure_read_loop`
- `verified_remote_mutation`
- `temporary_mutation_with_restore`
- `local_file_output`
- `process_or_target_lifecycle`
- `partial_diagnostic`
- `unsafe_raw_input`

The durable note must list every dispatcher connection line or a reproducible
grouping rule with exact line inventory, as the topology audit did. For each
archetype, name representative commands and inspect at least one real failure
path before dispatch, after dispatch with unknown outcome, after explicit
verification failure, and after successful completion where applicable.

Input dispatch, screenshot capture, file write, process launch, target creation,
and restoration are effects even when they are not `Runtime.evaluate` calls.
An operation that catches a sub-error and returns structured success belongs to
`partial_diagnostic`, not automatic retry.

### Milestone 3: decide useful recovery distinctions

Compare the complete matrix with actual operator decisions. A provisional
recovery meaning survives only if two failures with the same existing
`ErrorKind` or `failure_stage` require materially different next actions and
the implementation can determine that distinction without raw/private data or
guessing remote outcome.

For each surviving meaning, record exact derivation evidence, affected
workflow families, unsafe counterexamples, and whether it belongs in internal
diagnostics, error details, command-specific hints, or nowhere. Do not design a
serialized enum beyond this evidence. Any public contract requires a separate
ExecPlan with additive wire examples, compatibility policy, sanitizer tests,
and agent behavior documentation.

The outcome is one of `no_public_contract`, `contract_candidate_deferred`, or
`contract_candidate_promoted`. Promotion creates a plan; it does not implement
metadata in this inventory.

### Milestone 4: synchronize and review

Update stable architecture documentation only with reviewed facts. Synchronize
`docs/v0.29-roadmap.md`, `docs/v0.29-work-items.md`, `docs/plans/README.md`,
`CHANGELOG.md`, and `CONTINUITY.md` with the same outcome. Run validation and
obtain focused independent review. Archive this plan only after the reviewer
confirms complete command grouping, dispatch/effect separation, and absence of
implicit retry authorization.

## Concrete Steps

Run from repository root. Record concise source references and counts, not raw
payloads or complete command dumps.

Shared errors and transport boundaries:

    rg -n "AppError::new|with_failure_stage|ErrorKind::|timeout_at|\.send\(|next_event|wait_for_response" crates/core/src crates/cdp/src --glob '*.rs'

CLI command and runner ownership:

    rg -n "connect_runtime\(|CdpClient::connect|CdpHttpSession::" crates/cli/src --glob '*.rs'
    rg -n "Command::|Command::name|Jsonl|BrokenPipe|write_" crates/cli/src/app crates/cli/src/cli.rs --glob '*.rs'

Effects outside Runtime evaluation:

    rg -n "dispatch_key|insert_text|dispatch_mouse|capture|write\(|spawn\(|new_target_url\(|activate_target\(|removeEntity|restore" crates/cli/src crates/cdp/src --glob '*.rs'

Focused contract checks:

    cargo test -p tradingview-cdp diagnostics -- --nocapture
    cargo test -p tradingview-cli app::output -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_output -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop -- --nocapture

If the final diff remains documentation-only, run:

    cargo metadata --no-deps --format-version 1
    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    git diff --check

If any Rust or Cargo file changes, stop and revise this plan before proceeding.
After explicit scope review, the minimum additional baseline is:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace

## Validation and Acceptance

The durable inventory must account for every shared transport/error boundary
and every Desktop-backed command arm through a reproducible grouping rule.
Failures before method dispatch, unknown outcomes after dispatch, explicit
post-condition failures, restoration failures, local side effects, and partial
diagnostic success remain distinct.

No row may infer safe retry solely from `ErrorKind`, `failure_stage`, a command
name, or nominal read-only intent. `method_dispatch_unknown` and
`restoration_failed` never map to automatic retry. Broken stdout remains normal
consumer completion. Existing error kind, message, details, exit code, JSON/
JSONL contracts, timeout, source selection, and operation behavior do not
change.

The final result names the surviving operator-response distinctions and states
whether a future public contract is promoted, deferred, or declined. Any
promoted candidate has a separate plan and no implementation in this diff.
Public hygiene and focused existing contracts are green.

## Idempotence and Recovery

This is a read-only source audit. It may be rerun without TradingView Desktop,
network access, live CDP, GUI input, file-output smoke, process launch, target
creation, or mutation. Do not apply/drop stashes or push.

If production source changes during the audit, record the new commit and rerun
all inventories. Do not combine call-site counts from different commits. If a
classification cannot prove dispatch or effect state, use `unclassified` and
do not guess.

## Artifacts and Notes

The durable artifact is `docs/notes/cdp-recovery-semantics-inventory.md`. It
contains the audited commit, search/grouping rules, shared-boundary matrix,
workflow-archetype matrix, counterexamples, candidate decision, validation,
and focused review outcome.

Do not include raw errors, Runtime values, target IDs, endpoints, local paths,
credentials, account-local metadata, source code payloads, or screenshots.

## Interfaces and Dependencies

No Rust interface or dependency is added. Existing interfaces remain:

    tradingview_core::AppError
    tradingview_core::ErrorKind
    tradingview_cdp::CdpClient
    tradingview_cdp::RuntimeEvaluator
    failure_stage in public-safe AppError details

The documentation-only classification labels in this plan are not serialized,
not exhaustive public promises, and not valid input to ordinary commands.

## Open Questions

- Which provisional operator responses survive complete inventory is
  `UNCONFIRMED`.
- Whether existing evidence can safely distinguish WebSocket send failure from
  response-wait failure is `UNCONFIRMED`; do not assume it can.
- Whether a public recovery contract changes agent behavior enough to justify
  compatibility cost is `UNCONFIRMED`.
- Pre-dispatch retry, method-post-dispatch restart, wait commands, sessions, and
  brokers remain separate and unapproved.

Revision note (2026-07-17): created after the reviewed transport measurement
and topology audit slices. It inventories failure-specific recovery semantics
without adding retry or public recovery metadata.
