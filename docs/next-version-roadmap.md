# v0.33.0 roadmap

Status: offline specification work added before release qualification, 2026-09-25.
Alert history, login guidance, command-local connection reuse and the explicit read timeout are implemented.
The owner deferred technical snapshots on 2026-09-25; they no longer block
this release. Baseline: [released v0.32.0](plans/archives/tradingview-cli-official-mcp-client.md#publication-closeout-2026-09-22).
The [inventory](next-version-work-items.md) owns order; the
[operational-improvements plan](plans/tradingview-cli-mcp-operational-improvements.md)
owns contracts, decisions and acceptance.

## Outcome and selected scope

Make the independent MCP commands easier to operate after installation or
upgrade, and deliver alert history with qualified operational improvements:

- Read alert firing history without changing account state.
- Improve explicit login guidance and credential reuse across supported OSes,
  preserving noninteractive ordinary commands and structured output.
- Measure repeated initialization within one command; reuse a connection for
  mutation/readback only if the measured benefit warrants the change.
- Allow an explicit `--timeout` for provider reads while retaining the 30-second
  default and unchanged mutation deadlines. This addition was separately approved.
- Diagnose time-sensitive MCP fixtures and fix demonstrated synchronization
  problems without relaxing production deadlines or hiding failed assertions.

Deferred technical snapshots would complement arbitrary `mcp symbol`/`symbols`
columns and selected-chart/Pine observations. Compare overlapping data without promising
identical sources, values or timing. Official aggregate ratings remain provider
observations, not CLI trading recommendations or backtest admission.

## Agent usability addition (accepted 2026-09-25)

Maintain portable runtime skills in root `skills/`, with repository agent links
and real-file distribution. Support standard skill-manager discovery without
including contributor workflows. Add an offline `tv spec` index and selected
command details from the running binary, reusing clap definitions instead of
maintaining duplicate argument catalogs. Mark incomplete semantic coverage
explicitly and route dynamic values to existing discovery commands.

Output schemas and offline request validation are subsequent stages after their
contract coverage is established; they are not silently included in the first
spec implementation. No MCP server or provider access is part of specification
queries. The active work record owns staged acceptance.

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

Target v0.33.0 for additive alert history, the read timeout and qualified
operational improvements.
A separately useful fix can ship as v0.32.1, but a patch-first sequence is not
required. Keep the workspace at the released version until feature qualification
is complete. Technical snapshots are explicitly deferred by the owner, not a
release prerequisite. Resume their existing design when usable response evidence
or a concrete provider explanation becomes available and the owner resumes work.

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
