# Direction after v0.35.0

Baseline: [released v0.35.0](releases/v0.35.0.md), at d71fd532.
Publication and release workflow success were rechecked on 2026-10-05.
The [closeout](plans/archives/tradingview-cli-chart-analysis-contracts.md)
preserves acceptance evidence and qualification limits. Before this documentation
closeout, the sole post-tag commit was 9cba16e. It repairs test fixtures and has
successful CI. It does not warrant a patch release by itself.

Status: proposal for owner review. The next version and feature scope are not
selected. The [inventory](next-version-work-items.md) owns candidate order.

## Recommended next outcome

Improve confidence in frequently used analysis paths before expanding command
coverage. Start with bounded qualification of existing technical snapshots for
needed higher timeframes. Separately consider making price-source diagnosis
machine-readable through the existing `spec` command. Neither requires a new
data backend or a general schema-coverage program.

1. With explicit live authority and a chosen public symbol, qualify the needed
   technicals intervals. Prefer weekly and monthly for higher-timeframe use;
   include two-hour only when needed. The CLI already implements these choices.
   Use at most one request per approved interval and stop the set on a rate-limit
   or authentication failure. Do not poll, retry automatically, or switch source.
   Results qualify the observed request only and need not produce a release.
2. If the owner selects diagnosis support, agree on concrete before/after
   `spec diagnose quote-data` output. Today semantics is null. The proposed
   annotation would describe the scanner reference request, Desktop dependency,
   network observation, blocked/unavailable outcomes, and absence of chart or
   account mutation. Reuse existing spec facilities and diagnostic fixtures.
   Do not change acquisition, fallback, retry, or the diagnostic result contract.
3. Fix a demonstrated analysis defect ahead of broader annotations if one is
   found. A feature or contract change needs its own agreed examples and focused
   acceptance. Choose the release version only after that scope is settled.

## Alternatives and tradeoffs

| Candidate | User benefit | Evidence and remaining limit |
| --- | --- | --- |
| Existing technicals qualification | Know which requested intervals have actual provider evidence for multi-timeframe analysis | Already implemented; daily success is historical evidence, weekly previously returned provider-internal 429, monthly/two-hour success is unconfirmed. Qualification adds no command and proves no freshness/finality guarantee. |
| Targeted diagnosis semantics | Programs can inspect prerequisites and effects before investigating unavailable quote-data | Current offline spec returns semantics:null; the adapter requests scanner data before Desktop discovery. Annotation clarifies existing behavior but does not make prices more available. |
| Saved-Pine source identity | A future indicator-alert workflow could verify that supplied source matches the saved script | Current execution does not compare sources. Promote only after a real workflow and accessible identity evidence are established. Source identity alone does not solve input, plot-ID, or readback limitations. |
| Chart-read performance work | Could reduce analysis wait time if a bottleneck is reproduced | The completed attribution study did not reproduce the earlier tail. Reopen only under its recorded trigger, then measure the responsible phase before choosing a fix. No speedup is currently demonstrated. |
| Broad spec/schema completion | More commands become self-describing | Coverage alone is insufficient user value. Hotlist, depth, discover, and spec remain candidates until a concrete invocation or interpretation problem needs them. |

The source anchors and tests are in the inventory. Preserve v0.35.0 as the
released baseline if qualification finds no implementation defect and the owner
does not select an additive feature. A release is not required merely to empty
the candidate list.

## Boundaries

The PM works alone. New feature implementation waits for proposal review.
New live targets, authentication, provider/Desktop/account operations, dependency
additions, push, tags, workflow dispatch, and Release edits need their applicable
authority. Any release must refresh existing direct/dev dependencies and the
transitive graph, review changes and resolver constraints, and validate the
candidate before publication. Local Cargo work remains serial with one build
job and one test thread.

The [CDP strategy](notes/cdp-stability-and-autonomous-operation-strategy.md)
retains its triggers for retry/reconnect, renderer readiness, shared sessions,
brokers, daemons, and public timing/recovery metadata. Do not promote indicator
search, bar-cap expansion, additional intraday bars, drawing geometry, or Windows
MSIX/AUMID without a demonstrated need. Collection policy and research admission
remain outside the CLI.
