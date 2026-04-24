use serde_json::Value;

use crate::{cdp::RuntimeEvaluator, error::AppError};

use super::super::common::{CHART_API, MAX_TRADES_COUNT};

pub async fn data_strategy(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    try {{
                        var chart = {CHART_API}._chartWidget;
                        var sources = chart.model().model().dataSources();
                        var strat = null;
                        for (var i = 0; i < sources.length; i++) {{
                            var s = sources[i];
                            if (s.metaInfo && s.metaInfo().is_price_study === false && (s.reportData || s.performance)) {{ strat = s; break; }}
                        }}
                        if (!strat) return {{ metric_count: 0, source: "internal_api", metrics: {{}}, error: "No strategy found on chart. Add a strategy indicator first." }};
                        var metrics = {{}};
                        if (strat.reportData) {{
                            var rd = typeof strat.reportData === "function" ? strat.reportData() : strat.reportData;
                            if (rd && typeof rd === "object") {{
                                if (typeof rd.value === "function") rd = rd.value();
                                if (rd) {{
                                    var keys = Object.keys(rd);
                                    for (var k = 0; k < keys.length; k++) {{
                                        var val = rd[keys[k]];
                                        if (val !== null && val !== undefined && typeof val !== "function") metrics[keys[k]] = val;
                                    }}
                                }}
                            }}
                        }}
                        if (Object.keys(metrics).length === 0 && strat.performance) {{
                            var perf = strat.performance();
                            if (perf && typeof perf.value === "function") perf = perf.value();
                            if (perf && typeof perf === "object") {{
                                var pkeys = Object.keys(perf);
                                for (var p = 0; p < pkeys.length; p++) {{
                                    var pval = perf[pkeys[p]];
                                    if (pval !== null && pval !== undefined && typeof pval !== "function") metrics[pkeys[p]] = pval;
                                }}
                            }}
                        }}
                        return {{ metric_count: Object.keys(metrics).length, source: "internal_api", metrics: metrics }};
                    }} catch(e) {{
                        return {{ metric_count: 0, source: "internal_api", metrics: {{}}, error: e.message }};
                    }}
                }})()
                "#
            ),
            false,
        )
        .await
}

pub async fn data_trades(
    runtime: &mut impl RuntimeEvaluator,
    max_trades: Option<usize>,
) -> Result<Value, AppError> {
    let limit = max_trades
        .unwrap_or(MAX_TRADES_COUNT)
        .clamp(1, MAX_TRADES_COUNT);
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    try {{
                        var chart = {CHART_API}._chartWidget;
                        var sources = chart.model().model().dataSources();
                        var strat = null;
                        for (var i = 0; i < sources.length; i++) {{
                            var s = sources[i];
                            if (s.metaInfo && s.metaInfo().is_price_study === false && (s.ordersData || s.reportData)) {{ strat = s; break; }}
                        }}
                        if (!strat) return {{ trade_count: 0, source: "internal_api", trades: [], error: "No strategy found on chart." }};
                        var orders = null;
                        if (strat.ordersData) {{
                            orders = typeof strat.ordersData === "function" ? strat.ordersData() : strat.ordersData;
                            if (orders && typeof orders.value === "function") orders = orders.value();
                        }}
                        if (!orders || !Array.isArray(orders)) {{
                            if (strat._orders) orders = strat._orders;
                            else if (strat.tradesData) {{
                                orders = typeof strat.tradesData === "function" ? strat.tradesData() : strat.tradesData;
                                if (orders && typeof orders.value === "function") orders = orders.value();
                            }}
                        }}
                        if (!orders || !Array.isArray(orders)) return {{ trade_count: 0, source: "internal_api", trades: [], error: "ordersData() returned non-array." }};
                        var result = [];
                        for (var t = 0; t < Math.min(orders.length, {limit}); t++) {{
                            var o = orders[t];
                            if (typeof o === "object" && o !== null) {{
                                var trade = {{}};
                                var okeys = Object.keys(o);
                                for (var k = 0; k < okeys.length; k++) {{
                                    var v = o[okeys[k]];
                                    if (v !== null && v !== undefined && typeof v !== "function" && typeof v !== "object") trade[okeys[k]] = v;
                                }}
                                result.push(trade);
                            }}
                        }}
                        return {{ trade_count: result.length, source: "internal_api", trades: result }};
                    }} catch(e) {{
                        return {{ trade_count: 0, source: "internal_api", trades: [], error: e.message }};
                    }}
                }})()
                "#
            ),
            false,
        )
        .await
}

pub async fn data_equity(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    try {{
                        var chart = {CHART_API}._chartWidget;
                        var sources = chart.model().model().dataSources();
                        var strat = null;
                        for (var i = 0; i < sources.length; i++) {{
                            var s = sources[i];
                            if (s.metaInfo && s.metaInfo().is_price_study === false && (s.reportData || s.performance)) {{ strat = s; break; }}
                        }}
                        if (!strat) return {{ data_points: 0, source: "internal_api", data: [], error: "No strategy found on chart." }};
                        var data = [];
                        if (strat.equityData) {{
                            var eq = typeof strat.equityData === "function" ? strat.equityData() : strat.equityData;
                            if (eq && typeof eq.value === "function") eq = eq.value();
                            if (Array.isArray(eq)) data = eq;
                        }}
                        if (data.length === 0 && strat.bars) {{
                            var bars = typeof strat.bars === "function" ? strat.bars() : strat.bars;
                            if (bars && typeof bars.lastIndex === "function") {{
                                var end = bars.lastIndex();
                                var start = bars.firstIndex();
                                for (var i = start; i <= end; i++) {{
                                    var v = bars.valueAt(i);
                                    if (v) data.push({{ time: v[0], equity: v[1], drawdown: v[2] || null }});
                                }}
                            }}
                        }}
                        if (data.length === 0) {{
                            var perfData = {{}};
                            if (strat.performance) {{
                                var perf = strat.performance();
                                if (perf && typeof perf.value === "function") perf = perf.value();
                                if (perf && typeof perf === "object") {{
                                    var pkeys = Object.keys(perf);
                                    for (var p = 0; p < pkeys.length; p++) {{
                                        if (/equity|drawdown|profit|net/i.test(pkeys[p])) perfData[pkeys[p]] = perf[pkeys[p]];
                                    }}
                                }}
                            }}
                            if (Object.keys(perfData).length > 0) {{
                                return {{
                                    data_points: 0,
                                    source: "internal_api",
                                    data: [],
                                    equity_summary: perfData,
                                    note: "Full equity curve not available via API; equity summary metrics returned instead."
                                }};
                            }}
                        }}
                        return {{ data_points: data.length, source: "internal_api", data: data }};
                    }} catch(e) {{
                        return {{ data_points: 0, source: "internal_api", data: [], error: e.message }};
                    }}
                }})()
                "#
            ),
            false,
        )
        .await
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::super::test_support::FakeRuntime;
    use super::*;

    #[tokio::test]
    async fn data_strategy_returns_runtime_payload() {
        let payload = json!({
            "metric_count": 1,
            "source": "internal_api",
            "metrics": {"netProfit": 120.5}
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);

        let result = data_strategy(&mut runtime).await.unwrap();

        assert_eq!(result, payload);
        assert!(runtime.evaluated[0].0.contains("reportData"));
        assert!(runtime.evaluated[0].0.contains("performance"));
    }

    #[tokio::test]
    async fn data_trades_clamps_max_to_20() {
        let mut runtime =
            FakeRuntime::new([json!({"trade_count": 0, "source": "internal_api", "trades": []})]);

        let _ = data_trades(&mut runtime, Some(900)).await.unwrap();

        assert!(
            runtime.evaluated[0]
                .0
                .contains("Math.min(orders.length, 20)")
        );
    }

    #[tokio::test]
    async fn data_equity_returns_runtime_payload() {
        let payload = json!({
            "data_points": 1,
            "source": "internal_api",
            "data": [{"time": 1, "equity": 1000, "drawdown": null}]
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);

        let result = data_equity(&mut runtime).await.unwrap();

        assert_eq!(result, payload);
        assert!(runtime.evaluated[0].0.contains("equityData"));
    }
}
