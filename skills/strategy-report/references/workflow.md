# Strategy identity and availability

The three structured strategy commands share `strategy_context`. Confirm
`selected_entity_id`, `selection_reason`, visibility, and report availability
before combining their results. Multiple equally plausible strategies yield
`ambiguous`; resolving the intended report may require an explicitly requested
visibility change. A hidden strategy is not a zero-result strategy.

`panel_status: "unknown"` means the current Desktop build did not expose a
deterministic panel-state signal; it does not by itself make structured data
unavailable. Do not open the panel or change studies merely to remove that label.

Strategy Tester screenshots are visual evidence. After a requested panel/chart
change, add `--wait-for-render` if stable context is needed. Timeout does not
capture/overwrite an image and does not prove structured metrics are missing.
TradingView may expose metrics without a full equity curve; keep this gap in the
report instead of inferring a curve or claiming a complete backtest artifact.


## Reader limits

Use `tv spec data strategy`, `tv spec data trades`, or `tv spec data equity`
for offline command details when supported; older binaries retain textual help.
Check `data.error` as well as the envelope. Report availability means some report
capability exists, not that all three readers returned data. DOM fallback can
read formatted panel text or generic rendered rows; chart selection metadata
does not independently verify that those DOM rows belong to the same strategy.

Trades defaults to 20 and clamps `--max` to 1–20. It takes the first exposed
entries, without a newest-first or complete-history guarantee. Compare returned
and total counts where available; DOM rows cover only rendered content.

Do not label `data equity` as a strategy equity curve solely from its command
name or `source: internal_api`. The reader prefers Buy & Hold, then equity data
or strategy bars. When present, `series_source` identifies the observed branch
(`report_buy_hold`, `equity_data`, `strategy_bars`); `series_kind=buy_and_hold`
confirms the first branch, while the other two remain `unconfirmed`. Rows and
time fields vary by path. Current strategy-bars extraction preserves zero
drawdown and leaves missing values null. Older binaries lack these labels and
can turn zero drawdown into null; do not reconstruct a zero from that null.
Confirm meaning before calculating strategy returns. `performance_summary` or
`unavailable` with `series_kind=unavailable` provides no curve. An
`equity_summary` with zero data points remains a summary.
