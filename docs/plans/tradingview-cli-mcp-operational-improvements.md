# MCP operational improvements and two additional reads

Status: **login guidance and alert history implemented; native history acceptance
and technical response qualification remain open**, 2026-09-23.
Direction and priority live in the [roadmap](../next-version-roadmap.md) and
[inventory](../next-version-work-items.md). This is the single work record for
v0.33.0; the [v0.32.0 record](archives/tradingview-cli-official-mcp-client.md)
is closed historical evidence.

## Outcome, consumers and authority

Deliver alert firing history, official technical snapshots and a more usable
login/upgrade journey through the independent `tv mcp` surface. The owner
accepted the proposed direction and explicitly promoted technical snapshots
into planned scope. Connection reuse is conditional on measurement. Existing
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
