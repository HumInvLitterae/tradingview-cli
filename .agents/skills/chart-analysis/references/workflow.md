# Chart range, export, and visual evidence

## Range and export

No-argument `tv range` reads the selected viewport. Bounded `tv range --from
<UNIX_SECONDS> --to <UNIX_SECONDS>` may request older main-series history and
move the viewport. Read `history_paging.coverage_status`, `stop_reason`, and
`request_count` separately from `viewport_application.status`,
`matching_bar_count`, and `applied_range`. Endpoint coverage alone does not prove
that a weekend/session-gap interval has a bar or that the viewport moved.

`tv ohlcv` can return `chart_context`, `returned_bars_range`, and
`selected_chart_range_match`. Those diagnostics do not make it an independent
historical export. Use `tv export chart-bars` when this chart's range is the
intended source and moving its viewport is authorized. Report
`export_chart_bars.v1`, requested visible range, range operation, chart context,
returned bars range, and range-match status. For reproducible Desktop-free
historical input, use [historical bars](../../market-data/references/historical-bars.md).
Do not silently substitute one source for another.

## Screenshots and study identity

Use `tv screenshot --region chart` for the chart, `full` for the page, and
`strategy` for the detectable visible Strategy Tester panel, with an explicit
`--output <PATH>`. After changing chart/panel state, `--wait-for-render` opts
into bounded stable-context checks. Timeout captures nothing and leaves the
requested file untouched. `--wait-timeout-ms <500..30000>` requires the wait flag.

`tv values` preserves `name` and `values` and adds `entity_id`, `short_name`,
`study_kind`, compact `inputs`, and `visible`. Distinguish same-name instances
by identity and inputs, not ordering. Unknown identity fields remain unknown.
Arbitrary historical indicator-series computation is not implemented.

For an explicitly requested drawing operation, prefer high-level `tv draw`
commands. Native three-point `parallel_channel` needs paired `--price3` and
`--time3`, with the third time equal to the first point's time. Preserve the
verified returned entity ID for inspection/removal. Do not infer authority to
change studies or drawings from their appearance in a read result.
