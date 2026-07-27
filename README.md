# TradingView CLI

`tv` is a Rust-native command-line tool for TradingView workflows. It combines
TradingView Desktop automation through Chrome DevTools Protocol with
Desktop-free TradingView data reads for quotes, scanner rows, symbol metadata,
and fundamentals.

This project is inspired by practical workflows built around
[TradingView MCP Bridge](https://github.com/tradesdontlie/tradingview-mcp) by
`tradesdontlie`, but this repository is intentionally CLI-first. It is not an
MCP server and is not affiliated with TradingView Inc.

This tool requires the user's own valid TradingView account, data
entitlements, and, for Desktop-backed commands, a local TradingView Desktop
session. It does not bypass TradingView access controls, subscriptions,
paywalls, or exchange/data-provider licensing. Market data, Pine scripts,
alerts, layouts, and account state remain subject to TradingView and
data-provider terms.

## Installation

GitHub Releases are the first supported binary distribution path. For a
non-developer walkthrough from download to first checks, read the
[getting-started guide](docs/getting-started.md) or the
[Japanese getting-started guide](docs/ja/getting-started.md).

Version tags such as `v0.30.2` publish native archives like:

- `tv-<tag>-x86_64-unknown-linux-gnu.tar.gz`
- `tv-<tag>-x86_64-apple-darwin.tar.gz`
- `tv-<tag>-aarch64-apple-darwin.tar.gz`
- `tv-<tag>-x86_64-pc-windows-msvc.zip`
- `SHA256SUMS`

Each archive contains the binary, README, changelog, license, getting-started
docs, user-facing agent guides, and runtime-oriented TradingView CLI skills.
Verify the archive against `SHA256SUMS`, unpack it, place the executable on
your `PATH`, and confirm the binary:

```bash
tv --version
```

During local development, build or install from the workspace root:

```bash
cargo install --path crates/cli
```

Package-manager installers, code signing, notarization, and crates.io
publication are not part of the current release workflow.

## Quick Start

For the full first-run sequence, including agent setup and TradingView Desktop
startup choices, read the [getting-started guide](docs/getting-started.md) or
the [Japanese getting-started guide](docs/ja/getting-started.md).

For agent-assisted use, put `tv` on `PATH`, ask the agent to run
`tv --version` first, prefer Desktop-free reads when they are enough, and run
`tv readiness` before Desktop-backed chart reads or operations. The release
archive includes `AGENTS.md`, `CLAUDE.md`, and runtime skills for agent use. If
`tv` is not on `PATH`, have the agent set its current working directory to the
unpacked release folder and run `./tv ...` on macOS/Linux or `.\tv.exe ...` on
Windows.

Desktop-free reads do not require TradingView Desktop:

```bash
tv search "Apple"
tv info NASDAQ:AAPL
tv quote AAPL
tv snapshot NASDAQ:AAPL
tv compare NASDAQ:AAPL NYSE:IONQ
tv quotes AAPL MSFT NYSE:IONQ
tv watch compare NASDAQ:AAPL NASDAQ:MSFT --duration-ms 10000 --interval 2000
tv fundamentals NYSE:IONQ --group earnings
tv fundamentals AAPL --group dividends
tv events NASDAQ:AAPL --event-type earnings
tv events NASDAQ:AAPL --event-type dividends
tv events compare NASDAQ:AAPL NASDAQ:MSFT --event-type earnings
tv scanner scan --type stock --columns name,close,volume --limit 10
tv scanner scan --type stock --sort name --asc --max-results 500 --page-size 100
tv scanner metainfo --market america --field close --field premarket_close
```

`tv compare` returns raw per-symbol evidence plus a machine-readable summary
for resolution, section success, and missing-value counts. It does not rank,
score, or recommend symbols. `tv quotes`, `tv compare`, and `tv events compare`
accept at most 25 symbols and preserve input order. Batch quote items include
their zero-based `requested_index`. See `docs/observation-workflows.md` for the
practical choice between `quotes`, `compare`, `snapshot`, and chart follow-up
commands.

`tv snapshot` and `tv compare` may include `follow_up_hints[]`. These are
machine-readable descriptions of possible next evidence checks, including
whether the follow-up requires TradingView Desktop and the source category it
would read. They are not automatic actions, ranking, source mixing, or trading
recommendations.

`tv watch compare` is a bounded JSONL workflow for a known candidate set. It
polls the same Desktop-free scanner-backed quote source used by `tv quotes`,
emits readiness / sample / heartbeat / summary events with
`contract_version: "watch_compare.v1"`, and does not rank, recommend, or use
TradingView Desktop.

`tv events <SYMBOL>` is a Desktop-free `events.v1` readback for scanner-backed
earnings and dividend fields. It is event-shaped evidence, not a full event
calendar. It does not infer timezone, before/after-market, ranking,
recommendation, or buy/sell meaning beyond the scanner values TradingView
returns.

`tv events compare <SYMBOL>...` extends the same scanner-backed event readback
to a small ordered candidate set with `contract_version:
"events_compare.v1"`. It is not a full event calendar and does not rank or
recommend symbols.

`tv scanner scan --max-results <N>` performs a bounded sequence of Desktop-free
scanner pages while retaining the 100-row per-request cap. It returns only
after every accepted page completes, deduplicates symbols in first-seen order,
and reports page, total-count, duplicate, timing, and drift metadata. The result
is a sequential observation, not an atomic market snapshot. Use `--offset` for
one diagnostic page; it cannot be combined with aggregate mode.

Scanner-backed `tv quote <SYMBOL>` and `tv quotes <SYMBOL>...` are
Desktop-free, but they are not a realtime guarantee. Inspect `time`,
`update_mode`, and `delay_seconds` in price-bearing payloads when freshness
matters.

To use Desktop-backed reads or operations, launch TradingView Desktop with CDP
enabled:

```bash
tv launch
tv readiness
tv tab list
tv state
tv ohlcv --summary --count 100
tv chart compare NASDAQ:AAPL NASDAQ:MSFT
tv export chart-bars --from 1704067200 --to 1706745600 --summary
tv screenshot --region chart --output target/tv-chart.png --wait-for-render
tv screenshot --region strategy --output target/tv-strategy.png
```

On macOS, `tv launch` uses the system app launcher for the normal no-path case
and then checks CDP readiness. Use `--path <PATH>` only when you intentionally
want to start a specific executable. `--kill-existing` is opt-in because it can
terminate an existing TradingView Desktop session.

Both direct spawn and the macOS system launcher remove an incompatible
inherited Electron mode before starting TradingView. A warning response with
`cdp_ready: false` means the process may still be loading; run `tv readiness`
before retrying. A structured connection error after direct spawn means the
child exited or its state could not be verified. Start the app manually or
correct an explicit path before retrying, and do not add `--kill-existing`
without explicit approval.

`tv chart compare <SYMBOL>...` is a Desktop-backed comparison for a small
finalist set. It temporarily switches the selected chart to each symbol, reads
chart quote evidence, and reports `chart_compare.v1` item status and restore
readback. Use Desktop-free `tv compare` or `tv watch compare` for broad
scanner-backed comparison.

If multiple TradingView targets are open, use `target_cli_args` returned by
`tv tab list` or `tv readiness`:

```bash
tv --target-id <CDP_TARGET_ID> state
tv --target-id <CDP_TARGET_ID> ohlcv --count 1
```

Resolve the intended target once near the start of a workflow and reuse those
same `target_cli_args` for later chart-dependent commands. Re-run target
discovery when selection fails, the target set changes, or the user changes the
intended chart; do not require a separate readiness call before every read.

Common Desktop operations:

```bash
tv symbol NASDAQ:AAPL
tv timeframe 1D
tv values
tv watchlist get
tv alert list
tv pine get
tv pine open "Saved Script"
tv screener open --full-page
tv screener filters add --name RSI --min 70 --dry-run
tv draw position long --entry-price 100 --stop-loss 95 --take-profit 110
```

`tv pine open <NAME...>` is a Desktop-backed editor operation. It opens the
resolved saved script through TradingView's script manager and succeeds only
after the active saved-script binding is read back and verified. It does not
save or compile the script, and it does not fall back to source-only Monaco
replacement when binding cannot be verified. `switch_performed` distinguishes
an actual script switch from verifying an already active matching script. A
script that is not the active one must also appear as one unique exact row in
the popup semantically linked to the Pine-owned saved-script trigger;
otherwise the command fails closed without changing source or saving.
The success payload reports `script_id_available` and
`script_identity_verified` instead of exposing the account-local saved-script
ID.

Bounded stream observations emit newline-delimited JSON:

```bash
tv observe chart --duration-ms 10000 --heartbeat-ms 2000
tv stream quote --duration-ms 10000 --heartbeat-ms 2000
tv stream bars --max-events 5
```

Use `tv observe chart` when you want one Desktop-backed window that starts
with readiness details, follows the selected chart's last bar, and ends with a
summary event describing counts, elapsed time, controls, and end reason. Use
the lower-level `tv stream ...` commands when you already know which specific
chart sample type you need; bounded stream runs also end with a summary event.

`tv values` returns formatted selected-chart study values. Each row also
reports `entity_id`, `short_name`, `study_kind`, compact `inputs`, and
`visible` when available, so same-name study instances can be distinguished
without chart-order guesses. `tv stream values` uses the same identity fields
for visible numeric-value samples; identity or input changes are evidence, not
automatic study mutation.

For an explicit Desktop-backed WebSocket quote-data readback, use:

```bash
tv quote NASDAQ:RKLB --source quote-data
```

This source reports TradingView quote-data readbacks such as `qsd.rtc` or
regular quote-data `qsd.v.lp` separately from chart main-series quotes and
scanner `extended_hours`.

Browserless historical bars are Desktop-free and bounded:

```bash
tv bars AAPL --timeframe 1D --count 5
tv bars NASDAQ:AAPL --timeframe 1D --count 5
tv bars NASDAQ:AAPL --timeframe 1 --from 2026-05-20 --to 2026-05-20 --count 1000
tv bars NASDAQ:AAPL --timeframe 5 --from 2026-05-20 --to 2026-05-22 --count 1000
tv bars NASDAQ:AAPL --timeframe 60 --from 2026-05-01 --to 2026-05-22 --count 1000
tv bars NASDAQ:CRUS --timeframe 1D --from 2010-01-01 --to 2010-12-31
tv bars NASDAQ:CRUS --timeframe 1W --from 2010-01-01 --to 2010-12-31
```

Bare symbols such as `AAPL` are resolved through Desktop-free TradingView
symbol search before bars are read. Use `NASDAQ:AAPL` or another
`EXCHANGE:SYMBOL` form when the exchange matters. Read `requested_symbol`,
`resolved_symbol`, and `symbol_resolution` first so an agent can report what
was typed and what was actually used.

Then read `summary`, `range`, `requested_range`, `returned_range`, and
`range_coverage_status` for requested-vs-returned count and time coverage,
then inspect raw `bars[]` when exact OHLCV evidence is needed. In date-range
mode, `--count` is a safety cap on returned bars and defaults to 500. It can
be raised up to 5000 in date-range mode; recent count mode stays capped at
500. Date-range mode currently supports `1` (and its `1m` alias), `5`, `15`,
`30`, `60`, `1D`, `1W`, and `1M`; other intraday timeframes remain guarded. The `--to` date is an
inclusive calendar date. For intraday, weekly, and monthly date ranges, read
`range_alignment` to see that bar timestamps are period anchors and filtering
uses timestamps within the requested range. Read
`range_fetch_summary` to see bounded fetch-window counts, added
`request_more_data` attempts, returned-count caps, and truncation reasons.
Read `source_availability` and its `wait_summary` when bars are partial or
unavailable; those fields describe bounded historical-source behavior, not a
trading recommendation or proof that a symbol has no history. Use
`tv range` only for selected Desktop chart viewport movement; it is not a
historical export contract for `tv ohlcv`. Bounded
`tv range --from <UNIX_SECONDS> --to <UNIX_SECONDS>` requests older history
from that selected chart's main series when needed and reports
`history_paging` plus `viewport_application`. Read coverage, stop reason,
matching-bar count, and clamp status before treating the requested viewport as
applied. It does not fall back to Desktop-free `tv bars` or another source.

Use `tv export chart-bars --from <UNIX_SECONDS> --to <UNIX_SECONDS>` only when
you intentionally want the selected TradingView Desktop chart as the source. It
moves the visible range, reads selected-chart bars, and returns
`export_chart_bars.v1` diagnostics. For reproducible symbol-targeted
historical bars, prefer Desktop-free `tv bars --from/--to`.

Use `tv replay log --steps <N>` only when you intentionally want a bounded
record of the selected chart's Replay state transitions. It emits
`replay_step_log.v1` JSONL events and does not start or stop Replay, export
bars, capture screenshots, or replace `tv bars --from/--to`. Add
`--attach-ohlcv-summary [--ohlcv-count <N>]` only when each Replay step should
also carry explicit selected-chart OHLCV summary evidence; that attachment has
its own `replay_log_ohlcv_summary_attachment.v1` source metadata.

Use `tv screenshot --region strategy --output <PATH>` when a report needs
visual evidence of the visible Strategy Tester panel. It is screenshot
evidence only; use `tv data strategy`, `tv data trades`, and `tv data equity`
for structured strategy fields when TradingView exposes them. These reads now
return the same additive `strategy_context`, including candidate count,
selection reason, visibility, report availability, and an explicit unavailable
state. They do not open Strategy Tester or unhide a strategy.

Use `tv --help` for the full command list and `tv <COMMAND> --help` for command
details. See `docs/observation-workflows.md` for practical command sequences
that combine Desktop-free screening, Desktop-backed chart observation,
selected-chart export, screenshots, browserless bars, and fundamentals reads.

Create a native parallel channel with `tv draw shape --type parallel_channel`
and three explicit point pairs. The third pair is TradingView's width point,
so `--time3` must equal the first `--time`; use loaded bar timestamps for the
first two anchors. Success returns one verified chart-local `entity_id` for
exact follow-up with `tv draw get` or `tv draw remove`.

## What `tv` Does

`tv` is one binary with several source categories:

- Desktop-free reads: symbol search, symbol info, scanner-backed quote reads,
  batch quotes, fundamentals, scanner scans, hotlists, and metainfo.
- Desktop-backed reads: chart state, OHLCV from the selected chart, screenshots,
  readiness diagnostics, and chart-source quotes.
- Desktop-backed operations: chart symbol/timeframe/type changes,
  selected-chart export, watchlist, alerts, drawings, Pine Editor, Replay,
  Screener, panes, layouts, tabs, and compatibility UI automation.
- Hybrid commands: commands with explicit source or fallback behavior, such as
  `tv quote <SYMBOL> --source auto`.
- Browserless historical bars: bounded Desktop-free `tv bars`.

See `docs/command-source-taxonomy.md` for the durable command classification
and source/fallback semantics.

## Safety Boundary

Some commands only read public or browser-accessible TradingView data. Other
commands operate the user's local Desktop session or page state. Before using a
command that may change chart, account, editor, Replay, Screener, drawing,
alert, watchlist, or UI state, prefer read-only commands and dry-run modes where
available.

The default Chrome DevTools endpoint is `127.0.0.1:9222`. Override it with
`TV_CDP_HOST` and `TV_CDP_PORT` only when your local setup requires it.

## Output Contract

Most successful commands print one JSON envelope to stdout:

```json
{
  "success": true,
  "command": "quote",
  "data": {
    "symbol": "NASDAQ:AAPL"
  }
}
```

Errors use the same envelope shape on stderr:

```json
{
  "success": false,
  "command": "quote",
  "error": {
    "kind": "connection",
    "message": "CDP connection failed",
    "details": {
      "failure_stage": "websocket_connect"
    }
  }
}
```

Desktop CDP transport errors may include `failure_stage` with one of
`target_list`, `target_select`, `websocket_connect`, `method_call`,
`event_wait`, or `transport_unknown`. This classifies where the error surfaced;
it does not mean the command is safe to retry. Existing error kind, message,
and exit-code semantics remain authoritative.

`tv stream ...` commands print newline-delimited JSON envelopes. Stream samples
use `_event: "sample"`, optional heartbeats use `_event: "heartbeat"`, and
bounded normal exits emit a final `_event: "summary"` line. `tv observe chart`
starts with readiness, emits selected-chart sample or heartbeat events, and
also ends bounded runs with a summary line.

`tv watch compare ...` also prints newline-delimited JSON, but it is
Desktop-free and scanner-backed. Its events use `_watch: "compare"` and
`contract_version: "watch_compare.v1"` so agents can keep it separate from
selected-chart `observe` / `stream` events.

Exit codes are:

- `0`: success
- `1`: usage, validation, target ambiguity, or unexpected internal failure
- `2`: TradingView or CDP connection failure
- `3`: TradingView internal API unavailable
- `4`: timeout

The Rust CLI intentionally differs from the old JavaScript CLI wire format.
Downstream adapters should read command payloads from `data`. For migration
details, see `docs/breaking-changes-from-js-cli.md`.

## Documentation

- [docs/command-source-taxonomy.md](docs/command-source-taxonomy.md): command
  source categories, fallback
  boundaries, mutation expectations, and recommended agent use.
- [docs/observation-workflows.md](docs/observation-workflows.md): practical
  read sequences for screening,
  chart observation, screenshots, browserless bars, and fundamentals.
- [docs/architecture.md](docs/architecture.md): workspace architecture, crate
  boundaries, operation
  adapters, JSON contract, and safety model.
- [docs/rust-api.md](docs/rust-api.md): currently documented typed Rust API
  boundary for internal
  reusable read crates.
- [docs/development.md](docs/development.md): coding style, validation, tests,
  and contribution
  workflow.
- [docs/history-rewrite-recovery.md](docs/history-rewrite-recovery.md):
  existing-clone recovery guidance after the completed canonical history
  rewrite.
- [docs/release-packaging.md](docs/release-packaging.md): release archive contents and packaging checks.
- [docs/internal-tradingview-apis.md](docs/internal-tradingview-apis.md):
  public-safe reference for non-public
  TradingView dependencies.
- [docs/getting-started.md](docs/getting-started.md): user-facing setup, first
  checks, and AI-agent
  workflow.
- [docs/ja/getting-started.md](docs/ja/getting-started.md): Japanese
  user-facing setup and AI-agent
  workflow.
Historical notes and completed ExecPlans live under `docs/notes/` and
`docs/plans/archives/`. They explain how the current surface was built, but
they are not the best starting point for normal use.

## Development

The repository is a Cargo workspace. The `tv` binary is provided by the
`tradingview-cli` package under `crates/cli/`.

Useful local commands:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo metadata --no-deps --format-version 1
```

Optional local Git guardrails can be installed with Git 2.54 or newer:

```bash
mise run hooks:install
```

Without `mise`, run `scripts/install-config-hooks.sh`; on Windows, run
`scripts/install-config-hooks.ps1` from PowerShell. These hooks are local
helpers only. CI remains the authoritative validation baseline.

## License

This project is licensed under the MIT License. See `LICENSE`.
