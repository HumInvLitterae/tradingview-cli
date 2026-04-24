use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::{
    cdp::RuntimeEvaluator,
    error::{AppError, ErrorKind},
};

use super::common::{CHART_API, MAX_TRADES_COUNT, js_string, round2};

pub async fn study_values(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var chart = {CHART_API};
                    var sources = chart._chartWidget.model().model().dataSources();
                    var results = [];
                    for (var si = 0; si < sources.length; si++) {{
                        var s = sources[si];
                        if (!s.metaInfo) continue;
                        try {{
                            var meta = s.metaInfo();
                            var name = meta.description || meta.shortDescription || "";
                            if (!name) continue;
                            var values = {{}};
                            try {{
                                var dwv = s.dataWindowView();
                                if (dwv) {{
                                    var items = dwv.items();
                                    if (items) {{
                                        for (var i = 0; i < items.length; i++) {{
                                            var item = items[i];
                                            if (item._value && item._value !== "∅" && item._title) values[item._title] = item._value;
                                        }}
                                    }}
                                }}
                            }} catch(e) {{}}
                            if (Object.keys(values).length > 0) results.push({{ name: name, values: values }});
                        }} catch(e) {{}}
                    }}
                    return {{ study_count: results.length, studies: results }};
                }})()
                "#
            ),
            false,
        )
        .await
}
pub async fn data_indicator(
    runtime: &mut impl RuntimeEvaluator,
    entity_id: &str,
) -> Result<Value, AppError> {
    let entity_id_literal = js_string(entity_id)?;
    let data = runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var api = {CHART_API};
                    var entityId = {entity_id_literal};
                    var study = api.getStudyById(entityId);
                    if (!study) return {{ error: "Study not found: " + entityId }};
                    var result = {{ entity_id: entityId, visible: null, inputs: null }};
                    try {{ result.visible = study.isVisible(); }} catch(e) {{}}
                    try {{ result.inputs = study.getInputValues(); }} catch(e) {{ result.inputs_error = e.message; }}
                    if (Array.isArray(result.inputs)) {{
                        result.inputs = result.inputs.filter(function(input) {{
                            if (!input) return false;
                            if (input.id === "text" && typeof input.value === "string" && input.value.length > 200) return false;
                            if (typeof input.value === "string" && input.value.length > 500) return false;
                            return true;
                        }});
                    }}
                    return result;
                }})()
                "#
            ),
            false,
        )
        .await?;

    if let Some(message) = data.get("error").and_then(Value::as_str) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            message.to_string(),
        ));
    }

    Ok(data)
}

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

pub async fn data_lines(
    runtime: &mut impl RuntimeEvaluator,
    filter: Option<&str>,
    verbose: bool,
) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &build_graphics_expression("dwglines", "lines", filter.unwrap_or(""))?,
            false,
        )
        .await
        .map(|raw| summarize_pine_lines(raw, verbose))
}

pub async fn data_labels(
    runtime: &mut impl RuntimeEvaluator,
    filter: Option<&str>,
    max_labels: Option<usize>,
    verbose: bool,
) -> Result<Value, AppError> {
    let limit = max_labels.unwrap_or(50);
    runtime
        .evaluate(
            &build_graphics_expression("dwglabels", "labels", filter.unwrap_or(""))?,
            false,
        )
        .await
        .map(|raw| summarize_pine_labels(raw, limit, verbose))
}

pub async fn data_tables(
    runtime: &mut impl RuntimeEvaluator,
    filter: Option<&str>,
) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &build_graphics_expression("dwgtablecells", "tableCells", filter.unwrap_or(""))?,
            false,
        )
        .await
        .map(summarize_pine_tables)
}

pub async fn data_boxes(
    runtime: &mut impl RuntimeEvaluator,
    filter: Option<&str>,
    verbose: bool,
) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &build_graphics_expression("dwgboxes", "boxes", filter.unwrap_or(""))?,
            false,
        )
        .await
        .map(|raw| summarize_pine_boxes(raw, verbose))
}
fn build_graphics_expression(
    collection_name: &str,
    map_key: &str,
    filter: &str,
) -> Result<String, AppError> {
    let filter_literal = js_string(filter)?;
    Ok(format!(
        r#"
        (function() {{
            var chart = {CHART_API}._chartWidget;
            var model = chart.model();
            var sources = model.model().dataSources();
            var results = [];
            var filter = {filter_literal};
            for (var si = 0; si < sources.length; si++) {{
                var s = sources[si];
                if (!s.metaInfo) continue;
                try {{
                    var meta = s.metaInfo();
                    var name = meta.description || meta.shortDescription || "";
                    if (!name) continue;
                    if (filter && name.indexOf(filter) === -1) continue;
                    var g = s._graphics;
                    if (!g || !g._primitivesCollection) continue;
                    var pc = g._primitivesCollection;
                    var items = [];
                    try {{
                        var outer = pc.{collection_name};
                        if (outer) {{
                            var inner = outer.get("{map_key}");
                            if (inner) {{
                                var coll = inner.get(false);
                                if (coll && coll._primitivesDataById && coll._primitivesDataById.size > 0) {{
                                    coll._primitivesDataById.forEach(function(v, id) {{ items.push({{ id: id, raw: v }}); }});
                                }}
                            }}
                        }}
                    }} catch(e) {{}}
                    if (items.length === 0 && "{collection_name}" === "dwgtablecells") {{
                        try {{
                            var tcOuter = pc.dwgtablecells;
                            if (tcOuter) {{
                                var tcColl = tcOuter.get("tableCells");
                                if (tcColl && tcColl._primitivesDataById && tcColl._primitivesDataById.size > 0) {{
                                    tcColl._primitivesDataById.forEach(function(v, id) {{ items.push({{ id: id, raw: v }}); }});
                                }}
                            }}
                        }} catch(e) {{}}
                    }}
                    if (items.length > 0) results.push({{ name: name, count: items.length, items: items }});
                }} catch(e) {{}}
            }}
            return results;
        }})()
        "#
    ))
}

fn summarize_pine_lines(raw: Value, verbose: bool) -> Value {
    let empty = Vec::new();
    let studies = raw
        .as_array()
        .unwrap_or(&empty)
        .iter()
        .map(|study| {
            let mut horizontal_levels: Vec<f64> = Vec::new();
            let mut all_lines = Vec::new();
            for item in study
                .get("items")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let raw = item.get("raw").unwrap_or(&Value::Null);
                let y1 = raw.get("y1").and_then(Value::as_f64).map(round2);
                let y2 = raw.get("y2").and_then(Value::as_f64).map(round2);
                if let (Some(y1_value), Some(y2_value)) = (y1, y2)
                    && y1_value == y2_value
                    && !horizontal_levels.contains(&y1_value)
                {
                    horizontal_levels.push(y1_value);
                }
                if verbose {
                    all_lines.push(json!({
                        "id": item.get("id").cloned().unwrap_or(Value::Null),
                        "y1": y1,
                        "y2": y2,
                        "x1": raw.get("x1").cloned().unwrap_or(Value::Null),
                        "x2": raw.get("x2").cloned().unwrap_or(Value::Null),
                        "horizontal": y1.is_some() && y1 == y2,
                        "style": raw.get("st").cloned().unwrap_or(Value::Null),
                        "width": raw.get("w").cloned().unwrap_or(Value::Null),
                        "color": raw.get("ci").cloned().unwrap_or(Value::Null),
                    }));
                }
            }
            horizontal_levels.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
            let mut result = json!({
                "name": study.get("name").cloned().unwrap_or(Value::Null),
                "total_lines": study.get("count").cloned().unwrap_or(Value::Null),
                "horizontal_levels": horizontal_levels,
            });
            if verbose {
                result["all_lines"] = Value::Array(all_lines);
            }
            result
        })
        .collect::<Vec<_>>();
    json!({ "study_count": studies.len(), "studies": studies })
}

fn summarize_pine_labels(raw: Value, limit: usize, verbose: bool) -> Value {
    let empty = Vec::new();
    let studies = raw
        .as_array()
        .unwrap_or(&empty)
        .iter()
        .map(|study| {
            let mut labels = study
                .get("items")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|item| {
                    let raw = item.get("raw").unwrap_or(&Value::Null);
                    let text = raw.get("t").and_then(Value::as_str).unwrap_or("");
                    let price = raw.get("y").and_then(Value::as_f64).map(round2);
                    if text.is_empty() && price.is_none() {
                        return None;
                    }
                    if verbose {
                        Some(json!({
                            "id": item.get("id").cloned().unwrap_or(Value::Null),
                            "text": text,
                            "price": price,
                            "x": raw.get("x").cloned().unwrap_or(Value::Null),
                            "yloc": raw.get("yl").cloned().unwrap_or(Value::Null),
                            "size": raw.get("sz").cloned().unwrap_or(Value::Null),
                            "textColor": raw.get("tci").cloned().unwrap_or(Value::Null),
                            "color": raw.get("ci").cloned().unwrap_or(Value::Null),
                        }))
                    } else {
                        Some(json!({ "text": text, "price": price }))
                    }
                })
                .collect::<Vec<_>>();
            if labels.len() > limit {
                labels = labels.split_off(labels.len() - limit);
            }
            json!({
                "name": study.get("name").cloned().unwrap_or(Value::Null),
                "total_labels": study.get("count").cloned().unwrap_or(Value::Null),
                "showing": labels.len(),
                "labels": labels,
            })
        })
        .collect::<Vec<_>>();
    json!({ "study_count": studies.len(), "studies": studies })
}

fn summarize_pine_tables(raw: Value) -> Value {
    let empty = Vec::new();
    let studies = raw
        .as_array()
        .unwrap_or(&empty)
        .iter()
        .map(|study| {
            let mut tables: BTreeMap<i64, BTreeMap<i64, BTreeMap<i64, String>>> = BTreeMap::new();
            for item in study
                .get("items")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let raw = item.get("raw").unwrap_or(&Value::Null);
                let table_id = raw.get("tid").and_then(Value::as_i64).unwrap_or(0);
                let row = raw.get("row").and_then(Value::as_i64).unwrap_or(0);
                let col = raw.get("col").and_then(Value::as_i64).unwrap_or(0);
                let text = raw
                    .get("t")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                tables
                    .entry(table_id)
                    .or_default()
                    .entry(row)
                    .or_default()
                    .insert(col, text);
            }
            let table_list = tables
                .values()
                .map(|rows| {
                    let formatted = rows
                        .values()
                        .map(|cols| {
                            cols.values()
                                .filter(|value| !value.is_empty())
                                .cloned()
                                .collect::<Vec<_>>()
                                .join(" | ")
                        })
                        .filter(|row| !row.is_empty())
                        .collect::<Vec<_>>();
                    json!({ "rows": formatted })
                })
                .collect::<Vec<_>>();
            json!({
                "name": study.get("name").cloned().unwrap_or(Value::Null),
                "tables": table_list,
            })
        })
        .collect::<Vec<_>>();
    json!({ "study_count": studies.len(), "studies": studies })
}

fn summarize_pine_boxes(raw: Value, verbose: bool) -> Value {
    let empty = Vec::new();
    let studies = raw
        .as_array()
        .unwrap_or(&empty)
        .iter()
        .map(|study| {
            let mut zones: Vec<Value> = Vec::new();
            let mut seen = Vec::new();
            let mut all_boxes = Vec::new();
            for item in study
                .get("items")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let raw = item.get("raw").unwrap_or(&Value::Null);
                let y1 = raw.get("y1").and_then(Value::as_f64);
                let y2 = raw.get("y2").and_then(Value::as_f64);
                let high = y1.zip(y2).map(|(a, b)| round2(a.max(b)));
                let low = y1.zip(y2).map(|(a, b)| round2(a.min(b)));
                if let (Some(high), Some(low)) = (high, low) {
                    let key = format!("{high}:{low}");
                    if !seen.contains(&key) {
                        zones.push(json!({ "high": high, "low": low }));
                        seen.push(key);
                    }
                    if verbose {
                        all_boxes.push(json!({
                            "id": item.get("id").cloned().unwrap_or(Value::Null),
                            "high": high,
                            "low": low,
                            "x1": raw.get("x1").cloned().unwrap_or(Value::Null),
                            "x2": raw.get("x2").cloned().unwrap_or(Value::Null),
                            "borderColor": raw.get("c").cloned().unwrap_or(Value::Null),
                            "bgColor": raw.get("bc").cloned().unwrap_or(Value::Null),
                        }));
                    }
                }
            }
            zones.sort_by(|a, b| {
                b.get("high")
                    .and_then(Value::as_f64)
                    .partial_cmp(&a.get("high").and_then(Value::as_f64))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let mut result = json!({
                "name": study.get("name").cloned().unwrap_or(Value::Null),
                "total_boxes": study.get("count").cloned().unwrap_or(Value::Null),
                "zones": zones,
            });
            if verbose {
                result["all_boxes"] = Value::Array(all_boxes);
            }
            result
        })
        .collect::<Vec<_>>();
    json!({ "study_count": studies.len(), "studies": studies })
}

#[cfg(test)]
mod tests {
    use crate::error::ErrorKind;
    use serde_json::json;

    use super::super::test_support::FakeRuntime;
    use super::*;

    #[tokio::test]
    async fn study_values_returns_runtime_payload() {
        let payload = json!({
            "study_count": 1,
            "studies": [{"name": "Relative Strength", "values": {"RS": "98"}}]
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);

        let result = study_values(&mut runtime).await.unwrap();

        assert_eq!(result, payload);
        assert!(runtime.evaluated[0].0.contains("dataWindowView"));
    }

    #[tokio::test]
    async fn data_indicator_serializes_entity_id_and_filters_large_inputs() {
        let payload = json!({
            "entity_id": "eFu1Ot",
            "visible": true,
            "inputs": [{"id": "length", "value": 20}]
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);

        let result = data_indicator(&mut runtime, "eFu1Ot'); window.bad = true; ('")
            .await
            .unwrap();

        assert_eq!(result, payload);
        assert!(
            runtime.evaluated[0]
                .0
                .contains("\"eFu1Ot'); window.bad = true; ('\"")
        );
        assert!(runtime.evaluated[0].0.contains("getInputValues"));
    }

    #[tokio::test]
    async fn data_indicator_maps_missing_study_to_internal_api_error() {
        let mut runtime = FakeRuntime::new([json!({"error": "Study not found: missing"})]);

        let err = data_indicator(&mut runtime, "missing").await.unwrap_err();

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
    }

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

    #[tokio::test]
    async fn data_lines_summarizes_horizontal_levels() {
        let raw = json!([{
            "name": "Levels",
            "count": 2,
            "items": [
                {"id": "a", "raw": {"y1": 101.234, "y2": 101.234, "x1": 1, "x2": 2, "st": 0, "w": 1, "ci": "red"}},
                {"id": "b", "raw": {"y1": 99.1, "y2": 100.0}}
            ]
        }]);
        let mut runtime = FakeRuntime::new([raw]);

        let result = data_lines(&mut runtime, Some("Lev'); bad(); ('"), true)
            .await
            .unwrap();

        assert_eq!(result["study_count"], 1);
        assert_eq!(result["studies"][0]["horizontal_levels"], json!([101.23]));
        assert_eq!(
            result["studies"][0]["all_lines"].as_array().unwrap().len(),
            2
        );
        assert!(runtime.evaluated[0].0.contains("\"Lev'); bad(); ('\""));
    }

    #[tokio::test]
    async fn data_labels_respects_max_and_verbose() {
        let raw = json!([{
            "name": "Signals",
            "count": 3,
            "items": [
                {"id": "a", "raw": {"t": "A", "y": 1.0, "x": 1}},
                {"id": "b", "raw": {"t": "B", "y": 2.0, "x": 2}},
                {"id": "c", "raw": {"t": "C", "y": 3.0, "x": 3}}
            ]
        }]);
        let mut runtime = FakeRuntime::new([raw]);

        let result = data_labels(&mut runtime, None, Some(2), true)
            .await
            .unwrap();

        assert_eq!(result["studies"][0]["showing"], 2);
        assert_eq!(result["studies"][0]["labels"][0]["text"], "B");
        assert_eq!(result["studies"][0]["labels"][0]["id"], "b");
    }

    #[tokio::test]
    async fn data_tables_formats_rows() {
        let raw = json!([{
            "name": "Dashboard",
            "count": 3,
            "items": [
                {"id": "a", "raw": {"tid": 0, "row": 0, "col": 0, "t": "Trend"}},
                {"id": "b", "raw": {"tid": 0, "row": 0, "col": 1, "t": "Up"}},
                {"id": "c", "raw": {"tid": 0, "row": 1, "col": 0, "t": "Risk"}}
            ]
        }]);
        let mut runtime = FakeRuntime::new([raw]);

        let result = data_tables(&mut runtime, None).await.unwrap();

        assert_eq!(result["study_count"], 1);
        assert_eq!(result["studies"][0]["tables"][0]["rows"][0], "Trend | Up");
        assert_eq!(result["studies"][0]["tables"][0]["rows"][1], "Risk");
    }

    #[tokio::test]
    async fn data_boxes_summarizes_zones() {
        let raw = json!([{
            "name": "Zones",
            "count": 1,
            "items": [
                {"id": "a", "raw": {"y1": 90.0, "y2": 100.0, "x1": 1, "x2": 2, "c": "red", "bc": "pink"}}
            ]
        }]);
        let mut runtime = FakeRuntime::new([raw]);

        let result = data_boxes(&mut runtime, None, true).await.unwrap();

        assert_eq!(result["study_count"], 1);
        assert_eq!(
            result["studies"][0]["zones"],
            json!([{"high": 100.0, "low": 90.0}])
        );
        assert_eq!(result["studies"][0]["all_boxes"][0]["id"], "a");
    }
}
