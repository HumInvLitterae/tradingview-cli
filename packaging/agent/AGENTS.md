# TradingView CLI Agent Guide

This file is for users who unpack a `tv` release archive and want an AI agent to operate TradingView through the bundled command-line tool.

## Purpose

Use the bundled `tv` binary as the only interface for TradingView automation. The tool connects to the user's own running TradingView Desktop session through Chrome DevTools Protocol on `127.0.0.1:9222`.

This project is not affiliated with TradingView Inc. It does not bypass TradingView access controls, subscriptions, paywalls, exchange data agreements, or script ownership rules. Market data, Pine scripts, alerts, layouts, and account state remain subject to TradingView and data-provider terms.

## Finding the CLI

Prefer `tv` when it is on `PATH`. If the release archive was unpacked but not installed, use the local executable in the unpacked directory:

- macOS/Linux: `./tv`
- Windows: `.\tv.exe`

When reporting commands to the user, write them as `tv ...` unless the local executable path matters.

## Startup

1. Run `tv status`.
2. If TradingView is not connected, run `tv launch` once.
3. On Windows, prefer the standalone TradingView Desktop install. Microsoft Store/MSIX `TradingView.Desktop 3.1.0.7818` has been smoke-tested through the current direct AppX launch path, but Store/MSIX behavior can still change with TradingView Desktop, Electron, or Microsoft Store packaging updates.
4. If `tv launch` cannot find TradingView Desktop, ask the user for the executable path and use `tv launch --path <PATH>`.
5. If more than one chart target is open, run `tv tab list`, ask the user which target to use, and reuse that target's `target_cli_args`, for example `tv --target-id <ID> quote`, for chart-specific commands.
6. Do not use `TV_CDP_TARGET_ID`; explicit target handoff is `--target-id`.

## Safety Rules

- Prefer read-only commands first: `status`, `state`, `info`, `quote`,
  `quotes`, `ohlcv`, `values`, `scanner scan`, `scanner metainfo`,
  `watchlist get`, `pane list`, `layout list`, `alert list`, `pine get`, and
  `screenshot`.
- If `ohlcv` fails while `quote` or `symbol` works, keep the full JSON error envelope, inspect `error.details`, rerun `tv tab list`, choose the active target's `target_cli_args`, run `tv --target-id <ID> state`, and retry `tv --target-id <ID> ohlcv --count 1`.
- Use `tv timeframe <RESOLUTION>` for timeframe changes. `tv interval` is not
  a command. Use `tv info <SYMBOL>`, `tv quote <SYMBOL>`, and
  `tv quotes <SYMBOL>...` for Desktop-free symbol metadata and quote reads; use
  `tv info` or `tv quote` without a symbol only for current-chart metadata or
  quote data. Scanner-backed quote reads expose `time`, `update_mode`,
  `delay_seconds`, and extended-hours fields when TradingView returns them, but
  they are screening reads rather than realtime entitlement guarantees. Use
  `tv quote <SYMBOL> --source chart` only when the selected TradingView Desktop
  chart feed matters.
- Before mutating chart, account, Pine, replay, layout, tab, drawing, alert, or watchlist state, explain the expected effect and get explicit user approval.
- Use dry-run modes when available, especially for broad actions such as `alert delete --all --dry-run`, `draw clear --dry-run`, and `layout switch --dry-run`.
- Do not record real account-local identifiers in shared notes unless the user explicitly asks. Scrub saved-script ids, saved-script names, alert ids, layout ids, chart target ids, usernames, emails, account names, and machine-local paths.
- Never print secrets, cookies, session data, or private credentials. The CLI should operate through the user's existing local TradingView Desktop session.

## Useful Skills

The release archive includes CLI-oriented skills under `.agents/skills/` and `.claude/skills/`:

- `chart-analysis`: live chart review and screenshot-backed context.
- `market-data-interpretation`: quote, scanner, chart, OHLCV, freshness, and
  extended-hours interpretation.
- `multi-symbol-scan`: small serial symbol scans and comparisons.
- `pine-develop`: Pine Script read/edit/check/compile workflows.
- `replay-practice`: bounded TradingView replay practice.
- `screener-result-analysis`: scanner and Screener result explanation without
  turning rows into buy or sell recommendations.
- `screener-workflow`: Stock Screener target selection, reads, dry-run-first
  screen/filter/column operation, and disposable test-screen cleanup.
- `strategy-report`: strategy metrics, trades, and equity review.

Use those skills when the user's request matches their descriptions.
