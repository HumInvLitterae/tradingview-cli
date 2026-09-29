# Analysis reliability after v0.33.0

Status: three approved contracts implemented and locally validated 2026-09-29.
Legacy-account review is complete; the corrections below await contract approval. The PM is the
sole executor; no additional agents or sessions are authorized for this work.

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

## Legacy-account review and proposed corrections

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

Proposed contract fragments (not full envelopes):

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

These changes need owner approval because failure exits and fallback behavior
are public contracts. No dependency or persisted-format change is proposed.
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

## Work and validation

Implement the three reviewed contracts as separate usable slices in the order
above. For each, complete focused fixtures, spec/docs/skill updates and diff/public
hygiene checks before committing. Use one Cargo build job and one test thread;
no full local workspace suite or release rebuild by default. Run the smallest
executable JavaScript fixture gate needed for equity. Use candidate CI for broad
platform checks; native/live acceptance remains separately scoped.

The legacy-account review above proposes the next corrections; implementation
awaits owner approval of those public behavior changes.
Output schemas/offline input validation are later design work, starting with
real high-value chart/data consumers rather than all-command coverage.

A minor release is likely if these additive contracts ship together, but no
next version is committed yet. Keep version/notes preparation last. Publication,
provider inquiries, new live checks and downstream edits need their applicable
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
