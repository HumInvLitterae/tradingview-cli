# Command Source Taxonomy

This document defines how to describe where `tv` commands get data from and
whether they can change TradingView state.

For practical command sequences that combine these categories, see
`docs/observation-workflows.md`.

The project is not splitting the binary for now. `tv` remains the single user
command, and commands are classified by source and side-effect boundary instead
of by separate executable names. A future split such as `tv` plus `tvb` or
`tvd` may be reconsidered only after these boundaries prove stable in
downstream use.

## Categories

### Desktop-free read

`requires_desktop`: no. `may_mutate`: no. `fallback_allowed`: no Desktop
fallback unless a command explicitly says otherwise. `freshness_boundary`:
depends on the endpoint. Scanner price reads are useful screening reads but
are not realtime entitlement guarantees.

Use this category when the command can run without TradingView Desktop, CDP, or
visible chart state. Examples include `tv info <SYMBOL>`,
`tv fundamentals <SYMBOL>`, `tv quote <SYMBOL>` with the default scanner
source, `tv quotes <SYMBOL>...`, `tv compare <SYMBOL>...`,
`tv events <SYMBOL>`, `tv search <QUERY>`, `tv scanner scan`, `tv scanner hotlist`, and
`tv scanner metainfo`. Stable Desktop-free read payloads report
`source_category: "desktop_free_read"`,
`requires_desktop: false`, and `non_mutating: true`.

`tv scanner scan --max-results <N>` keeps this Desktop-free boundary while
reading sequential pages of at most 100 rows each. Its aggregate metadata
reports provider-total drift and duplicates; it does not claim an atomic
snapshot. A failed, malformed, premature-empty, or over-bound page returns an
error instead of a partial successful aggregate.

Recommended agent use: prefer these commands for broad discovery, one-off
symbol metadata, fundamentals, quote reads, and known-symbol comparison when
the exact selected Desktop chart feed is not required. Report source and
freshness metadata when the result is price-bearing. For `tv compare`, use
`summary` as an additive readback helper for resolution and section counts,
then inspect the ordered `items` evidence for the actual comparison.
`contract_version`, `requested_index`, per-item `follow_up_hints`, and
`summary.field_coverage` are also readback metadata; they do not change the
Desktop-free source boundary and do not imply ranking or recommendation.
`summary.coverage_status` is evidence coverage only: `complete`, `partial`, or
`blocked` describes whether requested items have usable sections and missing
fields, not which symbol is better. Per-item `missing_evidence` entries route
known gaps to stable follow-up kinds such as `snapshot` or `chart_quote`
without adding reads or changing the source category.
Per-item `movement` entries provide stable regular-session movement readback
for downstream tools. `movement.regular_change_percent` is derived from raw
scanner quote evidence at `sections.quote.data.change`; the raw quote section
remains the source of evidence. `movement.regular_change_abs` is not inferred
from price fields and remains null until a separate plan defines a source or
derivation policy.

`tv snapshot` uses the same Desktop-free boundary for one-symbol evidence.
Its contract metadata, coverage summary, missing-evidence readback, and
machine-readable follow-up hints describe the already collected quote, info,
and fundamentals sections. They do not trigger chart reads, screenshots,
retries, ranking, or recommendations, and the raw `sections` remain the source
of evidence.

Follow-up metadata uses a stable vocabulary across `compare` and `snapshot`:
`snapshot`, `chart_quote`, `observe_chart`, and `screenshot`. These values are
evidence-surface names only. They do not cross source boundaries automatically,
do not mutate the chart by themselves, and do not imply that `chart_quote`
contains scanner-style extended-hours values. `chart_quote` is the canonical
value; do not treat `quote_chart` as an alias.

Each `follow_up_hints[]` item is advisory metadata. Read `requires_desktop`,
`source_category`, `non_mutating`, `evidence_role`, and `auto_execute` before
running anything. `auto_execute: false` means the CLI will not run that
follow-up, switch charts, observe the chart, or take a screenshot on its own.
`next_action_hints[]` are human-facing wording for the same general direction;
they are not a machine contract and are not an instruction to blend sources.

`tv events <SYMBOL>` shapes scanner-backed fundamentals fields into
`contract_version: "events.v1"` for symbol-scoped earnings and dividend
readback. It reports `source: "scanner_fundamentals_rest"`,
`source_category: "desktop_free_read"`, requested / resolved symbol readback,
event type filters, event counts, and field availability. Treat it as
event-shaped scanner field evidence, not as a full event calendar. It must not
become a hidden fallback for fundamentals, quotes, compare, bars, or chart
reads, and it must not infer timezone, before/after-market, ranking,
recommendation, or trading judgment beyond the values TradingView returns.

`tv events compare <SYMBOL>...` uses the same scanner fundamentals source for
2 to 25 symbols and returns `contract_version: "events_compare.v1"` with
ordered item status, per-item `events.v1` payloads when available, public-safe
item errors, and summary counts. It is not a replacement for `tv compare`,
not a full event calendar, and not a ranking or recommendation surface.

`tv watch compare <SYMBOL>...` is also Desktop-free, but it is a bounded JSONL
workflow rather than a single JSON packet. It polls scanner-backed quote
evidence for a known candidate set and emits readiness, sample, heartbeat, and
summary events with `contract_version: "watch_compare.v1"`, `_watch:
"compare"`, `source: "scanner_scan_rest"`, and `source_category:
"desktop_free_read"`. Treat heartbeat and summary events as observation-window
readback, not market-data samples. The command does not connect to TradingView
Desktop, does not use selected-chart quote, does not read browserless bars,
and does not rank or recommend symbols.

`tv chart compare <SYMBOL>...` is the separated chart-backed compare surface.
It is not in this Desktop-free category. Keep `tv compare <SYMBOL>...` and
`tv watch compare <SYMBOL>...` scanner-backed. Use `tv chart compare` only
when the selected TradingView Desktop chart feed itself is the source under
review.

### Desktop-backed read

`requires_desktop`: yes. `may_mutate`: no intended account mutation, though
some reads depend on visible chart or page state. `fallback_allowed`: no hidden
Desktop-free fallback unless the command explicitly names it. `freshness_boundary`:
depends on target selection, chart readiness, and the visible TradingView app
state.

Use this category when the command reads the selected Desktop target or visible
state. Examples include `tv status`, `tv readiness`, `tv tab list`, `tv state`,
`tv ohlcv`, `tv quote` without a symbol, `tv quote <SYMBOL> --source chart`,
current-chart `tv info`, chart-model data reads, `tv screenshot`, and
`tv stream ...` JSONL observation commands. `tv observe chart` is also a
Desktop-backed read: it emits readiness first, then selected-chart bar samples
and heartbeats, then a final bounded-window summary as a workflow-level JSONL
observation. Screenshots are non-mutating visual evidence reads, but they do
write a local output file and report `writes_file: true`. Their optional
`--wait-for-render` phase is a bounded read of the same selected-chart context;
it does not change the screenshot source or mutate chart state.
Core Desktop-backed reads report `source_category: "desktop_backed_read"`,
`requires_desktop: true`, and `non_mutating` so agents can distinguish them
from scanner REST reads and account/page operations.

Recommended agent use: run `tv readiness` first when chart target, chart API,
or bars readiness is uncertain. Preserve structured readiness fields, then use
`tv screenshot --region chart|full --output <PATH>` only when structured fields
do not explain the visible state. Use
`tv screenshot --region strategy --output <PATH>` only when the visible
Strategy Tester panel itself is needed as visual evidence; use `tv data
strategy`, `tv data trades`, and `tv data equity` for structured strategy
fields. Read their additive `strategy_context` before interpreting results;
hidden, unready, missing, and ambiguous strategy states are diagnostics rather
than zero-performance evidence. The commands do not open Strategy Tester or
change study visibility. Add `--wait-for-render` after a chart or panel state
change when capture should require stable chart context; timeout writes no
image. For short monitoring windows, prefer bounded `tv observe chart
--duration-ms ... --heartbeat-ms ...` when readiness plus last-bar observation
is the workflow you need. Use lower-level bounded stream controls such as
`--duration-ms`, `--max-events`, and `--heartbeat-ms` when a specific
`tv stream ...` sample type is needed. Stream and observe JSONL events identify
chart samples with `source: "desktop_chart_stream"` and `source_category:
"desktop_backed_read"` so agents can distinguish them from Desktop-free
scanner reads or browserless historical bars.

`tv values` and `tv stream values` are selected-chart study reads. Their study
rows preserve the established `name` and `values` fields and add
`entity_id`, `short_name`, `study_kind`, compact `inputs`, and `visible`.
Use identity and compact inputs to distinguish same-name instances. Optional
metadata can be null, and `study_kind: "unknown"` is the conservative result
when the chart exposes no explicit kind marker. These fields do not authorize
automatic indicator mutation or name/order-based joins.

For `v0.18`, JSONL observation maturity is contract polish on these existing
selected-chart reads. `tv stream ...` sample, heartbeat, and summary events
carry `contract_version: "stream.v1"`. `tv observe chart` readiness, sample,
heartbeat, and summary events carry `contract_version: "observe_chart.v1"` and
`_observe: "chart"` while preserving underlying stream metadata such as
`_stream: "bars"` for selected-chart bar samples. The final summary event is
an observation-window readback with sample counts, heartbeat counts, elapsed
time, bounded controls, and end reason; it is not a market-data sample. This
does not change `_event`, source metadata, bounded controls, or event meaning.
Do not reinterpret it as realtime multi-symbol feed support, watch / JSONL
compare, browserless bars, scanner quote evidence, or quote-data readback.

Selected-chart historical export is now an explicit Desktop-backed operation:
`tv export chart-bars --from <UNIX_SECONDS> --to <UNIX_SECONDS>`. It moves the
visible range of the selected Desktop chart, reads selected-chart OHLCV bars,
and returns `contract_version: "export_chart_bars.v1"` with
`requested_visible_range`, `range_operation`, `chart_context`,
`returned_bars_range`, and `selected_chart_range_match`. It must not be a
hidden fallback for Desktop-free `tv bars --from/--to`. Use it only when the
selected Desktop chart itself is the intended source. `tv range` still reports
`operation: "visible_range"` so agents can distinguish viewport movement from
the combined selected-chart export workflow.

No-argument `tv range` remains `source_category: "desktop_backed_read"` and
`non_mutating: true`. Bounded `tv range --from/--to` is a
`desktop_backed_operation` with `non_mutating: false`: it may request older
main-series history and move the selected-chart viewport. Its additive
`history_paging` reports request count, endpoint coverage, stop reason,
exhaustion, limit, and timeout evidence. `viewport_application` separately
reports whether matching discrete bars were applied, clamped, absent because
the intervals did not overlap, or absent inside a market/session gap. Endpoint
coverage does not guarantee that a requested interval contains a bar.

Do not treat `tv quote <SYMBOL> --source chart` as a multi-symbol realtime
batch source. It is a correctness-first single-symbol read that may switch and
restore the visible chart to prove the selected-chart feed for one requested
symbol. Use Desktop-free `tv compare`, `tv quotes`, `scanner scan`, or
`snapshot` for broad symbol comparison unless the selected Desktop chart feed
is specifically required. Concurrent or external chart mutations can still
invalidate chart-source assumptions; downstream workflows should preserve
structured freshness and restore fields rather than adding manual sleep or
double-call loops.

For a small finalist set where selected-chart evidence is specifically needed,
use `tv chart compare <SYMBOL>...`. It serially reads chart-source quote
evidence for 2 to 10 symbols, reports `contract_version:
"chart_compare.v1"`, keeps ordered per-symbol status, and records before/after
chart context plus restore readback. It is a Desktop-backed operation because
it may temporarily switch the selected chart. It must not be used as a hidden
fallback for `tv compare`, `tv watch compare`, `tv bars`, Replay, chart
export, quote-data, or scanner reads.

Do not treat chart-source quote as scanner-style extended-hours evidence.
Chart-source quote reads the selected chart main-series last bar and reports a
`session_boundary` object with `price_session: "unknown"`,
`extended_hours_status: "not_provided"`, and
`extended_hours_guaranteed: false`. When premarket or postmarket fields matter,
use scanner-backed `tv quote`, `tv quotes`, `tv snapshot`, or `tv compare`
instead. The CLI does not inject scanner `extended_hours` values into
chart-source payloads.

TradingView Desktop also exposes a page quote session that can return
`premarket_*`, `postmarket_*`, and `market-status` field names. Treat that as a
separate Desktop quote-session evidence candidate, not as the selected chart
main-series quote. Regular-session probes showed those pre/post fields can
track current streaming values rather than scanner-backed premarket values, so
postmarket and premarket live evidence is required before exposing them as a
stable payload source. A postmarket probe observed `market-status.phase` as
`post-market`, but the selected quote-session pre/post close fields matched
each other and remained tied to quote-session streaming values in the
public-safe summary, so those fields still must not be treated as scanner-style
`extended_hours` values.

The visible TradingView Desktop right-side detail panel is another distinct
Desktop-backed visible UI source. A postmarket RKLB probe showed a visible
after-market price in that panel while scanner REST, chart main-series quote,
and the current quote-session selected field set reported different values.
The same source discovery narrowed that value to the right-side detail
widget's status/price nodes, with React metadata present on the matched node.
An additional bounded CDP Network/WebSocket smoke observed symbol-related
WebSocket traffic while the visible value was present, but did not identify a
captured communication candidate containing the visible after-hours price.
Scoped in-page widget inspection later found the right-panel detail widget
React chain and regular quote-like props, including current-session and
regular last-price fields, but did not expose the visible after-hours price
token in the compact prop/state hits. A later bounded WebSocket correlation
smoke sampled the visible after-market price during the same capture window and
found exact numeric matches in received WebSocket frame summaries. That
supports a push/WebSocket-backed source hypothesis. A follow-up HAR/live pass
narrowed the current best candidate to `qsd.rtc` in TradingView quote-data
WebSocket messages, with `rch` and `rchp` acting like regular-close-relative
change readbacks. This now has an explicit bounded read surface:
`tv quote <SYMBOL> --source quote-data`. The payload is source-labeled as
Desktop-backed WebSocket quote-data readback and remains separate from
`tv quote --source chart` and scanner REST `extended_hours`. It should not be
used as a multi-symbol realtime feed, and if no matching `qsd.rtc` arrives
during the bounded wait it returns structured unavailable details rather than
guessed data. In v0.14, quote-data payloads and structured unavailable
details carry command-local `contract_version: "quote_data.v1"` and
`source_availability` readback. Treat `source_availability.status:
"unavailable"` as "no matching quote-data source evidence arrived during the
bounded wait", not as "the symbol has no price". The same object includes
`unavailable_reason`, `timed_out`, and `next_action` for source diagnostics.
Those reasons distinguish missing WebSocket activity, missing qsd messages,
symbol mismatch, and matching qsd messages without a usable quote-data price
readback; they are not ranking, scoring, or market-price absence signals.
Success payloads also include `quote_data.session_readback` with normalized
spellings of TradingView-provided session fields only. `quote_data.price_readback`
labels which quote-data field produced the read: `kind: "rtc"` for `qsd.v.rtc`
and `kind: "regular_last"` for `qsd.v.lp`. `regular_close` is returned as
supporting source context when present, but it is not a standalone success
condition. During regular session, quote-data unavailable should not be read as
a Desktop API-wide limitation; it means no matching `rtc` or usable `lp`
arrived during the bounded wait.

When the issue is source availability rather than the quote value itself, use
`tv diagnose quote-data <SYMBOL>`. It is a Desktop-backed diagnostic packet,
not another price source. It reports sanitized target-selection status,
quote-data source availability, public-safe WebSocket/qsd counters, and a
separate scanner freshness reference. It does not merge scanner delayed REST,
chart main-series quote, or quote-data `rtc`, and it does not add quote-data
to `--source auto`.

### Desktop-backed operation

`requires_desktop`: yes. `may_mutate`: yes. `fallback_allowed`: only before a
mutation request is sent and only when the operation documents the fallback.
`freshness_boundary`: success requires a post-check against live TradingView
state.

Use this category when the command can change chart, account, editor, Replay,
Screener, layout, drawing, alert, watchlist, or generic UI state. Examples
include `tv symbol <SYMBOL>`, `tv watchlist add`, `tv watchlist remove`,
`tv alert create`, `tv export chart-bars`, `tv screener ...` mutation commands,
`tv pine open`, `tv pine save`, verified three-point `tv draw shape`, `tv draw position`, Replay controls, layout switching, and
generic `tv ui` automation.

Recommended agent use: explain the expected side effect, use `--dry-run` when
available, get user approval before normal mutation, and report whether
readback or post-check confirmed the requested after-state.

Replay has a split boundary. `tv replay status` is a Desktop-backed read and
should report `source_category: "desktop_backed_read"`,
`requires_desktop: true`, `non_mutating: true`, and `replay_context`.
`tv replay start`, `step`, `stop`, `autoplay`, and `trade` are
Desktop-backed operations and should report
`source_category: "desktop_backed_operation"`, `non_mutating: false`,
`operation`, and `replay_context`. Replay is selected-chart state and is not a
fallback for Desktop-free `tv bars --from/--to`.

`tv replay log --steps <N>` is bounded workflow evidence before any stable
Replay export command. It records start state, per-step Replay dates,
`replay_context`, selected-chart context, final end reason, and public-safe
failures as JSONL with `contract_version: "replay_step_log.v1"`.
`--attach-ohlcv-summary` is an explicit selected-chart OHLCV summary
attachment with `contract_version:
"replay_log_ohlcv_summary_attachment.v1"`, `source: "selected_chart_cdp"`,
and `source_category: "desktop_backed_read"`. Attachment failures are not
Replay step failures. Replay log must not attach `tv bars`,
`tv export chart-bars`, screenshots, scanner reads, or quote-data as hidden
fallbacks.

### Hybrid

`requires_desktop`: depends on selected source. `may_mutate`: can mutate chart
state when the Desktop-backed path switches symbols. `fallback_allowed`: only
as specified by the command. `freshness_boundary`: report the source actually
used.

Use this category when one command can choose between Desktop-free and
Desktop-backed paths. The current primary example is
`tv quote <SYMBOL> --source auto`, which is chart-first and falls back to
scanner only if the chart path is unavailable before any chart mutation.
After the chart path starts switching symbols, fallback is no longer allowed:
chart-source quote must prove that the quote symbol, current chart symbol, and
requested-symbol bars are stable, or return a structured readiness error.
When chart-source quote switches the visible chart, its payload reports
`non_mutating: false` together with `switch_performed`, `restored`, and
`freshness_check`.

Recommended agent use: use explicit `--source scanner` or `--source chart`
when source consistency matters. Use `--source auto` only when chart-first
behavior is desired and the scanner fallback is acceptable before mutation.

### Browserless Historical Bars

`requires_desktop`: false. `may_mutate`: no. `fallback_allowed`: no implicit
fallback. `freshness_boundary`: no realtime or entitlement guarantee; read
`data_quality`.

`tv bars <SYMBOL>` uses a bounded browserless TradingView WebSocket
chart-session path and reports `contract_version: "bars.v1"`,
`source: "tradingview_bars_ws"`, and
`source_category: "desktop_free_read"`. It is symbol-targeted historical
OHLCV evidence and does not replace chart-backed `tv ohlcv`, which reads the
selected Desktop chart through CDP.

Bare symbols such as `AAPL` are resolved through Desktop-free
`symbol_search_rest` before the bars read. The payload reports
`requested_symbol`, `resolved_symbol`, `symbol`, and `symbol_resolution` so
agents can see what the user typed and which `EXCHANGE:SYMBOL` was actually
used. Use an explicit exchange-qualified symbol, such as `NASDAQ:AAPL`, when
the exchange must be fixed.

Use count mode for recent bounded samples, for example
`tv bars AAPL --timeframe 1D --count 5`. Use date-range mode for
reproducible older intraday, daily, weekly, or monthly samples, for example
`tv bars NASDAQ:AAPL --timeframe 5 --from 2026-05-20 --to 2026-05-22`,
`tv bars NASDAQ:AAPL --timeframe 60 --from 2026-05-01 --to 2026-05-22`, or
`tv bars NASDAQ:CRUS --timeframe 1D --from 2010-01-01 --to 2010-12-31`.
The stable one-minute downstream form is
`tv bars EXCHANGE:SYMBOL --timeframe 1 --from YYYY-MM-DD --to YYYY-MM-DD
--count 5000`.
In date-range mode, `--count` is a safety cap on returned bars and defaults to
500. It can be raised up to 5000 in date-range mode; recent count mode stays
capped at 500. Date-range mode currently supports `1` (and its `1m` alias),
`5`, `15`, `30`, `60`, `1D`, `1W`, and `1M`; other intraday timeframes
remain guarded. The `--to`
date is an inclusive calendar date. `tv range` is only a selected Desktop chart viewport
operation; it does not make `tv ohlcv --count ...` a stable historical export
for that displayed period.

Recommended agent use: use when a workflow needs bounded historical bars for a
specific exchange-qualified symbol without requiring TradingView Desktop.
Report `source`, `contract_version`, `request_mode`, `summary`, `range`,
`requested_range`, `returned_range`, `range_coverage_status`,
`range_alignment`, `range_fetch_summary`, `source_availability`,
`data_quality`, and warnings. Read `range_coverage_status` as the primary
date-range coverage readback. For intraday, weekly, and monthly date ranges,
`range_alignment` reports period-start timestamp semantics and the
`timestamp_within_requested_range` filter policy. Read `range_fetch_summary`
for bounded fetch-window count, `request_more_data` count, observed / filtered
/ returned counts, returned-count cap truncation, and source / timeout
truncation reasons. Read
`summary.coverage_status`, `summary.requested_count_fulfilled`, and
`source_availability.wait_summary` before raw `bars[]`; they are historical
coverage and bounded-source diagnostics, not ranking, scoring, or trading
recommendations. Date-range completeness is determined by
`range_coverage_status` and `range_fetch_summary.range_truncated`; do not use
`data_quality.partial_result` alone because it also records returned-count
shortfall against `--count`. A larger corpus uses explicit non-overlapping
calendar windows and downstream period-start timestamp merging rather than a
larger CLI request. If `source_availability.status` is `unavailable`, read
`unavailable_reason` as a source diagnostic rather than proof that the symbol
has no price or no history. Do not treat `tv bars` as realtime streaming,
scanner quote, chart quote, or quote-data evidence.

Bars source failures may add `source_failure_stage` with the closed
bars-specific vocabulary `symbol_search`, `request_prepare`,
`websocket_connect`, `session_setup`, `series_setup`, `response_wait`,
`protocol`, `heartbeat_send`, `pagination`, `source_result`, or
`source_unknown`. Preserve the existing error and availability details. The
stage says where the failure surfaced, not whether a send reached TradingView
or whether retry is safe.

## Agent Guidance

Default to the narrowest source that answers the question:

- use Desktop-free reads for broad screening, metadata, fundamentals, and
  scanner-backed quotes;
- use Desktop-backed reads when the selected TradingView Desktop chart or
  visible state is the source of truth;
- use Desktop-backed operations only after the user accepts the side effect;
- use hybrid commands only when the fallback contract is useful to the task;
- use browserless historical bars only when bounded OHLCV evidence is useful
  and no realtime guarantee is required.

When sources disagree, do not collapse them into a single value. Report the
source names and freshness fields, then decide whether another read changes the
answer.
