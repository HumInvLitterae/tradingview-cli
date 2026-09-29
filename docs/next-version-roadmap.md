# Post-v0.33.0 roadmap

Status: direction and three initial contracts approved on 2026-09-29; implementation in progress.
Baseline: [released v0.33.0](releases/v0.33.0.md). The
[inventory](next-version-work-items.md) orders work; the
[analysis-reliability plan](plans/tradingview-cli-analysis-reliability.md)
owns proposals and acceptance. Version selection follows contract review;
keep the workspace at 0.33.0 during development.

## Outcome

Make chart-analysis results and failure reports easier to interpret correctly.
Prioritize MCP failure diagnostics, equity-series identity and truthful effects
on follow-up commands. Preserve explicit providers, existing command routing,
unknown values and downstream ownership of analysis/collection policy.

Each implementation slice includes its consumers, focused tests, docs and
standalone skill references. No production dependency is currently proposed.
macOS, Windows and Linux remain supported; use CI for broad platform coverage
and serial, scoped local checks to limit host load.

## Sequence

1. Add bounded, public-safe clues to provider-declared MCP failures without
   changing existing error codes or interpreting textual clues as HTTP status.
2. Identify the data source and confirmed meaning of `data equity` results;
   preserve zero drawdown and expose unknown semantics rather than invent them.
3. Correct chart-mutation hints and separately describe file writes. Retain
   advisory-only behavior and existing Rust/JSON consumers in the design.
4. Review legacy account operations against official MCP capabilities. Promote
   a fix or deprecation only for a demonstrated workflow, with concrete contracts.
5. Design output-schema discovery and offline request validation for frequently
   used commands after the first three slices settle. Do not require all 169
   commands to reach complete semantic/schema coverage before delivering value.

## Deferred work

Official technical snapshots retain their [existing proposal](plans/archives/tradingview-cli-mcp-operational-improvements.md#official-technical-snapshot).
On 2026-09-29 a directly connected Codex MCP price read still returned a provider
error mentioning the Screener endpoint and 429. Resume technical qualification
only with usable response evidence or a provider explanation and owner resumption;
no polling or provider workaround is implied.

The [CDP strategy](notes/cdp-stability-and-autonomous-operation-strategy.md)
retains its evidence gates for retry/reconnect, renderer readiness and connection
sharing. Do not add a broker/daemon or generalized recovery metadata without
measured need. Bar-cap/timeframe expansion, drawing geometry and Windows MSIX
activation still require concrete consumers. Source-aware caching, sealed
collection policy and admission to research/backtests remain downstream work.
