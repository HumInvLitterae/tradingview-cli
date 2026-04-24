use serde_json::{Value, json};
use std::time::Duration;

use crate::{
    cdp::{KeyEvent, KeyEventType, RuntimeEvaluator},
    error::{AppError, ErrorKind},
};

const FIND_MONACO: &str = r#"
(function findMonacoEditor() {
    var container = document.querySelector('.monaco-editor.pine-editor-monaco');
    if (!container) return null;
    var el = container;
    var fiberKey;
    for (var i = 0; i < 20; i++) {
        if (!el) break;
        fiberKey = Object.keys(el).find(function(k) { return k.startsWith('__reactFiber$'); });
        if (fiberKey) break;
        el = el.parentElement;
    }
    if (!fiberKey) return null;
    var current = el[fiberKey];
    for (var d = 0; d < 15; d++) {
        if (!current) break;
        var env = null;
        if (current.memoizedProps && current.memoizedProps.monacoEnv) {
            env = current.memoizedProps.monacoEnv;
        } else if (current.memoizedProps && current.memoizedProps.value && current.memoizedProps.value.monacoEnv) {
            env = current.memoizedProps.value.monacoEnv;
        }
        if (env) {
            if (env.editor && typeof env.editor.getEditors === 'function') {
                var editors = env.editor.getEditors();
                if (editors.length > 0) return { editor: editors[0], env: env };
            }
        }
        current = current.return;
    }
    return null;
})()
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EditorOpenState {
    editor_open_before: bool,
    opened_editor: bool,
}

#[cfg(test)]
const PINE_COMPILE_WAIT: Duration = Duration::from_millis(0);
#[cfg(not(test))]
const PINE_COMPILE_WAIT: Duration = Duration::from_millis(2500);

pub async fn pine_get(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    let open_state = ensure_pine_editor_open(runtime).await?;
    let value = runtime
        .evaluate(&with_monaco(PINE_GET_SOURCE_EXPRESSION), false)
        .await?;
    let source = value.as_str().ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Monaco editor found but source was not a string",
        )
        .with_details(value.clone())
    })?;

    Ok(json!({
        "source": source,
        "line_count": source.split('\n').count(),
        "char_count": source.chars().count(),
        "editor_open_before": open_state.editor_open_before,
        "opened_editor": open_state.opened_editor,
    }))
}

pub async fn pine_set(
    runtime: &mut impl RuntimeEvaluator,
    source: &str,
    input_source: &str,
) -> Result<Value, AppError> {
    let open_state = ensure_pine_editor_open(runtime).await?;
    let value = runtime
        .evaluate(&pine_set_source_expression(source), false)
        .await?;
    let observed_source = value.as_str().ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Monaco editor found but set source verification was not a string",
        )
        .with_details(value.clone())
    })?;

    if observed_source != source {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Pine source set verification failed",
        )
        .with_details(json!({
            "expected_char_count": source.chars().count(),
            "observed_char_count": observed_source.chars().count(),
        })));
    }

    Ok(json!({
        "lines_set": source.split('\n').count(),
        "char_count": source.chars().count(),
        "input_source": input_source,
        "editor_open_before": open_state.editor_open_before,
        "opened_editor": open_state.opened_editor,
    }))
}

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

pub async fn pine_list(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    let raw = runtime.evaluate(PINE_LIST_EXPRESSION, true).await?;
    let scripts = raw
        .get("scripts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    Ok(json!({
        "scripts": scripts,
        "count": scripts.len(),
        "source": "internal_api",
        "error": raw.get("error").cloned().unwrap_or(Value::Null),
    }))
}

async fn ensure_pine_editor_open(
    runtime: &mut impl RuntimeEvaluator,
) -> Result<EditorOpenState, AppError> {
    let editor_open_before = runtime
        .evaluate(
            &with_monaco("var m = __FIND_MONACO__; return m !== null;"),
            false,
        )
        .await?
        .as_bool()
        .unwrap_or(false);
    if editor_open_before {
        return Ok(EditorOpenState {
            editor_open_before,
            opened_editor: false,
        });
    }

    runtime
        .evaluate(
            r#"
            (function() {
                var bwb = window.TradingView && window.TradingView.bottomWidgetBar;
                if (bwb) {
                    if (typeof bwb.activateScriptEditorTab === 'function') bwb.activateScriptEditorTab();
                    else if (typeof bwb.showWidget === 'function') bwb.showWidget('pine-editor');
                    else if (typeof bwb.open === 'function') bwb.open('pine-editor');
                    else if (typeof bwb.show === 'function') bwb.show('pine-editor');
                }
                var btn = document.querySelector('[aria-label="Pine"]')
                    || document.querySelector('[data-name="pine-dialog-button"]');
                if (btn) btn.click();
                return true;
            })()
            "#,
            false,
        )
        .await?;

    for _ in 0..50 {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let ready = runtime
            .evaluate(
                &with_monaco("var m = __FIND_MONACO__; return m !== null;"),
                false,
            )
            .await?
            .as_bool()
            .unwrap_or(false);
        if ready {
            return Ok(EditorOpenState {
                editor_open_before,
                opened_editor: true,
            });
        }
    }

    Err(AppError::new(
        ErrorKind::InternalApiUnavailable,
        "Could not open Pine Editor or Monaco was not found",
    )
    .with_details(json!({
        "editor_open_before": editor_open_before,
        "opened_editor": false,
    })))
}

fn with_monaco(body: &str) -> String {
    format!(
        "(function() {{ {} }})()",
        body.replace("__FIND_MONACO__", FIND_MONACO)
    )
}

fn normalize_array(value: Value, error_message: &str) -> Result<Vec<Value>, AppError> {
    value.as_array().cloned().ok_or_else(|| {
        AppError::new(ErrorKind::InternalApiUnavailable, error_message).with_details(value)
    })
}

fn normalize_button_text(text: &str) -> String {
    let trimmed = text.trim();
    let chars = trimmed.chars().collect::<Vec<_>>();
    if chars.len() % 2 == 0 {
        let midpoint = chars.len() / 2;
        if chars[..midpoint] == chars[midpoint..] {
            return chars[..midpoint].iter().collect();
        }
    }
    trimmed.to_string()
}

async fn pine_study_count(runtime: &mut impl RuntimeEvaluator) -> Result<Option<i64>, AppError> {
    let value = runtime.evaluate(PINE_STUDY_COUNT_EXPRESSION, false).await?;
    Ok(value.as_i64())
}

async fn dispatch_ctrl_enter(runtime: &mut impl RuntimeEvaluator) -> Result<(), AppError> {
    dispatch_key(runtime, KeyEventType::KeyDown, "Enter", "Enter", 13, 2).await?;
    dispatch_key(runtime, KeyEventType::KeyUp, "Enter", "Enter", 13, 0).await
}

async fn dispatch_key(
    runtime: &mut impl RuntimeEvaluator,
    event_type: KeyEventType,
    key: &'static str,
    code: &'static str,
    windows_virtual_key_code: i64,
    modifiers: i64,
) -> Result<(), AppError> {
    runtime
        .dispatch_key_event(KeyEvent {
            event_type,
            key,
            code,
            windows_virtual_key_code,
            modifiers,
        })
        .await
}

fn pine_set_source_expression(source: &str) -> String {
    let source = serde_json::to_string(source).expect("string serialization should not fail");
    with_monaco(&format!(
        r#"
var m = __FIND_MONACO__;
if (!m) return null;
m.editor.setValue({source});
return m.editor.getValue();
"#
    ))
}

const PINE_GET_SOURCE_EXPRESSION: &str = r#"
var m = __FIND_MONACO__;
if (!m) return null;
return m.editor.getValue();
"#;

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
        if (!saveCandidate && isSaveAction(text) && /chart|チャート/.test(text)) {
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

const PINE_LIST_EXPRESSION: &str = r#"
fetch('https://pine-facade.tradingview.com/pine-facade/list/?filter=saved', { credentials: 'include' })
    .then(function(r) { return r.json(); })
    .then(function(data) {
        if (!Array.isArray(data)) return { scripts: [], error: 'Unexpected response from pine-facade' };
        return {
            scripts: data.map(function(s) {
                return {
                    id: s.scriptIdPart || null,
                    name: s.scriptName || s.scriptTitle || 'Untitled',
                    title: s.scriptTitle || null,
                    version: s.version || null,
                    modified: s.modified || null
                };
            })
        };
    })
    .catch(function(e) { return { scripts: [], error: e.message }; })
"#;

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::test_support::FakeRuntime;
    use super::*;

    #[tokio::test]
    async fn pine_get_returns_source_counts_and_open_state() {
        let mut runtime = FakeRuntime::new([json!(true), json!("//@version=6\nindicator(\"X\")")]);

        let result = pine_get(&mut runtime).await.unwrap();

        assert_eq!(result["source"], "//@version=6\nindicator(\"X\")");
        assert_eq!(result["line_count"], 2);
        assert_eq!(result["char_count"], 27);
        assert_eq!(result["editor_open_before"], true);
        assert_eq!(result["opened_editor"], false);
        assert!(runtime.evaluated[1].0.contains("getValue"));
    }

    #[tokio::test]
    async fn pine_get_opens_editor_when_needed() {
        let mut runtime = FakeRuntime::new([
            json!(false),
            json!(true),
            json!(false),
            json!(true),
            json!("plot(close)"),
        ]);

        let result = pine_get(&mut runtime).await.unwrap();

        assert_eq!(result["source"], "plot(close)");
        assert_eq!(result["editor_open_before"], false);
        assert_eq!(result["opened_editor"], true);
        assert!(runtime.evaluated[1].0.contains("activateScriptEditorTab"));
    }

    #[tokio::test]
    async fn pine_set_updates_source_and_returns_counts() {
        let source = "//@version=6\nindicator(\"Quoted \\\"X\\\"\")\nplot(close)";
        let mut runtime = FakeRuntime::new([json!(true), json!(source)]);

        let result = pine_set(&mut runtime, source, "stdin").await.unwrap();

        assert_eq!(result["lines_set"], 3);
        assert_eq!(result["char_count"], source.chars().count());
        assert_eq!(result["input_source"], "stdin");
        assert_eq!(result["editor_open_before"], true);
        assert_eq!(result["opened_editor"], false);
        assert!(runtime.evaluated[1].0.contains("setValue"));
        let serialized_source = serde_json::to_string(source).unwrap();
        assert!(runtime.evaluated[1].0.contains(&serialized_source));
    }

    #[tokio::test]
    async fn pine_set_opens_editor_when_needed() {
        let mut runtime = FakeRuntime::new([
            json!(false),
            json!(true),
            json!(false),
            json!(true),
            json!("plot(close)"),
        ]);

        let result = pine_set(&mut runtime, "plot(close)", "file").await.unwrap();

        assert_eq!(result["lines_set"], 1);
        assert_eq!(result["input_source"], "file");
        assert_eq!(result["editor_open_before"], false);
        assert_eq!(result["opened_editor"], true);
    }

    #[tokio::test]
    async fn pine_set_errors_when_verification_differs() {
        let mut runtime = FakeRuntime::new([json!(true), json!("plot(open)")]);

        let error = pine_set(&mut runtime, "plot(close)", "stdin")
            .await
            .unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.message, "Pine source set verification failed");
    }

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

    #[tokio::test]
    async fn pine_list_preserves_fetch_error_with_empty_list() {
        let mut runtime = FakeRuntime::new([json!({"scripts": [], "error": "Failed to fetch"})]);

        let result = pine_list(&mut runtime).await.unwrap();

        assert_eq!(result["count"], 0);
        assert_eq!(result["scripts"], json!([]));
        assert_eq!(result["source"], "internal_api");
        assert_eq!(result["error"], "Failed to fetch");
        assert!(runtime.evaluated[0].1);
    }

    #[tokio::test]
    async fn ensure_pine_editor_open_errors_when_monaco_never_appears() {
        let mut responses = vec![json!(false), json!(true)];
        responses.extend(std::iter::repeat_n(json!(false), 50));
        let mut runtime = FakeRuntime::new(responses);

        let error = pine_get(&mut runtime).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert!(error.message.contains("Could not open Pine Editor"));
    }
}
