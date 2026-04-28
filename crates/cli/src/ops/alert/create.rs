use serde_json::{Value, json};

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};

use super::{
    super::common::{js_string, require_finite},
    payload::{alert_api_error_allows_fallback, normalize_alert_create_payload},
};

const ALERT_CONDITIONS: [&str; 3] = ["crossing", "greater_than", "less_than"];

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

    use serde_json::{Value, json};

    use super::super::super::test_support::FakeRuntime;
    use super::*;

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
}
