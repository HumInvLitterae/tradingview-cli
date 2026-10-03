# Post-v0.34.0 roadmap

Baseline: [released v0.34.0](releases/v0.34.0.md), at 62d85b2. Publication and
the successful release workflow were verified on 2026-10-03; the
[completed record](plans/archives/tradingview-cli-analysis-reliability.md)
preserves implementation evidence and remaining qualification limits.
The next version number is not selected.

## Outcome and sequence

Make frequently used chart-analysis commands easier for agents and other
programs to invoke and interpret. The owner approved extending offline schema
and invocation support to `ohlcv`, starting with its existing summary output.
The [inventory](next-version-work-items.md) owns order and the
[chart-analysis plan](plans/tradingview-cli-chart-analysis-contracts.md) owns
contracts and acceptance.

1. Establish truthful missing-value behavior for selected-chart OHLCV before
   publishing its output schema. Agree on the concrete contract and affected
   consumer migration; do not silently replace unknown data with zero.
2. Add `tv schema ohlcv` and offline `tv validate -- ohlcv ...`, reusing the
   existing standard schema exporter, clap parser and execution rules. Include
   raw and summary modes, their focused fixtures and standalone skill guidance.
3. Extend Pine graphics offline support to lines/boxes/labels/tables for the
   selected-chart inspection workflow. All four paths are implemented
   with local acceptance. Continue from actual chart-analysis consumers rather
   than all-command coverage. CI remains owner-deferred.
4. Confirm the candidate scope/version, complete normal CI and prepare release
   notes/version/distribution as a separate coherent stage. Reuse unchanged
   acceptance evidence; new live checks require their applicable authority.

The owner also approved exposing the scalar threshold of an exact legacy
simple-price condition when condition.value is absent, without restoring raw
study-series or changing symbol identity. MCP technical snapshots are already
implemented; further interval qualification is a bounded verification task,
not a new feature.

## Working limits and deferred work

The PM works alone. Local Cargo checks use one build job and one test thread;
prefer affected fixtures and normal CI over repeated full builds. No next-version
number, dependency, new authentication, Desktop/account mutation, implicit
retry/source switch or push/publication is authorized by this direction.

The [CDP strategy](notes/cdp-stability-and-autonomous-operation-strategy.md)
retains its measured-need triggers for retry/reconnect, renderer readiness and
connection sharing. Bar-cap/timeframe expansion, drawing geometry and Windows
MSIX activation require concrete consumers. Saved-Pine source identity is a
separate candidate. Source-aware caching, collection policy and
research/backtest admission are outside this CLI's scope.
