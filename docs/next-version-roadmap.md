# Direction after v0.35.0

Baseline: [released v0.35.0](releases/v0.35.0.md), at d71fd532.
Publication and release workflow success were rechecked on 2026-10-05.
The [closeout](plans/archives/tradingview-cli-chart-analysis-contracts.md)
preserves acceptance evidence and qualification limits. Before this documentation
closeout, the sole post-tag commit was 9cba16e. It repairs test fixtures and has
successful CI. It does not warrant a patch release by itself.

Status: the owner approved bounded weekly/monthly technicals qualification and
the diagnostic spec examples on 2026-10-05. Those reads succeeded; the diagnostic
spec is implemented and locally validated, but unreleased. The next release
version is not selected.
The subsequent saved-Pine source-verification contract is approved and locally
implemented. Its live qualification remains pending an available Desktop CDP
session. The [inventory](next-version-work-items.md) owns scope and acceptance.

## Approved outcome and follow-up sequence

Improve confidence in frequently used analysis paths before expanding command
coverage. Bounded weekly/monthly qualification now has provider evidence. The
approved diagnostic spec makes the existing price-source troubleshooting path
machine-readable. Neither adds a data backend or a general schema-coverage
program.

1. The approved weekly/monthly requests succeeded for one public symbol. Any
   further live run needs its own target and authority. Qualify only needed
   technicals intervals. Prefer weekly and monthly for higher-timeframe use;
   include two-hour only when needed. The CLI already implements these choices.
   Use at most one request per approved interval and stop the set on a rate-limit
   or authentication failure. Do not poll, retry automatically, or switch source.
   Results qualify the observed request only and need not produce a release.
2. The approved `spec diagnose quote-data` examples are implemented and locally
   validated. Semantics now describes the scanner reference request, Desktop
   dependency, observation window, and blocked/unavailable outcomes. Acquisition,
   fallback, retry, and the diagnostic result contract are unchanged. Upstream CI
   and release packaging remain future release gates.
3. Fix a demonstrated analysis defect ahead of broader annotations if one is
   found. A feature or contract change needs its own agreed examples and focused
   acceptance. Choose the release version only after that scope is settled.

## Alternatives and tradeoffs

| Candidate | User benefit | Evidence and remaining limit |
| --- | --- | --- |
| Existing technicals qualification | Know which requested intervals have actual provider evidence for multi-timeframe analysis | Daily success is historical evidence; weekly/monthly succeeded in the approved bounded run. Two-hour remains unverified, and the earlier 429 cause is unresolved. This adds no command or freshness/finality guarantee. |
| Targeted diagnosis semantics | Programs can inspect prerequisites and effects before investigating unavailable quote-data | Implemented locally using the approved contract. The adapter requests scanner data before Desktop discovery. Annotation clarifies that behavior but does not make prices more available. |
| Saved-Pine source identity | A future indicator-alert workflow could verify that supplied source matches the saved script | [Implementation and acceptance](plans/tradingview-cli-indicator-source-verification.md) cover source mismatch rejection in both modes. Local fixtures passed; live qualification is pending CDP availability. Provider compatibility is UNCONFIRMED. Inputs, compiled plot IDs, and readback remain separate limits. |
| Chart-read performance work | Could reduce analysis wait time if a bottleneck is reproduced | The completed attribution study did not reproduce the earlier tail. Reopen only under its recorded trigger, then measure the responsible phase before choosing a fix. No speedup is currently demonstrated. |
| Broad spec/schema completion | More commands become self-describing | Coverage alone is insufficient user value. Hotlist, depth, discover, and spec remain candidates until a concrete invocation or interpretation problem needs them. |

The source anchors and tests are in the inventory. Preserve v0.35.0 as the
released baseline until a release is explicitly selected. This bounded run found
no provider failure requiring a fix. The diagnostic annotation is an unreleased
addition; do not expand its scope merely to fill a version.

## Boundaries

The PM works alone. Work beyond the approved diagnostic and source-verification
slices needs scope review.
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
