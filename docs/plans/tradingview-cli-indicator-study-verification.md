# Verify the chart study before indicator-alert operations

Status: proposed contract, 2026-10-05. Implementation and the new bounded live
acceptance below await owner approval. One executor, no additional agents or
sessions. The [inventory](../next-version-work-items.md) owns priority and the
[roadmap](../next-version-roadmap.md) owns direction.

## Outcome and limit

A user can choose which instance of a saved Pine indicator supplies the alert's
inputs. Both preview and creation must identify its saved revision, confirm the
selected compiled condition, and validate its native input IDs before proceeding.
Two instances of the same script must not silently select the first by label.

This changes admission: even a script without inputs must already have a matching
study on the selected chart. The CLI does not add, compile, update, or repair
that study. A dry-run becomes a chart-study preflight as well as the existing
saved-source check. It still does not create an alert or establish provider
acceptance, notification delivery, or complete post-create readback.

## Evidence and alternatives

The [bounded trial](../next-version-work-items.md#saved-study-observation-and-acceptance)
matched saved ID/version to meta.scriptIdPart, meta.pine.version, and the native
pineId/pineVersion inputs. It also matched the compiled alertcondition ID/title
for one no-input script. The entity was removed and the original chart preserved.
Input-bearing and duplicate-instance cases have not been qualified live.

Existing [state output](../../crates/cli/src/ops/chart.rs) returns every listed
study's id/name. Unlike values output, it does not require a numeric study value.
The existing indicator get command reads one entity's inputs, subject to its
published long-string filtering. Use these facilities for selection; do not add
a catalog command or use a lossy public summary to build an alert payload.

| Design | Capability | Decision |
| --- | --- | --- |
| Unique saved-ID/version match only, no selector | Works when exactly one matching instance exists; duplicates require changing the chart | Smaller interface, but cannot choose between ordinary same-script instances with different inputs |
| Optional entity selector plus unique automatic selection | Existing single-instance calls keep their argument syntax; duplicates can be resolved explicitly | Recommended; reuses state/indicator get and requires one optional CLI/Rust field |

Implement the second design. Do not introduce a saved selection file, preparation
token, second creation API, persistent journal, or automatic study insertion.
An entity ID selects a chart instance; it never bypasses saved-source, saved
revision, condition, or input checks.

## Proposed CLI and public results

Use the same explicit target throughout:

```sh
tv --target-id <target_id> state
tv --target-id <target_id> indicator get <entity_id>
tv --target-id <target_id> alert create-indicator \
  --script "Example indicator" --file <source.pine> \
  --condition-title "Example condition" --study-id <entity_id> --dry-run
```

`--study-id` is optional, case-sensitive, trimmed, and nonblank when supplied.
An omitted value requires exactly one native saved-ID/version match. With an
explicit ID, inspect that entity and require the same native identity checks.
Do not search another entity or fall back to a name when it is missing or wrong.

| Case | Before | Proposed behavior in both modes |
| --- | --- | --- |
| One matching saved revision on chart | Normal mode uses first matching label; preview ignores studies | Select it by native saved ID/version and verify condition/inputs |
| Two matching instances | Normal mode uses first label match | Reject without study-id; select the specified instance when its identity matches |
| Same label, different saved ID or version | Label can supply the inputs | Reject the selected mismatch; do not use it as a substitute |
| No chart study for a no-input source | Preview and base-only normal creation can proceed | Reject; require the user to put the saved indicator on the chart first |
| Compiled condition conflicts with the local candidate | Local candidate is used without native confirmation | Reject before alert listing or creation; no automatic ID remapping |
| Missing identity/condition metadata | Name or local parsing may suffice | Report unavailable evidence, not a verified selection |

Proposed additive success fragment for preview and normal creation:

```json
{"study":{"entity_id":"example-entity","selection":"explicit"}}
```

selection is explicit or unique. The returned study has passed preflight; this
is not a claim that every field of a newly created alert was verified. Keep
existing success fields. Preview additionally receives resolved symbol and
resolution plus the existing normal-mode input_metadata summary. It does not
return source text, saved IDs, digests, or raw system input values. The condition
confidence field retains its local-parser meaning; native agreement is the new
admission requirement, not a rewrite of the parser's confidence vocabulary.

Native preflight errors have created:false and no creation_outcome:unknown.
Existing saved-source errors remain unchanged. Use these proposed error details:

| Failure | Kind | phase / reason |
| --- | --- | --- |
| Blank study-id | validation | Local argument rejection before connection |
| Explicit entity missing | validation | study_identity_verification / study_not_found |
| Explicit entity has a different saved ID/version | validation | study_identity_verification / saved_revision_mismatch |
| No matching native revision | validation | study_identity_verification / no_matching_study |
| Multiple matching revisions without explicit ID | validation | study_identity_verification / ambiguous_study; include match_count |
| Chart or potentially matching identity cannot be read | internal_api_unavailable | study_identity_verification / study_metadata_unavailable |
| Selected compiled condition disagrees | validation | compiled_condition_verification / condition_mismatch |
| Compiled condition evidence cannot be read | internal_api_unavailable | compiled_condition_verification / condition_metadata_unavailable |

Input failures retain study_input_metadata_unavailable and the already approved
input-ID semantics. Missing metadata is not a zero-match result. Automatic
selection must stop when unreadable metadata could conceal another match;
explicit selection need not inspect unrelated entities. Compare the selected
plot ID/type and, when locally known, title. For an explicit condition ID with
no literal local title, require that exact native alertcondition ID without
claiming title agreement. Require native ID/version representations to agree;
never use top-level meta.version as the saved revision.

After a normal creation evaluation may have dispatched, retain the implemented
unknown-outcome semantics. A dry-run evaluation can never dispatch an alert;
its evaluation failure must not imply an unknown account write.

## Ownership and implementation sketch

The [request and adapter](../../crates/cli/src/ops/alert/indicator.rs) own the
operation. Add study_id: Option<&str> to public IndicatorAlertRequest, threaded
from AlertCommand::CreateIndicator through dispatch. Existing Rust struct-literal
callers must add study_id: None, or Some(entity_id). This is a Rust source
compatibility change even though the CLI flag is optional. Update repository
callers and the Rust API guide together; do not conceal it behind a second API.
The existing public operation signature otherwise remains unchanged.

Keep saved_script.rs as the exact catalog/revision/source guard. A private
indicator/study.rs may own the native selection, compiled-condition, and input
helper as one responsibility. Move the current label-first input selection out
rather than retaining two implementations. Do not export intermediate study
metadata or change generic model/CDP ownership.

The usage-driven native helper has this conceptual shape:

```text
resolveStudy(chart, verifiedSavedRevision, optionalEntityId, localCandidate)
    -> {entityId, selection, verifiedInputs, inputCount}
    or a bounded preflight error
```

Embed that helper in the existing operation evaluation so preview and execution
use the same preparation. Preview returns before any alert list/create request;
normal mode continues with the verified input snapshot and current creation
flow. Copy the selected input values into the request snapshot before awaiting
account reads. Do not resolve the study again by name. Preserve the existing
normal-success normalizer and extend its allowlisted output for study; assemble
preview from the native summary without a second selection. Missing/malformed
Runtime results cannot become successful previews.

No CLI flag, Rust field, or code stub is added until the contract is approved.
Implementation must update help/spec/examples, both distributed indicator guides,
CLI/Rust docs, changelog, affected normalizer fixtures, and request constructors.

## Acceptance and authority

After approval, implement and commit coherent verified units. Keep one Cargo job
and one test thread. Reuse the account-JavaScript gate and production-generated
expressions. Cover unique and explicit selection, duplicate instances with
separate inputs, ID/version conflicts, missing metadata, condition mismatch,
missing/duplicate inputs, source mismatch, and invalid Runtime results. Every
preflight rejection and every preview must issue zero alert-list/create calls.
Normal success must use the selected instance's values without retry/fallback.
Keep the completed unknown-outcome and input-ID regressions green.

Run affected Rust/model and CLI argument/spec tests, pinned JavaScript contracts,
scoped Clippy, formatting, hygiene, and standalone resource checks. No dependency
addition or broad local rebuild is planned. Local fixture acceptance is not live
normal-creation acceptance.

Proposed live acceptance, requiring approval together with implementation:
reuse the previously selected saved no-input test script/revision and test chart.
Check for unsaved edits, open and normally compile once, and identify exactly one
new matching entity. Run at most three dry-runs with the new implementation:
unique automatic selection, explicit selection of that entity, and an explicit
pre-existing nonmatching entity to confirm rejection. Do not edit/save source,
change inputs, add duplicate instances, or create alerts. Remove only the verified
new entity and confirm the original study set, symbol, resolution, and saved
source/revision. Stop without retry on uncertain mutation; never guess cleanup.
The editor may remain on the approved test script as in the prior trial.

This live run covers a no-input instance only. Input-bearing and duplicate
instances use fixtures in this slice and remain unqualified live. Normal alert
creation/readback and notifications remain separate permissions and evidence.
Push, tags, release publication, dependency additions, and extra agents are not
included. If the owner approves only local implementation, preserve that narrower
scope and leave this live acceptance pending.

## Progress

- Grounded current CLI/Rust consumers, native state discovery, source guard,
  input construction, and output normalization.
- Compared the two selection designs and selected the optional-ID proposal.
- Awaiting agreement on the public examples and optional bounded live acceptance.
