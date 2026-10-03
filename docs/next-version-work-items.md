# v0.35.0 ordered work inventory

Direction approved 2026-10-03. [Roadmap](next-version-roadmap.md) ·
[Contracts and acceptance](plans/tradingview-cli-chart-analysis-contracts.md).
v0.34.0 is published; its completed scope is in the
[archived record](plans/archives/tradingview-cli-analysis-reliability.md).

| Order | Work | State / completion condition |
| --- | --- | --- |
| 1 | Close v0.34.0 | Complete: publication, successful tag workflow and four native asset entries verified. Record archived and current entry points updated. No artifact rebuild or download repeated. |
| 2 | OHLCV missing-value contract | Concrete examples approved. Implementation preserves unknown cells/aggregates as null and real zero. Consumers must handle nulls explicitly; focused local checks passed, normal CI deferred by owner. |
| 3 | OHLCV schema and offline validation | Implemented for raw/summary mode through existing facilities, unchanged count normalization and zero-I/O validation. Focused fixture/schema/CLI checks passed; normal CI deferred by owner. |
| 4 | Pine graphics schema/validation | Lines/boxes/labels/tables schema and validation implemented; focused fixture/schema/CLI checks passed, acquisition/output unchanged. CI deferred by owner. |
| 5 | Pre-release dependency refresh | Required by owner on 2026-10-04. All direct/dev dependencies checked against current stable releases; Tokio and nine transitive packages updated; focused CLI/MCP/schema verification passed. The upstream generic-array exact constraint remains documented. Recheck immediately before release and validate any further changes. |
| 6 | Candidate CI | Pending; owner deferred this while implementation continued. Run normal CI on the refreshed dependency graph before release. |
| 7 | Scope/version and release preparation | v0.35.0 preparation approved 2026-10-04. Notes/guidance and all eight workspace/lock versions are prepared. Focused execution evidence is reused; candidate version verification follows this commit. Normal CI and native archive acceptance remain pending. Keep dependency updates before the release-version commit. |

## Bounded follow-ups

- Legacy alert price projection: the owner approved projecting a finite scalar
  into condition.value
  only for the exact simple-price creator shape when value is absent. Implemented
  with focused model/CLI acceptance; CI deferred. Existing values and zero remain
  intact; symbol-marker identity stays unresolved. No raw series restoration or
  automatic MCP migration is approved.

- Official technical snapshots: daily live success established; weekly previously
  returned provider-internal 429. Monthly/two-hour success remains unverified.
  Resume only the needed cases; stop on 429, with no polling or source fallback.
- Nonempty native alert history, graphical Linux OAuth and Windows skill-manager
  execution remain unverified. Existing fixtures/platform evidence are scoped.
- Saved-Pine source identity needs a concrete indicator-alert workflow before
  promotion. MCP simple-price alerts do not replace Pine alertconditions.
- Remaining spec annotations (diagnose quote-data, scanner hotlist, data depth,
  discover and spec) advance only for demonstrated workflow benefit.
- The older v0.33.0 public Release example correction remains an owner follow-up;
  its repository example is already corrected. No remote edit is included here.

## Working limits

One PM/executor; preserve unrelated work and stashes. Use scoped serial checks,
no automatic heavy pre-push tests, no new authentication, Desktop/account changes,
new dependency additions or push/publication. Updating existing
direct/dev and transitive dependencies before release is explicitly authorized.
Public-contract changes require agreed concrete examples; the OHLCV plan identifies that decision.
