use serde_json::{Value, json};

use tradingview_cdp::{KeyEventType, RuntimeEvaluator};
use tradingview_core::{AppError, ErrorKind};

use super::runtime::{
    FIND_MONACO, PINE_SAVE_WAIT, dispatch_key, ensure_pine_editor_open, with_monaco,
};

pub async fn pine_open(runtime: &mut impl RuntimeEvaluator, name: &str) -> Result<Value, AppError> {
    let open_state = ensure_pine_editor_open(runtime).await?;
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
    let script_id_available = raw
        .get("script_id_available")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let script_identity_verified = raw
        .get("script_identity_verified")
        .and_then(Value::as_bool)
        .unwrap_or(false);
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
        || !script_id_available
        || !script_identity_verified
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
        "script_id_available": script_id_available,
        "script_identity_verified": script_identity_verified,
        "version": raw.get("version").cloned().unwrap_or(Value::Null),
        "lines": raw.get("line_count").cloned().unwrap_or(Value::Null),
        "source": "internal_api",
        "source_category": "desktop_backed_operation",
        "requires_desktop": true,
        "non_mutating": false,
        "opened": true,
        "switch_performed": raw.get("switch_performed").and_then(Value::as_bool).unwrap_or(false),
        "slot_rebound": slot_rebound,
        "binding_verified": binding_verified,
        "binding_method": "pine_editor_overlay_state",
        "requested_script": {
            "name": name,
        },
        "observed_script": {
            "name": observed_name,
            "version": raw.pointer("/observed_script/version").cloned().unwrap_or(Value::Null),
        },
        "editor_open_before": open_state.editor_open_before,
        "opened_editor": open_state.opened_editor,
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
        "pine_owner_available": raw.pointer("/capabilities/pine_owner_available").and_then(Value::as_bool).unwrap_or(false),
        "menu_open_available": raw.pointer("/capabilities/menu_open_available").and_then(Value::as_bool).unwrap_or(false),
        "menu_scope_available": raw.pointer("/capabilities/menu_scope_available").and_then(Value::as_bool).unwrap_or(false),
        "menu_selection_available": raw.pointer("/capabilities/menu_selection_available").and_then(Value::as_bool).unwrap_or(false),
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
        .await
        .map_err(|error| pine_save_evaluation_error(error, "preflight"))?;
    if let Some(error) = before.get("error").and_then(Value::as_str) {
        return Err(pine_save_error(error, &before, "preflight"));
    }

    dispatch_key(
        runtime,
        KeyEventType::KeyDown,
        "s",
        "KeyS",
        83,
        pine_save_modifier_mask(),
    )
    .await?;
    dispatch_key(runtime, KeyEventType::KeyUp, "s", "KeyS", 83, 0).await?;
    tokio::time::sleep(PINE_SAVE_WAIT).await;

    let raw = runtime
        .evaluate(&pine_save_post_shortcut_expression(&before), true)
        .await
        .map_err(|error| pine_save_evaluation_error(error, "post_shortcut"))?;
    if let Some(error) = raw.get("error").and_then(Value::as_str) {
        return Err(pine_save_error(error, &raw, "post_shortcut"));
    }
    let saved = raw.get("saved").and_then(Value::as_bool);
    let dirty_after = raw.get("dirty_after").and_then(Value::as_bool);
    if dirty_after == Some(true) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Pine save did not clear the dirty state",
        )
        .with_details(pine_save_outcome_details(&raw)));
    }
    if saved != Some(true) || dirty_after != Some(false) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Pine save outcome was not explicitly verified",
        )
        .with_details(pine_save_outcome_details(&raw)));
    }

    Ok(json!({
        "saved": true,
        "action": "saved",
        "name": null,
        "dialog_handled": raw.get("dialog_handled").and_then(Value::as_bool).unwrap_or(false),
        "source": "dom_fallback",
        "editor_open_before": open_state.editor_open_before,
        "opened_editor": open_state.opened_editor,
        "dirty_before": raw.get("dirty_before").and_then(Value::as_bool).or_else(|| before.get("dirty_before").and_then(Value::as_bool)),
        "dirty_after": false,
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

fn pine_save_error(message: &str, details: &Value, stage: &str) -> AppError {
    let kind = match details.get("kind").and_then(Value::as_str) {
        Some("validation") => ErrorKind::Validation,
        _ => ErrorKind::InternalApiUnavailable,
    };
    let public_message = match message {
        "Pine save requires an already saved script; naming unsaved scripts is deferred" => message,
        _ => "Pine save page-side check failed",
    };
    AppError::new(kind, public_message).with_details(json!({
        "operation": "pine_save",
        "stage": stage,
        "dialog_open": details.get("dialog_open").and_then(Value::as_bool),
        "dirty_before": details.get("dirty_before").and_then(Value::as_bool),
        "next_action_hint": "The script was not confirmed saved; verify the Pine Editor state before retrying.",
    }))
}

fn pine_save_outcome_details(raw: &Value) -> Value {
    let source = match raw.get("source").and_then(Value::as_str) {
        Some("dom_fallback") => Some("dom_fallback"),
        _ => None,
    };
    json!({
        "operation": "pine_save",
        "stage": "post_shortcut",
        "saved": raw.get("saved").and_then(Value::as_bool),
        "dirty_before": raw.get("dirty_before").and_then(Value::as_bool),
        "dirty_after": raw.get("dirty_after").and_then(Value::as_bool),
        "dialog_handled": raw.get("dialog_handled").and_then(Value::as_bool),
        "source": source,
        "next_action_hint": "The script was not confirmed saved; verify the Pine Editor state before retrying.",
    })
}

fn pine_save_evaluation_error(error: AppError, stage: &str) -> AppError {
    AppError::new(error.kind, "Pine save evaluation failed").with_details(json!({
        "operation": "pine_save",
        "stage": stage,
        "next_action_hint": "The script was not confirmed saved; verify the Pine Editor state before retrying.",
    }))
}

const fn pine_save_modifier_mask() -> i64 {
    pine_save_modifier_mask_for(cfg!(target_os = "macos"))
}

const fn pine_save_modifier_mask_for(is_macos: bool) -> i64 {
    // CDP uses Meta for Command on macOS and Control on Windows/Linux.
    if is_macos { 4 } else { 2 }
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
	    var selected = __FIND_MONACO__;
	    var capabilities = {{
	        pine_owner_available: !!(selected && selected.owner && selected.container),
	        menu_open_available: false,
	        menu_scope_available: false,
	        menu_selection_available: false,
	        active_readback_available: false
    }};
    if (!capabilities.pine_owner_available) {{
        return {{
            error: "TradingView Pine Editor ownership is unavailable",
            kind: "internal_api_unavailable",
            capabilities: capabilities,
            slot_rebound: false,
            binding_verified: false,
            next_action_hint: "Do not save from this editor state; use a supported TradingView Desktop build and retry."
        }};
    }}

    function visible(element) {{
        if (!element || typeof element.getBoundingClientRect !== 'function') return false;
        try {{
            var rect = element.getBoundingClientRect();
            var style = window.getComputedStyle ? window.getComputedStyle(element) : null;
            return rect.width > 0 && rect.height > 0
                && (!style || (style.display !== 'none' && style.visibility !== 'hidden'));
        }} catch (e) {{ return false; }}
    }}
    function normalize(value) {{ return typeof value === 'string' ? value.trim() : ''; }}
    function stateFromStore(store) {{
        try {{ return store && typeof store.getState === 'function' ? store.getState() : null; }} catch (e) {{ return null; }}
    }}
    function activeFromStore(store) {{
        var state = stateFromStore(store);
        return state && state.script && typeof state.script === 'object' ? state.script : null;
    }}
    function findStore(node) {{
        var stores = [];
        var element = node;
        var fiber = null;
        for (var up = 0; up < 20 && element; up++, element = element.parentElement) {{
            var key = null;
            try {{ key = Object.keys(element).find(function(item) {{ return item.startsWith('__reactFiber$'); }}); }} catch (e) {{}}
            if (key) {{ fiber = element[key]; break; }}
        }}
        for (var depth = 0; depth < 120 && fiber; depth++, fiber = fiber.return) {{
            var props = fiber.memoizedProps;
            var store = props && props.store;
            var state = stateFromStore(store);
            if (state && state.script && state.openScript && state.saveScript && stores.indexOf(store) === -1) stores.push(store);
        }}
        return stores.length === 1 ? stores[0] : null;
    }}
    function identityMatches(active, match, displayName, version) {{
        if (!active || !match) return false;
        var observedName = active.scriptName || active.scriptTitle || null;
        return active.scriptIdPart != null
            && String(active.scriptIdPart) === String(match.scriptIdPart)
            && active.version != null
            && String(active.version) === String(version)
            && normalize(observedName) === normalize(displayName);
    }}
    async function waitFor(find, deadline) {{
        while (Date.now() < deadline) {{
            var value = null;
            try {{ value = find(); }} catch (e) {{ value = null; }}
            if (value) return value;
            await new Promise(function(resolve) {{ setTimeout(resolve, 50); }});
        }}
        return null;
    }}
    function visiblePopupRoots() {{
        try {{
            return Array.from(document.querySelectorAll('[role="menu"], [role="listbox"]')).filter(visible);
        }} catch (e) {{
            return [];
        }}
    }}
    function isPopupRoot(element) {{
        if (!visible(element)) return false;
        var role = null;
        try {{ role = element.getAttribute && element.getAttribute('role'); }} catch (e) {{ role = null; }}
        return role === 'menu' || role === 'listbox';
    }}
    function linkedPopupRoots(trigger) {{
        var ids = [];
        ['aria-controls', 'aria-owns'].forEach(function(attribute) {{
            var value = null;
            try {{ value = trigger.getAttribute && trigger.getAttribute(attribute); }} catch (e) {{ value = null; }}
            if (typeof value === 'string') {{
                value.split(/\s+/).filter(Boolean).forEach(function(id) {{
                    if (ids.indexOf(id) === -1) ids.push(id);
                }});
            }}
        }});
        var roots = ids.map(function(id) {{
            try {{ return document.getElementById(id); }} catch (e) {{ return null; }}
        }}).filter(isPopupRoot);
        return roots.filter(function(root, index) {{ return roots.indexOf(root) === index; }});
    }}
    function openedPopupRoot(trigger, before) {{
        var linked = linkedPopupRoots(trigger);
        if (linked.length === 1) return linked[0];
        if (linked.length > 1) return null;
        var opened = visiblePopupRoots().filter(function(root) {{ return before.indexOf(root) === -1; }});
        return opened.length === 1 ? opened[0] : null;
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

        var store = findStore(selected.container);
        capabilities.active_readback_available = !!store;
        if (!capabilities.active_readback_available) {{
            return {{
                error: "TradingView active saved-script readback is unavailable",
                kind: "internal_api_unavailable",
                name: displayName,
                capabilities: capabilities,
                slot_rebound: false,
                binding_verified: false,
                next_action_hint: "Do not save from this editor state; active script identity could not be verified."
            }};
        }}

        var operationDeadline = Date.now() + 8000;
        var activeBefore = activeFromStore(store);
        var switchPerformed = !identityMatches(activeBefore, match, displayName, version);
        if (switchPerformed) {{
            var currentName = activeBefore && (activeBefore.scriptName || activeBefore.scriptTitle) || null;
            var triggers = Array.from(selected.owner.querySelectorAll('[role="button"][aria-haspopup="menu"]'))
                .filter(visible)
                .filter(function(element) {{ return normalize(element.textContent) === normalize(currentName); }});
            capabilities.menu_open_available = triggers.length === 1;
            if (!capabilities.menu_open_available) {{
                return {{
                    error: "TradingView saved-script menu is unavailable",
                    kind: "internal_api_unavailable",
                    name: displayName,
                    capabilities: capabilities,
                    slot_rebound: false,
                    binding_verified: false
                }};
            }}
            var trigger = triggers[0];
            var popupRootsBefore = visiblePopupRoots();
            trigger.click();
            var menuDeadline = Math.min(operationDeadline, Date.now() + 2500);
            var menuRoot = await waitFor(function() {{
                return openedPopupRoot(trigger, popupRootsBefore);
            }}, menuDeadline);
            capabilities.menu_scope_available = !!menuRoot;
            if (!menuRoot) {{
                return {{
                    error: "TradingView saved-script menu ownership is unavailable or ambiguous",
                    kind: "internal_api_unavailable",
                    name: displayName,
                    capabilities: capabilities,
                    slot_rebound: false,
                    binding_verified: false
                }};
            }}
            var row = await waitFor(function() {{
                var rows = Array.from(menuRoot.querySelectorAll('[role="menuitemcheckbox"], [role="menuitemradio"], [role="option"]'))
                    .filter(visible)
                    .filter(function(element) {{ return normalize(element.textContent) === normalize(displayName); }});
                return rows.length === 1 ? rows[0] : null;
            }}, menuDeadline);
            capabilities.menu_selection_available = !!row;
            if (!row) {{
                return {{
                    error: "TradingView saved-script menu item is unavailable or ambiguous",
                    kind: "internal_api_unavailable",
                    name: displayName,
                    capabilities: capabilities,
                    slot_rebound: false,
                    binding_verified: false
                }};
            }}
            row.click();
        }} else {{
            capabilities.menu_open_available = true;
            capabilities.menu_scope_available = true;
            capabilities.menu_selection_available = true;
        }}

        var active = await waitFor(function() {{
            var observed = activeFromStore(store);
            return identityMatches(observed, match, displayName, version) ? observed : null;
        }}, operationDeadline);
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
                next_action_hint: "Do not save from this editor state; reopen the intended script in TradingView and verify it manually."
            }};
        }}

        return {{
            success: true,
            name: displayName,
            script_id_available: true,
            script_identity_verified: idMatches,
            version: version,
            line_count: source.split(/\r?\n/).length,
            slot_rebound: true,
            binding_verified: true,
            switch_performed: switchPerformed,
            observed_script: {{ name: observedName.trim(), version: observedVersion }},
            capabilities: capabilities
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
    .replace("__FIND_MONACO__", FIND_MONACO)
}

fn pine_save_preflight_expression() -> String {
    with_monaco(
        r#"
function dirtyState() {
    try {
        var buttons = Array.from(document.querySelectorAll('button, [role="button"], span, div'));
        var dirty = null;
        for (var i = 0; i < buttons.length; i++) {
            var text = (buttons[i].textContent || buttons[i].getAttribute('aria-label') || buttons[i].getAttribute('title') || '').trim();
            if (!text) continue;
            if (/unsaved version/i.test(text) || /未保存/.test(text)) dirty = true;
            if (/^saved$/i.test(text) || /^保存済み$/.test(text)) dirty = false;
        }
        return dirty;
    } catch(e) {
        return null;
    }
}
function finish() {
    return { ok: true, dirty_before: dirtyState() };
}
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
                saved: dirtyAfter === false,
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

    fn execute_pine_expression_fixture(expression: &str, setup: &str) -> Value {
        let script = format!(
            r#"
function installFixture(options) {{
    options = options || {{}};
    var menuOpen = false;
    var menuDeadlineAdjusted = false;
    var list = options.list || [{{ scriptName: 'Saved Script', scriptIdPart: 'abc123', version: 4 }}];
    var state = {{
        script: options.active || {{ scriptName: 'Old Script', scriptIdPart: 'old', version: 1 }},
        openScript: {{ status: 'idle' }},
        saveScript: {{ status: 'saved' }}
    }};
    var store = {{ getState: function() {{ return state; }} }};
    var row = {{
        textContent: 'Saved Script',
        getBoundingClientRect: function() {{ return {{ width: 120, height: 32 }}; }},
        click: function() {{
            if (options.bindingMismatch) {{
                state.script = {{ scriptName: 'Saved Script', scriptIdPart: 'different', version: 4 }};
            }} else {{
                state.script = {{ scriptName: 'Saved Script', scriptIdPart: 'abc123', version: 4 }};
            }}
            if (options.fastDeadline) {{
                var tick = Date.now();
                Date.now = function() {{ tick += 9000; return tick; }};
            }}
        }}
    }};
    var unrelatedRow = {{
        textContent: 'Saved Script',
        getBoundingClientRect: function() {{ return {{ width: 120, height: 32 }}; }},
        click: function() {{
            state.script = {{ scriptName: 'Saved Script', scriptIdPart: 'abc123', version: 4 }};
        }}
    }};
    var menuRoot = {{
        id: 'pine-saved-script-menu',
        getAttribute: function(attribute) {{ return attribute === 'role' ? 'menu' : null; }},
        getBoundingClientRect: function() {{
            return menuOpen ? {{ width: 320, height: 480 }} : {{ width: 0, height: 0 }};
        }},
        querySelectorAll: function(selector) {{
            if (options.fastMenuDeadline && !menuDeadlineAdjusted) {{
                menuDeadlineAdjusted = true;
                var menuTick = Date.now();
                Date.now = function() {{ menuTick += 9000; return menuTick; }};
            }}
            return selector.indexOf('menuitemcheckbox') !== -1 && menuOpen && !options.missingRow ? [row] : [];
        }}
    }};
    var unrelatedMenu = {{
        id: 'unrelated-menu',
        getAttribute: function(attribute) {{ return attribute === 'role' ? 'menu' : null; }},
        getBoundingClientRect: function() {{ return {{ width: 240, height: 200 }}; }},
        querySelectorAll: function(selector) {{
            return selector.indexOf('menuitemcheckbox') !== -1 && options.unrelatedExactRow ? [unrelatedRow] : [];
        }}
    }};
    var trigger = {{
        textContent: state.script.scriptName,
        getBoundingClientRect: function() {{ return {{ width: 120, height: 32 }}; }},
        getAttribute: function(attribute) {{
            return attribute === 'aria-controls' && !options.unlinkedMenu ? menuRoot.id : null;
        }},
        click: function() {{ menuOpen = true; }}
    }};
    var savedStateButton = {{
        textContent: 'Saved',
        getAttribute: function() {{ return null; }},
        getBoundingClientRect: function() {{ return {{ width: 80, height: 28 }}; }}
    }};
    var owner = {{
        getBoundingClientRect: function() {{ return {{ width: 800, height: 600 }}; }},
        querySelectorAll: function(selector) {{
            return selector.indexOf('aria-haspopup') !== -1 && !options.missingMenu ? [trigger] : [];
        }}
    }};
    var container = {{
        parentElement: null,
        closest: function(selector) {{ return selector === '[data-name="pine-dialog"]' && !options.missingOwner ? owner : null; }},
        contains: function(value) {{ return value === container; }},
        getBoundingClientRect: function() {{ return {{ width: 600, height: 400 }}; }}
    }};
    if (!options.missingStore) {{
        container['__reactFiber$fixture'] = {{ memoizedProps: {{ store: store }}, return: null }};
    }}
    var editor = {{
        getContainerDomNode: function() {{ return container; }},
        hasTextFocus: function() {{ return options.focused !== false; }}
    }};
    var editors = [editor];
    if (options.extraVisible) {{
        var secondContainer = {{
            closest: function() {{ return owner; }},
            contains: function() {{ return false; }},
            getBoundingClientRect: function() {{ return {{ width: 600, height: 400 }}; }}
        }};
        editors.push({{
            getContainerDomNode: function() {{ return secondContainer; }},
            hasTextFocus: function() {{ return false; }}
        }});
    }}
    if (options.hiddenStale) {{
        var hiddenContainer = {{
            closest: function() {{ return owner; }},
            contains: function() {{ return false; }},
            getBoundingClientRect: function() {{ return {{ width: 0, height: 0 }}; }}
        }};
        editors.unshift({{
            getContainerDomNode: function() {{ return hiddenContainer; }},
            hasTextFocus: function() {{ return false; }}
        }});
    }}
    var fiberContainers = [];
    if (options.crossRegistryVisible) {{
        var fiberContainer = {{
            parentElement: null,
            closest: function() {{ return owner; }},
            contains: function() {{ return false; }},
            getBoundingClientRect: function() {{ return {{ width: 600, height: 400 }}; }}
        }};
        var fiberEditor = {{
            getContainerDomNode: function() {{ return fiberContainer; }},
            hasTextFocus: function() {{ return false; }}
        }};
        fiberContainer['__reactFiber$environment'] = {{
            memoizedProps: {{ monacoEnv: {{ editor: {{ getEditors: function() {{ return [fiberEditor]; }} }} }} }},
            return: null
        }};
        fiberContainers.push(fiberContainer);
    }}
    global.document = {{
        activeElement: options.focused === false ? null : container,
        querySelectorAll: function(selector) {{
            if (selector === 'button, [role="button"], span, div') {{
                return options.dirtyUnknown ? [] : [savedStateButton];
            }}
            if (selector === '[data-name="pine-dialog"] .monaco-editor') return fiberContainers;
            if (selector === '[role="menu"], [role="listbox"]') {{
                var roots = options.unrelatedExactRow ? [unrelatedMenu] : [];
                if (menuOpen) roots.push(menuRoot);
                return roots;
            }}
            if (selector.indexOf('menuitemcheckbox') !== -1 && options.unrelatedExactRow) return [unrelatedRow];
            return [];
        }},
        getElementById: function(id) {{ return id === menuRoot.id ? menuRoot : null; }}
    }};
    global.window = global;
    window.monaco = {{ editor: {{ getEditors: function() {{ return editors; }} }} }};
    window.getComputedStyle = function() {{ return {{ display: 'block', visibility: 'visible' }}; }};
    global.fetch = function(url) {{
        if (url.indexOf('/list/') !== -1) {{
            return Promise.resolve({{ ok: true, json: function() {{ return Promise.resolve(list); }} }});
        }}
        return Promise.resolve({{ ok: true, json: function() {{ return Promise.resolve({{ source: 'line1\nline2' }}); }} }});
    }};
}}
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
            .expect("Node.js is required to execute the Pine JavaScript contract fixture");
        assert!(
            output.status.success(),
            "Pine JavaScript fixture failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).expect("Pine JavaScript fixture should return JSON")
    }

    fn execute_pine_open_fixture(setup: &str) -> Value {
        execute_pine_expression_fixture(&pine_open_expression("Saved Script"), setup)
    }

    fn pine_open_runtime(payload: Value) -> FakeRuntime {
        FakeRuntime::new([json!(true), payload])
    }

    #[test]
    #[ignore = "run through scripts/check-pine-open-js-contract.py with pinned Node.js"]
    fn javascript_pine_open_contract_is_fail_closed_and_verifies_binding() {
        let success = execute_pine_open_fixture("installFixture({});");
        assert_eq!(success["slot_rebound"], true);
        assert_eq!(success["binding_verified"], true);
        assert_eq!(success["observed_script"]["name"], "Saved Script");
        assert_eq!(success["line_count"], 2);
        assert_eq!(success["switch_performed"], true);
        assert_eq!(success["script_id_available"], true);
        assert_eq!(success["script_identity_verified"], true);
        assert!(success.get("script_id").is_none());
        assert_eq!(success["capabilities"]["active_readback_available"], true);
        assert_eq!(success["capabilities"]["menu_scope_available"], true);
        assert_eq!(success["capabilities"]["menu_selection_available"], true);

        let already_active = execute_pine_open_fixture(
            "installFixture({ active: { scriptName: 'Saved Script', scriptIdPart: 'abc123', version: 4 }, missingMenu: true });",
        );
        assert_eq!(already_active["binding_verified"], true);
        assert_eq!(already_active["switch_performed"], false);

        let focused_wins = execute_pine_open_fixture("installFixture({ extraVisible: true });");
        assert_eq!(focused_wins["binding_verified"], true);

        let hidden_stale_is_ignored =
            execute_pine_open_fixture("installFixture({ hiddenStale: true });");
        assert_eq!(hidden_stale_is_ignored["binding_verified"], true);

        let ambiguous_editors =
            execute_pine_open_fixture("installFixture({ focused: false, extraVisible: true });");
        assert_eq!(ambiguous_editors["kind"], "internal_api_unavailable");
        assert_eq!(ambiguous_editors["slot_rebound"], false);

        let cross_registry_ambiguity = execute_pine_open_fixture(
            "installFixture({ focused: false, crossRegistryVisible: true });",
        );
        assert_eq!(cross_registry_ambiguity["kind"], "internal_api_unavailable");
        assert_eq!(cross_registry_ambiguity["slot_rebound"], false);

        let ambiguous = execute_pine_open_fixture(
            r#"installFixture({ list: [
        { scriptName: 'Saved Script', scriptIdPart: 'first', version: 1 },
        { scriptName: 'Saved Script', scriptIdPart: 'second', version: 2 }
    ] });"#,
        );
        assert_eq!(ambiguous["kind"], "validation");
        assert_eq!(ambiguous["candidate_count"], 2);

        let unavailable = execute_pine_open_fixture("installFixture({ missingOwner: true });");
        assert_eq!(unavailable["kind"], "internal_api_unavailable");
        assert_eq!(unavailable["slot_rebound"], false);
        assert_eq!(unavailable["binding_verified"], false);

        let missing_readback = execute_pine_open_fixture("installFixture({ missingStore: true });");
        assert_eq!(missing_readback["kind"], "internal_api_unavailable");
        assert_eq!(
            missing_readback["capabilities"]["active_readback_available"],
            false
        );
        assert_eq!(missing_readback["slot_rebound"], false);

        let missing_menu = execute_pine_open_fixture("installFixture({ missingMenu: true });");
        assert_eq!(missing_menu["kind"], "internal_api_unavailable");
        assert_eq!(missing_menu["capabilities"]["menu_open_available"], false);

        let missing_row = execute_pine_open_fixture(
            "installFixture({ missingRow: true, fastMenuDeadline: true });",
        );
        assert_eq!(missing_row["kind"], "internal_api_unavailable");
        assert_eq!(missing_row["capabilities"]["menu_open_available"], true);
        assert_eq!(
            missing_row["capabilities"]["menu_selection_available"],
            false
        );

        let unrelated_exact_row = execute_pine_open_fixture(
            "installFixture({ missingRow: true, unrelatedExactRow: true, fastMenuDeadline: true });",
        );
        assert_eq!(unrelated_exact_row["kind"], "internal_api_unavailable");
        assert_eq!(unrelated_exact_row["binding_verified"], false);
        assert_eq!(
            unrelated_exact_row["capabilities"]["menu_selection_available"],
            false
        );

        let mismatch = execute_pine_open_fixture(
            "installFixture({ bindingMismatch: true, fastDeadline: true });",
        );
        assert_eq!(mismatch["kind"], "internal_api_unavailable");
        assert_eq!(mismatch["slot_rebound"], false);
        assert_eq!(mismatch["binding_verified"], false);
        assert!(!mismatch.to_string().contains("different"));

        let save_preflight = execute_pine_expression_fixture(
            &pine_save_preflight_expression(),
            "installFixture({ active: { scriptName: 'Saved Script', scriptIdPart: 'abc123', version: 4 } });",
        );
        assert_eq!(save_preflight["ok"], true);

        let save_post_shortcut = execute_pine_expression_fixture(
            &pine_save_post_shortcut_expression(&save_preflight),
            "installFixture({ active: { scriptName: 'Saved Script', scriptIdPart: 'abc123', version: 4 } });",
        );
        assert_eq!(save_post_shortcut["saved"], true);
        assert_eq!(save_post_shortcut["action"], "saved");

        let unknown_dirty_preflight = execute_pine_expression_fixture(
            &pine_save_preflight_expression(),
            "installFixture({ dirtyUnknown: true });",
        );
        assert_eq!(unknown_dirty_preflight["dirty_before"], Value::Null);
        let unknown_dirty_post = execute_pine_expression_fixture(
            &pine_save_post_shortcut_expression(&unknown_dirty_preflight),
            "installFixture({ dirtyUnknown: true });",
        );
        assert_eq!(unknown_dirty_post["saved"], false);
        assert_eq!(unknown_dirty_post["dirty_after"], Value::Null);
    }

    #[tokio::test]
    async fn pine_open_returns_success_payload() {
        let mut runtime = pine_open_runtime(json!({
            "success": true,
            "name": "Saved Script",
            "script_id": "private-id-that-must-not-be-returned",
            "script_id_available": true,
            "script_identity_verified": true,
            "version": 4,
            "line_count": 3,
            "slot_rebound": true,
            "binding_verified": true,
            "switch_performed": true,
            "observed_script": {"name": "Saved Script", "version": 4},
            "editor_open_before": true,
            "opened_editor": false
        }));

        let result = pine_open(&mut runtime, "Saved Script").await.unwrap();

        assert_eq!(result["name"], "Saved Script");
        assert_eq!(result["script_id_available"], true);
        assert_eq!(result["script_identity_verified"], true);
        assert!(result.get("script_id").is_none());
        assert!(
            !result
                .to_string()
                .contains("private-id-that-must-not-be-returned")
        );
        assert_eq!(result["version"], 4);
        assert_eq!(result["lines"], 3);
        assert_eq!(result["source"], "internal_api");
        assert_eq!(result["source_category"], "desktop_backed_operation");
        assert_eq!(result["requires_desktop"], true);
        assert_eq!(result["non_mutating"], false);
        assert_eq!(result["opened"], true);
        assert_eq!(result["switch_performed"], true);
        assert_eq!(result["slot_rebound"], true);
        assert_eq!(result["binding_verified"], true);
        assert_eq!(result["binding_method"], "pine_editor_overlay_state");
        assert_eq!(result["requested_script"]["name"], "Saved Script");
        assert_eq!(result["observed_script"]["name"], "Saved Script");
        assert_eq!(result["editor_open_before"], true);
        assert_eq!(result["opened_editor"], false);
        assert!(runtime.evaluated[1].0.contains("pine-facade/list"));
        assert!(runtime.evaluated[1].0.contains("pine-facade/get"));
        assert!(
            runtime.evaluated[1]
                .0
                .contains("[data-name=\"pine-dialog\"]")
        );
        assert!(runtime.evaluated[1].0.contains("state.saveScript"));
        assert!(!runtime.evaluated[1].0.contains("pineEditorTestApi"));
        assert!(!runtime.evaluated[1].0.contains("setValue"));
        assert!(runtime.evaluated[1].1);
    }

    #[tokio::test]
    async fn pine_open_maps_missing_script_to_validation() {
        let mut runtime = pine_open_runtime(json!({
            "error": "Script \"missing\" not found. Use pine list to see available scripts.",
            "kind": "validation"
        }));

        let error = pine_open(&mut runtime, "missing").await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(error.message.contains("not found"));
    }

    #[tokio::test]
    async fn pine_open_maps_ambiguous_match_to_validation_with_candidates() {
        let mut runtime = pine_open_runtime(json!({
            "error": "Multiple Pine scripts match \"test\"",
            "kind": "validation",
            "candidate_count": 2,
            "matches": [
                {"name": "Test A", "id": "must-not-leak"},
                {"name": "Test B", "version": 7}
            ]
        }));

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
        let mut runtime = pine_open_runtime(json!({
            "error": "Script source is empty",
            "kind": "internal_api_unavailable",
            "name": "Empty Script"
        }));

        let error = pine_open(&mut runtime, "Empty Script").await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.message, "Script source is empty");
    }

    #[tokio::test]
    async fn pine_open_rejects_malformed_success_payload() {
        let mut runtime = pine_open_runtime(json!({
            "success": true,
            "script_id": "private-id",
            "raw": {"source": "private source"}
        }));

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
        let mut runtime =
            FakeRuntime::new([json!(true)]).with_evaluate_app_error_after_responses(runtime_error);

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
            "script_id_available": true,
            "script_identity_verified": true,
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

        let mut mismatched_name = base.clone();
        mismatched_name["observed_script"]["name"] = json!("Different Name");
        fixtures.push(mismatched_name);

        let mut missing_id_availability = base.clone();
        missing_id_availability
            .as_object_mut()
            .unwrap()
            .remove("script_id_available");
        fixtures.push(missing_id_availability);

        let mut unverified_identity = base;
        unverified_identity["script_identity_verified"] = json!(false);
        fixtures.push(unverified_identity);

        for payload in fixtures {
            let mut runtime = pine_open_runtime(payload);
            let error = pine_open(&mut runtime, "Saved Script").await.unwrap_err();
            assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
            assert!(!error.details.unwrap().to_string().contains("abc123"));
        }
    }

    #[tokio::test]
    async fn pine_open_rejects_unverified_binding() {
        let mut runtime = pine_open_runtime(json!({
            "success": true,
            "name": "Saved Script",
            "script_id_available": true,
            "script_identity_verified": true,
            "version": 4,
            "line_count": 3,
            "slot_rebound": false,
            "binding_verified": false,
            "observed_script": {"name": "Other Script", "version": 2}
        }));

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
        let mut runtime = pine_open_runtime(json!({
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
        }));

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
                "dirty_after": false,
                "raw_source": "private source",
                "script_id": "private-script-id",
                "target_id": "private-target-id"
            }),
        ]);

        let result = pine_save(&mut runtime).await.unwrap();

        assert_eq!(result["saved"], true);
        assert_eq!(result["action"], "saved");
        assert_eq!(result["dialog_handled"], false);
        assert_eq!(result["dirty_before"], true);
        assert_eq!(result["dirty_after"], false);
        let serialized = result.to_string();
        assert!(!serialized.contains("private source"));
        assert!(!serialized.contains("private-script-id"));
        assert!(!serialized.contains("private-target-id"));
        assert_eq!(runtime.key_events.len(), 2);
        assert_eq!(runtime.key_events[0].key, "s");
        assert_eq!(runtime.key_events[0].code, "KeyS");
        assert_eq!(runtime.key_events[0].event_type, KeyEventType::KeyDown);
        #[cfg(target_os = "macos")]
        assert_eq!(runtime.key_events[0].modifiers, 4);
        #[cfg(not(target_os = "macos"))]
        assert_eq!(runtime.key_events[0].modifiers, 2);
        assert_eq!(runtime.key_events[1].event_type, KeyEventType::KeyUp);
        assert_eq!(runtime.key_events[1].modifiers, 0);
    }

    #[test]
    fn pine_save_modifier_masks_are_platform_specific() {
        assert_eq!(pine_save_modifier_mask_for(true), 4);
        assert_eq!(pine_save_modifier_mask_for(false), 2);
    }

    #[tokio::test]
    async fn pine_save_sanitizes_preflight_evaluation_failure() {
        let runtime_error = AppError::new(
            ErrorKind::InternalApiUnavailable,
            "private preflight exception",
        )
        .with_details(json!({
            "exceptionDetails": {
                "description": "private source and stack",
                "scriptId": "private-script-id",
                "objectId": "private-object-id"
            }
        }));
        let mut runtime =
            FakeRuntime::new([json!(true)]).with_evaluate_app_error_after_responses(runtime_error);

        let error = pine_save(&mut runtime).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.message, "Pine save evaluation failed");
        let details = error.details.unwrap().to_string();
        assert!(details.contains("preflight"));
        assert!(!details.contains("private preflight"));
        assert!(!details.contains("private source"));
        assert!(!details.contains("private-script-id"));
        assert!(!details.contains("private-object-id"));
        assert!(runtime.key_events.is_empty());
    }

    #[tokio::test]
    async fn pine_save_sanitizes_post_shortcut_evaluation_failure() {
        let runtime_error = AppError::new(ErrorKind::Timeout, "private post-save exception")
            .with_details(json!({
                "scriptId": "private-script-id",
                "objectId": "private-object-id"
            }));
        let mut runtime =
            FakeRuntime::new([json!(true), json!({"ok": true, "dirty_before": true})])
                .with_evaluate_app_error_after_responses(runtime_error);

        let error = pine_save(&mut runtime).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::Timeout);
        assert_eq!(error.message, "Pine save evaluation failed");
        let details = error.details.unwrap().to_string();
        assert!(details.contains("post_shortcut"));
        assert!(!details.contains("private post-save"));
        assert!(!details.contains("private-script-id"));
        assert!(!details.contains("private-object-id"));
        assert_eq!(runtime.key_events.len(), 2);
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
                "dirty_before": true,
                "raw_source": "private source",
                "script_id": "private-script-id",
                "target_id": "private-target-id"
            }),
        ]);

        let error = pine_save(&mut runtime).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert_eq!(
            error.message,
            "Pine save requires an already saved script; naming unsaved scripts is deferred"
        );
        let details = error.details.unwrap().to_string();
        assert!(!details.contains("private source"));
        assert!(!details.contains("private-script-id"));
        assert!(!details.contains("private-target-id"));
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
                "dirty_after": true,
                "raw_source": "private source",
                "script_id": "private-script-id",
                "target_id": "private-target-id"
            }),
        ]);

        let error = pine_save(&mut runtime).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.message, "Pine save did not clear the dirty state");
        let details = error.details.unwrap().to_string();
        assert!(!details.contains("private source"));
        assert!(!details.contains("private-script-id"));
        assert!(!details.contains("private-target-id"));
    }

    #[tokio::test]
    async fn pine_save_rejects_unverified_or_contradictory_outcomes() {
        let outcomes = [
            json!({"saved": true}),
            json!({"saved": true, "dirty_after": null}),
            json!({"saved": true, "dirty_after": "false"}),
            json!({"dirty_after": false}),
            json!({"saved": null, "dirty_after": false}),
            json!({"saved": "true", "dirty_after": false}),
            json!({
                "saved": false,
                "dirty_after": false,
                "raw_source": "private source",
                "script_id": "private-script-id",
                "target_id": "private-target-id"
            }),
        ];

        for outcome in outcomes {
            let mut runtime = FakeRuntime::new([
                json!(true),
                json!({"ok": true, "dirty_before": true}),
                outcome,
            ]);

            let error = pine_save(&mut runtime).await.unwrap_err();

            assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
            assert_eq!(
                error.message,
                "Pine save outcome was not explicitly verified"
            );
            let details = error.details.unwrap().to_string();
            assert!(!details.contains("private source"));
            assert!(!details.contains("private-script-id"));
            assert!(!details.contains("private-target-id"));
        }
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
