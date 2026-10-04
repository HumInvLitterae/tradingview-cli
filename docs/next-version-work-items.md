# Work inventory after v0.35.0

Updated 2026-10-05. [Direction](next-version-roadmap.md) ·
[Released contracts and evidence](plans/archives/tradingview-cli-chart-analysis-contracts.md).
The owner approved the diagnostic spec contract below and bounded weekly/monthly
technicals qualification on 2026-10-05. The source-verification contract was also
approved later that day. No release version is selected.

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
| 3 | Saved-Pine source identity | [Approved contract and acceptance](plans/tradingview-cli-indicator-source-verification.md). Implemented; focused Rust and generated-JavaScript fixtures passed. | Bounded live dry-run passed for an authorized saved revision after the owner restarted Desktop: exact source and CRLF passed; a textual mismatch was rejected. Normal alert creation was not exercised. Upstream CI and release packaging remain separate gates. |
| 4 | Pine Editor opening repair | A live pine open timed out before saved-script selection. The generic legacy widget API returned without opening the current Pine panel; the existing dedicated button succeeded. | Prefer the dedicated button when present, retain legacy APIs when absent, and preserve the public contract and readiness deadline. Regression failed before the fix and passed after it. Live pine open from a closed editor succeeded with opened_editor:true, slot_rebound:true, and binding_verified:true. |

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

## Pine Editor opening repair

The next-work request on 2026-10-05 was applied to the observed Pine opening
failure. One executor investigated and repaired it without additional agents.
The current page retained bottomWidgetBar.showWidget but had no pine-editor
widget registered. The opener returned after that call and never reached the
existing dedicated Pine button. Clicking that button opened the editor, and
saved-script binding verification succeeded. Source inspection also confirmed
that this API-first ordering predates the source-verification change.

The [shared editor opener](../crates/cli/src/ops/pine/editor/runtime.rs) now
prefers the dedicated button. Legacy APIs remain available when the button is
absent. This preserves the public CLI/JSON contract, existing editor selection,
source checks, and ten-second readiness budget. No new timeout, retry policy,
provider, dependency, or general UI facility was introduced.

The regression executed the production JavaScript under pinned Node 24.18.0.
Before the fix it returned method:showWidget and opened:false despite a working
Pine button. After the fix, the button path and the button-absent legacy path
both passed. The existing Pine JavaScript gate now runs both contract tests.
All 47 non-ignored Pine editor tests, both JavaScript contracts, scoped CLI
Clippy with warnings denied, formatting, and public/diff hygiene passed.

Live verification used the local debug binary and the authorized test script.
Starting with a closed panel, pine open returned editor_open_before:false,
opened_editor:true, slot_rebound:true, and binding_verified:true. No source
editing, save, compile, or alert creation occurred. The installed binary remains
unchanged. Full workspace tests, upstream CI, and release packaging were not
run for this bounded fix.

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

## Task and dependency checkpoint, 2026-10-05

The owner requested an inventory and existing-dependency refresh, followed by a
stop and report. Performance and stability analysis are a later decision, not
part of this checkpoint. The three approved code changes are implemented and
have their scoped acceptance evidence above. No unfinished implementation was
identified in those slices; this is not a new repository-wide defect audit.

Remaining work is separated by purpose:

- Release qualification: normal candidate CI across platforms, optimized native
  archives and package checks, version selection, changelog/release notes, and
  publication remain outstanding. No workflow was dispatched or pushed here.
- Retained live gaps: normal indicator-alert creation/readback, nonempty native
  alert history, graphical Linux OAuth, Windows skill-manager execution, and
  two-hour technicals remain unverified. These are not newly demonstrated bugs.
- Unresolved observations: the earlier technicals 429 and legacy alert symbol
  identity remain unexplained. The earlier launcher returned cdp_ready:false;
  the subsequent manual restart restored connectivity, but no launch defect
  was established. The demonstrated Pine Editor opener defect is fixed.
- Public documentation: reading the v0.33.0 GitHub Release again confirmed it
  still uses tv spec data values. The repository has tv spec values. Remote
  correction remains separate and was not performed.
- Deferred candidates: broader command annotations, chart-read performance,
  transport recovery, and the other candidates above remain unpromoted. No
  optimization or stability-improvement investigation was started here.

Dependency results:

- All 24 direct/dev Cargo requirements match current crates.io stable metadata.
  Cargo update --dry-run resolves zero changes under Rust 1.99.0. Cargo.toml and
  Cargo.lock stay unchanged. generic-array 0.14.9 exists, but crypto-common 0.1.7
  requires exactly 0.14.7 in the rmcp/oauth2/sha2 chain. The installed upstream
  manifest and reverse dependency tree confirm that constraint; no override was
  added. jsonschema 4.26.0 is also current on PyPI.
- JavaScript test pins move from Node 24.18.0 to the current 24.x LTS,
  [24.21.0](https://nodejs.org/en/blog/release/v24.21.0), matching mise.toml.
  Scripts, CI, release workflows, and current development guidance change
  together; earlier acceptance records retain their original versions.
- Existing action refs move to checkout@v7, setup-node@v7, setup-python@v7,
  upload-artifact@v7, and download-artifact@v8. Observed latest releases were
  respectively 7.0.1, 7.0.0, 7.0.0, 7.0.1, and 8.0.1. Existing major-tag style
  is retained; no new action or permissions are added.

Upstream review covered [checkout credential and trigger changes](https://github.com/actions/checkout/tree/v7.0.1),
[setup-node cache and input changes](https://github.com/actions/setup-node/tree/v7.0.0),
[setup-python changes](https://github.com/actions/setup-python/tree/v7.0.0),
[upload archive behavior](https://github.com/actions/upload-artifact/tree/v7.0.1),
and [download extraction and digest behavior](https://github.com/actions/download-artifact/tree/v8.0.1).
These workflows use hosted runners, push/pull_request triggers, no Docker action
or npm package manifest, and no removed inputs. The upload default still wraps
the already-built archive; pattern downloads still merge those files into dist.
The new download digest-mismatch error is adopted without an override. Release
asset names and package formats are unchanged. Native action execution remains
an upstream CI/release check, not a result of local YAML inspection.

Validation: all six JavaScript gates passed under Node 24.21.0, covering ten
executable contracts. YAML parsing and live action manifests confirmed all used
inputs remain supported. Parsed workflow comparison confirmed that only action
refs and Node versions changed, with triggers, permissions, job dependencies,
and artifact paths preserved. Output-schema validation passed with jsonschema
4.26.0 and Node 24.21.0. Public-hygiene self-test, public-hygiene scan, and diff
checks passed. No full workspace suite or optimized build was required because
Rust source and the resolved crate graph did not change. The owner-requested
checkpoint is complete; stop for review before further analysis.

## Working limits

One PM/executor; preserve unrelated work, staged changes, and stashes. Use scoped
serial checks and retain valid historical evidence. Heavy pre-push tests remain
disabled. New authentication, provider/Desktop/account operations, production
dependency additions, and remote writes need explicit authority. Public CLI,
JSON, Rust API, and persisted-format changes require agreed concrete examples.
Before a formal release, recheck and update existing dependencies, explain
resolver constraints, and run the affected checks and normal candidate CI.
