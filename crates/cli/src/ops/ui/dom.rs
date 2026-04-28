use serde_json::{Value, json};

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};

use super::super::common::js_string;
use super::selectors::validate_selector_strategy;

pub async fn ui_click(
    runtime: &mut impl RuntimeEvaluator,
    by: &str,
    value: &str,
) -> Result<Value, AppError> {
    validate_selector_strategy(by, &["text", "aria-label", "data-name", "class-contains"])?;
    if value.trim().is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Value must not be empty",
        ));
    }
    let by_literal = js_string(by)?;
    let value_literal = js_string(value)?;
    let result = runtime
        .evaluate(
            &format!(
                r#"
            (function() {{
                var by = {by_literal};
                var value = {value_literal};
                var element = null;
                function visible(el) {{
                    var rect = el.getBoundingClientRect();
                    return rect.width > 0 && rect.height > 0;
                }}
                function textOf(el) {{
                    return (el.textContent || el.innerText || '').trim();
                }}
                if (by === 'aria-label') {{
                    element = document.querySelector('[aria-label="' + CSS.escape(value) + '"]');
                }} else if (by === 'data-name') {{
                    element = document.querySelector('[data-name="' + CSS.escape(value) + '"]');
                }} else if (by === 'class-contains') {{
                    element = document.querySelector('[class*="' + value.replace(/"/g, '\\\\"') + '"]');
                }} else {{
                    var candidates = Array.from(document.querySelectorAll('button, a, [role="button"], [role="menuitem"], [role="tab"]'));
                    for (var i = 0; i < candidates.length; i++) {{
                        var text = textOf(candidates[i]);
                        if (visible(candidates[i]) && (text === value || text.toLowerCase() === value.toLowerCase())) {{
                            element = candidates[i];
                            break;
                        }}
                    }}
                }}
                if (!element) return {{ found: false, by: by, value: value }};
                element.click();
                return {{
                    found: true,
                    by: by,
                    value: value,
                    tag: element.tagName.toLowerCase(),
                    text: textOf(element).substring(0, 80),
                    aria_label: element.getAttribute('aria-label') || null,
                    data_name: element.getAttribute('data-name') || null
                }};
            }})()
            "#
            ),
            false,
        )
        .await?;

    if !result
        .get("found")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Err(AppError::new(
            ErrorKind::Validation,
            format!("No matching element found for {by}=\"{value}\""),
        )
        .with_details(result));
    }

    Ok(json!({ "clicked": result }))
}

pub async fn ui_find(
    runtime: &mut impl RuntimeEvaluator,
    query: &str,
    strategy: Option<&str>,
) -> Result<Value, AppError> {
    let query = query.trim();
    if query.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Query must not be empty",
        ));
    }
    let strategy = strategy.unwrap_or("text").trim();
    validate_selector_strategy(strategy, &["text", "aria-label", "css"])?;
    let query_literal = js_string(query)?;
    let strategy_literal = js_string(strategy)?;
    let results = runtime
        .evaluate(
            &format!(
                r#"
            (function() {{
                var query = {query_literal};
                var strategy = {strategy_literal};
                var results = [];
                function pushElement(el) {{
                    var rect = el.getBoundingClientRect();
                    results.push({{
                        tag: el.tagName.toLowerCase(),
                        text: (el.textContent || '').trim().substring(0, 80),
                        aria_label: el.getAttribute('aria-label') || null,
                        data_name: el.getAttribute('data-name') || null,
                        x: rect.x,
                        y: rect.y,
                        width: rect.width,
                        height: rect.height,
                        visible: rect.width > 0 && rect.height > 0 && el.offsetParent !== null
                    }});
                }}
                if (strategy === 'css') {{
                    var cssElements = document.querySelectorAll(query);
                    for (var i = 0; i < Math.min(cssElements.length, 20); i++) pushElement(cssElements[i]);
                }} else if (strategy === 'aria-label') {{
                    var ariaElements = document.querySelectorAll('[aria-label*="' + query.replace(/"/g, '\\\\"') + '"]');
                    for (var j = 0; j < Math.min(ariaElements.length, 20); j++) pushElement(ariaElements[j]);
                }} else {{
                    var all = document.querySelectorAll('button, a, [role="button"], [role="menuitem"], [role="tab"], input, select, label, span, div, h1, h2, h3, h4');
                    for (var k = 0; k < all.length; k++) {{
                        var text = (all[k].textContent || '').trim();
                        if (text.toLowerCase().indexOf(query.toLowerCase()) !== -1 && text.length < 200) {{
                            var rect = all[k].getBoundingClientRect();
                            if (rect.width > 0 && rect.height > 0) {{
                                pushElement(all[k]);
                                if (results.length >= 20) break;
                            }}
                        }}
                    }}
                }}
                return results;
            }})()
            "#
            ),
            false,
        )
        .await?;
    let elements = results.as_array().cloned().unwrap_or_default();
    Ok(json!({
        "query": query,
        "strategy": strategy,
        "count": elements.len(),
        "elements": elements,
    }))
}

pub async fn ui_panel(
    runtime: &mut impl RuntimeEvaluator,
    panel: &str,
    action: &str,
) -> Result<Value, AppError> {
    let panel = panel.trim();
    let action = action.trim();
    if panel.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Panel must not be empty",
        ));
    }
    if !matches!(action, "open" | "close" | "toggle") {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Panel action must be one of: open, close, toggle",
        ));
    }
    let panel_literal = js_string(panel)?;
    let action_literal = js_string(action)?;
    let result = runtime
        .evaluate(
            &format!(
                r#"
            (function() {{
                var panel = {panel_literal};
                var action = {action_literal};
                var bottomPanels = {{
                    'pine-editor': 'pine-editor',
                    'strategy-tester': 'backtesting'
                }};
                var rightPanels = {{
                    'watchlist': {{ dataName: 'base-watchlist-widget-button', ariaLabel: 'Watchlist' }},
                    'alerts': {{ dataName: 'alerts-button', ariaLabel: 'Alerts' }},
                    'trading': {{ dataName: 'trading-button', ariaLabel: 'Trading Panel' }}
                }};

                if (bottomPanels[panel]) {{
                    var bwb = window.TradingView && window.TradingView.bottomWidgetBar;
                    if (!bwb) return {{ error: 'bottomWidgetBar not available' }};
                    var widgetName = bottomPanels[panel];
                    var bottomArea = document.querySelector('[class*="layout__area--bottom"]');
                    var isOpen = !!(bottomArea && bottomArea.offsetHeight > 50);
                    if (panel === 'pine-editor') isOpen = isOpen && !!document.querySelector('.monaco-editor.pine-editor-monaco');
                    var performed = 'none';
                    if (action === 'open' || (action === 'toggle' && !isOpen)) {{
                        if (panel === 'pine-editor' && typeof bwb.activateScriptEditorTab === 'function') bwb.activateScriptEditorTab();
                        else if (typeof bwb.showWidget === 'function') bwb.showWidget(widgetName);
                        performed = 'opened';
                    }} else if (action === 'close' || (action === 'toggle' && isOpen)) {{
                        if (typeof bwb.hideWidget === 'function') bwb.hideWidget(widgetName);
                        performed = 'closed';
                    }} else {{
                        performed = isOpen ? 'already_open' : 'already_closed';
                    }}
                    return {{ panel: panel, action: action, was_open: isOpen, performed: performed }};
                }}

                var selector = rightPanels[panel];
                if (!selector) return {{ error: 'Unsupported panel: ' + panel }};
                var button = document.querySelector('[data-name="' + selector.dataName + '"]')
                    || document.querySelector('[aria-label="' + selector.ariaLabel + '"]');
                if (!button) return {{ error: 'Button not found for panel: ' + panel }};
                var rightArea = document.querySelector('[class*="layout__area--right"]');
                var sidebarOpen = !!(rightArea && rightArea.offsetWidth > 50);
                var isActive = button.getAttribute('aria-pressed') === 'true'
                    || String(button.className || '').indexOf('active') !== -1
                    || String(button.className || '').indexOf('Active') !== -1;
                var isOpen = isActive && sidebarOpen;
                var performed = 'none';
                if (action === 'open' && !isOpen) {{ button.click(); performed = 'opened'; }}
                else if (action === 'close' && isOpen) {{ button.click(); performed = 'closed'; }}
                else if (action === 'toggle') {{ button.click(); performed = isOpen ? 'closed' : 'opened'; }}
                else {{ performed = isOpen ? 'already_open' : 'already_closed'; }}
                return {{ panel: panel, action: action, was_open: isOpen, performed: performed }};
            }})()
            "#
            ),
            false,
        )
        .await?;
    if let Some(error) = result.get("error").and_then(Value::as_str) {
        return Err(AppError::new(ErrorKind::InternalApiUnavailable, error).with_details(result));
    }
    Ok(result)
}

pub async fn ui_fullscreen(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    let result = runtime
        .evaluate(
            r#"
            (function() {
                var button = document.querySelector('[data-name="header-toolbar-fullscreen"]');
                if (!button) return { found: false };
                button.click();
                return { found: true, action: 'fullscreen_toggled' };
            })()
            "#,
            false,
        )
        .await?;
    if !result
        .get("found")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Fullscreen button not found",
        )
        .with_details(result));
    }
    Ok(json!({ "action": "fullscreen_toggled" }))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::super::test_support::FakeRuntime;
    use super::*;

    #[tokio::test]
    async fn ui_click_returns_clicked_element() {
        let mut runtime = FakeRuntime::new([json!({
            "found": true,
            "by": "text",
            "value": "Indicators",
            "tag": "button",
            "text": "Indicators",
            "aria_label": "Indicators",
            "data_name": null
        })]);

        let result = ui_click(&mut runtime, "text", "Indicators").await.unwrap();

        assert_eq!(result["clicked"]["text"], "Indicators");
        assert!(runtime.evaluated[0].0.contains("document.querySelectorAll"));
    }

    #[tokio::test]
    async fn ui_click_maps_missing_element_to_validation() {
        let mut runtime = FakeRuntime::new([json!({"found": false})]);

        let error = ui_click(&mut runtime, "text", "Missing").await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
    }

    #[tokio::test]
    async fn ui_find_returns_elements() {
        let mut runtime = FakeRuntime::new([json!([
            {"tag": "button", "text": "Indicators", "visible": true}
        ])]);

        let result = ui_find(&mut runtime, "Indicators", Some("text"))
            .await
            .unwrap();

        assert_eq!(result["count"], 1);
        assert_eq!(result["elements"][0]["text"], "Indicators");
    }

    #[tokio::test]
    async fn ui_panel_returns_panel_action() {
        let mut runtime = FakeRuntime::new([json!({
            "panel": "watchlist",
            "action": "open",
            "was_open": false,
            "performed": "opened"
        })]);

        let result = ui_panel(&mut runtime, "watchlist", "open").await.unwrap();

        assert_eq!(result["performed"], "opened");
    }

    #[tokio::test]
    async fn ui_fullscreen_requires_button() {
        let mut runtime = FakeRuntime::new([json!({"found": false})]);

        let error = ui_fullscreen(&mut runtime).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
    }
}
