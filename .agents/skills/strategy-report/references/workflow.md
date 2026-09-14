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
