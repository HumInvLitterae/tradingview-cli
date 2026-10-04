# Work inventory after v0.35.0

Updated 2026-10-05. [Proposed direction](next-version-roadmap.md) ·
[Released contracts and evidence](plans/archives/tradingview-cli-chart-analysis-contracts.md).
No next-version feature implementation is approved by this inventory.

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

## Proposed priority

| Order | Candidate | Evidence / decision gate | Acceptance if selected |
| --- | --- | --- | --- |
| 1 | Needed technicals interval qualification | [Model](../crates/model/src/mcp_technicals.rs) already accepts 1W, 1M, and 2h; [transport fixture](../crates/mcp/src/fixtures.rs) covers 2h and one-attempt errors. Live symbol, intervals, and authority remain to be selected. | At most one request per approved interval; record provider outcome and unknown data conditions separately. Stop on rate-limit/authentication failure, with no retry or fallback. No release required for evidence alone. |
| 2 | `spec diagnose quote-data` semantics | [Adapter](../crates/cli/src/ops/diagnostics.rs) requests a scanner reference before target discovery; [spec dispatcher](../crates/cli/src/app/spec.rs) currently leaves semantics null. Agree on additive public examples before implementation. | Match actual requests/effects, status variants, and source separation using existing diagnostic and spec fixtures. Spec itself must remain offline. No automatic recovery or new output schema is implied. |
| 3 | Saved-Pine source identity | [Current indicator-alert contract](../crates/cli/src/app/spec/indicator_alert.rs) explicitly lacks source comparison and has other input/readback limits. Need a concrete indicator-alert workflow before promotion. | First establish whether saved source/version evidence can prove the required match; agree on mismatch/unavailable behavior. Do not promise complete alert identity from source comparison alone. |

Priority 1 is verification, not unimplemented timeframe support. Priority 2 is a
proposal to clarify one useful diagnostic path, not approval to fill all spec
gaps. A reproduced analysis defect can displace either candidate after triage.

## Diagnostic spec contract proposed for review

The existing [market-data guidance](../skills/market-data/references/quotes-and-events.md)
uses `diagnose quote-data` after an unavailable quote-data read. This gives the
annotation a specific consumer path. Existing help already explains that the
sources are separate; the improvement is structured discovery for that path,
not a new diagnostic command or a price-availability fix.

For `tv spec diagnose quote-data`, the relevant current data fragment is:

```json
{"coverage":{"semantics":"unavailable"},"semantics":null}
```

The proposed fragment is:

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
partial validation coverage, and `cli_spec.v1` remain unchanged. Add prose limits
and a synthetic invocation example using the existing spec conventions:

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

Implement only after agreement on these public examples. Extend the existing
quote spec module rather than add a dispatcher framework. Verify the spec under
invalid Desktop configuration, without executing the diagnostic command.
Use the existing spec CLI tests, diagnostic payload tests, and blank-symbol
contract; run scoped CLI Clippy, formatting, and documentation hygiene serially.
No schema/validate support or live Desktop acceptance is included in this slice.

## Retained qualification gaps and deferrals

- Technicals: daily success is existing evidence; weekly previously returned
  provider-internal 429. Monthly/two-hour success remains unverified. No new
  provider requests were made during closeout.
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
