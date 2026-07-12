use serde_json::Value;

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::AppError;

use super::super::common::{CHART_API, MAX_TRADES_COUNT};
use super::strategy_selection::{
    StrategyReadKind, attach_strategy_context, inspect_and_select_strategy, unavailable_payload,
};

pub async fn data_strategy(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    let selection = inspect_and_select_strategy(runtime).await?;
    if !selection.is_available() {
        return Ok(unavailable_payload(StrategyReadKind::Metrics, selection));
    }
    let entity_id = selection
        .entity_id
        .as_deref()
        .expect("available strategy has id");
    let payload = runtime
        .evaluate(&strategy_metrics_expression(entity_id), false)
        .await?;
    Ok(attach_strategy_context(payload, selection))
}

pub async fn data_trades(
    runtime: &mut impl RuntimeEvaluator,
    max_trades: Option<usize>,
) -> Result<Value, AppError> {
    let limit = max_trades
        .unwrap_or(MAX_TRADES_COUNT)
        .clamp(1, MAX_TRADES_COUNT);
    let selection = inspect_and_select_strategy(runtime).await?;
    if !selection.is_available() {
        return Ok(unavailable_payload(StrategyReadKind::Trades, selection));
    }
    let entity_id = selection
        .entity_id
        .as_deref()
        .expect("available strategy has id");
    let payload = runtime
        .evaluate(&strategy_trades_expression(entity_id, limit), false)
        .await?;
    Ok(attach_strategy_context(payload, selection))
}

pub async fn data_equity(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    let selection = inspect_and_select_strategy(runtime).await?;
    if !selection.is_available() {
        return Ok(unavailable_payload(StrategyReadKind::Equity, selection));
    }
    let entity_id = selection
        .entity_id
        .as_deref()
        .expect("available strategy has id");
    let payload = runtime
        .evaluate(&strategy_equity_expression(entity_id), false)
        .await?;
    Ok(attach_strategy_context(payload, selection))
}

fn strategy_metrics_expression(entity_id: &str) -> String {
    let entity_id = serde_json::to_string(entity_id).expect("strategy entity id serializes");
    format!(
        r#"
        (function() {{
            {STRATEGY_HELPERS}
            try {{
                var chart = {CHART_API}._chartWidget;
                var strat = __findStrategyByEntityId(chart.model().model().dataSources(), {entity_id});
                if (!strat) {{
                    return {{ metric_count: 0, source: "internal_api", metrics: {{}}, error: "Selected strategy is no longer available on chart." }};
                }}
                var metrics = {{}};
                if (strat._reportData && strat._reportData.performance) {{
                    __flattenScalars(metrics, strat._reportData.performance, "");
                }}
                if (Object.keys(metrics).length === 0 && strat.reportData) {{
                    var rd = __unwrapValue(typeof strat.reportData === "function" ? strat.reportData() : strat.reportData);
                    if (rd && typeof rd === "object") {{
                        if (rd.performance) __flattenScalars(metrics, rd.performance, "");
                        else __copyScalars(metrics, rd, "");
                    }}
                }}
                if (Object.keys(metrics).length === 0 && strat.performance) {{
                    var perf = __unwrapValue(strat.performance());
                    if (perf && typeof perf === "object") __copyScalars(metrics, perf, "");
                }}
                if (Object.keys(metrics).length > 0) {{
                    return {{ metric_count: Object.keys(metrics).length, source: "internal_api", metrics: metrics }};
                }}
                var dom = __strategyMetricsFromDom();
                return {{
                    metric_count: Object.keys(dom.metrics).length,
                    source: dom.source,
                    metrics: dom.metrics,
                    error: dom.error
                }};
            }} catch(e) {{
                return {{ metric_count: 0, source: "internal_api", metrics: {{}}, error: e.message }};
            }}
        }})()
        "#
    )
}

fn strategy_trades_expression(entity_id: &str, limit: usize) -> String {
    let entity_id = serde_json::to_string(entity_id).expect("strategy entity id serializes");
    format!(
        r#"
        (function() {{
            {STRATEGY_HELPERS}
            try {{
                var chart = {CHART_API}._chartWidget;
                var strat = __findStrategyByEntityId(chart.model().model().dataSources(), {entity_id});
                if (!strat) return {{ trade_count: 0, source: "internal_api", trades: [], error: "Selected strategy is no longer available on chart." }};
                if (strat._reportData && Array.isArray(strat._reportData.trades)) {{
                    var closedTrades = strat._reportData.trades;
                    var normalized = [];
                    for (var t = 0; t < Math.min(closedTrades.length, {limit}); t++) {{
                        var tr = closedTrades[t] || {{}};
                        var e = tr.e || {{}};
                        var x = tr.x || {{}};
                        normalized.push({{
                            entry_order_id: e.c || null,
                            entry_price: e.p ?? null,
                            entry_time_ms: e.tm ?? null,
                            entry_type: e.tp ?? null,
                            exit_order_id: x.c || null,
                            exit_price: x.p ?? null,
                            exit_time_ms: x.tm ?? null,
                            exit_type: x.tp ?? null,
                            quantity: tr.q ?? null,
                            pnl: tr.tp ? tr.tp.v : null,
                            pnl_pct: tr.tp ? tr.tp.p : null,
                            cum_pnl: tr.cp ? tr.cp.v : null,
                            cum_pnl_pct: tr.cp ? tr.cp.p : null,
                            runup: tr.rn ? tr.rn.v : null,
                            runup_pct: tr.rn ? tr.rn.p : null,
                            drawdown: tr.dd ? tr.dd.v : null,
                            drawdown_pct: tr.dd ? tr.dd.p : null
                        }});
                    }}
                    return {{
                        trade_count: normalized.length,
                        total_trade_count: closedTrades.length,
                        source: "internal_api",
                        trades: normalized
                    }};
                }}
                var orders = null;
                if (strat.ordersData) {{
                    orders = __unwrapValue(typeof strat.ordersData === "function" ? strat.ordersData() : strat.ordersData);
                }}
                if (!orders || !Array.isArray(orders)) {{
                    if (strat._orders) orders = strat._orders;
                    else if (strat.tradesData) {{
                        orders = __unwrapValue(typeof strat.tradesData === "function" ? strat.tradesData() : strat.tradesData);
                    }}
                }}
                if (orders && Array.isArray(orders)) {{
                    var result = [];
                    for (var i = 0; i < Math.min(orders.length, {limit}); i++) {{
                        var o = orders[i];
                        if (typeof o === "object" && o !== null) {{
                            var trade = {{}};
                            __copyScalars(trade, o, "");
                            result.push(trade);
                        }}
                    }}
                    return {{ trade_count: result.length, total_trade_count: orders.length, source: "internal_api", trades: result }};
                }}
                var dom = __strategyTradesFromDom({limit});
                return {{
                    trade_count: dom.trades.length,
                    source: dom.source,
                    trades: dom.trades,
                    error: dom.error,
                    note: dom.note
                }};
            }} catch(e) {{
                return {{ trade_count: 0, source: "internal_api", trades: [], error: e.message }};
            }}
        }})()
        "#
    )
}

fn strategy_equity_expression(entity_id: &str) -> String {
    let entity_id = serde_json::to_string(entity_id).expect("strategy entity id serializes");
    format!(
        r#"
        (function() {{
            {STRATEGY_HELPERS}
            try {{
                var chart = {CHART_API}._chartWidget;
                var strat = __findStrategyByEntityId(chart.model().model().dataSources(), {entity_id});
                if (!strat) return {{ data_points: 0, source: "internal_api", data: [], error: "Selected strategy is no longer available on chart." }};
                var data = [];
                if (strat._reportData && Array.isArray(strat._reportData.buyHold)) {{
                    var buyHold = strat._reportData.buyHold;
                    for (var bi = 0; bi < buyHold.length; bi++) {{
                        var point = buyHold[bi];
                        if (typeof point === "number") data.push({{ index: bi, value: point }});
                        else if (point && typeof point === "object") {{
                            var row = {{ index: bi }};
                            __copyScalars(row, point, "");
                            data.push(row);
                        }}
                    }}
                    if (data.length > 0) return {{ data_points: data.length, source: "internal_api", data: data }};
                }}
                if (strat.equityData) {{
                    var eq = __unwrapValue(typeof strat.equityData === "function" ? strat.equityData() : strat.equityData);
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
                    if (strat._reportData && strat._reportData.performance) {{
                        __flattenScalars(perfData, strat._reportData.performance, "");
                    }} else if (strat.performance) {{
                        var perf = __unwrapValue(strat.performance());
                        if (perf && typeof perf === "object") {{
                            var keys = Object.keys(perf);
                            for (var p = 0; p < keys.length; p++) {{
                                if (/equity|drawdown|profit|net/i.test(keys[p])) perfData[keys[p]] = perf[keys[p]];
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
    )
}

const STRATEGY_HELPERS: &str = r#"
function __unwrapValue(value) {
    if (value && typeof value.value === "function") return value.value();
    return value;
}
function __copyScalars(out, source, prefix) {
    if (!source || typeof source !== "object") return;
    var keys = Object.keys(source);
    for (var i = 0; i < keys.length; i++) {
        var key = keys[i];
        var val = source[key];
        if (val !== null && val !== undefined && typeof val !== "function" && typeof val !== "object") {
            out[prefix + key] = val;
        }
    }
}
function __flattenScalars(out, source, prefix) {
    if (!source || typeof source !== "object") return;
    var keys = Object.keys(source);
    for (var i = 0; i < keys.length; i++) {
        var key = keys[i];
        var val = source[key];
        var name = prefix ? prefix + "." + key : key;
        if (val !== null && val !== undefined && typeof val !== "function" && typeof val !== "object") {
            out[name] = val;
        } else if (val && typeof val === "object" && !Array.isArray(val)) {
            __flattenScalars(out, val, name);
        }
    }
}
function __findStrategyByEntityId(sources, entityId) {
    for (var i = 0; i < sources.length; i++) {
        var source = sources[i];
        try {
            var keys = ["id", "entityId"];
            for (var k = 0; k < keys.length; k++) {
                var id = source && source[keys[k]];
                id = typeof id === "function" ? id.call(source) : id;
                if (String(__unwrapValue(id) || "") === String(entityId)) return source;
            }
        } catch(e) {}
    }
    return null;
}
function __cleanText(text) {
    return String(text || "").replace(/[\u202a-\u202e\u2066-\u2069\u200e\u200f]/g, "").trim();
}
function __strategyMetricsFromDom() {
    try {
        var bodyText = document.body && document.body.innerText;
        if (!bodyText) return { source: "dom_fallback", metrics: {}, error: "Strategy Tester panel text not available." };
        var lines = bodyText.split("\n").map(__cleanText).filter(Boolean);
        var start = lines.indexOf("Strategy Report");
        if (start < 0) return { source: "dom_fallback", metrics: {}, error: "Strategy Tester panel not open." };
        var windowLines = lines.slice(start, start + 140);
        var metrics = {};
        function capture(label) {
            var idx = windowLines.indexOf(label);
            if (idx < 0 || idx + 1 >= windowLines.length) return;
            var value = windowLines[idx + 1];
            var unit = windowLines[idx + 2] || null;
            var pct = windowLines[idx + 3] || null;
            if (pct && pct.indexOf("%") >= 0) metrics[label] = { value: value, unit: unit, pct: pct };
            else metrics[label] = value;
        }
        ["Total P&L", "Max equity drawdown", "Total trades", "Profitable trades", "Profit factor", "Max contracts held"].forEach(capture);
        if (windowLines[1]) metrics.Strategy = windowLines[1];
        if (windowLines[2]) metrics["Date range"] = windowLines[2];
        return { source: "dom_fallback", metrics: metrics };
    } catch(e) {
        return { source: "dom_fallback", metrics: {}, error: e.message };
    }
}
function __strategyTradesFromDom(limit) {
    try {
        var rows = Array.from(document.querySelectorAll('[class*="listOfTrades"] [role="row"], [class*="strategyReport"] [role="row"], [class*="backtesting"] [role="row"]'));
        if (rows.length === 0) rows = Array.from(document.querySelectorAll('div[role="row"]'));
        if (rows.length === 0) {
            return { source: "dom_fallback", trades: [], error: "List of trades table not rendered. Open Strategy Tester and select the List of trades tab." };
        }
        var headerCells = Array.from(rows[0].querySelectorAll('[role="columnheader"], [role="cell"]'));
        var headers = headerCells.map(function(cell) { return __cleanText(cell.textContent); });
        var trades = [];
        for (var r = 1; r < rows.length && trades.length < limit; r++) {
            var cells = Array.from(rows[r].querySelectorAll('[role="cell"]')).map(function(cell) { return __cleanText(cell.textContent); });
            if (cells.length === 0) continue;
            var trade = {};
            for (var c = 0; c < cells.length; c++) trade[headers[c] || ("col_" + c)] = cells[c];
            trades.push(trade);
        }
        return {
            source: "dom_fallback",
            trades: trades,
            note: "DOM fallback returns only currently rendered Strategy Tester rows."
        };
    } catch(e) {
        return { source: "dom_fallback", trades: [], error: e.message };
    }
}
"#;

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::super::test_support::FakeRuntime;
    use super::*;

    fn ready_candidate() -> Value {
        json!({
            "candidates": [{
                "entity_id": "study-1",
                "detection_signals": ["tv_script_strategy"],
                "visible": true,
                "capabilities": {
                    "performance": true,
                    "trades": true,
                    "equity": true
                }
            }],
            "inspection_error": false
        })
    }

    #[tokio::test]
    async fn data_strategy_returns_runtime_payload() {
        let payload = json!({
            "metric_count": 1,
            "source": "internal_api",
            "metrics": {"netProfit": 120.5}
        });
        let mut runtime = FakeRuntime::new([ready_candidate(), payload.clone()]);

        let result = data_strategy(&mut runtime).await.unwrap();

        assert_eq!(result["metric_count"], payload["metric_count"]);
        assert_eq!(result["metrics"], payload["metrics"]);
        assert_eq!(result["strategy_context"]["selected_entity_id"], "study-1");
        assert!(runtime.evaluated[0].0.contains("isTVScriptStrategy"));
        assert!(runtime.evaluated[1].0.contains("_reportData.performance"));
        assert!(runtime.evaluated[1].0.contains("__strategyMetricsFromDom"));
        assert!(runtime.evaluated[1].0.contains("study-1"));
    }

    #[tokio::test]
    async fn data_trades_clamps_max_to_20() {
        let mut runtime = FakeRuntime::new([
            ready_candidate(),
            json!({"trade_count": 0, "source": "internal_api", "trades": []}),
        ]);

        let _ = data_trades(&mut runtime, Some(900)).await.unwrap();

        assert!(
            runtime.evaluated[1]
                .0
                .contains("Math.min(orders.length, 20)")
        );
        assert!(runtime.evaluated[1].0.contains("_reportData.trades"));
        assert!(runtime.evaluated[1].0.contains("total_trade_count"));
        assert!(
            runtime.evaluated[1]
                .0
                .contains("__strategyTradesFromDom(20)")
        );
    }

    #[tokio::test]
    async fn data_trades_preserves_total_trade_count_payload() {
        let payload = json!({
            "trade_count": 1,
            "total_trade_count": 12,
            "source": "internal_api",
            "trades": [{"entry_price": 100.0, "pnl": 12.5}]
        });
        let mut runtime = FakeRuntime::new([ready_candidate(), payload.clone()]);

        let result = data_trades(&mut runtime, Some(1)).await.unwrap();

        assert_eq!(result["trade_count"], payload["trade_count"]);
        assert_eq!(result["total_trade_count"], payload["total_trade_count"]);
        assert_eq!(result["trades"], payload["trades"]);
        assert_eq!(result["strategy_context"]["selected_entity_id"], "study-1");
    }

    #[tokio::test]
    async fn data_equity_returns_runtime_payload() {
        let payload = json!({
            "data_points": 1,
            "source": "internal_api",
            "data": [{"time": 1, "equity": 1000, "drawdown": null}]
        });
        let mut runtime = FakeRuntime::new([ready_candidate(), payload.clone()]);

        let result = data_equity(&mut runtime).await.unwrap();

        assert_eq!(result["data_points"], payload["data_points"]);
        assert_eq!(result["data"], payload["data"]);
        assert_eq!(result["strategy_context"]["selected_entity_id"], "study-1");
        assert!(runtime.evaluated[1].0.contains("equityData"));
        assert!(runtime.evaluated[1].0.contains("_reportData.buyHold"));
        assert!(runtime.evaluated[1].0.contains("_reportData.performance"));
    }

    #[tokio::test]
    async fn unavailable_strategy_stops_before_reader_and_keeps_legacy_error_text() {
        let mut runtime = FakeRuntime::new([json!({
            "candidates": [],
            "inspection_error": false
        })]);

        let result = data_trades(&mut runtime, Some(5)).await.unwrap();

        assert_eq!(runtime.evaluated.len(), 1);
        assert_eq!(result["trade_count"], 0);
        assert_eq!(result["error"], "No strategy found on chart.");
        assert_eq!(
            result["strategy_context"]["availability_status"],
            "not_found"
        );
    }

    #[tokio::test]
    async fn entity_id_only_descriptor_reaches_reader_fallback() {
        let inspection = json!({
            "candidates": [{
                "entity_id": "entity-only",
                "detection_signals": ["strategy_flag"],
                "visible": true,
                "capabilities": {
                    "performance": true,
                    "trades": false,
                    "equity": false
                }
            }],
            "inspection_error": false
        });
        let payload = json!({
            "metric_count": 1,
            "source": "internal_api",
            "metrics": {"netProfit": 1}
        });
        let mut runtime = FakeRuntime::new([inspection, payload]);

        let result = data_strategy(&mut runtime).await.unwrap();

        assert_eq!(
            result["strategy_context"]["selected_entity_id"],
            "entity-only"
        );
        assert!(runtime.evaluated[1].0.contains("entityId"));
        assert!(runtime.evaluated[1].0.contains("entity-only"));
    }

    #[tokio::test]
    async fn inspection_failure_keeps_nonterminal_metrics_payload() {
        let mut runtime = FakeRuntime::new([json!({
            "candidates": [],
            "inspection_error": true
        })]);

        let result = data_strategy(&mut runtime).await.unwrap();

        assert_eq!(runtime.evaluated.len(), 1);
        assert_eq!(result["metric_count"], 0);
        assert_eq!(result["source"], "internal_api");
        assert!(result["error"].as_str().is_some());
        assert_eq!(result["strategy_context"]["availability_status"], "unknown");
    }

    #[tokio::test]
    async fn malformed_inspection_keeps_nonterminal_equity_payload() {
        let mut runtime = FakeRuntime::new([json!({"unexpected": true})]);

        let result = data_equity(&mut runtime).await.unwrap();

        assert_eq!(runtime.evaluated.len(), 1);
        assert_eq!(result["data_points"], 0);
        assert!(result["error"].as_str().is_some());
        assert_eq!(result["strategy_context"]["availability_status"], "unknown");
    }
}
