# Add actionable transport diagnostics to Desktop-free bars

This ExecPlan is a living document. The sections `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to
date as work proceeds.

Maintain this document in accordance with `.agents/PLANS.md`.

## Purpose / Big Picture

When `tv bars` fails today, its structured error usually identifies the broad
error kind and source availability but may not say whether the failure occurred
while opening the WebSocket, setting up the chart session, waiting for source
data, parsing a protocol message, or requesting another history window. During
the v0.31 one-minute range smoke, the first attempt returned only
`kind: "connection"`; a later bounded comparison showed the feature and common
transport working. The coarse error forced unnecessary discussion before the
failure could be classified as transient.

After this change, Desktop-free bars failures will preserve their existing
error kind, message, details, and exit code while adding one public-safe
`source_failure_stage` value. An agent can report where the failure surfaced
and choose whether to correct input, re-observe source health, or stop for
operator review. This plan does not add automatic retry, reconnect, fallback,
timeout changes, shared sessions, or background work.

## Progress

- [x] (2026-07-27) Recorded the observed transient `connection` failure and
  successful bounded comparison in the v0.31 roadmap and work inventory.
- [x] (2026-07-27) Inspected the bars facade, request validation, symbol
  resolution, WebSocket setup, response loop, protocol parser, pagination,
  heartbeat, source-availability details, and existing CDP diagnostics.
- [x] (2026-07-27) Created this self-contained ExecPlan with a closed
  source-stage vocabulary and deterministic acceptance boundary.
- [ ] Obtain focused plan review.
- [ ] Implement the typed stage mapping and deterministic fixtures.
- [ ] Synchronize public docs and runtime guidance.
- [ ] Run focused and complete non-live validation.
- [ ] Obtain focused implementation review and archive the plan.

## Surprises & Discoveries

- Observation: the initial one-minute failure did not implicate the new
  timeframe contract.
  Evidence: existing five-minute recent and date-range reads and all three
  intended one-minute scenarios succeeded in the subsequent bounded
  production-binary comparison.

- Observation: bars already exposes useful source-availability and bounded-wait
  fields, but the top-level failure location remains ambiguous.
  Evidence: `crates/market/src/bars/transport.rs` maps connection, send, read,
  protocol, heartbeat, and pagination errors through `bars_error_details`
  without a stable stage field.

- Observation: the existing public `failure_stage` field belongs to Desktop
  CDP transport and has a reviewed closed vocabulary.
  Evidence: `crates/cdp/src/diagnostics.rs`, `README.md`, and
  `packaging/agent/AGENTS.md` define target-list, target-selection, CDP
  WebSocket, method-call, and event-wait stages.

## Decision Log

- Decision: add `source_failure_stage`, not new values to the existing CDP
  `failure_stage`.
  Rationale: Desktop-free bars uses a different source and lifecycle. Keeping
  the fields separate preserves the existing CDP contract and makes ownership
  explicit.
  Date/Author: 2026-07-27 / Codex

- Decision: use a closed typed vocabulary:
  `symbol_search`, `request_prepare`, `websocket_connect`, `session_setup`,
  `series_setup`, `response_wait`, `protocol`, `heartbeat_send`,
  `pagination_send`, `source_result`, and `source_unknown`.
  Rationale: each value corresponds to a materially different observed
  boundary while remaining free of endpoint, symbol, method-name, and payload
  data. `source_unknown` is a fail-closed fallback, not permission to retry.
  Date/Author: 2026-07-27 / Codex

- Decision: stage attribution is diagnostic only.
  Rationale: one transient failure does not justify retry policy. A later
  recovery plan must use repeated evidence and must distinguish work that was
  not dispatched from work whose remote outcome may be unknown.
  Date/Author: 2026-07-27 / Codex

- Decision: deterministic fault injection is the acceptance gate.
  Rationale: a live endpoint cannot reliably produce each failure family on
  demand. Unit and contract fixtures can prove exact mapping, preservation,
  sanitization, and call counts without extra network traffic.
  Date/Author: 2026-07-27 / Codex

## Outcomes & Retrospective

Planning is complete. No production implementation, public field, retry,
network probe, dependency, version, workflow, tag, push, or release operation
has been authorized or performed by this plan.

## Context and Orientation

`tv bars` is a Desktop-free bounded historical-bars command. Public facade
functions `bars_symbol` and `bars_symbol_range` live in
`crates/market/src/bars.rs`. They validate input, optionally resolve a bare
symbol through Desktop-free search, build a `BarsRequest`, call
`fetch_bars_ws`, reject an empty result through `no_bars_error`, and shape a
`bars.v1` success payload.

`crates/market/src/bars/transport.rs` owns the undocumented TradingView
WebSocket lifecycle. It prepares the request, connects, sends chart-session
setup messages, resolves the symbol inside that session, creates the series,
waits for messages, parses protocol frames, answers heartbeat pings, requests
older data, and returns a bounded result. A "stage" in this plan means one of
those locally owned phases where an error surfaced. It does not prove whether
TradingView applied a message before a send or response failure.

`crates/market/src/bars/payload.rs` owns `bars_error_details`,
`bars_source_availability`, and `no_bars_error`. Existing details include the
source contract, request mode, public symbol readback, availability state,
wait summary, and range-fetch summary. New stage data must be merged into these
details rather than replacing them.

`crates/core/src/error.rs` defines `AppError`, `ErrorKind`, and stable exit-code
mapping. This plan must not add an error kind or change exit codes.

Desktop CDP diagnostics live separately in
`crates/cdp/src/diagnostics.rs`. Its `failure_stage` field must not be changed
or reused for bars.

## Plan of Work

### Milestone 1: inventory every bars failure boundary

Create a durable source matrix in this living plan or a concise note under
`docs/notes/`. Account for every production `AppError` construction and
propagation in `crates/market/src/bars.rs`,
`crates/market/src/bars/transport.rs`,
`crates/market/src/bars/protocol.rs`, and the symbol-search call used by the
bars facade. For each row, record the existing kind, message family,
availability reason, whether any bars may already exist, and the exact new
stage.

The required mapping is:

- bare-symbol search transport or response failure: `symbol_search`;
- WebSocket request conversion before network access: `request_prepare`;
- connection or handshake timeout/failure: `websocket_connect`;
- authentication and chart-session setup sends: `session_setup`;
- in-session symbol resolution and initial series creation sends:
  `series_setup`;
- WebSocket read error, close, bounded wait timeout, or no-message/no-bars
  result: `response_wait`;
- malformed frame, protocol parse failure, or provider
  symbol/series/protocol error: `protocol`;
- heartbeat pong send failure: `heartbeat_send`;
- `request_more_data` send failure: `pagination_send`;
- a responsive completed result that contains no bars: `source_result`;
- any genuinely unclassified source boundary: `source_unknown`.

Do not classify validation errors because they occur before source access.
Cleanup failures remain intentionally ignored by the existing best-effort
cleanup contract and do not replace the primary outcome.

Acceptance for this milestone is a fresh `rg` inventory with no unexplained
production boundary.

### Milestone 2: add typed source-stage mapping

Add a private enum in the bars module, named `BarsFailureStage` unless the
existing module layout shows a more coherent private location. Implement an
`as_str()` mapping for the exact closed vocabulary above. Do not serialize the
enum directly and do not expose a new public Rust type.

Add one helper that merges
`"source_failure_stage": stage.as_str()` into a bars details object while
preserving every existing key. It must fail closed to `source_unknown` only
when a caller has no reviewed stage; normal production call sites must pass an
explicit variant.

Apply the helper at the owning boundary, before a higher layer loses the
location information. Preserve the original `AppError.kind`,
`AppError.message`, existing details, and exit code. Do not include the
WebSocket endpoint, requested or resolved symbol inside the new field, method
arguments, protocol payload, raw frame, credential, or dependency error text.
Existing safe symbol fields in `bars_error_details` remain unchanged.

The public error example should look like:

    {
      "success": false,
      "command": "bars",
      "error": {
        "kind": "connection",
        "message": "TradingView WebSocket connection failed: ...",
        "details": {
          "contract_version": "bars.v1",
          "source": "tradingview_bars_ws",
          "source_failure_stage": "websocket_connect"
        }
      }
    }

The example illustrates placement only. Tests and tracked evidence must use
fixed sanitized messages and must not retain a live dependency error suffix.

### Milestone 3: prove preservation and sanitization deterministically

Add deterministic tests close to the owning code. Reuse existing fake sink,
protocol, payload, and facade fixtures rather than creating a test-only
production API.

Tests must prove:

- every enum variant maps to its exact public string;
- unknown or malformed stage input cannot enter serialized output;
- existing details survive stage insertion;
- kind, message, and exit code are unchanged;
- connection, setup-send, series-send, read/close, timeout, protocol,
  heartbeat, pagination, and no-bars fixtures receive the intended stage;
- a failure after bars were observed preserves existing bar count,
  wait-summary, and range-fetch details;
- validation errors contain no `source_failure_stage`;
- serialized fixtures contain none of the forbidden raw values named above;
- CDP `failure_stage` vocabulary and tests remain unchanged;
- no fixture retries, reconnects, extends deadlines, substitutes a source, or
  promotes failure to success.

If an exact transport boundary cannot be injected without a production-only
hook, stop and revise this plan. Do not add a public or ordinary-build API only
to satisfy a test.

### Milestone 4: document agent interpretation

Update `README.md`, `docs/command-source-taxonomy.md`,
`docs/architecture.md`, `docs/observation-workflows.md`,
`packaging/agent/AGENTS.md`, and the relevant runtime-skill references.

Explain that `source_failure_stage` locates a Desktop-free source failure. It
does not authorize retry. Recommended interpretation is:

- `symbol_search`: use an exchange-qualified symbol when identity is known;
- `request_prepare`: report an internal source-request preparation problem;
- `websocket_connect`: re-observe source availability before considering a
  separately approved repeat;
- setup, series, heartbeat, or pagination send stages: preserve the failed
  outcome because dispatch may be uncertain;
- `response_wait`: report timeout/close/read evidence and preserve any partial
  source details;
- `protocol`: do not change symbol, timeframe, or source automatically;
- `source_result`: consume the existing availability and range diagnostics;
- `source_unknown`: stop and surface the original error.

These are explanations, not automatic actions implemented by the CLI.

## Concrete Steps

Run from the repository root:

    rg -n "AppError::new|with_details|connect_ws|send_ws|send_message|stream.next|parse_packets|no_bars_error|resolve_bars_symbol" crates/market/src/bars.rs crates/market/src/bars
    rg -n "failure_stage|source_failure_stage" crates/cdp/src crates/market/src README.md docs packaging/agent .agents/skills
    cargo test -p tradingview-market bars -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_bars -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    git diff --check

No live network test is required for acceptance. The observed real failure
motivates the contract; deterministic fixtures prove the mapping.

## Validation and Acceptance

Acceptance requires all of the following:

- every production bars failure boundary is inventoried or covered by a
  reproducible grouping rule;
- source-access failures include exactly one recognized
  `source_failure_stage`;
- validation errors do not include the field;
- existing error kind, message, details, and exit-code semantics are
  preserved;
- existing `bars.v1` success payloads do not change;
- existing CDP `failure_stage` values do not change;
- no raw endpoint, symbol addition beyond existing safe details, credential,
  protocol payload, frame, local path, or dependency error is added;
- no retry, reconnect, timeout change, fallback, background work, source
  substitution, or success promotion is introduced;
- focused and complete non-live validation are green;
- focused implementation review is green.

The observable result is a deterministic CLI or payload fixture whose
structured bars error contains the correct source stage while all previous
error semantics remain intact.

## Idempotence and Recovery

All searches, fixtures, and non-live validations are safe to rerun. Do not run
the live bars smoke or add a new network probe for this plan.

If implementation reveals that the source boundary cannot be classified
without changing send/response ownership or adding a public testing hook, stop
and revise the plan before editing production architecture. If validation
finds unrelated failures, record them separately and do not broaden this
slice.

## Artifacts and Notes

Keep only source inventories, fixed test fixtures, aggregate validation counts,
and public-safe examples. Do not retain live symbols, dates, prices, bars,
endpoints, raw frames, credentials, or one-off reviewer prompts.

The motivating evidence is deliberately narrow: one bars subprocess returned
a structured connection error, and a later bounded comparison succeeded. It
proves a diagnostic gap, not a reliability defect rate.

## Interfaces and Dependencies

No new dependency is required.

Define a private bars-owned enum equivalent to:

    enum BarsFailureStage {
        SymbolSearch,
        RequestPrepare,
        WebSocketConnect,
        SessionSetup,
        SeriesSetup,
        ResponseWait,
        Protocol,
        HeartbeatSend,
        PaginationSend,
        SourceResult,
        SourceUnknown,
    }

The exact helper signature may follow existing bars module ownership, but it
must accept an `AppError`, a `BarsFailureStage`, and existing bars details, and
return an `AppError` with preserved semantics plus the additive
`source_failure_stage` field.

Do not change `ErrorKind`, `AppError::exit_code`, CDP `PublicFailureStage`,
`bars.v1` success types, request timeouts, fetch sizes, or Cargo manifests.

## Open Questions

There are no unresolved questions blocking focused plan review.

The future recovery question remains deliberately open: which stages, if any,
justify a bounded repeat after repeated evidence? This plan must not answer it
by implementing retry.

Revision note (2026-07-27): created after the v0.31 one-minute bars handoff.
The plan separates Desktop-free source attribution from Desktop CDP
`failure_stage`, fixes a closed public-safe vocabulary, and explicitly defers
all recovery behavior.
