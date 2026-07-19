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
tv events NASDAQ:AAPL --event-type earnings
tv scanner scan --limit 10
tv scanner scan --sort name --asc --max-results 500 --page-size 100
```

Use scanner aggregate mode only when one page is insufficient. It keeps a
100-row request cap and reports sequential-page drift and duplicate metadata;
do not interpret the combined rows as an atomic market snapshot. A page error
produces no partial successful aggregate.

Before chart-dependent reads or operations, check Desktop readiness:

```bash
tv readiness
```

If TradingView Desktop is not connected, run:

```bash
tv launch
```

Desktop CDP errors can include a public-safe `failure_stage` detail such as
`target_list`, `target_select`, `websocket_connect`, `method_call`, or
`event_wait`. Use it to describe where the failure surfaced, not as automatic
retry permission. A method may have been dispatched before a `method_call`
failure, so preserve the original command outcome and ask before repeating a
mutation.

On macOS, normal `tv launch` uses the system app launcher and then checks CDP
readiness. Use `tv launch --path <PATH>` only when the user intentionally wants
to start a specific executable. Use `--kill-existing` only with explicit user
approval because it can terminate an existing TradingView Desktop session.

Both direct spawn and the normal macOS system launch remove an incompatible
inherited Electron mode before starting TradingView. If launch returns a
warning with `cdp_ready: false`, run `tv readiness` before retrying because the
app may still be loading. Treat a structured connection error after direct
spawn as evidence that the child exited or could not be verified; ask the user
to start the app manually or correct the explicit path. Do not add
`--kill-existing` without explicit user approval.

If `tv launch` cannot find TradingView Desktop, ask the user for the executable
path and use `tv launch --path <PATH>`.

If more than one chart target is open, run `tv tab list`, choose the intended
target with the user, and reuse that target's `target_cli_args`, for example:

```bash
tv --target-id <ID> state
tv --target-id <ID> ohlcv --count 1
```

Resolve the intended target once near the start of the workflow and keep using
those same arguments. Discover targets again when selection fails, the target
set changes, or the user changes the intended chart; do not add a readiness
call before every read merely as a precaution.

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
- Browserless historical bars use `tv bars <SYMBOL>` as a bounded
  Desktop-free read with `contract_version: "bars.v1"`. They do not guarantee
  realtime or entitlement status. Bare symbols such as `AAPL` are resolved
  through Desktop-free symbol search; use `NASDAQ:AAPL` or another
  `EXCHANGE:SYMBOL` form when the exchange must be fixed. Report
  `requested_symbol`, `resolved_symbol`, and `symbol_resolution` before using
  returned bars. Use `--from YYYY-MM-DD --to YYYY-MM-DD`
  with `--timeframe 5`, `15`, `30`, `60`, `1D`, `1W`, or `1M` for reproducible older
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
- Bounded watch compare uses `tv watch compare <SYMBOL>...`. It is a
  Desktop-free scanner-backed JSONL workflow with `contract_version:
  "watch_compare.v1"`, not a daemon, selected-chart feed, ranking, or trading
  recommendation. Read readiness, sample, heartbeat, and summary events by
  `_event` and preserve `source: "scanner_scan_rest"` when reporting it.
- `tv snapshot` and `tv compare` may return `follow_up_hints[]`. These are
  advisory evidence checks, not automatic actions. Read `kind`, `command`,
  `requires_desktop`, `source_category`, `non_mutating`, `evidence_role`, and
  `auto_execute: false` before deciding whether to run a separate follow-up.
- `tv events <SYMBOL>` returns scanner-backed earnings and dividends readback
  with `contract_version: "events.v1"`. `tv events compare <SYMBOL>...`
  returns ordered multi-symbol `events_compare.v1` readback from the same
  source. These are event-shaped field evidence, not complete event calendars.
  Do not use event-like fields as ranking, recommendation, trading judgment,
  or hidden fallback evidence.
- `tv quotes`, Desktop-free `tv compare`, and `tv events compare` accept at
  most 25 symbols and preserve input order. Batch quote items include a
  zero-based `requested_index`; it is ordering metadata, not ranking.
- Use `tv chart compare <SYMBOL>...` only for a small finalist set where the
  selected TradingView Desktop chart feed itself is the source under review.
  It is Desktop-backed, may temporarily switch the selected chart, and returns
  `chart_compare.v1` with ordered item status and restore readback. Use
  `tv compare` and `tv watch compare` for Desktop-free first-pass comparison.
- Selected-chart historical export is explicit: use `tv export chart-bars
  --from <UNIX_SECONDS> --to <UNIX_SECONDS>` only when the selected TradingView
  Desktop chart itself is the intended source. It moves the visible Desktop
  chart range, reads selected-chart bars, and returns
  `export_chart_bars.v1` diagnostics. It is not a fallback for Desktop-free
  `tv bars --from/--to`.
- `tv range` without bounds reads the selected-chart viewport. Bounded
  `tv range --from/--to` may load older selected-chart main-series history and
  move the viewport. Inspect `history_paging` and `viewport_application` for
  coverage, stop reason, matching bars, and clamp status; do not treat it as a
  fallback to `tv bars` or as historical export completeness.
- Replay-based extraction is not a stable historical export. `tv replay
  status` is a Desktop-backed read with `replay_context`; `tv replay start`,
  `step`, `stop`, `autoplay`, and `trade` are Desktop-backed operations that
  change Replay state or Replay trade state. Use them only when Replay state is
  the evidence under review. Use `tv replay log --steps <N>` as a bounded
  JSONL record of Replay state transitions, not as source-prepared OHLCV. Keep
  that evidence separate from `tv bars`. Use
  `--attach-ohlcv-summary [--ohlcv-count <N>]` only when selected-chart OHLCV
  summary evidence should be explicitly attached to each Replay step. Use
  `--attach-chart-screenshot --screenshot-output-dir <DIR>` only when each
  successful step needs a deterministic local chart PNG. Existing files are
  never overwritten; screenshot failure is separate from Replay step failure.
- Selected-chart JSONL observations use `tv observe chart` and lower-level
  `tv stream ...`. Read readiness, sample, heartbeat, and final summary events
  by `contract_version` (`observe_chart.v1` or `stream.v1`), `_event`, and
  source metadata. Summary events describe the bounded observation window; they
  are not market-data samples.
- `tv values` and `tv stream values` study rows expose public-safe
  `entity_id`, `short_name`, `study_kind`, compact `inputs`, and `visible`.
  Use `entity_id` plus inputs to distinguish same-name studies. Do not infer
  identity from row order, and do not mutate a study merely because its
  identity appears in readback.

## Safety Rules

- Prefer read-only commands first: `readiness`, `status`, `tab list`, `state`,
  `info`, `fundamentals`, `quote`, `quotes`, `ohlcv`, `values`,
  `scanner scan`, `scanner metainfo`, `watchlist get`, `pane list`,
  `layout list`, `alert list`, `pine get`, and `screenshot`.
- Use `tv screenshot --region chart|full|strategy --output <PATH>` when visual
  evidence is needed. `strategy` captures the visible Strategy Tester panel
  when detectable. Screenshots do not mutate TradingView state but do write the
  requested local file. After changing chart or panel state, add
  `--wait-for-render` when capture must wait for stable selected-chart context;
  a timeout writes no image.
- For `tv data strategy|trades|equity`, inspect `strategy_context` before using
  returned counts. Hidden, unready, missing, or ambiguous strategy state is a
  source diagnostic; the commands do not open Strategy Tester or unhide a
  study.
- Before mutating chart, account, Pine, Replay, layout, tab, drawing, alert,
  watchlist, Screener, or generic UI state, explain the expected effect and get
  explicit user approval.
- For native three-point `parallel_channel` creation, pass paired `--price3`
  and `--time3` to `tv draw shape`; `--time3` must equal the first point's
  time. Keep the verified returned entity ID for exact inspection or removal.
- `tv pine open <NAME...>` changes Pine Editor's active saved-script binding
  but does not save or compile. Treat success as valid only when
  `slot_rebound` and `binding_verified` are true; on failure, do not proceed to
  `tv pine save` from an unverified editor state. `switch_performed: false`
  means the requested script was already active and still passed the same
  identity/version/name verification. If a non-active script is absent from
  the popup semantically linked to the Pine-owned saved-script trigger,
  `pine open` fails closed; do not replace that failure with source injection
  followed by save.
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
- `multi-symbol-scan`: bounded symbol scans and comparisons.
- `pine-develop`: Pine Script read/edit/check/compile workflows.
- `replay-practice`: bounded TradingView replay practice.
- `screener-result-analysis`: scanner and Screener result explanation without
  turning rows into buy or sell recommendations.
- `screener-workflow`: Stock Screener reads, target selection, dry-run-first
  operations, and disposable test-screen cleanup.
- `strategy-report`: strategy metrics, trades, and equity review.

Use those skills when the user's request matches their descriptions.
