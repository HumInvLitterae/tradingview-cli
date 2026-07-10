# Preserve interleaved CDP events and enforce absolute wait deadlines

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while working. Maintain
it in accordance with `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, a Chrome DevTools Protocol event that arrives while `tv` is
waiting for a method response is retained and returned by the next event read.
This closes a race where `Network.enable` could receive quote-data evidence
before its response and silently discard that evidence. CDP response waits and
event waits also use one absolute deadline, so unrelated traffic cannot extend
a nominal timeout indefinitely.

The observable proof is a local WebSocket fixture that sends events before a
method response and later receives those events in order. Separate fixtures
keep sending unrelated traffic beyond a short deadline and still receive the
existing timeout error. CLI payloads and public Rust method signatures remain
unchanged.

## Progress

- [x] (2026-07-10) Completed and independently reviewed Gate 1 broken-pipe handling.
- [x] (2026-07-10) Re-read the current CDP client, quote-data consumer, roadmap, and prior review evidence.
- [x] (2026-07-10) Created this Gate 2 ExecPlan and made it current.
- [x] (2026-07-10) Added a FIFO queue bounded to 1024 events and 8 MiB of encoded event text.
- [x] (2026-07-10) Routed events received during method-response waits into the queue.
- [x] (2026-07-10) Made method-response and event waits use one absolute deadline each.
- [x] (2026-07-10) Added deterministic local WebSocket, queue-limit, FIFO, and deadline tests.
- [x] (2026-07-10) Ran focused tests, full workspace validation, repeated deadline tests, and a read-only smoke.
- [x] (2026-07-10) Recorded implementation outcomes and left Gate 2 open for independent review.
- [x] (2026-07-10) Completed independent read-only review; corrected the reported documentation drift and marked Gate 2 complete.

## Surprises & Discoveries

- Observation: `quote_data_bounded_read` calls `Network.enable` before creating
  its observer loop, so an event arriving before that method response is the
  concrete race affected by the current discard behavior.
  Evidence: `crates/cli/src/ops/market/quote_data.rs` calls `call_method` and
  only then begins calling `next_event`.

- Observation: the existing method-response and event loops each wrap every
  individual `stream.next()` in a fresh timeout.
  Evidence: both loops in `crates/cdp/src/client.rs` use
  `tokio::time::timeout(timeout, ...)` inside `loop`.

- Observation: the connected client can preserve interleaved events without a
  background reader or public API change.
  Evidence: the local WebSocket fixture sends two events before a matching
  response; `call_method` succeeds and two zero-wait `next_event` calls return
  those events in original order.

- Observation: short real-time deadline fixtures are stable with a generous
  upper-bound assertion.
  Evidence: both 50 ms deadline tests passed in the full workspace suite and
  in 20 consecutive focused repetitions; each completed near 50 ms and below
  the 200 ms guard.

- Observation: the read-only Desktop smoke required explicit target selection
  because three chart targets were open.
  Evidence: a locally selected target reported chart symbol `BATS:ELVN` and
  the quote-data diagnostic succeeded with source `desktop_quote_data_ws`.
  No target identifier or raw payload was retained.

## Decision Log

- Decision: preserve the sequential `&mut CdpClient` model.
  Rationale: event loss and timeout extension can be fixed inside the current
  client without concurrent requests or a background reader task.
  Date/Author: 2026-07-10 / Codex.

- Decision: bound retained events by both count and approximate encoded size:
  1024 events and 8 MiB.
  Rationale: a count-only limit does not bound memory when an event contains a
  large payload. These limits are generous for the short response-wait window
  while still preventing unbounded retained state.
  Date/Author: 2026-07-10 / Codex.

- Decision: fail explicitly on queue overflow instead of dropping the oldest
  or newest event.
  Rationale: silent eviction could recreate false-unavailable behavior. The
  failure uses `InternalApiUnavailable` and public-safe numeric diagnostics,
  without including the event or raw payload.
  Date/Author: 2026-07-10 / Codex.

- Decision: keep WebSocket send and connection establishment outside this
  slice's deadline.
  Rationale: Gate 2 fixes already-connected receive loops. HTTP and WebSocket
  connection deadlines belong to Gate 3.
  Date/Author: 2026-07-10 / Codex.

- Decision: estimate retained event bytes from the original CDP text message
  length and decrement that accounting when the FIFO entry is removed.
  Rationale: the original text length is available without reserializing the
  parsed value and gives a deterministic safety bound alongside the event-count
  limit. It is an encoded-size guard, not an exact heap allocator measurement.
  Date/Author: 2026-07-10 / Codex.

## Outcomes & Retrospective

Implementation, local validation, and independent read-only review are
complete. `CdpClient` now parses each text message once, retains interleaved
events in a bounded FIFO, and returns queued events before reading the socket.
Queue overflow is explicit and public-safe rather than silently evicting an
event. Both receive loops use a single `timeout_at` deadline while preserving
their existing timeout messages and error kinds.

The CDP focused tests, quote-data tests, CLI quote and diagnose contracts,
strict clippy, full workspace suite, metadata, formatting, diff checks, and
package-script syntax are green. The two deadline tests also passed 20 repeated
runs. A read-only Desktop smoke succeeded for the selected chart and retained
no raw event or target identifier. Independent review found no runtime issue;
its documentation-drift finding was corrected before Gate 2 was marked
complete. Connection establishment and WebSocket send deadlines remain
intentionally deferred to Gate 3.

## Context and Orientation

`crates/cdp/src/client.rs` owns the connected TradingView Desktop CDP
WebSocket. `CdpClient::call_method` sends a JSON request with an incrementing
numeric ID and waits for the matching response. `CdpClient::next_event` reads
CDP messages whose JSON object has a string `method` field. Both APIs are used
sequentially through `&mut CdpClient`.

Before Gate 2, `wait_for_response` parsed each text message only to look for
the target response ID. A method event had no matching response ID and was
discarded, so `next_event` could not recover it later. Both receive loops also
restarted their timeout after each ignored message.

The primary consumer affected by event loss is
`crates/cli/src/ops/market/quote_data.rs`. It enables the Network domain and
then observes `Network.webSocketFrameReceived` and
`Network.webSocketFrameSent` events for a bounded 3.5-second read. The
quote-data payload must not change in this slice.

## Plan of Work

Add a private pending-event container in `crates/cdp/src/client.rs`. It owns a
`VecDeque` of parsed JSON events and tracks their original text lengths. It
provides FIFO push and pop operations, subtracting retained bytes when an event
is removed. Production limits are 1024 entries and 8 MiB; tests may construct
smaller limits. A push that would exceed either limit returns a public-safe
`InternalApiUnavailable` error with the reason, current count and bytes,
limits, incoming event bytes, and `raw_event_included: false`.

Classify each incoming WebSocket message once. A valid text object with a
string `method` is an event and retains its encoded text length. Other valid
text is a possible response. Binary, ping, pong, and frame messages are
ignored; close and invalid JSON retain their existing errors.

Move response waiting into `CdpClient` so it can enqueue interleaved events.
After sending a request, compute one `tokio::time::Instant` deadline from the
existing 10-second timeout. Each receive uses `timeout_at` with that same
deadline. Matching success and CDP error responses keep their current behavior;
different response IDs remain ignored.

Make `next_event` pop an already queued event before computing a deadline or
reading the socket. When the queue is empty, compute one deadline from the
caller-supplied duration and use it for the entire loop. Events read directly
from the socket are returned immediately. Ignored messages cannot reset the
deadline.

Add local WebSocket tests in the CDP crate using the existing
`tokio-tungstenite` dependency. Add a dev-only Tokio feature declaration for
the test runtime, not a new crate. Test event-before-response ordering,
event-before-error-response retention, queued immediate reads, both absolute
deadline paths, and both queue limits. Use only synthetic protocol messages.

Run existing quote-data and CLI contract tests to prove payload and exit-code
compatibility. Update `docs/architecture.md` with the stable internal behavior,
then update this plan, roadmap, work inventory, changelog, and local continuity
ledger. After independent review and any resulting corrections, mark Gate 2
complete before creating the Gate 3 plan.

## Concrete Steps

Work from the repository root. The implementation should primarily change
`crates/cdp/src/client.rs`, with a dev-only Tokio test feature declaration in
`crates/cdp/Cargo.toml` if required.

Run focused validation:

    cargo test -p tradingview-cdp client -- --nocapture
    cargo test -p tradingview-cli ops::market::quote_data -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_diagnose -- --nocapture

Then run the workspace baseline:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

If TradingView Desktop is available, an optional non-mutating smoke may run
`target/debug/tv diagnose quote-data <EXCHANGE:SYMBOL>`. Record only status and
contract-level summary, never raw CDP events or WebSocket payloads.

## Validation and Acceptance

Acceptance requires all of the following:

- events received before a method response are returned later in original order;
- events received before a matching CDP error response are also retained;
- queued events are returned immediately, including with a zero wait duration;
- unrelated traffic cannot extend method-response or event waits beyond one deadline;
- queue overflow is explicit, bounded, and public-safe;
- existing CDP method error, timeout, invalid JSON, and closed-connection mappings remain unchanged;
- quote-data success and unavailable payloads, source metadata, and bounded wait remain unchanged;
- focused tests and the workspace baseline are green.

## Idempotence and Recovery

All tests use local or synthetic inputs and are safe to rerun. Do not use live
account data in fixtures. If the queue cannot be integrated without concurrent
CDP calls or a background reader, stop and report that architectural blocker
instead of widening the implementation. Do not use destructive Git commands.

## Interfaces and Dependencies

Keep `CdpClient::connect`, `CdpClient::call_method`, `CdpClient::next_event`,
and `RuntimeEvaluator` signatures unchanged. Keep queue types, message
classification, limits, and test constructors private to `tradingview-cdp`.
Use only the standard library, current Tokio, current serde/serde_json, and
current tokio-tungstenite dependencies.

Revision note: created on 2026-07-10 after Gate 1 implementation, validation,
and independent review completed without findings. Revised on 2026-07-10 after
implementation and local validation to record green evidence, then revised
again after independent review to record the documentation-drift correction and
Gate 2 completion.
