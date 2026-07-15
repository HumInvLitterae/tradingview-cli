# Getting started with `tv`

This guide is for people and AI agents using a downloaded `tv` release archive.
It explains how to make the binary runnable, how to ask an agent to use it, and
how to run the first TradingView checks safely.

`tv` is a command-line tool for TradingView workflows. Some commands read data
without opening TradingView Desktop. Other commands connect to the user's local
TradingView Desktop session. `tv` does not bypass TradingView accounts,
subscriptions, paywalls, exchange agreements, or script ownership rules.

## 1. Download a release archive

Download the archive for your operating system from GitHub Releases:

- Windows: `tv-<tag>-x86_64-pc-windows-msvc.zip`
- macOS Apple Silicon: `tv-<tag>-aarch64-apple-darwin.tar.gz`
- macOS Intel: `tv-<tag>-x86_64-apple-darwin.tar.gz`
- Linux: `tv-<tag>-x86_64-unknown-linux-gnu.tar.gz`
- Checksums: `SHA256SUMS`

Verify the archive against `SHA256SUMS` when possible, then unpack it. The
archive contains the binary, README, changelog, license, getting-started docs,
agent guides, and runtime skills.

## 2. Put `tv` somewhere convenient

You can run `tv` directly from the unpacked directory:

- macOS/Linux: `./tv --version`
- Windows PowerShell: `.\tv.exe --version`

For regular use, place the executable on your `PATH` so both you and an AI
agent can run commands as `tv ...`:

```bash
tv --version
```

If the command prints the expected version, the binary is reachable.

## 3. Use `tv` with an AI agent

The release archive includes `AGENTS.md`, `CLAUDE.md`, and runtime skills under
`.agents/skills/` and `.claude/skills/`. Give those files to any coding or
assistant application that can read local files and run shell commands.

How to start depends on the application you use:

- If `tv` is on `PATH`, the agent can run commands from any project directory.
- If `tv` is not on `PATH`, tell the agent to set its current working directory
  to the unpacked release folder and run `./tv ...` on macOS/Linux or
  `.\tv.exe ...` on Windows.
- If the agent works inside another project, either put `tv` on `PATH` or give
  the agent the full path to the unpacked executable.

A good first instruction is:

> Use the bundled `tv` CLI. First run `tv --version`. Use commands that do not
> need TradingView Desktop when they are enough. Before reading the Desktop
> chart, run `tv readiness`. Report the command you ran and which kind of
> TradingView data it used. Do not change TradingView state without asking me
> first.

The important practical rule is that similar-looking commands can read
different things. For example, `tv bars` reads historical bars without
TradingView Desktop, while `tv ohlcv` reads bars from the selected Desktop
chart. They are both useful, but they should not be treated as the same
evidence.

Useful starting points:

- `tv quote`, `tv quotes`, scanner, fundamentals, `tv events`, and `tv bars`
  read without TradingView Desktop.
- `tv bars` is the reproducible historical bars entry point.
- `tv watch compare` watches a known candidate set for a short time using
  Desktop-free scanner-backed quote reads and one JSON object per line.
- `tv range` reads the visible Desktop chart range. With `--from` and `--to`,
  it boundedly loads older selected-chart history when needed, moves the
  viewport only when matching bars exist, and reports paging/coverage status.
- `tv ohlcv` reads bars from the selected Desktop chart.
- `tv quote --source quote-data` is an explicit Desktop-backed quote-data
  WebSocket read.
- `tv observe chart` and `tv stream ...` observe the selected Desktop chart for
  a bounded time and print one JSON object per line.

When `tv snapshot` or `tv compare` returns `follow_up_hints[]`, treat those
entries as possible next checks. They include the command, source category,
whether TradingView Desktop is required, and `auto_execute: false`. Ask the
agent to report those fields before it runs a separate follow-up command.

Ask the agent to say which command it used and what kind of source it read
from. That prevents accidental comparisons between a historical data command,
a selected-chart command, and a quote command. `tv` does not turn market data
into rankings, scores, buy/sell recommendations, or trading advice.

## 4. Run a Desktop-free smoke test

Commands that do not need TradingView Desktop are the safest first check:

```bash
tv quote AAPL
tv info NASDAQ:AAPL
tv bars AAPL --timeframe 1D --count 5
tv bars NASDAQ:AAPL --timeframe 1D --count 5
tv events NASDAQ:AAPL --event-type earnings
tv events NASDAQ:AAPL --event-type dividends
tv watch compare NASDAQ:AAPL NASDAQ:MSFT --duration-ms 10000 --interval 2000
```

Use `tv events` when you want an event-shaped view of scanner-backed earnings
and dividend fields. It is not a full event calendar, and it does not turn
those fields into rankings, recommendations, or trading advice.

For historical sample preparation, use `tv bars` rather than moving a visible
Desktop chart:

```bash
tv bars NASDAQ:CRUS --timeframe 1D --from 2010-01-01 --to 2010-12-31
tv bars NASDAQ:CRUS --timeframe 1W --from 2010-01-01 --to 2010-12-31
tv bars NASDAQ:CRUS --timeframe 1M --from 2010-01-01 --to 2010-12-31
tv bars NASDAQ:AAPL --timeframe 5 --from 2026-05-20 --to 2026-05-22 --count 1000
tv bars NASDAQ:AAPL --timeframe 60 --from 2026-05-01 --to 2026-05-22 --count 1000
```

`tv bars` can resolve a bare symbol such as `AAPL` through Desktop-free
TradingView symbol search. If the exchange matters, use `EXCHANGE:SYMBOL`,
such as `NASDAQ:AAPL`. Ask your agent to report `requested_symbol`,
`resolved_symbol`, and `symbol_resolution` before using the returned bars, so
you can see whether the command used the intended exchange.

In date-range mode, read `range_coverage_status` and `range_alignment` before
interpreting raw `bars[]`. Date-range mode currently supports `5`, `15`,
`30`, `60`, `1D`, `1W`, and `1M`; other intraday timeframes remain guarded.
Intraday, weekly, and monthly bars use period-start timestamps and are filtered by
timestamps within the requested inclusive calendar range. The `--count` option
is the maximum number of bars to return in date-range mode; it defaults to 500
and can be raised up to 5000. For recent count mode, the maximum stays 500.
Read `range_fetch_summary` when you need to know whether the command used
additional fetch windows, reached the returned-bar safety cap, or stopped
because the source or bounded wait could not cover the full range.

## 5. Start TradingView Desktop for chart workflows

Commands that read or operate the selected Desktop chart require a local
TradingView Desktop session. This setup is a little different from opening the
app normally: `tv` needs TradingView Desktop to run with a local connection
port enabled so the command-line tool can talk to the app.

The easiest path is to let `tv` start or reuse TradingView Desktop:

```bash
tv launch
tv readiness
tv tab list
tv state
```

`tv launch` first checks whether the local connection is already available. If
it is, the command reuses the existing Desktop session. If it is not, the
command tries to start TradingView Desktop with the connection option that `tv`
needs. After that, `tv readiness` confirms that a chart target is available,
`tv tab list` shows available targets, and `tv state` confirms that the
selected chart can be read.

On macOS, the normal `tv launch` path uses the system app launcher so
TradingView Desktop is not tied to the command's child-process lifetime. Use
`tv launch --path <TRADINGVIEW_DESKTOP_PATH>` only when you intentionally want
to start a specific executable. Use `--kill-existing` only when you are ready
to terminate an existing TradingView Desktop session and start it again with
the local connection option.

Both direct spawn and the normal macOS system launch remove an incompatible
inherited Electron mode before starting TradingView. If launch succeeds with
`cdp_ready: false`, the app may still be loading; run `tv readiness` before
retrying. A structured connection error after direct spawn means the child
exited or its state could not be verified. Start the app manually or correct
the explicit path before retrying. Do not add `--kill-existing` unless you have
explicitly decided to terminate the current Desktop session.

If `tv launch` cannot find TradingView Desktop, use:

```bash
tv launch --path <TRADINGVIEW_DESKTOP_PATH>
```

If you are working with an AI agent, ask it to run the same sequence and report
the result of each step:

> Start TradingView Desktop for `tv` by running `tv launch`, then run
> `tv readiness`, `tv tab list`, and `tv state`. If `tv launch` cannot find the
> app, stop and ask me for the TradingView Desktop path. Do not change the chart
> symbol, timeframe, alerts, drawings, or account state while doing this setup.

You can also start TradingView Desktop yourself first, then ask the agent to
run `tv readiness`. If readiness fails, run `tv launch` from the CLI so the app
is started with the local connection option that `tv` needs.

When more than one TradingView target is open, use the `target_cli_args` shown
by `tv tab list` or `tv readiness`. The command will look like:

```bash
tv --target-id <ID> state
```

Treat the target id as local session metadata. Do not paste real target ids,
account-local ids, cookies, tokens, or private paths into shared notes.

## 6. Confirm chart reads before chart operations

Use read-only chart checks before changing chart, account, editor, Replay,
Screener, alert, watchlist, drawing, or layout state:

```bash
tv state
tv ohlcv --summary --count 100
tv screenshot --region chart --output target/tv-chart.png --wait-for-render
```

The wait flag is useful immediately after changing chart or panel state. It is
opt-in; without it, screenshot capture remains immediate. A bounded readiness
timeout writes no image. Use `--wait-timeout-ms <500..30000>` only with
`--wait-for-render` when the default 5000 ms is not suitable.

If you need to watch the selected chart briefly, use a bounded JSON-lines
observation:

```bash
tv observe chart --duration-ms 10000 --heartbeat-ms 2000
tv stream bars --max-events 5
```

These commands print readiness, sample, heartbeat, and final summary events.
They observe the selected Desktop chart. They are not Desktop-free historical
bars and they are not a multi-symbol realtime feed.

If you need to watch several known symbols briefly without using TradingView
Desktop, use:

```bash
tv watch compare NASDAQ:AAPL NASDAQ:MSFT --duration-ms 10000 --interval 2000 --heartbeat-ms 3000
```

This prints JSON-lines events with `contract_version: "watch_compare.v1"`.
Read the readiness, sample, heartbeat, and summary events by `_event`. It uses
scanner-backed quote reads, not the selected Desktop chart.

## 7. Next references

- `README.md`: project overview and command examples.
- `AGENTS.md` / `CLAUDE.md`: runtime guide for agents using a release archive.
- `docs/command-source-taxonomy.md`: detailed command source categories in the
  repository.
- `docs/observation-workflows.md`: practical read sequences in the repository.
