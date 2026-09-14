# Resolve and reuse a Desktop session

Read this when starting Desktop work without a confirmed target, or when target
selection, connection, or chart state changes. Do not repeat these checks before
every command. Respect an already supplied target and the task's live-operation
permissions.

## Connection and target

For a chart workflow, use `tv readiness` when connection or chart readiness is
unclear. For a full-page Screener workflow, `tv tab list` exposes
`screener_targets`; chart ambiguity is not itself a Screener target choice.
If multiple plausible targets remain, resolve the intended one from the request
and current target metadata, asking only if the ambiguity remains material.
Reuse returned `target_cli_args`, such as `tv --target-id <ID> ...`.
`TV_CDP_TARGET_ID` is not supported.

If Desktop is disconnected and launching it is within the requested workflow,
run `tv launch` once. On macOS the normal launcher uses the system app launcher.
Use `--path <PATH>` for an intentionally selected executable. `--kill-existing`
requires explicit approval because it terminates an existing session.

If launch returns `cdp_ready: false`, check readiness because the app may still
be loading. A direct-spawn error can mean the child exited or could not be
verified. Report that result and resolve the path or manual-start need; do not
escalate to killing the app. The launcher already handles incompatible inherited
Electron mode; do not add a separate environment workaround without evidence.

## Effects and evidence

A request to change a named chart state authorizes that concrete effect; reuse
approval for the same target and scope. Chart inspection alone does not
authorize changing symbol, timeframe, studies, tabs, Replay, or saved state.
A symbol-qualified chart quote and chart compare can temporarily switch the
chart: inspect restoration. Screenshots write a local file; choose the path
within the requested artifact scope.

After changing symbol or timeframe, use chart readback such as
`tv ohlcv --count 1` before interpreting new values. Do not replace readiness
checks with fixed sleeps or dummy reads. Rediscover targets after selection
failure, a changed target set, or a user-requested chart change.

## Failures

Keep the original JSON error. CDP `failure_stage` can identify `target_list`,
`target_select`, `websocket_connect`, `method_call`, or `event_wait`; it does not
prove whether a mutation was dispatched. Do not repeat a mutation with an
unknown outcome without resolving that outcome and the remaining permission.

For failed OHLCV with working symbol/quote reads, inspect `error.details.phase`,
`bar_index_state`, `chart_readiness`, and `next_action_hint`. Check readiness or
target selection if those fields identify a relevant problem. Retry only when
the diagnosed issue and approved scope justify it; preserve the failed attempt.
An optional visual check can help when available, but packaged CLI workflows
must not depend on a specific agent's computer-control tool.
