# v0.32.0 candidate ordered work inventory

Direction: [roadmap](next-version-roadmap.md). Detailed decisions and acceptance:
[the existing MCP work record](plans/tradingview-cli-official-mcp-client.md#release-checklist-and-documentation-plan-2026-09-22).

## Current state (2026-09-22)

v0.31.4 is released. The separate MCP command group and all agreed expansion
slices are implemented. Economic/dividend native acceptance is incomplete;
other slices and downstream observation intake have recorded acceptance.
Windows CI at `42b1267` passed and the owner reports a satisfactory basic Windows
machine check. No Windows retest gate is imposed for the already resolved fix;
refresh/long-running behavior is not implied by that brief report.

Remote main is `7650066`, a mise configuration update with successful CI. The
local inspection used `42b1267`; reconcile that remote change before preparing
the candidate. Workspace version remains 0.31.4 until release preparation.

## Remaining work in order

| Order | Work / owner | Completion condition |
| --- | --- | --- |
| 1 | Linux adapter and isolated service verification — upstream | Implement using existing secret-service dependency; verify synthetic-secret persistence, restart, replacement, deletion and noninteractive failures in isolated Linux. Real desktop OAuth remains separately unverified. |
| 2 | MCP documentation and skills — upstream | Complete the staged plan in the existing record: English/Japanese onboarding, command/source navigation, MCP-first capability routing, independently attachable skills, account-management skill and isolated-skill/archive checks. Can proceed while provider verification is unavailable. |
| 3 | Economic/dividend acceptance — upstream | Normal-deadline filtered codes, selected series, economic calendar and both dividend modes succeed through the public service. Respect renewed provider limits; successful dividend normalization is still missing. |
| 4 | Candidate scope — upstream PM / owner | Confirm v0.32.0 contents including the requested Linux adapter, separating implemented support from container/CI and real-desktop evidence. Persistent provider failure requires an explicit scope/qualification choice. |
| 5 | Candidate qualification — upstream | Integrate remote changes; inspect dependency/contract drift; applicable Rust and pinned JS gates plus native CI and package checks. Reuse unchanged evidence. |
| 6 | Release preparation — upstream PM | Align 0.32.0 versions/lock metadata, changelog and curated notes; build/stage release binaries for supported targets, verify resources, version and checksums. Separate coherent release commit. |
| 7 | Publication and closeout — owner / upstream PM | Current-turn authorization for remote publication, verified tag/assets/checksums, then archive completed work and synchronize plan entrypoints. |

## Nonblocking follow-up and retained boundaries

Downstream owns analysis/backtest adoption, provider-aware caches, fixed-source
policy changes and Desktop-provider migration. These are not prerequisites for
publishing the independent MCP interface. Unknown data semantics remain unknown.
Windows refresh/long-running testing can extend current evidence without reopening
the accepted correction. Linux implementation is requested; desktop validation
limits must remain explicit.

CDP retry/reconnect, broker/daemon, public recovery metadata, renderer/indicator
search, larger bar caps, legacy intraday ranges, drawing geometry and MSIX
activation retain their existing evidence triggers in the roadmap. Do not promote
new features to fill a provider wait. Existing same-scope verification/commit
authority persists; no extra agent, downstream edit or publication is authorized.
