# Analysis reliability after v0.33.0

Status: approved analysis, legacy-account and offline-tool changes are implemented.
Candidate CI, downstream adoption and release qualification remain. The PM is
the sole executor; no additional agents or sessions are authorized for this work.

## Outcome and scope

Help agents distinguish provider failure clues, equity-series meaning and
operation effects without changing providers or inferring missing evidence.
The [roadmap](../next-version-roadmap.md) owns direction and the
[inventory](../next-version-work-items.md) owns priority. The completed
[v0.33.0 record](archives/tradingview-cli-mcp-operational-improvements.md)
retains prior evidence and the deferred technical-snapshot design.

No new dependency, persisted format or live operation is proposed. Keep existing
commands and defaults. The owner approved the following output changes;
this plan is not approval to repair every legacy behavior found during spec work.

## Consumers inspected

In the downstream repository, `tradingview_mcp_bars_observe/inspect.rs` reads
mcp_error.v1, source, code, tool_attempts and optional automatic_retry. It does
not reject additional detail keys. Preserve those values and error exit status;
new clues must not change its rejection/admission decision.

The downstream bridge stores snapshot follow_up_hints as `Vec<Value>` and
`analyze/tradingview_snapshot.rs` forwards them. Its fixture still includes
chart_quote/non_mutating=true. Update consumer expectations when the correction
is adopted; forwarding JSON is not proof that every external consumer is safe.
No typed downstream `data equity` decoder was found in the inspected Rust tree.
Runtime skills and agent workflows remain consumers even without a typed decoder.
Downstream files were read only and will not be changed by this plan.

Upstream owners are [MCP data normalization](../../crates/model/src/mcp_data.rs),
[bar normalization](../../crates/model/src/mcp_bars.rs),
[equity extraction](../../crates/cli/src/ops/data/strategy.rs), and
[snapshot/compare hint types](../../crates/market/src/types.rs).

## Approved contracts

Examples are synthetic fragments, not complete envelopes or observed price data.

### MCP provider-declared failures

First scope: data and bars responses whose JSON explicitly contains success=false.
Keep existing mcp_error.v1, provider_error, stage and tool_attempts. Add the same
closed diagnostic vocabulary already used by the private investigation harness:

Before:
```json
{"code":"provider_error","stage":"tool_response","tool_attempts":1}
```

After, when the provider's error text mentions a rate limit and the known
Screener endpoint:
```json
{"code":"provider_error","stage":"tool_response","tool_attempts":1,"provider_error_hints":{"textual_clues":["rate_limit","screener_endpoint"],"root_cause":"unconfirmed"}}
```

Use a fixed allowlist of clue labels, never arbitrary strings, URLs, account IDs
or nested response content. Non-string or unrecognized error content produces
an empty textual_clues array. A clue describes text, not an observed HTTP code,
quota scope, reset time or retry permission. No new HTTP status is synthesized;
existing transport 429/Retry-After behavior stays separate. Successful payloads
are unchanged. Transport failures, MCP protocol error objects and unrelated
normalizers are outside this first slice unless review expands it explicitly.

Acceptance: fixture success=false with recognized and unrecognized strings,
non-string/nested errors and secret-like input; no leaked text, no code/attempt
changes, and unchanged success normalization. Share classification with the
private proof instead of maintaining two vocabularies. Downstream rejection
fixtures must remain valid. A live request is not needed to validate this logic.

### Equity observations

Retain the current branch order and row shape in the first correction. Add
observed extraction path separately from series meaning; do not rename an
unverified legacy array into a guaranteed strategy-equity curve.

| Existing branch | series_source | series_kind |
| --- | --- | --- |
| `_reportData.buyHold` | `report_buy_hold` | `buy_and_hold` |
| `equityData` | `equity_data` | `unconfirmed` |
| strategy bars | `strategy_bars` | `unconfirmed` |
| performance-only fallback | `performance_summary` | `unavailable` |
| no series/error | `unavailable` | `unavailable` |

Before, a bars row whose observed drawdown is zero:
```json
{"source":"internal_api","data":[{"time":1,"equity":1000,"drawdown":null}]}
```

After:
```json
{"source":"internal_api","series_source":"strategy_bars","series_kind":"unconfirmed","data":[{"time":1,"equity":1000,"drawdown":0}]}
```

For the report branch, existing data stays intact but gains
series_source=report_buy_hold and series_kind=buy_and_hold. Keep data_points,
strategy_context, existing error/summary behavior and other fields. Unknown
values stay unknown; only the known zero-to-null bug is corrected. Removing the
Buy & Hold fallback or preferring another branch would change selected data and
needs a separate before/after decision. More useful guaranteed strategy-equity
selection requires actual evidence about TradingView's internal arrays.

Acceptance: execute the generated extraction JavaScript with synthetic strategy
objects for each branch, including simultaneous branches, empty data, missing
strategy and zero/missing drawdown. Do not rely only on Rust substring assertions.
Update spec and standalone strategy-report guidance to explain these labels.
Fixtures prove branch interpretation, not current live TradingView field meaning.

### Follow-up effects

The existing public Rust docs define hint non_mutating as no chart mutation.
Preserve that meaning: screenshot may create a file while non_mutating stays
true. Correct chart_quote to false because it can switch/restore the chart.
Add explicit effects alongside the existing fields, keeping auto_execute=false.

Before:
```json
{"kind":"chart_quote","non_mutating":true,"auto_execute":false}
```

After:
```json
{"kind":"chart_quote","non_mutating":false,"effects":{"chart_mutation":true,"local_file_write":false},"auto_execute":false}
```

Screenshot after:
```json
{"kind":"screenshot","non_mutating":true,"effects":{"chart_mutation":false,"local_file_write":true},"auto_execute":false}
```

The booleans describe possible effects of the shown invocation, not evidence
that it ran. They grant no permission. Apply consistently to every existing
snapshot/compare hint; read-only follow-ups report both effects false. Packet
level non_mutating remains true because producing hints does not execute them.

Adding required fields to SnapshotFollowUpHint/CompareFollowUpHint would break
Rust struct-literal callers. Prefer derived serialization metadata that preserves
existing public field construction, with focused JSON and Rust construction tests.
If this cannot be achieved cleanly, return the concrete Rust API change for review.
Do not redefine source_category or add automatic execution/fallback.

Acceptance: synthetic snapshots/comparisons covering all hint kinds and partial
packets, explicit chart_quote/screenshot distinctions, stable public struct
construction and unchanged auto_execute=false. Update the downstream stale
fixture through its owner and sync relevant standalone guidance with the change.

## Approved legacy-account corrections

Reviewed current upstream and downstream sources on 2026-09-29. This is source
inspection, not evidence that an account was changed or a failure occurred live.
The downstream bridge still calls legacy watchlist add/add-bulk and alert
list/create. Its archived mutation-adoption record confirms historical adoption;
current invocation frequency remains unmeasured.

| Priority | Finding and consumer consequence | Recommended action |
| --- | --- | --- |
| 1 | [Alert listing](../../crates/cli/src/ops/alert/list.rs) returns transport errors inside successful data; malformed row collections become empty arrays. The downstream alert-create workflow reads that list before deciding what to create, and its decoder ignores data.error. | Return a real CLI error for failed or malformed reads. Apply the same row-validation rule to private list readers used before/after create, indicator-create and delete so malformed readback cannot confirm a mutation. |
| 2 | [Watchlist mutation](../../crates/cli/src/ops/layout/watchlist.rs) permits DOM fallback after a POST failure and can verify membership against another active list when the original disappears. Downstream applies missing symbols through these commands. | Permit fallback only before dispatch; verify only the original list ID. Keep successful output, already-present behavior and bulk partial-result policy. |
| 3 | [Price-alert creation](../../crates/cli/src/ops/alert/create.rs) can fall back to a dialog that never selects the requested condition; created=true means a button click. The downstream decoder does not require a persisted ID. | Remove this automatic DOM creation path. Preserve verified internal-API creation and return its preflight error when unavailable. Do not silently substitute MCP. |

Approved contract fragments (not full envelopes):

- Failed alert list, before: `success:true` with
  `data:{alert_count:0,alerts:[],error:"HTTP 403: Forbidden"}`. After:
  `success:false`, error kind `internal_api_unavailable`, exit 3, and sanitized
  details `{source:"internal_api",phase:"list_unavailable"}`. Malformed rows
  use phase `invalid_response`; a valid empty row array still succeeds. Keep
  existing public Rust function signatures; strict checks belong at the CLI
  adapter and shared private JavaScript boundary. Preserve other transport errors.
- Watchlist POST failure, before: `phase:"mutation_unavailable"` with
  `api_fallback_allowed:true` permits a second DOM attempt. After: the same phase
  with `api_fallback_allowed:false` returns an error without that attempt.
  If the original ID is absent during readback, return `post_check_failed`
  rather than accepting another list's membership. Neither failure implies
  rollback. Keep pre-dispatch fallback for unavailable/unsupported active lists;
  reject post-dispatch fallback even if an inconsistent payload requests it.
- Price-alert preflight failure, before: API failure can become
  `{source:"dom_fallback",price_set:true,created:true}` after clicking Create.
  After: return the API error without opening/clicking the dialog. Verified
  API success retains its existing shape and condition mapping. This reduces
  legacy availability when the private API is unavailable; it removes an
  unverified success path rather than claiming a replacement source.

The owner approved these failure-exit and fallback changes. No dependency or persisted-format change is proposed.
An alternative is fully verifying the DOM condition, identity and persistence;
that would require more UI maintenance and separately authorized native writes.
Do not implement that larger path merely to preserve a misleading success.

### MCP migration and remaining gaps

The [official catalog](https://www.tradingview.com/mcp/docs), checked 2026-09-29,
documents explicit-ID watchlist management and simple-price alert lifecycle.
Adding an existing watchlist member moves it to the end. The active-list tool
can activate/create a list despite its read-only label; it is unsuitable for
non-mutating target discovery. Use list/get and an explicit owner-selected ID.
The documented price-alert surface does not replace Pine alertcondition creation.

Current [tv MCP contracts](../official-mcp.md#explicit-alert-changes)
already support explicit symbol/condition and lifecycle operations. Legacy
`greater_than`/`less_than` mean `cross_up`/`cross_down`; mapping them to MCP
`greater`/`less` would change intent. Legacy notifications and one-shot behavior
also differ from tv MCP defaults. The CLI's MCP slice omits message and expiry
editing even though the provider has those inputs. Retain legacy API access
for consumers needing those fields; do not add message support or drop fields
without a separate consumer/privacy contract.

Downstream must own provider selection, list-ID binding and migration of its
saved results. Its current duplicate-alert matching has a separate mismatch:
`AlertCondition::target_price` reads condition.series[].value, but upstream
sanitization removes raw series. It also does not map cross_up/cross_down in
`normalize_live_condition`. Missing evidence must block duplicate-sensitive
creation rather than imply no match. MCP outputs omit message text, so replacing
that decoder alone cannot preserve its message-based matching policy. No raw
study-series restoration or message retention is proposed upstream. Downstream
files and policies remain unchanged by this review.

Saved Pine script/source equivalence is still unverified in
[indicator creation](../../crates/cli/src/ops/alert/indicator.rs): local source
selects alertcondition metadata while saved metadata supplies the script ID and
version. No direct consumer was found in the inspected downstream Rust bridge.
Keep this as a separate investigation, with verified saved-source identity as
its trigger; MCP simple-price alerts are not a substitute.

### Acceptance and implementation order

Implement approved corrections in table order, with each behavior, its tests,
spec and standalone guidance in the same commit. Execute the generated list
and watchlist JavaScript against synthetic fetch responses, including declared
provider errors, malformed collections, a valid empty list, write-success with
lost response, missing original list and changed active selection. Verify zero
second mutation/DOM calls after dispatch, and no false delete confirmation.
Rust adapter tests must show nonzero error exits reach consumers, supported
pre-dispatch watchlist fallback remains, and price-alert preflight failure does
not evaluate the DOM creation path. Keep existing bulk continuation semantics.
Use the existing pinned Node contract-gate pattern and one Cargo job/test thread.
No native/provider action is required for these deterministic failure contracts;
actual availability and successful live mutations remain unverified separately.

## Approved offline schemas and invocation validation

The initial consumers are the downstream bridge's `fetch_values_summary` /
`fetch_values_rows` and MCP-bars observation decoder. Current code also calls
`ohlcv --summary` and Pine graphics reads; those are the next candidates, not
part of the first slice. No usage-frequency ranking was measured. Keep chart
analysis first by shipping `values` alongside `mcp bars`, then expand from
concrete consumer needs rather than all-command coverage.

### Approved public surface

Add two offline commands without changing data output or existing spec fields.
The clap-derived spec index naturally gains the two new command paths:

```sh
tv schema
tv schema values
tv schema mcp bars
tv validate -- values
tv validate -- mcp bars NASDAQ:EXAMPLE --timeframe 1D --count 20
tv validate -- --target-id example-target values
```

`schema` with no path lists only supported command paths. A path exports the
running binary's schema; it does not fetch a sample. Unknown paths and known
unsupported paths use a validation error with distinct `unknown_command` and
`unsupported_command` codes in cli_schema.v1 details. The index returns
commands as path arrays; a detail returns command, coverage, unchecked and schema.
Successful detail metadata (the schema document is omitted from this fragment):

```json
{
  "success": true,
  "command": "schema",
  "data": {
    "contract_version": "cli_schema.v1",
    "binary_version": "<build identity>",
    "command": ["values"],
    "coverage": "documented_fields",
    "unchecked": ["dynamic_study_values", "runtime_semantics", "error_details"]
  }
}
```

The additional `data.schema` object supplies success/error branches for the existing
CLI envelope; consumers validate stdout success or stderr error JSON against
`data.schema`. Preserve the actual command markers (`values` versus `mcp`) and
startup-error variants. Exported schema versioning does not introduce a new
`values` payload contract or change `mcp_bars.v1`.

Use [JSON Schema 2020-12](https://json-schema.org/draft/2020-12/json-schema-core).
Keep references local to the document and allow additive object properties.
For `values`, describe study_count, studies, name and the normalized identity
fields including nullable inputs/visibility; dynamic values stay unconstrained
where the current reader does not enforce a scalar type. Do not force legacy
provider values into a narrower schema by changing production output.
For MCP bars, describe request/source, bars, provider/client observations,
nullability, count-status alternatives and transport fields. Both short and
empty successful acquisitions must validate. Error envelopes describe common
kind/message fields and object-or-null details, without claiming complete
per-error detail coverage. Date-format annotations are not evidence of market
freshness. Schema conformance never proves series meaning, completeness or
cross-field invariants such as bar_count matching array length.

`validate` requires `--` followed by argv tokens **without** the executable name.
It does not shell-split a command string, execute the target, read input files,
load Desktop configuration, initialize credentials or contact a provider.
No stdin/JSON request form or saved-output validation is included initially.
Target selection belongs inside the candidate argv; reject outer target flags.
Nested help/version and recursive offline commands return a validation error
with code non_executable_request instead of executing them or printing help.
Empty invocations fail syntax validation.

The same argv can be passed to the real command after an explicit decision to
execute it. A valid result means only that implemented local checks passed:

```json
{"success":true,"command":"validate","data":{"contract_version":"cli_validate.v1","command":["mcp","bars"],"status":"valid","checks":{"syntax":"passed","local_constraints":"passed","runtime":"not_checked"}}}
```

Before, `--count 5001` is assembled from prose and rejected during normal
invocation. After, `tv validate -- mcp bars NASDAQ:EXAMPLE --count 5001` returns
exit 1 and stderr JSON without touching credentials:

```json
{"success":false,"command":"validate","error":{"kind":"validation","message":"Invalid command input","details":{"contract_version":"cli_validate.v1","command":["mcp","bars"],"status":"invalid","code":"invalid_request","field":"count","checks":{"syntax":"passed","local_constraints":"failed","runtime":"not_checked"}}}}
```

A known but unsupported candidate, such as `tv validate -- data equity`, returns
exit 1 with status `unsupported`, code `unsupported_command`, syntax `passed`,
local_constraints `not_checked`, and runtime `not_checked`; it never claims the
input is invalid merely because validation is unsupported. Syntax failures use
status `invalid`, code `invalid_syntax`, syntax `failed`, local_constraints
`not_checked`, and a nullable canonical command/field. Do not echo argv, user
values, account IDs, source text, file paths or raw clap diagnostics. Error
messages use a small stable vocabulary; failed queries are not persisted.
A valid request can still fail during execution or lack user authorization.

### Implementation and acceptance

1. Add schema export for these two paths in the CLI, backed by embedded schema
   documents and the current output producers. Keep `tv spec` compact and
   unchanged; route schema lookup in the runner before configuration/credentials,
   as spec already does. Do not generate schemas from prose annotations or turn
   every Value-based output into a new typed public Rust API.
2. Add argv validation for the same paths using the real clap tree and pure
   request construction. Extract/reuse MCP-bars request preparation, including
   date-range rejection, qualified symbols, timeframe/count limits, Desktop
   target rejection and timeout rules. Execution and validation must share
   these checks rather than duplicate constants or maintain a second parser.
   The current MCP client validates its deadline after constructing the client;
   reuse the pure deadline rule before that point without moving auth work into
   validation. Preserve existing execution error ordering when extracting it.
3. Update offline help, `docs/cli-spec.md` and standalone market-data/chart-analysis
   references in each feature commit. Downstream adoption is optional: consumers
   can cache schemas by binary identity and retain help/spec fallback on older
   binaries. Do not add an extra validation process to every downstream read by
   default, change its data admission, or replace its decoders automatically.

Use actual normalizer/adapter fixtures for schemas, including same-name/hidden
studies, unknown identity, malformed dynamic values, short/empty bars and error
envelopes. Validate the schema documents and samples with a standards-compliant
test-only validator; no new production dependency is proposed. Select and pin
test tooling against current versions during implementation. Mutation tests
must reject wrong known-field types and missing required fields while allowing
extra fields and legitimate nulls. Do not build a partial JSON Schema engine.

Invocation tests compare shared request preparation with real execution paths
for counts, timeframes, date bounds, deadlines, aliases and globals. Prove zero
Desktop/provider calls, credential-worker launches and candidate file reads for
valid, invalid and unsupported requests, including --help/--version, nested
validate and a malformed Desktop environment. Test output streams/exit status
and secret-like argument redaction. Keep one Cargo job/test thread and use CI
for broad platform coverage; none of these tests requires live account access.

Embedding schema documents is preferable to a new runtime schema-generation
library for the first two Value-based outputs. The maintenance cost is explicit
schema/fixture review when producers change. Adding all schemas to `tv spec`
would avoid one command but inflate routine lookups. Fully typed output models
or generated argument schemas are larger alternatives without a current need.
The owner approved these two CLI/JSON contracts; both are implemented.

## Resumed official technical snapshots

The owner reopened this deferred feature on 2026-09-30. Reuse the original
[scope and live authorization](archives/tradingview-cli-mcp-operational-improvements.md#official-technical-snapshot),
not its old failure evidence as current availability. Keep the implementation in
this active record; the archived record remains historical.

Daily AAPL qualification succeeded through the updated Rust SDK and the direct
MCP connector. The response is `success:true` with a `data` object containing
symbol, interval, oscillators, moving_averages and summary. Nine oscillator
numbers and twelve moving-average numbers were returned. EMA30/SMA30 were absent
despite the broader official tool description. Summary contains three rating
strings (`recommendation`, `ma`, `other`) and one numeric `value`. No market-data
timestamp, adjustment, delay, session or finality evidence was supplied.
A weekly connector request returned an application-level 429 referring to the
screener provider; no monthly/two-hour calls or repeated weekly attempts followed.
Successful daily retrieval does not establish availability at other intervals.

The owner approved `tv mcp technicals SYMBOL --timeframe 1D` and the concrete
`mcp_technicals.v1` contract. Keep the provider's `summary` object together
(recommendation/value/ma/other); do not introduce ratings/rating_value mappings.
Reuse provider_observation evidence and client_observation.received_at rather
than another timestamp/conditions format. Emit 23 indicator slots using provider
keys, null for missing values, and available/empty status. Requested and reported
identity stay separate; malformed containers/types or contradictory echoes fail.
See [the complete contract](../official-mcp.md#official-technical-snapshots-next-version).

Implementation uses one model-owned request/normalizer and the existing MCP
service. The production tool replaces the proof-only validator; fixed opt-in
proof operations remain bounded. CLI, spec and standalone guidance are updated.
Focused fixtures cover null/missing/empty/zero, malformed fields/identity,
provider failures and no replay. Preserve existing bars timeframes and routes;
no historical series or local indicator calculation is introduced.

The public-service daily proof succeeded with one tool attempt: identity and
interval echoes matched, 21 of 23 indicator slots held values, and summary kept
three strings plus a number. Four model tests, four MCP technical/proof tests,
41 spec tests and two focused CLI contract tests passed. Native evidence is
daily-only; no weekly retry, monthly/two-hour call or account mutation followed.
Scoped CLI/model/MCP Clippy, formatting, public hygiene, standalone skill
validation and placeholder package staging also passed. Platform CI remains pending.

## Work and validation

Implement the three reviewed contracts as separate usable slices in the order
above. For each, complete focused fixtures, spec/docs/skill updates and diff/public
hygiene checks before committing. Use one Cargo build job and one test thread;
no full local workspace suite or release rebuild by default. Run the smallest
executable JavaScript fixture gate needed for equity. Use candidate CI for broad
platform checks; native/live acceptance remains separately scoped.

Implement the approved legacy-account corrections above before schema design.
The two-command schema/validation implementation is authorized. No production
dependency change is needed.

Recommend v0.34.0 for the three additive commands. The
[release-note draft](../releases/v0.34.0.md) covers the settled scope. The workspace
version remains 0.33.0 until owner confirmation; keep final version preparation
after candidate CI. Publication, provider inquiries, new live checks and downstream edits need their applicable
explicit authorization. Do not repeat provider probes while the known failure
persists merely to advance this plan.

## Progress

- v0.33.0 publication verified and historical record archived.
- Current source and downstream error/hint consumers inspected read-only.
- Owner approved all three output contracts. MCP diagnostics implemented; focused
  normalization/privacy and private-proof checks cover the shared classifier.
- Equity extraction and unavailable outcomes now identify source/meaning; zero
  drawdown is preserved. Generated-JavaScript fixtures cover branch precedence,
  all output paths and missing values without asserting live series semantics.
- Snapshot/compare hints serialize chart/file effects without new required Rust
  fields. Market fixtures cover every built-in kind, partial packets and existing
  construction. Downstream adoption still needs its stale hint fixture updated.
- Scoped model, MCP service/proof, CLI spec/strategy and market tests passed;
  the equity gate passed with pinned Node 24.18.0. Scoped Clippy, formatting,
  public hygiene, standalone skill references and placeholder package staging
  passed. CI/platform and live Desktop/provider execution were not rerun.
- Legacy alert-list validation is shared across read and mutation paths; generated
  JavaScript fixtures distinguish valid empty lists from failed/malformed reads.
- Watchlist fallback now requires a known pre-dispatch failure. Readback requires
  the original list; generated-JavaScript fixtures cover lost responses and
  changing/missing targets, and adapter fixtures reject contradictory flags.
- Price-alert creation now uses only the verified API path; preflight failures
  do not evaluate dialog code. Existing successful API payloads, conditions and
  notification defaults remain unchanged. Next: downstream duplicate-matching
  corrections and adoption, candidate CI, then release qualification. The first
  schema/validation slice is implemented below; saved-Pine identity work remains
  a separate candidate.
- Legacy corrections passed focused alert/watchlist/model/spec tests, both
  pinned-Node account gates, scoped CLI/model Clippy, formatting, public hygiene
  and standalone/package skill checks. No live mutation, full local workspace
  suite, platform CI or release build was run for these corrections.
- Schema export is implemented for values and MCP bars. Embedded documents are
  used because these producers build Value outputs; serde-derived schema alone
  would leave their payloads unspecified. No custom schema engine or production
  dependency was added. python-jsonschema 4.26.0 was verified against PyPI and is
  pinned only for fixture validation.
- Invocation validation reuses clap and the execution-path bars/target/deadline
  rules. Native clap partial matches identify malformed outer validate requests
  so diagnostics can omit input values; no handwritten argv scanner or second
  command definition was introduced. Request preparation remains before client
  construction and execution deadlines remain after it, preserving error order.
- Both offline commands are implemented. Candidate CI, downstream adoption and
  next-release qualification remain; further schema/command coverage is deferred
  until a concrete consumer needs it.
- Dependency refresh: rmcp 3.5.0 and tokio-rustls 0.26.6. The initial protocol pin
  was removed after owner review: preserving old behavior alone did not establish
  a compatibility requirement. The SDK now advertises 2026-07-28; fixtures verify
  both older-server negotiation and current-version replies with request metadata.
  The existing initialize lifecycle remains; discovery-first behavior is not
  selected automatically. SDK null-result decoding preserves null instead of
  using text content. Other direct Rust dependencies are current; crypto-common
  requires generic-array exactly 0.14.7.
  Both negotiation fixtures and scoped MCP Clippy passed. A live batch read with
  the SDK default returned both valid symbols and marked the invalid control
  missing, with one tool attempt. This verifies retrieval with the updated
  client, not the negotiated server version or the cause of earlier 429 failures.
- Offline validation passed three unit tests and five executable contract tests;
  all 41 spec and ten MCP execution-contract tests passed. The schema gate passed
  with production fixtures, field mutations and self-contained standard schema
  references. Scoped CLI/MCP Clippy, formatting, public hygiene and standalone
  skill/placeholder-package checks passed. These are local fixture checks, not
  platform CI or live Desktop/provider acceptance.
- Release-scope review: the approved implementation slices are complete with
  focused evidence. Candidate CI remains pending; the latest remote success still covers
  v0.33.0. The v0.34.0 notes are a draft, not publication evidence. Preserve the
  known native/platform limits rather than restarting unrelated live probes.
