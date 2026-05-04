use serde_json::{Value, json};

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};

use super::super::common::{
    BARS_PATH, CHART_API, DEFAULT_OHLCV_COUNT, MAX_OHLCV_COUNT, desktop_backed_read_metadata,
    merge_object, round2,
};

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
                    var result = [];
                    var first = safeCall(function() {{ return bars.firstIndex(); }});
                    var end = safeCall(function() {{ return bars.lastIndex(); }});
                    if (first === null || end === null || !isFinite(first) || !isFinite(end)) {{
                        return readinessFailure("bars_index_unreadable", chart, bars, null);
                    }}
                    var start = Math.max(first, end - {limit} + 1);
                    for (var i = start; i <= end; i++) {{
                        var v = bars.valueAt(i);
                        if (v) result.push({{time: v[0], open: v[1], high: v[2], low: v[3], close: v[4], volume: v[5] || 0}});
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
fn normalized_count(count: Option<usize>) -> usize {
    count
        .unwrap_or(DEFAULT_OHLCV_COUNT)
        .clamp(1, MAX_OHLCV_COUNT)
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
    let highs = bars
        .iter()
        .filter_map(|bar| bar.get("high").and_then(Value::as_f64))
        .collect::<Vec<_>>();
    let lows = bars
        .iter()
        .filter_map(|bar| bar.get("low").and_then(Value::as_f64))
        .collect::<Vec<_>>();
    let volumes = bars
        .iter()
        .filter_map(|bar| bar.get("volume").and_then(Value::as_f64))
        .collect::<Vec<_>>();
    let open = first.get("open").and_then(Value::as_f64).unwrap_or(0.0);
    let close = last.get("close").and_then(Value::as_f64).unwrap_or(0.0);
    let high = highs.iter().copied().reduce(f64::max).unwrap_or(0.0);
    let low = lows.iter().copied().reduce(f64::min).unwrap_or(0.0);
    let volume: f64 = volumes.iter().sum();
    let avg_volume = if volumes.is_empty() {
        0.0
    } else {
        (volume / volumes.len() as f64).round()
    };
    let change = round2(close - open);
    let change_pct = if open == 0.0 {
        "0%".to_string()
    } else {
        format!("{}%", round2(((close - open) / open) * 100.0))
    };
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
        "range": round2(high - low),
        "change": change,
        "change_pct": change_pct,
        "avg_volume": avg_volume,
        "volume": volume,
        "last_5_bars": last_5_bars,
    });
    for field in ["symbol", "resolution", "timeframe"] {
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
        assert!(runtime.evaluated[0].0.contains("bar_count"));
    }

    #[test]
    fn ohlcv_summary_returns_legacy_practical_fields() {
        let data = json!({
            "symbol": "NASDAQ:AAPL",
            "resolution": "D",
            "timeframe": "D",
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
        assert!(summary["last_5_bars"].as_array().unwrap().len() == 2);
    }

    #[test]
    fn summarize_ohlcv_rejects_missing_bars() {
        let error = summarize_ohlcv(json!({})).unwrap_err();
        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
    }
}
