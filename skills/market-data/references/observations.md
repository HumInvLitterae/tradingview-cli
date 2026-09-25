# Bounded observations

| Need | Command | Contract/source |
| --- | --- | --- |
| Poll known symbols through scanner REST | `tv watch compare <SYMBOL>... --duration-ms <MS> --interval <MS>` | `watch_compare.v1`, `scanner_scan_rest`, Desktop-free |
| Observe the selected chart's last bar | `tv observe chart --duration-ms <MS> --heartbeat-ms <MS>` | `observe_chart.v1`, Desktop-backed |
| A specific lower-level chart sample type | `tv stream ... --duration-ms <MS> --max-events <N>` | `stream.v1`, Desktop-backed |

Choose explicit duration/event bounds from the request. Inspect event type,
contract, source, chart/symbol context, counts, and end reason. Readiness is
preflight context, a sample is data, a heartbeat is liveness, and a final summary
reports the observation window. Heartbeats and summaries are not market updates.
Do not describe these bounded commands as a daemon or guaranteed realtime feed.

For Desktop commands, first resolve the
[Desktop session](desktop-session.md).
`tv values` and `tv stream values` identify studies by `entity_id` and compact
inputs, not row order; optional identity fields can be null and `study_kind`
can be `unknown`. These fields do not authorize changing a study.


For `observe chart`, use `tv spec observe chart` for offline details when available
and help otherwise. Max-events counts emitted distinct samples, not polls,
heartbeats or errors. Unchanged data can prevent that count from being reached;
include duration-ms when the run must end within a bounded observation window.
Setup time and in-flight reads are outside that window's strict deadline checks.

Monitor stderr error envelopes as well as stdout JSONL: sample failures continue
the loop, and heartbeats or a successful final summary do not prove error-free
collection. Summary has no error count. Broken output pipes or interruption can
omit it. Inspect each sample's symbol/resolution because external chart changes
are not prevented. `_ts` is client milliseconds; `bar_time` is the chart timestamp,
not a closed-bar assertion. The current reader can default missing volume to zero.


For `stream values`, `stream quote` and `stream bars`, use `tv spec stream <kind>`
when available and help otherwise. These streams share the deduplicated count
and error/termination limits above, but emit no initial readiness event.

`stream values` reads numeric internal last-bar properties rather than the
formatted strings returned by `values`. Explicitly hidden studies are skipped
when visibility is readable; missing data and individual study failures can
omit rows. Unknown visibility stays unknown. Samples have no explicit timeframe
or guaranteed per-study timestamp. Verify chart context before collecting.

`stream quote` and `stream bars` read the chart's current last-bar OHLCV, including
an unfinished bar. They are not scanner or official-MCP price feeds. Quote uses
`time` and omits resolution/bar_index; bars uses `bar_time` and includes both.
Quote does not supply scanner-style extended-hours fields. Polling can miss
intervening updates; neither command exports historical bars.


## Pine graphics and layout streams

Use `stream lines`, `stream labels` or `stream tables` for repeated Pine graphics
reads. Their optional `--filter` is a trimmed, case-insensitive substring of the
chart study name, unlike the single-read `data` commands. Rows have study names
but no entity IDs, so same-name instances remain ambiguous. Hidden studies are
not excluded. Unreadable primitives and individual study failures can omit rows;
`study_count` is the returned row count. Samples lack resolution and primitive
timestamps. Verify chart and script coordinate context before interpretation.

- Lines returns unique raw endpoint levels sorted descending. Sloped lines also
  contribute an endpoint; these are not verified horizontal levels. Truthy
  endpoint fallback can replace zero, and coordinates/IDs/styles are omitted.
- Labels keeps nonempty text and the first 50 entries per study in internal
  iteration order, without truncation counts or configurable limits. A zero y
  fallback can become null. Label IDs and x coordinates are absent.
- Tables returns nested text rows rather than the pipe-joined `data tables`
  summary. Styling, primitive IDs and proof of complete extraction are absent.

`stream all` reads last-bar OHLCV from chart panes in the current layout; it does
not combine the other stream kinds. Inspect each pane's `error` even when the
sample succeeds. `pane_count` includes failed panes and indexes describe current
widget order, not stable identities. Successful rows have symbol, resolution and
`time`; missing volume can default to zero. Panes are read sequentially, without
an atomic cross-symbol snapshot guarantee or a saved-layout/tab switch.

Defaults are 1000 ms for lines/labels, 2000 ms for tables and 500 ms for all.
Use `tv spec stream <kind>` when available and help otherwise. The same JSONL
error channels, deduplication and duration/count limits described above apply.
