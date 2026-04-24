use serde_json::{Value, json};

use crate::{
    cdp::{KeyEvent, KeyEventType, RuntimeEvaluator},
    error::{AppError, ErrorKind},
};

use super::common::{CHART_API, CHART_WIDGET_COLLECTION, js_string};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PaneLayout {
    code: &'static str,
    name: &'static str,
}

const PANE_LAYOUTS: [PaneLayout; 18] = [
    PaneLayout {
        code: "s",
        name: "1 chart",
    },
    PaneLayout {
        code: "2h",
        name: "2 horizontal",
    },
    PaneLayout {
        code: "2v",
        name: "2 vertical",
    },
    PaneLayout {
        code: "2-1",
        name: "2 top, 1 bottom",
    },
    PaneLayout {
        code: "1-2",
        name: "1 top, 2 bottom",
    },
    PaneLayout {
        code: "3h",
        name: "3 horizontal",
    },
    PaneLayout {
        code: "3v",
        name: "3 vertical",
    },
    PaneLayout {
        code: "3s",
        name: "3 custom",
    },
    PaneLayout {
        code: "4",
        name: "2x2 grid",
    },
    PaneLayout {
        code: "4h",
        name: "4 horizontal",
    },
    PaneLayout {
        code: "4v",
        name: "4 vertical",
    },
    PaneLayout {
        code: "4s",
        name: "4 custom",
    },
    PaneLayout {
        code: "6",
        name: "6 charts",
    },
    PaneLayout {
        code: "8",
        name: "8 charts",
    },
    PaneLayout {
        code: "10",
        name: "10 charts",
    },
    PaneLayout {
        code: "12",
        name: "12 charts",
    },
    PaneLayout {
        code: "14",
        name: "14 charts",
    },
    PaneLayout {
        code: "16",
        name: "16 charts",
    },
];

pub fn validate_pane_layout(layout: &str) -> Result<(), AppError> {
    parse_pane_layout(layout).map(|_| ())
}

fn parse_pane_layout(layout: &str) -> Result<PaneLayout, AppError> {
    let normalized: String = layout
        .chars()
        .filter(|character| !character.is_ascii_whitespace())
        .flat_map(char::to_lowercase)
        .collect();

    let canonical = match normalized.as_str() {
        "single" | "1" | "1x1" => "s",
        "2x1" => "2h",
        "1x2" => "2v",
        "2x2" | "grid" | "quad" => "4",
        "3x1" => "3h",
        "1x3" => "3v",
        other => other,
    };

    PANE_LAYOUTS
        .iter()
        .copied()
        .find(|layout| layout.code == canonical)
        .ok_or_else(|| {
            AppError::new(
                ErrorKind::Validation,
                format!("Unknown pane layout: {layout}"),
            )
            .with_details(json!({
                "supported": supported_pane_layouts(),
                "aliases": {
                    "single": "s",
                    "1": "s",
                    "1x1": "s",
                    "2x1": "2h",
                    "1x2": "2v",
                    "2x2": "4",
                    "grid": "4",
                    "quad": "4",
                    "3x1": "3h",
                    "1x3": "3v"
                }
            }))
        })
}

fn supported_pane_layouts() -> Vec<Value> {
    PANE_LAYOUTS
        .iter()
        .map(|layout| json!({ "layout": layout.code, "layout_name": layout.name }))
        .collect()
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

pub async fn watchlist_add(
    runtime: &mut impl RuntimeEvaluator,
    symbol: &str,
) -> Result<Value, AppError> {
    let panel_state = runtime
        .evaluate(
            r#"
            (function() {
                var rightArea = document.querySelector('[class*="layout__area--right"]');
                if (rightArea && rightArea.offsetWidth >= 50 && rightArea.querySelector('[data-symbol-full]')) {
                    return { opened: false, already_open: true, source: 'visible_watchlist_rows' };
                }

                var btn = document.querySelector('[data-name="base-watchlist-widget-button"]')
                    || document.querySelector('[aria-label*="Watchlist"]');
                if (!btn) return { error: 'Watchlist button not found' };
                var isActive = btn.getAttribute('aria-pressed') === 'true'
                    || btn.classList.toString().indexOf('Active') !== -1
                    || btn.classList.toString().indexOf('active') !== -1;
                if (!isActive) {
                    btn.click();
                    return { opened: true };
                }
                return { opened: false };
            })()
            "#,
            false,
        )
        .await?;

    if let Some(message) = panel_state.get("error").and_then(Value::as_str) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            message.to_string(),
        ));
    }

    if panel_state
        .get("opened")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        runtime
            .evaluate(
                "new Promise(function(resolve) { setTimeout(function() { resolve(true); }, 500); })",
                true,
            )
            .await?;
    }

    let add_clicked = runtime
        .evaluate(
            r#"
            (function() {
                var selectors = [
                    '[data-name="add-symbol-button"]',
                    '[aria-label="Add symbol"]',
                    '[aria-label*="Add symbol"]',
                    'button[class*="addSymbol"]'
                ];
                for (var s = 0; s < selectors.length; s++) {
                    var btn = document.querySelector(selectors[s]);
                    if (btn && btn.offsetParent !== null) {
                        btn.click();
                        return { found: true, selector: selectors[s] };
                    }
                }
                var container = document.querySelector('[class*="layout__area--right"]');
                if (container) {
                    var buttons = container.querySelectorAll('button');
                    for (var i = 0; i < buttons.length; i++) {
                        var ariaLabel = buttons[i].getAttribute('aria-label') || '';
                        if (/add.*symbol/i.test(ariaLabel) || buttons[i].textContent.trim() === '+') {
                            buttons[i].click();
                            return { found: true, method: 'fallback' };
                        }
                    }
                }
                return { found: false };
            })()
            "#,
            false,
        )
        .await?;

    if !add_clicked
        .get("found")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Add symbol button not found in watchlist panel",
        ));
    }

    runtime
        .evaluate(
            "new Promise(function(resolve) { setTimeout(function() { resolve(true); }, 300); })",
            true,
        )
        .await?;
    runtime.insert_text(symbol).await?;
    runtime
        .evaluate(
            "new Promise(function(resolve) { setTimeout(function() { resolve(true); }, 500); })",
            true,
        )
        .await?;
    dispatch_key(runtime, KeyEventType::KeyDown, "Enter", "Enter", 13).await?;
    dispatch_key(runtime, KeyEventType::KeyUp, "Enter", "Enter", 13).await?;
    runtime
        .evaluate(
            "new Promise(function(resolve) { setTimeout(function() { resolve(true); }, 300); })",
            true,
        )
        .await?;
    dispatch_key(runtime, KeyEventType::KeyDown, "Escape", "Escape", 27).await?;
    dispatch_key(runtime, KeyEventType::KeyUp, "Escape", "Escape", 27).await?;

    Ok(json!({
        "symbol": symbol,
        "requested_symbol": symbol,
        "action": "added",
        "source": "dom_input",
        "opened_panel": panel_state.get("opened").cloned().unwrap_or(Value::Bool(false)),
        "add_button": add_clicked,
    }))
}

async fn dispatch_key(
    runtime: &mut impl RuntimeEvaluator,
    event_type: KeyEventType,
    key: &'static str,
    code: &'static str,
    windows_virtual_key_code: i64,
) -> Result<(), AppError> {
    runtime
        .dispatch_key_event(KeyEvent {
            event_type,
            key,
            code,
            windows_virtual_key_code,
        })
        .await
}

pub async fn pane_list(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var layoutNames = {{
                        "s": "1 chart",
                        "single": "1 chart",
                        "2h": "2 horizontal",
                        "2v": "2 vertical",
                        "2-1": "2 top, 1 bottom",
                        "1-2": "1 top, 2 bottom",
                        "3h": "3 horizontal",
                        "3v": "3 vertical",
                        "3s": "3 custom",
                        "2x2": "2x2 grid",
                        "4": "2x2 grid",
                        "4h": "4 horizontal",
                        "4v": "4 vertical",
                        "4s": "4 custom",
                        "6": "6 charts",
                        "8": "8 charts",
                        "10": "10 charts",
                        "12": "12 charts",
                        "14": "14 charts",
                        "16": "16 charts"
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

pub async fn pane_layout(
    runtime: &mut impl RuntimeEvaluator,
    layout: &str,
) -> Result<Value, AppError> {
    let layout = parse_pane_layout(layout)?;
    let layout_literal = js_string(layout.code)?;
    let layout_name_literal = js_string(layout.name)?;
    let result = runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    try {{
                        var cwc = {CHART_WIDGET_COLLECTION};
                        if (!cwc || typeof cwc.setLayout !== "function") {{
                            return {{ error: "Chart widget collection setLayout unavailable" }};
                        }}
                        cwc.setLayout({layout_literal});
                        return new Promise(function(resolve) {{
                            setTimeout(function() {{
                                try {{
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

                                    resolve({{
                                        layout: {layout_literal},
                                        layout_name: {layout_name_literal},
                                        observed_layout: layoutType,
                                        chart_count: count,
                                        active_index: activeIndex,
                                        panes: panes
                                    }});
                                }} catch(e) {{
                                    resolve({{ error: e && e.message ? e.message : String(e) }});
                                }}
                            }}, 500);
                        }});
                    }} catch(e) {{
                        return {{ error: e && e.message ? e.message : String(e) }};
                    }}
                }})()
                "#
            ),
            true,
        )
        .await?;

    if let Some(message) = result.get("error").and_then(Value::as_str) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            message.to_string(),
        ));
    }

    Ok(result)
}

pub async fn pane_focus(
    runtime: &mut impl RuntimeEvaluator,
    index: usize,
) -> Result<Value, AppError> {
    let result = runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    try {{
                        var cwc = {CHART_WIDGET_COLLECTION};
                        var all = cwc && typeof cwc.getAll === "function" ? cwc.getAll() : [];
                        if ({index} >= all.length) {{
                            return {{ error: "Pane index {index} out of range", total: all.length }};
                        }}
                        var chart = all[{index}];
                        if (chart && chart._mainDiv && typeof chart._mainDiv.click === "function") {{
                            chart._mainDiv.click();
                        }}
                        return {{ focused: {index}, total: all.length }};
                    }} catch(e) {{
                        return {{ error: e && e.message ? e.message : String(e) }};
                    }}
                }})()
                "#
            ),
            false,
        )
        .await?;

    if let Some(message) = result.get("error").and_then(Value::as_str) {
        return Err(
            AppError::new(ErrorKind::InternalApiUnavailable, message.to_string()).with_details(
                json!({
                    "index": index,
                    "total_panes": result.get("total").cloned().unwrap_or(Value::Null),
                }),
            ),
        );
    }

    Ok(json!({
        "focused_index": result.get("focused").cloned().unwrap_or_else(|| json!(index)),
        "total_panes": result.get("total").cloned().unwrap_or(Value::Null),
    }))
}

pub async fn pane_symbol(
    runtime: &mut impl RuntimeEvaluator,
    index: usize,
    symbol: &str,
) -> Result<Value, AppError> {
    let focus = pane_focus(runtime, index).await?;
    runtime
        .evaluate(
            "new Promise(function(resolve) { setTimeout(function() { resolve(true); }, 300); })",
            true,
        )
        .await?;

    let symbol_literal = js_string(symbol)?;
    let result = runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    try {{
                        var chart = {CHART_API};
                        if (!chart || typeof chart.setSymbol !== "function") {{
                            return {{ error: "Active chart setSymbol unavailable" }};
                        }}
                        chart.setSymbol({symbol_literal}, {{}});
                        return new Promise(function(resolve) {{
                            setTimeout(function() {{
                                resolve({{
                                    index: {index},
                                    symbol: {symbol_literal},
                                    requested_symbol: {symbol_literal},
                                    source: "active_chart_after_focus"
                                }});
                            }}, 500);
                        }});
                    }} catch(e) {{
                        return {{ error: e && e.message ? e.message : String(e) }};
                    }}
                }})()
                "#
            ),
            true,
        )
        .await?;

    if let Some(message) = result.get("error").and_then(Value::as_str) {
        return Err(
            AppError::new(ErrorKind::InternalApiUnavailable, message.to_string()).with_details(
                json!({
                    "index": index,
                    "symbol": symbol,
                }),
            ),
        );
    }

    Ok(json!({
        "index": result.get("index").cloned().unwrap_or_else(|| json!(index)),
        "symbol": result.get("symbol").cloned().unwrap_or_else(|| json!(symbol)),
        "requested_symbol": result.get("requested_symbol").cloned().unwrap_or_else(|| json!(symbol)),
        "source": result.get("source").cloned().unwrap_or_else(|| json!("active_chart_after_focus")),
        "focused_index": focus.get("focused_index").cloned().unwrap_or_else(|| json!(index)),
        "total_panes": focus.get("total_panes").cloned().unwrap_or(Value::Null),
    }))
}

#[cfg(test)]
mod tests {
    use crate::cdp::KeyEventType;
    use serde_json::json;

    use super::super::test_support::FakeRuntime;
    use super::*;

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
    async fn watchlist_add_opens_panel_clicks_add_and_sends_input() {
        let mut runtime = FakeRuntime::new([
            json!({"opened": true}),
            json!(true),
            json!({"found": true, "selector": "[data-name=\"add-symbol-button\"]"}),
            json!(true),
            json!(true),
            json!(true),
        ]);

        let result = watchlist_add(&mut runtime, "NASDAQ:AAPL").await.unwrap();

        assert_eq!(result["symbol"], "NASDAQ:AAPL");
        assert_eq!(result["action"], "added");
        assert_eq!(runtime.inserted_text, vec!["NASDAQ:AAPL"]);
        assert_eq!(runtime.key_events.len(), 4);
        assert_eq!(runtime.key_events[0].event_type, KeyEventType::KeyDown);
        assert_eq!(runtime.key_events[0].key, "Enter");
        assert_eq!(runtime.key_events[1].event_type, KeyEventType::KeyUp);
        assert_eq!(runtime.key_events[2].key, "Escape");
        assert!(
            runtime.evaluated[0]
                .0
                .contains("base-watchlist-widget-button")
        );
        assert!(runtime.evaluated[2].0.contains("add-symbol-button"));
    }

    #[tokio::test]
    async fn watchlist_add_continues_when_watchlist_rows_are_already_visible() {
        let mut runtime = FakeRuntime::new([
            json!({"opened": false, "already_open": true, "source": "visible_watchlist_rows"}),
            json!({"found": true, "method": "fallback"}),
            json!(true),
            json!(true),
            json!(true),
        ]);

        let result = watchlist_add(&mut runtime, "NASDAQ:AAPL").await.unwrap();

        assert_eq!(result["symbol"], "NASDAQ:AAPL");
        assert_eq!(result["opened_panel"], false);
        assert_eq!(runtime.inserted_text, vec!["NASDAQ:AAPL"]);
        assert_eq!(runtime.key_events.len(), 4);
    }

    #[tokio::test]
    async fn watchlist_add_maps_missing_watchlist_ui_to_internal_api_error() {
        let mut runtime = FakeRuntime::new([json!({"error": "Watchlist button not found"})]);

        let err = watchlist_add(&mut runtime, "NASDAQ:AAPL")
            .await
            .unwrap_err();

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
        assert!(runtime.inserted_text.is_empty());
    }

    #[tokio::test]
    async fn watchlist_add_maps_missing_add_button_to_internal_api_error() {
        let mut runtime = FakeRuntime::new([json!({"opened": false}), json!({"found": false})]);

        let err = watchlist_add(&mut runtime, "NASDAQ:AAPL")
            .await
            .unwrap_err();

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
        assert!(runtime.inserted_text.is_empty());
    }

    #[tokio::test]
    async fn pane_list_returns_runtime_payload() {
        let payload = json!({
            "layout": "single",
            "layout_name": "1 chart",
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
    fn validate_pane_layout_accepts_aliases() {
        assert!(validate_pane_layout("2x2").is_ok());
        assert_eq!(parse_pane_layout("2x2").unwrap().code, "4");
        assert_eq!(parse_pane_layout(" single ").unwrap().code, "s");
    }

    #[test]
    fn validate_pane_layout_rejects_unknown_layout() {
        let err = validate_pane_layout("banana").unwrap_err();

        assert_eq!(err.kind, ErrorKind::Validation);
        assert!(
            err.details
                .as_ref()
                .and_then(|details| details.get("supported"))
                .and_then(Value::as_array)
                .is_some_and(|supported| supported.contains(&json!({
                    "layout": "4",
                    "layout_name": "2x2 grid"
                })))
        );
    }

    #[tokio::test]
    async fn pane_layout_sets_canonical_layout_and_returns_runtime_payload() {
        let mut runtime = FakeRuntime::new([json!({
            "layout": "4",
            "layout_name": "2x2 grid",
            "observed_layout": "4",
            "chart_count": 4,
            "active_index": 0,
            "panes": []
        })]);

        let result = pane_layout(&mut runtime, "2x2").await.unwrap();

        assert_eq!(result["layout"], "4");
        assert_eq!(result["layout_name"], "2x2 grid");
        assert!(runtime.evaluated[0].0.contains("setLayout(\"4\")"));
        assert!(runtime.evaluated[0].1);
    }

    #[tokio::test]
    async fn pane_layout_maps_runtime_error_to_internal_api_error() {
        let mut runtime = FakeRuntime::new([json!({
            "error": "Chart widget collection setLayout unavailable"
        })]);

        let err = pane_layout(&mut runtime, "s").await.unwrap_err();

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
    }

    #[tokio::test]
    async fn pane_focus_returns_practical_old_cli_fields() {
        let mut runtime = FakeRuntime::new([json!({"focused": 1, "total": 2})]);

        let result = pane_focus(&mut runtime, 1).await.unwrap();

        assert_eq!(result["focused_index"], 1);
        assert_eq!(result["total_panes"], 2);
        assert!(runtime.evaluated[0].0.contains("_mainDiv.click"));
    }

    #[tokio::test]
    async fn pane_focus_maps_range_error_to_internal_api_error() {
        let mut runtime = FakeRuntime::new([json!({
            "error": "Pane index 3 out of range",
            "total": 1
        })]);

        let err = pane_focus(&mut runtime, 3).await.unwrap_err();

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(err.details.unwrap()["total_panes"], 1);
    }

    #[tokio::test]
    async fn pane_symbol_focuses_then_sets_symbol() {
        let mut runtime = FakeRuntime::new([
            json!({"focused": 1, "total": 2}),
            json!(true),
            json!({
                "index": 1,
                "symbol": "NASDAQ:AAPL",
                "requested_symbol": "NASDAQ:AAPL",
                "source": "active_chart_after_focus"
            }),
        ]);

        let result = pane_symbol(&mut runtime, 1, "NASDAQ:AAPL").await.unwrap();

        assert_eq!(result["index"], 1);
        assert_eq!(result["symbol"], "NASDAQ:AAPL");
        assert_eq!(result["focused_index"], 1);
        assert_eq!(result["total_panes"], 2);
        assert!(runtime.evaluated[0].0.contains("_mainDiv.click"));
        assert!(runtime.evaluated[2].0.contains("setSymbol(\"NASDAQ:AAPL\""));
    }

    #[tokio::test]
    async fn pane_symbol_serializes_symbol_as_js_string() {
        let mut runtime = FakeRuntime::new([
            json!({"focused": 0, "total": 1}),
            json!(true),
            json!({"index": 0, "symbol": "NYSE:BRK\"B"}),
        ]);

        pane_symbol(&mut runtime, 0, "NYSE:BRK\"B").await.unwrap();

        assert!(
            runtime.evaluated[2]
                .0
                .contains("setSymbol(\"NYSE:BRK\\\"B\"")
        );
    }
}
