use serde_json::{Value, json};

use crate::{cdp::RuntimeEvaluator, error::AppError};

use super::super::common::{CHART_API, DEFAULT_OHLCV_COUNT, MAX_OHLCV_COUNT, js_string, round2};

pub async fn data_shapes(
    runtime: &mut impl RuntimeEvaluator,
    filter: Option<&str>,
    count: Option<usize>,
    verbose: bool,
) -> Result<Value, AppError> {
    let limit = normalized_count(count);
    runtime
        .evaluate(
            &build_shapes_expression(filter.unwrap_or(""), limit)?,
            false,
        )
        .await
        .map(|raw| summarize_pine_shapes(raw, limit, verbose))
}

fn normalized_count(count: Option<usize>) -> usize {
    count.unwrap_or(DEFAULT_OHLCV_COUNT).min(MAX_OHLCV_COUNT)
}

fn build_shapes_expression(filter: &str, limit: usize) -> Result<String, AppError> {
    let filter_literal = js_string(filter)?;
    Ok(format!(
        r#"
        (function() {{
            var chart = {CHART_API}._chartWidget;
            var model = chart.model();
            var sources = model.model().dataSources();
            var mainSeries = model.mainSeries && model.mainSeries();
            var mainBars = mainSeries && mainSeries.bars && mainSeries.bars();
            var filter = {filter_literal};
            var maxBars = {limit};
            var results = [];

            function valueAt(series, index) {{
                if (!series || !series.valueAt) return null;
                try {{ return series.valueAt(index); }} catch (e) {{ return null; }}
            }}

            function timestampFrom(row) {{
                if (!row) return null;
                if (Array.isArray(row)) return row[0];
                return row.time || row.timestamp || row._time || null;
            }}

            function numberFrom(row, arrayIndex, keys) {{
                if (!row) return null;
                if (Array.isArray(row)) return row[arrayIndex];
                for (var i = 0; i < keys.length; i++) {{
                    var v = row[keys[i]];
                    if (typeof v === 'number') return v;
                }}
                return null;
            }}

            function ohlcFor(index) {{
                var row = valueAt(mainBars, index);
                if (!row) return null;
                var timestamp = timestampFrom(row);
                var result = {{
                    timestamp: timestamp,
                    open: numberFrom(row, 1, ['open', 'o']),
                    high: numberFrom(row, 2, ['high', 'h']),
                    low: numberFrom(row, 3, ['low', 'l']),
                    close: numberFrom(row, 4, ['close', 'c'])
                }};
                if (typeof timestamp === 'number') {{
                    try {{ result.time = new Date(timestamp * 1000).toISOString(); }} catch (e) {{}}
                }}
                return result;
            }}

            function isActiveShapeValue(value) {{
                if (value === null || value === undefined || value === false || value === 0) return false;
                if (typeof value === 'number') return isFinite(value) && value !== 0;
                if (typeof value === 'string') return value.length > 0;
                return true;
            }}

            for (var si = 0; si < sources.length; si++) {{
                var source = sources[si];
                if (!source.metaInfo) continue;
                try {{
                    var meta = source.metaInfo();
                    var name = meta.description || meta.shortDescription || meta.id || '';
                    if (!name) continue;
                    if (filter && name.indexOf(filter) === -1) continue;
                    if (!meta.plots || !source._data) continue;

                    var shapePlots = [];
                    for (var pi = 0; pi < meta.plots.length; pi++) {{
                        var plot = meta.plots[pi];
                        if (!plot || plot.type !== 'shapes') continue;
                        var style = meta.styles && meta.styles[plot.id] ? meta.styles[plot.id] : {{}};
                        var defaults = meta.defaults && meta.defaults.styles && meta.defaults.styles[plot.id]
                            ? meta.defaults.styles[plot.id]
                            : {{}};
                        shapePlots.push({{
                            plotIndex: pi,
                            dataIndex: pi + 1,
                            id: plot.id,
                            title: style.title || plot.title || plot.id,
                            shape: defaults.plottype || defaults.shape || 'unknown',
                            location: defaults.location || 'unknown',
                            color: defaults.color || null,
                            size: style.size || defaults.size || 'auto'
                        }});
                    }}
                    if (shapePlots.length === 0) continue;

                    var data = source._data;
                    var lastIdx = data.lastIndex && data.lastIndex();
                    var firstAvailable = data.firstIndex && data.firstIndex();
                    if (typeof lastIdx !== 'number' || typeof firstAvailable !== 'number') continue;
                    var firstIdx = Math.max(firstAvailable, lastIdx - maxBars + 1);
                    var signals = [];

                    for (var barIndex = lastIdx; barIndex >= firstIdx; barIndex--) {{
                        var row = valueAt(data, barIndex);
                        if (!row) continue;
                        for (var sp = 0; sp < shapePlots.length; sp++) {{
                            var shapePlot = shapePlots[sp];
                            var value = row[shapePlot.dataIndex];
                            if (!isActiveShapeValue(value)) continue;
                            signals.push({{
                                plot: shapePlot.title,
                                shape: shapePlot.shape,
                                location: shapePlot.location,
                                color: shapePlot.color,
                                barIndex: barIndex,
                                value: value,
                                ohlc: ohlcFor(barIndex),
                                plot_id: shapePlot.id,
                                data_index: shapePlot.dataIndex,
                                size: shapePlot.size
                            }});
                        }}
                    }}

                    results.push({{
                        name: name,
                        shapePlots: shapePlots,
                        signals: signals,
                        signalCount: signals.length,
                        barsScanned: Math.max(0, lastIdx - firstIdx + 1)
                    }});
                }} catch (e) {{}}
            }}
            return results;
        }})()
        "#
    ))
}

fn summarize_pine_shapes(raw: Value, limit: usize, verbose: bool) -> Value {
    let empty = Vec::new();
    let studies = raw
        .as_array()
        .unwrap_or(&empty)
        .iter()
        .map(|study| {
            let shape_plots = study
                .get("shapePlots")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .map(|plot| summarize_shape_plot(plot, verbose))
                .collect::<Vec<_>>();
            let signals = study
                .get("signals")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .map(|signal| summarize_shape_signal(signal, verbose))
                .collect::<Vec<_>>();
            json!({
                "name": study.get("name").cloned().unwrap_or(Value::Null),
                "shape_plot_count": shape_plots.len(),
                "shape_plots": shape_plots,
                "signal_count": study
                    .get("signalCount")
                    .cloned()
                    .unwrap_or_else(|| json!(signals.len())),
                "bars_scanned": study.get("barsScanned").cloned().unwrap_or(Value::Null),
                "signals": signals,
            })
        })
        .collect::<Vec<_>>();
    json!({
        "study_count": studies.len(),
        "scan_count": limit,
        "studies": studies,
    })
}

fn summarize_shape_plot(plot: &Value, verbose: bool) -> Value {
    let mut summary = json!({
        "title": plot.get("title").cloned().unwrap_or(Value::Null),
        "shape": plot.get("shape").cloned().unwrap_or(Value::Null),
        "location": plot.get("location").cloned().unwrap_or(Value::Null),
        "color": plot.get("color").cloned().unwrap_or(Value::Null),
    });
    if verbose {
        summary["id"] = plot.get("id").cloned().unwrap_or(Value::Null);
        summary["plot_index"] = plot.get("plotIndex").cloned().unwrap_or(Value::Null);
        summary["data_index"] = plot.get("dataIndex").cloned().unwrap_or(Value::Null);
        summary["size"] = plot.get("size").cloned().unwrap_or(Value::Null);
    }
    summary
}

fn summarize_shape_signal(signal: &Value, verbose: bool) -> Value {
    let mut summary = json!({
        "plot": signal.get("plot").cloned().unwrap_or(Value::Null),
        "shape": signal.get("shape").cloned().unwrap_or(Value::Null),
        "location": signal.get("location").cloned().unwrap_or(Value::Null),
        "color": signal.get("color").cloned().unwrap_or(Value::Null),
        "bar_index": signal.get("barIndex").cloned().unwrap_or(Value::Null),
        "value": signal.get("value").cloned().unwrap_or(Value::Null),
        "ohlc": summarize_ohlc(signal.get("ohlc").unwrap_or(&Value::Null)),
    });
    if verbose {
        summary["plot_id"] = signal.get("plot_id").cloned().unwrap_or(Value::Null);
        summary["data_index"] = signal.get("data_index").cloned().unwrap_or(Value::Null);
        summary["size"] = signal.get("size").cloned().unwrap_or(Value::Null);
    }
    summary
}

fn summarize_ohlc(ohlc: &Value) -> Value {
    if !ohlc.is_object() {
        return Value::Null;
    }
    json!({
        "timestamp": ohlc.get("timestamp").cloned().unwrap_or(Value::Null),
        "time": ohlc.get("time").cloned().unwrap_or(Value::Null),
        "open": rounded_field(ohlc, "open"),
        "high": rounded_field(ohlc, "high"),
        "low": rounded_field(ohlc, "low"),
        "close": rounded_field(ohlc, "close"),
    })
}

fn rounded_field(value: &Value, key: &str) -> Value {
    value
        .get(key)
        .and_then(Value::as_f64)
        .map(round2)
        .map(Value::from)
        .unwrap_or(Value::Null)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::super::test_support::FakeRuntime;
    use super::*;

    #[tokio::test]
    async fn data_shapes_summarizes_signals_and_verbose_metadata() {
        let raw = json!([{
            "name": "Flow Matrix",
            "shapePlots": [{
                "plotIndex": 2,
                "dataIndex": 3,
                "id": "buy_signal",
                "title": "Buy",
                "shape": "shape.triangleup",
                "location": "BelowBar",
                "color": "green",
                "size": "small"
            }],
            "signals": [{
                "plot": "Buy",
                "shape": "shape.triangleup",
                "location": "BelowBar",
                "color": "green",
                "barIndex": 42,
                "value": 1,
                "ohlc": {
                    "timestamp": 1770000000,
                    "time": "2026-02-02T00:00:00.000Z",
                    "open": 101.234,
                    "high": 105.678,
                    "low": 99.123,
                    "close": 104.456
                },
                "plot_id": "buy_signal",
                "data_index": 3,
                "size": "small"
            }],
            "signalCount": 1,
            "barsScanned": 100
        }]);
        let mut runtime = FakeRuntime::new([raw]);

        let result = data_shapes(&mut runtime, Some("Flow'); bad(); ('"), Some(100), true)
            .await
            .unwrap();

        assert_eq!(result["study_count"], 1);
        assert_eq!(result["scan_count"], 100);
        assert_eq!(result["studies"][0]["shape_plot_count"], 1);
        assert_eq!(result["studies"][0]["shape_plots"][0]["id"], "buy_signal");
        assert_eq!(result["studies"][0]["signals"][0]["bar_index"], 42);
        assert_eq!(result["studies"][0]["signals"][0]["ohlc"]["high"], 105.68);
        assert!(runtime.evaluated[0].0.contains("\"Flow'); bad(); ('\""));
    }

    #[tokio::test]
    async fn data_shapes_clamps_count_to_maximum() {
        let mut runtime = FakeRuntime::new([json!([])]);

        let result = data_shapes(&mut runtime, None, Some(900), false)
            .await
            .unwrap();

        assert_eq!(result["scan_count"], 500);
        assert!(runtime.evaluated[0].0.contains("var maxBars = 500;"));
    }

    #[tokio::test]
    async fn data_shapes_returns_empty_success_for_missing_raw_array() {
        let mut runtime = FakeRuntime::new([Value::Null]);

        let result = data_shapes(&mut runtime, None, None, false).await.unwrap();

        assert_eq!(result["study_count"], 0);
        assert_eq!(result["scan_count"], 100);
        assert_eq!(result["studies"], json!([]));
    }
}
