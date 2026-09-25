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
name or `source: internal_api`. The current reader prefers Buy & Hold data when
available, then tries equity data or strategy bars; output lacks a reliable series
provenance discriminator. Rows and time fields vary by path, and the bars path
converts zero drawdown to null. Confirm series meaning independently before
calculating strategy returns. `equity_summary` with zero data points is a summary,
not a curve. Keep unknown series identity explicit.
