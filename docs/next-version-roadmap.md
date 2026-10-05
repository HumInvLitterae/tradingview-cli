# Direction after v0.36.0

Status: the unknown-outcome and native input-ID corrections are implemented and
committed. Opt-in compiled-condition and study-input verification is also
implemented and locally accepted. A verified no-input preview succeeded without
adding a study. Input-bearing previews also passed unique selection, duplicate
rejection, and explicit selection with temporary instances cleaned up. The inventory and implementation plan retain the fixture/live
limits. Complete created-alert identity and live normal creation remain separate
work. One executor works without delegated agents; publication is not implied.

## Released baseline

[v0.36.0](releases/v0.36.0.md) is published at
5c72c081b8e0539454a8be0e001702735656585e. GitHub release metadata and successful
CI and Release runs at that commit were checked on 2026-10-05. Four native
archives and SHA256SUMS are uploaded. This planning pass did not download or
execute them.

The release includes saved-Pine source verification, the Pine Editor opening
repair, malformed bars-frame rejection, and offline quote-data diagnostic
semantics. The [inventory](next-version-work-items.md) preserves their local
acceptance evidence. They are completed work, not candidates for the next release.

## Approved direction

Make indicator-alert results useful for deciding whether to inspect account
state, correct a request, or proceed. A user must not interpret an unconfirmed
creation as proof that nothing was created. After that correction, investigate
whether the selected study, its inputs, and the resulting alert can be tied to
the intended saved revision and condition.

This direction was selected because the existing supported workflow has
specific gaps visible in code and its user guide. Current usage frequency is
UNCONFIRMED. No incorrect or duplicate live alert was observed in this planning
pass. This proposal does not promise complete indicator-alert verification in
one release.

## Proposed order and release boundaries

| Order | Deliverable | User-visible completion | Gate |
| --- | --- | --- | --- |
| 1 | Distinguish creation failure from an unknown outcome | A failed post-create readback cannot be read as proof of non-creation; guidance directs the user to inspect state before repeating a request | Approved error examples; implemented with production JavaScript fault fixtures, see inventory |
| 2 | Qualify exact study selection and input identity | Determine whether duplicate study names and incomplete inputs can be rejected or resolved using stable native identity | Native identity and saved compiled metadata were observed in bounded probes; the input-ID correction is implemented and accepted |
| 3 | Implement the supported identity and readback contract | For the agreed supported case, use the intended inputs and confirm the newly created alert against the agreed fields | Opt-in preflight is approved and implemented; complete post-create identity and live create/readback/cleanup need their own agreed scope |
| 4 | Release the completed slice | Ship a coherent improvement with matching CLI guidance, fixtures, CI, and native packages | Refresh existing dependencies, preserve resource limits, and obtain publication authority |

Order 1 can ship independently if the later identity investigation is inconclusive.
A defect-only subset may use v0.36.1; including the approved additive CLI/JSON
behavior calls for v0.37.0. Select the version with the release scope rather than
enlarging scope to fill a minor release.

### Approved first contract

The v0.36.0 [adapter](../crates/cli/src/ops/alert/indicator.rs) reported
created:false for post_list_unavailable and post_check_failed after an accepted
create request. The [model normalizer](../crates/model/src/alert.rs) carries those
fields into error details. The [user guide](../skills/pine-develop/references/indicator-alerts.md)
already warns that failed readback does not prove non-creation.

The approved examples below are error-detail fragments, not replacement envelopes:

| Case | v0.36.0 behavior | Approved meaning |
| --- | --- | --- |
| Saved-source mismatch before dispatch | Validation error; no creation attempted | Retain existing behavior |
| Creation accepted, subsequent list unavailable | phase:post_list_unavailable, created:false | phase retained; created:null, creation_outcome:unknown |
| Creation accepted, no uniquely confirmed matching result | phase:post_check_failed, created:false | phase retained; created:null, creation_outcome:unknown |
| Confirmed new alert | Success with created:true | Preserve the successful shape |

The owner approved created:null and creation_outcome:unknown for unconfirmed
creation errors. This nullable change is intentional; callers must not treat
null as confirmed false. The implementation also covers POST request/response
failures, including HTTP/provider errors, and lost or malformed Runtime results.
Those outcomes cannot establish absence of a side effect. Existing error kinds,
exit codes, phases, preflight errors, and successful shapes remain unchanged. Do not add automatic retries,
automatic deletion, or an account-persistent journal to solve this problem.
Acceptance must prove that ambiguous results cannot enter the confirmed-success
path and that no second creation request is issued.

### Identity scope and remaining readback

Legacy creation still selects the first chart study matching a name/title, but
the approved correction now preserves and validates native input IDs. Legacy
condition candidates remain local-parser results. The opt-in path verifies saved
compiled conditions and exact native saved-revision inputs, with an explicit
selector for duplicate instances. It never substitutes names or defaults when
required evidence is missing.

Readback still checks a new ID, condition ID, message, and symbol when present.
It does not establish every input, saved version, or resolution. The stronger
preflight does not close those post-create gaps. Any further identity work must
agree the fields to be verified and cover unrelated concurrent creation and an
accepted create followed by failed readback. Missing evidence remains
UNCONFIRMED rather than an invented match.

## Current investigation decision

The initial read-only observation found native script/version and condition
metadata, but no chart study matching the saved catalog. A subsequent authorized
compile/read/cleanup trial established that binding for one saved no-input
indicator. Non-default input and duplicate-study cases remain unqualified live.
Before its correction, the production input mapping turned synthetic internal
text into in_0 instead of retaining the intended value 20. The [inventory](next-version-work-items.md#approved-input-id-correction)
records the reproduction, approved before/after examples, and acceptance.

The owner approved that input-ID correction before a new selector or full
identity workflow. Local implementation preserves native IDs and rejects
incomplete metadata without a live account write. The count now means verified
user inputs. No broader identity behavior is implied.

The approved trial used unchanged saved source, produced exactly one study,
matched both native ID/version representations and the condition ID/title, then
removed only that new entity. The original chart studies, symbol, resolution,
and saved source/revision were preserved. This qualifies the native metadata
path for that limited case, not input-bearing scripts or full alert creation.
Use this evidence when proposing missing/ambiguous identity behavior; do not
promote a new selector or broader admission change without concrete examples.

The [study-verification plan](plans/tradingview-cli-indicator-study-verification.md)
now records the approved opt-in verification implementation, preserving existing
no-flag behavior. Saved compiled metadata removes the need for a chart study
when there are no user inputs. Input-bearing verified calls use an exact native
revision, with study-id resolving multiple instances. A separate Rust function preserves existing
request literals and callers. This supersedes mandatory chart admission. Local
implementation and bounded acceptance passed, including a verified no-input
preview without chart insertion. Post-create readback limits remain.

## Other candidates and tradeoffs

| Candidate | Decision for this draft | Reopen when |
| --- | --- | --- |
| Technicals two-hour qualification and prior 429 diagnosis | Separate bounded qualification, not a release feature | A needed interval or a new failure has an authorized target; no automatic polling or fallback |
| Nonempty native alert history, graphical Linux OAuth, Windows skill-manager execution | Keep explicit platform/provider acceptance gaps | An appropriate account or platform and bounded test are available |
| Chart-read speed or reconnect changes | Defer; the recorded 40-read run did not reach the investigation trigger | An ordinary or authorized read exceeds one second, times out, or fails a method call |
| Hotlist, depth, discover, and broader spec/schema coverage | Defer blanket expansion | A concrete caller cannot discover a required invocation or interpret its result |
| Indicator search, larger bar caps, additional intraday bars, drawing geometry, Windows MSIX/AUMID | No promotion | A demonstrated workflow need justifies a separate proposal |

An analysis-first alternative would focus on a demonstrated data-read defect or
needed interval. It should take priority if such evidence arrives. Current
qualification gaps alone do not establish a product defect or justify another
backend. The [CDP strategy](notes/cdp-stability-and-autonomous-operation-strategy.md)
continues to govern transport proposals.

## Execution limits

The owner approved the outcome and input corrections, bounded metadata research,
and the opt-in verification contract in the linked ExecPlan. Implementation and
local acceptance are complete. Dependency additions, live account writes,
complete post-create identity, and publication remain outside that scope.
Keep private consumer details out of public records. Run local Cargo checks
serially with one build job and one test thread, reuse unchanged valid evidence,
and avoid redundant broad builds. Before release, review existing dependency
updates and resolver constraints, run affected checks and normal CI, and qualify
native packages. Fixture success, read-only qualification, normal creation, and
notification delivery remain separate evidence. This slice does not promise
notification delivery.
