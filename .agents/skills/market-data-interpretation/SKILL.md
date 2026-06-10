---
name: market-data-interpretation
description: Interpret TradingView market data returned by the Rust `tv` CLI. Use when explaining quote, batch quote, scanner, chart quote, OHLCV, extended-hours, freshness, missing-value, or source differences.
---

# Market Data Interpretation

Use this skill when the task is to explain market data returned by `tv`, not
just to run a command.

Keep the main answer practical: name the source, state the observed values,
separate interpretation from data, and say what remains unknown. Do not turn
CLI readback into ranking, recommendation, scoring, or buy/sell advice.

For detailed source semantics, read `references/source-boundaries.md` only when
the task needs them. If command choice is unclear, check
`docs/observation-workflows.md`.

## First Checks

1. Identify which source produced the value before interpreting it.
2. Preserve `contract_version`, `source`, `source_category`,
   `requires_desktop`, and `non_mutating` when they are present.
3. Treat missing or `null` fields as unknown, not as zero.
4. If sources disagree, report both sources rather than forcing one value.
5. If the task asks for a next read, choose one explicit command; do not imply
   that the CLI will run follow-ups automatically.

## Command Choice

| Need | Prefer |
| --- | --- |
| Several symbols, quote fields only | `tv quotes <SYMBOL>...` |
| Several known symbols with quote, info, and fundamentals | `tv compare <SYMBOL>...` |
| Several known symbols over a short scanner-backed window | `tv watch compare <SYMBOL>...` |
| Finalist set with selected-chart quote evidence | `tv chart compare <SYMBOL>...` |
| One symbol with Desktop-free detail | `tv snapshot <SYMBOL>` |
| One symbol's fundamentals | `tv fundamentals <SYMBOL> --group ...` |
| One symbol's earnings/dividend events | `tv events <SYMBOL>` |
| Several symbols' earnings/dividend events | `tv events compare <SYMBOL>...` |
| Reproducible historical bars | `tv bars <SYMBOL> --from ... --to ...` |
| Selected-chart quote or bars | `tv quote --source chart`, `tv ohlcv`, or `tv state` |
| Explicit visible-chart export | `tv export chart-bars --from ... --to ...` |
| Replay step context with bars summary | `tv replay log --steps <N> --attach-ohlcv-summary` |
| Visual evidence | `tv screenshot --region chart|full|strategy --output <PATH>` |

## Source Rules

- Desktop-free scanner reads are good first-pass evidence, but they are not
  realtime entitlement guarantees. Report `time`, `update_mode`, and
  `delay_seconds` when freshness matters.
- Chart-backed reads depend on the selected TradingView Desktop chart. Run
  `tv readiness` and use `tv tab list` / `--target-id` when the active chart is
  ambiguous.
- `tv chart compare` is Desktop-backed `chart_compare.v1` for a small finalist
  set. It may temporarily switch the selected chart. Do not describe it as
  scanner-backed compare or as ranking.
- `tv quote --source auto` is chart-first with scanner fallback only before
  chart mutation. Do not describe it as a blended source.
- `tv bars` is Desktop-free historical bars evidence. Bare symbols may resolve
  through symbol search; report `requested_symbol`, `resolved_symbol`, and
  `symbol_resolution`.
- `tv events` is scanner-backed event evidence for earnings and dividends.
  Use `events.v1` for one symbol and `events_compare.v1` for an ordered
  candidate set. It is not a complete event calendar and does not infer
  timezone, confirmed status, ranking, recommendation, or trading meaning.
- `tv observe chart`, `tv stream ...`, and `tv watch compare` are JSONL
  workflows. Read readiness / sample / heartbeat / summary event types before
  interpreting samples.

## Follow-Up Hints

`snapshot.v1` and `compare.v1` can return `follow_up_hints[]`. Treat these as
available evidence surfaces only. Stable kinds include `snapshot`,
`chart_quote`, `observe_chart`, and `screenshot`.

Before running a follow-up, read `requires_desktop`, `source_category`,
`non_mutating`, `evidence_role`, and `auto_execute`. `auto_execute: false`
means the CLI did not run that command and did not mix sources.

## Extended Hours And Quote-Data

Use scanner-backed quote, snapshot, compare, or quotes when scanner
extended-hours fields matter. Use `tv quote <SYMBOL> --source quote-data` only
when the task explicitly asks for Desktop-backed quote-data readback such as
`qsd.rtc`.

If quote-data is unavailable, report `source_availability` as source
diagnostics. Do not treat unavailable quote-data as proof that a symbol has no
price.

## Reporting Shape

1. Name the source and freshness metadata.
2. List the observed values that matter.
3. Explain only what follows from those values.
4. Name missing or uncertain fields.
5. Suggest the next read only if it materially changes confidence.
