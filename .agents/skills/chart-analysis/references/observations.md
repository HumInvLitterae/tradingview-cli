# Bounded chart observations

| Need | Command | Contract/source |
| --- | --- | --- |
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
