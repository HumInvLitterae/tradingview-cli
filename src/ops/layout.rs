use serde_json::{Value, json};

use crate::{
    cdp::{KeyEvent, KeyEventType, RuntimeEvaluator},
    error::{AppError, ErrorKind},
};

use super::common::{CHART_API, CHART_WIDGET_COLLECTION};

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
    async fn watchlist_add_maps_missing_watchlist_button_to_internal_api_error() {
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
}
