# v0.33.0 roadmap

Status: direction accepted on 2026-09-22; public contracts are proposed, not
implemented. Baseline: [released v0.32.0](plans/archives/tradingview-cli-official-mcp-client.md#publication-closeout-2026-09-22).
The [inventory](next-version-work-items.md) owns order; the
[operational-improvements plan](plans/tradingview-cli-mcp-operational-improvements.md)
owns contracts, decisions and acceptance.

## Outcome and selected scope

Make the independent MCP commands easier to operate after installation or
upgrade, and complete two useful read workflows:

- Read alert firing history without changing account state.
- Retrieve an official technical-indicator snapshot for one symbol/timeframe.
  This is planned release scope, not an optional backlog item. The owner
  explicitly requested inclusion alongside alert history.
- Improve explicit login guidance and credential reuse across supported OSes,
  preserving noninteractive ordinary commands and structured output.
- Measure repeated initialization within one command; reuse a connection for
  mutation/readback only if the measured benefit warrants the change.
- Diagnose time-sensitive MCP fixtures and fix demonstrated synchronization
  problems without relaxing production deadlines or hiding failed assertions.

Technical snapshots complement arbitrary `mcp symbol`/`symbols` columns and
selected-chart/Pine observations. Compare overlapping data without promising
identical sources, values or timing. Official aggregate ratings remain provider
observations, not CLI trading recommendations or backtest admission.

## Delivery boundaries

Keep existing commands, defaults and versioned contracts. New reads remain
under `tv mcp`. No implicit provider switch, retry, persistent metadata cache,
background service, broker or cross-process shared session is planned.
Keep arbitrary tool forwarding, webhook editing, active-watchlist activation,
Pine reconstruction and automatic trading outside this version.

No dependency additions are currently proposed. macOS, Windows and Linux stay
in scope; reuse accepted Windows evidence and distinguish Linux service/CI
coverage from unverified graphical OAuth. Signing/notarization is a separate
cost and distribution decision, not an implied fix for OS consent.

## Version and release decision

Target v0.33.0 for the two additive reads and qualified operational improvements.
A separately useful fix can ship as v0.32.1, but a patch-first sequence is not
required. Keep the workspace at the released version until feature qualification
is complete. If either new read cannot be qualified, present a concrete scope
or schedule choice; do not silently demote the technical snapshot.

## Retained evidence triggers

The [CDP strategy](notes/cdp-stability-and-autonomous-operation-strategy.md) still
owns retry/reconnect, renderer lifecycle and shared-service triggers. Reopen
chart-read latency attribution when a current workload reproduces the problem.
Higher bar caps and more legacy intraday date ranges need concrete consumers;
drawing geometry and Windows MSIX activation keep their existing triggers.

Old calendar and alert pause/resume candidates must be assessed against the
already released MCP capabilities, not automatically re-promoted as missing
features. Downstream source-aware caching, collection policy and analytical
admission remain downstream responsibilities.
