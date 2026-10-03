use serde_json::{Value, json};

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};
use tradingview_model::visible_range::{
    VisibleRangeValidationFailure, validate_visible_range_bounds,
};

use super::super::chart::set_visible_range;
use super::super::common::{
    BARS_PATH, CHART_API, DEFAULT_OHLCV_COUNT, MAX_OHLCV_COUNT, desktop_backed_read_metadata,
    merge_object, round2,
};

const EXPORT_CHART_BARS_CONTRACT_VERSION: &str = "export_chart_bars.v1";
const EXPORT_CHART_BARS_SOURCE: &str = "selected_chart_cdp";
const DESKTOP_BACKED_OPERATION_CATEGORY: &str = "desktop_backed_operation";

pub async fn ohlcv_bars(
    runtime: &mut impl RuntimeEvaluator,
    count: Option<usize>,
) -> Result<Value, AppError> {
    let limit = normalized_count(count);
    let data = runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    function safeCall(fn) {{
                        try {{ return fn(); }} catch (e) {{ return null; }}
                    }}
                    function chartContext(chart) {{
                        if (!chart) return null;
                        var visibleRange = safeCall(function() {{ return chart.getVisibleRange(); }});
                        var barsRange = safeCall(function() {{ return chart.getVisibleBarsRange(); }});
                        return {{
                            symbol: safeCall(function() {{ return chart.symbol(); }}),
                            timeframe: safeCall(function() {{ return chart.resolution(); }}),
                            resolution: safeCall(function() {{ return chart.resolution(); }}),
                            visible_range: visibleRange,
                            bars_range: barsRange,
                            source: "direct_bars",
                            source_category: "desktop_backed_read",
                            requires_desktop: true,
                            non_mutating: true
                        }};
                    }}
                    function returnedBarsRange(result) {{
                        if (!result || result.length === 0) {{
                            return {{
                                first_time: null,
                                last_time: null,
                                bar_count: 0
                            }};
                        }}
                        return {{
                            first_time: result[0].time,
                            last_time: result[result.length - 1].time,
                            bar_count: result.length
                        }};
                    }}
                    function rangeMatch(visibleRange, returnedRange) {{
                        if (!visibleRange || !returnedRange) return "unknown";
                        var visibleFrom = Number(visibleRange.from);
                        var visibleTo = Number(visibleRange.to);
                        var firstTime = Number(returnedRange.first_time);
                        var lastTime = Number(returnedRange.last_time);
                        if (!isFinite(visibleFrom) || !isFinite(visibleTo) || !isFinite(firstTime) || !isFinite(lastTime)) return "unknown";
                        if (lastTime >= visibleFrom && firstTime <= visibleTo) return "overlaps_visible_range";
                        return "outside_visible_range";
                    }}
                    function readinessFailure(reason, chart, bars, extra) {{
                        var firstIndex = null;
                        var lastIndex = null;
                        var size = null;
                        var hasFirstIndex = !!(bars && typeof bars.firstIndex === 'function');
                        var hasLastIndex = !!(bars && typeof bars.lastIndex === 'function');
                        if (hasFirstIndex) firstIndex = safeCall(function() {{ return bars.firstIndex(); }});
                        if (hasLastIndex) lastIndex = safeCall(function() {{ return bars.lastIndex(); }});
                        if (bars && typeof bars.size === 'function') size = safeCall(function() {{ return bars.size(); }});
                        return Object.assign({{
                            _tv_ohlcv_error: true,
                            phase: "ohlcv_bars_read",
                            reason: reason,
                            source: "direct_bars",
                            source_category: "desktop_backed_read",
                            requires_desktop: true,
                            non_mutating: true,
                            chart_api_available: !!chart,
                            bars_available: !!bars,
                            chart_symbol: chart ? safeCall(function() {{ return chart.symbol(); }}) : null,
                            resolution: chart ? safeCall(function() {{ return chart.resolution(); }}) : null,
                            visible_range: chart ? safeCall(function() {{ return chart.getVisibleRange(); }}) : null,
                            bars_range: chart ? safeCall(function() {{ return chart.getVisibleBarsRange(); }}) : null,
                            bar_index_state: {{
                                has_first_index: hasFirstIndex,
                                has_last_index: hasLastIndex,
                                first_index: firstIndex,
                                last_index: lastIndex,
                                size: size,
                                result_count: 0
                            }}
                        }}, extra || {{}});
                    }}

                    var chart = null;
                    var bars = null;
                    try {{ chart = {CHART_API}; }} catch (e) {{
                        return readinessFailure("chart_api_unavailable", null, null, {{
                            chart_api_error: String(e && e.message ? e.message : e)
                        }});
                    }}
                    try {{ bars = {BARS_PATH}; }} catch (e) {{
                        return readinessFailure("bars_path_unavailable", chart, null, {{
                            bars_error: String(e && e.message ? e.message : e)
                        }});
                    }}
                    if (!bars || typeof bars.lastIndex !== 'function' || typeof bars.firstIndex !== 'function') {{
                        return readinessFailure("bars_index_api_unavailable", chart, bars, null);
                    }}
                    function numericCell(value) {{
                        return typeof value === 'number' && Number.isFinite(value) ? value : null;
                    }}
                    var result = [];
                    var first = safeCall(function() {{ return bars.firstIndex(); }});
                    var end = safeCall(function() {{ return bars.lastIndex(); }});
                    if (first === null || end === null || !isFinite(first) || !isFinite(end)) {{
                        return readinessFailure("bars_index_unreadable", chart, bars, null);
                    }}
                    var start = Math.max(first, end - {limit} + 1);
                    for (var i = start; i <= end; i++) {{
                        var v = bars.valueAt(i);
                        if (v == null) continue;
                        if (
                            typeof v[0] !== 'number' ||
                            !Number.isFinite(v[0])
                        ) {{
                            return readinessFailure("bars_row_invalid", chart, bars, null);
                        }}
                        result.push({{
                            time: v[0],
                            open: numericCell(v[1]),
                            high: numericCell(v[2]),
                            low: numericCell(v[3]),
                            close: numericCell(v[4]),
                            volume: numericCell(v[5])
                        }});
                    }}
                    if (result.length === 0) {{
                        return readinessFailure("bars_empty", chart, bars, {{
                            bar_index_state: {{
                                has_first_index: true,
                                has_last_index: true,
                                first_index: start,
                                last_index: end,
                                size: (typeof bars.size === 'function') ? safeCall(function() {{ return bars.size(); }}) : null,
                                result_count: 0
                            }}
                        }});
                    }}
                    var chartContextValue = chartContext(chart);
                    var returnedRange = returnedBarsRange(result);
                    return {{
                        symbol: chart.symbol(),
                        resolution: chart.resolution(),
                        timeframe: chart.resolution(),
                        bar_count: result.length,
                        total_available: (typeof bars.size === 'function') ? bars.size() : null,
                        source: "direct_bars",
                        source_category: "desktop_backed_read",
                        requires_desktop: true,
                        non_mutating: true,
                        chart_context: chartContextValue,
                        returned_bars_range: returnedRange,
                        selected_chart_range_match: rangeMatch(chartContextValue && chartContextValue.visible_range, returnedRange),
                        bars: result
                    }};
                }})()
                "#
            ),
            false,
        )
        .await?;
    if data
        .get("_tv_ohlcv_error")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Err(ohlcv_readiness_error(data));
    }
    Ok(data)
}

fn ohlcv_readiness_error(mut details: Value) -> AppError {
    if let Some(object) = details.as_object_mut() {
        object.remove("_tv_ohlcv_error");
        object.insert(
            "next_action_hint".to_string(),
            json!(
                "Run `tv tab list`, choose the active chart target's target_cli_args, then run `tv --target-id <ID> state` and `tv --target-id <ID> ohlcv --count 1`. Do not use TV_CDP_TARGET_ID."
            ),
        );
    }
    AppError::new(
        ErrorKind::InternalApiUnavailable,
        "Could not extract OHLCV data because chart bars are not available",
    )
    .with_details(details)
}

pub async fn ohlcv_summary(
    runtime: &mut impl RuntimeEvaluator,
    count: Option<usize>,
) -> Result<Value, AppError> {
    let data = ohlcv_bars(runtime, count).await?;
    summarize_ohlcv(data)
}

pub fn validate_export_chart_bars_request(
    from: f64,
    to: f64,
    count: Option<usize>,
) -> Result<(), AppError> {
    validate_visible_range_bounds(from, to).map_err(|failure| match failure {
        VisibleRangeValidationFailure::NonFinite { field } => AppError::new(
            ErrorKind::Validation,
            format!("{field} must be a finite number"),
        ),
        VisibleRangeValidationFailure::InvalidOrder { from, to } => AppError::new(
            ErrorKind::Validation,
            "export chart-bars requires --from to be less than --to",
        )
        .with_details(json!({
            "from": from,
            "to": to,
        })),
    })?;
    if let Some(count) = count
        && (count == 0 || count > MAX_OHLCV_COUNT)
    {
        return Err(AppError::new(
            ErrorKind::Validation,
            format!("export chart-bars --count must be between 1 and {MAX_OHLCV_COUNT}"),
        )
        .with_details(json!({
            "min": 1,
            "max": MAX_OHLCV_COUNT,
            "count": count,
        })));
    }
    Ok(())
}

pub async fn export_chart_bars(
    runtime: &mut impl RuntimeEvaluator,
    from: f64,
    to: f64,
    count: Option<usize>,
    summary: bool,
) -> Result<Value, AppError> {
    validate_export_chart_bars_request(from, to, count)?;
    let requested_visible_range = json!({
        "from": from,
        "to": to,
    });
    let range_operation = set_visible_range(runtime, from, to)
        .await
        .map_err(|err| export_chart_bars_error(err, requested_visible_range.clone(), "range"))?;
    let bars_data = ohlcv_bars(runtime, Some(count.unwrap_or(MAX_OHLCV_COUNT)))
        .await
        .map_err(|err| {
            export_chart_bars_error(err, requested_visible_range.clone(), "ohlcv_bars_read")
        })?;
    let mut payload = if summary {
        let mut summary_payload = summarize_ohlcv(bars_data)?;
        if let Some(object) = summary_payload.as_object_mut() {
            object.remove("last_5_bars");
        }
        summary_payload
    } else {
        bars_data
    };
    add_export_chart_bars_metadata(
        &mut payload,
        requested_visible_range,
        range_operation,
        summary,
    )?;
    Ok(payload)
}

fn normalized_count(count: Option<usize>) -> usize {
    count
        .unwrap_or(DEFAULT_OHLCV_COUNT)
        .clamp(1, MAX_OHLCV_COUNT)
}

fn add_export_chart_bars_metadata(
    payload: &mut Value,
    requested_visible_range: Value,
    range_operation: Value,
    summary: bool,
) -> Result<(), AppError> {
    let Some(object) = payload.as_object_mut() else {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Selected-chart export payload was not an object",
        ));
    };
    object.insert(
        "contract_version".to_string(),
        Value::String(EXPORT_CHART_BARS_CONTRACT_VERSION.to_string()),
    );
    object.insert(
        "operation".to_string(),
        Value::String("chart_bars_export".to_string()),
    );
    object.insert(
        "source".to_string(),
        Value::String(EXPORT_CHART_BARS_SOURCE.to_string()),
    );
    object.insert(
        "source_category".to_string(),
        Value::String(DESKTOP_BACKED_OPERATION_CATEGORY.to_string()),
    );
    object.insert("requires_desktop".to_string(), Value::Bool(true));
    object.insert("non_mutating".to_string(), Value::Bool(false));
    object.insert(
        "output_mode".to_string(),
        Value::String(if summary { "summary" } else { "bars" }.to_string()),
    );
    object.insert(
        "requested_visible_range".to_string(),
        requested_visible_range,
    );
    object.insert("range_operation".to_string(), range_operation);
    Ok(())
}

fn export_chart_bars_error(
    mut err: AppError,
    requested_visible_range: Value,
    phase: &str,
) -> AppError {
    let mut details = err.details.take().unwrap_or_else(|| json!({}));
    if !details.is_object() {
        details = json!({
            "upstream_details": details,
        });
    }
    if let Some(object) = details.as_object_mut() {
        object.insert(
            "contract_version".to_string(),
            Value::String(EXPORT_CHART_BARS_CONTRACT_VERSION.to_string()),
        );
        object.insert(
            "operation".to_string(),
            Value::String("chart_bars_export".to_string()),
        );
        object.insert("phase".to_string(), Value::String(phase.to_string()));
        object.insert(
            "requested_visible_range".to_string(),
            requested_visible_range,
        );
        object.insert(
            "source".to_string(),
            Value::String(EXPORT_CHART_BARS_SOURCE.to_string()),
        );
        object.insert(
            "source_category".to_string(),
            Value::String(DESKTOP_BACKED_OPERATION_CATEGORY.to_string()),
        );
        object.insert("requires_desktop".to_string(), Value::Bool(true));
        object.insert("non_mutating".to_string(), Value::Bool(false));
        object.insert(
            "next_action_hint".to_string(),
            Value::String(
                "Run `tv readiness`, `tv state`, and `tv range` to confirm the selected TradingView Desktop chart before retrying export chart-bars.".to_string(),
            ),
        );
    }
    err.details = Some(details);
    err
}

fn finite_number(value: f64) -> Option<f64> {
    value.is_finite().then_some(value)
}

fn numeric_field(bar: &Value, field: &str) -> Option<f64> {
    bar.get(field)
        .and_then(Value::as_f64)
        .and_then(finite_number)
}

fn summarize_ohlcv(data: Value) -> Result<Value, AppError> {
    let bars = data.get("bars").and_then(Value::as_array).ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "OHLCV data did not include bars",
        )
    })?;
    let first = bars.first().ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Could not extract OHLCV data. The chart may still be loading.",
        )
    })?;
    let last = bars.last().expect("non-empty bars should have last bar");
    if bars.iter().any(|bar| numeric_field(bar, "time").is_none()) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "OHLCV data included an invalid bar timestamp",
        ));
    }
    let open = numeric_field(first, "open");
    let close = numeric_field(last, "close");
    let high = bars.iter().try_fold(f64::NEG_INFINITY, |value, bar| {
        Some(value.max(numeric_field(bar, "high")?))
    });
    let low = bars.iter().try_fold(f64::INFINITY, |value, bar| {
        Some(value.min(numeric_field(bar, "low")?))
    });
    let volume = bars
        .iter()
        .try_fold(0.0, |value, bar| {
            Some(value + numeric_field(bar, "volume")?)
        })
        .and_then(finite_number);
    let avg_volume = volume.map(|value| (value / bars.len() as f64).round());
    let range = high
        .zip(low)
        .and_then(|(high, low)| finite_number(round2(high - low)));
    let change = open
        .zip(close)
        .and_then(|(open, close)| finite_number(round2(close - open)));
    let change_pct = open.zip(close).and_then(|(open, close)| {
        if open == 0.0 {
            return None;
        }
        finite_number(round2(((close - open) / open) * 100.0)).map(|value| format!("{value}%"))
    });
    let last_5_bars = bars
        .iter()
        .skip(bars.len().saturating_sub(5))
        .cloned()
        .collect::<Vec<_>>();
    let mut summary = json!({
        "bar_count": bars.len(),
        "period": {
            "from": first.get("time").cloned().unwrap_or(Value::Null),
            "to": last.get("time").cloned().unwrap_or(Value::Null),
        },
        "first_time": first.get("time").cloned().unwrap_or(Value::Null),
        "last_time": last.get("time").cloned().unwrap_or(Value::Null),
        "open": open,
        "close": close,
        "high": high,
        "low": low,
        "range": range,
        "change": change,
        "change_pct": change_pct,
        "avg_volume": avg_volume,
        "volume": volume,
        "last_5_bars": last_5_bars,
    });
    for field in [
        "symbol",
        "resolution",
        "timeframe",
        "chart_context",
        "returned_bars_range",
        "selected_chart_range_match",
    ] {
        if let Some(value) = data.get(field) {
            summary[field] = value.clone();
        }
    }
    merge_object(
        &mut summary,
        desktop_backed_read_metadata("direct_bars", true),
    );
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::ops::test_support::FakeRuntime;

    use super::*;

    #[tokio::test]
    async fn ohlcv_count_is_clamped_to_500() {
        let mut runtime = FakeRuntime::new([
            json!({"symbol": "NASDAQ:AAPL", "timeframe": "D", "bars": [{"time": 1, "open": 1, "high": 1, "low": 1, "close": 1, "volume": 1}]}),
        ]);
        let _ = ohlcv_bars(&mut runtime, Some(900)).await;
        assert!(runtime.evaluated[0].0.contains("end - 500 + 1"));
    }

    #[tokio::test]
    async fn ohlcv_missing_bars_returns_readiness_details() {
        let mut runtime = FakeRuntime::new([json!({
            "_tv_ohlcv_error": true,
            "phase": "ohlcv_bars_read",
            "reason": "bars_index_api_unavailable",
            "chart_api_available": true,
            "bars_available": false,
            "chart_symbol": "NASDAQ:IONQ",
            "resolution": "D",
            "bar_index_state": {"has_first_index": false, "has_last_index": false, "first_index": null, "last_index": null, "size": null, "result_count": 0}
        })]);
        let error = ohlcv_bars(&mut runtime, Some(5)).await.unwrap_err();
        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        let details = error.details.as_ref().unwrap();
        assert_eq!(details["phase"], "ohlcv_bars_read");
        assert_eq!(details["reason"], "bars_index_api_unavailable");
        assert_eq!(details["bar_index_state"]["result_count"], 0);
        assert!(
            details["next_action_hint"]
                .as_str()
                .unwrap()
                .contains("--target-id")
        );
    }

    #[tokio::test]
    async fn ohlcv_empty_bars_returns_readiness_details() {
        let mut runtime = FakeRuntime::new([json!({
            "_tv_ohlcv_error": true,
            "phase": "ohlcv_bars_read",
            "reason": "bars_empty",
            "chart_api_available": true,
            "bars_available": true,
            "chart_symbol": "NASDAQ:IONQ",
            "resolution": "D",
            "bar_index_state": {"has_first_index": true, "has_last_index": true, "first_index": 10, "last_index": 12, "size": 20, "result_count": 0}
        })]);
        let error = ohlcv_bars(&mut runtime, Some(5)).await.unwrap_err();
        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.details.as_ref().unwrap()["reason"], "bars_empty");
    }

    #[tokio::test]
    async fn ohlcv_success_preserves_practical_fields() {
        let payload = json!({
            "symbol": "NASDAQ:AAPL",
            "resolution": "D",
            "timeframe": "D",
            "bar_count": 2,
            "source": "direct_bars",
            "source_category": "desktop_backed_read",
            "requires_desktop": true,
            "non_mutating": true,
            "chart_context": {
                "symbol": "NASDAQ:AAPL",
                "timeframe": "D",
                "visible_range": {"from": 1, "to": 2},
                "bars_range": {"from": 1, "to": 2},
                "source_category": "desktop_backed_read"
            },
            "returned_bars_range": {"first_time": 1, "last_time": 2, "bar_count": 2},
            "selected_chart_range_match": "overlaps_visible_range",
            "bars": [
                {"time": 1, "open": 100.0, "high": 110.0, "low": 95.0, "close": 105.0, "volume": 10.0},
                {"time": 2, "open": 105.0, "high": 120.0, "low": 101.0, "close": 115.0, "volume": 20.0}
            ]
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);
        let result = ohlcv_bars(&mut runtime, Some(2)).await.unwrap();
        assert_eq!(result, payload);
        assert_eq!(result["source_category"], "desktop_backed_read");
        assert_eq!(result["requires_desktop"], true);
        assert_eq!(result["non_mutating"], true);
        assert_eq!(result["chart_context"]["symbol"], "NASDAQ:AAPL");
        assert_eq!(result["returned_bars_range"]["bar_count"], 2);
        assert_eq!(
            result["selected_chart_range_match"],
            "overlaps_visible_range"
        );
        assert!(runtime.evaluated[0].0.contains("bar_count"));
        assert!(runtime.evaluated[0].0.contains("chart_context"));
        assert!(
            runtime.evaluated[0]
                .0
                .contains("selected_chart_range_match")
        );
    }

    #[test]
    fn ohlcv_summary_returns_legacy_practical_fields() {
        let data = json!({
            "symbol": "NASDAQ:AAPL",
            "resolution": "D",
            "timeframe": "D",
            "chart_context": {
                "symbol": "NASDAQ:AAPL",
                "timeframe": "D",
                "visible_range": {"from": 1, "to": 2},
                "bars_range": {"from": 1, "to": 2},
                "source_category": "desktop_backed_read"
            },
            "returned_bars_range": {"first_time": 1, "last_time": 2, "bar_count": 2},
            "selected_chart_range_match": "overlaps_visible_range",
            "bars": [
                {"time": 1, "open": 100.0, "high": 110.0, "low": 95.0, "close": 105.0, "volume": 10.0},
                {"time": 2, "open": 105.0, "high": 120.0, "low": 101.0, "close": 115.0, "volume": 30.0}
            ]
        });
        let summary = summarize_ohlcv(data).unwrap();
        assert_eq!(summary["bar_count"], 2);
        assert_eq!(summary["symbol"], "NASDAQ:AAPL");
        assert_eq!(summary["open"], 100.0);
        assert_eq!(summary["close"], 115.0);
        assert_eq!(summary["high"], 120.0);
        assert_eq!(summary["low"], 95.0);
        assert_eq!(summary["volume"], 40.0);
        assert_eq!(summary["avg_volume"], 20.0);
        assert_eq!(summary["change"], 15.0);
        assert_eq!(summary["change_pct"], "15%");
        assert_eq!(summary["source"], "direct_bars");
        assert_eq!(summary["source_category"], "desktop_backed_read");
        assert_eq!(summary["requires_desktop"], true);
        assert_eq!(summary["non_mutating"], true);
        assert_eq!(summary["chart_context"]["symbol"], "NASDAQ:AAPL");
        assert_eq!(summary["returned_bars_range"]["bar_count"], 2);
        assert_eq!(
            summary["selected_chart_range_match"],
            "overlaps_visible_range"
        );
        assert!(summary["last_5_bars"].as_array().unwrap().len() == 2);
    }

    #[tokio::test]
    async fn export_chart_bars_adds_operation_contract_metadata() {
        let bars_payload = json!({
            "symbol": "NASDAQ:AAPL",
            "resolution": "D",
            "timeframe": "D",
            "bar_count": 2,
            "source": "direct_bars",
            "source_category": "desktop_backed_read",
            "requires_desktop": true,
            "non_mutating": true,
            "chart_context": {
                "symbol": "NASDAQ:AAPL",
                "timeframe": "D",
                "visible_range": {"from": 1.0, "to": 2.0},
                "bars_range": {"from": 1.0, "to": 2.0}
            },
            "returned_bars_range": {"first_time": 1.0, "last_time": 2.0, "bar_count": 2},
            "selected_chart_range_match": "overlaps_visible_range",
            "bars": [
                {"time": 1.0, "open": 100.0, "high": 110.0, "low": 95.0, "close": 105.0, "volume": 10.0},
                {"time": 2.0, "open": 105.0, "high": 120.0, "low": 101.0, "close": 115.0, "volume": 20.0}
            ]
        });
        let mut runtime = FakeRuntime::new([
            json!({
                "status": "ok", "earliest": 1.0, "latest": 2.0,
                "more_available": true, "request_method_available": true,
                "availability_method_available": true
            }),
            json!({
                "status": "ok", "visible_range": {"from": 1.0, "to": 2.0},
                "bars": [
                    {"index": 1, "timestamp": 1.0},
                    {"index": 2, "timestamp": 2.0}
                ]
            }),
            json!({"from": 1.0, "to": 2.0}),
            bars_payload,
        ]);

        let result = export_chart_bars(&mut runtime, 1.0, 2.0, Some(2), false)
            .await
            .unwrap();

        assert_eq!(result["contract_version"], "export_chart_bars.v1");
        assert_eq!(result["operation"], "chart_bars_export");
        assert_eq!(result["source"], "selected_chart_cdp");
        assert_eq!(result["source_category"], "desktop_backed_operation");
        assert_eq!(result["requires_desktop"], true);
        assert_eq!(result["non_mutating"], false);
        assert_eq!(result["output_mode"], "bars");
        assert_eq!(result["requested_visible_range"]["from"], 1.0);
        assert_eq!(
            result["range_operation"]["history_paging"]["stop_reason"],
            "paging_not_needed"
        );
        assert_eq!(
            result["range_operation"]["viewport_application"]["status"],
            "applied"
        );
        assert_eq!(result["chart_context"]["symbol"], "NASDAQ:AAPL");
        assert_eq!(result["returned_bars_range"]["bar_count"], 2);
        assert!(result["bars"].is_array());
        assert_eq!(runtime.evaluated.len(), 4);
    }

    #[tokio::test]
    async fn export_chart_bars_summary_omits_raw_bars() {
        let mut runtime = FakeRuntime::new([
            json!({
                "status": "ok", "earliest": 1.0, "latest": 2.0,
                "more_available": true, "request_method_available": true,
                "availability_method_available": true
            }),
            json!({
                "status": "ok", "visible_range": {"from": 1.0, "to": 2.0},
                "bars": [
                    {"index": 1, "timestamp": 1.0},
                    {"index": 2, "timestamp": 2.0}
                ]
            }),
            json!({"from": 1.0, "to": 2.0}),
            json!({
                "symbol": "NASDAQ:AAPL",
                "resolution": "D",
                "timeframe": "D",
                "chart_context": {"symbol": "NASDAQ:AAPL"},
                "returned_bars_range": {"first_time": 1.0, "last_time": 2.0, "bar_count": 2},
                "selected_chart_range_match": "overlaps_visible_range",
                "bars": [
                    {"time": 1.0, "open": 100.0, "high": 110.0, "low": 95.0, "close": 105.0, "volume": 10.0},
                    {"time": 2.0, "open": 105.0, "high": 120.0, "low": 101.0, "close": 115.0, "volume": null}
                ]
            }),
        ]);

        let result = export_chart_bars(&mut runtime, 1.0, 2.0, None, true)
            .await
            .unwrap();

        assert_eq!(result["contract_version"], "export_chart_bars.v1");
        assert_eq!(result["output_mode"], "summary");
        assert_eq!(result["bar_count"], 2);
        assert!(result["volume"].is_null());
        assert!(result["avg_volume"].is_null());
        assert_eq!(result["close"], 115.0);
        assert!(result.get("bars").is_none());
        assert!(result.get("last_5_bars").is_none());
        assert_eq!(result["chart_context"]["symbol"], "NASDAQ:AAPL");
    }

    #[test]
    fn export_chart_bars_validation_rejects_bad_ranges_and_counts() {
        assert!(validate_export_chart_bars_request(f64::NAN, 2.0, Some(1)).is_err());
        assert!(validate_export_chart_bars_request(2.0, 2.0, Some(1)).is_err());
        assert!(validate_export_chart_bars_request(1.0, 2.0, Some(0)).is_err());
        assert!(validate_export_chart_bars_request(1.0, 2.0, Some(501)).is_err());
        assert!(validate_export_chart_bars_request(1.0, 2.0, Some(500)).is_ok());
    }

    #[test]
    fn ohlcv_summary_preserves_missing_cells_and_real_zero() {
        let complete = json!({"bars": [
            {"time": 1, "open": 1, "high": 3, "low": 0, "close": 2, "volume": 10},
            {"time": 2, "open": 2, "high": 4, "low": 1, "close": 3, "volume": 0}
        ]});
        let summary = summarize_ohlcv(complete.clone()).unwrap();
        assert_eq!(summary["volume"], 10.0);
        assert_eq!(summary["avg_volume"], 5.0);
        for field in ["open", "close", "high", "low", "volume"] {
            let mut missing = complete.clone();
            let index = if field == "open" { 0 } else { 1 };
            missing["bars"][index][field] = Value::Null;
            let summary = summarize_ohlcv(missing).unwrap();
            assert!(summary[field].is_null(), "{field}");
            match field {
                "open" | "close" => {
                    assert!(summary["change"].is_null());
                    assert!(summary["change_pct"].is_null());
                }
                "high" | "low" => assert!(summary["range"].is_null()),
                "volume" => assert!(summary["avg_volume"].is_null()),
                _ => unreachable!(),
            }
        }
        let mut zero = complete;
        zero["bars"][0]["open"] = json!(0);
        let summary = summarize_ohlcv(zero).unwrap();
        assert_eq!(summary["open"], 0.0);
        assert_eq!(summary["change"], 3.0);
        assert!(summary["change_pct"].is_null());
    }

    #[test]
    fn ohlcv_summary_rejects_bad_timestamps_and_keeps_overflow_unknown() {
        for time in [Value::Null, json!("1")] {
            let data = json!({"bars": [{"time": time, "open": 1}]});
            assert!(summarize_ohlcv(data).is_err());
        }
        let summary = summarize_ohlcv(json!({"bars": [
            {"time": 1, "open": -1e308, "high": 1e308, "low": -1e308, "close": 1e308, "volume": 1e308},
            {"time": 2, "open": 1, "high": 1e308, "low": -1e308, "close": 1e308, "volume": 1e308}
        ]})).unwrap();
        for field in ["range", "change", "change_pct", "volume", "avg_volume"] {
            assert!(summary[field].is_null(), "{field}");
        }
        assert_eq!(summary["high"], 1e308);
    }

    #[test]
    fn summarize_ohlcv_rejects_missing_bars() {
        let error = summarize_ohlcv(json!({})).unwrap_err();
        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
    }
}
