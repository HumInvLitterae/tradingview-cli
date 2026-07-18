# CDP recovery-semantics inventory

Status: source inventory completed on commit `4f63bec`; focused independent
inventory review pending. The labels below are documentation-only and are not
public JSON or Rust contracts.

## Scope and grouping

This inventory covers shared errors and all Desktop-backed workflows under
`crates/core/src`, `crates/cdp/src`, and `crates/cli/src`. It reuses the exact
75 dispatcher connection lines and three runner owners recorded in
`docs/notes/cdp-connection-evaluation-topology-audit.md`. Each dispatcher line
maps through one command family below; conditional read/set arms name both the
primary effect and the read-mode exception. Test modules are excluded at their
`mod tests` boundary.

The source searches were:

    rg -n "AppError::new|with_failure_stage|ErrorKind::|timeout_at|\.send\(|next_event|wait_for_response" crates/core/src crates/cdp/src --glob '*.rs'
    rg -n "connect_runtime\(|CdpClient::connect|CdpHttpSession::" crates/cli/src --glob '*.rs'
    rg -n "Command::|Command::name|Jsonl|BrokenPipe|write_" crates/cli/src/app crates/cli/src/cli.rs --glob '*.rs'
    rg -n "dispatch_key|insert_text|dispatch_mouse|capture|write\(|spawn\(|new_target_url\(|activate_target\(|removeEntity|restore" crates/cli/src crates/cdp/src --glob '*.rs'

## Shared failure-boundary matrix

| Boundary | Existing classification | Dispatch state | Effect state | Provisional operator response |
| --- | --- | --- | --- | --- |
| CLI validation / source-file read / transport-config parse | usually `Validation`, no `failure_stage` | `before_transport` | `none_observed` | `correct_request` |
| HTTP client construction | `Internal`, no known stage | `before_transport` | `none_observed` | `correct_request` or `unclassified` |
| Target-list connect, timeout, status, or payload failure | `Connection`, `Timeout`, or `InternalApiUnavailable`; `target_list` | `before_method_dispatch` | `none_observed` | `rediscover_then_retry_candidate` |
| Target selection none or ambiguity | `Connection` or `TargetAmbiguous`; `target_select` | `before_method_dispatch` | `none_observed` | `correct_request` or `rediscover_then_retry_candidate` |
| WebSocket URL missing / handshake failure | `Connection` or `Timeout`; `websocket_connect` | `before_method_dispatch` | `none_observed` | `rediscover_then_retry_candidate` |
| Target creation / activation HTTP send, status, or payload failure | HTTP error without a precise public dispatch marker | `method_dispatch_unknown` | `remote_effect_possible` | `reobserve_before_action` |
| CDP request send timeout or sink error | `Timeout` or `Connection`; `method_call` | `method_dispatch_unknown` | depends on method; conservatively `remote_effect_possible` | `reobserve_before_action` |
| CDP response wait timeout / disconnect | `Timeout` or `Connection`; `method_call` | `method_dispatch_unknown` | depends on method; conservatively `remote_effect_possible` | `reobserve_before_action` |
| CDP protocol error response | `InternalApiUnavailable`; `method_call` | `method_response_received` | method-specific, possibly `remote_effect_possible` | `reobserve_before_action` or `manual_recovery_required` |
| `Runtime.evaluate` `exceptionDetails` | `InternalApiUnavailable`; response was received | `method_response_received` | `remote_effect_possible` because expression progress is unknown | `reobserve_before_action` |
| Runtime result shape or explicit verification mismatch | operation-specific `InternalApiUnavailable` | `postcondition_failed` | read-only or `remote_effect_possible` by workflow | `reobserve_before_action` |
| Restore setter/evaluation/readback failure | operation-specific failure | `restoration_failed` | `temporary_effect_unrestored` | `manual_recovery_required` |
| Event wait timeout / disconnect with no new method in the wait | `Timeout` or `Connection`; `event_wait` | inherited from prior subscription/setup | `remote_effect_possible` or `partial_result_available` | `reobserve_before_action` |
| Screenshot capture failure before bytes return | CDP method error | `method_dispatch_unknown` | `none_observed` locally | `reobserve_before_action` |
| File create/write failure after bytes return | local I/O error | `method_response_received` | `local_effect_possible` | `manual_recovery_required` |
| Stdout broken pipe | successful `BrokenPipe` disposition | response already produced | `local_effect_possible` only at consumer boundary | `no_recovery_needed` |
| Other stdout serialization/write failure | `Internal`, exit 1 | response already produced | `local_effect_possible` | `manual_recovery_required` |
| Nonterminal JSONL sample failure | application error emitted to stderr | operation-specific | `partial_result_available` | `consume_partial_result` |
| Stderr broken pipe for nonterminal JSONL error | suppressed line | operation-specific | `partial_result_available` | `consume_partial_result` |

The current `method_call` stage wraps both `send_message_until` and
`wait_for_response`. It cannot prove whether bytes were transmitted or whether
TradingView executed the method. No recovery contract may map `method_call`
directly to retry.

A Runtime exception is not a dispatch-unknown state: CDP returned a response.
The uncertainty belongs to the effect axis because the expression may have
performed some work before throwing.

The method-wait rows also include pending-event queue overflow and invalid CDP
JSON received while waiting for a response. Both occur after method dispatch
and therefore remain `method_dispatch_unknown`; overflow is
`InternalApiUnavailable` and malformed framing is `Connection`. The
post-response payload-shape row includes screenshot responses whose data is
missing or cannot be decoded: dispatch is known, no local file has yet been
written, and the malformed result must not be retried automatically.

## Workflow-archetype matrix

| Archetype | Complete command-family mapping | Failure/effect exception | Safe response shape |
| --- | --- | --- | --- |
| `pure_read_one_shot` | state, current info, values, discover, ui-state, OHLCV, chart reads, watchlist get, alert list, drawing/data reads, Pine errors/console/list/get, pane/layout lists, Replay status, quote/chart quote/quote-data, readiness/status | Runtime expressions can still throw after response; quote/chart workflows may use temporary switching and therefore move to the restore archetype | pre-dispatch candidate only; post-dispatch reobserve |
| `pure_read_loop` | stream, observe, Replay log reads and OHLCV attachments, screenshot render wait | prior samples remain useful; per-sample error may be nonterminal | consume partial result; do not restart loop automatically |
| `verified_remote_mutation` | symbol/timeframe/type/range/scroll setters, watchlist mutations, alerts, indicator add/remove/toggle/set, drawings, Pine set/new/open/save/compile, pane/layout changes, Replay controls/trades, Screener dialog/filter/column/screen changes | method outcome may be unknown; post-check failure differs from transport failure | reobserve before action; manual recovery when operation-specific |
| `temporary_mutation_with_restore` | chart compare, selected-chart quote switching, Screener read-open/restore helpers, visible-range operations with restoration obligations | restore failure may leave selected chart or UI state changed | manual recovery required; never automatic retry |
| `local_file_output` | screenshot output, chart-bar export | capture/read may succeed before local file failure; a partial file may exist | inspect/remove local output before rerun |
| `process_or_target_lifecycle` | launch, tab switch/new/close, Screener full-page target create/activate | process or target may exist even when response/verification fails | reobserve process/target inventory before action |
| `partial_diagnostic` | diagnose, readiness, status best-effort subreads, Replay attachment error continuation | structured success or prior events remain available | consume partial result |
| `unsafe_raw_input` | explicit UI eval, keyboard, type, click, scroll, selector-driven input | focus and target state can change; some paths intentionally lack a semantic post-check | reobserve before action or manual recovery |

Conditional command arms are completely covered as follows. Symbol, timeframe,
chart type, and visible range use `pure_read_one_shot` without a requested
value and `verified_remote_mutation` with one. Quote `auto` is a source-policy
fallback, not recovery: a pre-connect chart failure may select the existing
scanner path, but no post-dispatch chart result is retried. Screener command
dispatch uses read, mutation, restoration, or lifecycle archetypes according to
the selected subcommand. UI command dispatch uses `unsafe_raw_input` except
closed read-only selector/DOM queries, which use `pure_read_one_shot`.

The topology audit's exact dispatcher line inventory and the command families
above account for all 75 lines. The three runner owners map to
`pure_read_loop`. Direct readiness, status, diagnostics, launch, tab, Desktop,
and Screener target owners map to the named exception archetypes and are not
silent retries.

## Counterexamples to command-level retryability

- Screenshot is primarily a read, but rerunning after a local write failure can
  overwrite or coexist with a partial file.
- Chart compare reads evidence, but it temporarily changes and restores chart
  state; restore failure needs manual inspection.
- Replay log advances Replay before attaching optional read evidence. An
  attachment failure must not repeat the step.
- Pine compile/save and drawing/alert operations may mutate before a response or
  post-check fails.
- Launch can start a process even when CDP readiness remains unavailable.
- Stream and observe retain prior samples; restarting changes the observation
  window and can duplicate events.
- Broken stdout is successful consumer completion and must not produce another
  attempt.

## Candidate decision

Outcome: `contract_candidate_deferred`.

The inventory confirms that materially different operator responses exist:
correcting input, reconsidering a pre-dispatch connection, consuming partial
evidence, reobserving uncertain remote state, and performing operation-specific
manual recovery are not interchangeable. A static `idempotent` or `retryable`
flag would be incorrect.

A public recovery field is not yet safely derivable. Existing `failure_stage:
method_call` combines send and response-wait failures. Effect state is known
only inside operation-specific code, and many failures need command-specific
instructions rather than one generic enum. Publishing the provisional labels
now would imply certainty the runtime does not possess.

No public contract is promoted in this slice. A future candidate requires
separate evidence for dispatch/effect markers, additive wire examples,
sanitization, compatibility, and agent behavior. Pre-dispatch retry remains a
separate deferred candidate triggered by observed target-list or WebSocket
connection failures; this inventory does not authorize it.
