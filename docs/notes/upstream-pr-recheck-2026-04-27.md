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

## New PR delta

| PR | Category | Rust disposition |
| --- | --- | --- |
| [#114](https://github.com/tradesdontlie/tradingview-mcp/pull/114) `Document Microsoft Store TradingView Desktop incompatibility` | `docs/compatibility` | Windows live-verification backlog. It says Microsoft Store/MSIX TradingView Desktop may not expose the requested remote debugging port because of packaged-app sandboxing. Rust now records the boundary in `docs/notes/windows-store-msix-launch-boundary.md`: standalone Desktop is the recommended launch path until Store/MSIX CDP startup is proven on Windows. |
| [#113](https://github.com/tradesdontlie/tradingview-mcp/pull/113) `Add advanced trading bot that reads live TradingView chart` | `workflow/helper` | Keep outside core CLI. It may inspire downstream examples or skills, but a trading bot, confluence engine, paper-position manager, and drawing workflow should not be imported into the Rust binary by default. |
| [#112](https://github.com/tradesdontlie/tradingview-mcp/pull/112) `feat(alerts): add alert_create_indicator for Pine alertcondition signals` | `feature` | Feature candidate, not an immediate patch. Rust has API-backed price alert creation, but not indicator/Pine `alertcondition()` alert creation. This needs a separate feasibility plan because it depends on script metadata and alert-condition identifiers. Do not copy account-linked examples from the PR body into tracked docs. |
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

3. Treat `#112` as a later feature feasibility slice.
   Indicator/Pine alert-condition alerts may belong in the CLI, but only after
   the script metadata, safety boundary, and readback contract are clear.

4. Keep `#113` outside the Rust core CLI.
   Workflow packs and trading bots are downstream material unless a later plan
   extracts a narrow command that clearly belongs here.

## Next re-check work

This starter pass only covers the newest delta since the previous triage note.
The next pass should work downward through the still-open list and update the
older dispositions where upstream PRs have been superseded, closed, or made more
specific by newer compatibility reports.
