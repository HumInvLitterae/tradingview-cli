# Analysis reliability after v0.33.0

Status: contracts approved 2026-09-29; implementation in progress. The PM is the
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

## Work and validation

Implement the three reviewed contracts as separate usable slices in the order
above. For each, complete focused fixtures, spec/docs/skill updates and diff/public
hygiene checks before committing. Use one Cargo build job and one test thread;
no full local workspace suite or release rebuild by default. Run the smallest
executable JavaScript fixture gate needed for equity. Use candidate CI for broad
platform checks; native/live acceptance remains separately scoped.

Then review legacy watchlist/alert behavior against actual use and explicit MCP
alternatives. Uncertain-write DOM fallback, click-only alert creation and
saved-indicator/source mismatch remain known candidates, not promised fixes.
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
