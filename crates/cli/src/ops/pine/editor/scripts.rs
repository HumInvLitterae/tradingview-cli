use serde_json::{Value, json};

use tradingview_cdp::{KeyEventType, RuntimeEvaluator};
use tradingview_core::{AppError, ErrorKind};

use super::runtime::{PINE_SAVE_WAIT, dispatch_key, ensure_pine_editor_open, with_monaco};

pub async fn pine_open(runtime: &mut impl RuntimeEvaluator, name: &str) -> Result<Value, AppError> {
    let raw = runtime
        .evaluate(&pine_open_expression(name), true)
        .await
        .map_err(|error| {
            AppError::new(error.kind, "Pine saved-script binding evaluation failed")
                .with_details(pine_open_error_details(&Value::Null, name))
        })?;

    if let Some(error) = raw.get("error").and_then(Value::as_str) {
        let kind = match raw.get("kind").and_then(Value::as_str) {
            Some("validation") => ErrorKind::Validation,
            _ => ErrorKind::InternalApiUnavailable,
        };
        return Err(AppError::new(kind, error).with_details(pine_open_error_details(&raw, name)));
    }

    if raw.get("success").and_then(Value::as_bool) != Some(true) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Pine open payload did not confirm success",
        )
        .with_details(pine_open_error_details(&raw, name)));
    }

    let script_name = raw
        .get("name")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            AppError::new(
                ErrorKind::InternalApiUnavailable,
                "Pine open payload did not include script name",
            )
            .with_details(pine_open_error_details(&raw, name))
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
            .with_details(pine_open_error_details(&raw, name))
        })?;
    let observed_name = raw
        .pointer("/observed_script/name")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            AppError::new(
                ErrorKind::InternalApiUnavailable,
                "Pine open payload did not include observed script identity",
            )
            .with_details(pine_open_error_details(&raw, name))
        })?;
    let resolved_version = raw
        .get("version")
        .and_then(pine_open_version_key)
        .ok_or_else(|| {
            AppError::new(
                ErrorKind::InternalApiUnavailable,
                "Pine open payload did not include a valid script version",
            )
            .with_details(pine_open_error_details(&raw, name))
        })?;
    let observed_version = raw
        .pointer("/observed_script/version")
        .and_then(pine_open_version_key)
        .ok_or_else(|| {
            AppError::new(
                ErrorKind::InternalApiUnavailable,
                "Pine open payload did not include a valid observed script version",
            )
            .with_details(pine_open_error_details(&raw, name))
        })?;
    let slot_rebound = raw
        .get("slot_rebound")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let binding_verified = raw
        .get("binding_verified")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if !slot_rebound
        || !binding_verified
        || script_name.trim() != observed_name.trim()
        || resolved_version != observed_version
    {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Pine open did not verify the active saved-script binding",
        )
        .with_details(pine_open_error_details(&raw, name)));
    }

    Ok(json!({
        "operation": "pine_open",
        "name": script_name,
        "script_id": script_id,
        "version": raw.get("version").cloned().unwrap_or(Value::Null),
        "lines": raw.get("line_count").cloned().unwrap_or(Value::Null),
        "source": "internal_api",
        "source_category": "desktop_backed_operation",
        "requires_desktop": true,
        "non_mutating": false,
        "opened": true,
        "slot_rebound": slot_rebound,
        "binding_verified": binding_verified,
        "binding_method": "pine_editor_internal_api",
        "requested_script": {
            "name": name,
        },
        "observed_script": {
            "name": observed_name,
            "version": raw.pointer("/observed_script/version").cloned().unwrap_or(Value::Null),
        },
        "editor_open_before": raw.get("editor_open_before").and_then(Value::as_bool).unwrap_or(false),
        "opened_editor": raw.get("opened_editor").and_then(Value::as_bool).unwrap_or(false),
    }))
}

fn pine_open_version_key(value: &Value) -> Option<String> {
    match value {
        Value::String(version) if !version.trim().is_empty() => Some(version.trim().to_string()),
        Value::Number(version) => Some(version.to_string()),
        _ => None,
    }
}

fn pine_open_error_details(raw: &Value, requested_name: &str) -> Value {
    let matches = raw
        .get("matches")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .take(10)
                .filter_map(|item| {
                    item.get("name")
                        .and_then(Value::as_str)
                        .filter(|name| !name.trim().is_empty())
                        .map(|name| json!({ "name": name }))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let capabilities = json!({
        "factory_available": raw.pointer("/capabilities/factory_available").and_then(Value::as_bool).unwrap_or(false),
        "open_editor_available": raw.pointer("/capabilities/open_editor_available").and_then(Value::as_bool).unwrap_or(false),
        "open_script_available": raw.pointer("/capabilities/open_script_available").and_then(Value::as_bool).unwrap_or(false),
        "active_readback_available": raw.pointer("/capabilities/active_readback_available").and_then(Value::as_bool).unwrap_or(false),
    });
    let observed_name = raw
        .pointer("/observed_script/name")
        .and_then(Value::as_str)
        .filter(|name| !name.trim().is_empty());
    let observed_version = raw
        .pointer("/observed_script/version")
        .filter(|version| version.is_string() || version.is_number());
    let observed_script = if observed_name.is_some() || observed_version.is_some() {
        json!({
            "name": observed_name,
            "version": observed_version.cloned().unwrap_or(Value::Null),
        })
    } else {
        Value::Null
    };

    json!({
        "operation": "pine_open",
        "requested_script": { "name": requested_name },
        "candidate_count": raw.get("candidate_count").and_then(Value::as_u64).unwrap_or(matches.len() as u64).min(10),
        "matches": matches,
        "capabilities": capabilities,
        "slot_rebound": raw.get("slot_rebound").and_then(Value::as_bool).unwrap_or(false),
        "binding_verified": raw.get("binding_verified").and_then(Value::as_bool).unwrap_or(false),
        "observed_script": observed_script,
        "next_action_hint": "Use a supported TradingView Desktop build and retry; do not save from an unverified editor binding.",
    })
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
    let requested_name = serde_json::to_string(name).expect("string serialization should not fail");
    let target = serde_json::to_string(&name.to_ascii_lowercase())
        .expect("string serialization should not fail");
    format!(
        r#"
	(async function() {{
	    var requestedName = {requested_name};
	    var target = {target};
	    function hasMethod(value, key) {{
	        try {{ return !!value && typeof value[key] === 'function'; }} catch (e) {{ return false; }}
	    }}
	    var root = null;
	    try {{ root = window && window.TradingViewApi; }} catch (e) {{ root = null; }}
	    var factoryAvailable = hasMethod(root, 'pineEditorTestApi');
	    var api = null;
	    try {{ api = factoryAvailable ? root.pineEditorTestApi() : null; }} catch (e) {{ api = null; }}
	    var capabilities = {{
	        factory_available: factoryAvailable,
	        open_editor_available: hasMethod(api, 'openEditor'),
	        open_script_available: hasMethod(api, 'openScript'),
	        active_readback_available: false
    }};
    if (!capabilities.factory_available || !capabilities.open_editor_available || !capabilities.open_script_available) {{
        return {{
            error: "TradingView saved-script open API is unavailable",
            kind: "internal_api_unavailable",
            capabilities: capabilities,
            slot_rebound: false,
            binding_verified: false,
            next_action_hint: "Do not save from this editor state; use a supported TradingView Desktop build and retry."
        }};
    }}

    try {{
        var response = await fetch('https://pine-facade.tradingview.com/pine-facade/list/?filter=saved', {{ credentials: 'include' }});
        if (!response || response.ok === false) {{
            return {{ error: "Pine saved-script list is unavailable", kind: "internal_api_unavailable", capabilities: capabilities }};
        }}
        var scripts = await response.json();
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
        if (exact.length === 1) {{
            match = exact[0];
        }} else if (exact.length > 1) {{
            return {{
                error: 'Multiple Pine scripts exactly match "' + requestedName + '"',
                kind: "validation",
                candidate_count: exact.length,
                matches: exact.slice(0, 10).map(function(s) {{
                    return {{ name: s.scriptName || s.scriptTitle || 'Untitled' }};
                }})
            }};
        }} else if (partial.length === 1) {{
            match = partial[0];
        }} else if (partial.length > 1) {{
            return {{
                error: 'Multiple Pine scripts match "' + target + '"',
                kind: "validation",
                candidate_count: partial.length,
                matches: partial.slice(0, 10).map(function(s) {{
                    return {{ name: s.scriptName || s.scriptTitle || 'Untitled' }};
                }})
            }};
        }} else {{
            return {{ error: 'Script "' + target + '" not found. Use pine list to see available scripts.', kind: "validation" }};
        }}

        var id = match.scriptIdPart;
        var version = match.version || 1;
        var displayName = match.scriptName || match.scriptTitle || 'Untitled';
        if (!id) return {{ error: "Matched script did not include a usable identity", kind: "internal_api_unavailable", name: displayName, capabilities: capabilities }};

        var sourceResponse = await fetch('https://pine-facade.tradingview.com/pine-facade/get/' + encodeURIComponent(id) + '/' + encodeURIComponent(version), {{ credentials: 'include' }});
        if (!sourceResponse || sourceResponse.ok === false) {{
            return {{ error: "Pine saved-script source is unavailable", kind: "internal_api_unavailable", name: displayName, capabilities: capabilities }};
        }}
        var data = await sourceResponse.json();
        var source = data && data.source || '';
        if (!source) return {{ error: "Script source is empty", kind: "internal_api_unavailable", name: displayName, capabilities: capabilities }};

        var editorOpenBefore = !!api._pineEditor;
        await Promise.resolve(api.openEditor());
        api = root.pineEditorTestApi();
        var provider = api && api._pineEditor && api._pineEditor._storeProvider;
	        capabilities.active_readback_available = hasMethod(provider, 'getEditorActiveScript');
        if (!capabilities.active_readback_available) {{
            return {{
                error: "TradingView active saved-script readback is unavailable",
                kind: "internal_api_unavailable",
                name: displayName,
                capabilities: capabilities,
                slot_rebound: false,
                binding_verified: false,
                editor_open_before: editorOpenBefore,
                opened_editor: !editorOpenBefore,
                next_action_hint: "Do not save from this editor state; active script identity could not be verified."
            }};
        }}

        await Promise.resolve(api.openScript({{ scriptIdPart: id, version: version }}));
        var active = null;
        try {{ active = provider.getEditorActiveScript(); }} catch (e) {{ active = null; }}
        var observedId = active && active.scriptIdPart != null ? String(active.scriptIdPart) : null;
        var observedVersion = active && active.version != null ? active.version : null;
        var observedName = active && (active.scriptName || active.scriptTitle) || null;
        var idMatches = observedId !== null && observedId === String(id);
        var versionMatches = observedVersion !== null && String(observedVersion) === String(version);
        var nameMatches = typeof observedName === 'string' && observedName.trim() === String(displayName).trim();
        var bindingVerified = idMatches && versionMatches && nameMatches;
        if (!bindingVerified) {{
            return {{
                error: "TradingView active saved-script binding did not match the request",
                kind: "internal_api_unavailable",
                name: displayName,
                capabilities: capabilities,
                slot_rebound: false,
                binding_verified: false,
                observed_script: {{ name: observedName, version: observedVersion }},
                editor_open_before: editorOpenBefore,
                opened_editor: !editorOpenBefore,
                next_action_hint: "Do not save from this editor state; reopen the intended script in TradingView and verify it manually."
            }};
        }}

        return {{
            success: true,
            name: displayName,
            script_id: id,
            version: version,
            line_count: source.split(/\r?\n/).length,
            slot_rebound: true,
            binding_verified: true,
            observed_script: {{ name: observedName.trim(), version: observedVersion }},
            capabilities: capabilities,
            editor_open_before: editorOpenBefore,
            opened_editor: !editorOpenBefore
        }};
    }} catch (e) {{
        return {{
            error: "TradingView saved-script binding operation failed",
            kind: "internal_api_unavailable",
            capabilities: capabilities,
            slot_rebound: false,
            binding_verified: false,
            next_action_hint: "Do not save from this editor state; retry after confirming Pine Editor is available."
        }};
    }}
}})()
"#
    )
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
    use std::process::Command;

    use serde_json::json;

    use crate::ops::test_support::FakeRuntime;

    use super::*;

    fn execute_pine_open_fixture(setup: &str) -> Value {
        let expression = pine_open_expression("Saved Script");
        let script = format!(
            r#"
{setup}
Promise.resolve({expression}).then(function(result) {{
    process.stdout.write(JSON.stringify(result));
}}).catch(function(error) {{
    process.stderr.write(String(error && error.stack || error));
    process.exit(1);
}});
"#
        );
        let output = Command::new("node")
            .args(["-e", &script])
            .output()
            .expect("Node.js is required to execute the Pine open JavaScript contract fixture");
        assert!(
            output.status.success(),
            "Pine open fixture failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).expect("Pine open fixture should return JSON")
    }

    #[test]
    #[ignore = "run through scripts/check-pine-open-js-contract.py with pinned Node.js"]
    fn javascript_pine_open_contract_is_fail_closed_and_verifies_binding() {
        let success = execute_pine_open_fixture(
            r#"
	var active = { scriptIdPart: 'old', version: 1, scriptName: 'Old Script' };
	var api = {
	    _pineEditor: null,
	    openEditor: function() {
	        var self = this;
	        return Promise.resolve().then(function() {
	            self._pineEditor = { _storeProvider: { getEditorActiveScript: function() { return active; } } };
	        });
	    },
	    openScript: function(request) {
	        return Promise.resolve().then(function() {
	            active = { scriptIdPart: request.scriptIdPart, version: request.version, scriptName: 'Saved Script' };
	        });
	    }
	};
global.window = { TradingViewApi: { pineEditorTestApi: function() { return api; } } };
global.fetch = function(url) {
    if (url.indexOf('/list/') !== -1) {
        return Promise.resolve({ ok: true, json: function() { return Promise.resolve([
            { scriptName: 'Saved Script', scriptIdPart: 'abc123', version: 4 }
        ]); } });
    }
    return Promise.resolve({ ok: true, json: function() { return Promise.resolve({ source: 'line1\nline2' }); } });
};
"#,
        );
        assert_eq!(success["slot_rebound"], true);
        assert_eq!(success["binding_verified"], true);
        assert_eq!(success["observed_script"]["name"], "Saved Script");
        assert_eq!(success["line_count"], 2);
        assert_eq!(success["editor_open_before"], false);
        assert_eq!(success["opened_editor"], true);

        let unique_partial = execute_pine_open_fixture(
            r#"
var active = { scriptIdPart: 'old', version: 1, scriptName: 'Old Script' };
var api = {
    _pineEditor: { _storeProvider: { getEditorActiveScript: function() { return active; } } },
    openEditor: function() { return Promise.resolve(); },
    openScript: function(request) {
        return Promise.resolve().then(function() {
            active = { scriptIdPart: request.scriptIdPart, version: request.version, scriptName: 'My Saved Script' };
        });
    }
};
global.window = { TradingViewApi: { pineEditorTestApi: function() { return api; } } };
global.fetch = function(url) {
    if (url.indexOf('/list/') !== -1) {
        return Promise.resolve({ ok: true, json: function() { return Promise.resolve([
            { scriptName: 'My Saved Script', scriptIdPart: 'partial-id', version: 5 }
        ]); } });
    }
    return Promise.resolve({ ok: true, json: function() { return Promise.resolve({ source: 'line1' }); } });
};
"#,
        );
        assert_eq!(unique_partial["binding_verified"], true);
        assert_eq!(unique_partial["name"], "My Saved Script");

        let ambiguous = execute_pine_open_fixture(
            r#"
var api = {
    _pineEditor: null,
    openEditor: function() { throw new Error('openEditor must not run for ambiguity'); },
    openScript: function() { throw new Error('openScript must not run for ambiguity'); }
};
global.window = { TradingViewApi: { pineEditorTestApi: function() { return api; } } };
global.fetch = function() {
    return Promise.resolve({ ok: true, json: function() { return Promise.resolve([
        { scriptName: 'Saved Script', scriptIdPart: 'first', version: 1 },
        { scriptName: 'Saved Script', scriptIdPart: 'second', version: 2 }
    ]); } });
};
"#,
        );
        assert_eq!(ambiguous["kind"], "validation");
        assert_eq!(ambiguous["candidate_count"], 2);

        let open_editor_rejection = execute_pine_open_fixture(
            r#"
var api = {
    _pineEditor: null,
    openEditor: function() { return Promise.reject(new Error('private-open-editor')); },
    openScript: function() { throw new Error('openScript must not run after openEditor rejection'); }
};
global.window = { TradingViewApi: { pineEditorTestApi: function() { return api; } } };
global.fetch = function(url) {
    if (url.indexOf('/list/') !== -1) {
        return Promise.resolve({ ok: true, json: function() { return Promise.resolve([
            { scriptName: 'Saved Script', scriptIdPart: 'abc123', version: 4 }
        ]); } });
    }
    return Promise.resolve({ ok: true, json: function() { return Promise.resolve({ source: 'line1' }); } });
};
"#,
        );
        assert_eq!(open_editor_rejection["kind"], "internal_api_unavailable");
        assert!(
            !open_editor_rejection
                .to_string()
                .contains("private-open-editor")
        );

        let open_script_rejection = execute_pine_open_fixture(
            r#"
var active = { scriptIdPart: 'old', version: 1, scriptName: 'Old Script' };
var api = {
    _pineEditor: { _storeProvider: { getEditorActiveScript: function() { return active; } } },
    openEditor: function() { return Promise.resolve(); },
    openScript: function() { return Promise.reject(new Error('private-open-script')); }
};
global.window = { TradingViewApi: { pineEditorTestApi: function() { return api; } } };
global.fetch = function(url) {
    if (url.indexOf('/list/') !== -1) {
        return Promise.resolve({ ok: true, json: function() { return Promise.resolve([
            { scriptName: 'Saved Script', scriptIdPart: 'abc123', version: 4 }
        ]); } });
    }
    return Promise.resolve({ ok: true, json: function() { return Promise.resolve({ source: 'line1' }); } });
};
"#,
        );
        assert_eq!(open_script_rejection["kind"], "internal_api_unavailable");
        assert!(
            !open_script_rejection
                .to_string()
                .contains("private-open-script")
        );

        let unavailable = execute_pine_open_fixture(
            r#"
global.window = { TradingViewApi: { pineEditorTestApi: function() { return { openEditor: function() {} }; } } };
global.fetch = function() { throw new Error('fetch must not run when methods are unavailable'); };
"#,
        );
        assert_eq!(unavailable["kind"], "internal_api_unavailable");
        assert_eq!(unavailable["slot_rebound"], false);
        assert_eq!(unavailable["binding_verified"], false);

        let missing_readback = execute_pine_open_fixture(
            r#"
var api = {
    _pineEditor: null,
    openEditor: function() { this._pineEditor = { _storeProvider: {} }; return Promise.resolve(); },
    openScript: function() { throw new Error('openScript must not run without readback'); }
};
global.window = { TradingViewApi: { pineEditorTestApi: function() { return api; } } };
global.fetch = function(url) {
    if (url.indexOf('/list/') !== -1) {
        return Promise.resolve({ ok: true, json: function() { return Promise.resolve([
            { scriptName: 'Saved Script', scriptIdPart: 'abc123', version: 4 }
        ]); } });
    }
    return Promise.resolve({ ok: true, json: function() { return Promise.resolve({ source: 'line1' }); } });
};
"#,
        );
        assert_eq!(missing_readback["kind"], "internal_api_unavailable");
        assert_eq!(
            missing_readback["capabilities"]["active_readback_available"],
            false
        );
        assert_eq!(missing_readback["slot_rebound"], false);

        let mismatch = execute_pine_open_fixture(
            r#"
var active = { scriptIdPart: 'old', version: 1, scriptName: 'Old Script' };
var api = {
    _pineEditor: { _storeProvider: { getEditorActiveScript: function() { return active; } } },
    openEditor: function() { return Promise.resolve(); },
    openScript: function() { return Promise.resolve(); }
};
global.window = { TradingViewApi: { pineEditorTestApi: function() { return api; } } };
global.fetch = function(url) {
    if (url.indexOf('/list/') !== -1) {
        return Promise.resolve({ ok: true, json: function() { return Promise.resolve([
            { scriptName: 'Saved Script', scriptIdPart: 'abc123', version: 4 }
        ]); } });
    }
    return Promise.resolve({ ok: true, json: function() { return Promise.resolve({ source: 'line1' }); } });
};
"#,
        );
        assert_eq!(mismatch["kind"], "internal_api_unavailable");
        assert_eq!(mismatch["slot_rebound"], false);
        assert_eq!(mismatch["binding_verified"], false);
        assert_eq!(mismatch["observed_script"]["name"], "Old Script");

        let id_mismatch = execute_pine_open_fixture(
            r#"
var active = { scriptIdPart: 'different-id', version: 4, scriptName: 'Saved Script' };
var api = {
    _pineEditor: { _storeProvider: { getEditorActiveScript: function() { return active; } } },
    openEditor: function() { return Promise.resolve(); },
    openScript: function() { return Promise.resolve(); }
};
global.window = { TradingViewApi: { pineEditorTestApi: function() { return api; } } };
global.fetch = function(url) {
    if (url.indexOf('/list/') !== -1) {
        return Promise.resolve({ ok: true, json: function() { return Promise.resolve([
            { scriptName: 'Saved Script', scriptIdPart: 'abc123', version: 4 }
        ]); } });
    }
    return Promise.resolve({ ok: true, json: function() { return Promise.resolve({ source: 'line1' }); } });
};
"#,
        );
        assert_eq!(id_mismatch["binding_verified"], false);
        assert_eq!(id_mismatch["observed_script"]["version"], 4);

        let version_mismatch = execute_pine_open_fixture(
            r#"
var active = { scriptIdPart: 'abc123', version: 3, scriptName: 'Saved Script' };
var api = {
    _pineEditor: { _storeProvider: { getEditorActiveScript: function() { return active; } } },
    openEditor: function() { return Promise.resolve(); },
    openScript: function() { return Promise.resolve(); }
};
global.window = { TradingViewApi: { pineEditorTestApi: function() { return api; } } };
global.fetch = function(url) {
    if (url.indexOf('/list/') !== -1) {
        return Promise.resolve({ ok: true, json: function() { return Promise.resolve([
            { scriptName: 'Saved Script', scriptIdPart: 'abc123', version: 4 }
        ]); } });
    }
    return Promise.resolve({ ok: true, json: function() { return Promise.resolve({ source: 'line1' }); } });
};
"#,
        );
        assert_eq!(version_mismatch["binding_verified"], false);
        assert_eq!(version_mismatch["observed_script"]["name"], "Saved Script");

        let name_mismatch = execute_pine_open_fixture(
            r#"
var active = { scriptIdPart: 'abc123', version: 4, scriptName: 'Different Name' };
var api = {
    _pineEditor: { _storeProvider: { getEditorActiveScript: function() { return active; } } },
    openEditor: function() { return Promise.resolve(); },
    openScript: function() { return Promise.resolve(); }
};
global.window = { TradingViewApi: { pineEditorTestApi: function() { return api; } } };
global.fetch = function(url) {
    if (url.indexOf('/list/') !== -1) {
        return Promise.resolve({ ok: true, json: function() { return Promise.resolve([
            { scriptName: 'Saved Script', scriptIdPart: 'abc123', version: 4 }
        ]); } });
    }
    return Promise.resolve({ ok: true, json: function() { return Promise.resolve({ source: 'line1' }); } });
};
"#,
        );
        assert_eq!(name_mismatch["kind"], "internal_api_unavailable");
        assert_eq!(name_mismatch["binding_verified"], false);
        assert_eq!(name_mismatch["observed_script"]["name"], "Different Name");

        let throwing_readback = execute_pine_open_fixture(
            r#"
var api = {
    _pineEditor: { _storeProvider: { getEditorActiveScript: function() { throw new Error('private'); } } },
    openEditor: function() { return Promise.resolve(); },
    openScript: function() { return Promise.resolve(); }
};
global.window = { TradingViewApi: { pineEditorTestApi: function() { return api; } } };
global.fetch = function(url) {
    if (url.indexOf('/list/') !== -1) {
        return Promise.resolve({ ok: true, json: function() { return Promise.resolve([
            { scriptName: 'Saved Script', scriptIdPart: 'abc123', version: 4 }
        ]); } });
    }
    return Promise.resolve({ ok: true, json: function() { return Promise.resolve({ source: 'line1' }); } });
};
"#,
        );
        assert_eq!(throwing_readback["kind"], "internal_api_unavailable");
        assert_eq!(throwing_readback["binding_verified"], false);
        assert!(!throwing_readback.to_string().contains("private"));
    }

    #[tokio::test]
    async fn pine_open_returns_success_payload() {
        let mut runtime = FakeRuntime::new([json!({
            "success": true,
            "name": "Saved Script",
            "script_id": "abc123",
            "version": 4,
            "line_count": 3,
            "slot_rebound": true,
            "binding_verified": true,
            "observed_script": {"name": "Saved Script", "version": 4},
            "editor_open_before": true,
            "opened_editor": false
        })]);

        let result = pine_open(&mut runtime, "Saved Script").await.unwrap();

        assert_eq!(result["name"], "Saved Script");
        assert_eq!(result["script_id"], "abc123");
        assert_eq!(result["version"], 4);
        assert_eq!(result["lines"], 3);
        assert_eq!(result["source"], "internal_api");
        assert_eq!(result["source_category"], "desktop_backed_operation");
        assert_eq!(result["requires_desktop"], true);
        assert_eq!(result["non_mutating"], false);
        assert_eq!(result["opened"], true);
        assert_eq!(result["slot_rebound"], true);
        assert_eq!(result["binding_verified"], true);
        assert_eq!(result["binding_method"], "pine_editor_internal_api");
        assert_eq!(result["requested_script"]["name"], "Saved Script");
        assert_eq!(result["observed_script"]["name"], "Saved Script");
        assert_eq!(result["editor_open_before"], true);
        assert_eq!(result["opened_editor"], false);
        assert!(runtime.evaluated[0].0.contains("pine-facade/list"));
        assert!(runtime.evaluated[0].0.contains("pine-facade/get"));
        assert!(runtime.evaluated[0].0.contains("pineEditorTestApi"));
        assert!(runtime.evaluated[0].0.contains("getEditorActiveScript"));
        assert!(!runtime.evaluated[0].0.contains("setValue"));
        assert!(runtime.evaluated[0].1);
    }

    #[tokio::test]
    async fn pine_open_maps_missing_script_to_validation() {
        let mut runtime = FakeRuntime::new([json!({
            "error": "Script \"missing\" not found. Use pine list to see available scripts.",
            "kind": "validation"
        })]);

        let error = pine_open(&mut runtime, "missing").await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(error.message.contains("not found"));
    }

    #[tokio::test]
    async fn pine_open_maps_ambiguous_match_to_validation_with_candidates() {
        let mut runtime = FakeRuntime::new([json!({
            "error": "Multiple Pine scripts match \"test\"",
            "kind": "validation",
            "candidate_count": 2,
            "matches": [
                {"name": "Test A", "id": "must-not-leak"},
                {"name": "Test B", "version": 7}
            ]
        })]);

        let error = pine_open(&mut runtime, "test").await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(error.message.contains("Multiple Pine scripts match"));
        let details = error.details.unwrap();
        assert_eq!(details["matches"][0], json!({"name": "Test A"}));
        assert_eq!(details["candidate_count"], 2);
        assert!(!details.to_string().contains("must-not-leak"));
    }

    #[tokio::test]
    async fn pine_open_rejects_empty_source_payload() {
        let mut runtime = FakeRuntime::new([json!({
            "error": "Script source is empty",
            "kind": "internal_api_unavailable",
            "name": "Empty Script"
        })]);

        let error = pine_open(&mut runtime, "Empty Script").await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.message, "Script source is empty");
    }

    #[tokio::test]
    async fn pine_open_rejects_malformed_success_payload() {
        let mut runtime = FakeRuntime::new([json!({
            "success": true,
            "script_id": "private-id",
            "raw": {"source": "private source"}
        })]);

        let error = pine_open(&mut runtime, "Broken").await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(
            error.message,
            "Pine open payload did not include script name"
        );
        let details = error.details.unwrap().to_string();
        assert!(!details.contains("private-id"));
        assert!(!details.contains("private source"));
    }

    #[tokio::test]
    async fn pine_open_sanitizes_runtime_evaluation_failure() {
        let runtime_error = AppError::new(
            ErrorKind::InternalApiUnavailable,
            "private runtime exception description",
        )
        .with_details(json!({
            "exceptionDetails": {
                "description": "private stack and source",
                "scriptId": "private-script-id"
            }
        }));
        let mut runtime = FakeRuntime::new([]).with_evaluate_app_error(runtime_error);

        let error = pine_open(&mut runtime, "Saved Script").await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.message, "Pine saved-script binding evaluation failed");
        let serialized = error.details.unwrap().to_string();
        assert!(!serialized.contains("private runtime"));
        assert!(!serialized.contains("private stack"));
        assert!(!serialized.contains("private-script-id"));
        assert!(serialized.contains("pine_open"));
    }

    #[tokio::test]
    async fn pine_open_rejects_contradictory_success_payloads() {
        let base = json!({
            "success": true,
            "name": "Saved Script",
            "script_id": "abc123",
            "version": 4,
            "line_count": 3,
            "slot_rebound": true,
            "binding_verified": true,
            "observed_script": {"name": "Saved Script", "version": 4}
        });
        let mut fixtures = Vec::new();

        let mut missing_success = base.clone();
        missing_success.as_object_mut().unwrap().remove("success");
        fixtures.push(missing_success);

        let mut invalid_version = base.clone();
        invalid_version["version"] = json!(true);
        fixtures.push(invalid_version);

        let mut missing_observed_version = base.clone();
        missing_observed_version["observed_script"] = json!({"name": "Saved Script"});
        fixtures.push(missing_observed_version);

        let mut mismatched_version = base.clone();
        mismatched_version["observed_script"]["version"] = json!(3);
        fixtures.push(mismatched_version);

        let mut mismatched_name = base;
        mismatched_name["observed_script"]["name"] = json!("Different Name");
        fixtures.push(mismatched_name);

        for payload in fixtures {
            let mut runtime = FakeRuntime::new([payload]);
            let error = pine_open(&mut runtime, "Saved Script").await.unwrap_err();
            assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
            assert!(!error.details.unwrap().to_string().contains("abc123"));
        }
    }

    #[tokio::test]
    async fn pine_open_rejects_unverified_binding() {
        let mut runtime = FakeRuntime::new([json!({
            "success": true,
            "name": "Saved Script",
            "script_id": "abc123",
            "version": 4,
            "line_count": 3,
            "slot_rebound": false,
            "binding_verified": false,
            "observed_script": {"name": "Other Script", "version": 2}
        })]);

        let error = pine_open(&mut runtime, "Saved Script").await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(
            error.message,
            "Pine open did not verify the active saved-script binding"
        );
        let details = error.details.unwrap();
        assert_eq!(details["binding_verified"], false);
        assert_eq!(details["observed_script"]["name"], "Other Script");
        assert!(!details.to_string().contains("abc123"));
    }

    #[tokio::test]
    async fn pine_open_sanitizes_internal_api_failure() {
        let mut runtime = FakeRuntime::new([json!({
            "error": "TradingView saved-script binding operation failed",
            "kind": "internal_api_unavailable",
            "script_id": "private-id",
            "raw": {"source": "private source"},
            "observed_script": {"name": "Observed", "version": 4, "script_id": "observed-private"},
            "capabilities": {
                "factory_available": true,
                "open_editor_available": true,
                "open_script_available": true,
                "active_readback_available": false,
                "raw": "capability-private"
            },
            "next_action_hint": "Do not save from this editor state."
        })]);

        let error = pine_open(&mut runtime, "Saved Script").await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        let serialized = error.details.unwrap().to_string();
        assert!(!serialized.contains("private-id"));
        assert!(!serialized.contains("private source"));
        assert!(!serialized.contains("observed-private"));
        assert!(!serialized.contains("capability-private"));
        assert!(serialized.contains("active_readback_available"));
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
