---
name: chart-analysis
description: Analyze a live TradingView chart with the Rust `tv` CLI. Use when the user wants chart review, technical analysis, quote/OHLCV context, study values, symbol/timeframe setup, visible range checks, or screenshot-backed chart evidence.
---

# Chart Analysis

Use this skill for TradingView Desktop chart review with the Rust `tv` CLI.
Keep the chart source explicit: selected-chart reads are Desktop-backed, while
`tv snapshot`, `tv events`, and `tv bars` are Desktop-free context reads.

Use this `SKILL.md` for routine chart work. Read `references/workflow.md` only
when you need the detailed command map, source-boundary notes, quote-data
details, or unsupported-feature reminders. If command choice is unclear, check
`docs/observation-workflows.md`.

## Start With Readiness

1. Run `tv readiness`.
2. If there is no connection, run `tv launch` once. On macOS, normal
   `tv launch` uses the system app launcher. Use `tv launch --path <PATH>`
   only when the user intentionally wants a specific executable.
3. If the app still cannot connect, say that TradingView Desktop may need to
   be started manually or with an explicit path.
4. If multiple chart targets are open, run `tv tab list` and use the returned
   `target_cli_args`, for example `tv --target-id <ID> ...`.
5. Use `tv state`, `tv range`, or `tv ohlcv --summary` when readiness says the
   selected chart or bar state still needs confirmation.

## Core Workflow

1. Before mutating the chart, gather Desktop-free context only when useful:
   `tv snapshot <SYMBOL>`, `tv quote <SYMBOL>`, `tv info <SYMBOL>`,
   `tv fundamentals <SYMBOL> --group ...`, or `tv events <SYMBOL>`.
2. Change chart state only when the task needs selected-chart evidence:
   `tv symbol <SYMBOL>`, `tv timeframe <RESOLUTION>`, and `tv type <TYPE>`.
3. After changing symbol or timeframe, confirm fresh chart data with
   `tv ohlcv --count 1` or `tv ohlcv --summary`.
4. Read selected-chart evidence with `tv state`, `tv quote --source chart`,
   `tv ohlcv --summary`, `tv values`, and `tv data lines|labels|tables|boxes`.
5. Use `tv screenshot --region chart --output <PATH>` when visual evidence
   would materially help.
6. Use `tv observe chart --duration-ms ...` only for a short selected-chart
   observation window. Use `tv stream ...` only when a lower-level sample type
   is specifically needed.

## Historical And Export Evidence

- Use `tv bars <SYMBOL> --from ... --to ...` for reproducible Desktop-free
  historical bars. Bare symbols may resolve through symbol search; report
  `requested_symbol`, `resolved_symbol`, and `symbol_resolution`.
- Use `tv export chart-bars --from <UNIX_SECONDS> --to <UNIX_SECONDS>` only
  when the selected TradingView Desktop chart itself is the source under
  review. Report `export_chart_bars.v1`, chart context, requested and actual
  visible range, returned bars range, and range-match status.
- Use `tv replay status` or `tv replay log --steps <N>` only as
  Desktop-backed Replay workflow evidence. Replay operations mutate chart
  state; they are not a replacement for `tv bars`.

## Source Boundaries

- Do not blend Desktop-free scanner reads, selected-chart reads, `tv bars`,
  chart export, Replay, screenshots, or quote-data into one unstated source.
- `tv quote --source auto` is chart-first with scanner fallback before chart
  mutation. Report the actual source metadata instead of describing it as a
  combined feed.
- `follow_up_hints[]` from `snapshot.v1` or `compare.v1` are advisory
  evidence surfaces. `auto_execute: false` means the CLI did not run them.
- `tv events` is scanner-backed earnings/dividend evidence, not a full event
  calendar and not selected-chart evidence.
- Chart-backed compare is not a stable command. Use Desktop-free `tv compare`
  or `tv watch compare` for broad comparison, then selected-chart evidence for
  finalists.

## Recovery

If `tv ohlcv` fails but symbol or quote reads work, keep the full JSON error
envelope. Inspect `error.details.phase`, `bar_index_state`,
`chart_readiness`, and `next_action_hint`; then rerun `tv readiness`,
choose a target with `tv tab list` if needed, and retry with that
`--target-id`.

If a visual check is needed and the current environment supports it, use it as
an optional aid after structured CLI checks. Do not make visual-control tools
part of the default workflow for packaged or CLI-only agents.

## Reporting

Lead with the practical chart read, then cite the CLI evidence. Separate
observed values from inference. Avoid ranking, recommendation, buy/sell advice,
or invented indicator values that were not returned by `tv values` or visible
in inspected evidence.
