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
`tv search <QUERY>`, `tv scanner scan`, `tv scanner hotlist`, and
`tv scanner metainfo`. Stable Desktop-free read payloads report
`source_category: "desktop_free_read"`,
`requires_desktop: false`, and `non_mutating: true`.

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
and heartbeats as a workflow-level JSONL observation. Screenshots are
non-mutating visual evidence reads, but they do write a local output file and
report `writes_file: true`.
Core Desktop-backed reads report `source_category: "desktop_backed_read"`,
`requires_desktop: true`, and `non_mutating` so agents can distinguish them
from scanner REST reads and account/page operations.

Recommended agent use: run `tv readiness` first when chart target, chart API,
or bars readiness is uncertain. Preserve structured readiness fields, then use
`tv screenshot --region chart|full --output <PATH>` only when structured fields
do not explain the visible state. For short monitoring windows, prefer bounded
`tv observe chart --duration-ms ... --heartbeat-ms ...` when readiness plus
last-bar observation is the workflow you need. Use lower-level bounded stream
controls such as `--duration-ms`, `--max-events`, and `--heartbeat-ms` when a
specific `tv stream ...` sample type is needed. Stream and observe JSONL events
identify chart samples with `source: "desktop_chart_stream"` and
`source_category: "desktop_backed_read"` so agents can distinguish them from
Desktop-free scanner reads or experimental browserless bars.

Do not treat `tv quote <SYMBOL> --source chart` as a multi-symbol realtime
batch source. It is a correctness-first single-symbol read that may switch and
restore the visible chart to prove the selected-chart feed for one requested
symbol. Use Desktop-free `tv compare`, `tv quotes`, `scanner scan`, or
`snapshot` for broad symbol comparison unless the selected Desktop chart feed
is specifically required. Concurrent or external chart mutations can still
invalidate chart-source assumptions; downstream workflows should preserve
structured freshness and restore fields rather than adding manual sleep or
double-call loops.

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
symbol mismatch, and matching qsd messages without `rtc`; they are not
ranking, scoring, or market-price absence signals. Success payloads also
include `quote_data.session_readback` with normalized spellings of
TradingView-provided session fields only. During regular session, quote-data
unavailable should not be read as a Desktop API-wide limitation. The current
success contract is centered on matching non-null `qsd.rtc`; if regular
`qsd` frames expose fields such as `lp` or `regular_close` without `rtc`,
that is field-semantics evidence for a later additive plan, not a reason to
guess a price.

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
`tv alert create`, `tv screener ...` mutation commands, `tv pine save`,
`tv draw position`, Replay controls, layout switching, and generic `tv ui`
automation.

Recommended agent use: explain the expected side effect, use `--dry-run` when
available, get user approval before normal mutation, and report whether
readback or post-check confirmed the requested after-state.

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

### Experimental

`requires_desktop`: command-specific. `may_mutate`: no unless explicitly
documented. `fallback_allowed`: no implicit fallback. `freshness_boundary`:
must be reported as experimental and not treated as stable market data.

Use this category for lab-gated commands that are intentionally not stable
surface yet. The current example is
`TV_EXPERIMENTAL_BARS=1 tv bars <EXCHANGE:SYMBOL>`, which uses an
undocumented TradingView WebSocket chart-session path and does not replace
chart-backed `tv ohlcv`.

Recommended agent use: use only when the user or workflow explicitly accepts
experimental data. Report `source`, `experimental`, `data_quality`, and
warnings. Do not build durable downstream assumptions on this category without
a later stabilization plan.

## Agent Guidance

Default to the narrowest source that answers the question:

- use Desktop-free reads for broad screening, metadata, fundamentals, and
  scanner-backed quotes;
- use Desktop-backed reads when the selected TradingView Desktop chart or
  visible state is the source of truth;
- use Desktop-backed operations only after the user accepts the side effect;
- use hybrid commands only when the fallback contract is useful to the task;
- use experimental commands only when lab data is explicitly acceptable.

When sources disagree, do not collapse them into a single value. Report the
source names and freshness fields, then decide whether another read changes the
answer.
