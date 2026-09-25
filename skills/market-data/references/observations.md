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
