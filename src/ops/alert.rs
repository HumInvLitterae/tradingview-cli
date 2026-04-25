use serde_json::{Value, json};

use crate::{
    cdp::RuntimeEvaluator,
    error::{AppError, ErrorKind},
};

use super::common::{js_string, require_finite};

const ALERT_CONDITIONS: [&str; 3] = ["crossing", "greater_than", "less_than"];

pub async fn alert_list(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            r#"
            (async function() {
                try {
                    const response = await fetch('https://pricealerts.tradingview.com/list_alerts', {
                        credentials: 'include',
                        headers: {
                            'accept': 'application/json'
                        }
                    });

                    if (!response.ok) {
                        return {
                            alert_count: 0,
                            source: 'internal_api',
                            alerts: [],
                            error: 'HTTP ' + response.status + ': ' + response.statusText
                        };
                    }

                    const data = await response.json();
                    const rows = Array.isArray(data.r) ? data.r : [];
                    const alerts = rows.map(function(alert) {
                        return {
                            alert_id: alert.alert_id || alert.id || null,
                            symbol: alert.symbol || (alert.condition && alert.condition.symbol) || null,
                            type: alert.type || null,
                            message: alert.message || alert.description || '',
                            active: alert.active !== false,
                            condition: alert.condition || null,
                            resolution: alert.resolution || alert.interval || null,
                            created: alert.created || alert.create_time || null,
                            last_fired: alert.last_fired || alert.last_fire_time || null,
                            expiration: alert.expiration || alert.expire_time || null
                        };
                    });

                    return {
                        alert_count: alerts.length,
                        source: 'internal_api',
                        alerts: alerts
                    };
                } catch (error) {
                    return {
                        alert_count: 0,
                        source: 'internal_api',
                        alerts: [],
                        error: error && error.message ? error.message : String(error)
                    };
                }
            })()
            "#,
            true,
        )
        .await
        .map(normalize_alert_list_payload)
}

pub fn validate_alert_condition(condition: &str) -> Result<(), AppError> {
    let normalized = normalize_alert_condition(condition)?;
    if ALERT_CONDITIONS.contains(&normalized.as_str()) {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorKind::Validation,
            format!(
                "Unknown alert condition: {condition}. Use crossing, greater_than, or less_than."
            ),
        )
        .with_details(json!({
            "supported": ALERT_CONDITIONS,
        })))
    }
}

pub async fn alert_create(
    runtime: &mut impl RuntimeEvaluator,
    price: f64,
    condition: &str,
    message: Option<&str>,
) -> Result<Value, AppError> {
    require_finite(price, "price")?;
    validate_alert_condition(condition)?;

    let condition = normalize_alert_condition(condition)?;
    let price_text = price.to_string();
    let price_literal = js_string(&price_text)?;
    let condition_literal = js_string(&condition)?;
    let message_text = message
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("(none)");
    let message_literal = js_string(message_text)?;
    let should_set_message = message_text != "(none)";

    let result = runtime
        .evaluate(
            &format!(
                r#"
            (async function() {{
                function sleep(ms) {{
                    return new Promise(function(resolve) {{ setTimeout(resolve, ms); }});
                }}

                function setInputValue(input, value) {{
                    var setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set;
                    setter.call(input, value);
                    input.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    input.dispatchEvent(new Event('change', {{ bubbles: true }}));
                }}

                function setTextAreaValue(textarea, value) {{
                    var setter = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value').set;
                    setter.call(textarea, value);
                    textarea.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    textarea.dispatchEvent(new Event('change', {{ bubbles: true }}));
                }}

                function visibleRect(element) {{
                    var rect = element.getBoundingClientRect();
                    return rect.width > 0 && rect.height > 0 ? rect : null;
                }}

                function textOf(element) {{
                    return (element.textContent || element.innerText || '').trim();
                }}

                function findAlertDialog() {{
                    var dialogs = Array.from(document.querySelectorAll('[role="dialog"], [class*="dialog"], [class*="popup"]'));
                    for (var i = 0; i < dialogs.length; i++) {{
                        if (visibleRect(dialogs[i]) && /アラート|alert/i.test(textOf(dialogs[i]))) {{
                            return dialogs[i];
                        }}
                    }}
                    return document;
                }}

                var openButton = document.querySelector('[data-name="set-alert-button"]')
                    || document.querySelector('[aria-label="Create Alert"]')
                    || document.querySelector('[aria-label="アラート作成"]')
                    || document.querySelector('[data-name="alerts"]');
                var opened = false;
                var openSelector = null;
                if (openButton) {{
                    var ariaLabel = openButton.getAttribute('aria-label');
                    var dataName = openButton.getAttribute('data-name');
                    if (dataName === 'set-alert-button') {{
                        openSelector = '[data-name="set-alert-button"]';
                    }} else if (ariaLabel === 'Create Alert') {{
                        openSelector = '[aria-label="Create Alert"]';
                    }} else if (ariaLabel === 'アラート作成') {{
                        openSelector = '[aria-label="アラート作成"]';
                    }} else {{
                        openSelector = '[data-name="alerts"]';
                    }}
                    openButton.click();
                    opened = true;
                }}

                await sleep(1000);

                var scope = findAlertDialog();
                var inputs = Array.from(scope.querySelectorAll('input'));
                var priceInput = null;
                for (var i = 0; i < inputs.length; i++) {{
                    var value = inputs[i].value || '';
                    if (/^-?\d+([.,]\d+)?$/.test(value.trim())) {{
                        priceInput = inputs[i];
                        break;
                    }}
                }}
                if (!priceInput && inputs.length > 0) {{
                    priceInput = inputs[inputs.length - 1];
                }}

                var priceSet = false;
                if (priceInput) {{
                    setInputValue(priceInput, {price_literal});
                    priceSet = true;
                }}

                var messageSet = false;
                if ({should_set_message}) {{
                    scope = findAlertDialog();
                    var textarea = scope.querySelector('textarea');
                    if (!textarea) {{
                        var labels = Array.from(scope.querySelectorAll('*'));
                        var messageLabel = null;
                        for (var k = 0; k < labels.length; k++) {{
                            if (/^(message|メッセージ)$/i.test(textOf(labels[k]))) {{
                                messageLabel = labels[k];
                                break;
                            }}
                        }}
                        if (messageLabel) {{
                            var labelRect = visibleRect(messageLabel);
                            var candidates = Array.from(scope.querySelectorAll('button')).filter(function(button) {{
                                var rect = visibleRect(button);
                                if (!rect || !labelRect || rect.top <= labelRect.top) return false;
                                return !/^(create|作成|cancel|キャンセル|apply|適用)$/i.test(textOf(button));
                            }}).sort(function(left, right) {{
                                return left.getBoundingClientRect().top - right.getBoundingClientRect().top;
                            }});
                            if (candidates.length > 0) {{
                                candidates[0].click();
                                await sleep(300);
                            }}
                        }}
                    }}

                    scope = findAlertDialog();
                    textarea = scope.querySelector('textarea')
                        || document.querySelector('textarea[placeholder*="message"], textarea[placeholder*="メッセージ"]');
                    if (textarea) {{
                        setTextAreaValue(textarea, {message_literal});
                        messageSet = true;
                        await sleep(100);
                        var applyButton = Array.from(scope.querySelectorAll('button[data-name="submit"], button')).find(function(button) {{
                            return /^(apply|適用)$/i.test(textOf(button));
                        }});
                        if (applyButton) {{
                            applyButton.click();
                            await sleep(300);
                        }}
                    }}
                }}

                await sleep(500);

                var createButton = null;
                scope = findAlertDialog();
                var buttons = Array.from(scope.querySelectorAll('button[data-name="submit"], button'));
                for (var j = 0; j < buttons.length; j++) {{
                    if (/^(create|作成)$/i.test(textOf(buttons[j]))) {{
                        createButton = buttons[j];
                        break;
                    }}
                }}
                if (!createButton) {{
                    createButton = buttons.find(function(button) {{
                        return button.getAttribute('type') === 'submit' && !/^(apply|適用)$/i.test(textOf(button));
                    }});
                }}

                var created = false;
                if (createButton) {{
                    createButton.click();
                    created = true;
                }}

                return {{
                    opened: opened,
                    open_selector: openSelector,
                    price: {price},
                    condition: {condition_literal},
                    message: {message_literal},
                    price_set: priceSet,
                    message_set: messageSet,
                    created: created,
                    source: 'dom_fallback'
                }};
            }})()
            "#
            ),
            true,
        )
        .await?;

    normalize_alert_create_payload(result)
}

pub async fn alert_delete(
    runtime: &mut impl RuntimeEvaluator,
    alert_id: &str,
) -> Result<Value, AppError> {
    let alert_id = alert_id.trim();
    if alert_id.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Alert ID must not be empty",
        ));
    }

    let alert_id_literal = js_string(alert_id)?;
    let result = runtime
        .evaluate(
            &format!(
                r#"
            (async function() {{
                const requestedAlertId = {alert_id_literal};

                function normalizeAlert(alert) {{
                    return {{
                        alert_id: alert.alert_id || alert.id || null,
                        symbol: alert.symbol || (alert.condition && alert.condition.symbol) || null,
                        type: alert.type || null,
                        message: alert.message || alert.description || '',
                        active: alert.active !== false,
                        condition: alert.condition || null,
                        resolution: alert.resolution || alert.interval || null,
                        created: alert.created || alert.create_time || null,
                        last_fired: alert.last_fired || alert.last_fire_time || null,
                        expiration: alert.expiration || alert.expire_time || null
                    }};
                }}

                async function listAlerts() {{
                    const response = await fetch('https://pricealerts.tradingview.com/list_alerts', {{
                        credentials: 'include',
                        headers: {{ 'accept': 'application/json' }}
                    }});
                    if (!response.ok) {{
                        return {{
                            ok: false,
                            error: 'HTTP ' + response.status + ': ' + response.statusText,
                            alerts: []
                        }};
                    }}
                    const data = await response.json();
                    if (data.err) {{
                        return {{
                            ok: false,
                            error: data.errmsg || (data.err && data.err.code) || 'Alert list failed',
                            alerts: []
                        }};
                    }}
                    const rows = Array.isArray(data.r) ? data.r : [];
                    return {{ ok: true, alerts: rows.map(normalizeAlert) }};
                }}

                function findAlert(alerts) {{
                    return alerts.find(function(alert) {{
                        return String(alert.alert_id) === String(requestedAlertId);
                    }}) || null;
                }}

                try {{
                    const before = await listAlerts();
                    if (!before.ok) {{
                        return {{
                            error: before.error,
                            error_kind: 'internal_api_unavailable',
                            alert_id: requestedAlertId,
                            source: 'internal_api'
                        }};
                    }}

                    const matched = findAlert(before.alerts);
                    if (!matched) {{
                        return {{
                            error: 'Alert not found: ' + requestedAlertId,
                            error_kind: 'validation',
                            alert_id: requestedAlertId,
                            source: 'internal_api',
                            before_count: before.alerts.length,
                            matched_before: false
                        }};
                    }}

                    const alertIdValue = /^\\d+$/.test(String(requestedAlertId))
                        ? Number(requestedAlertId)
                        : requestedAlertId;
                    const deleteUrl = new URL('https://pricealerts.tradingview.com/delete_alerts', location.origin);
                    deleteUrl.searchParams.set('build_time', String(window.BUILD_TIME || ''));
                    deleteUrl.searchParams.set('log_username', String((window.user && window.user.username) || ''));
                    const deleteResponse = await fetch(deleteUrl.toString(), {{
                        method: 'POST',
                        credentials: 'include',
                        headers: {{ 'accept': 'application/json' }},
                        body: JSON.stringify({{ payload: {{ alert_ids: [alertIdValue] }} }})
                    }});
                    if (!deleteResponse.ok) {{
                        return {{
                            error: 'HTTP ' + deleteResponse.status + ': ' + deleteResponse.statusText,
                            error_kind: 'internal_api_unavailable',
                            alert_id: requestedAlertId,
                            source: 'internal_api',
                            before_count: before.alerts.length,
                            matched_before: true,
                            matched_alert: matched
                        }};
                    }}

                    const deleteData = await deleteResponse.json();
                    if (deleteData.err) {{
                        return {{
                            error: deleteData.errmsg || (deleteData.err && deleteData.err.code) || 'Alert delete failed',
                            error_kind: 'internal_api_unavailable',
                            alert_id: requestedAlertId,
                            source: 'internal_api',
                            before_count: before.alerts.length,
                            matched_before: true,
                            matched_alert: matched,
                            delete_response: deleteData
                        }};
                    }}

                    const after = await listAlerts();
                    if (!after.ok) {{
                        return {{
                            error: after.error,
                            error_kind: 'internal_api_unavailable',
                            alert_id: requestedAlertId,
                            source: 'internal_api',
                            before_count: before.alerts.length,
                            matched_before: true,
                            matched_alert: matched,
                            delete_response: deleteData
                        }};
                    }}

                    const matchedAfter = findAlert(after.alerts);
                    return {{
                        alert_id: requestedAlertId,
                        deleted: !matchedAfter,
                        source: 'internal_api',
                        before_count: before.alerts.length,
                        after_count: after.alerts.length,
                        matched_before: true,
                        matched_after: !!matchedAfter,
                        matched_alert: matched,
                        delete_response: deleteData
                    }};
                }} catch (error) {{
                    return {{
                        error: error && error.message ? error.message : String(error),
                        error_kind: 'internal_api_unavailable',
                        alert_id: requestedAlertId,
                        source: 'internal_api'
                    }};
                }}
            }})()
            "#
            ),
            true,
        )
        .await?;

    normalize_alert_delete_payload(result)
}

pub async fn alert_delete_all(
    runtime: &mut impl RuntimeEvaluator,
    dry_run: bool,
) -> Result<Value, AppError> {
    let result = runtime
        .evaluate(
            &format!(
                r#"
            (async function() {{
                const dryRun = {dry_run};

                function normalizeAlert(alert) {{
                    return {{
                        alert_id: alert.alert_id || alert.id || null,
                        symbol: alert.symbol || (alert.condition && alert.condition.symbol) || null,
                        type: alert.type || null,
                        message: alert.message || alert.description || '',
                        active: alert.active !== false,
                        condition: alert.condition || null,
                        resolution: alert.resolution || alert.interval || null,
                        created: alert.created || alert.create_time || null,
                        last_fired: alert.last_fired || alert.last_fire_time || null,
                        expiration: alert.expiration || alert.expire_time || null
                    }};
                }}

                async function listAlerts() {{
                    const response = await fetch('https://pricealerts.tradingview.com/list_alerts', {{
                        credentials: 'include',
                        headers: {{ 'accept': 'application/json' }}
                    }});
                    if (!response.ok) {{
                        return {{
                            ok: false,
                            error: 'HTTP ' + response.status + ': ' + response.statusText,
                            alerts: []
                        }};
                    }}
                    const data = await response.json();
                    if (data.err) {{
                        return {{
                            ok: false,
                            error: data.errmsg || (data.err && data.err.code) || 'Alert list failed',
                            alerts: []
                        }};
                    }}
                    const rows = Array.isArray(data.r) ? data.r : [];
                    return {{ ok: true, alerts: rows.map(normalizeAlert) }};
                }}

                function alertIds(alerts) {{
                    return alerts
                        .map(function(alert) {{ return alert.alert_id; }})
                        .filter(function(id) {{ return id !== null && id !== undefined && String(id).trim() !== ''; }});
                }}

                function wireAlertId(id) {{
                    return /^\\d+$/.test(String(id)) ? Number(id) : id;
                }}

                try {{
                    const before = await listAlerts();
                    if (!before.ok) {{
                        return {{
                            error: before.error,
                            error_kind: 'internal_api_unavailable',
                            source: 'internal_api'
                        }};
                    }}

                    const targetIds = alertIds(before.alerts);
                    if (targetIds.length !== before.alerts.length) {{
                        return {{
                            error: 'Alert list contained alerts without alert_id',
                            error_kind: 'internal_api_unavailable',
                            source: 'internal_api',
                            before_count: before.alerts.length,
                            target_alert_ids: targetIds,
                            target_alerts: before.alerts
                        }};
                    }}

                    if (dryRun) {{
                        return {{
                            action: 'dry_run',
                            dry_run: true,
                            deleted: false,
                            source: 'internal_api',
                            before_count: before.alerts.length,
                            after_count: before.alerts.length,
                            target_alert_ids: targetIds,
                            target_alerts: before.alerts
                        }};
                    }}

                    if (targetIds.length === 0) {{
                        return {{
                            action: 'noop',
                            dry_run: false,
                            deleted: false,
                            source: 'internal_api',
                            before_count: 0,
                            after_count: 0,
                            target_alert_ids: [],
                            target_alerts: []
                        }};
                    }}

                    const deleteUrl = new URL('https://pricealerts.tradingview.com/delete_alerts', location.origin);
                    deleteUrl.searchParams.set('build_time', String(window.BUILD_TIME || ''));
                    deleteUrl.searchParams.set('log_username', String((window.user && window.user.username) || ''));
                    const deleteResponse = await fetch(deleteUrl.toString(), {{
                        method: 'POST',
                        credentials: 'include',
                        headers: {{ 'accept': 'application/json' }},
                        body: JSON.stringify({{ payload: {{ alert_ids: targetIds.map(wireAlertId) }} }})
                    }});
                    if (!deleteResponse.ok) {{
                        return {{
                            error: 'HTTP ' + deleteResponse.status + ': ' + deleteResponse.statusText,
                            error_kind: 'internal_api_unavailable',
                            source: 'internal_api',
                            before_count: before.alerts.length,
                            target_alert_ids: targetIds,
                            target_alerts: before.alerts
                        }};
                    }}

                    const deleteData = await deleteResponse.json();
                    if (deleteData.err) {{
                        return {{
                            error: deleteData.errmsg || (deleteData.err && deleteData.err.code) || 'Alert delete failed',
                            error_kind: 'internal_api_unavailable',
                            source: 'internal_api',
                            before_count: before.alerts.length,
                            target_alert_ids: targetIds,
                            target_alerts: before.alerts,
                            delete_response: deleteData
                        }};
                    }}

                    const after = await listAlerts();
                    if (!after.ok) {{
                        return {{
                            error: after.error,
                            error_kind: 'internal_api_unavailable',
                            source: 'internal_api',
                            before_count: before.alerts.length,
                            target_alert_ids: targetIds,
                            target_alerts: before.alerts,
                            delete_response: deleteData
                        }};
                    }}

                    const remainingTargetIds = new Set(alertIds(after.alerts).map(String));
                    const stillPresent = targetIds.filter(function(id) {{ return remainingTargetIds.has(String(id)); }});
                    return {{
                        action: 'delete_all',
                        dry_run: false,
                        deleted: stillPresent.length === 0,
                        source: 'internal_api',
                        before_count: before.alerts.length,
                        after_count: after.alerts.length,
                        target_alert_ids: targetIds,
                        target_alerts: before.alerts,
                        remaining_target_alert_ids: stillPresent,
                        delete_response: deleteData
                    }};
                }} catch (error) {{
                    return {{
                        error: error && error.message ? error.message : String(error),
                        error_kind: 'internal_api_unavailable',
                        source: 'internal_api'
                    }};
                }}
            }})()
            "#
            ),
            true,
        )
        .await?;

    normalize_alert_delete_all_payload(result)
}

fn normalize_alert_list_payload(data: Value) -> Value {
    let alerts = data
        .get("alerts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut payload = json!({
        "alert_count": data
            .get("alert_count")
            .and_then(Value::as_u64)
            .unwrap_or(alerts.len() as u64),
        "source": data
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or("internal_api"),
        "alerts": alerts,
    });

    if let Some(error) = data.get("error").cloned() {
        payload["error"] = error;
    }

    payload
}

fn normalize_alert_create_payload(data: Value) -> Result<Value, AppError> {
    if !data
        .get("price_set")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Alert price input could not be set",
        )
        .with_details(data));
    }

    if !data
        .get("created")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Alert create button could not be clicked",
        )
        .with_details(data));
    }

    Ok(json!({
        "price": data.get("price").cloned().unwrap_or(Value::Null),
        "condition": data
            .get("condition")
            .and_then(Value::as_str)
            .unwrap_or("crossing"),
        "message": data
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("(none)"),
        "price_set": true,
        "source": data
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or("dom_fallback"),
        "created": true,
        "opened": data
            .get("opened")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        "open_selector": data.get("open_selector").cloned().unwrap_or(Value::Null),
        "message_set": data
            .get("message_set")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    }))
}

fn normalize_alert_delete_payload(data: Value) -> Result<Value, AppError> {
    if let Some(message) = data.get("error").and_then(Value::as_str) {
        let kind = match data.get("error_kind").and_then(Value::as_str) {
            Some("validation") => ErrorKind::Validation,
            _ => ErrorKind::InternalApiUnavailable,
        };
        return Err(AppError::new(kind, message.to_string()).with_details(data));
    }

    if !data
        .get("deleted")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Alert delete did not remove the requested alert",
        )
        .with_details(data));
    }

    Ok(json!({
        "alert_id": data.get("alert_id").cloned().unwrap_or(Value::Null),
        "deleted": true,
        "source": data
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or("internal_api"),
        "before_count": data.get("before_count").cloned().unwrap_or(Value::Null),
        "after_count": data.get("after_count").cloned().unwrap_or(Value::Null),
        "matched_before": data
            .get("matched_before")
            .and_then(Value::as_bool)
            .unwrap_or(true),
        "matched_after": data
            .get("matched_after")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        "matched_alert": data.get("matched_alert").cloned().unwrap_or(Value::Null),
    }))
}

fn normalize_alert_delete_all_payload(data: Value) -> Result<Value, AppError> {
    if let Some(message) = data.get("error").and_then(Value::as_str) {
        let kind = match data.get("error_kind").and_then(Value::as_str) {
            Some("validation") => ErrorKind::Validation,
            _ => ErrorKind::InternalApiUnavailable,
        };
        return Err(AppError::new(kind, message.to_string()).with_details(data));
    }

    if data
        .get("action")
        .and_then(Value::as_str)
        .is_some_and(|action| action == "delete_all")
        && !data
            .get("deleted")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Alert delete --all did not remove all target alerts",
        )
        .with_details(data));
    }

    Ok(json!({
        "action": data.get("action").cloned().unwrap_or_else(|| json!("delete_all")),
        "dry_run": data.get("dry_run").and_then(Value::as_bool).unwrap_or(false),
        "deleted": data.get("deleted").and_then(Value::as_bool).unwrap_or(false),
        "source": data.get("source").and_then(Value::as_str).unwrap_or("internal_api"),
        "before_count": data.get("before_count").cloned().unwrap_or(Value::Null),
        "after_count": data.get("after_count").cloned().unwrap_or(Value::Null),
        "target_alert_ids": data.get("target_alert_ids").cloned().unwrap_or_else(|| json!([])),
        "target_alerts": data.get("target_alerts").cloned().unwrap_or_else(|| json!([])),
        "remaining_target_alert_ids": data
            .get("remaining_target_alert_ids")
            .cloned()
            .unwrap_or_else(|| json!([])),
    }))
}

fn normalize_alert_condition(condition: &str) -> Result<String, AppError> {
    let normalized = condition.trim().to_ascii_lowercase().replace('-', "_");
    if normalized.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Alert condition must not be empty",
        ));
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use serde_json::json;

    use super::*;
    use crate::ops::test_support::FakeRuntime;

    #[tokio::test]
    async fn alert_list_returns_runtime_payload() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "alert_count": 1,
            "source": "internal_api",
            "alerts": [
                {
                    "alert_id": "alert-1",
                    "symbol": "NASDAQ:AAPL",
                    "type": "price",
                    "message": "Breakout",
                    "active": true,
                    "condition": { "operator": "greater" },
                    "resolution": "1D",
                    "created": 1777000000,
                    "last_fired": null,
                    "expiration": 1777600000
                }
            ]
        })]));

        let data = alert_list(&mut runtime).await.unwrap();

        assert_eq!(data["alert_count"], 1);
        assert_eq!(data["source"], "internal_api");
        assert_eq!(data["alerts"][0]["alert_id"], "alert-1");
        assert_eq!(data["alerts"][0]["symbol"], "NASDAQ:AAPL");
        assert!(runtime.evaluated[0].0.contains("list_alerts"));
        assert!(!runtime.evaluated[0].0.contains("content-type"));
        assert!(runtime.evaluated[0].1);
    }

    #[tokio::test]
    async fn alert_list_preserves_api_error_payload() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "alert_count": 0,
            "source": "internal_api",
            "alerts": [],
            "error": "HTTP 403: Forbidden"
        })]));

        let data = alert_list(&mut runtime).await.unwrap();

        assert_eq!(data["alert_count"], 0);
        assert_eq!(data["alerts"].as_array().unwrap().len(), 0);
        assert_eq!(data["error"], "HTTP 403: Forbidden");
    }

    #[tokio::test]
    async fn alert_list_defaults_malformed_payload_to_empty_list() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "source": "internal_api"
        })]));

        let data = alert_list(&mut runtime).await.unwrap();

        assert_eq!(data["alert_count"], 0);
        assert_eq!(data["source"], "internal_api");
        assert_eq!(data["alerts"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn alert_create_returns_practical_old_cli_fields() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "opened": true,
            "open_selector": "[aria-label=\"Create Alert\"]",
            "price": 123.45,
            "condition": "crossing",
            "message": "Breakout",
            "price_set": true,
            "message_set": true,
            "created": true,
            "source": "dom_fallback"
        })]));

        let data = alert_create(&mut runtime, 123.45, "crossing", Some("Breakout"))
            .await
            .unwrap();

        assert_eq!(data["price"], 123.45);
        assert_eq!(data["condition"], "crossing");
        assert_eq!(data["message"], "Breakout");
        assert_eq!(data["price_set"], true);
        assert_eq!(data["source"], "dom_fallback");
        assert!(runtime.evaluated[0].0.contains("Create Alert"));
        assert!(runtime.evaluated[0].0.contains("set-alert-button"));
        assert!(runtime.evaluated[0].0.contains("\"Breakout\""));
        assert!(runtime.evaluated[0].1);
    }

    #[tokio::test]
    async fn alert_create_defaults_message_to_none() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "price": 100.0,
            "condition": "greater_than",
            "message": "(none)",
            "price_set": true,
            "created": true,
            "source": "dom_fallback"
        })]));

        let data = alert_create(&mut runtime, 100.0, "greater-than", None)
            .await
            .unwrap();

        assert_eq!(data["condition"], "greater_than");
        assert_eq!(data["message"], "(none)");
        assert!(!runtime.evaluated[0].0.contains("greater-than"));
    }

    #[tokio::test]
    async fn alert_create_rejects_invalid_condition() {
        let mut runtime = FakeRuntime::new(VecDeque::new());

        let error = alert_create(&mut runtime, 100.0, "above", None)
            .await
            .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(runtime.evaluated.is_empty());
    }

    #[tokio::test]
    async fn alert_create_rejects_non_finite_price() {
        let mut runtime = FakeRuntime::new(VecDeque::new());

        let error = alert_create(&mut runtime, f64::NAN, "crossing", None)
            .await
            .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(runtime.evaluated.is_empty());
    }

    #[tokio::test]
    async fn alert_create_fails_when_price_was_not_set() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "price": 100.0,
            "condition": "crossing",
            "message": "(none)",
            "price_set": false,
            "created": true,
            "source": "dom_fallback"
        })]));

        let error = alert_create(&mut runtime, 100.0, "crossing", None)
            .await
            .unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.message, "Alert price input could not be set");
    }

    #[tokio::test]
    async fn alert_create_fails_when_create_button_was_not_clicked() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "price": 100.0,
            "condition": "crossing",
            "message": "(none)",
            "price_set": true,
            "created": false,
            "source": "dom_fallback"
        })]));

        let error = alert_create(&mut runtime, 100.0, "crossing", None)
            .await
            .unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.message, "Alert create button could not be clicked");
    }

    #[tokio::test]
    async fn alert_delete_returns_practical_fields() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "alert_id": "4546454367",
            "deleted": true,
            "source": "internal_api",
            "before_count": 1,
            "after_count": 0,
            "matched_before": true,
            "matched_after": false,
            "matched_alert": {
                "alert_id": "4546454367",
                "message": "smoke"
            },
            "delete_response": { "s": "ok" }
        })]));

        let data = alert_delete(&mut runtime, "4546454367").await.unwrap();

        assert_eq!(data["alert_id"], "4546454367");
        assert_eq!(data["deleted"], true);
        assert_eq!(data["source"], "internal_api");
        assert_eq!(data["before_count"], 1);
        assert_eq!(data["after_count"], 0);
        assert_eq!(data["matched_alert"]["message"], "smoke");
        assert!(runtime.evaluated[0].0.contains("delete_alerts"));
        assert!(runtime.evaluated[0].0.contains("alert_ids"));
        assert!(runtime.evaluated[0].0.contains("log_username"));
        assert!(runtime.evaluated[0].0.contains("\"4546454367\""));
        assert!(runtime.evaluated[0].1);
    }

    #[tokio::test]
    async fn alert_delete_rejects_empty_id_before_evaluating() {
        let mut runtime = FakeRuntime::new(VecDeque::new());

        let error = alert_delete(&mut runtime, " ").await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(runtime.evaluated.is_empty());
    }

    #[tokio::test]
    async fn alert_delete_maps_missing_alert_to_validation() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "error": "Alert not found: missing",
            "error_kind": "validation",
            "alert_id": "missing",
            "source": "internal_api",
            "before_count": 3,
            "matched_before": false
        })]));

        let error = alert_delete(&mut runtime, "missing").await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert_eq!(error.details.unwrap()["matched_before"], false);
    }

    #[tokio::test]
    async fn alert_delete_maps_failed_delete_to_internal_api_unavailable() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "error": "Alert delete failed",
            "error_kind": "internal_api_unavailable",
            "alert_id": "4546454367",
            "source": "internal_api"
        })]));

        let error = alert_delete(&mut runtime, "4546454367").await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
    }

    #[tokio::test]
    async fn alert_delete_all_returns_dry_run_targets() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "action": "dry_run",
            "dry_run": true,
            "deleted": false,
            "source": "internal_api",
            "before_count": 2,
            "after_count": 2,
            "target_alert_ids": ["1", "2"],
            "target_alerts": [
                { "alert_id": "1", "message": "one" },
                { "alert_id": "2", "message": "two" }
            ]
        })]));

        let data = alert_delete_all(&mut runtime, true).await.unwrap();

        assert_eq!(data["action"], "dry_run");
        assert_eq!(data["dry_run"], true);
        assert_eq!(data["deleted"], false);
        assert_eq!(data["before_count"], 2);
        assert_eq!(data["target_alert_ids"][0], "1");
        assert!(runtime.evaluated[0].0.contains("list_alerts"));
        assert!(runtime.evaluated[0].0.contains("delete_alerts"));
        assert!(runtime.evaluated[0].1);
    }

    #[tokio::test]
    async fn alert_delete_all_returns_noop_when_empty() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "action": "noop",
            "dry_run": false,
            "deleted": false,
            "source": "internal_api",
            "before_count": 0,
            "after_count": 0,
            "target_alert_ids": [],
            "target_alerts": []
        })]));

        let data = alert_delete_all(&mut runtime, false).await.unwrap();

        assert_eq!(data["action"], "noop");
        assert_eq!(data["deleted"], false);
        assert_eq!(data["before_count"], 0);
        assert_eq!(data["after_count"], 0);
    }

    #[tokio::test]
    async fn alert_delete_all_returns_success_payload() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "action": "delete_all",
            "dry_run": false,
            "deleted": true,
            "source": "internal_api",
            "before_count": 2,
            "after_count": 0,
            "target_alert_ids": ["1", "2"],
            "target_alerts": [
                { "alert_id": "1", "message": "one" },
                { "alert_id": "2", "message": "two" }
            ],
            "remaining_target_alert_ids": [],
            "delete_response": { "s": "ok" }
        })]));

        let data = alert_delete_all(&mut runtime, false).await.unwrap();

        assert_eq!(data["action"], "delete_all");
        assert_eq!(data["deleted"], true);
        assert_eq!(data["after_count"], 0);
        assert_eq!(
            data["remaining_target_alert_ids"].as_array().unwrap().len(),
            0
        );
    }

    #[tokio::test]
    async fn alert_delete_all_requires_target_absence_after_delete() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "action": "delete_all",
            "dry_run": false,
            "deleted": false,
            "source": "internal_api",
            "before_count": 2,
            "after_count": 1,
            "target_alert_ids": ["1", "2"],
            "target_alerts": [
                { "alert_id": "1", "message": "one" },
                { "alert_id": "2", "message": "two" }
            ],
            "remaining_target_alert_ids": ["2"]
        })]));

        let error = alert_delete_all(&mut runtime, false).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(
            error.message,
            "Alert delete --all did not remove all target alerts"
        );
    }
}
