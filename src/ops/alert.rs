use serde_json::{Value, json};

use crate::{
    cdp::RuntimeEvaluator,
    error::{AppError, ErrorKind},
};

use super::{
    common::{js_string, require_finite},
    pine::{PineAlertconditionCandidate, pine_alertcondition_candidates},
};

const ALERT_CONDITIONS: [&str; 3] = ["crossing", "greater_than", "less_than"];

#[derive(Debug, Clone)]
pub struct IndicatorAlertDryRunRequest<'a> {
    pub script: &'a str,
    pub source: &'a str,
    pub input_source: &'static str,
    pub condition_title: Option<&'a str>,
    pub alert_cond_id: Option<&'a str>,
    pub symbol: Option<&'a str>,
    pub resolution: Option<&'a str>,
    pub message: Option<&'a str>,
}

#[derive(Debug, Clone)]
struct SavedPineScriptPreview {
    name: String,
    title: Option<String>,
    version: Option<Value>,
    modified: Option<Value>,
    script_id_available: bool,
}

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

pub async fn alert_create_indicator_dry_run(
    runtime: &mut impl RuntimeEvaluator,
    request: IndicatorAlertDryRunRequest<'_>,
) -> Result<Value, AppError> {
    let script = require_non_empty(request.script, "script")?;
    let candidate = select_alertcondition_candidate(
        request.source,
        request.condition_title,
        request.alert_cond_id,
    )?;
    let saved_script = resolve_saved_pine_script(runtime, script).await?;

    Ok(json!({
        "action": "dry_run",
        "dry_run": true,
        "would_create": true,
        "mutation_supported": false,
        "source": "indicator_alert_dry_run",
        "input_source": request.input_source,
        "script": {
            "requested": script,
            "name": saved_script.name,
            "title": saved_script.title,
            "version": saved_script.version,
            "modified": saved_script.modified,
            "script_id_available": saved_script.script_id_available,
        },
        "condition": {
            "selector": if request.alert_cond_id.is_some() { "alert_cond_id" } else { "condition_title" },
            "alert_cond_id": candidate.alert_cond_id,
            "plot_index": candidate.plot_index,
            "preceding_output_count": candidate.preceding_output_count,
            "title": candidate.title,
            "message": candidate.message,
            "line": candidate.line,
            "column": candidate.column,
            "confidence": candidate.confidence,
        },
        "request": {
            "symbol": request.symbol.map(str::trim).filter(|value| !value.is_empty()),
            "resolution": request.resolution.map(str::trim).filter(|value| !value.is_empty()),
            "message": request.message.map(str::trim).filter(|value| !value.is_empty()),
        },
        "note": "Dry run only. No TradingView alert was created; normal indicator alert mutation remains deferred.",
    }))
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

fn require_non_empty<'a>(value: &'a str, label: &str) -> Result<&'a str, AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(AppError::new(
            ErrorKind::Validation,
            format!("{label} must not be empty"),
        ))
    } else {
        Ok(trimmed)
    }
}

fn select_alertcondition_candidate(
    source: &str,
    condition_title: Option<&str>,
    alert_cond_id: Option<&str>,
) -> Result<PineAlertconditionCandidate, AppError> {
    if condition_title.is_some() == alert_cond_id.is_some() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Use exactly one of --condition-title <TEXT> or --alert-cond-id <ID>",
        ));
    }

    let candidates = pine_alertcondition_candidates(source);
    if candidates.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "No alertcondition() candidates found in Pine source",
        ));
    }

    let matches = if let Some(id) = alert_cond_id {
        let id = require_non_empty(id, "alert_cond_id")?;
        validate_alert_cond_id(id)?;
        candidates
            .into_iter()
            .filter(|candidate| candidate.alert_cond_id == id)
            .collect::<Vec<_>>()
    } else {
        let title = require_non_empty(condition_title.unwrap_or_default(), "condition_title")?;
        candidates
            .into_iter()
            .filter(|candidate| {
                candidate
                    .title
                    .as_deref()
                    .is_some_and(|candidate_title| candidate_title == title)
            })
            .collect::<Vec<_>>()
    };

    match matches.as_slice() {
        [candidate] => Ok(candidate.clone()),
        [] => Err(AppError::new(
            ErrorKind::Validation,
            "No matching alertcondition() candidate found",
        )
        .with_details(json!({
            "available_candidates": pine_alertcondition_candidates(source)
                .into_iter()
                .map(public_alertcondition_candidate)
                .collect::<Vec<_>>(),
        }))),
        _ => Err(AppError::new(
            ErrorKind::Validation,
            "Multiple alertcondition() candidates match the selector",
        )
        .with_details(json!({
            "matching_candidates": matches
                .into_iter()
                .map(public_alertcondition_candidate)
                .collect::<Vec<_>>(),
        }))),
    }
}

fn validate_alert_cond_id(value: &str) -> Result<(), AppError> {
    let Some(index) = value.strip_prefix("plot_") else {
        return Err(AppError::new(
            ErrorKind::Validation,
            "alert_cond_id must use the plot_<N> format",
        ));
    };
    if index.is_empty() || index.parse::<usize>().is_err() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "alert_cond_id must use the plot_<N> format",
        ));
    }
    Ok(())
}

fn public_alertcondition_candidate(candidate: PineAlertconditionCandidate) -> Value {
    json!({
        "alert_cond_id": candidate.alert_cond_id,
        "plot_index": candidate.plot_index,
        "title": candidate.title,
        "message": candidate.message,
        "line": candidate.line,
        "column": candidate.column,
        "confidence": candidate.confidence,
    })
}

async fn resolve_saved_pine_script(
    runtime: &mut impl RuntimeEvaluator,
    script: &str,
) -> Result<SavedPineScriptPreview, AppError> {
    let script_literal = js_string(script)?;
    let result = runtime
        .evaluate(
            &format!(
                r#"
            fetch('https://pine-facade.tradingview.com/pine-facade/list/?filter=saved', {{ credentials: 'include' }})
                .then(function(response) {{
                    return response.json().then(function(data) {{
                        return {{ ok: response.ok, status: response.status, statusText: response.statusText, data: data }};
                    }});
                }})
                .then(function(result) {{
                    const requested = {script_literal};
                    if (!result.ok) {{
                        return {{ error: 'HTTP ' + result.status + ': ' + result.statusText, kind: 'internal_api_unavailable' }};
                    }}
                    if (!Array.isArray(result.data)) {{
                        return {{ error: 'Unexpected response from pine-facade', kind: 'internal_api_unavailable' }};
                    }}
                    const scripts = result.data.map(function(s) {{
                        return {{
                            name: s.scriptName || s.scriptTitle || 'Untitled',
                            title: s.scriptTitle || null,
                            version: s.version || null,
                            modified: s.modified || null,
                            script_id_available: !!s.scriptIdPart
                        }};
                    }});
                    const matches = scripts.filter(function(script) {{
                        return script.name === requested || script.title === requested;
                    }});
                    return {{
                        requested: requested,
                        match_count: matches.length,
                        match: matches.length === 1 ? matches[0] : null,
                        candidates: matches.length === 1 ? [] : scripts.slice(0, 20)
                    }};
                }})
                .catch(function(error) {{
                    return {{ error: error && error.message ? error.message : String(error), kind: 'internal_api_unavailable' }};
                }})
            "#
            ),
            true,
        )
        .await?;

    normalize_saved_pine_script_match(result)
}

fn normalize_saved_pine_script_match(data: Value) -> Result<SavedPineScriptPreview, AppError> {
    if let Some(error) = data.get("error").and_then(Value::as_str) {
        return Err(
            AppError::new(ErrorKind::InternalApiUnavailable, error.to_string()).with_details(data),
        );
    }

    let match_count = data.get("match_count").and_then(Value::as_u64).unwrap_or(0);
    if match_count != 1 {
        let message = if match_count == 0 {
            "No saved Pine script matches --script"
        } else {
            "Multiple saved Pine scripts match --script"
        };
        return Err(
            AppError::new(ErrorKind::Validation, message).with_details(json!({
                "requested": data.get("requested").cloned().unwrap_or(Value::Null),
                "match_count": match_count,
                "candidates": data.get("candidates").cloned().unwrap_or_else(|| json!([])),
            })),
        );
    }

    let matched = data
        .get("match")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            AppError::new(
                ErrorKind::InternalApiUnavailable,
                "Pine script match payload was malformed",
            )
            .with_details(data.clone())
        })?;

    let name = matched
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("Untitled")
        .to_string();
    let title = matched
        .get("title")
        .and_then(Value::as_str)
        .map(str::to_string);
    let version = matched
        .get("version")
        .cloned()
        .filter(|value| !value.is_null());
    let modified = matched
        .get("modified")
        .cloned()
        .filter(|value| !value.is_null());
    let script_id_available = matched
        .get("script_id_available")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    Ok(SavedPineScriptPreview {
        name,
        title,
        version,
        modified,
        script_id_available,
    })
}

pub async fn alert_create_via_api(
    runtime: &mut impl RuntimeEvaluator,
    price: f64,
    condition: &str,
    message: Option<&str>,
) -> Result<Value, AppError> {
    require_finite(price, "price")?;
    validate_alert_condition(condition)?;

    let condition = normalize_alert_condition(condition)?;
    let condition_type = alert_condition_type(&condition);
    let message_text = message
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("(none)");
    let condition_literal = js_string(&condition)?;
    let condition_type_literal = js_string(condition_type)?;
    let message_literal = js_string(message_text)?;

    let result = runtime
        .evaluate(
            &format!(
                r#"
            (async function() {{
                const requestedPrice = {price};
                const requestedCondition = {condition_literal};
                const requestedConditionType = {condition_type_literal};
                const requestedMessage = {message_literal};
                const source = 'internal_api';

                function publicAlert(alert) {{
                    if (!alert) return null;
                    return {{
                        alert_id: alert.alert_id || alert.id || null,
                        symbol: alert.symbol || (alert.condition && alert.condition.symbol) || null,
                        type: alert.type || null,
                        message: alert.message || alert.description || '',
                        active: alert.active !== false,
                        condition: alert.condition || null,
                        resolution: alert.resolution || alert.interval || null,
                        created: alert.created || alert.create_time || null,
                        expiration: alert.expiration || alert.expire_time || null
                    }};
                }}

                function normalizeRows(data) {{
                    const rows = Array.isArray(data && data.r) ? data.r : [];
                    return rows.map(publicAlert);
                }}

                async function listAlerts() {{
                    let response;
                    let data;
                    try {{
                        response = await fetch('https://pricealerts.tradingview.com/list_alerts', {{
                            credentials: 'include',
                            headers: {{ 'accept': 'application/json' }}
                        }});
                        data = await response.json();
                    }} catch (error) {{
                        return {{
                            ok: false,
                            error: error && error.message ? error.message : String(error)
                        }};
                    }}
                    if (!response.ok) {{
                        return {{
                            ok: false,
                            error: 'HTTP ' + response.status + ': ' + response.statusText
                        }};
                    }}
                    if (data && data.err) {{
                        return {{
                            ok: false,
                            error: data.errmsg || (data.err && data.err.code) || 'Alert list failed'
                        }};
                    }}
                    return {{ ok: true, alerts: normalizeRows(data) }};
                }}

                function readChartMetadata() {{
                    try {{
                        const chart = window.TradingViewApi &&
                            window.TradingViewApi._activeChartWidgetWV &&
                            window.TradingViewApi._activeChartWidgetWV.value &&
                            window.TradingViewApi._activeChartWidgetWV.value();
                        const model = chart && chart._chartWidget && chart._chartWidget.model &&
                            chart._chartWidget.model();
                        const mainSeries = model && model.mainSeries && model.mainSeries();
                        const ext = chart && chart.symbolExt && chart.symbolExt();
                        const info = mainSeries && mainSeries.symbolInfo && mainSeries.symbolInfo();
                        const symbol = (mainSeries && mainSeries.symbol && mainSeries.symbol()) ||
                            (ext && (ext.pro_name || ext.full_name || ext.symbol)) ||
                            (info && (info.pro_name || info.full_name || info.symbol)) ||
                            null;
                        const resolution = String(
                            (chart && chart.resolution && chart.resolution()) ||
                            (mainSeries && mainSeries.interval && mainSeries.interval()) ||
                            '1'
                        );
                        const currency = (ext && (ext.currency_id || ext.currency || ext['currency-id'])) ||
                            (info && (info.currency_id || info.currency_code || info.currency || info['currency-id'])) ||
                            null;
                        if (!symbol) {{
                            return {{ error: 'Active chart symbol unavailable' }};
                        }}
                        return {{
                            symbol,
                            resolution,
                            currency: currency || 'USD'
                        }};
                    }} catch (error) {{
                        return {{
                            error: error && error.message ? error.message : String(error)
                        }};
                    }}
                }}

                function alertIds(alerts) {{
                    const ids = {{}};
                    alerts.forEach(function(alert) {{
                        const id = alert && alert.alert_id;
                        if (id !== null && id !== undefined) ids[String(id)] = true;
                    }});
                    return ids;
                }}

                function conditionValue(alert) {{
                    const series = alert && alert.condition && Array.isArray(alert.condition.series)
                        ? alert.condition.series
                        : [];
                    for (let i = 0; i < series.length; i++) {{
                        if (series[i] && series[i].type === 'value' && typeof series[i].value === 'number') {{
                            return series[i].value;
                        }}
                    }}
                    return null;
                }}

                function matchingNewAlert(alerts, beforeIds, symbolMarker) {{
                    const tolerance = Math.max(0.000001, Math.abs(requestedPrice) * 0.000001);
                    return alerts.find(function(alert) {{
                        const id = alert && alert.alert_id;
                        if (id !== null && id !== undefined && beforeIds[String(id)]) return false;
                        if (!alert || alert.message !== requestedMessage) return false;
                        if (alert.symbol !== symbolMarker) return false;
                        if (!alert.condition || alert.condition.type !== requestedConditionType) return false;
                        const value = conditionValue(alert);
                        return typeof value === 'number' && Math.abs(value - requestedPrice) <= tolerance;
                    }}) || null;
                }}

                const chartMeta = readChartMetadata();
                if (chartMeta.error) {{
                    return {{
                        error: chartMeta.error,
                        error_kind: 'internal_api_unavailable',
                        phase: 'chart_metadata_unavailable',
                        api_fallback_allowed: true,
                        price: requestedPrice,
                        condition: requestedCondition,
                        message: requestedMessage,
                        price_set: false,
                        created: false,
                        source
                    }};
                }}

                const before = await listAlerts();
                if (!before.ok) {{
                    return {{
                        error: before.error,
                        error_kind: 'internal_api_unavailable',
                        phase: 'pre_list_unavailable',
                        api_fallback_allowed: true,
                        price: requestedPrice,
                        condition: requestedCondition,
                        message: requestedMessage,
                        price_set: false,
                        created: false,
                        source
                    }};
                }}

                const symbolMarker = '=' + JSON.stringify({{
                    symbol: chartMeta.symbol,
                    adjustment: 'splits',
                    'currency-id': chartMeta.currency
                }});
                const expiration = new Date(Date.now() + 30 * 24 * 60 * 60 * 1000).toISOString();
                const payload = {{
                    symbol: symbolMarker,
                    resolution: chartMeta.resolution,
                    message: requestedMessage,
                    sound_file: null,
                    sound_duration: 0,
                    popup: true,
                    expiration,
                    auto_deactivate: true,
                    email: false,
                    sms_over_email: false,
                    mobile_push: true,
                    web_hook: null,
                    name: null,
                    conditions: [{{
                        type: requestedConditionType,
                        frequency: 'on_first_fire',
                        series: [{{ type: 'barset' }}, {{ type: 'value', value: requestedPrice }}],
                        resolution: chartMeta.resolution
                    }}],
                    active: true,
                    ignore_warnings: true
                }};

                let createResponse;
                let createText;
                let createData = null;
                try {{
                    createResponse = await fetch('https://pricealerts.tradingview.com/create_alert', {{
                        method: 'POST',
                        credentials: 'include',
                        body: JSON.stringify({{ payload }})
                    }});
                    createText = await createResponse.text();
                    try {{
                        createData = createText ? JSON.parse(createText) : null;
                    }} catch (_) {{}}
                }} catch (error) {{
                    return {{
                        error: error && error.message ? error.message : String(error),
                        error_kind: 'internal_api_unavailable',
                        phase: 'create_request_unavailable',
                        api_fallback_allowed: true,
                        price: requestedPrice,
                        condition: requestedCondition,
                        message: requestedMessage,
                        price_set: true,
                        created: false,
                        source,
                        before_count: before.alerts.length
                    }};
                }}

                if (!createResponse.ok || (createData && createData.err)) {{
                    return {{
                        error: createData && createData.errmsg
                            ? createData.errmsg
                            : 'HTTP ' + createResponse.status + ': ' + createResponse.statusText,
                        error_kind: 'internal_api_unavailable',
                        phase: 'create_request_failed',
                        api_fallback_allowed: false,
                        price: requestedPrice,
                        condition: requestedCondition,
                        message: requestedMessage,
                        price_set: true,
                        created: false,
                        source,
                        before_count: before.alerts.length,
                        status: createResponse.status,
                        body_excerpt: String(createText || '').slice(0, 160)
                    }};
                }}

                const after = await listAlerts();
                if (!after.ok) {{
                    return {{
                        error: after.error,
                        error_kind: 'internal_api_unavailable',
                        phase: 'post_list_unavailable',
                        api_fallback_allowed: false,
                        price: requestedPrice,
                        condition: requestedCondition,
                        condition_type: requestedConditionType,
                        message: requestedMessage,
                        price_set: true,
                        created: false,
                        source,
                        symbol: chartMeta.symbol,
                        resolution: chartMeta.resolution,
                        before_count: before.alerts.length
                    }};
                }}

                const matched = matchingNewAlert(after.alerts, alertIds(before.alerts), symbolMarker);
                if (!matched) {{
                    return {{
                        error: 'Alert create did not confirm a matching new alert',
                        error_kind: 'internal_api_unavailable',
                        phase: 'post_check_failed',
                        api_fallback_allowed: false,
                        price: requestedPrice,
                        condition: requestedCondition,
                        condition_type: requestedConditionType,
                        message: requestedMessage,
                        price_set: true,
                        created: false,
                        source,
                        symbol: chartMeta.symbol,
                        resolution: chartMeta.resolution,
                        before_count: before.alerts.length,
                        after_count: after.alerts.length
                    }};
                }}

                return {{
                    alert_id: matched.alert_id || null,
                    price: requestedPrice,
                    condition: requestedCondition,
                    condition_type: requestedConditionType,
                    message: requestedMessage,
                    price_set: true,
                    message_set: requestedMessage !== '(none)',
                    created: true,
                    opened: false,
                    open_selector: null,
                    source,
                    symbol: chartMeta.symbol,
                    resolution: chartMeta.resolution,
                    before_count: before.alerts.length,
                    after_count: after.alerts.length,
                    matched_alert: matched
                }};
            }})()
            "#
            ),
            true,
        )
        .await?;

    normalize_alert_create_payload(result)
}

pub async fn alert_create(
    runtime: &mut impl RuntimeEvaluator,
    price: f64,
    condition: &str,
    message: Option<&str>,
) -> Result<Value, AppError> {
    require_finite(price, "price")?;
    validate_alert_condition(condition)?;

    match alert_create_via_api(runtime, price, condition, message).await {
        Ok(data) => return Ok(data),
        Err(error) if alert_api_error_allows_fallback(&error) => {}
        Err(error) => return Err(error),
    }

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
                    const deleteResponse = await fetch('https://pricealerts.tradingview.com/delete_alerts', {{
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

                    const deleteResponse = await fetch('https://pricealerts.tradingview.com/delete_alerts', {{
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
    if let Some(message) = data.get("error").and_then(Value::as_str) {
        let kind = match data.get("error_kind").and_then(Value::as_str) {
            Some("validation") => ErrorKind::Validation,
            _ => ErrorKind::InternalApiUnavailable,
        };
        return Err(AppError::new(kind, message.to_string()).with_details(data));
    }

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
        "alert_id": data.get("alert_id").cloned().unwrap_or(Value::Null),
        "symbol": data.get("symbol").cloned().unwrap_or(Value::Null),
        "resolution": data.get("resolution").cloned().unwrap_or(Value::Null),
        "condition_type": data.get("condition_type").cloned().unwrap_or(Value::Null),
        "before_count": data.get("before_count").cloned().unwrap_or(Value::Null),
        "after_count": data.get("after_count").cloned().unwrap_or(Value::Null),
        "matched_alert": data.get("matched_alert").cloned().unwrap_or(Value::Null),
    }))
}

fn alert_api_error_allows_fallback(error: &AppError) -> bool {
    error
        .details
        .as_ref()
        .and_then(|details| details.get("api_fallback_allowed"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
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

fn alert_condition_type(condition: &str) -> &'static str {
    match condition {
        "greater_than" => "cross_up",
        "less_than" => "cross_down",
        _ => "cross",
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use serde_json::json;

    use super::*;
    use crate::ops::test_support::FakeRuntime;

    fn alert_create_api_fallback() -> Value {
        json!({
            "error": "Alert create API unavailable in test",
            "error_kind": "internal_api_unavailable",
            "phase": "pre_list_unavailable",
            "api_fallback_allowed": true,
            "price": 100.0,
            "condition": "crossing",
            "message": "(none)",
            "price_set": false,
            "created": false,
            "source": "internal_api"
        })
    }

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
            "alert_id": "4530000001",
            "opened": false,
            "open_selector": null,
            "price": 123.45,
            "condition": "crossing",
            "condition_type": "cross",
            "message": "Breakout",
            "price_set": true,
            "message_set": true,
            "created": true,
            "source": "internal_api",
            "symbol": "NASDAQ:AAPL",
            "resolution": "1",
            "before_count": 2,
            "after_count": 3,
            "matched_alert": {"alert_id": "4530000001", "message": "Breakout"}
        })]));

        let data = alert_create(&mut runtime, 123.45, "crossing", Some("Breakout"))
            .await
            .unwrap();

        assert_eq!(data["alert_id"], "4530000001");
        assert_eq!(data["price"], 123.45);
        assert_eq!(data["condition"], "crossing");
        assert_eq!(data["condition_type"], "cross");
        assert_eq!(data["message"], "Breakout");
        assert_eq!(data["price_set"], true);
        assert_eq!(data["source"], "internal_api");
        assert_eq!(data["created"], true);
        assert_eq!(data["symbol"], "NASDAQ:AAPL");
        assert_eq!(data["before_count"], 2);
        assert_eq!(data["after_count"], 3);
        assert!(runtime.evaluated[0].0.contains("create_alert"));
        assert!(runtime.evaluated[0].0.contains("list_alerts"));
        assert!(!runtime.evaluated[0].0.contains("Content-Type"));
        assert!(runtime.evaluated[0].1);
    }

    #[tokio::test]
    async fn alert_create_falls_back_to_dom_when_api_is_unavailable_before_mutation() {
        let mut runtime = FakeRuntime::new(VecDeque::from([
            alert_create_api_fallback(),
            json!({
                "opened": true,
                "open_selector": "[aria-label=\"Create Alert\"]",
                "price": 123.45,
                "condition": "crossing",
                "message": "Breakout",
                "price_set": true,
                "message_set": true,
                "created": true,
                "source": "dom_fallback"
            }),
        ]));

        let data = alert_create(&mut runtime, 123.45, "crossing", Some("Breakout"))
            .await
            .unwrap();

        assert_eq!(data["price"], 123.45);
        assert_eq!(data["condition"], "crossing");
        assert_eq!(data["message"], "Breakout");
        assert_eq!(data["price_set"], true);
        assert_eq!(data["source"], "dom_fallback");
        assert!(runtime.evaluated[1].0.contains("Create Alert"));
        assert!(runtime.evaluated[1].0.contains("set-alert-button"));
        assert!(runtime.evaluated[1].0.contains("\"Breakout\""));
        assert!(runtime.evaluated[1].1);
    }

    #[tokio::test]
    async fn alert_create_defaults_message_to_none() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "alert_id": "4530000002",
            "price": 100.0,
            "condition": "greater_than",
            "condition_type": "cross_up",
            "message": "(none)",
            "price_set": true,
            "created": true,
            "source": "internal_api"
        })]));

        let data = alert_create(&mut runtime, 100.0, "greater-than", None)
            .await
            .unwrap();

        assert_eq!(data["condition"], "greater_than");
        assert_eq!(data["condition_type"], "cross_up");
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
        let mut runtime = FakeRuntime::new(VecDeque::from([
            alert_create_api_fallback(),
            json!({
                "price": 100.0,
                "condition": "crossing",
                "message": "(none)",
                "price_set": false,
                "created": true,
                "source": "dom_fallback"
            }),
        ]));

        let error = alert_create(&mut runtime, 100.0, "crossing", None)
            .await
            .unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.message, "Alert price input could not be set");
    }

    #[tokio::test]
    async fn alert_create_fails_when_create_button_was_not_clicked() {
        let mut runtime = FakeRuntime::new(VecDeque::from([
            alert_create_api_fallback(),
            json!({
                "price": 100.0,
                "condition": "crossing",
                "message": "(none)",
                "price_set": true,
                "created": false,
                "source": "dom_fallback"
            }),
        ]));

        let error = alert_create(&mut runtime, 100.0, "crossing", None)
            .await
            .unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.message, "Alert create button could not be clicked");
    }

    #[tokio::test]
    async fn alert_create_api_post_check_failure_does_not_fallback_to_dom() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "error": "Alert create did not confirm a matching new alert",
            "error_kind": "internal_api_unavailable",
            "phase": "post_check_failed",
            "api_fallback_allowed": false,
            "price": 100.0,
            "condition": "crossing",
            "message": "(none)",
            "price_set": true,
            "created": false,
            "source": "internal_api"
        })]));

        let error = alert_create(&mut runtime, 100.0, "crossing", None)
            .await
            .unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(
            error.message,
            "Alert create did not confirm a matching new alert"
        );
        assert_eq!(runtime.evaluated.len(), 1);
    }

    #[tokio::test]
    async fn alert_create_api_request_failure_does_not_fallback_after_post_attempt() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "error": "HTTP 400: Bad Request",
            "error_kind": "internal_api_unavailable",
            "phase": "create_request_failed",
            "api_fallback_allowed": false,
            "price": 100.0,
            "condition": "crossing",
            "message": "(none)",
            "price_set": true,
            "created": false,
            "source": "internal_api"
        })]));

        let error = alert_create(&mut runtime, 100.0, "crossing", None)
            .await
            .unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.message, "HTTP 400: Bad Request");
        assert_eq!(runtime.evaluated.len(), 1);
    }

    #[tokio::test]
    async fn alert_indicator_dry_run_returns_sanitized_preview() {
        let source = r#"//@version=6
indicator("Signals")
plot(close)
alertcondition(close > open, "Long", "Long message")"#;
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "requested": "Signals",
            "match_count": 1,
            "match": {
                "name": "Signals",
                "title": "Signals",
                "version": 4,
                "modified": 123,
                "script_id_available": true
            },
            "candidates": []
        })]));

        let data = alert_create_indicator_dry_run(
            &mut runtime,
            IndicatorAlertDryRunRequest {
                script: "Signals",
                source,
                input_source: "stdin",
                condition_title: Some("Long"),
                alert_cond_id: None,
                symbol: Some("NASDAQ:AAPL"),
                resolution: Some("1D"),
                message: Some("Test alert"),
            },
        )
        .await
        .unwrap();

        assert_eq!(data["action"], "dry_run");
        assert_eq!(data["dry_run"], true);
        assert_eq!(data["would_create"], true);
        assert_eq!(data["mutation_supported"], false);
        assert_eq!(data["script"]["requested"], "Signals");
        assert_eq!(data["script"]["name"], "Signals");
        assert_eq!(data["script"]["script_id_available"], true);
        assert!(data["script"].get("id").is_none());
        assert_eq!(data["condition"]["alert_cond_id"], "plot_1");
        assert_eq!(data["condition"]["title"], "Long");
        assert_eq!(data["request"]["symbol"], "NASDAQ:AAPL");
        assert!(runtime.evaluated[0].0.contains("pine-facade/list"));
        assert!(!runtime.evaluated[0].0.contains("script_id:"));
    }

    #[tokio::test]
    async fn alert_indicator_dry_run_rejects_ambiguous_condition_selector() {
        let mut runtime = FakeRuntime::new(VecDeque::new());

        let error = alert_create_indicator_dry_run(
            &mut runtime,
            IndicatorAlertDryRunRequest {
                script: "Signals",
                source: "alertcondition(close > open, \"Long\")",
                input_source: "stdin",
                condition_title: Some("Long"),
                alert_cond_id: Some("plot_0"),
                symbol: None,
                resolution: None,
                message: None,
            },
        )
        .await
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(runtime.evaluated.is_empty());
    }

    #[tokio::test]
    async fn alert_indicator_dry_run_rejects_missing_saved_script_match() {
        let source = "alertcondition(close > open, \"Long\")";
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "requested": "Signals",
            "match_count": 0,
            "match": null,
            "candidates": [
                { "name": "Other", "title": "Other", "version": 1, "modified": null, "script_id_available": true }
            ]
        })]));

        let error = alert_create_indicator_dry_run(
            &mut runtime,
            IndicatorAlertDryRunRequest {
                script: "Signals",
                source,
                input_source: "stdin",
                condition_title: Some("Long"),
                alert_cond_id: None,
                symbol: None,
                resolution: None,
                message: None,
            },
        )
        .await
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert_eq!(error.message, "No saved Pine script matches --script");
        assert_eq!(error.details.unwrap()["match_count"], 0);
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
        assert!(!runtime.evaluated[0].0.contains("log_username"));
        assert!(!runtime.evaluated[0].0.contains("build_time"));
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
        assert!(runtime.evaluated[0].0.contains("delete_alerts"));
        assert!(!runtime.evaluated[0].0.contains("log_username"));
        assert!(!runtime.evaluated[0].0.contains("build_time"));
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
