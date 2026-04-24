use serde_json::{Value, json};

use crate::{
    cdp::RuntimeEvaluator,
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
