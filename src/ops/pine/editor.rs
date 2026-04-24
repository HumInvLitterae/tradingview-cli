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

#[cfg(test)]
const PINE_SAVE_WAIT: Duration = Duration::from_millis(0);
#[cfg(not(test))]
const PINE_SAVE_WAIT: Duration = Duration::from_millis(800);

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

pub fn validate_pine_script_type(script_type: &str) -> Result<&'static str, AppError> {
    match script_type.trim().to_ascii_lowercase().as_str() {
        "indicator" => Ok("indicator"),
        "strategy" => Ok("strategy"),
        "library" => Ok("library"),
        _ => Err(AppError::new(
            ErrorKind::Validation,
            "Pine script type must be one of: indicator, strategy, library",
        )
        .with_details(json!({ "script_type": script_type }))),
    }
}

pub async fn pine_new(
    runtime: &mut impl RuntimeEvaluator,
    script_type: &str,
) -> Result<Value, AppError> {
    let script_type = validate_pine_script_type(script_type)?;
    let template = pine_template(script_type);
    let open_state = ensure_pine_editor_open(runtime).await?;
    let value = runtime
        .evaluate(&pine_set_source_expression(template), false)
        .await?;
    let observed_source = value.as_str().ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Monaco editor found but new script verification was not a string",
        )
        .with_details(value.clone())
    })?;

    if observed_source != template {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Pine new script verification failed",
        )
        .with_details(json!({
            "expected_char_count": template.chars().count(),
            "observed_char_count": observed_source.chars().count(),
        })));
    }

    Ok(json!({
        "type": script_type,
        "action": "new_script_created",
        "template": template,
        "lines_set": template.split('\n').count(),
        "char_count": template.chars().count(),
        "editor_open_before": open_state.editor_open_before,
        "opened_editor": open_state.opened_editor,
    }))
}

pub async fn pine_open(runtime: &mut impl RuntimeEvaluator, name: &str) -> Result<Value, AppError> {
    let open_state = ensure_pine_editor_open(runtime).await?;
    let raw = runtime.evaluate(&pine_open_expression(name), true).await?;

    if let Some(error) = raw.get("error").and_then(Value::as_str) {
        let kind = match raw.get("kind").and_then(Value::as_str) {
            Some("validation") => ErrorKind::Validation,
            _ => ErrorKind::InternalApiUnavailable,
        };
        return Err(AppError::new(kind, error).with_details(raw));
    }

    let source = raw
        .get("source_text")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            AppError::new(
                ErrorKind::InternalApiUnavailable,
                "Pine open payload did not include source text",
            )
            .with_details(raw.clone())
        })?;
    let script_name = raw
        .get("name")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            AppError::new(
                ErrorKind::InternalApiUnavailable,
                "Pine open payload did not include script name",
            )
            .with_details(raw.clone())
        })?;
    let script_id = raw
        .get("script_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            AppError::new(
                ErrorKind::InternalApiUnavailable,
                "Pine open payload did not include script id",
            )
            .with_details(raw.clone())
        })?;

    Ok(json!({
        "name": script_name,
        "script_id": script_id,
        "version": raw.get("version").cloned().unwrap_or(Value::Null),
        "lines": source.split('\n').count(),
        "source": "internal_api",
        "opened": true,
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

pub async fn pine_save(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    let open_state = ensure_pine_editor_open(runtime).await?;
    let before = runtime
        .evaluate(&pine_save_preflight_expression(), false)
        .await?;
    if let Some(error) = before.get("error").and_then(Value::as_str) {
        return Err(pine_save_error(error.to_string(), before));
    }

    dispatch_key(runtime, KeyEventType::KeyDown, "s", "KeyS", 83, 2).await?;
    dispatch_key(runtime, KeyEventType::KeyUp, "s", "KeyS", 83, 0).await?;
    tokio::time::sleep(PINE_SAVE_WAIT).await;

    let raw = runtime
        .evaluate(&pine_save_post_shortcut_expression(&before), true)
        .await?;
    if let Some(error) = raw.get("error").and_then(Value::as_str) {
        return Err(pine_save_error(error.to_string(), raw));
    }
    if raw
        .get("dirty_after")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Pine save did not clear the dirty state",
        )
        .with_details(raw));
    }

    Ok(json!({
        "saved": raw.get("saved").and_then(Value::as_bool).unwrap_or(true),
        "action": raw.get("action").cloned().unwrap_or_else(|| json!("saved")),
        "name": raw.get("name").cloned().unwrap_or(Value::Null),
        "dialog_handled": raw.get("dialog_handled").and_then(Value::as_bool).unwrap_or(false),
        "source": raw.get("source").cloned().unwrap_or_else(|| json!("dom_fallback")),
        "editor_open_before": open_state.editor_open_before,
        "opened_editor": open_state.opened_editor,
        "dirty_before": raw.get("dirty_before").cloned().unwrap_or_else(|| before.get("dirty_before").cloned().unwrap_or(Value::Null)),
        "dirty_after": raw.get("dirty_after").cloned().unwrap_or(Value::Null),
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

fn pine_save_error(message: String, details: Value) -> AppError {
    let kind = match details.get("kind").and_then(Value::as_str) {
        Some("validation") => ErrorKind::Validation,
        _ => ErrorKind::InternalApiUnavailable,
    };
    AppError::new(kind, message).with_details(details)
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

fn pine_template(script_type: &str) -> &'static str {
    match script_type {
        "strategy" => "//@version=6\nstrategy(\"My strategy\", overlay=true)\n",
        "library" => {
            "//@version=6\n// @description TODO: add library description here\nlibrary(\"MyLibrary\")\n"
        }
        _ => "//@version=6\nindicator(\"My script\")\nplot(close)",
    }
}

fn pine_open_expression(name: &str) -> String {
    let target = serde_json::to_string(&name.to_ascii_lowercase())
        .expect("string serialization should not fail");
    with_monaco(&format!(
        r#"
var m = __FIND_MONACO__;
if (!m) return {{ error: "Monaco editor not found to inject source", kind: "internal_api_unavailable" }};
var target = {target};
return fetch('https://pine-facade.tradingview.com/pine-facade/list/?filter=saved', {{ credentials: 'include' }})
    .then(function(r) {{ return r.json(); }})
    .then(function(scripts) {{
        if (!Array.isArray(scripts)) return {{ error: "pine-facade returned unexpected data", kind: "internal_api_unavailable" }};
        var exact = [];
        var partial = [];
        for (var i = 0; i < scripts.length; i++) {{
            var script = scripts[i] || {{}};
            var sn = (script.scriptName || '').toLowerCase();
            var st = (script.scriptTitle || '').toLowerCase();
            if (sn === target || st === target) exact.push(script);
            else if (sn.indexOf(target) !== -1 || st.indexOf(target) !== -1) partial.push(script);
        }}
        var match = null;
        if (exact.length > 0) {{
            match = exact[0];
        }} else if (partial.length === 1) {{
            match = partial[0];
        }} else if (partial.length > 1) {{
            return {{
                error: 'Multiple Pine scripts match "' + target + '"',
                kind: "validation",
                matches: partial.slice(0, 10).map(function(s) {{
                    return {{ name: s.scriptName || s.scriptTitle || 'Untitled', id: s.scriptIdPart || null, version: s.version || null }};
                }})
            }};
        }} else {{
            return {{ error: 'Script "' + target + '" not found. Use pine list to see available scripts.', kind: "validation" }};
        }}

        var id = match.scriptIdPart;
        var version = match.version || 1;
        var displayName = match.scriptName || match.scriptTitle || 'Untitled';
        if (!id) return {{ error: "Matched script did not include script id", kind: "internal_api_unavailable", name: displayName }};

        return fetch('https://pine-facade.tradingview.com/pine-facade/get/' + id + '/' + version, {{ credentials: 'include' }})
            .then(function(r2) {{ return r2.json(); }})
            .then(function(data) {{
                var source = data && data.source || '';
                if (!source) return {{ error: "Script source is empty", kind: "internal_api_unavailable", name: displayName, script_id: id, version: version }};
                m.editor.setValue(source);
                var observed = m.editor.getValue();
                if (observed !== source) return {{ error: "Pine open source verification failed", kind: "internal_api_unavailable", name: displayName, script_id: id, version: version }};
                return {{ success: true, name: displayName, script_id: id, version: version, source_text: source }};
            }});
    }})
    .catch(function(e) {{ return {{ error: e.message, kind: "internal_api_unavailable" }}; }});
"#
    ))
}

fn pine_save_preflight_expression() -> String {
    with_monaco(
        r#"
function dirtyState() {{
    try {{
        var buttons = Array.from(document.querySelectorAll('button, [role="button"], span, div'));
        var dirty = null;
        for (var i = 0; i < buttons.length; i++) {{
            var text = (buttons[i].textContent || buttons[i].getAttribute('aria-label') || buttons[i].getAttribute('title') || '').trim();
            if (!text) continue;
            if (/unsaved version/i.test(text) || /未保存/.test(text)) dirty = true;
            if (/^saved$/i.test(text) || /^保存済み$/.test(text)) dirty = false;
        }}
        return dirty;
    }} catch(e) {{
        return null;
    }}
}}
function finish() {{
    return {{ ok: true, dirty_before: dirtyState() }};
}}
return finish();
"#,
    )
}

fn pine_save_post_shortcut_expression(preflight: &Value) -> String {
    let dirty_before = serde_json::to_string(
        &preflight
            .get("dirty_before")
            .cloned()
            .unwrap_or(Value::Null),
    )
    .expect("JSON serialization should not fail");
    with_monaco(&format!(
        r#"
var dirtyBefore = {dirty_before};
function visible(el) {{
    if (!el) return false;
    var rect = el.getBoundingClientRect();
    var style = window.getComputedStyle(el);
    return rect.width > 0 && rect.height > 0 && style.visibility !== 'hidden' && style.display !== 'none';
}}
function label(el) {{
    return (el.textContent || el.getAttribute('aria-label') || el.getAttribute('title') || '').trim();
}}
function dirtyState() {{
    try {{
        var nodes = Array.from(document.querySelectorAll('button, [role="button"], span, div'));
        var dirty = null;
        for (var i = 0; i < nodes.length; i++) {{
            var text = label(nodes[i]);
            if (!text) continue;
            if (/unsaved version/i.test(text) || /未保存/.test(text)) dirty = true;
            if (/^saved$/i.test(text) || /^保存済み$/.test(text)) dirty = false;
        }}
        return dirty;
    }} catch(e) {{
        return null;
    }}
}}
function saveDialog() {{
    var dialogs = Array.from(document.querySelectorAll('[role="dialog"], [class*="dialog"], [class*="modal"], [class*="popup"]')).filter(visible);
    for (var i = 0; i < dialogs.length; i++) {{
        var text = dialogs[i].textContent || '';
        var buttons = Array.from(dialogs[i].querySelectorAll('button')).filter(visible);
        var hasSave = buttons.some(function(button) {{ return /^save$/i.test(label(button)) || /^保存$/.test(label(button)); }});
        var hasInput = dialogs[i].querySelector('input, textarea');
        if (hasSave && (hasInput || /save/i.test(text) || /保存/.test(text))) return dialogs[i];
    }}
    var inputs = Array.from(document.querySelectorAll('input, textarea')).filter(visible);
    for (var j = 0; j < inputs.length; j++) {{
        var parent = inputs[j].parentElement;
        for (var depth = 0; parent && depth < 8; depth++, parent = parent.parentElement) {{
            var parentText = parent.textContent || '';
            var parentButtons = Array.from(parent.querySelectorAll('button')).filter(visible);
            var parentHasSave = parentButtons.some(function(button) {{ return /^save$/i.test(label(button)) || /^保存$/.test(label(button)); }});
            if (parentHasSave && (/script/i.test(parentText) || /スクリプト/.test(parentText) || /保存/.test(parentText))) return parent;
        }}
    }}
    return null;
}}
function finish() {{
    var dialog = saveDialog();
    if (dialog) {{
        return {{ error: "Pine save requires an already saved script; naming unsaved scripts is deferred", kind: "validation", dialog_open: true, dirty_before: dirtyBefore }};
    }}
    return new Promise(function(resolve) {{
        setTimeout(function() {{
            var dirtyAfter = dirtyState();
            resolve({{
                saved: dirtyAfter !== true,
                action: "saved",
                name: null,
                dialog_handled: false,
                source: "dom_fallback",
                dirty_before: dirtyBefore,
                dirty_after: dirtyAfter
            }});
        }}, 1200);
    }});
}}
return finish();
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

    use super::super::super::test_support::FakeRuntime;
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

    #[test]
    fn validate_pine_script_type_accepts_known_types() {
        assert_eq!(validate_pine_script_type("indicator").unwrap(), "indicator");
        assert_eq!(validate_pine_script_type("STRATEGY").unwrap(), "strategy");
        assert_eq!(validate_pine_script_type(" library ").unwrap(), "library");
    }

    #[test]
    fn validate_pine_script_type_rejects_unknown_type() {
        let error = validate_pine_script_type("study").unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(error.message.contains("indicator, strategy, library"));
    }

    #[tokio::test]
    async fn pine_new_sets_indicator_template_by_default_shape() {
        let template = pine_template("indicator");
        let mut runtime = FakeRuntime::new([json!(true), json!(template)]);

        let result = pine_new(&mut runtime, "indicator").await.unwrap();

        assert_eq!(result["type"], "indicator");
        assert_eq!(result["action"], "new_script_created");
        assert_eq!(result["template"], template);
        assert_eq!(result["lines_set"], 3);
        assert_eq!(result["char_count"], template.chars().count());
        assert_eq!(result["editor_open_before"], true);
        assert_eq!(result["opened_editor"], false);
        assert!(runtime.evaluated[1].0.contains("setValue"));
    }

    #[tokio::test]
    async fn pine_new_supports_strategy_and_library_templates() {
        let strategy = pine_template("strategy");
        let mut runtime = FakeRuntime::new([json!(true), json!(strategy)]);

        let result = pine_new(&mut runtime, "strategy").await.unwrap();

        assert_eq!(result["type"], "strategy");
        assert!(result["template"].as_str().unwrap().contains("strategy("));

        let library = pine_template("library");
        let mut runtime = FakeRuntime::new([json!(true), json!(library)]);

        let result = pine_new(&mut runtime, "library").await.unwrap();

        assert_eq!(result["type"], "library");
        assert!(result["template"].as_str().unwrap().contains("library("));
    }

    #[tokio::test]
    async fn pine_new_errors_when_verification_differs() {
        let mut runtime = FakeRuntime::new([json!(true), json!("plot(open)")]);

        let error = pine_new(&mut runtime, "indicator").await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.message, "Pine new script verification failed");
    }

    #[tokio::test]
    async fn pine_open_returns_success_payload() {
        let source = "//@version=6\nindicator(\"Saved\")\nplot(close)";
        let mut runtime = FakeRuntime::new([
            json!(true),
            json!({
                "success": true,
                "name": "Saved Script",
                "script_id": "abc123",
                "version": 4,
                "source_text": source
            }),
        ]);

        let result = pine_open(&mut runtime, "Saved Script").await.unwrap();

        assert_eq!(result["name"], "Saved Script");
        assert_eq!(result["script_id"], "abc123");
        assert_eq!(result["version"], 4);
        assert_eq!(result["lines"], 3);
        assert_eq!(result["source"], "internal_api");
        assert_eq!(result["opened"], true);
        assert_eq!(result["editor_open_before"], true);
        assert_eq!(result["opened_editor"], false);
        assert!(runtime.evaluated[1].0.contains("pine-facade/list"));
        assert!(runtime.evaluated[1].0.contains("pine-facade/get"));
        assert!(runtime.evaluated[1].1);
    }

    #[tokio::test]
    async fn pine_open_maps_missing_script_to_validation() {
        let mut runtime = FakeRuntime::new([
            json!(true),
            json!({
                "error": "Script \"missing\" not found. Use pine list to see available scripts.",
                "kind": "validation"
            }),
        ]);

        let error = pine_open(&mut runtime, "missing").await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(error.message.contains("not found"));
    }

    #[tokio::test]
    async fn pine_open_maps_ambiguous_match_to_validation_with_candidates() {
        let mut runtime = FakeRuntime::new([
            json!(true),
            json!({
                "error": "Multiple Pine scripts match \"test\"",
                "kind": "validation",
                "matches": [{"name": "Test A"}, {"name": "Test B"}]
            }),
        ]);

        let error = pine_open(&mut runtime, "test").await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(error.message.contains("Multiple Pine scripts match"));
        assert_eq!(error.details.unwrap()["matches"][0]["name"], "Test A");
    }

    #[tokio::test]
    async fn pine_open_rejects_empty_source_payload() {
        let mut runtime = FakeRuntime::new([
            json!(true),
            json!({
                "error": "Script source is empty",
                "kind": "internal_api_unavailable",
                "name": "Empty Script"
            }),
        ]);

        let error = pine_open(&mut runtime, "Empty Script").await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.message, "Script source is empty");
    }

    #[tokio::test]
    async fn pine_open_rejects_malformed_success_payload() {
        let mut runtime = FakeRuntime::new([json!(true), json!({"success": true})]);

        let error = pine_open(&mut runtime, "Broken").await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(
            error.message,
            "Pine open payload did not include source text"
        );
    }

    #[tokio::test]
    async fn pine_save_returns_success_payload_for_existing_script() {
        let mut runtime = FakeRuntime::new([
            json!(true),
            json!({"ok": true, "dirty_before": true}),
            json!({
                "saved": true,
                "action": "saved",
                "name": null,
                "dialog_handled": false,
                "source": "dom_fallback",
                "dirty_before": true,
                "dirty_after": false
            }),
        ]);

        let result = pine_save(&mut runtime).await.unwrap();

        assert_eq!(result["saved"], true);
        assert_eq!(result["action"], "saved");
        assert_eq!(result["dialog_handled"], false);
        assert_eq!(result["dirty_before"], true);
        assert_eq!(result["dirty_after"], false);
        assert_eq!(runtime.key_events.len(), 2);
        assert_eq!(runtime.key_events[0].key, "s");
        assert_eq!(runtime.key_events[0].code, "KeyS");
        assert_eq!(runtime.key_events[0].modifiers, 2);
    }

    #[tokio::test]
    async fn pine_save_rejects_unsaved_script_dialog() {
        let mut runtime = FakeRuntime::new([
            json!(true),
            json!({"ok": true, "dirty_before": true}),
            json!({
                "error": "Pine save requires an already saved script; naming unsaved scripts is deferred",
                "kind": "validation",
                "dialog_open": true,
                "dirty_before": true
            }),
        ]);

        let error = pine_save(&mut runtime).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert_eq!(
            error.message,
            "Pine save requires an already saved script; naming unsaved scripts is deferred"
        );
        assert_eq!(runtime.key_events.len(), 2);
    }

    #[tokio::test]
    async fn pine_save_serializes_no_user_script_name() {
        let mut runtime = FakeRuntime::new([
            json!(true),
            json!({"ok": true, "dirty_before": true}),
            json!({
                "saved": true,
                "action": "saved",
                "name": null,
                "dialog_handled": false,
                "source": "dom_fallback",
                "dirty_before": true,
                "dirty_after": false
            }),
        ]);

        let result = pine_save(&mut runtime).await.unwrap();

        assert_eq!(result["saved"], true);
        assert_eq!(result["action"], "saved");
        assert_eq!(result["name"], Value::Null);
        assert_eq!(result["dialog_handled"], false);
        assert!(!runtime.evaluated[1].0.contains("pine-facade/list"));
        assert!(!runtime.evaluated[2].0.contains("requestedName"));
    }

    #[tokio::test]
    async fn pine_save_errors_when_dirty_state_remains() {
        let mut runtime = FakeRuntime::new([
            json!(true),
            json!({"ok": true, "dirty_before": true}),
            json!({
                "saved": false,
                "action": "saved",
                "name": null,
                "dialog_handled": false,
                "source": "dom_fallback",
                "dirty_before": true,
                "dirty_after": true
            }),
        ]);

        let error = pine_save(&mut runtime).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.message, "Pine save did not clear the dirty state");
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
