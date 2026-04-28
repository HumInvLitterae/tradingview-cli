use serde_json::{Value, json};

use tradingview_cdp::{KeyEventType, RuntimeEvaluator};
use tradingview_core::{AppError, ErrorKind};

use super::runtime::{PINE_SAVE_WAIT, dispatch_key, ensure_pine_editor_open, with_monaco};

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

fn pine_save_error(message: String, details: Value) -> AppError {
    let kind = match details.get("kind").and_then(Value::as_str) {
        Some("validation") => ErrorKind::Validation,
        _ => ErrorKind::InternalApiUnavailable,
    };
    AppError::new(kind, message).with_details(details)
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

    use crate::ops::test_support::FakeRuntime;

    use super::*;

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
    async fn pine_list_preserves_fetch_error_with_empty_list() {
        let mut runtime = FakeRuntime::new([json!({"scripts": [], "error": "Failed to fetch"})]);

        let result = pine_list(&mut runtime).await.unwrap();

        assert_eq!(result["count"], 0);
        assert_eq!(result["scripts"], json!([]));
        assert_eq!(result["source"], "internal_api");
        assert_eq!(result["error"], "Failed to fetch");
        assert!(runtime.evaluated[0].1);
    }
}
