# MCP operational improvements and two additional reads

Status: **login guidance, history, connection reuse and read timeout implemented;
technical snapshots deferred; skill layout and offline spec implemented**,
2026-09-25.
Direction and priority live in the [roadmap](../next-version-roadmap.md) and
[inventory](../next-version-work-items.md). This is the single work record for
v0.33.0; the [v0.32.0 record](archives/tradingview-cli-official-mcp-client.md)
is closed historical evidence.

## Outcome, consumers and authority

Deliver alert firing history and a more usable login/upgrade journey through
the independent `tv mcp` surface. The owner initially included technical
snapshots, then explicitly deferred them on 2026-09-25. Their proposed contract
and dated investigation below are retained for future resumption, not as a
current release requirement. Connection reuse is conditional on measurement. Existing
CLI consumers keep their commands, envelopes and source semantics.

The PM remains the sole executor; no additional agent/session is authorized.
The owner approved the CLI/JSON proposals below and the bundled new live-read
scope on 2026-09-22 under [AGENTS.md](../../AGENTS.md). Their values are synthetic client-output proposals,
not observed provider payloads. No new Rust dependency or persisted format is
proposed. Local documentation commits are within the established PM authority;
push/tag/workflow/release and downstream edits are not authorized here.

Reuse existing same-target verification authority. The new verification scope
below is also approved; no arbitrary count/time approval cycle is introduced.
Actual rate limits, deadlines, dispatch guards and user-controlled OS consent
remain mandatory. This planning stage performs no account or credential I/O.

## Current evidence and alternatives

- v0.32.0 is published at `54dabec`; its release workflow succeeded. Downstream
  reports released-binary daily observation/readback and unchanged D/W/M intake
  contracts. Account-history and technical snapshots are not yet accepted by a
  downstream analytical consumer.
- `client::failure` already gives `next_action: tv mcp login` for missing
  credentials or OS interaction. Improve the explicit login journey rather than
  add a redundant doctor command or change the ordinary structured error.
- `transport::call` initializes and discovers tools on every call. Watchlist and
  alert changes call it once for mutation and again through `account::readback`.
  This proves repeated setup, not its latency cost. Start with private counters
  and timings in the synthetic harness; do not add public timing metadata.
- Two upstream MCP fixtures timed out in an earlier parallel run and passed
  serially; CI later passed. Investigate reproducibility, not an assumed
  production defect. Downstream test scheduling has a separate owner.
- [Official documentation](https://www.tradingview.com/mcp/docs), checked on
  2026-09-22, lists `get_alerts_log` and `get_technicals_rating`. Neither was in
  the released v0.32.0 closed tool registry. Documentation is not actual wire/schema
  evidence; inspect the authenticated catalog and responses before freezing
  their decoders.

Do not implement technical snapshots by silently substituting existing column
reads. Compare overlap for usefulness and interpretation, but keep the dedicated
provider tool explicit. Existing `symbol`/`symbols` is better for arbitrary fields
or batches; the new command is a single-timeframe snapshot without field-name
assembly. No equality or chart/Pine equivalence is promised.

## Proposed CLI and JSON contracts

### Official technical snapshot

Before: v0.32.0 has arbitrary `mcp symbol` columns and chart-owned study reads,
but no dedicated official technical snapshot command.

After (proposed):

```sh
tv mcp technicals NASDAQ:EXAMPLE --timeframe 1D
```

One exchange-qualified symbol; default `1D`. Accept the provider's documented
`1m`, `5m`, `15m`, `30m`, `1h`, `2h`, `4h`, `1D`, `1W`, `1M` values after
catalog verification. In particular, `2h` support here must not widen MCP bars
or legacy date-range validation. No range, batch or local indicator calculation.

Illustrative client envelope (abbreviated indicator inventory):

```json
{
  "success": true,
  "command": "mcp",
  "data": {
    "contract_version": "mcp_technicals.v1",
    "source": "tradingview_mcp",
    "requested": {"symbol": "NASDAQ:EXAMPLE", "timeframe": "1D"},
    "reported": {"symbol": null, "timeframe": null, "data_as_of": null},
    "retrieved_at_unix_ms": 1700000000000,
    "indicators": [{"name": "RSI", "value": 52.5}],
    "ratings": {"summary": null, "moving_averages": null, "oscillators": null},
    "conditions": {"delay": null, "adjustment": null, "session": null, "finality": null}
  }
}
```

`reported` contains provider evidence only, never the requested values copied
as confirmation. A contradictory symbol/timeframe echo is an invalid response.
Indicator names and scalar value types must be fixed from inspected response
shape; do not preserve arbitrary nested payloads as an escape hatch. Known
missing values are null, never zero. Null ratings are unreported, not neutral.
Preserve provider rating values without local thresholds or trading actions.
Do not label the snapshot a historical series or point-in-time input.

An absent required container, invalid numeric value or changed type fails with
`mcp_error.v1`/`invalid_response`. Valid containers with absent optional entries
remain successful with unknown values. A response with no usable observations
must have an explicit empty state in the final reviewed schema, not fabricated
indicator values; its actual provider shape is still UNCONFIRMED.

### Alert firing history

Before: v0.32.0 can read/manage alerts but cannot read their fire history.

After (proposed):

```sh
tv mcp alert history --symbol NASDAQ:EXAMPLE --days 7 --limit 100
```

Require `--symbol` in the first CLI slice to avoid accidental account-wide
reads. Default days 7 and limit 100; limit 1..2000 per official documentation.
Days must be a positive integer; confirm any server upper bound instead of
inventing one. No pagination, account writes, monitoring or event-triggering.

```json
{
  "success": true,
  "command": "mcp",
  "data": {
    "contract_version": "mcp_alert_history.v1",
    "source": "tradingview_mcp",
    "requested": {"symbol": "NASDAQ:EXAMPLE", "days": 7, "limit": 100},
    "retrieved_at_unix_ms": 1700000000000,
    "events": [
      {
        "alert_id": 123,
        "symbol": "NASDAQ:EXAMPLE",
        "fired_at_unix_seconds": 1699999900,
        "webhook_delivery_status": null
      }
    ],
    "returned_count": 1,
    "limit_reached": false,
    "coverage": "unconfirmed"
  }
}
```

Names/values above are proposed normalized fields. Preserve only confirmed
provider fields; do not invent IDs, timestamps or delivery status. Establish
required identity/time fields and timestamp units from actual schema evidence.
Reject contradictory symbol rows rather than silently relabel/filter them.
Do not infer alert active state, receipt by a person, or trade execution.
Messages, webhook URLs, arbitrary raw conditions and raw payloads are excluded
from this first public output. Inspect private payloads only in untracked local
artifacts; fixtures use synthetic events.

Empty history is `events: []`, `returned_count: 0`, `limit_reached: false` and
`coverage: unconfirmed`. At the requested limit, `limit_reached: true` reports
only measured saturation, not proof of additional events. Fewer events than
requested is not proof of complete history. These are request caps, not a
promise to return N events. Unsupported paging/range flags fail before I/O.

### Shared failures and compatibility

Existing `mcp_error.v1`, exit codes and zero-attempt pre-I/O validation apply.
For example, an invalid timeframe returns validation/unsupported capability with
`tool_attempts: 0`; absent credentials returns `auth_required` plus the existing
login action. A dispatched 429 preserves `rate_limited` and only confirmed
Retry-After evidence; timeout uses `deadline_exceeded`; malformed responses use
`invalid_response`. No silent alternate source, zero-filled data or automatic
replay. Wire failures and optional missing values are separate outcomes.

No new success/error fields are added to existing commands. For the login
journey, propose concise human guidance on stderr before interactive waits;
success stdout and JSON failures remain unchanged. Inspect current CLI capture
conventions before choosing progress output so failure stderr stays consumable.
If that cannot be achieved without changing a consumer contract, return the
specific before/after choice rather than mixing prose into JSON stderr.

## Implementation and acceptance sequence

1. Contracts and the new scoped reads are approved.
   Verify actual tool names/schema/response shape. Revise material contract
   differences before implementation; do not encode guessed provider evidence.
2. Improve login reuse/interaction guidance. Test existing credentials, expired
   token, missing/locked OS store, update-related consent and interrupted login.
   Ordinary commands remain noninteractive; no extra registration on reusable
   credentials. Do not promise signing/notarization effects.
3. Add history validation/normalization to the account model and a closed tool
   adapter; add technical snapshot shaping in the I/O-free model and a closed
   service adapter. CLI owns parsing/envelopes, MCP owns auth/transport. Each
   command must be independently usable and tested before the next slice.
4. Measure mutation/readback setup. If useful, let one command own the live
   session and validated tool bindings, then close it within the same deadline.
   Preserve one mutation dispatch, readback uncertainty, cooldown and credential
   ownership. Do not cache credentials/catalogs across processes. Compare call
   counts and timings before/after; no reduction in actual tool admission checks.
5. Correct demonstrated fixture timing dependencies alongside affected changes.
   Separate intentional timeout tests from server/worker startup readiness;
   retain failures and diagnostics rather than treating serial pass as a fix.
6. Update usage/source maps and the account-management/market-data skills with
   all mandatory references inside each skill directory. Existing chart/Pine
   routes remain explicit. Do not install or modify downstream skills here.
7. Run applicable development/platform checks, qualify the two reads and report
   limitations. Release preparation is a later separate commit; no version bump
   or published capability claim in this planning change.

Fixture acceptance covers CLI-to-model integration, request defaults/bounds,
source/echo validation, null/empty/limit cases, malformed types, 401/429/timeout,
no implicit retry and sensitive-field exclusion. Connection reuse adds count
assertions for initialization/catalog calls and mutation/readback failure paths.
Run focused tests then the required Rust baseline for production changes;
unchanged JavaScript gates may reuse valid evidence. Verify normal parallel CI
on all supported OSes; Linux graphical OAuth remains a separately stated limit.

## Live verification proposal and remaining decisions

The owner approved the following scope; reuse this approval for necessary
verification and fixes within the same targets and effects:

- Use the existing chosen account and selected installed/trusted executable;
  no new account setup or OAuth scopes are expected. The technical tool reads
  NASDAQ:AAPL daily, weekly and monthly snapshots, compares available overlapping
  columns only when necessary, and checks `2h` if claiming that interval.
- Read NASDAQ:AAPL alert history for the previous seven days with limit 100.
  This accesses account-local event content even though output is redacted.
  No alert creation, edit, firing, deletion or webhook delivery is requested.
  If the history is empty, record empty-only native evidence; do not manufacture
  account activity to obtain a success fixture.
- Reuse existing credential-operation authority for the same target/effects.
  Announce any browser/OS interaction before starting it and wait for the user.
  An executable change alone does not imply new provider-operation authority.
- Connection optimization uses synthetic mutation/readback for measurement.
  Any additional live mutation requires a concrete disposable target proposal;
  it is not a hidden prerequisite for either new read.

The approved contract direction covers the two CLI shapes, source/unknown
handling, limited history output and interaction guidance. Provider response details remain
UNCONFIRMED until observation; public documentation is insufficient to assert
them. No new dependencies are expected; any demonstrated need is a separate
concrete proposal with current-version verification.

## Progress

- Scope accepted, including technical snapshots as a planned feature.
- v0.32.0 publication and release-workflow success rechecked; old plan archived.
- Current code and downstream readback reviewed; no new consumer defect claimed.
- Roadmap/inventory synchronized; synthetic contract examples proposed here.
- At planning completion, no implementation or live request had occurred.
  The subsequent preparation checkpoint below supersedes that state.
- Planning checks passed: 59 local Markdown file references, two synthetic
  JSON examples, public-hygiene self-test/scan (709 tracked files), and diff
  whitespace checks. Credential-language review found only policy/examples.
  No runtime resources or Rust code changed, so no rebuild or functional suite
  was run for this documentation-only change.
- Owner approved the contracts and bundled scoped reads. Material differences
  discovered in real response shapes remain decision points; do not re-ask for
  already approved operations.
- Before feature implementation, the owner requested disabling expensive local
  pre-push checks by default and minimizing workstation load. Installers and
  normal hook enablement now leave the baseline disabled; explicit baseline
  runs default to one build job and one test thread. CI remains unchanged.
- Subsequent local work uses one Cargo operation at a time, focused checks and
  existing build artifacts. Broad/release checks require a concrete need and
  should not be repeated after unchanged inputs.
- Hook prerequisite validated with a temporary repository and mocked Cargo:
  install/enable leave pre-push disabled, disabled pre-push invokes no Cargo,
  explicit baseline uses 1/1 defaults, caller overrides work, and command
  failure stops subsequent checks. Bash syntax, TOML parsing, public hygiene
  and diff checks passed. This checkout's baseline hook is disabled.
  PowerShell received the equivalent source change but was not executed because
  it is unavailable on this host. No Rust compilation or functional test ran.
- Next: continue the approved contract qualification and implementation with
  these resource limits.


## Contract qualification preparation (2026-09-22)

Added six explicit development-harness operations, not public commands:
`next-read-catalog`, four technical timeframe shape reads and
`alert-history-shape`. The closed registry accepts only the approved requests
for these proof tools. The existing transport, absolute 30-second deadline,
OS credential worker, admission/cooldown and no-replay behavior are reused.
No dependencies, public success/error contracts or persisted credential formats
changed. The installed v0.32.0 binary was preserved and selected as worker.

The new observation module retains only known field names, scalar types,
array lengths and identity comparisons. It suppresses all scalar values and
unknown keys, including account IDs, message bodies and webhook URLs. It does
not store raw provider payloads or turn unobserved shapes into fixtures.

### Executed evidence

- Two new pure tests passed: exact proof-scope validation and observation
  redaction, including unknown keys and empty arrays. Three existing tool
  registry tests also passed. Tests used one thread.
- The incremental test build took 16.76 seconds and the example build took
  12.02 seconds, with one Cargo job and no overlapping builds. Existing target
  artifacts were reused. An attempted lower process priority was denied by the
  local sandbox; no reduced-priority claim is made. Full workspace/release
  builds and broad Clippy were not repeated for this preparatory change.
- Two explicit catalog attempts ended within the normal operation deadline
  without catalog completion. In the second attempt, persisted counters rose
  by two metadata and two protocol requests, with no tool call, registration,
  token exchange or refresh. Credential restoration therefore progressed far
  enough to begin MCP protocol traffic; this does not prove server acceptance.
- Separate public GET checks for the configured protected-resource and issuer
  metadata endpoints returned HTTP 200 and valid JSON. An initial manual
  resource-path probe omitted the `/mcp` suffix and is not evidence about the
  configured endpoint. No endpoint configuration was changed.
- Existing HTTP diagnostics retained `authorization_metadata` as their last
  named stage while subsequent protocol traffic had begun. That field cannot
  identify the timeout's phase. Last HTTP status 200 and no tool dispatch do
  not establish whether initialization, notification or catalog waiting was
  responsible. No current all-MCP outage, 429 or credential defect is inferred.

### Implementation handoff

The next step at that checkpoint was narrow, private attribution of MCP
initialization versus catalog waiting before another live attempt. Preserve normal deadlines and
avoid raw SDK logging or token/payload capture. The current evidence does not
justify raising timeouts, adding retries or implementing schema guesses.

Once discovery completes, use the prepared shape operations sequentially.
Capture the documented indicator and rating containers, symbol/timeframe echoes
and actual timestamp units; for history, distinguish empty-only native evidence
from nonempty synthetic coverage. Finalize model normalizers only after these
facts are established. New fixture values must be synthetic, with unknowns
preserved. Login-journey and fixture-readiness work can proceed independently.

No technical snapshot or alert-history tool call has yet been dispatched.
Both public features remain planned and unimplemented; no new user approval is
needed for the existing scoped reads. Material contract differences still need
review. Preparation is ready; runtime contract qualification remains open.


## Protocol attribution and login guidance (2026-09-22)

Implemented private per-method HTTP diagnostics with latest phase, status and
remaining operation budget. Concurrent event-stream setup no longer obscures
which request was waiting. Only closed protocol names and nonsecret scalars are
recorded; public `mcp_error.v1` is unchanged. Synthetic delayed-header fixtures
prove separate initialization and catalog attribution with zero tool calls and
no tokens/arguments in diagnostics.

The catalog subsequently completed with the existing credentials and a
successful token refresh. Both proposed tools were present, their documented
arguments passed schema checks, and both advertised read-only hints. This does
not establish that their data responses work. The old timeouts did not recur in
that call; their root cause is still unconfirmed.

Later observations distinguish separate outcomes:

- A daily technical call completed initialization (HTTP 200), initialized
  notification (202) and catalog (200), then timed out awaiting tool-response
  headers. No response status for that tool call was received.
- An independent alert-history attempt timed out awaiting initialization headers,
  before dispatching the history tool. These failures have different boundaries.
- A subsequent daily technical call returned JSON with a success boolean and an
  initially unrecognized field. This first shape receipt was not data acceptance.
  The proof was tightened to preserve the provider success flag and classify
  explicit `success:false` as a failure. A new read confirmed `success:false`
  plus an error string, with no indicator values. The error string itself is
  suppressed; its cause is not established. Do not infer HTTP 429 or a daily
  quota from this application-level failure.
- The subsequent history read completed with `success:true`, an empty `events`
  array, and numeric `count`/`days` fields. This is empty-only native response
  evidence for the approved symbol/window, not a validated nonempty event schema
  or a complete-history claim. No alert was created, edited, fired or deleted.

The technical weekly/monthly/two-hour reads were not dispatched after the daily
failure. Both new public commands remain planned; there is no schema guess,
timeout extension, implicit source switch or mutation replay. Successful catalog
and history observations rule out treating all MCP commands as unavailable.

### Login usability delivered

Explicit login now announces credential reuse/possible interaction on a real
terminal, explains verifying the executable/item before an OS permission, and
announces browser opening before invoking it. It only says the URL was handed
off after browser launch succeeds. All human progress notices are suppressed
when stderr is captured, preserving parseable error output for CLI consumers.
No registration, consent, worker or saved-format policy changed. The installed
released binary remains untouched; the new terminal journey has not been
exercised with a fresh live authorization.

### Validation and next work

Focused offline validation passed: the protocol timeout test (both phases),
validated-record reuse test, two auth tests including a child-process check that
captured stderr contains only JSON, three proof-scope/redaction/application-error
tests, and the existing no-replay/no-reinitialization fixture. The first new
protocol fixture omitted a required request default and failed before I/O; it
was corrected to use the existing model request builder, then passed.

All Cargo work was sequential with one build job and one test thread, using the
existing target cache. No full workspace or release build was run. macOS native
observations are as described above; no Windows/Linux runtime claims are added.
The private diagnostic and login changes compile in the example; they do not
qualify either new public command.

Next: investigate the technical tool's application failure with public-safe
cause classification, qualify successful technical values and nonempty history
fields from designated evidence, and implement the reviewed normalizers. Login
prompt policy is delivered; transport optimization still requires measurement
and fixture-readiness changes still require a reproduced issue. Reuse current
approval rather than asking again for the same reads.


## Investigation budget decision (2026-09-22)

Repeated normal-deadline probes exhausted the 30-second budget during setup:
one catalog request began with about five seconds remaining, another with less
than four. A synthetic atomic write/fsync in a disposable directory under the
same local state root completed in 1.24 ms and was removed; this does not
reproduce the multi-second delay as local state-write latency.

Use the existing development-only 180-second shape-investigation budget for
these six proof operations, as already used by economic schema investigations.
This supersedes the earlier self-imposed no-extension rule for these private
probes only. The owner-approved targets/effects, single dispatch, real limits,
credential policy and public 30-second deadline are unchanged. No automatic
retry is added. Waiting longer may obtain schema evidence but cannot establish
normal-deadline acceptance. Report investigation and public readiness separately.
This is a private diagnostic procedure within the approved read scope, not a
public timeout/API change or new access permission.


## Provider-limit classification and schema evidence (2026-09-22)

Added private closed-vocabulary classification of provider error text. The
report retains only matched categories and `root_cause: unconfirmed`, never the
error string, nested error values, request URLs or identifiers. Application-level
hints do not change the public error code, create HTTP-status evidence, choose
a reset time or trigger automatic retries.

The catalog probe now summarizes optional output schemas without descriptions,
examples, defaults or enum values. Both selected tools returned an object schema
without field definitions in the captured outline. Input schema compatibility
remains established; the output schema does not supply missing nonempty event
or technical field definitions. The published official documentation still
lists the commands/parameters but is not substitute response-shape evidence.

Normal-deadline attempts again expired before tool dispatch. With the private
investigation budget, catalog inspection completed and a daily technical read
returned `success:false` with a string error matching the `rate_limit` category.
This is provider-text evidence, not an observed outer HTTP 429 or Retry-After.
Stop further technical probes until the provider limit permits them; do not
claim that all MCP commands are limited. No additional timeframe was called.
The successful empty history response from the previous checkpoint remains
valid only as empty-response evidence. No reset time or general availability
was inferred.

A question is pending to the owner: permit one same-account, last-seven-days,
maximum-100-event history read without the symbol filter, for shape discovery
only. This expands the current AAPL-only access to potentially other symbols,
messages and delivery details, so it has not been executed or silently inferred
from the prior approval. Store/report only shape, never actual account values.
No creation, edit, firing, deletion or webhook delivery is part of the request.
The proposed public CLI still requires a symbol. If the owner declines or the
broader response is empty, retain the missing evidence explicitly; do not guess
nonempty fields or manufacture activity.

Validation: four scoped proof tests and one output-schema sanitization test
passed. The example was incrementally built with one Cargo job; tests used one
thread. Public-hygiene, formatting, Markdown reference and diff checks passed.
No new dependencies, public contracts, installed binary, local pre-push settings
or workspace version changed. No full workspace test or release build ran.

Next: obtain nonempty history structure within an approved read scope, then
complete its normalizer; resume technical shape qualification only when limits
permit. The two features remain required planned scope, not silently deferred
or claimed ready for release. Independent measurement and deterministic fixture
work may proceed while provider data qualification is unavailable.


## Alert history implementation (2026-09-23)

The owner instructed proceeding after the concrete account-wide shape-read
proposal. That instruction was treated as approval for the same account,
seven-day window, maximum 100 events, shape-only investigation and necessary
verification. Two explicit reads returned one event each. Only flat field names,
types and a masked timestamp pattern were retained; no event values, account
identifiers, message bodies or webhook contents were saved. The public CLI
continues to require a symbol; the development-only unfiltered request remains
fixed to seven days and 100 events. No account mutation occurred.

Observed event fields include numeric `tv_alert_id` and `fire_id`, string
`symbol`, `fired_at`, `bar_time`, `resolution`, `message`, and null `name` and
`webhook`. The firing timestamp has UTC second syntax `YYYY-MM-DDTHH:MM:SSZ`.
The implemented normalizer uses `tv_alert_id` for alert identity and `fired_at`
for firing time, never the event ID or bar time. Synthetic fixtures exercise
those distinctions. Non-null webhook delivery semantics remain unqualified;
`webhook_delivery_status` is always null in this slice, with no raw passthrough.

Implemented the approved `tv mcp alert history --symbol … --days 7 --limit 100`
command and `mcp_alert_history.v1`. It preserves row order and repeated events,
rejects contradictory symbols/counts/window echoes and malformed required
identity/time fields, and reports empty versus saturated results without
claiming coverage. Existing commands and public deadlines are unchanged.
Documentation, source taxonomy and the independently usable account-management
skill now describe this command and its limits.

### Acceptance evidence and limits

- Focused model checks passed, including UTC/calendar conversion and 14 account
  tests. The service fixture covers valid, empty, mismatched, malformed,
  provider-error and HTTP-429 results with one dispatch and no replay.
- All nine MCP CLI contract tests, three registry tests and five proof tests
  passed. Scoped strict Clippy passed for model, MCP and CLI, all targets.
- Formatting, diff/public hygiene, skill validation, standalone references and
  runtime-package self-tests passed. Placeholder-binary staging passed with
  seven skills per root; this is not release-binary validation.
- Cargo operations were sequential with one build job and one test thread.
  No full workspace test, release build or Windows/Linux native run was added.
- Nonempty provider structure was obtained with the private 180-second
  investigation budget. A separate AAPL/seven-day/100-event call through the
  public service's normal 30-second deadline ended with `deadline_exceeded`,
  `stage: tool_response`, `tool_attempts: 1`. Thus public-service native success
  remains unqualified. No retry or public timeout increase was introduced.

The installed released binary remains unchanged and served only as the trusted
credential worker. Technical reads remain stopped after the earlier provider
rate-limit textual clue; no reset time or all-MCP outage is inferred. Next:
qualify normal-deadline history success when available, obtain successful
technical response evidence when limits permit, and continue independent
synthetic transport measurement. Neither feature is declared release-ready.


## Technical recheck and setup measurement (2026-09-23)

One explicit daily technical shape recheck used the already approved account,
symbol and interval, with the installed binary as credential worker. It again
returned `success:false`, string `error`, and a `rate_limit` textual clue.
This does not establish HTTP 429, a reset time or all-tool unavailability.
Weekly, monthly and two-hour probes were not dispatched. Successful technical
fields remain unqualified; do not implement a decoder from guessed values.

Added an opt-in loopback measurement using the actual account service path,
synthetic credentials and disposable admission state. It runs watchlist rename
and alert stop, each followed by successful readback. All six cases dispatched
exactly two tools, with two initializations, two initialized notifications and
two catalog reads. The mutation was dispatched once; no live mutation occurred.

| Added delay per initialization/catalog response | Watchlist elapsed | Alert elapsed |
| --- | --- | --- |
| 0 ms | 1115 ms | 1149 ms |
| 100 ms | 1333 ms | 1322 ms |
| 600 ms | 2623 ms | 2609 ms |

These are one-run synthetic observations, not a production benchmark. The
one-second tool-admission interval hides some second-setup latency: removing
100 ms initialization plus 100 ms catalog waiting does not imply a 200 ms
end-to-end saving. At 600 ms per response, the repeated setup exceeds admission
spacing. The evidence supports proceeding with command-local reuse to eliminate
redundant protocol requests; the actual latency gain must be measured afterward.
It does not explain the provider's technical error or establish a timeout fix.

Implementation boundary for the next slice: one account command owns one
session and its catalog until readback completes, under the existing absolute
deadline. Validate each selected tool's arguments/schema before dispatch,
including readback targets learned from the mutation reply. Preserve the
mutation result when readback capability/transport fails; do not preflight a
readback in a way that changes existing mutation behavior. Keep admission before
each tool, no retry/reinitialization, no process-wide catalog/credential cache,
and existing cleanup/error semantics. Measure identical scenarios afterward
and extend missing-capability, deadline, 401/429 and uncertain-readback fixtures.

The measurement passed all six cases in 11.44 seconds; its incremental build
was 10.16 seconds with one Cargo job. It is ignored during ordinary CI and
explicitly documented in development guidance. Only fixture code and records
changed in this slice; no production API, dependency or timeout changed.

The existing transport fault/no-reinitialization regression also passed.
Formatting, diff and public-hygiene checks passed. No full workspace suite,
release build, installed-binary replacement or external publication was run.


## Command-local connection reuse (2026-09-23)

Implemented one transport session per account mutation command, retained through
its readback. Initialization occurs once. Catalog pages are fetched lazily and
reused only inside that command; when the readback tool is on a later page,
lookup resumes from the recorded cursor without repeating initialization or
previous pages. Each dispatch still validates arguments and the chosen schema,
then applies admission under the original absolute deadline. No reconnect,
replay, credential cache or cross-process session was introduced.

Readback capability checks occur after the mutation reply, so a missing tool or
changed readback schema cannot prevent an otherwise valid existing mutation.
The shared readback path closes the session and represents its failures within
the readback result, retaining the received mutation response. Failed mutations
and replies without a usable target also close the session. Ordinary independent
reads use the same transport primitive but finish their own session as before.
No public command, JSON, dependency or persisted-state format changed.

Added regression scenarios for both watchlists and alerts: absent readback tool,
changed readback schema, 401, 404, 429, readback deadline expiry and paginated
catalog lookup. They verify a single mutation, a single initialization, expected
readback dispatch counts and no token refresh/replay. The first fixture run used
an incorrect expected code for schema drift; it was corrected to the existing
`schema_changed` contract without changing production error mappings.


### Verification and measured outcome

The MCP library suite passed: 61 tests, with the opt-in measurement ignored.
All-target Clippy for the MCP crate passed with warnings denied. The same six
measurement scenarios then passed explicitly, with one initialization, one
initialized notification and one catalog read for two tools in every case.

| Added delay per setup response | Watchlist before / after | Alert before / after |
| --- | --- | --- |
| 0 ms | 1115 / 1109 ms | 1149 / 1121 ms |
| 100 ms | 1333 / 1323 ms | 1322 / 1353 ms |
| 600 ms | 2623 / 2335 ms | 2609 / 2336 ms |

These single-run synthetic comparisons establish reduced setup request counts,
not a general speedup percentage. At 600 ms, elapsed time decreased by about
0.27–0.29 seconds; at shorter delays, one-second admission spacing dominates and
the observed differences are small or within scheduling noise. Both tool calls
and their admission checks remain necessary. No production latency claim or
provider-error fix follows from this result.

Formatting, public-hygiene and diff checks passed. Cargo work was sequential,
one build job and one test thread, using existing artifacts. The library suite
took 89.92 seconds; the explicit measurement took 10.88 seconds; scoped Clippy
took 6.44 seconds. No workspace-wide/release build, real account mutation or
Windows/Linux runtime operation ran. Native history acceptance and successful
technical schema qualification remain open. The next provider read must reuse
approved scope and respect actual limits; do not substitute guessed fields.


## Native read recheck and completed-response diagnostics (2026-09-23)

Rebuilt only the incremental proof executable after connection reuse, retaining
the installed released executable as credential worker. The original
symbol/seven-day/100-event public-service history check again timed out at its
normal deadline after one tool dispatch (`stage: tool_response`). This does not
invalidate fixture normalization, but normal-deadline native acceptance remains
open. Mutation/readback reuse does not remove setup from a standalone read.

One daily technical shape recheck again returned `success:false` with a
rate-limit textual clue and no indicator containers. No other interval was
called. The official MCP documentation was rechecked: the dedicated technical
tool and its documented intervals remain present, but it supplies no successful
response example that resolves the unobserved wire shape. Neither a quota reset
nor an all-MCP outage is established. Further repeated technical calls are not
an implementation strategy; retain the planned feature pending usable evidence.

The private proof previously attached HTTP phase diagnostics only to transport
failures, losing that context when a longer-budget response arrived. It now
includes the existing sanitized diagnostics on received responses as well.
This changes only investigation output, not public CLI contracts or deadlines.
No raw payload, URL, header or credential is added. The existing protocol-phase
regression passed; the example was incrementally rebuilt, both sequentially with
one Cargo job and one test thread. Full suites and release builds were not rerun.


The subsequent AAPL history investigation succeeded with an empty event array.
Its sanitized protocol checkpoints, measured from the 180-second operation
budget, were: initialization JSON complete at 12.170 s (HTTP 200), initialized
notification accepted at 21.098 s (202), catalog complete at 26.867 s (200), and
history JSON complete at 50.781 s (200). The catalog-to-history interval was
23.914 s. These checkpoints include earlier work and scheduling; they are not
isolated server processing measurements and do not identify the latency root
cause. The longer-budget success is not normal-deadline acceptance and does not
justify silently increasing the public deadline. It establishes that this
bounded history request can succeed, while the observed total exceeds 30 s.

Current decision boundary: both public history acceptance and technical response
qualification remain incomplete. Do not repeat the same probes merely on every
continuation or mark unavailable technical fields implemented. A new provider
availability signal or a specific new diagnostic hypothesis should motivate the
next live attempt. Any configurable public timeout proposal needs its own
concrete CLI/behavior review; it is not included in this diagnostic change.
Formatting, public-hygiene and diff checks passed; no real account mutation,
new dependency, installed-binary replacement, push or release occurred.


## Independent latency investigation and timeout proposal (2026-09-23)

Source review found one reused reqwest client per command, existing connection
pooling and HTTP/2 support, no HTTP retry, and no intentional multi-second sleep
between initialization and catalog requests. Admission delays tool dispatch,
not metadata or initialization. No evidence justifies weakening binding checks,
removing durable pre-dispatch counters, forcing a protocol or caching metadata
across commands.

Anonymous curl GETs reproduced latency independently of the Rust client,
credential worker and state files. Bodies were discarded. For the MCP protected
resource metadata, the first observation completed TLS at 0.438 s but received
headers at 11.592 s. A separate reused-connection comparison returned 8.842 s
then 0.962 s (`num_connects` 1 then 0); issuer metadata returned in 0.063 s.
Explicit HTTP/1.1 pairs completed in 2.406/0.624 s and 5.985/4.826 s. An HTTP/2
pair completed in 13.092/2.919 s. These sequential small samples have different
network/server conditions and overlapping warm-request results; they do not
prove HTTP/2 causes the delay or identify an origin versus intermediary fault.
No authenticated MCP or technical tool call was repeated in this investigation.

Also corrected stale `tv mcp --help` text that still said Linux credential
support was unavailable. It now identifies Secret Service and its session-bus /
unlocked-collection requirement, consistent with the implemented adapter.

### Proposed explicit read timeout — NOT APPROVED OR IMPLEMENTED

Keep the existing 30-second default. Add an explicit MCP-group option only for
provider read operations, such as:

```sh
tv mcp --timeout-secs 90 alert history --symbol NASDAQ:EXAMPLE --days 7 --limit 100
```

Scope: Bars, Data, Financial, Research, Economic and Account reads. Login,
logout, local status and all watchlist/alert mutations reject the override
before credential/provider access. This avoids changing uncertainty windows for
account mutations. Positive integer seconds 1..180 would be accepted; invalid
numeric bounds return existing `mcp_error.v1` with `code: invalid_request`,
`reason: timeout_seconds`, `tool_attempts: 0`. Unsupported operation/override
combinations use `unsupported_capability` with zero attempts. Non-integer CLI
values follow existing argument parsing errors.

Before/after: the same request without the option still has a 30-second total
budget and can return `deadline_exceeded`. With `--timeout-secs 90`, the client
may wait up to 90 seconds total for local admission, discovery, credentials,
connection, dispatch and response/cleanup. Success uses the unchanged
`mcp_alert_history.v1` envelope; expiry still uses `mcp_error.v1` /
`deadline_exceeded`. Longer waiting is not guaranteed success. No retry, source
fallback, new JSON field, persisted configuration or default increase is added.
Actual cooldown and provider errors terminate normally; this does not solve the
technical tool's rate-limit response.

Alternatives: preserve the current behavior and wait for better provider/network
availability, or increase the default for all operations. Recommend the explicit
read-only option because callers retain their current maximum wait unless they
choose otherwise. Default increases remain outside the approved scope. This
public CLI/behavior proposal requires owner agreement under AGENTS.md before
implementation; research and the platform-help correction do not depend on it.

If approved, add operation-level validation before store/provider access, carry
one selected absolute deadline through the existing service, and test default,
explicit bounds, rejection on mutations, delayed success/expiry, no replay and
unchanged success/error envelopes. Update help/usage and independently usable
skills where they explain long reads. Then qualify the approved AAPL history
request with an explicit budget, separately from the still-open default-deadline
acceptance. Keep official technical snapshots planned until their wire response
can be qualified; do not equate a timeout option with completing that feature.


The existing focused MCP help/legacy-bars contract test passed, and the complete
help output was read back successfully with the corrected platform text.
Formatting, public-hygiene and diff checks passed. Local Cargo used one build
job and one test thread; no workspace-wide suite or release build ran. The
explicit timeout proposal was presented for owner review; its implementation
remains pending that decision.


## Explicit read timeout approved and implemented (2026-09-23)

The owner approved the preceding read-only timeout proposal, then chose the
shorter spelling `--timeout`. That choice supersedes `--timeout-secs`; no alias
is provided. Implemented the MCP-group option, accepting seconds before or after
a read subcommand. The previously recorded range, eligibility, unchanged default,
error contract and total-budget semantics are retained.

The CLI forwards the option to the service. Operation-level validation selects
one absolute deadline before local admission, credential-worker setup or provider
access. Existing callers of the internal service `run` retain default behavior.
No new dependency, persisted setting, public JSON field, automatic retry or
mutation deadline change was introduced. Usage docs and the independently
packaged connection references in market-data, account-management and
screener-workflow now explain the option and its limits.

Focused verification passed for invalid bounds, unsupported operations,
zero-attempt failures, absence of state-directory creation for rejected options,
unchanged default durations, accepted boundaries, and whole-operation expiry
versus delayed success without replay. The old `--timeout-secs` spelling is
rejected. The existing MCP help/legacy-bars test also passed. A selected short
operation deadline expires during catalog setup in the fixture; it does not
restart per request. Native verification is recorded separately below.


### Native acceptance and final checks

The fixed development harness invoked the same public service with the selected
90-second timeout for the approved AAPL/seven-day/100-event history request. It
used the installed trusted executable only as credential worker, preserving
PATH's released binary. The request succeeded with `mcp_alert_history.v1`,
`returned_count: 0` and `coverage: unconfirmed`. This qualifies empty-history
native behavior with an explicit timeout. It does not establish nonempty native
normalization, complete coverage or reliable success at the unchanged default;
the preceding default-30-second failure remains valid scoped evidence.

The affected MCP/CLI all-target Clippy check passed. Formatting, public hygiene,
changed Markdown links and twelve JSON examples passed. All three touched skills
passed metadata and standalone-reference checks. Runtime-package self-tests
passed (11 cases), and disposable placeholder-binary staging passed with seven
skills per root; it is not release-binary validation. All Cargo operations were
sequential with one build job and one test thread. No workspace-wide test,
release build, Windows/Linux runtime test, account mutation, push or installed
binary replacement occurred. Technical response qualification remains open and
was not retried for this timeout change.


## Technical upstream-error qualification (2026-09-23)

A new diagnostic hypothesis distinguished an outer MCP HTTP limit from an
application error mentioning the screener endpoint. Added one private
closed-vocabulary `screener_endpoint` hint for the literal known service
hostname in error text; no URL, query, error string or account identifier is
retained. A synthetic error containing a private path/query verifies that those
values are absent from the report. This does not change public error mapping.

The single approved AAPL daily recheck returned HTTP 200 with completed JSON
at 3.190 seconds into the investigation budget. Initialization and catalog also
completed with HTTP 200. The tool's structured response was `success:false`,
with a string error matching both `rate_limit` and `screener_endpoint`. No
indicator/rating containers were returned. This is a provider application
failure, not a client deadline expiry or an observed outer HTTP 429. The text
points to the official service's screener data path; the client did not directly
request that endpoint, so the upstream status, quota ownership and reset remain
unconfirmed. A longer timeout cannot resolve this received failure.

Stop identical technical rechecks until new availability evidence or a provider
response gives a reason to resume. Keep technical snapshots in the approved
version scope, but mark their implementation blocked on usable source evidence;
do not silently defer them, invent a wire schema or substitute another source.
The history and timeout features remain independently implemented and verified.

### Public-safe provider inquiry draft (not sent)

The official MCP tool `get_technicals_rating` (catalog wire name
`mcp-tv-get-technicals-rating`) fails for `symbol: NASDAQ:AAPL`, `interval: 1D`.
Observed on 2026-09-22 UTC / 2026-09-23 JST. Authenticated initialization and
catalog discovery succeed. The tool HTTP response is 200 with structured
`success:false` and a string error containing rate-limit and
`scanner.tradingview.com` references; no technical observations are returned.
The most recent response arrived within about 3.2 seconds. Other account reads
have succeeded separately; this report does not claim a universal MCP outage.
No raw error, credentials, account identifiers or opaque request IDs are included.

Please confirm whether this is an upstream screener access limitation, whether
there is a supported recovery/reset condition, and the successful response schema
for this tool. In particular, document indicator/rating containers, missing-value
representation, symbol/interval echoes and any data timestamp. This draft is
provided for owner review only; no external message or support ticket was sent.

### Resume and verification

Resume the approved daily shape read after a provider fix/availability indication
or new diagnostic evidence. If it succeeds, qualify the actual fields before
implementing normalization; then verify the approved weekly/monthly/two-hour
requests and preserved unknown conditions. Timeout extension alone is not that
availability indication. No additional account authorization is needed for the
already approved same-target reads.

The targeted redaction/classification test passed. The incremental proof build,
formatting, public-hygiene and diff checks passed. Builds were sequential with
one Cargo job and tests used one thread. No broad suite, release build, other
technical timeframe, new dependency, account mutation or installed-binary change
was performed. The official documentation was rechecked and still advertises
the tool and supported intervals, without the missing successful response shape.


## Documentation and remaining-work reconciliation (2026-09-23)

Audited implemented commands against help, the user reference, README, packaged
runtime guidance and standalone skill references. Corrected the roadmap's stale
claim that all approved work was unimplemented and the user reference's blanket
30-second statement, which omitted the explicit read timeout. README now clearly
separates development-only history/timeout capabilities from released v0.32.0.
Its detailed reference uses the repository URL because release packages do not
include the full contributor documentation tree.

Replaced accumulated inventory checkpoint prose with a current state table and
concrete next actions; dated evidence remains in this record. The plan index now
reflects delivered history, timeout and connection reuse, while retaining the
blocked technical snapshot in scope. Standalone skill routing and references
already cover implemented behavior and need no further edits in this pass.

Current-candidate cross-platform/default-concurrency CI is still pending; prior
release Windows/Linux results are not reused as current implementation proof.
No new fixture timing defect was established by the scoped serial checks, so no
speculative synchronization changes or heavier parallel local test was added.
Technical snapshots still require a usable provider response before production
normalization and associated guidance. No version bump, release notes or scope
reduction was started, and the provider inquiry draft remains unsent.

Validation passed: changed Markdown local links, 14 JSON examples, public-hygiene
scan and diff whitespace. Disposable placeholder-binary staging verified guide
parity, seven skills per root and all standalone references; it is not a release
binary check. No Rust source changed, so no Cargo build/test was repeated.
No provider or credential access, additional agent, push or publication occurred.


## Cross-tool control investigation (2026-09-23)

The owner explicitly requested further investigation rather than treating the
previous failure as the end of research. Tested a new concrete hypothesis:
a technical-tool-specific or interval-specific problem versus a shared screener
data-path failure. Reviewed the closed wire aliases and request validation;
the dedicated request uses the documented qualified symbol and `1D` interval,
and authenticated catalog validation had accepted its schema.

Added a fixed `technical-control-shape` development operation using the already
approved same-account AAPL `close`, `volume`, `market_cap_basic` request through
`get_symbol_data`. It shares auth, transport and sanitized observation handling
with the technical probe, but does not use the technical tool or an interval
argument. No arbitrary argument forwarding or production command was added.
The test verifies request validation, successful scalar-type observation without
values, and failure classification without raw error leakage.

| Observation | Dedicated technical request (previous) | Basic symbol control (this investigation) |
| --- | --- | --- |
| Wire tool | `mcp-tv-get-technicals-rating` | `mcp-tv-get-symbol-data` |
| Symbol | NASDAQ:AAPL | NASDAQ:AAPL |
| Other arguments | interval 1D | close, volume, market_cap_basic columns |
| Outer HTTP | 200 | 200 |
| Provider response | success false, string error | success false, string error |
| Retained textual hints | rate_limit, screener_endpoint | rate_limit, screener_endpoint |
| Usable observations | none | none |
| Total elapsed | approximately 3.190 s | approximately 73.581 s |

These were separate observations, not simultaneous benchmark samples. The control
catalog completed at 43.824 s; the final HTTP-200 JSON completed at 73.581 s under
the existing private investigation budget. The control therefore reached a
provider application failure rather than expiring locally. No additional
technical timeframe or dedicated-tool retry was issued after this result.

Inference: failure is not confined to the dedicated technical request or its
interval argument; a basic column read for the same symbol is also affected.
This supports investigating the shared official-MCP screener path. It does not
establish that every symbol/tool/account is affected, prove server-internal
behavior, or identify user/IP/server quota ownership or a reset time. No local
argument/normalization fix is supported by this evidence. Directly bypassing
MCP, changing account/source, or fabricating technical values is not a remedy.

### Addendum to the unsent provider inquiry

The same account's `get_symbol_data` request for AAPL and three basic columns
also returned HTTP 200 with `success:false` and error references to rate limiting
and `scanner.tradingview.com`. No interval was sent. Please investigate the common
screener acquisition path and clarify whether the restriction is account-scoped
or upstream-service-scoped. This addendum and the earlier draft are not sent.
A successful basic read would be a concrete availability signal to justify a
new dedicated-tool probe; a provider explanation or response fixture can also
supply new evidence. The dedicated technical normalizer still requires its own
successful schema and must not reuse basic columns as if they were that response.

Validation: the new control/redaction test and incremental proof build passed,
with one Cargo job and one test thread. Formatting, public-hygiene and diff
checks passed. No broad suite, release build, dependency, account mutation,
installed-binary replacement, push or external message occurred.


## Retry-After check and owner-directed wait (2026-09-23)

The owner requested only an outer Retry-After check, then waiting if it supplies
no usable guidance. Private tool-response diagnostics now retain header presence
and parsed seconds even for HTTP 200; raw headers are never retained. Missing
headers produce null seconds rather than the existing local 60-second fallback.
Unparseable values remain present with null seconds. Public errors and cooldown
behavior are unchanged; a diagnostic clock failure cannot change the response.

One approved AAPL daily technical request returned HTTP 200 with completed JSON
at approximately 31.463 seconds under the private investigation budget. Its
Retry-After header was absent. The body still reported a rate-limit failure with
a screener-endpoint reference. This establishes absence only on the outer MCP
response, not on the internal screener response, which remains inaccessible.
There is no provider reset time to report. Stop additional tool reads and wait
as requested; no polling automation or external inquiry was started.

The existing Retry-After parser test and new presence/redaction test passed.
The incremental proof build, formatting, public-hygiene and diff checks passed.
Cargo ran sequentially with one build job/test thread; no broad suite or release
build ran. No account mutation, installed-binary replacement or push occurred.


## Owner-requested retry after 20 hours (2026-09-23 UTC)

The owner reopened the paused read after more than 20 hours. The TradingView
tool was not exposed to this Codex task on this turn, so the established fixed
proof operation used the same approved AAPL daily request and the installed
released binary only as credential worker. An initial sandboxed invocation
failed at local state access without reaching the provider; the authorized
native invocation then completed. That local error is not provider evidence.

The native call reached the official MCP tool once. Initialization, catalog and
tool response all returned outer HTTP 200; the tool result was `success:false`
with only the existing `rate_limit` and `screener_endpoint` textual hints. No
indicator values or successful response shape were obtained. The outer tool
response again had no Retry-After header. The attempt used the existing private
investigation deadline; the tool JSON completed at about 52 seconds, which is
not default-30-second acceptance. These observations do not reveal the internal
screener response headers, quota owner or reset time.

No weekly/monthly/two-hour or basic-column follow-up was dispatched. The owner
requested a retry, not recurring polling; stop the same reads again while this
condition persists. The dedicated technical contract remains unimplemented and
in planned scope. No code, dependencies, account state or installed binary were
changed by this verification.


## Owner deferral and dependency review (2026-09-25)

The owner deferred technical implementation to finish the other work. This
supersedes earlier statements that technical snapshots must remain in v0.33.0.
No replacement feature is promoted. Retain the existing contract proposal and
unsent inquiry; do not poll or retry merely because a dependency changed.

The owner's dependency update selects rmcp 3.4.1 and thiserror 2.0.21.
The [rmcp release notes](https://github.com/modelcontextprotocol/rust-sdk/releases/tag/rmcp-v3.4.1)
and [transport fix](https://github.com/modelcontextprotocol/rust-sdk/pull/1288)
describe fallback after discovery rejection during connection setup. Our
observed failure followed successful initialization/catalog and an HTTP 200 tool
response reporting an internal screener 429. The independent Codex client also
returned that application error. The SDK update is therefore not evidence of a
provider fix; the updated SDK has not been live-qualified in this review.

Remaining order: run affected regression checks with the new lockfile; inspect
current-candidate platform/default-concurrency CI when available; review history
acceptance limits (explicit 90-second empty success, nonempty wire observation
plus fixtures, default-30-second native success unqualified); finish docs and
standalone package qualification; prepare version/notes separately. Preserve
those limits in release documentation rather than claim unobserved coverage.
No live requests or Rust builds were needed for this scope/dependency review.


## Portable skills and offline command specification (2026-09-25)

The owner approved the proposed skill layout and agent-oriented command
specification direction. The PM remains sole executor. This extends the current
version work; it does not reopen technical snapshots or authorize publication.

### Skill source and distribution

Before: runtime sources lived in `.agents/skills`. After: seven standalone
sources live in `skills/`; repository `.agents/skills/<name>` entries link to
those sources and `.claude/skills` retains its shared-root link. Release staging
copies real files from `skills/` into both archive roots. No installed skill
requires its source checkout or siblings. Contributor-only skills stay local
and use internal metadata for npm discovery.

Windows without symlink support uses the documented local `gh skill` install
into user scope, not a second editable tracked copy. No custom sync utility is
needed. Existing ignored local remnants were not promoted to distribution.

Acceptance observed: gh 2.101.0 local install into a disposable directory
installed exactly seven runtime skills; skills 1.7.0 discovery and isolated
copy installation found exactly those seven without contributor skills or
duplicates. Copied npm contents matched the sources; both installations passed
standalone reference checks. All seven skill metadata validations and the
12 package-validator tests passed. Package staging with a
placeholder binary passed parity and standalone reference validation. Remote
installation awaits publication; Windows installer execution is not established
by these macOS observations. CI already stages this layout on Windows/Linux.

### Next implementation: tv spec

Before: agents inspect textual help and skills to assemble commands. After:
`tv spec` returns a compact JSON index, and `tv spec mcp alert history` returns
that command's arguments, constraints, source/prerequisites, contract name and
an argv example in the existing success envelope, with `cli_spec.v1` in data.
Unknown paths return the existing validation-error envelope without I/O.

Use clap's command tree for names, defaults, required flags, aliases, arity and
available enumerations. Do not parse rendered help or duplicate that tree.
Reuse existing validation constants and public contract definitions for facts
that clap does not describe. Return explicit coverage/unknown fields when a
command lacks semantic annotations; syntax discovery must not claim complete
runtime validation. Keep dynamic values out of the static catalog, but identify
the existing discovery command and result field from which to obtain them.
Index every public command and initially qualify useful MCP read detail,
including alert history. Preserve each command's actual source and side effects.

Specification lookup must precede Desktop configuration, credential access and
provider initialization. It needs no account, Desktop, network or new production
dependency. Include the running binary's version; fetch only requested detail,
not the entire catalog by default. Existing commands and output contracts stay
unchanged. The existing `tv discover` probes Desktop internals and is not reused
for this offline operation.

Acceptance: compare the index to clap's public tree; test nested lookup,
unknown paths, inherited options, defaults and supported enums; check history
annotations against actual validation; execute CLI specs with unusable Desktop
configuration and absent credentials to prove offline operation. Compare a few
representative command-selection tasks against help for lookup count and output
size without claiming unmeasured token or latency savings. Keep local checks
focused and serial. Update standalone skills only after the command exists.

`tv schema` and `tv validate` remain follow-on design stages. Their concrete
contract coverage must be established before exposing them; no partial validator
may claim that provider or account conditions were verified.


### Offline spec implementation and acceptance (2026-09-25)

Implemented `tv spec` and nested command lookup with `cli_spec.v1` in the
existing success envelope. Lookup executes before Desktop configuration and
credential/provider setup. Unknown paths produce a normal validation error.
The index omits autogenerated help routes, which otherwise duplicate commands.
Argument metadata derives from the built clap tree, including inherited flags,
recognized types, arity, defaults, aliases, choices and conflicts. Conditional
requirements and adapter validation remain explicitly partial. Alert history
has semantic annotations; other commands return null/unavailable semantics.

History cap/output-contract constants and MCP read-deadline constants are shared
with execution. No request policy or existing output contract changed. The
source is `tradingview_mcp`, matching existing output rather than the shorthand
used in the initial discussion. Dynamic discovery points to search output's
symbol field. Examples parse with the actual CLI. Read metadata also explains
possible local credential renewal.

Scoped unit tests cover every indexed path, inherited flags/defaults, enum
choices, example argv and history request bounds. Subprocess tests exercise
success and unknown paths with invalid Desktop configuration. No provider call,
credential operation, installed-binary replacement or account mutation is used.
Runtime guidance directs agents to request a known path directly and to retain
help fallback for old binaries. Output schemas and offline validation commands
remain unimplemented follow-on work, not advertised as available.

Output-size comparison on this development build (UTF-8 bytes):

- index: 30507 spec bytes, 2625 help bytes
- mcp alert history: 5465 spec bytes, 552 help bytes
- quote: 4015 spec bytes, 1505 help bytes

Each known-path spec and help lookup takes one invocation. These are output-size
measurements, not model-token counts or a behavioral agent evaluation. No token
or latency saving is claimed; spec returns more structured information.

Validation passed: three spec unit tests, two CLI subprocess tests, three history
model tests and the focused MCP whole-operation timeout fixture, all serial.
Scoped CLI library/binary Clippy passed with warnings denied; formatting, diff
and public hygiene passed. Package validator self-tests (12), staged runtime
guidance and both changed skill metadata checks passed. The index contains 195
paths. These checks used the updated lockfile (including rmcp 3.4.1); no full
workspace test, release build, live provider probe or platform CI was run.


### Primary MCP read specifications (2026-09-25)

The owner accepted staged semantic coverage: primary MCP reads first, then
account mutations, then Desktop/other commands. Implemented this first stage for
search, columns, symbol, symbols and bars; history retains its contract and now
shares the same symbol/discovery/deadline helpers. Other semantic annotations
remain unavailable, and validation coverage remains partial. This is not a
schema export or an offline invocation validator.

Before: `tv spec mcp bars` returned clap syntax with null semantics. After: it
also returns `mcp_bars.v1`, authenticated Desktop-free read effects, supported
timeframes/count, rejected date ranges, symbol discovery and an executable argv
example, with source/coverage limitations. Symbol reads expose actual default
columns and field discovery; batch reads distinguish returned/missing/unreported.
Column catalog mode depends on group/search presence, not market alone. Search
query tokens join with spaces before the UTF-8 byte limit is checked.

The model owns shared limits, accepted enumerations, default columns and output
contract identifiers; normalizers and request validation use those same values.
This preserves current request acceptance, source semantics and existing output
contracts. CLI-specific discovery and interpretation live in the spec adapter,
separate from clap metadata extraction. No dependency, account access or provider
probe is needed. Skills and the public spec reference describe the new coverage.

Targeted tests compare declared boundaries with actual model constructors,
including multibyte query length, batch uniqueness/caps and default fields.
All examples parse using the real CLI. Subprocess coverage queries all six
annotated reads with invalid Desktop configuration. Existing model MCP fixtures
cover normalization and request behavior after extracting shared constants.

Validation passed: five spec unit tests, 50 existing model MCP tests, two CLI
subprocess tests (covering all six annotated read paths), scoped CLI Clippy with
warnings denied, package validator self-tests and staging, updated skill metadata,
formatting and diff checks. All Cargo execution was serial with one build job.
No full-workspace suite, release build, provider access or installed-binary
replacement was performed. Remaining semantic stages are account mutations and
Desktop/other commands; unannotated MCP families remain explicitly unavailable.
