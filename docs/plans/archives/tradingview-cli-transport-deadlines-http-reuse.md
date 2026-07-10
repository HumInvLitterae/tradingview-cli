# Bound transport operations and reuse HTTP clients

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while working. Maintain
this plan according to `.agents/PLANS.md`.

## Purpose / Big Picture

`tv` already gives its users bounded CDP response and event waits, but a slow
HTTP connection, a stalled response body, or a WebSocket handshake can still
wait without a clear end. Repeated Desktop-free reads also create a new HTTP
client for each request, so a batch cannot reuse an existing connection.

After this work, public TradingView HTTP reads, local CDP HTTP reads, CDP
WebSocket connection/setup, and historical-bars WebSocket setup all terminate
within defined limits. A multi-symbol Desktop-free operation uses one configured
HTTP client for its whole operation. Users see the existing success payloads and
ordered partial-result behavior; only transport timeouts become explicit
`timeout` errors instead of waiting indefinitely.

## Progress

- [x] (2026-07-10) Committed the completed Gate 2 CDP buffering work separately as `a75c88d`.
- [x] (2026-07-10) Re-read the current transport, HTTP call sites, batch callers, and Gate 3 inventory.
- [x] (2026-07-10) Created this Gate 3 ExecPlan and made it current.
- [x] (2026-07-10) Added configured HTTP clients and private client-taking helpers for sequential Desktop-free reads.
- [x] (2026-07-10) Added CDP and bars WebSocket handshake/setup/send deadlines without changing public call signatures.
- [x] (2026-07-10) Added deterministic local HTTP/WebSocket deadline and connection-reuse tests.
- [x] (2026-07-10) Ran focused transport and CLI contract tests plus the workspace baseline; opt-in live smokes remain intentionally skipped.
- [x] (2026-07-10) Recorded outcomes as `implemented; independent review pending`; do not start Gate 4 or commit Gate 3.
- [x] (2026-07-11) Independent review found three medium implementation/test gaps and one local-ledger drift; the overall deadline design was approved.
- [x] (2026-07-11) Preserved bars diagnostics on heartbeat pong timeout, reused one CDP HTTP session through the Screener fallback, added an operation-level multi-symbol reuse fixture, and added effective scanner/Pine HTTP timeout tests.
- [x] (2026-07-11) Re-ran focused tests, strict clippy, and the full workspace baseline after review corrections; all non-ignored tests pass.
- [x] (2026-07-11) Focused independent re-review approved the four corrections with no remaining findings.

## Surprises & Discoveries

- Observation: client construction was split across `tradingview-market`,
  `tradingview-scanner`, `tradingview-pine`, `tradingview-cdp`, and a few CLI
  CDP activation/version paths.
  Evidence: the pre-Gate-3 source used `reqwest::Client::new()` or
  `reqwest::get()` at those boundaries. The implementation now owns a
  configured client in each HTTP-owning crate and passes it through sequential
  multi-read helpers.

- Observation: `tv launch` probes CDP version at a one-second cadence for 15
  attempts, so an unbounded probe could extend the intended readiness window.
  Evidence: `LAUNCH_READY_ATTEMPTS` is 15 and `LAUNCH_READY_DELAY` is one
  second in `crates/cli/src/ops/launch.rs`.

- Observation: the first review found that the heartbeat pong send was the only
  bars send-timeout branch that returned without `bars_error_details`.
  Evidence: the new pending-pong test now checks `source_availability`,
  `websocket_send_timeout`, bar count, wait summary, and range fetch summary.

- Observation: a low-level keep-alive test was insufficient evidence that a
  top-level multi-symbol operation reused its client.
  Evidence: the quote fixture now runs the actual ordered two-symbol operation,
  observes two scanner requests over one accepted TCP connection, and checks
  requested/result order.

## Decision Log

- Decision: use separate deadline policies for public TradingView HTTP, local
  CDP HTTP, CDP WebSocket connection, and bars WebSocket connection.
  Rationale: public endpoints may require DNS/TLS and a response body, while
  the CDP endpoint is normally local and launch polls it every second. One
  arbitrary global timeout would either make local failure reporting slow or
  make public reads unnecessarily fragile.
  Date/Author: 2026-07-10 / Codex.

- Decision: public HTTP uses a five-second connection deadline and a
  fifteen-second total request deadline; CDP HTTP uses one and three seconds;
  CDP WebSocket connection uses five seconds; bars setup phases reuse the
  existing ten-second bars request timeout.
  Rationale: the bars and CDP receive paths already establish ten-second
  bounded waits. The shorter CDP HTTP policy keeps launch readiness bounded;
  the public HTTP policy leaves room for a normal remote response while still
  covering a stalled body.
  Date/Author: 2026-07-10 / Codex.

- Decision: preserve public function signatures and add private client-taking
  helpers in market code. Add an opaque, additive `CdpHttpSession` only where
  the CLI needs several CDP HTTP calls in one top-level operation.
  Rationale: callers keep their current Rust API, while batches, snapshots,
  compare packets, events compare, tabs, Screener setup, and launch polling can
  reuse a connection pool without exposing `reqwest` through the core crate.
  Date/Author: 2026-07-10 / Codex.

- Decision: map only `reqwest` timeout errors to `ErrorKind::Timeout`; preserve
  current non-timeout error kinds and status handling.
  Rationale: Gate 3 makes waiting finite. Gate 5 owns the broader error-taxonomy
  decision and must not be preempted by this implementation.
  Date/Author: 2026-07-10 / Codex.

## Outcomes & Retrospective

Implementation, correction, validation, and independent review are complete.
The first review approved the deadline design and identified four narrow
corrections; focused re-review approved those corrections with no remaining
findings. Public TradingView HTTP clients use a five-second connection
timeout and fifteen-second total deadline. Local CDP HTTP sessions use one and
three seconds, CDP WebSocket handshakes use five seconds, and bars WebSocket
connection, setup, read-side sends, and cleanup use the existing ten-second
bars request deadline. Local fixtures cover stalled HTTP headers and bodies,
stalled CDP and bars WebSocket handshakes, a pending WebSocket send, and
keep-alive reuse for configured public and CDP HTTP clients. The change keeps
public success payloads and function signatures intact; only actual transport
timeouts now map to the existing timeout error.

This plan is ready to archive and commit. Gate 4 may receive its own ExecPlan
only after this Gate 3 commit is complete.

## Context and Orientation

`tradingview-market`, `tradingview-scanner`, and `tradingview-pine` perform
credential-free public HTTP reads. `tradingview-cdp` discovers local TradingView
Desktop targets through CDP HTTP and then connects a WebSocket to a selected
target. `tradingview-market` uses a separate public WebSocket path for
historical bars. A `reqwest::Client` owns an HTTP connection pool; reusing the
same client means sequential requests to the same server can reuse an existing
connection without changing how many requests the command makes.

`ErrorKind::Timeout` already maps to exit code 4. The existing command payloads
and their source metadata must not change. Raw target identifiers, raw frames,
response bodies, credentials, and local absolute paths must not enter new error
details, fixtures, or tracked docs.

The current market batch, compare, snapshot, and events-compare paths are
sequential. This plan keeps that order and adds no concurrency. `tv chart
compare` remains excluded because it changes and restores selected-chart state.

## Plan of Work

Create crate-local configured HTTP client builders. They set the connection and
total request deadlines and map builder failure to an internal error. HTTP send
or JSON-body errors map to `Timeout` only when `reqwest` identifies the error
as a timeout; every other mapping remains the existing mapping for that call
site. Do not set a read timeout because it resets after every successful read;
the total request deadline is the intended hard bound.

In `tradingview-market`, add private `*_with_client` helpers for symbol search,
scanner quote, fundamentals, information, events, snapshot, compare, and bars
bare-symbol resolution. Public wrappers construct one client, then pass it
through all sequential reads in that top-level operation. This includes fallback
symbol searches after scanner validation errors. Scanner and Pine single-read
operations use their crate-local configured client once per operation.

In `tradingview-cdp`, introduce `CdpHttpSession`, an opaque client plus cloned
`TransportConfig`. It exposes target list, target creation, target activation,
and version-read methods. Existing free functions remain and construct a
one-operation session for compatibility. CLI tab workflows, full-page Screener
workflows, and launch readiness reuse one session across repeated CDP HTTP
calls. Launch keeps its intended fifteen-second readiness window by applying an
overall deadline and a one-second maximum probe request.

Wrap `CdpClient::connect` in a five-second handshake timeout. Start the
existing ten-second CDP request deadline before sending a method request and
use that same deadline for send and matching-response wait. Keep `call_method`
and `next_event` signatures unchanged.

For bars, wrap the WebSocket handshake in the existing ten-second request
timeout. Give all initial session-setup sends one shared ten-second deadline.
Use the existing read-loop deadline for pongs and `request_more_data` sends.
Use a final ten-second best-effort cleanup deadline for chart-delete and close.
Timeout failures must retain the bars source diagnostics and use a public-safe
unavailable reason that distinguishes connection timeout from send timeout.

## Concrete Steps

Work from the repository root.

1. Add configured-client and timeout-mapping helpers, then refactor callers to
   pass one client through every sequential multi-read operation.

2. Add `CdpHttpSession`, migrate the multi-call CLI CDP workflows, then bound
   CDP WebSocket connect/send.

3. Add bars handshake/setup/read/cleanup deadline helpers without changing the
   protocol framing or bar payload shaping.

4. Add local test fixtures that accept TCP connections, delay HTTP or WebSocket
   progress, and count accepted connections. Use shortened injected durations
   in tests; production constants remain the policies recorded above.

5. Update this plan, `docs/v0.26-roadmap.md`, `docs/v0.26-work-items.md`,
   `docs/architecture.md`, `docs/plans/README.md`, `CHANGELOG.md`, and the
   local `CONTINUITY.md` as evidence becomes available.

## Validation and Acceptance

Run these commands from the repository root and expect every non-ignored test
to pass:

    cargo test -p tradingview-market http -- --nocapture
    cargo test -p tradingview-market quote -- --nocapture
    cargo test -p tradingview-market bars -- --nocapture
    cargo test -p tradingview-scanner http -- --nocapture
    cargo test -p tradingview-pine http -- --nocapture
    cargo test -p tradingview-cdp transport -- --nocapture
    cargo test -p tradingview-cdp client -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_bars -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

The deterministic fixtures must prove that a stalled HTTP header/body and a
stalled WebSocket handshake terminate as `Timeout`; a pending WebSocket send
does not wait forever; a heartbeat pong timeout retains full bars diagnostics;
and at least one actual multi-symbol Desktop-free operation and one repeated
CDP HTTP operation reuse one keep-alive connection. Existing
success JSON, ordered item results, source metadata, and request counts must
remain unchanged.

Optional read-only smokes may use `tv search AAPL`, `tv compare AAPL MSFT`,
`tv bars AAPL --timeframe 1D --count 5`, and `tv status`. Record only command,
source, and success/timeout status in tracked docs.

## Idempotence and Recovery

All deterministic fixtures use local loopback sockets and synthetic protocol
messages. They are safe to rerun. If client reuse would require caching,
concurrency, a global singleton, or a public contract change, stop and record
the blocker rather than widening this gate. Do not use destructive Git commands
or create a Gate 4 plan before independent review is complete.

## Interfaces and Dependencies

Keep `CdpClient::connect`, `CdpClient::call_method`, `CdpClient::next_event`,
all current `tradingview-market` public functions, and CLI command payloads
compatible. `CdpHttpSession` is additive and opaque. Use only existing
`reqwest`, Tokio, and tokio-tungstenite dependencies; Tokio dev features may be
enabled for local fixtures, but no new crate dependency is allowed.

Revision note: created on 2026-07-10 after Gate 2 was independently reviewed,
its documentation correction was committed, and Gate 3 became the active gate.
