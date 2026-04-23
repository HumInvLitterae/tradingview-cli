use std::{fs, path::Path};

use serde_json::{Value, json};

use crate::{
    cdp::RuntimeEvaluator,
    error::{AppError, ErrorKind},
    transport::{self, TargetSelection, TransportConfig},
};

const CHART_API: &str = "window.TradingViewApi._activeChartWidgetWV.value()";
const BARS_PATH: &str =
    "window.TradingViewApi._activeChartWidgetWV.value()._chartWidget.model().mainSeries().bars()";

pub async fn status(config: &TransportConfig) -> Result<Value, AppError> {
    let targets = transport::fetch_targets(config).await?;
    let data = match transport::select_target(&targets) {
        TargetSelection::Selected(target) => json!({
            "connected": true,
            "target_id": target.id,
            "target_url": target.url,
            "target_title": target.title,
            "cdp_host": config.host,
            "cdp_port": config.port,
        }),
        TargetSelection::None => json!({
            "connected": false,
            "cdp_host": config.host,
            "cdp_port": config.port,
            "error": "No TradingView chart target found",
            "candidates": targets,
        }),
        TargetSelection::Ambiguous(candidates) => json!({
            "connected": false,
            "cdp_host": config.host,
            "cdp_port": config.port,
            "error": "Multiple TradingView chart targets found",
            "candidates": candidates,
        }),
    };
    Ok(data)
}

pub async fn state(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var chart = {CHART_API};
                    var visibleRange = null;
                    try {{ visibleRange = chart.getVisibleRange(); }} catch(e) {{}}
                    return {{
                        symbol: chart.symbol(),
                        timeframe: chart.resolution(),
                        chart_type: chart.chartType(),
                        visible_range: visibleRange
                    }};
                }})()
                "#
            ),
            false,
        )
        .await
}

pub async fn quote(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var chart = {CHART_API};
                    var bars = {BARS_PATH};
                    var quote = {{
                        symbol: chart.symbol(),
                        last: null,
                        open: null,
                        high: null,
                        low: null,
                        volume: null
                    }};
                    if (bars && typeof bars.lastIndex === 'function') {{
                        var last = bars.valueAt(bars.lastIndex());
                        if (last) {{
                            quote.open = last[1];
                            quote.high = last[2];
                            quote.low = last[3];
                            quote.last = last[4];
                            quote.volume = last[5] || 0;
                        }}
                    }}
                    return quote;
                }})()
                "#
            ),
            false,
        )
        .await
}

pub async fn ohlcv_summary(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var chart = {CHART_API};
                    var bars = {BARS_PATH};
                    if (!bars || typeof bars.lastIndex !== 'function') {{
                        throw new Error('Could not extract OHLCV data. The chart may still be loading.');
                    }}
                    var result = [];
                    var end = bars.lastIndex();
                    var start = Math.max(bars.firstIndex(), end - 99);
                    for (var i = start; i <= end; i++) {{
                        var v = bars.valueAt(i);
                        if (v) result.push({{time: v[0], open: v[1], high: v[2], low: v[3], close: v[4], volume: v[5] || 0}});
                    }}
                    if (result.length === 0) {{
                        throw new Error('Could not extract OHLCV data. The chart may still be loading.');
                    }}
                    var first = result[0];
                    var last = result[result.length - 1];
                    var high = Math.max.apply(null, result.map(function(bar) {{ return bar.high; }}));
                    var low = Math.min.apply(null, result.map(function(bar) {{ return bar.low; }}));
                    var volume = result.reduce(function(sum, bar) {{ return sum + (bar.volume || 0); }}, 0);
                    return {{
                        symbol: chart.symbol(),
                        timeframe: chart.resolution(),
                        bar_count: result.length,
                        first_time: first.time,
                        last_time: last.time,
                        open: first.open,
                        high: high,
                        low: low,
                        close: last.close,
                        volume: volume
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
                            resolve({{
                                requested_symbol: {symbol_literal},
                                observed_symbol: chart.symbol()
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
                    return {{
                        requested_timeframe: {timeframe_literal},
                        observed_timeframe: chart.resolution()
                    }};
                }})()
                "#
            ),
            false,
        )
        .await
}

pub async fn screenshot_full(
    runtime: &mut impl RuntimeEvaluator,
    output_path: &str,
) -> Result<Value, AppError> {
    let bytes = runtime.capture_full_screenshot().await?;
    let path = Path::new(output_path);
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|err| {
            AppError::new(
                ErrorKind::Internal,
                format!("Could not create screenshot output directory: {err}"),
            )
        })?;
    }
    fs::write(path, &bytes).map_err(|err| {
        AppError::new(
            ErrorKind::Internal,
            format!("Could not write screenshot output: {err}"),
        )
    })?;
    Ok(json!({
        "output_path": output_path,
        "region": "full",
        "size_bytes": bytes.len(),
    }))
}

fn js_string(value: &str) -> Result<String, AppError> {
    serde_json::to_string(value).map_err(|err| {
        AppError::new(
            ErrorKind::Internal,
            format!("Could not serialize JavaScript string literal: {err}"),
        )
    })
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use tempfile::tempdir;

    use super::*;

    struct FakeRuntime {
        evaluated: Vec<(String, bool)>,
        responses: VecDeque<Value>,
        screenshot: Vec<u8>,
    }

    impl FakeRuntime {
        fn new(responses: impl Into<VecDeque<Value>>) -> Self {
            Self {
                evaluated: Vec::new(),
                responses: responses.into(),
                screenshot: vec![137, 80, 78, 71],
            }
        }
    }

    impl RuntimeEvaluator for FakeRuntime {
        async fn evaluate(
            &mut self,
            expression: &str,
            await_promise: bool,
        ) -> Result<Value, AppError> {
            self.evaluated.push((expression.to_string(), await_promise));
            Ok(self.responses.pop_front().unwrap_or(Value::Null))
        }

        async fn capture_full_screenshot(&mut self) -> Result<Vec<u8>, AppError> {
            Ok(self.screenshot.clone())
        }
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
    async fn screenshot_full_writes_png_bytes() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("full.png");
        let mut runtime = FakeRuntime::new([]);

        let data = screenshot_full(&mut runtime, output.to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(data["region"], "full");
        assert_eq!(data["size_bytes"], 4);
        assert_eq!(fs::read(output).unwrap(), vec![137, 80, 78, 71]);
    }
}
