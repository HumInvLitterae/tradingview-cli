# v0.32.0 candidate ordered work inventory

Direction: [roadmap](next-version-roadmap.md). Detailed decisions and acceptance:
[the existing MCP work record](plans/tradingview-cli-official-mcp-client.md#release-checklist-and-documentation-plan-2026-09-22).

## Current state (2026-09-22)

v0.31.4 is released. The separate MCP command group and all agreed expansion
slices are implemented. Both dividend modes now pass native acceptance;
other slices and downstream observation intake have recorded acceptance.
Windows CI at `42b1267` passed and the owner reports a satisfactory basic Windows
machine check. No Windows retest gate is imposed for the already resolved fix;
refresh/long-running behavior is not implied by that brief report.

The verified remote CI baseline is `ccd780d`, including Linux credential
integration and all OS/JavaScript jobs. The later development-harness correction
passed focused checks and live dividend acceptance. The workspace is prepared
at 0.32.0; v0.31.4 remains the latest verified published release.

## Remaining work in order

| Order | Work / owner | Completion condition |
| --- | --- | --- |
| 1 | Linux adapter and isolated service verification — upstream | Implemented with secret-service and approved Linux-only zbus; isolated service tests cover synthetic-secret persistence, restart, replacement, deletion and noninteractive failures. Linux CI also passed. Real desktop OAuth remains unverified. |
| 2 | MCP documentation and skills — upstream | Implemented: English/Japanese onboarding, MCP-first routing, account-management plus six existing independent skills, isolated-reference/shared-copy/archive checks. Final evidence is in the MCP record. |
| 3 | Economic/dividend acceptance — upstream | Complete: economic reads and both dividend modes normalized successfully within normal deadlines. Dividend-only proof avoids repeating completed economic reads and distinguishes tool kind from reused mode names. |
| 4 | Candidate scope — upstream PM / owner | Selected: all implemented MCP slices, Linux credentials and seven independent skills. Scoped CI/native evidence and unverified graphical Linux OAuth remain distinct. |
| 5 | Candidate qualification — upstream | CI/functional qualification complete; external locked graph unchanged. Verify the local ARM release artifact after the preparation commit; other optimized target builds belong to the tag workflow after publication approval. |
| 6 | Release preparation — upstream PM | 0.32.0 versions/lock metadata, changelog and curated notes prepared together. Build/stage the local candidate once, verify version/resources/checksum and record the result in the handoff without a self-hash tracking commit. |
| 7 | Publication and closeout — owner / upstream PM | Current-turn authorization for remote publication, verified tag/assets/checksums, then archive completed work and synchronize plan entrypoints. |

## Nonblocking follow-up and retained boundaries

Downstream owns analysis/backtest adoption, provider-aware caches, fixed-source
policy changes and Desktop-provider migration. These are not prerequisites for
publishing the independent MCP interface. Unknown data semantics remain unknown.
Windows refresh/long-running testing can extend current evidence without reopening
the accepted correction. Linux is implemented; real-desktop validation limits remain explicit.

CDP retry/reconnect, broker/daemon, public recovery metadata, renderer/indicator
search, larger bar caps, legacy intraday ranges, drawing geometry and MSIX
activation retain their existing evidence triggers in the roadmap. Do not promote
new features to fill a provider wait. Existing same-scope verification/commit
authority persists; no extra agent, downstream edit or publication is authorized.
