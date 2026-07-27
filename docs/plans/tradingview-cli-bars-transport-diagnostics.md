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
- [x] (2026-07-27) Focused plan review identified four blockers: incorrect
  facade lifecycle ordering, two incomplete stage mappings, an unspecified
  deterministic injection seam, and stale durable state.
- [x] (2026-07-27) Corrected the lifecycle and mapping, fixed the decorator
  contract, specified private production decomposition for deterministic
  tests, and synchronized roadmap and ledger state.
- [x] (2026-07-28) Corrected the pagination stage spelling to the required
  public contract, `pagination`, throughout the vocabulary, mapping, enum, and
  fixture expectations.
- [x] (2026-07-28) Confirmed the narrow pagination vocabulary correction and
  proceeded with the reviewed deterministic implementation boundary.
- [x] (2026-07-28) Implemented the typed stage mapping, private initial-setup
  and pagination helpers, and loopback/sink fault fixtures.
- [x] (2026-07-28) Synchronized CLI help, public docs, packaged agent guidance,
  and runtime skill interpretation.
- [x] (2026-07-28) Passed formatting, strict workspace Clippy, focused bars and
  CLI contracts, the full workspace suite and doctests, metadata, hygiene,
  package syntax, guide parity, and diff hygiene.
- [x] (2026-07-28) Focused implementation review confirmed the production
  mapping and found two acceptance gaps: no direct zero-result facade fixture
  and incomplete CHANGELOG state.
- [x] (2026-07-28) Added a production-shared result-shaping fixture for
  `source_result` and synchronized the CHANGELOG and current-state documents.
- [ ] Obtain narrow focused re-review and archive the plan.

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
  `pagination`, `source_result`, and `source_unknown`.
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

Implementation and full non-live validation are complete. Desktop-free bars
source failures now receive the additive `source_failure_stage` field without
changing success payloads or adding recovery behavior. Focused-review
corrections are applied and narrow re-review remains before archive.

The deterministic bars module suite completed with 37 passing tests and one
ignored live heartbeat probe. CLI bars contracts completed 4/4. Strict
workspace Clippy, the full workspace suite and doctests, metadata, public
hygiene, packaging syntax, contributor-guide parity, and diff hygiene passed.
No live TradingView request was run for this diagnostic slice.

## Context and Orientation

`tv bars` is a Desktop-free bounded historical-bars command. Public facade
functions `bars_symbol` and `bars_symbol_range` live in
`crates/market/src/bars.rs`. Their current lifecycle is: prepare the configured
HTTP client, optionally resolve a bare symbol through Desktop-free REST search,
validate the bars request, call `fetch_bars_ws`, reject an empty result through
`no_bars_error`, and shape a `bars.v1` success payload. This ordering means a
bare-symbol source failure can occur before a `BarsRequest` exists, and
validation can occur after a successful REST search.

`configured_client()` is local generic HTTP-client preparation. Its failure
does not contact a source and receives no `source_failure_stage`. A validation
error also receives no stage because it is not itself a source failure, not
because validation necessarily precedes source access. A bare-symbol REST
transport or response failure is a source failure and receives
`symbol_search`; a responsive search result that cannot resolve the requested
identity remains the existing validation outcome without a source stage.

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
- authentication, chart-session creation, and `switch_timezone` sends:
  `session_setup`;
- in-session symbol resolution and initial series creation sends:
  `series_setup`;
- WebSocket read error, close, or bounded timeout error generated inside the
  read loop: `response_wait`;
- malformed frame, protocol parse failure, or provider
  symbol/series/protocol error: `protocol`;
- heartbeat pong send failure: `heartbeat_send`;
- `request_more_data` send failure: `pagination`;
- any `Ok(BarsResult)` that reaches the facade with zero bars:
  `source_result`, regardless of `BarsResult.completed`;
- any genuinely unclassified source boundary: `source_unknown`.

The existing `completed`, `source_availability`, `wait_summary`, and
`range_fetch_summary` fields explain why a zero-bar `source_result` was
complete, timed out, source-exhausted, or stopped for no progress. Do not
duplicate that reason in the stage.

Do not classify validation errors because they are not source failures.
`configured_client()` failure is also unstaged local preparation. Explicitly
include `switch_timezone` in the inventory even though it is sent after series
creation; its role is chart-session configuration and its stage is
`session_setup`.
Cleanup failures remain intentionally ignored by the existing best-effort
cleanup contract and do not replace the primary outcome.

Acceptance for this milestone is a fresh `rg` inventory with no unexplained
production boundary.

### Milestone 2: add typed source-stage mapping

Add a private enum in the bars module, named `BarsFailureStage` unless the
existing module layout shows a more coherent private location. Implement an
`as_str()` mapping for the exact closed vocabulary above. Do not serialize the
enum directly and do not expose a new public Rust type.

Add one private decorator with the exact ownership shape:

    fn with_source_failure_stage(
        error: AppError,
        stage: BarsFailureStage,
    ) -> AppError

The decorator inserts `"source_failure_stage": stage.as_str()` last into
existing object details. With no details it creates an object containing only
the stage. A non-object details value is not expected at reviewed call sites;
if encountered, omit that prior value and add a fixed
`"previous_details_omitted": true` marker rather than serializing unknown raw
content. This mirrors the repository's fail-closed diagnostic policy.

Transport call sites first construct their existing `bars_error_details`, then
apply the decorator. Bare-symbol REST failures apply the decorator directly to
the existing HTTP `AppError`, preserving its `operation`,
`http_failure_class`, and optional `status` without requiring a `BarsRequest`.
Normal production call sites always pass an explicit stage.

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

Add deterministic tests close to the owning code. The current lifecycle is too
monolithic to prove each call-site mapping with the existing sink alone.
Decompose it into private production helpers that are called by
`fetch_bars_ws` in ordinary builds:

- a private generic initial-setup helper that performs exactly five sends in
  existing order: `set_auth_token`, `chart_create_session`, `resolve_symbol`,
  `create_series`, and `switch_timezone`;
- a private pagination helper for the single `request_more_data` send;
- the existing private heartbeat helper;
- a private generic response-loop runner, or a scripted local WebSocket fixture
  against the production response-loop code, for read error, close, timeout,
  protocol, heartbeat, and pagination paths.

Use a fake sink that fails on the configured Nth send to prove the initial five
call sites. The first, second, and fifth failures map to `session_setup`; the
third and fourth map to `series_setup`. The helper extraction must preserve
send order, arguments, one setup deadline, response deadline, call counts, and
existing return behavior. These are private production boundaries, not public
or test-only production APIs.

Bare-symbol search needs no transport seam in this slice: test the pure stage
decorator with representative existing HTTP object details and preserve the
existing HTTP transport tests.

Tests must prove:

- every enum variant maps to its exact public string;
- unknown or malformed stage input cannot enter serialized output;
- existing details survive stage insertion;
- kind, message, and exit code are unchanged;
- connection, setup-send, series-send, read/close, timeout, protocol,
  heartbeat, pagination, and no-bars fixtures receive the intended stage;
- all five initial sends retain their existing order and exact stage mapping;
- a failure after bars were observed preserves existing bar count,
  wait-summary, and range-fetch details;
- validation errors contain no `source_failure_stage`;
- serialized fixtures contain none of the forbidden raw values named above;
- CDP `failure_stage` vocabulary and tests remain unchanged;
- no fixture retries, reconnects, extends deadlines, substitutes a source, or
  promotes failure to success.
- `source_unknown` is covered as the decorator fallback but no reviewed normal
  production call site intentionally uses it.

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
- `session_setup`: report a common authentication/chart-session bootstrap
  failure and do not recommend changing the request;
- `series_setup`: report a request-specific symbol/timeframe/series boundary,
  but do not infer invalid input or non-dispatch;
- `heartbeat_send` or `pagination`: preserve the failed outcome and any partial
  bars because dispatch may be uncertain;
- `response_wait`: report timeout/close/read evidence and preserve any partial
  source details;
- `protocol`: do not change symbol, timeframe, or source automatically;
- `source_result`: consume the existing availability and range diagnostics;
- `source_unknown`: stop and surface the original error.

These are explanations, not automatic actions implemented by the CLI.
For every send stage, the value identifies only where the local send operation
returned an error. Remote receipt, processing, and effect are all unknown.

Document that `source_failure_stage` is a field name available for
source-owned diagnostics, but its closed vocabulary is command/source
specific. A future Desktop-free source may use the same field only with its
own reviewed vocabulary; it must not silently inherit the bars values.

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
        Pagination,
        SourceResult,
        SourceUnknown,
    }

Define the decorator exactly as:

    fn with_source_failure_stage(
        error: AppError,
        stage: BarsFailureStage,
    ) -> AppError

It returns an `AppError` with preserved kind, message, and existing object
details plus the additive `source_failure_stage` field. It does not accept a
`BarsRequest` or a separately supplied details object.

Do not change `ErrorKind`, `AppError::exit_code`, CDP `PublicFailureStage`,
`bars.v1` success types, request timeouts, fetch sizes, or Cargo manifests.

## Open Questions

There are no unresolved questions blocking focused plan re-review.

The future recovery question remains deliberately open: which stages, if any,
justify a bounded repeat after repeated evidence? This plan must not answer it
by implementing retry.

Revision note (2026-07-27): created after the v0.31 one-minute bars handoff.
The plan separates Desktop-free source attribution from Desktop CDP
`failure_stage`, fixes a closed public-safe vocabulary, and explicitly defers
all recovery behavior.

Revision note (2026-07-27): corrected four focused-review findings. The facade
lifecycle now matches source, `switch_timezone` and all zero-bar results have
unique mappings, the decorator works before `BarsRequest` creation, private
production helper seams make exact fault injection executable, and durable
state is synchronized for re-review.

Revision note (2026-07-28): corrected the remaining public vocabulary mismatch.
The pagination send boundary is now named `pagination` / `Pagination`
throughout the decision, mapping, implementation sketch, guidance, and
deterministic fixture contract. No behavior, recovery policy, or implementation
scope changed.

Revision note (2026-07-28): implemented the reviewed contract. Private
production helpers preserve the five setup sends and pagination call, while
sink and loopback fixtures exercise setup, connection, response, protocol,
heartbeat, and pagination failures without live TradingView access. Public
guidance treats stage attribution as diagnostic only.

Revision note (2026-07-28): corrected two focused implementation-review gaps.
The production facade and deterministic fixture now share `bars_result`, which
proves that an empty `BarsResult` preserves no-bars diagnostics and gains
`source_result`. CHANGELOG and durable state now record full validation and the
narrow re-review gate.
