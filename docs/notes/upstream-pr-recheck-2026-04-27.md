# Upstream pull request re-check 2026-04-27

This note starts the post-`v0.2.0` re-check of open pull requests on
`tradesdontlie/tradingview-mcp`.

The goal is to find compatibility evidence, bug fixes, and feature ideas that
should influence the Rust CLI. It is not a plan to copy JavaScript changes into
Rust.

## Sources checked

- Repository: https://github.com/tradesdontlie/tradingview-mcp
- Command:
  `gh pr list -R tradesdontlie/tradingview-mcp --state open --limit 100 --json number,title,updatedAt,createdAt,url,author,headRefName,isDraft`
- Follow-up:
  `gh pr view` on the newest PRs that were not covered by the previous
  `docs/notes/upstream-pr-triage-2026-04-25.md` pass.
- Rust comparison points:
  `src/cdp.rs`, `src/transport.rs`, `src/ops/launch.rs`, and
  `src/ops/alert.rs`.

As of this pass, the upstream repository reports 54 open PRs. The previous
durable triage note recorded 47 open PRs, so the newest delta is the main focus
of this starter pass.

Refresh on 2026-04-28 still reports 54 open PRs. No new open PR appeared after
the previous delta pass. The most recently updated item is now `#90`, which is
already classified in `docs/notes/upstream-pr-triage-2026-04-25.md` as addressed
by Rust strategy-data compatibility work.

## New PR delta

| PR | Category | Rust disposition |
| --- | --- | --- |
| [#114](https://github.com/tradesdontlie/tradingview-mcp/pull/114) `Document Microsoft Store TradingView Desktop incompatibility` | `docs/compatibility` | Windows live-verification backlog. It says Microsoft Store/MSIX TradingView Desktop may not expose the requested remote debugging port because of packaged-app sandboxing. Rust now records the boundary in `docs/notes/windows-store-msix-launch-boundary.md`: standalone Desktop is the recommended launch path until Store/MSIX CDP startup is proven on Windows. |
| [#113](https://github.com/tradesdontlie/tradingview-mcp/pull/113) `Add advanced trading bot that reads live TradingView chart` | `workflow/helper` | Keep outside core CLI. It may inspire downstream examples or skills, but a trading bot, confluence engine, paper-position manager, and drawing workflow should not be imported into the Rust binary by default. |
| [#112](https://github.com/tradesdontlie/tradingview-mcp/pull/112) `feat(alerts): add alert_create_indicator for Pine alertcondition signals` | `feature` | Feasibility recorded in `docs/plans/archives/tradingview-cli-indicator-alertcondition-feasibility.md`. Rust intentionally does not expose the raw PR primitive because it depends on account-linked script metadata, exact alert-condition ids, Pine input payloads, and webhook/message mutation fields. The adopted Rust surface is local discovery with `tv pine alertconditions [--file <PATH>]` plus guarded `tv alert create-indicator ... [--dry-run]`; normal mode creates through the alert endpoint only when metadata can be resolved safely and verifies the new alert by readback. |
| [#110](https://github.com/tradesdontlie/tradingview-mcp/pull/110) `Support MSIX/Microsoft Store TradingView installs on Windows` | `bugfix/compatibility` | Windows live-verification backlog. The PR points at AUMID-style activation and IPv4 loopback behavior. Rust already addressed the IPv4 default in the CDP compatibility slice; AUMID / shortcut activation remains unimplemented until a Windows environment can prove it opens CDP reliably. |
| [#109](https://github.com/tradesdontlie/tradingview-mcp/pull/109) `Add .claude/ to .gitignore and sync package-lock.json bin entry` | `maintenance/node-only` | No Rust action. This is original-project packaging hygiene. |
| [#108](https://github.com/tradesdontlie/tradingview-mcp/pull/108) `fix: TradingView Electron 38 CDP compatibility` | `bugfix/compatibility` | Addressed for Rust in the CDP transport compatibility slice: the default host is `127.0.0.1`, `CdpClient::connect` no longer sends initial `Runtime.enable`, `Page.enable`, or `DOM.enable` calls, and `tv tab list` exposes app-window targets for diagnostics while chart command auto-selection still avoids file URL app-window targets. |
| [#107](https://github.com/tradesdontlie/tradingview-mcp/pull/107) `Integration: 16 fixes (alerts REST, DI restore, TV 3.1.0 compat, i18n, draw_shape hardening)` | `mixed` | Do not cherry-pick. Treat as a bundle to audit after the focused `#108`, `#110`/`#114`, and `#112` passes. Some areas are likely already addressed in Rust, such as API-backed watchlist and alert creation, while the TV Desktop 3.1.0 compatibility evidence may still be valuable. |

## Initial recommendations

1. The focused CDP transport compatibility pass for `#108` is addressed in
   Rust. Future work should monitor live TradingView Desktop regressions rather
   than re-opening domain-enable bootstrap by default.

2. Keep Windows Store/MSIX launch as a Windows live-verification backlog item.
   Detection and CDP-enabled launch are separate concerns. Until a Windows smoke
   proves Store/MSIX startup with the debug port, the Rust docs should recommend
   the standalone Desktop installer and leave AUMID / shortcut activation for a
   separate Windows-specific ExecPlan.

3. Treat `#112` as incorporated through a narrower Rust surface.
   Rust has local static discovery through `tv pine alertconditions [--file
   <PATH>]` and guarded account-session create/preview through
   `tv alert create-indicator ... [--dry-run]`. It still avoids raw
   saved-script-id, raw input, raw plot-offset, and webhook API exposure.

4. Keep `#113` outside the Rust core CLI.
   Workflow packs and trading bots are downstream material unless a later plan
   extracts a narrow command that clearly belongs here.

## Current closure boundary

Treat the upstream PR follow-up as complete enough for the current `v0.3.0`
planning phase. The remaining open upstream PRs are not ignored, but their Rust
dispositions are now one of:

- addressed by existing Rust implementation or docs;
- Windows live-verification backlog;
- intentionally outside the core CLI as workflow/helper material;
- evidence-gated deferred work such as layout-dialog policy or broader
  Screener editing;
- original-project Node/MCP maintenance with no Rust action.

Future upstream work should be periodic monitoring, or a focused re-check only
when a new upstream PR appears or a live Rust regression points back to an
upstream report.

## Next work

The next `v0.3.0` work item is not another broad upstream sweep. Move to
credential-safe direct HTTP feasibility for read-only surfaces, using
`docs/plans/archives/tradingview-cli-direct-http-feasibility.md` as the
completed research plan.
