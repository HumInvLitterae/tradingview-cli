use serde_json::Value;

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::AppError;

use super::common::{
    BARS_PATH, CHART_API, CHART_TYPES, js_string, parse_chart_type, require_finite,
};

pub async fn state(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var chart = {CHART_API};
                    var bars = null;
                    var visibleRange = null;
                    var studies = [];
                    try {{ visibleRange = chart.getVisibleRange(); }} catch(e) {{}}
                    try {{ bars = {BARS_PATH}; }} catch(e) {{}}
                    try {{
                        var allStudies = chart.getAllStudies();
                        studies = allStudies.map(function(s) {{
                            return {{ id: s.id, name: s.name || s.title || "unknown" }};
                        }});
                    }} catch(e) {{}}
                    var resolution = chart.resolution();
                    var chartType = chart.chartType();
                    var barsAvailable = !!(bars && typeof bars.lastIndex === 'function' && typeof bars.firstIndex === 'function');
                    var firstIndex = null;
                    var lastIndex = null;
                    try {{ if (barsAvailable) firstIndex = bars.firstIndex(); }} catch(e) {{}}
                    try {{ if (barsAvailable) lastIndex = bars.lastIndex(); }} catch(e) {{}}
                    return {{
                        source: "chart_api",
                        symbol: chart.symbol(),
                        resolution: resolution,
                        timeframe: resolution,
                        chartType: chartType,
                        chart_type: chartType,
                        studies: studies,
                        visible_range: visibleRange,
                        chart_readiness: {{
                            chart_api_available: true,
                            bars_available: barsAvailable,
                            chart_symbol: chart.symbol(),
                            resolution: resolution,
                            bar_index_state: {{
                                has_first_index: barsAvailable,
                                has_last_index: barsAvailable,
                                first_index: firstIndex,
                                last_index: lastIndex
                            }},
                            next_action_hint: barsAvailable
                                ? "Chart API and bars index methods are available. Use `tv ohlcv --count 1` to confirm readable bar values."
                                : "Chart API is available, but bars index methods are not ready. Run `tv tab list`, select the active chart target, then retry `tv --target-id <ID> state` and `tv --target-id <ID> ohlcv --count 1`."
                        }}
                    }};
                }})()
                "#
            ),
            false,
        )
        .await
}

pub async fn symbol_info(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var chart = {CHART_API};
                    var info = chart.symbolExt();
                    return {{
                        symbol: info.symbol,
                        full_name: info.full_name,
                        exchange: info.exchange,
                        description: info.description,
                        type: info.type,
                        pro_name: info.pro_name,
                        typespecs: info.typespecs,
                        resolution: chart.resolution(),
                        chart_type: chart.chartType()
                    }};
                }})()
                "#
            ),
            false,
        )
        .await
}

pub async fn set_symbol(
    runtime: &mut impl RuntimeEvaluator,
    symbol: &str,
) -> Result<Value, AppError> {
    let symbol_literal = js_string(symbol)?;
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var chart = {CHART_API};
                    return new Promise(function(resolve) {{
                        chart.setSymbol({symbol_literal}, {{}});
                        setTimeout(function() {{
                            var observed = chart.symbol();
                            resolve({{
                                symbol: observed,
                                chart_ready: String(observed).toUpperCase().indexOf(String({symbol_literal}).toUpperCase()) >= 0,
                                requested_symbol: {symbol_literal},
                                observed_symbol: observed
                            }});
                        }}, 500);
                    }});
                }})()
                "#
            ),
            true,
        )
        .await
}

pub async fn current_symbol(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var chart = {CHART_API};
                    return {{
                        symbol: chart.symbol(),
                        resolution: chart.resolution()
                    }};
                }})()
                "#
            ),
            false,
        )
        .await
}

pub async fn set_timeframe(
    runtime: &mut impl RuntimeEvaluator,
    timeframe: &str,
) -> Result<Value, AppError> {
    let timeframe_literal = js_string(timeframe)?;
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var chart = {CHART_API};
                    chart.setResolution({timeframe_literal}, {{}});
                    var observed = chart.resolution();
                    return {{
                        timeframe: observed,
                        chart_ready: String(observed) === String({timeframe_literal}),
                        requested_timeframe: {timeframe_literal},
                        observed_timeframe: observed
                    }};
                }})()
                "#
            ),
            false,
        )
        .await
}

pub async fn current_timeframe(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var chart = {CHART_API};
                    return {{
                        resolution: chart.resolution(),
                        timeframe: chart.resolution(),
                        symbol: chart.symbol()
                    }};
                }})()
                "#
            ),
            false,
        )
        .await
}

pub async fn current_chart_type(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var chart = {CHART_API};
                    var typeNames = {chart_type_names_json};
                    var typeNum = chart.chartType();
                    return {{
                        chart_type: typeNames[typeNum] || typeNum,
                        type_num: typeNum,
                        symbol: chart.symbol(),
                        resolution: chart.resolution()
                    }};
                }})()
                "#,
                chart_type_names_json = serde_json::to_string(&CHART_TYPES)
                    .expect("static chart type names should serialize")
            ),
            false,
        )
        .await
}

pub async fn set_chart_type(
    runtime: &mut impl RuntimeEvaluator,
    chart_type: &str,
) -> Result<Value, AppError> {
    let (type_num, canonical_name) = parse_chart_type(chart_type)?;
    let requested_chart_type = js_string(canonical_name)?;
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var chart = {CHART_API};
                    var typeNames = {chart_type_names_json};
                    var previousTypeNum = chart.chartType();
                    chart.setChartType({type_num});
                    var observedTypeNum = chart.chartType();
                    return {{
                        chart_type: typeNames[observedTypeNum] || observedTypeNum,
                        type_num: observedTypeNum,
                        requested_chart_type: {requested_chart_type},
                        requested_type_num: {type_num},
                        previous_chart_type: typeNames[previousTypeNum] || previousTypeNum,
                        previous_type_num: previousTypeNum,
                        observed_chart_type: typeNames[observedTypeNum] || observedTypeNum,
                        observed_type_num: observedTypeNum
                    }};
                }})()
                "#,
                chart_type_names_json = serde_json::to_string(&CHART_TYPES)
                    .expect("static chart type names should serialize")
            ),
            false,
        )
        .await
}

pub fn validate_chart_type(chart_type: &str) -> Result<(), AppError> {
    parse_chart_type(chart_type).map(|_| ())
}

pub async fn visible_range(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var chart = {CHART_API};
                    return {{
                        visible_range: chart.getVisibleRange(),
                        bars_range: chart.getVisibleBarsRange()
                    }};
                }})()
                "#
            ),
            false,
        )
        .await
}

pub async fn set_visible_range(
    runtime: &mut impl RuntimeEvaluator,
    from: f64,
    to: f64,
) -> Result<Value, AppError> {
    require_finite(from, "from")?;
    require_finite(to, "to")?;
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var chart = {CHART_API};
                    var m = chart._chartWidget.model();
                    var ts = m.timeScale();
                    var bars = m.mainSeries().bars();
                    var startIdx = bars.firstIndex();
                    var endIdx = bars.lastIndex();
                    var fromIdx = startIdx, toIdx = endIdx;
                    for (var i = startIdx; i <= endIdx; i++) {{
                        var v = bars.valueAt(i);
                        if (v && v[0] >= {from} && fromIdx === startIdx) fromIdx = i;
                        if (v && v[0] <= {to}) toIdx = i;
                    }}
                    ts.zoomToBarsRange(fromIdx, toIdx);
                    return new Promise(function(resolve) {{
                        setTimeout(function() {{
                            var actual = null;
                            try {{ actual = chart.getVisibleRange(); }} catch(e) {{}}
                            resolve({{
                                requested: {{ from: {from}, to: {to} }},
                                actual: actual || {{ from: 0, to: 0 }}
                            }});
                        }}, 500);
                    }});
                }})()
                "#
            ),
            true,
        )
        .await
}

pub async fn scroll_to_date(
    runtime: &mut impl RuntimeEvaluator,
    date: &str,
) -> Result<Value, AppError> {
    let date_literal = js_string(date)?;
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var date = {date_literal};
                    var timestamp;
                    if (/^\d+$/.test(date)) timestamp = Number(date);
                    else timestamp = Math.floor(new Date(date).getTime() / 1000);
                    if (!Number.isFinite(timestamp)) throw new Error("Could not parse date: " + date + ". Use ISO format (2024-01-15) or unix timestamp.");
                    var chart = {CHART_API};
                    var resolution = chart.resolution();
                    var secsPerBar = 60;
                    var res = String(resolution);
                    if (res === "D" || res === "1D") secsPerBar = 86400;
                    else if (res === "W" || res === "1W") secsPerBar = 604800;
                    else if (res === "M" || res === "1M") secsPerBar = 2592000;
                    else {{
                        var mins = parseInt(res, 10);
                        if (!Number.isNaN(mins)) secsPerBar = mins * 60;
                    }}
                    var halfWindow = 25 * secsPerBar;
                    var from = timestamp - halfWindow;
                    var to = timestamp + halfWindow;
                    var m = chart._chartWidget.model();
                    var ts = m.timeScale();
                    var bars = m.mainSeries().bars();
                    var startIdx = bars.firstIndex();
                    var endIdx = bars.lastIndex();
                    var fromIdx = startIdx, toIdx = endIdx;
                    for (var i = startIdx; i <= endIdx; i++) {{
                        var v = bars.valueAt(i);
                        if (v && v[0] >= from && fromIdx === startIdx) fromIdx = i;
                        if (v && v[0] <= to) toIdx = i;
                    }}
                    ts.zoomToBarsRange(fromIdx, toIdx);
                    return new Promise(function(resolve) {{
                        setTimeout(function() {{
                            resolve({{
                                date: date,
                                centered_on: timestamp,
                                resolution: resolution,
                                window: {{ from: from, to: to }}
                            }});
                        }}, 500);
                    }});
                }})()
                "#
            ),
            true,
        )
        .await
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tradingview_core::ErrorKind;

    use super::super::test_support::FakeRuntime;
    use super::*;

    #[tokio::test]
    async fn state_includes_chart_readiness_expression() {
        let payload = json!({
            "source": "chart_api",
            "symbol": "NASDAQ:AAPL",
            "resolution": "D",
            "timeframe": "D",
            "chartType": 1,
            "chart_type": 1,
            "studies": [],
            "visible_range": null,
            "chart_readiness": {
                "chart_api_available": true,
                "bars_available": true,
                "chart_symbol": "NASDAQ:AAPL",
                "resolution": "D"
            }
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);

        let result = state(&mut runtime).await.unwrap();

        assert_eq!(result, payload);
        assert!(runtime.evaluated[0].0.contains("chart_readiness"));
        assert!(runtime.evaluated[0].0.contains("barsAvailable"));
    }

    #[tokio::test]
    async fn set_symbol_serializes_user_input_as_js_string() {
        let mut runtime = FakeRuntime::new([json!({"observed_symbol": "AAPL"})]);

        let _ = set_symbol(&mut runtime, "AAPL'); window.bad = true; ('").await;

        let expression = &runtime.evaluated[0].0;
        assert!(expression.contains("\"AAPL'); window.bad = true; ('\""));
        assert!(runtime.evaluated[0].1);
    }

    #[tokio::test]
    async fn set_timeframe_serializes_user_input_as_js_string() {
        let mut runtime = FakeRuntime::new([json!({"observed_timeframe": "D"})]);

        let _ = set_timeframe(&mut runtime, "D").await;

        assert!(runtime.evaluated[0].0.contains("\"D\""));
    }

    #[tokio::test]
    async fn current_chart_type_returns_runtime_payload() {
        let payload = json!({
            "chart_type": "Candles",
            "type_num": 1,
            "symbol": "NASDAQ:AAPL",
            "resolution": "D"
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);

        let result = current_chart_type(&mut runtime).await.unwrap();

        assert_eq!(result, payload);
        assert!(runtime.evaluated[0].0.contains("chart.chartType()"));
        assert!(!runtime.evaluated[0].1);
    }

    #[tokio::test]
    async fn set_chart_type_accepts_name_and_returns_runtime_payload() {
        let payload = json!({
            "chart_type": "Line",
            "type_num": 2,
            "requested_chart_type": "Line",
            "requested_type_num": 2,
            "previous_chart_type": "Candles",
            "previous_type_num": 1,
            "observed_chart_type": "Line",
            "observed_type_num": 2
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);

        let result = set_chart_type(&mut runtime, "line").await.unwrap();

        assert_eq!(result, payload);
        assert!(runtime.evaluated[0].0.contains("chart.setChartType(2)"));
        assert!(runtime.evaluated[0].0.contains("\"Line\""));
    }

    #[tokio::test]
    async fn set_chart_type_accepts_number_and_separator_alias() {
        let mut runtime = FakeRuntime::new([json!({"type_num": 8})]);

        let _ = set_chart_type(&mut runtime, "heikin-ashi").await.unwrap();

        assert!(runtime.evaluated[0].0.contains("chart.setChartType(8)"));

        let mut runtime = FakeRuntime::new([json!({"type_num": 1})]);

        let _ = set_chart_type(&mut runtime, "1").await.unwrap();

        assert!(runtime.evaluated[0].0.contains("chart.setChartType(1)"));
    }

    #[tokio::test]
    async fn set_chart_type_rejects_unknown_type_before_evaluating() {
        let mut runtime = FakeRuntime::new([]);

        let err = set_chart_type(&mut runtime, "not-a-chart-type")
            .await
            .expect_err("unknown chart type should be rejected");

        assert_eq!(err.kind, ErrorKind::Validation);
        assert!(runtime.evaluated.is_empty());
    }

    #[tokio::test]
    async fn scroll_to_date_serializes_user_input_as_js_string() {
        let mut runtime = FakeRuntime::new([json!({"date": "2026-03-03"})]);

        let _ = scroll_to_date(&mut runtime, "2026-03-03'; window.bad = true; '").await;

        assert!(
            runtime.evaluated[0]
                .0
                .contains("\"2026-03-03'; window.bad = true; '\"")
        );
        assert!(runtime.evaluated[0].1);
    }

    #[tokio::test]
    async fn symbol_info_uses_symbol_ext() {
        let mut runtime = FakeRuntime::new([json!({"symbol": "AAPL"})]);

        let _ = symbol_info(&mut runtime).await;

        assert!(runtime.evaluated[0].0.contains("chart.symbolExt()"));
        assert!(!runtime.evaluated[0].1);
    }

    #[tokio::test]
    async fn set_visible_range_rejects_non_finite_values() {
        let mut runtime = FakeRuntime::new([]);

        let err = set_visible_range(&mut runtime, f64::NAN, 1.0)
            .await
            .expect_err("NaN should be rejected");

        assert_eq!(err.kind, ErrorKind::Validation);
    }
}
