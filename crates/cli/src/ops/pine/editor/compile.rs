use serde_json::{Value, json};

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};

use super::runtime::{
    PINE_COMPILE_WAIT, dispatch_ctrl_enter, ensure_pine_editor_open, normalize_array,
    normalize_button_text, with_monaco,
};

pub async fn pine_compile(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    let open_state = ensure_pine_editor_open(runtime).await?;
    let studies_before = pine_study_count(runtime).await?;
    let button_result = runtime
        .evaluate(PINE_COMPILE_BUTTON_EXPRESSION, false)
        .await?;

    if button_result
        .get("blocked_save")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Pine compile found only a save-related action button",
        )
        .with_details(button_result));
    }

    let clicked = button_result
        .get("clicked")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let button_clicked = button_result
        .get("button_text")
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .map(normalize_button_text);

    let action = if clicked {
        button_clicked.unwrap_or_else(|| "compile_button".to_string())
    } else {
        dispatch_ctrl_enter(runtime).await?;
        "keyboard_shortcut".to_string()
    };

    tokio::time::sleep(PINE_COMPILE_WAIT).await;

    let errors = runtime
        .evaluate(&with_monaco(PINE_ERRORS_EXPRESSION), false)
        .await?;
    let errors = normalize_array(errors, "Pine marker payload was not an array")?;
    let studies_after = pine_study_count(runtime).await?;
    let study_added = match (studies_before, studies_after) {
        (Some(before), Some(after)) => Some(after > before),
        _ => None,
    };

    Ok(json!({
        "button_clicked": action,
        "has_errors": !errors.is_empty(),
        "error_count": errors.len(),
        "errors": errors,
        "study_added": study_added,
        "studies_before": studies_before,
        "studies_after": studies_after,
        "editor_open_before": open_state.editor_open_before,
        "opened_editor": open_state.opened_editor,
    }))
}

pub async fn pine_raw_compile(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    let open_state = ensure_pine_editor_open(runtime).await?;
    let button_result = runtime
        .evaluate(PINE_RAW_COMPILE_BUTTON_EXPRESSION, false)
        .await?;
    let clicked = button_result
        .get("clicked")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let button_clicked = button_result
        .get("button_text")
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .map(normalize_button_text);

    let action = if clicked {
        button_clicked.unwrap_or_else(|| "raw_compile_button".to_string())
    } else {
        dispatch_ctrl_enter(runtime).await?;
        "keyboard_shortcut".to_string()
    };

    tokio::time::sleep(PINE_COMPILE_WAIT).await;

    Ok(json!({
        "button_clicked": action,
        "source": "dom_fallback",
        "editor_open_before": open_state.editor_open_before,
        "opened_editor": open_state.opened_editor,
        "raw_compile": true,
    }))
}

pub async fn pine_errors(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    let open_state = ensure_pine_editor_open(runtime).await?;
    let errors = runtime
        .evaluate(&with_monaco(PINE_ERRORS_EXPRESSION), false)
        .await?;
    let errors = normalize_array(errors, "Pine marker payload was not an array")?;

    Ok(json!({
        "has_errors": !errors.is_empty(),
        "error_count": errors.len(),
        "errors": errors,
        "editor_open_before": open_state.editor_open_before,
        "opened_editor": open_state.opened_editor,
    }))
}

pub async fn pine_console(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    let open_state = ensure_pine_editor_open(runtime).await?;
    let entries = runtime.evaluate(PINE_CONSOLE_EXPRESSION, false).await?;
    let entries = normalize_array(entries, "Pine console payload was not an array")?;

    Ok(json!({
        "entries": entries,
        "entry_count": entries.len(),
        "editor_open_before": open_state.editor_open_before,
        "opened_editor": open_state.opened_editor,
    }))
}

async fn pine_study_count(runtime: &mut impl RuntimeEvaluator) -> Result<Option<i64>, AppError> {
    let value = runtime.evaluate(PINE_STUDY_COUNT_EXPRESSION, false).await?;
    Ok(value.as_i64())
}

const PINE_ERRORS_EXPRESSION: &str = r#"
var m = __FIND_MONACO__;
if (!m) return [];
var model = m.editor.getModel();
if (!model) return [];
var markers = m.env.editor.getModelMarkers({ resource: model.uri });
return markers.map(function(mk) {
    return {
        line: mk.startLineNumber,
        column: mk.startColumn,
        message: mk.message,
        severity: mk.severity
    };
});
"#;

const PINE_STUDY_COUNT_EXPRESSION: &str = r#"
(function() {
    try {
        var chart = window.TradingViewApi && window.TradingViewApi._activeChartWidgetWV && window.TradingViewApi._activeChartWidgetWV.value();
        if (chart && typeof chart.getAllStudies === 'function') return chart.getAllStudies().length;
    } catch(e) {}
    return null;
})()
"#;

const PINE_COMPILE_BUTTON_EXPRESSION: &str = r#"
(function() {
    var editor = document.querySelector('.monaco-editor.pine-editor-monaco');
    var scope = editor && (
        editor.closest('[data-name="pine-dialog"]')
        || editor.closest('[class*="dialog"]')
        || editor.closest('[class*="pine"]')
    );
    if (!scope) scope = document.querySelector('[data-name="pine-dialog"]') || document;
    var buttons = Array.from(scope.querySelectorAll('button'));
    var saveCandidate = null;
    var compileCandidate = null;

    function visible(button) {
        var rect = button.getBoundingClientRect();
        return rect.width > 0 && rect.height > 0 && button.offsetParent !== null;
    }
    function label(button) {
        return (button.textContent || button.getAttribute('aria-label') || button.getAttribute('title') || '').trim();
    }
    function isSaveAction(text) {
        return /save/i.test(text) || /保存/.test(text);
    }
    function isCompileAction(text) {
        if (/^(Add to chart|Update on chart)$/i.test(text)) return true;
        if (/チャート/.test(text) && /(追加|更新)/.test(text)) return true;
        if (/^(차트에 넣기|차트 업데이트)$/.test(text)) return true;
        return false;
    }

    for (var i = 0; i < buttons.length; i++) {
        var text = label(buttons[i]);
        if (!text || !visible(buttons[i])) continue;
        if (isCompileAction(text) && isSaveAction(text)) {
            if (!saveCandidate) saveCandidate = { button: buttons[i], text: text };
            continue;
        }
        if (isCompileAction(text)) {
            compileCandidate = { button: buttons[i], text: text };
            break;
        }
        if (!saveCandidate && isSaveAction(text) && /chart|チャート|차트/.test(text)) {
            saveCandidate = { button: buttons[i], text: text };
        }
    }

    if (compileCandidate) {
        compileCandidate.button.click();
        return { clicked: true, button_text: compileCandidate.text, blocked_save: false };
    }
    if (saveCandidate) {
        return { clicked: false, button_text: saveCandidate.text, blocked_save: true };
    }
    return { clicked: false, button_text: null, blocked_save: false };
})()
"#;

const PINE_RAW_COMPILE_BUTTON_EXPRESSION: &str = r#"
(function() {
    var buttons = Array.from(document.querySelectorAll('button'));
    var fallback = null;
    var saveButton = null;

    function visible(button) {
        var rect = button.getBoundingClientRect();
        return rect.width > 0 && rect.height > 0 && button.offsetParent !== null;
    }
    function label(button) {
        return (button.textContent || button.getAttribute('aria-label') || button.getAttribute('title') || '').trim();
    }

    for (var i = 0; i < buttons.length; i++) {
        if (!visible(buttons[i])) continue;
        var text = label(buttons[i]);
        if (/save and add to chart/i.test(text) || (/保存/.test(text) && /チャート/.test(text))) {
            buttons[i].click();
            return { clicked: true, button_text: text || 'Save and add to chart' };
        }
        if (!fallback && /^(Add to chart|Update on chart)/i.test(text)) {
            fallback = buttons[i];
        }
        if (!fallback && /チャート/.test(text) && /(追加|更新)/.test(text)) {
            fallback = buttons[i];
        }
        if (!fallback && /^(차트에 넣기|차트 업데이트)$/.test(text)) {
            fallback = buttons[i];
        }
        if (!saveButton && String(buttons[i].className || '').indexOf('saveButton') !== -1) {
            saveButton = buttons[i];
        }
    }

    if (fallback) {
        fallback.click();
        return { clicked: true, button_text: label(fallback) || 'Add to chart' };
    }
    if (saveButton) {
        saveButton.click();
        return { clicked: true, button_text: 'Pine Save' };
    }
    return { clicked: false, button_text: null };
})()
"#;

const PINE_CONSOLE_EXPRESSION: &str = r#"
(function() {
    var results = [];
    var rows = document.querySelectorAll('[class*="consoleRow"], [class*="log-"], [class*="consoleLine"]');
    if (rows.length === 0) {
        var bottomArea = document.querySelector('[class*="layout__area--bottom"]')
            || document.querySelector('[class*="bottom-widgetbar-content"]');
        if (bottomArea) {
            rows = bottomArea.querySelectorAll('[class*="message"], [class*="log"], [class*="console"]');
        }
    }
    if (rows.length === 0) {
        var pinePanel = document.querySelector('.pine-editor-container')
            || document.querySelector('[class*="pine-editor"]')
            || document.querySelector('[class*="layout__area--bottom"]');
        if (pinePanel) {
            rows = Array.from(rows || []);
            var allSpans = pinePanel.querySelectorAll('span, div');
            for (var s = 0; s < allSpans.length; s++) {
                var txt = allSpans[s].textContent.trim();
                var cls = allSpans[s].className || '';
                var looksLikeSource = /\/\/@version|Pine Script® code is subject|indicator\(|strategy\(|library\(/.test(txt);
                if (!looksLikeSource && txt.length < 500 && (/^\d{2}:\d{2}:\d{2}/.test(txt) || /error|warning|info/i.test(cls))) {
                    rows.push(allSpans[s]);
                }
            }
        }
    }
    for (var i = 0; i < rows.length; i++) {
        var text = rows[i].textContent.trim();
        if (!text) continue;
        if (/\/\/@version|Pine Script® code is subject|indicator\(|strategy\(|library\(/.test(text)) continue;
        if (text.length >= 500) continue;
        var ts = null;
        var tsMatch = text.match(/^(\d{4}-\d{2}-\d{2}\s+)?\d{2}:\d{2}:\d{2}/);
        if (tsMatch) ts = tsMatch[0];
        var type = 'info';
        var cls = rows[i].className || '';
        if (/error/i.test(cls) || /error/i.test(text.substring(0, 30))) type = 'error';
        else if (/compil/i.test(text.substring(0, 40))) type = 'compile';
        else if (/warn/i.test(cls)) type = 'warning';
        results.push({ timestamp: ts, type: type, message: text });
    }
    return results;
})()
"#;

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::ops::test_support::FakeRuntime;

    use super::*;
    use tradingview_cdp::KeyEventType;

    #[tokio::test]
    async fn pine_compile_clicks_safe_button_and_returns_markers() {
        let markers = json!([
            {"line": 3, "column": 1, "message": "Syntax error", "severity": 8}
        ]);
        let mut runtime = FakeRuntime::new([
            json!(true),
            json!(4),
            json!({"clicked": true, "button_text": "チャートに追加チャートに追加", "blocked_save": false}),
            markers,
            json!(4),
        ]);

        let result = pine_compile(&mut runtime).await.unwrap();

        assert_eq!(result["button_clicked"], "チャートに追加");
        assert_eq!(result["has_errors"], true);
        assert_eq!(result["error_count"], 1);
        assert_eq!(result["errors"][0]["message"], "Syntax error");
        assert_eq!(result["study_added"], false);
        assert_eq!(result["studies_before"], 4);
        assert_eq!(result["studies_after"], 4);
        assert!(runtime.evaluated[2].0.contains("blocked_save"));
        assert!(runtime.key_events.is_empty());
    }

    #[tokio::test]
    async fn pine_compile_accepts_korean_compile_button_label() {
        let mut runtime = FakeRuntime::new([
            json!(true),
            json!(4),
            json!({"clicked": true, "button_text": "차트 업데이트", "blocked_save": false}),
            json!([]),
            json!(4),
        ]);

        let result = pine_compile(&mut runtime).await.unwrap();

        assert_eq!(result["button_clicked"], "차트 업데이트");
        assert!(runtime.evaluated[2].0.contains("차트에 넣기"));
        assert!(runtime.evaluated[2].0.contains("차트 업데이트"));
        assert!(runtime.key_events.is_empty());
    }

    #[tokio::test]
    async fn pine_compile_rejects_save_related_button() {
        let mut runtime = FakeRuntime::new([
            json!(true),
            json!(4),
            json!({"clicked": false, "button_text": "Save and add to chart", "blocked_save": true}),
        ]);

        let error = pine_compile(&mut runtime).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(
            error.message,
            "Pine compile found only a save-related action button"
        );
        assert!(runtime.key_events.is_empty());
    }

    #[tokio::test]
    async fn pine_compile_uses_ctrl_enter_fallback() {
        let mut runtime = FakeRuntime::new([
            json!(true),
            json!(2),
            json!({"clicked": false, "button_text": null, "blocked_save": false}),
            json!([]),
            json!(2),
        ]);

        let result = pine_compile(&mut runtime).await.unwrap();

        assert_eq!(result["button_clicked"], "keyboard_shortcut");
        assert_eq!(result["has_errors"], false);
        assert_eq!(runtime.key_events.len(), 2);
        assert_eq!(runtime.key_events[0].event_type, KeyEventType::KeyDown);
        assert_eq!(runtime.key_events[0].key, "Enter");
        assert_eq!(runtime.key_events[0].modifiers, 2);
        assert_eq!(runtime.key_events[1].event_type, KeyEventType::KeyUp);
    }

    #[tokio::test]
    async fn pine_raw_compile_clicks_save_related_button() {
        let mut runtime = FakeRuntime::new([
            json!(true),
            json!({"clicked": true, "button_text": "Save and add to chart"}),
        ]);

        let result = pine_raw_compile(&mut runtime).await.unwrap();

        assert_eq!(result["button_clicked"], "Save and add to chart");
        assert_eq!(result["raw_compile"], true);
        assert_eq!(result["source"], "dom_fallback");
        assert!(runtime.evaluated[1].0.contains("Save and add to chart"));
        assert!(runtime.key_events.is_empty());
    }

    #[tokio::test]
    async fn pine_raw_compile_includes_korean_compile_fallback_labels() {
        let mut runtime = FakeRuntime::new([json!(true), json!({"clicked": false})]);

        let _ = pine_raw_compile(&mut runtime).await.unwrap();

        assert!(runtime.evaluated[1].0.contains("차트에 넣기"));
        assert!(runtime.evaluated[1].0.contains("차트 업데이트"));
    }

    #[tokio::test]
    async fn pine_raw_compile_uses_ctrl_enter_fallback() {
        let mut runtime = FakeRuntime::new([json!(true), json!({"clicked": false})]);

        let result = pine_raw_compile(&mut runtime).await.unwrap();

        assert_eq!(result["button_clicked"], "keyboard_shortcut");
        assert_eq!(runtime.key_events.len(), 2);
        assert_eq!(runtime.key_events[0].key, "Enter");
        assert_eq!(runtime.key_events[0].modifiers, 2);
    }

    #[tokio::test]
    async fn pine_compile_rejects_malformed_marker_payload() {
        let mut runtime = FakeRuntime::new([
            json!(true),
            json!(2),
            json!({"clicked": true, "button_text": "Update on chart", "blocked_save": false}),
            json!({"bad": true}),
        ]);

        let error = pine_compile(&mut runtime).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.message, "Pine marker payload was not an array");
    }

    #[tokio::test]
    async fn pine_errors_returns_marker_payload() {
        let markers = json!([
            {"line": 2, "column": 1, "message": "Unknown identifier", "severity": 8}
        ]);
        let mut runtime = FakeRuntime::new([json!(true), markers]);

        let result = pine_errors(&mut runtime).await.unwrap();

        assert_eq!(result["has_errors"], true);
        assert_eq!(result["error_count"], 1);
        assert_eq!(result["errors"][0]["message"], "Unknown identifier");
    }

    #[tokio::test]
    async fn pine_console_returns_entries() {
        let mut runtime = FakeRuntime::new([
            json!(true),
            json!([{"timestamp": "12:00:00", "type": "info", "message": "hello"}]),
        ]);

        let result = pine_console(&mut runtime).await.unwrap();

        assert_eq!(result["entry_count"], 1);
        assert_eq!(result["entries"][0]["message"], "hello");
    }
}
