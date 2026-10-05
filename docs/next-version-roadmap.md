# Direction after v0.36.0

Status: on 2026-10-05 the owner approved the proposed direction, the unknown
creation-outcome correction, and the study/input identity investigation. The
correction is implemented and committed; the inventory owns acceptance and
investigation results. A bounded metadata read has now exposed an input-position
defect, reproduced using synthetic data. Its proposed correction is the next
scope decision, ahead of a broader identity feature. Broader identity behavior remains conditional. One executor, no
delegated agents. No new live operation or publication is authorized.

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
| 2 | Qualify exact study selection and input identity | Determine whether duplicate study names and incomplete inputs can be rejected or resolved using stable native identity | Inspect existing study metadata first; a bounded read-only Desktop probe needs a named target and authority; no new selector is promised yet |
| 3 | Implement the supported identity and readback contract | For the agreed supported case, use the intended inputs and confirm the newly created alert against the agreed fields | Agree selector/error examples and supported limits; require an authorized bounded create/readback/cleanup run before claiming live creation acceptance |
| 4 | Release the completed slice | Ship a coherent improvement with matching CLI guidance, fixtures, CI, and native packages | Refresh existing dependencies, preserve resource limits, and obtain publication authority |

Order 1 can ship independently if the later identity investigation is inconclusive.
A defect-only release may be v0.36.1. Reserve v0.37.0 for approved additive CLI or
JSON behavior and the supported identity workflow. Select the version after
contract review; do not enlarge scope merely to fill a minor release.

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

### Identity investigation and conditional implementation

Current creation selects the first chart study matching a name/title, maps input
order to in_0 and subsequent keys, and derives condition candidates from local
source. Readback checks a new ID, condition ID, message, and symbol when present.
It does not establish every input, saved version, or resolution. Saved-source
agreement alone does not close those gaps.

First determine what stable saved-script identity, study entity identity, input
metadata, compiled condition information, and created-alert identity the existing
native APIs actually expose. Reuse adequate native facilities. If a field cannot
be established, record it as UNCONFIRMED rather than replacing it with a name
match or invented default.

Propose the smallest supported workflow from that evidence. Include duplicate
names, changed inputs, missing metadata, concurrent unrelated alert creation,
and an accepted create followed by a failed readback in its acceptance cases.
A possible explicit study selector remains a design candidate, not an approved
public flag. If complete identity is infeasible, report the precise supported
subset and its omitted behavior before seeking implementation approval.

## Current investigation decision

The bounded read-only observation found native script/version and condition
metadata, but no chart study matching the saved catalog. Full saved-study binding
is therefore still unqualified. More immediately, the production input mapping
turned synthetic internal text into in_0 instead of retaining the intended value
20. The [inventory](next-version-work-items.md#proposed-next-correction-not-yet-approved)
records the reproduction and concrete before/after proposal.

Prioritize that input-ID correction before a new selector or full identity
workflow. It can be implemented and fixture-tested without a live account write.
Admission of incomplete metadata and the public input count change need approval
of the recorded examples. No broader identity behavior is implied.

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

The approval covers the first correction and source investigation. It does not
authorize broader identity contracts, dependency additions, live account changes,
or publication. The inventory owns priority and this bounded slice's work record.
A broader identity implementation needs its own agreed contract and ExecPlan. Keep private consumer details out
of public records. Run local Cargo checks serially with one build job and one
test thread, reuse unchanged valid evidence, and avoid redundant broad builds.
Before release, review existing dependency updates and resolver constraints,
run affected checks and normal CI, and qualify native packages. Fixture success,
read-only qualification, normal creation, and notification delivery remain
separate evidence. The proposed scope does not promise notification delivery.
