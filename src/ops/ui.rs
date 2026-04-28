use serde_json::{Value, json};

use crate::cdp::{KeyEvent, KeyEventType, MouseEvent, MouseEventType, RuntimeEvaluator};
use tradingview_core::{AppError, ErrorKind};

use super::common::{js_string, require_finite};

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

pub async fn ui_keyboard(
    runtime: &mut impl RuntimeEvaluator,
    key: &str,
    ctrl: bool,
    shift: bool,
    alt: bool,
    meta: bool,
) -> Result<Value, AppError> {
    let mapping = key_mapping(key)?;
    let modifiers = modifier_mask(ctrl, shift, alt, meta);
    runtime
        .dispatch_key_event(KeyEvent {
            event_type: KeyEventType::KeyDown,
            key: mapping.key,
            code: mapping.code,
            windows_virtual_key_code: mapping.windows_virtual_key_code,
            modifiers,
        })
        .await?;
    runtime
        .dispatch_key_event(KeyEvent {
            event_type: KeyEventType::KeyUp,
            key: mapping.key,
            code: mapping.code,
            windows_virtual_key_code: mapping.windows_virtual_key_code,
            modifiers: 0,
        })
        .await?;
    Ok(json!({
        "key": mapping.key,
        "modifiers": modifier_names(ctrl, shift, alt, meta),
    }))
}

pub async fn ui_type(runtime: &mut impl RuntimeEvaluator, text: &str) -> Result<Value, AppError> {
    runtime.insert_text(text).await?;
    Ok(json!({
        "typed": text.chars().take(100).collect::<String>(),
        "length": text.chars().count(),
    }))
}

pub async fn ui_hover(
    runtime: &mut impl RuntimeEvaluator,
    by: &str,
    value: &str,
) -> Result<Value, AppError> {
    let coords = ui_element_coordinates(runtime, by, value).await?;
    runtime
        .dispatch_mouse_event(MouseEvent {
            event_type: MouseEventType::Moved,
            x: coords.x,
            y: coords.y,
            button: None,
            buttons: None,
            click_count: None,
            delta_x: None,
            delta_y: None,
        })
        .await?;
    Ok(json!({
        "hovered": {
            "by": by,
            "value": value,
            "tag": coords.tag,
            "x": coords.x,
            "y": coords.y
        }
    }))
}

pub async fn ui_scroll(
    runtime: &mut impl RuntimeEvaluator,
    direction: &str,
    amount: Option<f64>,
) -> Result<Value, AppError> {
    let direction = direction.trim().to_ascii_lowercase();
    if !matches!(direction.as_str(), "up" | "down" | "left" | "right") {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Scroll direction must be one of: up, down, left, right",
        ));
    }
    let amount = amount.unwrap_or(300.0);
    require_finite(amount, "amount")?;
    let center = runtime
        .evaluate(
            r#"
            (function() {
                var element = document.querySelector('[data-name="pane-canvas"]')
                    || document.querySelector('[class*="chart-container"]')
                    || document.querySelector('canvas');
                if (!element) return { x: window.innerWidth / 2, y: window.innerHeight / 2 };
                var rect = element.getBoundingClientRect();
                return { x: rect.x + rect.width / 2, y: rect.y + rect.height / 2 };
            })()
            "#,
            false,
        )
        .await?;
    let x = number_field(&center, "x")?;
    let y = number_field(&center, "y")?;
    let (delta_x, delta_y) = match direction.as_str() {
        "up" => (0.0, -amount),
        "down" => (0.0, amount),
        "left" => (-amount, 0.0),
        "right" => (amount, 0.0),
        _ => unreachable!("direction validated"),
    };
    runtime
        .dispatch_mouse_event(MouseEvent {
            event_type: MouseEventType::Wheel,
            x,
            y,
            button: None,
            buttons: None,
            click_count: None,
            delta_x: Some(delta_x),
            delta_y: Some(delta_y),
        })
        .await?;
    Ok(json!({
        "direction": direction,
        "amount": amount,
        "x": x,
        "y": y,
    }))
}

pub async fn ui_mouse(
    runtime: &mut impl RuntimeEvaluator,
    x: f64,
    y: f64,
    right: bool,
    double: bool,
) -> Result<Value, AppError> {
    require_finite(x, "x")?;
    require_finite(y, "y")?;
    let button = if right { "right" } else { "left" };
    let buttons = if right { 2 } else { 1 };
    dispatch_mouse_click(runtime, x, y, button, buttons, 1).await?;
    if double {
        dispatch_mouse_click(runtime, x, y, button, buttons, 2).await?;
    }
    Ok(json!({
        "x": x,
        "y": y,
        "button": button,
        "double_click": double,
    }))
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

pub async fn ui_eval(
    runtime: &mut impl RuntimeEvaluator,
    expression: &str,
) -> Result<Value, AppError> {
    let result = runtime.evaluate(expression, true).await?;
    Ok(json!({
        "result": result,
        "unsafe_eval_enabled": true,
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

async fn dispatch_mouse_click(
    runtime: &mut impl RuntimeEvaluator,
    x: f64,
    y: f64,
    button: &'static str,
    buttons: i64,
    click_count: i64,
) -> Result<(), AppError> {
    runtime
        .dispatch_mouse_event(MouseEvent {
            event_type: MouseEventType::Moved,
            x,
            y,
            button: None,
            buttons: None,
            click_count: None,
            delta_x: None,
            delta_y: None,
        })
        .await?;
    runtime
        .dispatch_mouse_event(MouseEvent {
            event_type: MouseEventType::Pressed,
            x,
            y,
            button: Some(button),
            buttons: Some(buttons),
            click_count: Some(click_count),
            delta_x: None,
            delta_y: None,
        })
        .await?;
    runtime
        .dispatch_mouse_event(MouseEvent {
            event_type: MouseEventType::Released,
            x,
            y,
            button: Some(button),
            buttons: Some(0),
            click_count: Some(click_count),
            delta_x: None,
            delta_y: None,
        })
        .await
}

async fn ui_element_coordinates(
    runtime: &mut impl RuntimeEvaluator,
    by: &str,
    value: &str,
) -> Result<ElementCoordinates, AppError> {
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
                function textOf(el) {{
                    return (el.textContent || el.innerText || '').trim();
                }}
                if (by === 'aria-label') element = document.querySelector('[aria-label="' + CSS.escape(value) + '"]') || document.querySelector('[aria-label*="' + value.replace(/"/g, '\\\\"') + '"]');
                else if (by === 'data-name') element = document.querySelector('[data-name="' + CSS.escape(value) + '"]');
                else if (by === 'class-contains') element = document.querySelector('[class*="' + value.replace(/"/g, '\\\\"') + '"]');
                else {{
                    var candidates = Array.from(document.querySelectorAll('button, a, [role="button"], [role="menuitem"], [role="tab"], span, div'));
                    for (var i = 0; i < candidates.length; i++) {{
                        var text = textOf(candidates[i]);
                        if (text === value || text.toLowerCase() === value.toLowerCase()) {{
                            element = candidates[i];
                            break;
                        }}
                    }}
                }}
                if (!element) return null;
                var rect = element.getBoundingClientRect();
                return {{
                    x: rect.x + rect.width / 2,
                    y: rect.y + rect.height / 2,
                    tag: element.tagName.toLowerCase()
                }};
            }})()
            "#
            ),
            false,
        )
        .await?;
    if result.is_null() {
        return Err(AppError::new(
            ErrorKind::Validation,
            format!("Element not found for {by}=\"{value}\""),
        ));
    }
    Ok(ElementCoordinates {
        x: number_field(&result, "x")?,
        y: number_field(&result, "y")?,
        tag: result
            .get("tag")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
    })
}

fn validate_selector_strategy(value: &str, allowed: &[&str]) -> Result<(), AppError> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorKind::Validation,
            format!("Unsupported selector strategy: {value}"),
        )
        .with_details(json!({ "supported": allowed })))
    }
}

fn number_field(value: &Value, field: &str) -> Result<f64, AppError> {
    let number = value.get(field).and_then(Value::as_f64).ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!("UI payload did not include numeric {field}"),
        )
        .with_details(value.clone())
    })?;
    require_finite(number, field)?;
    Ok(number)
}

fn modifier_mask(ctrl: bool, shift: bool, alt: bool, meta: bool) -> i64 {
    let mut mask = 0;
    if alt {
        mask |= 1;
    }
    if ctrl {
        mask |= 2;
    }
    if meta {
        mask |= 4;
    }
    if shift {
        mask |= 8;
    }
    mask
}

fn modifier_names(ctrl: bool, shift: bool, alt: bool, meta: bool) -> Vec<&'static str> {
    let mut names = Vec::new();
    if ctrl {
        names.push("ctrl");
    }
    if shift {
        names.push("shift");
    }
    if alt {
        names.push("alt");
    }
    if meta {
        names.push("meta");
    }
    names
}

struct ElementCoordinates {
    x: f64,
    y: f64,
    tag: String,
}

struct KeyMapping {
    key: &'static str,
    code: &'static str,
    windows_virtual_key_code: i64,
}

fn key_mapping(key: &str) -> Result<KeyMapping, AppError> {
    let normalized = key.trim();
    if normalized.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Key must not be empty",
        ));
    }
    let mapping = match normalized {
        "Enter" => ("Enter", "Enter", 13),
        "Escape" => ("Escape", "Escape", 27),
        "Tab" => ("Tab", "Tab", 9),
        "Backspace" => ("Backspace", "Backspace", 8),
        "Delete" => ("Delete", "Delete", 46),
        "ArrowUp" => ("ArrowUp", "ArrowUp", 38),
        "ArrowDown" => ("ArrowDown", "ArrowDown", 40),
        "ArrowLeft" => ("ArrowLeft", "ArrowLeft", 37),
        "ArrowRight" => ("ArrowRight", "ArrowRight", 39),
        "Space" => ("Space", "Space", 32),
        "Home" => ("Home", "Home", 36),
        "End" => ("End", "End", 35),
        "PageUp" => ("PageUp", "PageUp", 33),
        "PageDown" => ("PageDown", "PageDown", 34),
        "F1" => ("F1", "F1", 112),
        "F2" => ("F2", "F2", 113),
        "F5" => ("F5", "F5", 116),
        "a" | "A" => ("a", "KeyA", 65),
        "b" | "B" => ("b", "KeyB", 66),
        "c" | "C" => ("c", "KeyC", 67),
        "d" | "D" => ("d", "KeyD", 68),
        "e" | "E" => ("e", "KeyE", 69),
        "f" | "F" => ("f", "KeyF", 70),
        "g" | "G" => ("g", "KeyG", 71),
        "h" | "H" => ("h", "KeyH", 72),
        "i" | "I" => ("i", "KeyI", 73),
        "j" | "J" => ("j", "KeyJ", 74),
        "k" | "K" => ("k", "KeyK", 75),
        "l" | "L" => ("l", "KeyL", 76),
        "m" | "M" => ("m", "KeyM", 77),
        "n" | "N" => ("n", "KeyN", 78),
        "o" | "O" => ("o", "KeyO", 79),
        "p" | "P" => ("p", "KeyP", 80),
        "q" | "Q" => ("q", "KeyQ", 81),
        "r" | "R" => ("r", "KeyR", 82),
        "s" | "S" => ("s", "KeyS", 83),
        "t" | "T" => ("t", "KeyT", 84),
        "u" | "U" => ("u", "KeyU", 85),
        "v" | "V" => ("v", "KeyV", 86),
        "w" | "W" => ("w", "KeyW", 87),
        "x" | "X" => ("x", "KeyX", 88),
        "y" | "Y" => ("y", "KeyY", 89),
        "z" | "Z" => ("z", "KeyZ", 90),
        _ => {
            return Err(AppError::new(
                ErrorKind::Validation,
                format!("Unsupported key: {normalized}"),
            ));
        }
    };
    Ok(KeyMapping {
        key: mapping.0,
        code: mapping.1,
        windows_virtual_key_code: mapping.2,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::test_support::FakeRuntime;
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
    async fn ui_keyboard_dispatches_key_events() {
        let mut runtime = FakeRuntime::new([]);

        let result = ui_keyboard(&mut runtime, "Escape", true, false, false, false)
            .await
            .unwrap();

        assert_eq!(result["key"], "Escape");
        assert_eq!(runtime.key_events.len(), 2);
        assert_eq!(runtime.key_events[0].event_type, KeyEventType::KeyDown);
        assert_eq!(runtime.key_events[0].modifiers, 2);
        assert_eq!(runtime.key_events[1].event_type, KeyEventType::KeyUp);
    }

    #[tokio::test]
    async fn ui_type_inserts_text() {
        let mut runtime = FakeRuntime::new([]);

        let result = ui_type(&mut runtime, "hello").await.unwrap();

        assert_eq!(result["length"], 5);
        assert_eq!(runtime.inserted_text, vec!["hello"]);
    }

    #[tokio::test]
    async fn ui_hover_moves_to_element_center() {
        let mut runtime = FakeRuntime::new([json!({"x": 10.0, "y": 20.0, "tag": "button"})]);

        let result = ui_hover(&mut runtime, "text", "Alerts").await.unwrap();

        assert_eq!(result["hovered"]["x"], 10.0);
        assert_eq!(runtime.mouse_events.len(), 1);
        assert_eq!(runtime.mouse_events[0].event_type, MouseEventType::Moved);
    }

    #[tokio::test]
    async fn ui_scroll_dispatches_wheel_event() {
        let mut runtime = FakeRuntime::new([json!({"x": 100.0, "y": 200.0})]);

        let result = ui_scroll(&mut runtime, "up", Some(150.0)).await.unwrap();

        assert_eq!(result["direction"], "up");
        assert_eq!(runtime.mouse_events.len(), 1);
        assert_eq!(runtime.mouse_events[0].event_type, MouseEventType::Wheel);
        assert_eq!(runtime.mouse_events[0].delta_y, Some(-150.0));
    }

    #[tokio::test]
    async fn ui_mouse_dispatches_click_events() {
        let mut runtime = FakeRuntime::new([]);

        let result = ui_mouse(&mut runtime, 1.0, 2.0, true, false).await.unwrap();

        assert_eq!(result["button"], "right");
        assert_eq!(runtime.mouse_events.len(), 3);
        assert_eq!(runtime.mouse_events[1].button, Some("right"));
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
    async fn ui_eval_returns_runtime_result() {
        let mut runtime = FakeRuntime::new([json!(2)]);

        let result = ui_eval(&mut runtime, "1+1").await.unwrap();

        assert_eq!(result["result"], 2);
        assert_eq!(result["unsafe_eval_enabled"], true);
        assert_eq!(runtime.evaluated[0].0, "1+1");
        assert!(runtime.evaluated[0].1);
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
