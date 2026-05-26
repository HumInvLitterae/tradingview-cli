# TradingView CLI Agent Guide

This guide is bundled in release archives for users and agents operating the
`tv` binary. It is not a contributor guide.

## Purpose

Use `tv` as the command-line interface for TradingView reads and automation.
Some commands run without TradingView Desktop. Commands that read or operate
the live chart, UI, Pine Editor, Replay, Screener, alerts, layouts, watchlists,
or drawings use the user's own local TradingView Desktop session through Chrome
DevTools Protocol.

This project is not affiliated with TradingView Inc. It does not bypass
TradingView access controls, subscriptions, paywalls, exchange data agreements,
or script ownership rules. Market data, Pine scripts, alerts, layouts, and
account state remain subject to TradingView and data-provider terms.

## Finding the CLI

Prefer `tv` when it is on `PATH`. If the archive was unpacked but not
installed, use the local executable in the unpacked directory:

- macOS/Linux: `./tv`
- Windows: `.\tv.exe`

Run `tv --version` to confirm which binary is available. When reporting
commands to the user, write them as `tv ...` unless the local executable path
matters.

For a user-first setup walkthrough, read `docs/getting-started.md` from the
release archive. Japanese user guidance is available at
`docs/ja/getting-started.md`.

## First Checks

Use Desktop-free reads when they are enough:

```bash
tv quote AAPL
tv quotes AAPL MSFT NYSE:IONQ
tv info NASDAQ:AAPL
tv fundamentals NYSE:IONQ --group earnings
tv scanner scan --limit 10
```

Before chart-dependent reads or operations, check Desktop readiness:

```bash
tv readiness
```

If TradingView Desktop is not connected, run:

```bash
tv launch
```

If `tv launch` cannot find TradingView Desktop, ask the user for the executable
path and use `tv launch --path <PATH>`.

If more than one chart target is open, run `tv tab list`, choose the intended
target with the user, and reuse that target's `target_cli_args`, for example:

```bash
tv --target-id <ID> state
tv --target-id <ID> ohlcv --count 1
```

Do not use `TV_CDP_TARGET_ID`; explicit target handoff is `--target-id`.

## Source Categories

`tv` is one binary with different source categories:

- Desktop-free reads do not need TradingView Desktop. Prefer them for broad
  market data and symbol discovery.
- Desktop-backed reads depend on the selected Desktop target or visible chart
  state. Use `tv readiness` first when target or chart state may be unclear.
- Desktop-backed operations may change chart, account, editor, Replay,
  Screener, layout, drawing, alert, watchlist, or UI state.
- Hybrid commands choose between sources explicitly, such as
  `tv quote <SYMBOL> --source auto`.
- Browserless historical bars use `tv bars <EXCHANGE:SYMBOL>` as a bounded
  Desktop-free read with `contract_version: "bars.v1"`. They do not guarantee
  realtime or entitlement status. Use `--from YYYY-MM-DD --to YYYY-MM-DD`
  with `--timeframe 15`, `60`, `1D`, `1W`, or `1M` for reproducible older
  intraday, daily, weekly, or monthly samples; other intraday timeframes
  remain guarded in date-range mode. `--to` is an inclusive calendar date.
  Read `summary` / `range`, `requested_range` / `returned_range`,
  `range_coverage_status`, and `range_alignment` before inspecting raw
  `bars[]`. In date-range mode, `--count` defaults to 500 and may be raised
  up to 5000 as a returned-bar safety cap; recent count mode remains capped at
  500. Read
  `range_fetch_summary` for fetch-window count, `request_more_data` count,
  returned-count caps, and truncation reasons, and read
  `source_availability` / `wait_summary` when bars are partial or unavailable.
- Selected-chart JSONL observations use `tv observe chart` and lower-level
  `tv stream ...`. Read readiness, sample, heartbeat, and final summary events
  by `contract_version` (`observe_chart.v1` or `stream.v1`), `_event`, and
  source metadata. Summary events describe the bounded observation window; they
  are not market-data samples.

## Safety Rules

- Prefer read-only commands first: `readiness`, `status`, `tab list`, `state`,
  `info`, `fundamentals`, `quote`, `quotes`, `ohlcv`, `values`,
  `scanner scan`, `scanner metainfo`, `watchlist get`, `pane list`,
  `layout list`, `alert list`, `pine get`, and `screenshot`.
- Use `tv screenshot --region chart|full --output <PATH>` when visual evidence
  is needed. Screenshots do not mutate TradingView state but do write the
  requested local file.
- Before mutating chart, account, Pine, Replay, layout, tab, drawing, alert,
  watchlist, Screener, or generic UI state, explain the expected effect and get
  explicit user approval.
- Use dry-run modes when available, especially for broad actions such as
  `alert delete --all --dry-run`, `draw clear --dry-run`,
  `layout switch --dry-run`, and Screener mutations.
- Do not record real account-local identifiers in shared notes unless the user
  explicitly asks. Scrub saved-script ids, saved-script names, alert ids,
  layout ids, chart target ids, usernames, emails, account names, and
  machine-local paths.
- Never print secrets, cookies, session data, or private credentials. The CLI
  should operate through the user's own local TradingView session.

## Useful Skills

The release archive includes CLI-oriented skills under `.agents/skills/` and
`.claude/skills/`:

- `chart-analysis`: live chart review and screenshot-backed context.
- `market-data-interpretation`: quote, scanner, chart, OHLCV, freshness, and
  extended-hours interpretation.
- `multi-symbol-scan`: small serial symbol scans and comparisons.
- `pine-develop`: Pine Script read/edit/check/compile workflows.
- `replay-practice`: bounded TradingView replay practice.
- `screener-result-analysis`: scanner and Screener result explanation without
  turning rows into buy or sell recommendations.
- `screener-workflow`: Stock Screener reads, target selection, dry-run-first
  operations, and disposable test-screen cleanup.
- `strategy-report`: strategy metrics, trades, and equity review.

Use those skills when the user's request matches their descriptions.
