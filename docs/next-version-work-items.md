# Work inventory after v0.35.0

Updated 2026-10-05. [Direction](next-version-roadmap.md) ·
[Released contracts and evidence](plans/archives/tradingview-cli-chart-analysis-contracts.md).
The owner approved the diagnostic spec contract below and bounded weekly/monthly
technicals qualification on 2026-10-05. No release version is selected.

## Released baseline

v0.35.0 includes nullable selected-chart OHLCV across raw output, summaries,
exports, and Replay attachments; offline OHLCV and four Pine graphics paths;
exact legacy simple-price threshold projection; dependency refresh; and
standalone distribution guidance. Unknown values must not revert to zero.
Schema describes structure, and validation checks syntax and implemented local
constraints. Neither establishes runtime data quality.

Release metadata lists four native archives and SHA256SUMS. The release workflow
succeeded at d71fd532. Post-tag 9cba16e changes only CLI tests and has successful
CI. The closeout preserves the separate failed tag-CI record. No archive download
or execution was repeated for this inventory update.

## Approved work and remaining candidates

| Order | Work | Evidence / decision gate | Acceptance / boundary |
| --- | --- | --- | --- |
| 1 | Needed technicals interval qualification | Complete for the approved weekly/monthly requests; see the bounded evidence below. Two-hour remains unverified. | One request per interval, no retry or fallback. This evidence does not require a release. |
| 2 | `spec diagnose quote-data` semantics | Concrete examples approved. Implemented in the existing [quote spec module](../crates/cli/src/app/spec/quotes.rs); focused local verification passed; not yet released. | Match actual requests/effects, status variants, and source separation. Spec remains offline; acquisition, schemas, and recovery are unchanged. |
| 3 | Saved-Pine source identity | [Investigation and proposed contract](plans/tradingview-cli-indicator-source-verification.md) demonstrate a local plot-ID revision mismatch and identify an existing saved-source GET. No live miscreated alert was observed; current versioned source retrieval remains UNCONFIRMED. | Review fail-closed behavior for both dry-run and creation. Implementation is not yet approved; release qualification needs an authorized script, local source, and Desktop target. |

Priority 1 qualifies existing timeframe support. Priority 2 covers one diagnostic
path, not all spec gaps. A reproduced analysis defect takes priority over
further annotation work after triage.

## Approved diagnostic spec contract

The existing [market-data guidance](../skills/market-data/references/quotes-and-events.md)
uses `diagnose quote-data` after an unavailable quote-data read. This gives the
annotation a specific consumer path. Existing help already explains that the
sources are separate; the improvement is structured discovery for that path,
not a new diagnostic command or a price-availability fix.

For `tv spec diagnose quote-data`, the pre-change data fragment was:

```json
{"coverage":{"semantics":"unavailable"},"semantics":null}
```

The approved fragment is:

```json
{
  "coverage": {"semantics": "documented"},
  "semantics": {
    "source": "quote_data_diagnostics",
    "output_contract": null,
    "requires": {"desktop": true, "authentication": null},
    "effects": {
      "provider_request": true,
      "chart_mutation": false,
      "account_mutation": false,
      "local_file_write": false
    },
    "constraints": {"symbol": {"trimmed": true, "nonblank": true}}
  }
}
```

These are fragments, not replacement envelopes. Existing syntax metadata,
partial validation coverage, and `cli_spec.v1` remain unchanged. The spec includes
a synthetic invocation example and the following limits:

- Scanner reference acquisition precedes Desktop target discovery, even when
  the Desktop is unavailable. A scanner error remains a separate reference
  result and does not prevent Desktop diagnosis.
- The quote-data reader enables CDP network observation and waits for matching
  events. Its 3.5-second observation window excludes scanner acquisition,
  target discovery, connection, and Network.enable. It is not a whole-command
  deadline. No new symbol subscription or chart switch is performed.
- A successful CLI envelope can contain diagnostic_status blocked or unavailable.
  Inspect that field and quote_data.payload_status. Successful transport alone
  does not establish a price, freshness, or matching event availability.
- Scanner/chart values are not merged. Hints do not execute recovery, and
  quote-data is not added to auto routing. Authentication remains unspecified
  because this command does not establish the Desktop feed's account entitlements.

The owner approved these public examples on 2026-10-05. The implementation
extends the existing quote spec module without a new dispatcher. Source
inspection established execution order and the observation-window boundary.
No schema/validate support or live Desktop acceptance is included in this slice.

Local acceptance on 2026-10-05:

- The new offline CLI contract first failed on semantics:unavailable, then passed
  after implementation. All 38 spec CLI tests and both diagnostic CLI tests passed.
- The quote-spec example parser test and all seven diagnostic unit tests passed.
  The built CLI's output matched every field in the approved JSON fragment.
- Scoped CLI Clippy for the library and both affected integration targets passed
  with warnings denied. Formatting and public/diff hygiene passed.
- The standalone market-data skill reference check and disposable resource
  staging passed with seven skills per root. Staging validates resource parity;
  it is not optimized archive or installed-binary acceptance.

Checks ran serially with one Cargo job and one test thread. The full workspace
suite and upstream CI were not repeated, and no new Desktop operation was run.
The fixed installed binary was not replaced. The implementation is unreleased.

## Bounded technicals evidence

On 2026-10-05, the owner-authorized `NASDAQ:AAPL` weekly and monthly reads each
succeeded with `status: available` and `transport.tool_attempts: 1`. Both
provider-reported symbols and intervals matched their requests. Each response
contained 23 indicator slots, with EMA30 and SMA30 null and 21 numeric values.
Data time, delay, adjustment, session, and finality remained unconfirmed.
No retry, polling, source fallback, or two-hour request was performed.

The installed CLI identified itself as v0.35.0 at 9cba16e, dirty:false, for
aarch64-apple-darwin. This is bounded provider evidence for those requests,
not archive/binary equality, universal availability, a resolution of the earlier
429 cause, or current/final data qualification. No raw response or credential
state is retained in this record. The [model](../crates/model/src/mcp_technicals.rs)
and [transport fixture](../crates/mcp/src/fixtures.rs) remain unchanged.

## Retained qualification gaps and deferrals

- Technicals: daily success is historical evidence. Weekly/monthly success is
  scoped to the bounded observations above. The earlier provider-internal 429
  remains unexplained; two-hour success remains unverified.
- Nonempty native alert history, graphical Linux OAuth, and Windows skill-manager
  execution remain unverified. Fixture and CI success do not replace those runs.
- Legacy alert symbol-marker identity remains unresolved. The released scalar
  threshold projection does not establish safe symbol matching or restore raw
  study-series.
- Remaining hotlist, depth, discover, and spec semantics need demonstrated usage
  value. Offline inspection confirms hotlist and depth semantics are unavailable;
  that is a coverage gap, not evidence of broken acquisition.
- [Chart-read attribution](plans/archives/tradingview-cli-chart-read-latency-attribution.md)
  stays deferred until a chart read exceeds one second, times out, or records
  method_call failure in ordinary operation or a separately authorized run.
  The trigger opens investigation, not retry, timeout, or broker implementation.
- [CDP strategy](notes/cdp-stability-and-autonomous-operation-strategy.md) governs
  reconnect, shared process, public timing/recovery, and renderer proposals.
  Indicator search, 5,000-bar expansion, additional intraday bars, drawing
  geometry, and Windows MSIX/AUMID are not promoted.
- The older v0.33.0 public Release example correction remains an owner follow-up.
  Its repository example is already corrected. Remote editing is separate.

## Working limits

One PM/executor; preserve unrelated work, staged changes, and stashes. Use scoped
serial checks and retain valid historical evidence. Heavy pre-push tests remain
disabled. New authentication, provider/Desktop/account operations, production
dependency additions, and remote writes need explicit authority. Public CLI,
JSON, Rust API, and persisted-format changes require agreed concrete examples.
Before a formal release, recheck and update existing dependencies, explain
resolver constraints, and run the affected checks and normal candidate CI.
