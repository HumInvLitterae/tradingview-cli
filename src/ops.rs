use std::{fs, io::Cursor, path::Path};

use image::ImageFormat;
use serde_json::{Value, json};

use crate::{
    cdp::{CdpClient, RuntimeEvaluator, ScreenshotClip},
    error::{AppError, ErrorKind},
    transport::{self, TargetSelection, TransportConfig},
};

const CHART_API: &str = "window.TradingViewApi._activeChartWidgetWV.value()";
const CHART_WIDGET_COLLECTION: &str = "window.TradingViewApi._chartWidgetCollection";
const BARS_PATH: &str =
    "window.TradingViewApi._activeChartWidgetWV.value()._chartWidget.model().mainSeries().bars()";
const SYMBOL_SEARCH_URL: &str = "https://symbol-search.tradingview.com/symbol_search/v3/";
const DEFAULT_OHLCV_COUNT: usize = 100;
const MAX_OHLCV_COUNT: usize = 500;

pub async fn status(config: &TransportConfig) -> Result<Value, AppError> {
    let targets = transport::fetch_targets(config).await?;
    let data = match transport::select_target(&targets) {
        TargetSelection::Selected(target) => {
            let mut data = json!({
                "connected": true,
                "cdp_connected": true,
                "target_id": target.id,
                "target_url": target.url,
                "target_title": target.title,
                "cdp_host": config.host,
                "cdp_port": config.port,
                "chart_symbol": "unknown",
                "chart_resolution": "unknown",
                "chart_type": null,
                "api_available": false,
            });
            let mut runtime = CdpClient::connect(&target).await?;
            if let Ok(chart) = chart_status(&mut runtime).await {
                merge_object(&mut data, chart);
            }
            data
        }
        TargetSelection::None => json!({
            "connected": false,
            "cdp_connected": false,
            "cdp_host": config.host,
            "cdp_port": config.port,
            "chart_symbol": "unknown",
            "chart_resolution": "unknown",
            "chart_type": null,
            "api_available": false,
            "error": "No TradingView chart target found",
            "candidates": targets,
        }),
        TargetSelection::Ambiguous(candidates) => json!({
            "connected": false,
            "cdp_connected": false,
            "cdp_host": config.host,
            "cdp_port": config.port,
            "chart_symbol": "unknown",
            "chart_resolution": "unknown",
            "chart_type": null,
            "api_available": false,
            "error": "Multiple TradingView chart targets found",
            "candidates": candidates,
        }),
    };
    Ok(data)
}

async fn chart_status(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var result = {{
                        chart_symbol: "unknown",
                        chart_resolution: "unknown",
                        chart_type: null,
                        api_available: false
                    }};
                    try {{
                        var chart = {CHART_API};
                        result.chart_symbol = chart.symbol();
                        result.chart_resolution = chart.resolution();
                        result.chart_type = chart.chartType();
                        result.api_available = true;
                    }} catch(e) {{
                        result.api_error = e && e.message ? e.message : String(e);
                    }}
                    return result;
                }})()
                "#
            ),
            false,
        )
        .await
}

pub async fn state(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var chart = {CHART_API};
                    var visibleRange = null;
                    var studies = [];
                    try {{ visibleRange = chart.getVisibleRange(); }} catch(e) {{}}
                    try {{
                        var allStudies = chart.getAllStudies();
                        studies = allStudies.map(function(s) {{
                            return {{ id: s.id, name: s.name || s.title || "unknown" }};
                        }});
                    }} catch(e) {{}}
                    var resolution = chart.resolution();
                    var chartType = chart.chartType();
                    return {{
                        symbol: chart.symbol(),
                        resolution: resolution,
                        timeframe: resolution,
                        chartType: chartType,
                        chart_type: chartType,
                        studies: studies,
                        visible_range: visibleRange
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

pub async fn symbol_search(query: &str) -> Result<Value, AppError> {
    let url = reqwest::Url::parse_with_params(
        SYMBOL_SEARCH_URL,
        &[
            ("text", query),
            ("hl", "1"),
            ("exchange", ""),
            ("lang", "en"),
            ("search_type", ""),
            ("domain", "production"),
        ],
    )
    .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))?;
    let response = reqwest::Client::new()
        .get(url)
        .header("Origin", "https://www.tradingview.com")
        .header("Referer", "https://www.tradingview.com/")
        .send()
        .await
        .map_err(|err| AppError::new(ErrorKind::Connection, err.to_string()))?;

    let status = response.status();
    if !status.is_success() {
        return Err(AppError::new(
            ErrorKind::Connection,
            format!("Symbol search API returned {status}"),
        ));
    }

    let value = response
        .json::<Value>()
        .await
        .map_err(|err| AppError::new(ErrorKind::InternalApiUnavailable, err.to_string()))?;
    Ok(normalize_symbol_search_response(query, &value))
}

pub async fn quote(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var chart = {CHART_API};
                    var bars = {BARS_PATH};
                    var ext = {{}};
                    try {{ ext = chart.symbolExt() || {{}}; }} catch(e) {{}}
                    var quote = {{
                        symbol: chart.symbol(),
                        time: null,
                        last: null,
                        close: null,
                        open: null,
                        high: null,
                        low: null,
                        volume: null,
                        description: ext.description || null,
                        exchange: ext.exchange || null,
                        type: ext.type || null
                    }};
                    if (bars && typeof bars.lastIndex === 'function') {{
                        var last = bars.valueAt(bars.lastIndex());
                        if (last) {{
                            quote.time = last[0];
                            quote.open = last[1];
                            quote.high = last[2];
                            quote.low = last[3];
                            quote.close = last[4];
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

pub async fn ohlcv_bars(
    runtime: &mut impl RuntimeEvaluator,
    count: Option<usize>,
) -> Result<Value, AppError> {
    let limit = normalized_count(count);
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
                    var start = Math.max(bars.firstIndex(), end - {limit} + 1);
                    for (var i = start; i <= end; i++) {{
                        var v = bars.valueAt(i);
                        if (v) result.push({{time: v[0], open: v[1], high: v[2], low: v[3], close: v[4], volume: v[5] || 0}});
                    }}
                    if (result.length === 0) {{
                        throw new Error('Could not extract OHLCV data. The chart may still be loading.');
                    }}
                    return {{
                        symbol: chart.symbol(),
                        resolution: chart.resolution(),
                        timeframe: chart.resolution(),
                        bar_count: result.length,
                        total_available: (typeof bars.size === 'function') ? bars.size() : null,
                        source: "direct_bars",
                        bars: result
                    }};
                }})()
                "#
            ),
            false,
        )
        .await
}

pub async fn ohlcv_summary(
    runtime: &mut impl RuntimeEvaluator,
    count: Option<usize>,
) -> Result<Value, AppError> {
    let data = ohlcv_bars(runtime, count).await?;
    summarize_ohlcv(data)
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

pub async fn watchlist_get(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            r#"
            (function() {
                try {
                    var rightArea = document.querySelector('[class*="layout__area--right"]');
                    if (!rightArea || rightArea.offsetWidth < 50) return { count: 0, source: "panel_closed", symbols: [] };
                } catch(e) {}

                var results = [];
                var seen = {};
                var container = document.querySelector('[class*="layout__area--right"]');
                if (!container) return { count: 0, source: "no_container", symbols: [] };

                var symbolEls = container.querySelectorAll('[data-symbol-full]');
                for (var i = 0; i < symbolEls.length; i++) {
                    var sym = symbolEls[i].getAttribute('data-symbol-full');
                    if (!sym || seen[sym]) continue;
                    seen[sym] = true;

                    var row = symbolEls[i].closest('[class*="row"]') || symbolEls[i].parentElement;
                    var cells = row ? row.querySelectorAll('[class*="cell"], [class*="column"]') : [];
                    var nums = [];
                    for (var j = 0; j < cells.length; j++) {
                        var t = cells[j].textContent.trim();
                        if (t && /^[\-+]?[\d,]+\.?\d*%?$/.test(t.replace(/[\s,]/g, ''))) nums.push(t);
                    }
                    results.push({ symbol: sym, last: nums[0] || null, change: nums[1] || null, change_percent: nums[2] || null });
                }

                if (results.length > 0) return { count: results.length, source: "data_attributes", symbols: results };

                var items = container.querySelectorAll('[class*="symbolName"], [class*="tickerName"], [class*="symbol-"]');
                for (var k = 0; k < items.length; k++) {
                    var text = items[k].textContent.trim();
                    if (text && /^[A-Z][A-Z0-9.:!]{0,20}$/.test(text) && !seen[text]) {
                        seen[text] = true;
                        results.push({ symbol: text, last: null, change: null, change_percent: null });
                    }
                }

                return { count: results.length, source: results.length > 0 ? "text_scan" : "empty", symbols: results };
            })()
            "#,
            false,
        )
        .await
}

pub async fn pane_list(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var layoutNames = {{
                        "s": "single",
                        "single": "single",
                        "2h": "2 horizontal",
                        "2v": "2 vertical",
                        "2x2": "2 by 2",
                        "4": "4 panes",
                        "6": "6 panes",
                        "8": "8 panes"
                    }};
                    var cwc = {CHART_WIDGET_COLLECTION};
                    var layoutType = cwc._layoutType;
                    if (typeof layoutType === "object" && layoutType && typeof layoutType.value === "function") layoutType = layoutType.value();
                    var count = cwc.inlineChartsCount;
                    if (typeof count === "object" && count && typeof count.value === "function") count = count.value();

                    var all = cwc.getAll();
                    var panes = [];
                    for (var i = 0; i < all.length; i++) {{
                        try {{
                            var c = all[i];
                            var model = c.model ? c.model() : null;
                            var mainSeries = model ? model.mainSeries() : null;
                            var sym = mainSeries ? mainSeries.symbol() : "unknown";
                            var res = mainSeries ? mainSeries.interval() : null;
                            panes.push({{ index: i, symbol: sym, resolution: res || null }});
                        }} catch(e) {{
                            panes.push({{ index: i, symbol: null, resolution: null, error: e.message }});
                        }}
                    }}

                    var activeChart = {CHART_API};
                    var activeIndex = null;
                    for (var j = 0; j < all.length; j++) {{
                        try {{
                            if (all[j].model && activeChart._chartWidget && all[j] === activeChart._chartWidget) {{
                                activeIndex = j;
                                break;
                            }}
                        }} catch(e) {{}}
                    }}

                    return {{
                        layout: layoutType,
                        layout_name: layoutNames[layoutType] || layoutType,
                        chart_count: count,
                        active_index: activeIndex,
                        panes: panes
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
    let bytes = runtime.capture_screenshot().await?;
    write_screenshot(output_path, &bytes)?;
    Ok(json!({
        "output_path": output_path,
        "region": "full",
        "size_bytes": bytes.len(),
    }))
}

pub async fn screenshot_chart(
    runtime: &mut impl RuntimeEvaluator,
    output_path: &str,
) -> Result<Value, AppError> {
    let bounds = runtime
        .evaluate(
            r#"
            (function() {
                var el = document.querySelector('[data-name="pane-canvas"]')
                    || document.querySelector('[class*="chart-container"]')
                    || document.querySelector('canvas');
                if (!el) return null;
                var rect = el.getBoundingClientRect();
                var viewport = window.visualViewport || {};
                return {
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: rect.height,
                    viewport_width: viewport.width || window.innerWidth,
                    viewport_height: viewport.height || window.innerHeight
                };
            })()
            "#,
            false,
        )
        .await?;
    let bounds = screenshot_bounds_from_value(&bounds)?;
    let full_bytes = runtime.capture_screenshot().await?;
    let bytes = crop_screenshot_to_bounds(&full_bytes, &bounds)?;
    write_screenshot(output_path, &bytes)?;
    Ok(json!({
        "output_path": output_path,
        "region": "chart",
        "size_bytes": bytes.len(),
        "clip": bounds.clip,
    }))
}

fn write_screenshot(output_path: &str, bytes: &[u8]) -> Result<(), AppError> {
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
    fs::write(path, bytes).map_err(|err| {
        AppError::new(
            ErrorKind::Internal,
            format!("Could not write screenshot output: {err}"),
        )
    })?;
    Ok(())
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
    Ok(summary)
}

struct ScreenshotBounds {
    clip: ScreenshotClip,
    viewport_width: f64,
    viewport_height: f64,
}

fn screenshot_bounds_from_value(bounds: &Value) -> Result<ScreenshotBounds, AppError> {
    let Some(object) = bounds.as_object() else {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Could not find TradingView chart bounds for screenshot",
        ));
    };
    let number = |key: &str| -> Result<f64, AppError> {
        object.get(key).and_then(Value::as_f64).ok_or_else(|| {
            AppError::new(
                ErrorKind::InternalApiUnavailable,
                format!("TradingView chart bounds did not include numeric {key}"),
            )
        })
    };
    let clip = ScreenshotClip {
        x: number("x")?,
        y: number("y")?,
        width: number("width")?,
        height: number("height")?,
        scale: 1.0,
    };
    let viewport_width = number("viewport_width")?;
    let viewport_height = number("viewport_height")?;
    if !clip.x.is_finite()
        || !clip.y.is_finite()
        || !clip.width.is_finite()
        || !clip.height.is_finite()
        || !viewport_width.is_finite()
        || !viewport_height.is_finite()
        || clip.width <= 0.0
        || clip.height <= 0.0
        || viewport_width <= 0.0
        || viewport_height <= 0.0
    {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "TradingView chart bounds were invalid for screenshot",
        )
        .with_details(bounds.clone()));
    }
    Ok(ScreenshotBounds {
        clip,
        viewport_width,
        viewport_height,
    })
}

fn crop_screenshot_to_bounds(
    screenshot: &[u8],
    bounds: &ScreenshotBounds,
) -> Result<Vec<u8>, AppError> {
    let image = image::load_from_memory(screenshot).map_err(|err| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!("Could not decode screenshot PNG for chart crop: {err}"),
        )
    })?;
    let image_width = image.width();
    let image_height = image.height();
    if image_width == 0 || image_height == 0 {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screenshot PNG was empty",
        ));
    }

    let scale_x = image_width as f64 / bounds.viewport_width;
    let scale_y = image_height as f64 / bounds.viewport_height;
    let x = scaled_floor(bounds.clip.x, scale_x, image_width.saturating_sub(1));
    let y = scaled_floor(bounds.clip.y, scale_y, image_height.saturating_sub(1));
    let right = scaled_ceil(
        bounds.clip.x + bounds.clip.width,
        scale_x,
        image_width,
        x + 1,
    );
    let bottom = scaled_ceil(
        bounds.clip.y + bounds.clip.height,
        scale_y,
        image_height,
        y + 1,
    );
    let width = right.saturating_sub(x);
    let height = bottom.saturating_sub(y);
    if width == 0 || height == 0 {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "TradingView chart bounds were outside the screenshot",
        ));
    }

    let cropped = image.crop_imm(x, y, width, height);
    let mut cursor = Cursor::new(Vec::new());
    cropped
        .write_to(&mut cursor, ImageFormat::Png)
        .map_err(|err| {
            AppError::new(
                ErrorKind::Internal,
                format!("Could not encode cropped chart screenshot: {err}"),
            )
        })?;
    Ok(cursor.into_inner())
}

fn scaled_floor(value: f64, scale: f64, max: u32) -> u32 {
    (value * scale).floor().clamp(0.0, max as f64) as u32
}

fn scaled_ceil(value: f64, scale: f64, max: u32, min: u32) -> u32 {
    (value * scale).ceil().clamp(min as f64, max as f64) as u32
}

fn normalize_symbol_search_response(query: &str, value: &Value) -> Value {
    let rows = value
        .get("symbols")
        .and_then(Value::as_array)
        .or_else(|| value.as_array())
        .cloned()
        .unwrap_or_default();
    let results = rows
        .into_iter()
        .take(15)
        .map(|row| {
            let symbol = strip_em(
                row.get("symbol")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
            );
            let description = strip_em(
                row.get("description")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
            );
            let exchange = row
                .get("exchange")
                .or_else(|| row.get("prefix"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let symbol_type = row
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let full_name = if exchange.is_empty() {
                symbol.clone()
            } else {
                format!("{exchange}:{symbol}")
            };
            json!({
                "symbol": symbol,
                "description": description,
                "exchange": exchange,
                "type": symbol_type,
                "full_name": full_name,
            })
        })
        .collect::<Vec<_>>();

    json!({
        "query": query,
        "source": "rest_api",
        "count": results.len(),
        "results": results,
    })
}

fn strip_em(value: &str) -> String {
    value.replace("<em>", "").replace("</em>", "")
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn require_finite(value: f64, label: &str) -> Result<(), AppError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorKind::Validation,
            format!("{label} must be a finite number"),
        ))
    }
}

fn merge_object(target: &mut Value, source: Value) {
    let (Some(target), Some(source)) = (target.as_object_mut(), source.as_object()) else {
        return;
    };
    for (key, value) in source {
        target.insert(key.clone(), value.clone());
    }
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

    use image::{ImageBuffer, ImageFormat, Rgba};
    use tempfile::tempdir;

    use super::*;

    struct FakeRuntime {
        evaluated: Vec<(String, bool)>,
        responses: VecDeque<Value>,
        screenshot: Vec<u8>,
        screenshot_count: usize,
    }

    impl FakeRuntime {
        fn new(responses: impl Into<VecDeque<Value>>) -> Self {
            Self {
                evaluated: Vec::new(),
                responses: responses.into(),
                screenshot: vec![137, 80, 78, 71],
                screenshot_count: 0,
            }
        }

        fn with_screenshot(mut self, screenshot: Vec<u8>) -> Self {
            self.screenshot = screenshot;
            self
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

        async fn capture_screenshot(&mut self) -> Result<Vec<u8>, AppError> {
            self.screenshot_count += 1;
            Ok(self.screenshot.clone())
        }
    }

    fn png_fixture(width: u32, height: u32) -> Vec<u8> {
        let image = ImageBuffer::from_fn(width, height, |x, y| {
            Rgba([(x % 255) as u8, (y % 255) as u8, 100, 255])
        });
        let mut cursor = Cursor::new(Vec::new());
        image
            .write_to(&mut cursor, ImageFormat::Png)
            .expect("test PNG should encode");
        cursor.into_inner()
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
    async fn watchlist_get_returns_runtime_payload() {
        let payload = json!({
            "count": 1,
            "source": "data_attributes",
            "symbols": [{"symbol": "NASDAQ:AAPL", "last": "100", "change": "1", "change_percent": "1%"}]
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);

        let result = watchlist_get(&mut runtime).await.unwrap();

        assert_eq!(result, payload);
        assert!(runtime.evaluated[0].0.contains("data-symbol-full"));
    }

    #[tokio::test]
    async fn pane_list_returns_runtime_payload() {
        let payload = json!({
            "layout": "single",
            "layout_name": "single",
            "chart_count": 1,
            "active_index": 0,
            "panes": [{"index": 0, "symbol": "NASDAQ:AAPL", "resolution": "D"}]
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);

        let result = pane_list(&mut runtime).await.unwrap();

        assert_eq!(result, payload);
        assert!(runtime.evaluated[0].0.contains("_chartWidgetCollection"));
    }

    #[test]
    fn normalize_symbol_search_response_handles_object_and_em_tags() {
        let response = json!({
            "symbols": [{
                "symbol": "<em>AAPL</em>",
                "description": "Apple <em>Inc</em>",
                "exchange": "NASDAQ",
                "type": "stock"
            }]
        });

        let result = normalize_symbol_search_response("AAPL", &response);

        assert_eq!(result["query"], "AAPL");
        assert_eq!(result["source"], "rest_api");
        assert_eq!(result["count"], 1);
        assert_eq!(result["results"][0]["symbol"], "AAPL");
        assert_eq!(result["results"][0]["description"], "Apple Inc");
        assert_eq!(result["results"][0]["full_name"], "NASDAQ:AAPL");
    }

    #[tokio::test]
    async fn ohlcv_count_is_clamped_to_500() {
        let mut runtime = FakeRuntime::new([
            json!({"symbol": "NASDAQ:AAPL", "timeframe": "D", "bars": [{"time": 1, "open": 1, "high": 1, "low": 1, "close": 1, "volume": 1}]}),
        ]);

        let _ = ohlcv_bars(&mut runtime, Some(900)).await;

        assert!(runtime.evaluated[0].0.contains("end - 500 + 1"));
    }

    #[tokio::test]
    async fn ohlcv_summary_returns_legacy_practical_fields() {
        let mut runtime = FakeRuntime::new([json!({
            "symbol": "NASDAQ:AAPL",
            "resolution": "D",
            "timeframe": "D",
            "bars": [
                {"time": 10, "open": 10.0, "high": 12.0, "low": 9.0, "close": 11.0, "volume": 100.0},
                {"time": 20, "open": 11.0, "high": 13.0, "low": 10.0, "close": 12.0, "volume": 200.0}
            ]
        })]);

        let summary = ohlcv_summary(&mut runtime, Some(2)).await.unwrap();

        assert_eq!(summary["bar_count"], 2);
        assert_eq!(summary["period"]["from"], 10);
        assert_eq!(summary["period"]["to"], 20);
        assert_eq!(summary["range"], 4.0);
        assert_eq!(summary["change"], 2.0);
        assert_eq!(summary["avg_volume"], 150.0);
        assert_eq!(summary["symbol"], "NASDAQ:AAPL");
        assert_eq!(summary["timeframe"], "D");
        assert_eq!(summary["last_5_bars"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn set_visible_range_rejects_non_finite_values() {
        let mut runtime = FakeRuntime::new([]);

        let err = set_visible_range(&mut runtime, f64::NAN, 1.0)
            .await
            .expect_err("NaN should be rejected");

        assert_eq!(err.kind, ErrorKind::Validation);
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
        assert_eq!(runtime.screenshot_count, 1);
        assert_eq!(fs::read(output).unwrap(), vec![137, 80, 78, 71]);
    }

    #[tokio::test]
    async fn screenshot_chart_writes_clipped_png_bytes() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("chart.png");
        let mut runtime = FakeRuntime::new([json!({
            "x": 10.0,
            "y": 20.0,
            "width": 640.0,
            "height": 360.0,
            "viewport_width": 1000.0,
            "viewport_height": 500.0
        })])
        .with_screenshot(png_fixture(1000, 500));

        let data = screenshot_chart(&mut runtime, output.to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(data["region"], "chart");
        assert!(data["size_bytes"].as_u64().unwrap() > 0);
        assert_eq!(data["clip"]["x"], 10.0);
        assert_eq!(data["clip"]["width"], 640.0);
        assert!(
            runtime.evaluated[0]
                .0
                .contains("[data-name=\"pane-canvas\"]")
        );
        assert!(
            runtime.evaluated[0]
                .0
                .contains("[class*=\"chart-container\"]")
        );
        assert_eq!(runtime.screenshot_count, 1);

        let cropped = image::load_from_memory(&fs::read(output).unwrap()).unwrap();
        assert_eq!(cropped.width(), 640);
        assert_eq!(cropped.height(), 360);
    }

    #[tokio::test]
    async fn screenshot_chart_rejects_missing_or_invalid_bounds() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("chart.png");
        let mut runtime = FakeRuntime::new([json!({
            "x": 10.0,
            "y": 20.0,
            "width": 0.0,
            "height": 360.0,
            "viewport_width": 1000.0,
            "viewport_height": 500.0
        })]);

        let err = screenshot_chart(&mut runtime, output.to_str().unwrap())
            .await
            .expect_err("zero-width chart bounds should be rejected");

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(runtime.screenshot_count, 0);
    }
}
