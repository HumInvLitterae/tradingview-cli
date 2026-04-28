use serde_json::{Value, json};

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};

use super::{
    super::{
        common::js_string,
        pine::{PineAlertconditionCandidate, pine_alertcondition_candidates},
    },
    payload::normalize_indicator_alert_create_payload,
};

#[derive(Debug, Clone)]
pub struct IndicatorAlertRequest<'a> {
    pub script: &'a str,
    pub source: &'a str,
    pub input_source: &'static str,
    pub condition_title: Option<&'a str>,
    pub alert_cond_id: Option<&'a str>,
    pub symbol: Option<&'a str>,
    pub resolution: Option<&'a str>,
    pub message: Option<&'a str>,
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
struct SavedPineScriptMatch {
    id: Option<String>,
    name: String,
    title: Option<String>,
    version: Option<Value>,
    modified: Option<Value>,
    script_id_available: bool,
}

pub async fn alert_create_indicator(
    runtime: &mut impl RuntimeEvaluator,
    request: IndicatorAlertRequest<'_>,
) -> Result<Value, AppError> {
    let script = require_non_empty(request.script, "script")?;
    let candidate = select_alertcondition_candidate(
        request.source,
        request.condition_title,
        request.alert_cond_id,
    )?;
    let saved_script = resolve_saved_pine_script(runtime, script).await?;

    if request.dry_run {
        return Ok(json!({
        "action": "dry_run",
        "dry_run": true,
        "would_create": true,
        "mutation_supported": true,
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
        "note": "Dry run only. No TradingView alert was created. Normal create still requires saved script metadata and post-create readback.",
        }));
    }

    alert_create_indicator_via_api(runtime, script, &candidate, &saved_script, &request).await
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
) -> Result<SavedPineScriptMatch, AppError> {
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
                            script_id: s.scriptIdPart || null,
                            script_id_available: !!s.scriptIdPart
                        }};
                    }});
                    function publicScript(script) {{
                        return {{
                            name: script.name,
                            title: script.title,
                            version: script.version,
                            modified: script.modified,
                            script_id_available: script.script_id_available
                        }};
                    }}
                    const matches = scripts.filter(function(script) {{
                        return script.name === requested || script.title === requested;
                    }});
                    return {{
                        requested: requested,
                        match_count: matches.length,
                        match: matches.length === 1 ? matches[0] : null,
                        candidates: matches.length === 1 ? [] : scripts.slice(0, 20).map(publicScript)
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

fn normalize_saved_pine_script_match(data: Value) -> Result<SavedPineScriptMatch, AppError> {
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
    let id = matched
        .get("script_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string);
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

    Ok(SavedPineScriptMatch {
        id,
        name,
        title,
        version,
        modified,
        script_id_available,
    })
}

async fn alert_create_indicator_via_api(
    runtime: &mut impl RuntimeEvaluator,
    script: &str,
    candidate: &PineAlertconditionCandidate,
    saved_script: &SavedPineScriptMatch,
    request: &IndicatorAlertRequest<'_>,
) -> Result<Value, AppError> {
    let script_id = saved_script.id.as_deref().ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Saved Pine script id was unavailable for indicator alert creation",
        )
        .with_details(json!({
            "script": {
                "requested": script,
                "name": saved_script.name,
                "title": saved_script.title,
                "script_id_available": saved_script.script_id_available,
            },
            "phase": "saved_script_metadata_unavailable",
        }))
    })?;

    let script_literal = js_string(script)?;
    let script_name_literal = js_string(&saved_script.name)?;
    let script_title_literal = saved_script
        .title
        .as_deref()
        .map(js_string)
        .transpose()?
        .unwrap_or_else(|| "null".to_string());
    let script_id_literal = js_string(script_id)?;
    let pine_version = saved_script
        .version
        .as_ref()
        .and_then(|value| value.as_str().map(str::to_string))
        .or_else(|| saved_script.version.as_ref().map(Value::to_string))
        .unwrap_or_else(|| "1.0".to_string());
    let pine_version_literal = js_string(&pine_version)?;
    let alert_cond_id_literal = js_string(&candidate.alert_cond_id)?;
    let condition_title_literal = candidate
        .title
        .as_deref()
        .map(js_string)
        .transpose()?
        .unwrap_or_else(|| "null".to_string());
    let condition_message_literal = candidate
        .message
        .as_deref()
        .map(js_string)
        .transpose()?
        .unwrap_or_else(|| "null".to_string());
    let requested_message = request
        .message
        .and_then(|value| {
            let trimmed = value.trim();
            (!trimmed.is_empty()).then_some(trimmed)
        })
        .or(candidate.message.as_deref())
        .unwrap_or("(none)");
    let message_literal = js_string(requested_message)?;
    let symbol_literal = request
        .symbol
        .and_then(|value| {
            let trimmed = value.trim();
            (!trimmed.is_empty()).then_some(trimmed)
        })
        .map(js_string)
        .transpose()?
        .unwrap_or_else(|| "null".to_string());
    let resolution_literal = request
        .resolution
        .and_then(|value| {
            let trimmed = value.trim();
            (!trimmed.is_empty()).then_some(trimmed)
        })
        .map(js_string)
        .transpose()?
        .unwrap_or_else(|| "null".to_string());
    let offsets_by_plot = offsets_by_plot(candidate.plot_index);
    let offsets_json = serde_json::to_string(&offsets_by_plot).map_err(|err| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!("Could not serialize plot offsets: {err}"),
        )
    })?;
    let pine_features = pine_features(request.source);
    let pine_features_json = serde_json::to_string(&pine_features).map_err(|err| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!("Could not serialize Pine feature metadata: {err}"),
        )
    })?;
    let source_has_inputs = source_has_pine_inputs(request.source);

    let result = runtime
        .evaluate(
            &format!(
                r#"
            (async function() {{
                const source = 'indicator_alert_api';
                const requestedScript = {script_literal};
                const savedScriptName = {script_name_literal};
                const savedScriptTitle = {script_title_literal};
                const pineId = {script_id_literal};
                const pineVersion = {pine_version_literal};
                const requestedAlertCondId = {alert_cond_id_literal};
                const requestedConditionTitle = {condition_title_literal};
                const conditionSourceMessage = {condition_message_literal};
                const requestedMessage = {message_literal};
                const requestedSymbol = {symbol_literal};
                const requestedResolution = {resolution_literal};
                const offsetsByPlot = {offsets_json};
                const pineFeatures = {pine_features_json};
                const sourceHasInputs = {source_has_inputs};

                function publicAlert(alert) {{
                    if (!alert) return null;
                    const condition = alertCondition(alert);
                    return {{
                        alert_id: alert.alert_id || alert.id || null,
                        symbol: alert.symbol || (alert.condition && alert.condition.symbol) || null,
                        type: alert.type || null,
                        message: alert.message || alert.description || '',
                        active: alert.active !== false,
                        resolution: alert.resolution || alert.interval || (condition && condition.resolution) || null,
                        created: alert.created || alert.create_time || null,
                        expiration: alert.expiration || alert.expire_time || null,
                        condition: condition ? {{
                            type: condition.type || null,
                            alert_cond_id: condition.alert_cond_id || null,
                            frequency: condition.frequency || null,
                            resolution: condition.resolution || null,
                            has_study_series: hasStudySeries(condition)
                        }} : null
                    }};
                }}

                function normalizeRows(data) {{
                    return Array.isArray(data && data.r) ? data.r : [];
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
                    const rows = normalizeRows(data);
                    return {{ ok: true, rows, alerts: rows.map(publicAlert) }};
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
                        const symbol = requestedSymbol ||
                            (mainSeries && mainSeries.symbol && mainSeries.symbol()) ||
                            (ext && (ext.pro_name || ext.full_name || ext.symbol)) ||
                            (info && (info.pro_name || info.full_name || info.symbol)) ||
                            null;
                        const resolution = String(
                            requestedResolution ||
                            (chart && chart.resolution && chart.resolution()) ||
                            (mainSeries && mainSeries.interval && mainSeries.interval()) ||
                            '1'
                        );
                        const currency = (ext && (ext.currency_id || ext.currency || ext['currency-id'])) ||
                            (info && (info.currency_id || info.currency_code || info.currency || info['currency-id'])) ||
                            'USD';
                        if (!symbol) {{
                            return {{ error: 'Active chart symbol unavailable and --symbol was not provided' }};
                        }}
                        return {{ chart, symbol, resolution, currency }};
                    }} catch (error) {{
                        return {{
                            error: error && error.message ? error.message : String(error)
                        }};
                    }}
                }}

                function labelOf(value) {{
                    if (!value) return null;
                    if (typeof value === 'string') return value;
                    if (typeof value.name === 'function') {{
                        try {{ return value.name(); }} catch (_) {{}}
                    }}
                    if (typeof value.title === 'function') {{
                        try {{ return value.title(); }} catch (_) {{}}
                    }}
                    return value.name || value.title || value.description || value.shortDescription || null;
                }}

                function sameScriptLabel(label) {{
                    if (!label) return false;
                    return label === savedScriptName || label === savedScriptTitle || label === requestedScript;
                }}

                function readStudyInputs(chart) {{
                    const base = {{
                        pineFeatures: JSON.stringify(pineFeatures),
                        __fast_calc: false,
                        __profile: false
                    }};
                    if (!chart || typeof chart.getAllStudies !== 'function') {{
                        if (sourceHasInputs) {{
                            return {{
                                ok: false,
                                error: 'Active chart study list is unavailable for Pine input metadata'
                            }};
                        }}
                        return {{ ok: true, inputs: base, input_count: 0, input_source: 'default_no_inputs', study_matched: false }};
                    }}

                    const studies = chart.getAllStudies() || [];
                    for (let i = 0; i < studies.length; i++) {{
                        const info = studies[i] || {{}};
                        let study = null;
                        try {{
                            study = info.id && chart.getStudyById ? chart.getStudyById(info.id) : null;
                        }} catch (_) {{}}
                        const labels = [
                            labelOf(info),
                            labelOf(study),
                            labelOf(study && study._study),
                            info.name,
                            info.title
                        ].filter(Boolean);
                        if (!labels.some(sameScriptLabel)) continue;
                        let values = [];
                        try {{
                            if (study && typeof study.getInputValues === 'function') {{
                                values = study.getInputValues() || [];
                            }}
                        }} catch (error) {{
                            return {{
                                ok: false,
                                error: 'Could not read active study input values: ' + (error && error.message ? error.message : String(error))
                            }};
                        }}
                        const inputs = Object.assign({{}}, base);
                        for (let j = 0; j < values.length; j++) {{
                            const input = values[j] || {{}};
                            let value = input.value;
                            if (value === undefined && input.val !== undefined) value = input.val;
                            if (value === undefined && input.defval !== undefined) value = input.defval;
                            inputs['in_' + j] = value;
                        }}
                        return {{
                            ok: true,
                            inputs,
                            input_count: values.length,
                            input_source: 'active_chart_study',
                            study_matched: true
                        }};
                    }}

                    if (sourceHasInputs) {{
                        return {{
                            ok: false,
                            error: 'Pine source declares input.* calls, but no matching active chart study exposed input values'
                        }};
                    }}
                    return {{ ok: true, inputs: base, input_count: 0, input_source: 'default_no_inputs', study_matched: false }};
                }}

                function alertIds(rows) {{
                    const ids = {{}};
                    rows.forEach(function(alert) {{
                        const id = alert && (alert.alert_id || alert.id);
                        if (id !== null && id !== undefined) ids[String(id)] = true;
                    }});
                    return ids;
                }}

                function alertCondition(alert) {{
                    if (!alert) return null;
                    if (alert.condition && typeof alert.condition === 'object') return alert.condition;
                    if (Array.isArray(alert.conditions) && alert.conditions.length > 0) return alert.conditions[0];
                    return null;
                }}

                function hasStudySeries(condition) {{
                    return !!(condition && Array.isArray(condition.series) && condition.series.some(function(series) {{
                        return series && series.type === 'study';
                    }}));
                }}

                function conditionAlertCondId(condition) {{
                    if (!condition) return null;
                    if (condition.alert_cond_id) return condition.alert_cond_id;
                    if (condition.alertCondId) return condition.alertCondId;
                    if (Array.isArray(condition.series)) {{
                        for (let i = 0; i < condition.series.length; i++) {{
                            const series = condition.series[i] || {{}};
                            if (series.alert_cond_id) return series.alert_cond_id;
                            if (series.alertCondId) return series.alertCondId;
                        }}
                    }}
                    return null;
                }}

                function matchingNewAlert(rows, beforeIds, symbolMarker) {{
                    for (let i = 0; i < rows.length; i++) {{
                        const alert = rows[i] || {{}};
                        const id = alert.alert_id || alert.id;
                        if (id !== null && id !== undefined && beforeIds[String(id)]) continue;
                        const condition = alertCondition(alert);
                        if (!condition || condition.type !== 'alert_cond') continue;
                        if (conditionAlertCondId(condition) !== requestedAlertCondId) continue;
                        const message = alert.message || alert.description || '';
                        if (message !== requestedMessage) continue;
                        const symbol = alert.symbol || (alert.condition && alert.condition.symbol) || null;
                        if (symbol && symbol !== symbolMarker) continue;
                        return alert;
                    }}
                    return null;
                }}

                const chartMeta = readChartMetadata();
                if (chartMeta.error) {{
                    return {{
                        error: chartMeta.error,
                        error_kind: 'internal_api_unavailable',
                        phase: 'chart_metadata_unavailable',
                        created: false,
                        source
                    }};
                }}

                const studyInputs = readStudyInputs(chartMeta.chart);
                if (!studyInputs.ok) {{
                    return {{
                        error: studyInputs.error,
                        error_kind: 'internal_api_unavailable',
                        phase: 'study_input_metadata_unavailable',
                        created: false,
                        source,
                        input_metadata_required: sourceHasInputs
                    }};
                }}

                const before = await listAlerts();
                if (!before.ok) {{
                    return {{
                        error: before.error,
                        error_kind: 'internal_api_unavailable',
                        phase: 'pre_list_unavailable',
                        created: false,
                        source
                    }};
                }}

                const symbolMarker = '=' + JSON.stringify({{
                    symbol: chartMeta.symbol,
                    adjustment: 'dividends',
                    'currency-id': chartMeta.currency
                }});
                const expiration = new Date(Date.now() + 30 * 24 * 60 * 60 * 1000).toISOString();
                const payload = {{
                    symbol: symbolMarker,
                    resolution: String(chartMeta.resolution),
                    message: requestedMessage,
                    sound_file: null,
                    sound_duration: 0,
                    popup: false,
                    expiration,
                    auto_deactivate: false,
                    email: false,
                    sms_over_email: false,
                    mobile_push: false,
                    web_hook: null,
                    name: null,
                    conditions: [{{
                        type: 'alert_cond',
                        frequency: 'on_bar_close',
                        alert_cond_id: requestedAlertCondId,
                        series: [{{
                            type: 'study',
                            study: 'Script@tv-scripting-101',
                            offsets_by_plot: offsetsByPlot,
                            inputs: studyInputs.inputs,
                            pine_id: pineId,
                            pine_version: pineVersion
                        }}],
                        resolution: String(chartMeta.resolution)
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
                        created: false,
                        source,
                        before_count: before.rows.length
                    }};
                }}

                if (!createResponse.ok || (createData && createData.err) || (createData && createData.s && createData.s !== 'ok')) {{
                    return {{
                        error: createData && createData.errmsg
                            ? createData.errmsg
                            : createData && createData.err && createData.err.code
                                ? createData.err.code
                                : 'HTTP ' + createResponse.status + ': ' + createResponse.statusText,
                        error_kind: 'internal_api_unavailable',
                        phase: 'create_request_failed',
                        created: false,
                        source,
                        before_count: before.rows.length,
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
                        created: false,
                        source,
                        symbol: chartMeta.symbol,
                        resolution: chartMeta.resolution,
                        before_count: before.rows.length
                    }};
                }}

                const matched = matchingNewAlert(after.rows, alertIds(before.rows), symbolMarker);
                if (!matched) {{
                    return {{
                        error: 'Indicator alert create did not confirm a matching new alert',
                        error_kind: 'internal_api_unavailable',
                        phase: 'post_check_failed',
                        created: false,
                        source,
                        symbol: chartMeta.symbol,
                        resolution: chartMeta.resolution,
                        alert_cond_id: requestedAlertCondId,
                        before_count: before.rows.length,
                        after_count: after.rows.length
                    }};
                }}

                const publicMatched = publicAlert(matched);
                return {{
                    action: 'create_indicator',
                    dry_run: false,
                    alert_id: publicMatched && publicMatched.alert_id || null,
                    created: true,
                    source,
                    symbol: chartMeta.symbol,
                    resolution: String(chartMeta.resolution),
                    message: requestedMessage,
                    before_count: before.rows.length,
                    after_count: after.rows.length,
                    script: {{
                        requested: requestedScript,
                        name: savedScriptName,
                        title: savedScriptTitle,
                        version: pineVersion,
                        script_id_available: true
                    }},
                    condition: {{
                        alert_cond_id: requestedAlertCondId,
                        title: requestedConditionTitle,
                        message: conditionSourceMessage,
                        plot_index: {plot_index},
                        confidence: {confidence}
                    }},
                    input_metadata: {{
                        source: studyInputs.input_source,
                        input_count: studyInputs.input_count,
                        study_matched: studyInputs.study_matched,
                        source_has_inputs: sourceHasInputs
                    }},
                    matched_alert: publicMatched
                }};
            }})()
            "#,
                plot_index = candidate.plot_index,
                confidence = js_string(candidate.confidence)?
            ),
            true,
        )
        .await?;

    normalize_indicator_alert_create_payload(result)
}

fn offsets_by_plot(plot_index: usize) -> Value {
    let mut map = serde_json::Map::new();
    for index in 0..plot_index {
        map.insert(format!("plot_{index}"), json!(0));
    }
    Value::Object(map)
}

fn source_has_pine_inputs(source: &str) -> bool {
    source.contains("input.") || source.contains("input(")
}

fn pine_features(source: &str) -> Value {
    let mut features = serde_json::Map::new();
    for (needle, key) in [
        ("indicator(", "indicator"),
        ("strategy(", "strategy"),
        ("plot(", "plot"),
        ("plotshape(", "plotshape"),
        ("plotchar(", "plotchar"),
        ("bgcolor(", "bgcolor"),
        ("alertcondition(", "alertcondition"),
        ("request.security", "request.security"),
        ("ta.", "ta"),
        ("math.", "math"),
        ("array.", "array"),
        ("line.", "line"),
        ("label.", "label"),
        ("box.", "box"),
        ("table.", "table"),
        ("input.", "input"),
        ("input(", "input"),
    ] {
        if source.contains(needle) {
            features.insert(key.to_string(), json!(1));
        }
    }
    Value::Object(features)
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use serde_json::json;

    use super::super::super::test_support::FakeRuntime;
    use super::*;

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

        let data = alert_create_indicator(
            &mut runtime,
            IndicatorAlertRequest {
                script: "Signals",
                source,
                input_source: "stdin",
                condition_title: Some("Long"),
                alert_cond_id: None,
                symbol: Some("NASDAQ:AAPL"),
                resolution: Some("1D"),
                message: Some("Test alert"),
                dry_run: true,
            },
        )
        .await
        .unwrap();

        assert_eq!(data["action"], "dry_run");
        assert_eq!(data["dry_run"], true);
        assert_eq!(data["would_create"], true);
        assert_eq!(data["mutation_supported"], true);
        assert_eq!(data["script"]["requested"], "Signals");
        assert_eq!(data["script"]["name"], "Signals");
        assert_eq!(data["script"]["script_id_available"], true);
        assert!(data["script"].get("id").is_none());
        assert_eq!(data["condition"]["alert_cond_id"], "plot_1");
        assert_eq!(data["condition"]["title"], "Long");
        assert_eq!(data["request"]["symbol"], "NASDAQ:AAPL");
        assert!(runtime.evaluated[0].0.contains("pine-facade/list"));
        assert!(data["script"].get("script_id").is_none());
    }

    #[tokio::test]
    async fn alert_indicator_dry_run_rejects_ambiguous_condition_selector() {
        let mut runtime = FakeRuntime::new(VecDeque::new());

        let error = alert_create_indicator(
            &mut runtime,
            IndicatorAlertRequest {
                script: "Signals",
                source: "alertcondition(close > open, \"Long\")",
                input_source: "stdin",
                condition_title: Some("Long"),
                alert_cond_id: Some("plot_0"),
                symbol: None,
                resolution: None,
                message: None,
                dry_run: true,
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

        let error = alert_create_indicator(
            &mut runtime,
            IndicatorAlertRequest {
                script: "Signals",
                source,
                input_source: "stdin",
                condition_title: Some("Long"),
                alert_cond_id: None,
                symbol: None,
                resolution: None,
                message: None,
                dry_run: true,
            },
        )
        .await
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert_eq!(error.message, "No saved Pine script matches --script");
        assert_eq!(error.details.unwrap()["match_count"], 0);
    }

    #[tokio::test]
    async fn alert_indicator_create_returns_sanitized_success() {
        let source = r#"//@version=6
indicator("Signals")
plot(close)
alertcondition(close > open, "Long", "Long message")"#;
        let mut runtime = FakeRuntime::new(VecDeque::from([
            json!({
                "requested": "Signals",
                "match_count": 1,
                "match": {
                    "name": "Signals",
                    "title": "Signals",
                    "version": 4,
                    "modified": 123,
                    "script_id": "SAVED_SCRIPT_ID_REDACTED",
                    "script_id_available": true
                },
                "candidates": []
            }),
            json!({
                "action": "create_indicator",
                "dry_run": false,
                "alert_id": "4550000001",
                "created": true,
                "source": "indicator_alert_api",
                "symbol": "NASDAQ:AAPL",
                "resolution": "1D",
                "message": "Long message",
                "before_count": 1,
                "after_count": 2,
                "script": {
                    "requested": "Signals",
                    "name": "Signals",
                    "title": "Signals",
                    "version": "4",
                    "script_id_available": true
                },
                "condition": {
                    "alert_cond_id": "plot_1",
                    "title": "Long",
                    "message": "Long message",
                    "plot_index": 1,
                    "confidence": "best_effort"
                },
                "input_metadata": {
                    "source": "default_no_inputs",
                    "input_count": 0,
                    "study_matched": false,
                    "source_has_inputs": false
                },
                "matched_alert": {
                    "alert_id": "4550000001",
                    "message": "Long message",
                    "condition": {
                        "type": "alert_cond",
                        "alert_cond_id": "plot_1",
                        "has_study_series": true
                    }
                }
            }),
        ]));

        let data = alert_create_indicator(
            &mut runtime,
            IndicatorAlertRequest {
                script: "Signals",
                source,
                input_source: "stdin",
                condition_title: Some("Long"),
                alert_cond_id: None,
                symbol: Some("NASDAQ:AAPL"),
                resolution: Some("1D"),
                message: None,
                dry_run: false,
            },
        )
        .await
        .unwrap();

        assert_eq!(data["action"], "create_indicator");
        assert_eq!(data["dry_run"], false);
        assert_eq!(data["created"], true);
        assert_eq!(data["source"], "indicator_alert_api");
        assert_eq!(data["alert_id"], "4550000001");
        assert_eq!(data["condition"]["alert_cond_id"], "plot_1");
        assert!(data["script"].get("id").is_none());
        assert!(data["matched_alert"]["condition"].get("pine_id").is_none());
        assert_eq!(runtime.evaluated.len(), 2);
        assert!(runtime.evaluated[1].0.contains("create_alert"));
        assert!(runtime.evaluated[1].0.contains("list_alerts"));
        assert!(!runtime.evaluated[1].0.contains("Content-Type"));
    }

    #[tokio::test]
    async fn alert_indicator_create_rejects_missing_script_id_before_create_request() {
        let source = "alertcondition(close > open, \"Long\")";
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "requested": "Signals",
            "match_count": 1,
            "match": {
                "name": "Signals",
                "title": "Signals",
                "version": 1,
                "modified": null,
                "script_id": null,
                "script_id_available": false
            },
            "candidates": []
        })]));

        let error = alert_create_indicator(
            &mut runtime,
            IndicatorAlertRequest {
                script: "Signals",
                source,
                input_source: "stdin",
                condition_title: Some("Long"),
                alert_cond_id: None,
                symbol: None,
                resolution: None,
                message: None,
                dry_run: false,
            },
        )
        .await
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(
            error.message,
            "Saved Pine script id was unavailable for indicator alert creation"
        );
        assert_eq!(runtime.evaluated.len(), 1);
    }

    #[tokio::test]
    async fn alert_indicator_create_post_check_failure_does_not_fallback() {
        let source = "alertcondition(close > open, \"Long\")";
        let mut runtime = FakeRuntime::new(VecDeque::from([
            json!({
                "requested": "Signals",
                "match_count": 1,
                "match": {
                    "name": "Signals",
                    "title": "Signals",
                    "version": 1,
                    "modified": null,
                    "script_id": "SAVED_SCRIPT_ID_REDACTED",
                    "script_id_available": true
                },
                "candidates": []
            }),
            json!({
                "error": "Indicator alert create did not confirm a matching new alert",
                "error_kind": "internal_api_unavailable",
                "phase": "post_check_failed",
                "created": false,
                "source": "indicator_alert_api"
            }),
        ]));

        let error = alert_create_indicator(
            &mut runtime,
            IndicatorAlertRequest {
                script: "Signals",
                source,
                input_source: "stdin",
                condition_title: Some("Long"),
                alert_cond_id: None,
                symbol: None,
                resolution: None,
                message: None,
                dry_run: false,
            },
        )
        .await
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(
            error.message,
            "Indicator alert create did not confirm a matching new alert"
        );
        assert_eq!(runtime.evaluated.len(), 2);
    }
}
