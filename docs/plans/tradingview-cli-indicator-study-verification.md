# Opt-in verification for indicator alerts

Status: implementation approved and locally accepted on 2026-10-05.
This supersedes the earlier mandatory-chart proposal in this same record.
One executor, no additional agents or sessions. The
[inventory](../next-version-work-items.md) owns priority and observed evidence;
the [roadmap](../next-version-roadmap.md) owns direction.

## Outcome and compatibility

A user can opt into saved compiled-condition verification without adding a
no-input indicator to the chart. For an input-bearing script, verified creation
uses the values of an exact saved-revision chart instance. Multiple matching
instances require an explicit selection. Saved defaults do not substitute for
chart values.

Keep existing no-flag CLI behavior, JSON results, public IndicatorAlertRequest
fields, and alert_create_indicator signature. Preserve the completed
unknown-outcome and native input-ID corrections. Compatibility also preserves
existing limits: legacy preview checks saved source but not compiled conditions
or chart inputs; legacy normal creation still selects input studies by label.
The new verification guarantee applies only to the opt-in operation.

This is not Desktop-free creation. Saved private metadata uses the existing
Desktop session, and normal creation still uses current chart/request metadata
and the existing account path. Stronger preflight does not establish complete
post-create identity, provider acceptance, or notification delivery.

## Evidence and alternatives

The [research record](../next-version-work-items.md#chart-free-metadata-and-compatibility-research)
contains the bounded observations. An authenticated GET for an explicit saved
ID/version returned metaInfo with matching scriptIdPart and pine.version plus
the expected alertcondition ID/title while that script was absent from the
chart. A second existing input-bearing revision returned four user-input
IDs/defaults matching its native declarations. No chart or account state was
changed. The actual chart values in that second case equalled defaults, so this
does not qualify non-default values through real alert creation.

The current translate_light source check returned language-analysis fields but
no compiled metaInfo in one synthetic probe. It is not the metadata
source for this implementation. The saved translate endpoint is an internal service used by a public
community client, not a documented stable TradingView API. Missing or malformed
metadata must fail the opt-in check; it must never silently use legacy behavior.

| Design | Capability and cost | Decision |
| --- | --- | --- |
| Require a chart study for every existing call | Strong chart-instance checks, but breaks no-input usage and preview; no longer necessary for compiled conditions | Superseded |
| Add optional verification and study selection | Existing workflows retain behavior; verified no-input calls need no study; input-bearing calls use exact chart values | Recommended |
| Add a second CLI creation command | Same possible capability, but duplicates command discovery/help for the same action | No demonstrated CLI benefit over opt-in arguments |
| Automatically use saved defaults without a chart | Could eventually support input-bearing chart-free preparation, but changes which values drive an alert | Separate explicit input-source contract and qualification required; not this slice |

A separate Rust entry point is justified by source compatibility: adding even
an optional public struct field breaks existing struct-literal callers, and an
extra function parameter breaks calls. Keep the existing public surface and
share private preparation/creation helpers rather than duplicate adapters.

## CLI behavior

Add --verify and --study-id. An explicit study-id implies verification; it is
case-sensitive, trimmed, and nonblank. No flag means the current behavior.
The owner approved these names and contracts; they are implemented locally.

```sh
# Existing invocation: behavior and output remain compatible.
tv --target-id <target_id> alert create-indicator \
  --script "Example indicator" --file <source.pine> \
  --condition-title "Example condition" --dry-run

# Verify saved compiled condition; no study needed when there are no user inputs.
tv --target-id <target_id> alert create-indicator \
  --script "Example indicator" --file <source.pine> \
  --condition-title "Example condition" --verify --dry-run

# Select the exact instance supplying current input values; implies --verify.
tv --target-id <target_id> alert create-indicator \
  --script "Example indicator" --file <source.pine> \
  --condition-title "Example condition" --study-id <entity_id> --dry-run
```

Existing state output lists study IDs, including studies without a numeric
value. Existing indicator get can help inspect an instance, but its long-string
filtering makes it unsuitable as a complete alert-input payload. Read native
values directly and retain the implemented native-ID validation.

| Case | Legacy invocation | Opt-in verification |
| --- | --- | --- |
| No user inputs and no chart study | Preview and base-only normal path can proceed | Read saved compiled metadata and proceed without a study |
| Inputs and one exact native saved revision | Preview skips input checks; normal path uses labels | Use that instance's current values by declared ID |
| Multiple exact instances | Normal path can use first matching label | Reject unless study-id selects one |
| Same label but different saved ID/version | Label may supply values | Reject explicit mismatch; automatic mode never substitutes it |
| Inputs but no exact instance | Normal mode requires a matching label; preview can pass | Reject; no fallback to saved defaults |
| Explicit study-id for a no-input script | Flag does not exist | Require that entity to match; explicit selection is not ignored |
| Compiled condition or required metadata unavailable | No compiled check | Reject before alert listing/creation; no fallback |

Both verified preview and normal mode use the same saved-source, compiled
condition, and input preparation. Preview stops before all alert-list/create
requests. Normal mode continues with the prepared input snapshot and current
creation behavior. Resolve symbol/resolution through existing request/chart
rules; verification does not add a new source or a different default.

## Public results and errors

Leave legacy JSON unchanged. Only opt-in success adds this verification summary:

```json
{
  "verification": {
    "condition_source": "saved_compilation",
    "input_source": "none",
    "input_count": 0,
    "study": null
  }
}
```

For selected chart values, input_source is active_chart_study and study is
{"entity_id":"example-entity","selection":"explicit"}; automatic selection
uses unique. An explicitly selected no-input study still returns its selection,
with input_source none and input_count zero. Preview additionally returns the
resolved symbol and resolution in its request. Keep the parser's confidence
meaning unchanged; compilation agreement is separate evidence. Do not return
saved IDs, source, compiled payloads, or system input values.

The existing success normalizer must retain this summary only after validation.
Invalid Runtime output must not become a verified success. Preflight errors
have created:false; after a normal create may have dispatched, retain current
created:null and creation_outcome:unknown semantics. Verified preview cannot
report an unknown account write because it cannot dispatch one.

| Failure | Kind | phase / reason |
| --- | --- | --- |
| Blank study-id | validation | Local argument rejection before connection |
| Saved compiled identity disagrees | validation | compiled_condition_verification / saved_revision_mismatch |
| Compiled condition disagrees with selected local ID/title | validation | compiled_condition_verification / condition_mismatch |
| Compilation response or identity/condition/input definitions unavailable | internal_api_unavailable | compiled_condition_verification / compiled_metadata_unavailable |
| Explicit entity missing | validation | study_identity_verification / study_not_found |
| Explicit entity revision mismatch | validation | study_identity_verification / saved_revision_mismatch |
| Required exact instance absent | validation | study_identity_verification / no_matching_study |
| Multiple exact instances without selection | validation | study_identity_verification / ambiguous_study; include match_count |
| Required native identity unavailable | internal_api_unavailable | study_identity_verification / study_metadata_unavailable |

Keep existing saved-source and native-input error semantics. Missing metadata
is not a zero-match result. Automatic selection must stop if unreadable relevant
metadata could conceal another match; explicit selection need not inspect other
entities. Compare exact plot ID/type and the local title when known. Do not
remap a local candidate to a different compiled condition. Require native saved
ID/version representations to agree. Do not use top-level meta.version as the
saved revision or use last instead of the verified version.

## Ownership and implementation

IndicatorAlertRequest and alert_create_indicator remain unchanged. The additive
public entry point in the existing alert operation module is:

```text
IndicatorStudySelection<'a> = Automatic | Entity(&'a str)
alert_create_indicator_verified(runtime, request, selection) -> Result<Value, AppError>
```

Automatic means no entity for zero compiled user inputs; otherwise require one
exact native revision. Entity requires that exact instance even with no inputs.
The new function is a distinct stronger contract, not an alias changing legacy
semantics. CLI flags select the appropriate function. Update Rust API docs and
exports alongside implementation; retain a compile fixture using the unchanged
request literal and function signature.

Reuse saved_script.rs for catalog/revision/source equality. A private compiled
metadata helper fetches the exact saved revision and validates its identity,
condition, and user-input declarations. A private study helper resolves an
instance and snapshots its actual values. Share native input validation and the
existing create/readback path across modes; do not introduce a second transport,
persistent selection file, public intermediate representation, or generic
verification framework. Compile metadata and native declared input IDs must
agree when chart values are used. No automatic study insertion or source edit.

The owner approved this compatibility-preserving implementation. No new
dependency is included. Help/spec, CLI/Rust docs, changelog, both distributed
indicator guides, output normalization, and affected tests are updated together.

## Acceptance and authority

Verify legacy behavior and the new path separately.
Fixtures must prove no extra metadata request, changed JSON, or study requirement
for no-flag calls. Exercise saved compilation without a study, input-bearing
unique/explicit selection, duplicates, wrong revisions, missing metadata,
condition mismatch, malformed Runtime output, and preserved native input IDs.
Every verified preflight failure and preview must issue zero alert-list/create
calls. Normal verified success must use the selected actual values once without
fallback or retry. Keep the approved outcome/input regressions green.

Run affected Rust and CLI contracts, production-JavaScript fixtures, scoped
Clippy, formatting, public hygiene, and standalone resources with one Cargo job
and one test thread. These checks do not establish real alert acceptance.

Read-only live acceptance uses one verified no-input preview of the previously
qualified saved test revision, without adding a study. Source may be read into
a temporary local file for that invocation and removed afterward. Compare chart
entity IDs, symbol, and resolution before and after the preview. Input-bearing
non-default and duplicate-instance qualification needs its own concrete targets
and bounds.
No new live mutation run is authorized in this implementation. Full normal
creation/readback, source/input changes, publication, and extra agents remain
outside scope. The earlier proposed compile/three-preview/cleanup run is
superseded; it was never approved or performed.

## Progress

- Completed the earlier bounded native saved-study trial; retained its evidence.
- Demonstrated saved compiled metadata retrieval without chart insertion and
  input definitions/defaults for one existing input-bearing revision.
- Replaced mandatory admission changes with this compatibility-preserving
  opt-in proposal. The owner approved the CLI/Rust/JSON examples and implementation.

### Implementation checkpoint

- Ground and sketch: traced CLI dispatch, unchanged public request, saved-source
  guard, generated account expression, and success/error normalization. Keep
  legacy preview's early return and legacy network request count.
- Blocking first steps: agree contracts (complete), then share the current native
  input validator and preview shaping before adding verified selection.
- Independent workstreams: CLI wiring, adapter, and executable fixtures depend on
  the same contract; one executor owns all changes sequentially.
- Shared mutable state: one checkout and one Cargo job/test thread; no concurrent
  writers, delegated agents, or new sessions.
- Smallest safe decomposition: one shared account expression with optional
  compiled/study preparation; separate public entry points protect compatibility.
  A second complete creation adapter would duplicate request/readback policy.
- Implement and verify: complete; acceptance and remaining live limits follow.
- Review and commit: reviewed boundaries, compatibility, naming, formatting, and
  the final diff; commit this coherent implementation with its guidance/evidence.
- Delegation, parallel design agents, and PR publication: skipped under the
  repository's explicit authority boundaries. The existing plan is the work record.

The compatibility boundary is the no-flag CLI/JSON and named operation API.
The public clap parser variant also reflects the new flags; direct constructors
need verify:false and study_id:None, as documented in the Rust API guide. The
repository has no direct constructor beyond dispatch destructuring. Preserving
that parser representation too would require a separate parsing model; this
slice retains the normal clap representation and the unchanged operation request.

Implementation review kept pure verification-result shaping and the selection
policy enum in tradingview-model, following the existing alert normalizer
boundary. The operation re-exports the enum at the agreed CLI Rust path. Browser
metadata reads and native selection remain private CLI JavaScript. Both modes
share the existing creation/readback expression and one native input validator.
Saved compilation is awaited before reading chart context and input values, so
those chart-derived values are captured without an intervening network wait.

The result normalizer uses the model crate's existing serde_json validation
pattern, including closed verification fields and consistent selection/counts.
It does not introduce a new dependency or duplicate the normal create normalizer.

### Acceptance, 2026-10-05

- The CLI alert module passed 39 Rust tests. Its five ignored JavaScript tests
  ran through the pinned Node 24.21.0 account gate, which passed all six tests
  including the existing watchlist coverage. The new production-generated
  verification matrix passed 96 preview/normal scenarios through the real Rust
  operation and result normalizer.
- Model alert-related tests passed 14 cases. CLI indicator-alert execution/help
  passed six tests, the offline spec test passed, all three spec examples parsed,
  and the five offline schema/validation integration tests passed.
- Scoped model-library and CLI library/binary/affected integration-target Clippy
  passed with warnings denied. Formatting, diff checks, public-hygiene checks,
  14 packaging self-tests, both standalone skill references, shared-guide equality,
  and 51 changed-document local link targets passed. Manifests and lockfile did
  not change.
- A locally built CLI ran --verify --dry-run for the previously qualified saved
  no-input revision while its study was absent. The preview reported
  saved_compilation, input_source none, input_count zero, and study null; the
  local/native condition candidate agreed. The original 27 study IDs, chart
  symbol metadata, and resolution were unchanged.
- The first live preview succeeded, but the inspection helper compared its
  resolved symbol with symbolExt instead of the production main-series symbol.
  A read-only inspection found different exchange-qualified codes in those two
  native fields. The implementation already followed the unchanged main-series
  precedence. One bounded read-only repeat with that existing rule confirmed
  symbol/resolution and all other preview checks. No CLI symbol rule was changed
  to make the check pass. Both temporary source directories were removed.

The malformed-result and chart-context timing regressions failed before their
fixes and passed afterward. No input edit, source edit/save, Pine Editor compile,
study insertion, normal alert creation, notification, or live alert-list call
was performed. Non-default inputs and duplicate instances are fixture-qualified,
not live-qualified. Full workspace/platform suites, CI, optimized archives, and
complete post-create identity remain separate acceptance work.
