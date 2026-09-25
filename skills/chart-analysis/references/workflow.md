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
historical input, use the existing bars command and inspect returned coverage:
`tv bars <EXCHANGE:SYMBOL> --from <YYYY-MM-DD> --to <YYYY-MM-DD>`.
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


## Indicator command details

On binaries with `spec`, use `tv spec indicator <action>` for chart-local IDs,
input constraints and readback. Get IDs from `tv state` on the selected target.
`indicator toggle` with no flags shows the study; it does not invert visibility.
Add requires an exact metainfo name and verifies scalar inputs. Set can update
matched keys while reporting unmatched keys, so read actual inputs afterward.
There is no dedicated metainfo search command, and failed insertion cleanup is
not a rollback guarantee. Older binaries retain textual help.


## Drawing command details

`tv spec draw <action>` describes coordinate pairs, position-price ordering and
conditional deletion. `draw clear` deletes all drawings unless `--dry-run` is
present. Resolve IDs through `draw list` on the same target. Position drawings
are not trades. Three-point parallel channels have stricter validation than
generic shapes; inspect geometry/readback after creation instead of assuming
that a returned ID proves every property. Use help on older binaries.


## Interpreting study values

`tv spec values` describes the read when supported; use help on older binaries.
Values are formatted data-window observations, not numeric time series with a
per-value timestamp or closed-bar guarantee. Rows without readable values can
be omitted, while hidden studies can still return values. Missing output is not
zero. Compact inputs are not a complete parameter export. Read `data indicator`
with a confirmed chart-local entity ID when more input detail is needed.


## Pine-generated lines, labels, tables and boxes

Use `tv spec data <lines|labels|tables|boxes>` for command details when supported;
older binaries retain help. These read Pine graphics, not hand-drawn objects.
`--filter` is a case-sensitive study-name substring, not an entity ID selector.
Rows lack study IDs, so same-name instances can remain ambiguous. Empty results
can reflect inaccessible primitives; hidden studies are not automatically excluded.

Line levels and box zones use two-decimal rounding and deduplication; a rounded
horizontal line or zone is not independently established support/resistance.
Verbose coordinates are internal values, not guaranteed timestamps or prices.
Label `--max` defaults to 500 per study and retains the last readable entries in
iteration order, not guaranteed chronological order. Inspect available/showing/
truncated separately; the limit applies after Desktop extraction.

Tables are lossy row strings joined with ` | `: empty cells, coordinates and table
IDs are omitted. Do not reconstruct a rectangular dataset by splitting strings.
Keep script meaning and the chart context separate from these display summaries.
