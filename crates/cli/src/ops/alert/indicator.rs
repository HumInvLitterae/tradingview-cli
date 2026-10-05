mod saved_script;

use serde_json::{Value, json};

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};

use super::{
    super::{
        common::js_string,
        pine::{PineAlertconditionCandidate, pine_alertcondition_candidates},
    },
    ALERT_LIST_READER,
    payload::{
        normalize_indicator_alert_create_payload, normalize_indicator_alert_verified_payload,
    },
};
use saved_script::{VerifiedSavedPineScript, resolve_verified};

pub use tradingview_model::alert::IndicatorStudySelection;

const STUDY_INPUTS: &str = include_str!("indicator/study_inputs.js");
const VERIFICATION_SCRIPT: &str = include_str!("indicator/verification.js");

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

pub async fn alert_create_indicator(
    runtime: &mut impl RuntimeEvaluator,
    request: IndicatorAlertRequest<'_>,
) -> Result<Value, AppError> {
    create_indicator(runtime, request, None).await
}

/// Verify the saved compiled condition and the selected instance's input values.
pub async fn alert_create_indicator_verified(
    runtime: &mut impl RuntimeEvaluator,
    request: IndicatorAlertRequest<'_>,
    selection: IndicatorStudySelection<'_>,
) -> Result<Value, AppError> {
    let selection = match selection {
        IndicatorStudySelection::Automatic => IndicatorStudySelection::Automatic,
        IndicatorStudySelection::Entity(id) => {
            IndicatorStudySelection::Entity(require_non_empty(id, "study_id")?)
        }
    };
    create_indicator(runtime, request, Some(selection)).await
}

async fn create_indicator(
    runtime: &mut impl RuntimeEvaluator,
    request: IndicatorAlertRequest<'_>,
    selection: Option<IndicatorStudySelection<'_>>,
) -> Result<Value, AppError> {
    let script = require_non_empty(request.script, "script")?;
    let candidate = select_alertcondition_candidate(
        request.source,
        request.condition_title,
        request.alert_cond_id,
    )?;
    let saved_script = resolve_verified(runtime, script, request.source).await?;

    if request.dry_run && selection.is_none() {
        return Ok(indicator_preview(
            script,
            &candidate,
            &saved_script,
            &request,
        ));
    }

    let prepared = alert_create_indicator_via_api(
        runtime,
        script,
        &candidate,
        &saved_script,
        &request,
        selection,
    )
    .await?;
    if request.dry_run {
        let mut preview = indicator_preview(script, &candidate, &saved_script, &request);
        preview["verification"] = prepared["verification"].clone();
        preview["request"]["symbol"] = prepared["symbol"].clone();
        preview["request"]["resolution"] = prepared["resolution"].clone();
        preview["note"] = json!(
            "Dry run only. Saved compilation and required chart inputs passed preflight. No alert was listed or created; provider acceptance and creation readback were not exercised."
        );
        Ok(preview)
    } else {
        Ok(prepared)
    }
}

fn indicator_preview(
    script: &str,
    candidate: &PineAlertconditionCandidate,
    saved_script: &VerifiedSavedPineScript,
    request: &IndicatorAlertRequest<'_>,
) -> Value {
    json!({
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
            "script_id_available": true,
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
    })
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

async fn alert_create_indicator_via_api(
    runtime: &mut impl RuntimeEvaluator,
    script: &str,
    candidate: &PineAlertconditionCandidate,
    saved_script: &VerifiedSavedPineScript,
    request: &IndicatorAlertRequest<'_>,
    selection: Option<IndicatorStudySelection<'_>>,
) -> Result<Value, AppError> {
    let script_literal = js_string(script)?;
    let script_name_literal = js_string(&saved_script.name)?;
    let script_title_literal = saved_script
        .title
        .as_deref()
        .map(js_string)
        .transpose()?
        .unwrap_or_else(|| "null".to_string());
    let script_id_literal = js_string(&saved_script.id)?;
    let pine_version = saved_script.version.to_string();
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
    let verification_request = match selection {
        None => Value::Null,
        Some(IndicatorStudySelection::Automatic) => json!({"study_id": null}),
        Some(IndicatorStudySelection::Entity(id)) => json!({"study_id": id}),
    };
    let dry_run = request.dry_run;

    let result = runtime
        .evaluate(
            &format!(
                r#"
            (async function() {{
                const source = {dry_run} ? 'indicator_alert_dry_run' : 'indicator_alert_api';
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
                const verificationRequest = {verification_request};
                const dryRun = {dry_run};
                const baseInputs = {{
                    pineFeatures: JSON.stringify(pineFeatures),
                    __fast_calc: false,
                    __profile: false
                }};

                {STUDY_INPUTS}
                {VERIFICATION_SCRIPT}

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

                {ALERT_LIST_READER}
                async function listAlerts() {{
                    const result = await __readAlertRows();
                    if (!result.ok) return result;
                    return {{ ok: true, rows: result.alerts, alerts: result.alerts.map(publicAlert) }};
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

                function legacyStudyInputs(chart) {{
                    const base = baseInputs;
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
                        const unavailable = {{
                            ok: false,
                            error: 'Active study inputs could not be verified against declared input IDs'
                        }};
                        let values;
                        let declared;
                        try {{
                            const source = study && (study._study || study);
                            const meta = source && typeof source.metaInfo === 'function' ? source.metaInfo() : null;
                            declared = meta && meta.inputs;
                            values = study && typeof study.getInputValues === 'function' ? study.getInputValues() : null;
                        }} catch (_) {{
                            return unavailable;
                        }}
                        const inputs = nativeStudyInputs(declared, values, base);
                        if (!inputs.ok) return inputs;
                        return {{
                            ...inputs,
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

                const compiled = verificationRequest ? await savedCompilation() : null;
                if (compiled && !compiled.ok) return compiled;

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

                const studyInputs = verificationRequest
                    ? verifiedStudyInputs(chartMeta.chart, baseInputs, verificationRequest.study_id, compiled.userIds)
                    : legacyStudyInputs(chartMeta.chart);
                if (!studyInputs.ok) {{
                    if (verificationRequest) return studyInputs;
                    return {{
                        error: studyInputs.error,
                        error_kind: 'internal_api_unavailable',
                        phase: 'study_input_metadata_unavailable',
                        created: false,
                        source,
                        input_metadata_required: sourceHasInputs
                    }};
                }}

                if (dryRun) {{
                    return {{
                        action: 'dry_run',
                        created: false,
                        symbol: chartMeta.symbol,
                        resolution: String(chartMeta.resolution),
                        verification: studyInputs.verification
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
                        created: null,
                        creation_outcome: 'unknown',
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
                        created: null,
                        creation_outcome: 'unknown',
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
                        created: null,
                        creation_outcome: 'unknown',
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
                        created: null,
                        creation_outcome: 'unknown',
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
                    matched_alert: publicMatched,
                    ...(verificationRequest ? {{verification: studyInputs.verification}} : {{}})
                }};
            }})()
            "#,
                plot_index = candidate.plot_index,
                confidence = js_string(candidate.confidence)?
            ),
            true,
        )
        .await;

    let result = result.and_then(|data| match selection {
        Some(selection) => {
            normalize_indicator_alert_verified_payload(data, request.dry_run, selection)
        }
        None => normalize_indicator_alert_create_payload(data),
    });
    result.map_err(|mut error| {
        let mut details = error
            .details
            .take()
            .and_then(|value| value.as_object().cloned())
            .unwrap_or_default();
        // A preview cannot dispatch; normal failures need returned preflight proof.
        if request.dry_run {
            details.insert("created".into(), json!(false));
            details.remove("creation_outcome");
        } else if details.get("created").and_then(Value::as_bool) != Some(false) {
            details.insert("created".into(), Value::Null);
            details.insert("creation_outcome".into(), json!("unknown"));
        }
        error.with_details(Value::Object(details))
    })
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
mod tests;
