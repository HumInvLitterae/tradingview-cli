# Work inventory after v0.36.0

Updated 2026-10-05. [Proposed direction](next-version-roadmap.md).

## Current status and next queue

v0.36.0 is published at 5c72c081b8e0539454a8be0e001702735656585e.
GitHub release metadata, four uploaded native archives plus SHA256SUMS, and
successful CI and Release runs at that commit were checked on 2026-10-05.
No archive was downloaded or executed in this planning pass.

The diagnostic spec, saved-source guard, Pine Editor repair, and bars-frame fix
below are released. Statements about pending publication or CI in the historical
sections describe their original checkpoints and are superseded by this status.
Live qualification gaps are not closed by publication.

| Priority | Proposed work | State / next decision |
| --- | --- | --- |
| 1 | Indicator-alert unknown creation outcomes | Approved and implemented locally. Production-JavaScript regression reproduced the incorrect false result; see acceptance below. |
| 2 | Exact study and input identity | Bounded read-only observation completed. Native identity/version/condition fields exist; positional input remapping reproduced a wrong payload with synthetic data. The owner approved the bounded input-ID correction; implementation and acceptance follow below. |
| 3 | Supported identity and creation readback | The authorized one-study compile/read/cleanup trial passed for a no-input saved indicator. Full identity behavior, input-bearing qualification, and creation readback remain separate. |
| 4 | Release the completed slice | Version remains provisional: defect-only v0.36.1 or additive v0.37.0, selected after scope review. |

The owner approved priorities 1 and 2 on 2026-10-05. Priority 3 remains
conditional on demonstrated identity support and agreed public examples.
No additional agents are planned. The roadmap owns direction and contract
examples. This inventory records the bounded correction and investigation below.
Historical v0.36.0 acceptance remains unchanged.

## Approved outcome correction and identity investigation, 2026-10-05

Scope: represent unknown indicator-alert creation outcomes accurately, and
investigate existing study/input identity facilities. Preserve confirmed-success
and preflight behavior. No new selector, stronger identity promise, automatic
retry, account operation, dependency, or publication is included.
Completed authorized work is committed in coherent batches after verification.
One executor performs the work without additional agents.

### Implementation and contract

The [adapter](../crates/cli/src/ops/alert/indicator.rs) now reports created:null
and creation_outcome:unknown for create_request_unavailable,
create_request_failed, post_list_unavailable, and post_check_failed. HTTP/provider
failure does not establish absence of a write. The Rust evaluation boundary also
marks lost, exceptional, or malformed results unknown while preserving the
original error kind, exit code, and available details such as failure_stage.
Only an explicit returned preflight result retains created:false. Source
verification fails before this creation evaluation and keeps its original errors.
Confirmed success and dry-run retain their existing shapes.

The native Runtime already exposes errors and the existing normalizer already
preserves error details. Reuse them; a new outcome framework, persistent journal,
transport retry, or model-crate API is unnecessary. Update the spec, CLI guide,
both distributed indicator-alert references, and changelog with the same meaning.

### Acceptance

The new production-generated JavaScript regression failed before the correction
with `post_list: creation cannot be ruled out` and `false !== null`.
A separate Runtime-loss regression failed because created was absent rather
than null. After correction, the JavaScript matrix covers failed readback,
no matching alert, fetch rejection, response-body failure, HTTP failure,
provider error, pre-list failure, and confirmed success. Each dispatched case
issues exactly one POST; pre-list failure issues none. Returned results also
pass through the real Rust adapter, normalizer, and error envelope.

Runtime fixtures cover timeout, connection failure, and evaluation error with
failure_stage retained, plus null, empty, and non-object results. Source mismatch
and unavailable saved identity still stop before any alert operation.
Focused acceptance passed with one Cargo build job and one test thread:

- Alert Rust module: 33 passed; three JavaScript tests were skipped here and
  exercised through the dedicated gate.
- Pinned Node 24.21.0 account JavaScript gate: four passed, including the new
  eight-case production-expression matrix.
- Indicator-alert offline spec integration test: one passed. The built binary's
  spec output also includes both new error fields with TV_CDP_PORT invalid.
- Scoped CLI library/spec-target Clippy passed with warnings denied.
- Formatting, diff checks, public-hygiene self-test and tracked-file scan,
  14 package-checker self-tests, both affected standalone skill references,
  and 28 changed-document local link targets passed.

No full workspace suite, live creation, notification, Desktop read, normal CI,
or optimized archive qualification was performed. This is local fixture and
offline acceptance, not live creation acceptance. The correction is committed as
28a8b07. Priority 1 is complete locally; priority 2 has source findings and
now has the bounded observation below; the next correction needs scope review.

### Identity feasibility findings

| Required fact | Existing facility and evidence | Conclusion |
| --- | --- | --- |
| Select one chart study | [Indicator reads](../crates/cli/src/ops/data/indicator.rs) use getStudyById(entity_id); indicator edits already accept entity IDs | Existing native selection can be reused. Current live availability is not requalified. |
| Read inputs by their actual IDs | [Indicator edits](../crates/cli/src/ops/indicator.rs) use getInputValues entries by input.id; the pre-correction alert adapter assigned returned positions to in_N | Native input IDs are available in existing code paths. Their mapping to alert payload keys needs qualification before replacing the positional mapping. |
| Reuse existing public output as a complete input payload | [Study-value shaping](../crates/cli/src/ops/data/study_values.rs) caps and filters inputs; indicator get also filters long text/string values | Unsuitable as a complete creation payload. Reuse native reads, not the intentionally lossy public summaries. |
| Tie a chart study to the saved revision | The saved-script preflight identifies catalog ID/version, while the chart path matches labels | The required chart-study-to-saved-version relationship remains UNCONFIRMED. Entity ID alone does not prove it. |
| Verify compiled condition identity | Local Pine parsing produces best-effort plot candidates; current creation does not read compiled condition metadata | Source equality does not establish this relationship. Native metadata availability remains UNCONFIRMED. |
| Identify exactly the created alert | Current readback searches new list IDs by condition/message and optional symbol, without binding a create-response ID | Concurrent matching alerts and full saved-version/input/resolution confirmation remain outside the completed correction. |

### Bounded observation and reproduced input defect

On 2026-10-05, the owner requested the next work after the read-only observation
had been identified as the next step. One executor selected the existing active
test layout from the Desktop target inventory and confirmed readiness. The
installed CLI identified v0.36.0 at 5c72c081, dirty:false. No archive equality
claim is made. Custom metadata reads used the existing explicitly enabled
ui eval path for those processes only; no persistent configuration changed.
An initial top-level eval lookup was invalid, and the first ui eval call was
rejected by its default opt-in guard before execution.

The bounded sequence read metadata from one chart and issued one saved-catalog
GET. It did not save, compile, insert, edit, switch, create, delete, or retry an
account mutation. No source values, account IDs, raw responses, or target IDs
are retained here. Aggregate observations:

- All 24 observed Pine studies exposed scriptIdPart and pine.version.
  Their pineId/pineVersion input fields agreed with those metadata fields in
  all 24 cases. Top-level meta.version differed from pine.version in all cases;
  the inspected example used engine metadata version 101. Do not substitute it
  for the saved Pine revision.
- User input IDs matched declared metadata IDs in all 24 observed studies.
  Returned inputs included system fields before in_0. In one inspected shape,
  text, pineId, pineVersion, and pineFeatures preceded four user inputs; the
  array ended with __fast_calc and __profile.
- Four studies exposed alertcondition plots, with 2, 6, 6, and 6 conditions.
  All observed condition IDs had the plot_N shape and had titles in styles.
  This confirms metadata availability on this chart, not source-to-compiled
  condition equivalence for a selected saved test script.
- No chart study's scriptIdPart matched the saved catalog. Therefore a complete
  saved-script-to-chart-version qualification remains UNCONFIRMED. Do not add a
  study merely to fill this evidence gap without authorization.

Before correction, the adapter ignored returned input IDs and assigned each
array element by position to in_N. A temporary Rust test used the existing FakeRuntime to
capture the exact production-generated creation expression, then executed it
under Node 24.21.0 with synthetic chart inputs and intercepted fetch. No real
POST was made. The relevant synthetic sequence was:

```json
[
  {"id":"text","value":"synthetic-source"},
  {"id":"pineId","value":"synthetic-id"},
  {"id":"pineVersion","value":"4.0"},
  {"id":"pineFeatures","value":"{}"},
  {"id":"in_0","value":20},
  {"id":"__fast_calc","value":false},
  {"id":"__profile","value":false}
]
```

The assertion that the outgoing in_0 equals 20 failed with actual
synthetic-source. Thus the production mapping relocates a system text value
into the first user input for this observed shape. This is a reproduced payload
construction defect, not an observed wrong live alert or disclosure of real
script content. The temporary failing test was removed after capturing the
result; no executable change was committed with that investigation. The approved
correction below reintroduces the regression through the account-JavaScript gate.

### Approved input-ID correction

The owner approved these before/after examples on 2026-10-05. Fix input-ID
handling before introducing any new study selector. Keep the existing source
guard, successful envelope, no-retry rule, and account-operation limits.

| Case | Before correction | Approved behavior |
| --- | --- | --- |
| System fields precede in_0=20 | System text becomes outgoing in_0; 20 shifts to in_4 | Preserve in_0=20 by ID; never copy text/pineId/pineVersion into user-input slots |
| User inputs are returned in a different order | Values move to different parameter IDs | Match declared user-input IDs and retain each value under its own ID |
| A declared user input is absent or duplicated | Positional assembly can proceed | Stop before alert listing/creation with study_input_metadata_unavailable |
| Metadata is absent or the input shape cannot be verified | Completeness is not established | Stop before creation rather than infer positions or fill defaults |
| input_metadata.input_count | Counts all returned entries, including system fields | Count the verified user inputs placed in the request |

This approved change intentionally tightens admission and changes the count's
meaning. Preserve supported scalar and array values rather than imposing a new
arbitrary type policy. Reuse native
meta.inputs/getInputValues reads; lossy public summaries are not a payload source.
Keep generated base metadata under its existing handling. Tests must reject
missing/duplicate IDs and prove internal text is absent from outgoing user slots.
The existing production-JavaScript harness can qualify construction without a
live account write. Normal creation acceptance remains a separate authorized run.

Implementation uses the existing native meta.inputs/getInputValues boundary.
Both arrays reject duplicate/malformed IDs. The six observed system IDs are
recognized but never assigned to user slots; only declared in_N entries with
actual value or the existing val alias are copied. The old defval fallback is
removed. Unknown IDs fail closed rather than acquiring a new implicit mapping.
The generated base pineFeatures/__fast_calc/__profile values remain unchanged.
No new public selector, helper service, dependency, or transport behavior is added.

The permanent production-JavaScript regression first failed with synthetic-source
at in_0 and the intended 20 at in_4. After correction it covers 17 cases: system
prefixes, reordered/sparse IDs, zero/false/string/array values, the val alias,
zero user inputs, missing/duplicate/undeclared inputs, default-only values,
unavailable or malformed arrays, unsupported IDs, and throwing native getters.
Accepted cases retain exact user IDs and values, omit system text, count only
user inputs, and issue one POST. Rejected cases issue neither list nor create.
Results pass through the real Rust normalizer. The test is a child of the
existing indicator tests and runs in the existing pinned account-JavaScript gate.
Focused validation uses one Cargo job and one test thread:

- Account JavaScript gate: five passed, including all 17 input cases.
- Alert Rust module: 33 passed, with four JavaScript tests left to the dedicated
  gate. Indicator-alert offline spec integration: one passed.
- Package checker: 14 self-tests passed; both affected standalone skill
  references passed. Public hygiene self-test/scan, formatting, diff checks,
  and 29 changed-document local links passed.
- The first CLI integration attempt failed during linking with ENOSPC. After
  removing only inactive older CLI incremental caches, the same scoped command
  passed. Current binaries and source were retained; no full clean was used.

Scoped CLI library/spec-target Clippy passed with warnings denied. The built
offline spec describes ID preservation and the corrected input count with an
invalid Desktop port. No new live Desktop/provider/account operation, full
workspace suite, optimized archive, or CI run is included.
Fixture acceptance does not qualify normal creation or complete saved-study
identity. The installed binary remains unchanged.

Do not expand this correction into a new --study-id flag, saved-version binding,
or full post-create identity proof. Those require a saved test indicator on a
chosen chart. Creating/saving or adding such an indicator would change state;
its exact source, instances, input settings, and cleanup must be agreed first.
A successful metadata read alone does not authorize those actions.

### Saved-study observation and acceptance

On the owner's next-work request, read-only target discovery confirmed the same
active test layout. A saved-catalog read found two existing test indicators.
Their exact saved revisions were read without exposing source text or IDs in
tracked files. Local production pine alertconditions parsing reported one
candidate at plot_1 after one preceding output for each script. Both sources
contained no input-call tokens and declared indicators rather than strategies.
This is static source evidence, not successful compilation or current study
binding. No saved source, editor selection, chart study, or alert was changed.

Use one of these existing saved indicators for the next bounded trial. This
avoids creating or overwriting cloud source. The concrete script name, saved
revision, and target are selected privately with the owner, not embedded here.
The owner explicitly approved opening/selecting that saved script in Pine,
one normal compile that may add a chart study, and removal of only the verified
new study. The procedure below was executed once; it is not standing authority
for another target or script. No source edit, save, new script, input edit, or alert operation is
included. The editor may remain open with the test script selected; existing
unsaved work must not be discarded to start the trial.

1. Reconfirm the explicit target, exact saved revision, and absence of a matching
   study. Inspect existing editor state first. If switching could discard
   unpreserved edits or binding cannot be established, stop before switching.
   Capture the pre-trial study identities in memory.
2. Open the approved saved script through pine open and require slot_rebound and
   binding_verified. Compare editor source with the saved revision without
   modifying either. Use normal pine compile once; do not use raw-compile,
   save-related actions, or retry after an uncertain dispatch.
3. Require exactly one newly added study and match its scriptIdPart/pineId and
   pine.version/pineVersion to the selected catalog revision in memory. Do not
   use top-level meta.version as the saved revision. Compare the compiled
   alertcondition ID/title with the local candidate and confirm the empty
   declared user-input set. Do not call alert create-indicator, even as a
   substitute for this metadata observation.
4. Remove only that new entity after its identity is verified and the owner has
   authorized cleanup. Check that it is absent and all pre-existing study IDs
   remain. If multiple additions, unknown identity, or an uncertain operation
   prevent safe cleanup, stop and report the remaining state; never guess an ID.
5. Retain aggregate match/failure results only. A successful trial qualifies one
   no-input script on one target. It does not cover non-default inputs,
   duplicate-name selection, saved-revision mismatch, full create readback, or
   notification delivery.

Acceptance on 2026-10-05:

- The active editor's source matched its own saved revision before selection.
  No unsaved edits were discarded. pine open returned slot_rebound:true and
  binding_verified:true, and the selected editor text matched the approved
  saved revision before compilation.
- Normal pine compile ran once and returned zero errors. Study count increased
  from 27 to 28. Exactly one new entity was present; every original entity
  remained. Symbol and resolution were unchanged.
- The new study's meta.scriptIdPart and pineId input both matched the selected
  saved ID. Its meta.pine.version and pineVersion input both matched the selected
  saved revision. These comparisons occurred in memory; identifiers are not
  retained in this record.
- Exactly one compiled alertcondition had the expected plot_1 ID and matching
  title from styles. Declared and returned user-input counts were both zero.
  This agrees with the production local parser for this source only.
- The verified new entity was removed once through indicator remove. Readback
  confirmed its absence and exactly the original 27-entity set. Symbol and
  resolution remained unchanged. A saved-catalog/source re-read confirmed
  the saved revision and source were unchanged. Editor source also still matched
  the selected saved source; the editor remains on the approved test script.

No source edit/save, input change, raw compile, alert request, or mutation retry
occurred. The run used the installed v0.36.0 CLI for editor/indicator operations
and bounded metadata evaluations for identity comparisons. It does not validate
the unreleased alert input fix through a real creation request. Temporary local
recovery material is removed after successful cleanup and documentation checks.

This closes native metadata feasibility for one saved no-input indicator:
existing facilities can connect the saved revision, chart entity, and compiled
condition without relying on display-name equality. It does not establish a
public selector policy or full alert-create readback. A later input-bearing or
duplicate-study trial needs its own source/instance/input examples; do not imply
this trial covers it. The next implementation decision can now use demonstrated
native identity fields, with missing or ambiguous identity handled explicitly.
No new CLI flag or automatic admission change is introduced by this observation.

## Historical v0.36.0 preparation evidence

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

## Performance and stability follow-up, 2026-10-05

The owner approved analysis and demonstrated improvements after the dependency
checkpoint. One executor continues without additional agents. Inspect existing
measurement and boundary behavior first, reproduce concrete failures with local
fixtures, and apply bounded repairs. New live targets, public contracts, shared
sessions, retries, and publication are not implied by this scope.

Source review revisited the prior chart-read/topology deferrals, current CDP
HTTP and WebSocket deadlines/event limits, MCP session reuse/deadlines, launcher
behavior, and Desktop-free bars parsing. No new chart-read timing evidence
justifies promoting a broker, retry policy, or status snapshot change. macOS
pkill excludes ancestors by default, so a launcher self-kill is not established
by the full-argument match alone. The earlier launch observation remains
unresolved; no restart was performed.

The bars parser calculates an unchecked payload end from a remote length and
then slices a UTF-8 string. Before the fix, both new parser regressions failed:
a maximum usize length panicked with "attempt to add with overflow", and a
length ending within a multi-byte character panicked with "not a char boundary".
The production fetch path also panicked when the existing loopback fixture sent
the oversized frame. These are synthetic-provider reproductions, not observed
TradingView service incidents.

The repair uses str::get on the remaining payload instead of calculating an
unchecked absolute end. This standard facility validates both range and UTF-8
boundaries before the parser advances. It reuses the existing truncated-frame
error, InternalApiUnavailable kind, and protocol-stage propagation without a
new guard service, dependency, source fallback, or public field.

Acceptance: all 42 deterministic bars tests passed, including both regressions,
the loopback production-fetch test, valid Unicode in text and binary frames,
combined frames, heartbeat handling, and existing availability/range contracts.
The live heartbeat test stayed ignored. Scoped market Clippy with warnings
denied, formatting, and public/diff hygiene passed. Tests ran serially with one
build job and one test thread. No live provider/Desktop operation, optimized
build, full workspace suite, or performance benchmark was run. No speedup or
new service-wide stability rate is claimed. The bounded follow-up is complete;
normal candidate CI and native package qualification remain the next gates.

## Offline timing follow-up, 2026-10-05

After the owner requested measurement, build 430f0bf once with cargo build
--release --locked -p tradingview-cli --bin tv using the existing target directory
and one job. The installed binary remains unchanged. Run three offline commands
20 times per binary, alternating old/candidate order and rotating command order,
after one excluded warmup for each command/binary pair. Measure subprocess spawn,
CLI execution, and captured output with a monotonic clock; validate JSON after
the timer. Successful content must match across runs, excluding the separately
verified embedded binary-version field. All 120 measured invocations succeeded.

The current optimized candidate results, in milliseconds:

| Command / synthetic input | Median | p95 | Min–max |
| --- | --- | --- | --- |
| spec ohlcv | 9.705 | 11.924 | 8.515–14.032 |
| pine alertconditions, 3 lines / 1 candidate | 8.818 | 16.350 | 7.620–19.163 |
| pine alertconditions, 222 lines / 20 candidates | 9.129 | 12.321 | 8.403–14.654 |

The synthetic inputs contain a version directive, indicator declaration, and
one alertcondition for the small case. The larger case has the two declarations,
200 simple arithmetic assignments, and 20 distinct alertconditions. Output
validation checked exact candidate counts and stable complete payloads.

The fixed 9cba16e binary had medians 9.560, 8.765, and 9.038 ms respectively.
These are reference observations, not a controlled build comparison: its build
flags were not re-established. The host has ten logical CPUs, one-minute load
was approximately 5.3–5.5, and existing applications were left running. No
profiler or isolated cold-start run was performed, so these values do not
identify a performance limiter or justify an optimization. Raw timings and the
local reproduction script are retained outside tracked files.

This measures warm-cache local CLI work only. It does not measure provider or
Desktop latency. The useful next measurement is the existing test chart's
OHLCV summary and study values, with a fixed target and bounded read count.
The owner subsequently approved that read-only scope; results follow below.

## Bounded Desktop timing, 2026-10-05

With explicit owner approval, confirm the existing test chart through tab list
and run the optimized 430f0bf binary with that fixed target. Alternate ohlcv
--summary --count 20 and values for 20 rounds, without extra warmups or retries.
The script limits each subprocess to 12 seconds and the whole run to two minutes,
and stops at the first command/validation failure. It measures spawn through
captured output; JSON/schema validation runs outside the per-command timer.
No chart setting, foreground state, script, or account state was changed.

| Operation | Success | Median ms | p95 ms | Min–max ms | First call ms |
| --- | --- | --- | --- | --- | --- |
| OHLCV summary, 20 bars | 20/20 | 16.836 | 22.951 | 14.124–57.923 | 57.923 |
| Study values, 13 studies / 27 value fields | 20/20 | 19.814 | 35.058 | 14.846–44.918 | 35.058 |

All 40 invocations and their output validation completed in 0.882 seconds.
Every summary contained 20 bars; every values response contained 13 studies and
27 value fields. The checked chart/study identities stayed unchanged throughout
the run. Responses passed the existing output schemas; no raw prices, study
identifiers, target identifiers, or account payloads are retained in this record.
The local measurement script and per-call durations remain outside tracked files.

This was one already-running Desktop session on the same ten-logical-CPU host,
with a one-minute load average around 5.9 and existing applications left running.
The first call is included in the distribution; the Desktop session and OS
caches were not reset. p95 uses the nearest-rank sample. No comparison binary or
phase profiler ran against Desktop, so the result establishes observed total
latency, not a speedup or causal attribution.

No call exceeded the prior investigation's one-second trigger, timed out, or
returned a transport failure. This bounded measurement does not justify a new
retry, broker, timeout, or topology optimization. It does not qualify cold app
startup, other charts, symbol changes, longer sessions, or provider/MCP latency.
Retain the existing trigger-based defer and proceed toward candidate CI and
native package qualification rather than adding an unmeasured optimization.

## Local v0.36.0 preparation

The next-work request covers local release preparation. Use v0.36.0 for the
candidate because it includes additive diagnostic semantics and tighter
indicator-alert admission, not only a patch-level defect correction. The
[release notes](releases/v0.36.0.md) describe those user-visible changes and the
Pine opening/bars parser fixes. This does not authorize push, tags, workflow
dispatch, or publication.

Workspace versions and release notes are prepared locally. After stopping the
resource-heavy baseline, the owner authorized resumption with completed tests
omitted. Keep the one-job/one-test-thread limit and do not repeat broad local
checks. Fixed installed binaries and saved stashes remain untouched.

Candidate validation is accounted for as follows:

- Before interruption, 24 test executables completed with 802 passing tests and
  24 ignored tests. Those executables were not run again.
- The 13 unfinished executables completed with 316 passing tests, zero failures,
  and nine ignored tests. They ran directly from compiled artifacts, one at a
  time, in their package directories. Artifact enumeration with Cargo --no-run
  rebuilt only the CLI once; the remaining execution did not invoke Cargo.
- The 42 bars tests from the prior stability fix were reused because the source
  and external dependencies are unchanged. That prefix was filtered from the
  resumed market suite. Its live heartbeat test remained ignored. The combined
  unit/integration evidence covers 1,160 passing tests; this is combined
  evidence, not a claim that one uninterrupted workspace run completed.
- Rustdoc examples were not rerun. Their owning market/scanner library files
  and public API declarations are unchanged from the successful 9cba16e CI
  baseline. Normal candidate CI will cover doc-tests again.
- Full-workspace Clippy with warnings denied, formatting, package-checker and
  public-hygiene self-tests, and document links passed before interruption.
  The unchanged Node 24.21.0 JavaScript and schema evidence above is reused.

The final resolver recheck found zero available Cargo updates; the upstream
exact generic-array constraint remains. Cargo metadata confirms all eight
workspace crates at 0.36.0. Cargo.lock changes only those workspace versions.

Resource staging passed with seven runtime skills per agent root, complete
references, and guide parity. The staged debug binary reports 0.36.0, matches
the build output's SHA-256, and runs diagnostic spec and OHLCV schema commands
from outside the repository. Its embedded provenance is ff9e252-dirty because
it was built from the candidate draft before commit. This qualifies resources
and standalone execution, not optimized archive acceptance. No new optimized
build or native archive was run; those remain CI/release qualification work.

Local candidate preparation is complete within the resource constraint. The
next step is an owner-authorized main push and normal CI. Native optimized
packages and publication remain outstanding; no push, tag, or remote workflow
was initiated during this preparation.


Final candidate corrections preserve Pine condition IDs and positions across
LF, CRLF, lone CR, and mixed line endings. The account-management indicator guide
also matches the Pine guide, with a package check preventing drift. Focused
regressions and validation are recorded in the
[source-verification plan](plans/tradingview-cli-indicator-source-verification.md#final-review-correction-2026-10-05).

## Working limits

One PM/executor; preserve unrelated work, staged changes, and stashes. Use scoped
serial checks and retain valid historical evidence. Heavy pre-push tests remain
disabled. New authentication, provider/Desktop/account operations, production
dependency additions, and remote writes need explicit authority. Public CLI,
JSON, Rust API, and persisted-format changes require agreed concrete examples.
Before a formal release, recheck and update existing dependencies, explain
resolver constraints, and run the affected checks and normal candidate CI.
