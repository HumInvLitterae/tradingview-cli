use serde_json::{Map, Value};

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};

use super::common::{CHART_API, js_string};

pub fn parse_indicator_inputs(raw: &str) -> Result<Value, AppError> {
    let value: Value = serde_json::from_str(raw).map_err(|err| {
        AppError::new(
            ErrorKind::Validation,
            format!("--inputs must be a JSON object: {err}"),
        )
    })?;

    let Some(object) = value.as_object() else {
        return Err(AppError::new(
            ErrorKind::Validation,
            "--inputs must be a JSON object",
        ));
    };
    if object.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "--inputs must not be empty",
        ));
    }

    Ok(value)
}

pub fn parse_indicator_add_inputs(raw: &str) -> Result<Value, AppError> {
    const MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_991;
    let value = parse_indicator_inputs(raw)?;
    let object = value
        .as_object()
        .expect("parse_indicator_inputs guarantees an object");
    if let Some((key, _)) = object.iter().find(|(_, value)| {
        !matches!(
            value,
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_)
        )
    }) {
        return Err(AppError::new(
            ErrorKind::Validation,
            format!("Indicator add input `{key}` must be null, boolean, number, or string"),
        ));
    }
    if let Some((key, _)) = object.iter().find(|(_, value)| {
        let Value::Number(number) = value else {
            return false;
        };
        if let Some(value) = number.as_i64() {
            return !(-MAX_SAFE_INTEGER..=MAX_SAFE_INTEGER).contains(&value);
        }
        if let Some(value) = number.as_u64() {
            return value > MAX_SAFE_INTEGER as u64;
        }
        number.as_f64().is_none_or(|value| {
            !value.is_finite() || (value.fract() == 0.0 && value.abs() > MAX_SAFE_INTEGER as f64)
        })
    }) {
        return Err(AppError::new(
            ErrorKind::Validation,
            format!(
                "Indicator add input `{key}` must be losslessly representable as a JavaScript number"
            ),
        ));
    }
    Ok(value)
}

pub async fn indicator_add(
    runtime: &mut impl RuntimeEvaluator,
    indicator: &str,
    inputs: Option<&Value>,
) -> Result<Value, AppError> {
    let indicator_literal = js_string(indicator)?;
    let inputs_json = inputs
        .map(serde_json::to_string)
        .transpose()
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))?
        .unwrap_or_else(|| "{}".to_string());
    let inputs_literal = js_string(&inputs_json)?;
    let expected_input_count = inputs.and_then(Value::as_object).map_or(0, Map::len) as u64;

    let data = runtime
        .evaluate(
            &format!(
                r#"
                (async function() {{
                    function studyRows(chart) {{
                        return chart.getAllStudies().map(function(study) {{
                            return {{ id: study.id, name: typeof study.name === "string" ? study.name : null }};
                        }});
                    }}
                    function cleanup(chart, rows) {{
                        var result = {{ cleanup_candidate_count: rows.length, cleanup_removed: false, cleanup_absent: false }};
                        if (rows.length !== 1 || typeof rows[0].id !== "string" || !rows[0].id) return result;
                        try {{
                            chart.removeEntity(rows[0].id);
                            result.cleanup_removed = true;
                            result.cleanup_absent = studyRows(chart).every(function(row) {{ return row.id !== rows[0].id; }});
                        }} catch(e) {{}}
                        return result;
                    }}
                    function failure(stage, mutationPerformed, extra, cleanupRows) {{
                        var result = {{
                            action: "add",
                            indicator: {indicator_literal},
                            success: false,
                            error_stage: stage,
                            mutation_performed: mutationPerformed,
                            new_study_count: null,
                            awaited: false,
                            name_verified: false,
                            inputs_verified: false
                        }};
                        if (extra) Object.keys(extra).forEach(function(key) {{ result[key] = extra[key]; }});
                        if (mutationPerformed && Array.isArray(cleanupRows)) Object.assign(result, cleanup(chart, cleanupRows));
                        return result;
                    }}

                    var chart = {CHART_API};
                    var overrides = JSON.parse({inputs_literal});
                    var requestedName = String({indicator_literal}).trim();
                    var repository = null;
                    var candidates = [];
                    try {{
                        repository = chart.studyMetaIntoRepository().getInternalMetaInfoArray();
                        candidates = repository.filter(function(meta) {{
                            return meta && typeof meta.description === "string" && meta.description.trim() === requestedName;
                        }});
                    }} catch(e) {{
                        return failure("metainfo_unavailable", false, {{ candidate_count: 0 }});
                    }}
                    if (candidates.length !== 1) {{
                        return failure("indicator_resolution", false, {{ candidate_count: candidates.length }});
                    }}
                    var meta = candidates[0];
                    if (typeof meta.id !== "string" || !meta.id || typeof meta.is_price_study !== "boolean" || !Array.isArray(meta.inputs)) {{
                        return failure("metainfo_unavailable", false, {{ candidate_count: 1 }});
                    }}
                    var definitions = new Set();
                    meta.inputs.forEach(function(input) {{ if (input && typeof input.id === "string") definitions.add(input.id); }});
                    var requestedKeys = Object.keys(overrides);
                    var unmatched = requestedKeys.filter(function(key) {{ return !definitions.has(key); }});
                    if (unmatched.length > 0) {{
                        return failure("input_validation", false, {{ unmatched_input_count: unmatched.length }});
                    }}

                    var beforeRows = studyRows(chart);
                    var beforeIds = beforeRows.map(function(row) {{ return row.id; }});
                    var model = null;
                    var inserter = null;
                    try {{
                        model = chart._chartWidget.model();
                        inserter = model.createStudyInserter.call(model, {{ type: "java", studyId: meta.id }}, []);
                        if (!inserter || typeof inserter.insert !== "function" || typeof inserter.setForceOverlay !== "function") {{
                            return failure("inserter_unavailable", false, null);
                        }}
                        inserter.setForceOverlay.call(inserter, meta.is_price_study);
                    }} catch(e) {{
                        return failure("inserter_unavailable", false, null);
                    }}

                    var insertion = null;
                    try {{
                        insertion = inserter.insert.call(inserter, function() {{
                            return Promise.resolve({{ inputs: overrides, parentSources: [] }});
                        }});
                    }} catch(e) {{
                        return failure("insert_rejected", true, null, studyRows(chart).filter(function(row) {{ return beforeIds.indexOf(row.id) === -1; }}));
                    }}
                    if (!insertion || typeof insertion.then !== "function") {{
                        return failure("insert_not_awaitable", true, null, studyRows(chart).filter(function(row) {{ return beforeIds.indexOf(row.id) === -1; }}));
                    }}
                    var timeoutToken = {{}};
                    try {{
                        await Promise.race([
                            insertion,
                            new Promise(function(_, reject) {{ setTimeout(function() {{ reject(timeoutToken); }}, 8000); }})
                        ]);
                    }} catch(e) {{
                        if (e === timeoutToken) return failure("insert_timeout", true, null);
                        return failure("insert_rejected", true, null, studyRows(chart).filter(function(row) {{ return beforeIds.indexOf(row.id) === -1; }}));
                    }}

                    var afterRows = studyRows(chart);
                    var added = afterRows.filter(function(row) {{ return beforeIds.indexOf(row.id) === -1; }});
                    if (added.length !== 1) {{
                        return failure("study_count_mismatch", true, {{ new_study_count: added.length, awaited: true }}, added);
                    }}
                    var row = added[0];
                    var nameVerified = typeof row.name === "string" && row.name.trim() === requestedName;
                    var instance = null;
                    var values = null;
                    try {{ instance = chart.getStudyById(row.id); }} catch(e) {{}}
                    try {{ if (instance && typeof instance.getInputValues === "function") values = instance.getInputValues(); }} catch(e) {{}}
                    var inputsVerified = Array.isArray(values) && requestedKeys.every(function(key) {{
                        var matches = values.filter(function(input) {{ return input && input.id === key; }});
                        return matches.length === 1 && matches[0].value === overrides[key];
                    }});
                    if (!nameVerified || !inputsVerified) {{
                        return failure("readback_mismatch", true, {{
                            new_study_count: 1,
                            awaited: true,
                            name_verified: nameVerified,
                            inputs_verified: inputsVerified
                        }}, added);
                    }}
                    return {{
                        action: "add",
                        indicator: {indicator_literal},
                        success: true,
                        mutation_performed: true,
                        entity_id: row.id,
                        new_study_count: 1,
                        before_count: beforeRows.length,
                        after_count: afterRows.length,
                        input_count: requestedKeys.length,
                        awaited: true,
                        name_verified: true,
                        inputs_verified: true
                    }};
                }})()
                "#
            ),
            true,
        )
        .await
        .map_err(|error| {
            AppError::new(error.kind, "Indicator insertion evaluation failed").with_details(
                serde_json::json!({
                    "requested_indicator": indicator,
                    "next_action_hint": "Confirm the selected TradingView chart is ready and retry once."
                }),
            )
        })?;

    let entity_id = data.get("entity_id").and_then(Value::as_str);
    let before_count = data.get("before_count").and_then(Value::as_u64);
    let after_count = data.get("after_count").and_then(Value::as_u64);
    let input_count = data.get("input_count").and_then(Value::as_u64);
    let valid_success = data.get("success").and_then(Value::as_bool) == Some(true)
        && data.get("action").and_then(Value::as_str) == Some("add")
        && data.get("indicator").and_then(Value::as_str) == Some(indicator)
        && entity_id.is_some_and(|id| !id.is_empty())
        && data.get("new_study_count").and_then(Value::as_u64) == Some(1)
        && before_count
            .zip(after_count)
            .is_some_and(|(before, after)| before.checked_add(1) == Some(after))
        && input_count == Some(expected_input_count)
        && data.get("mutation_performed").and_then(Value::as_bool) == Some(true)
        && data.get("awaited").and_then(Value::as_bool) == Some(true)
        && data.get("name_verified").and_then(Value::as_bool) == Some(true)
        && data.get("inputs_verified").and_then(Value::as_bool) == Some(true)
        && data.get("error_stage").is_none()
        && data.get("cleanup_candidate_count").is_none()
        && data.get("cleanup_removed").is_none()
        && data.get("cleanup_absent").is_none()
        && data.get("candidate_count").is_none()
        && data.get("unmatched_input_count").is_none()
        && data.get("error").is_none();
    if !valid_success {
        let kind = if data.get("mutation_performed").and_then(Value::as_bool) == Some(false)
            && matches!(
                data.get("error_stage").and_then(Value::as_str),
                Some("indicator_resolution" | "input_validation")
            ) {
            ErrorKind::Validation
        } else {
            ErrorKind::InternalApiUnavailable
        };
        return Err(AppError::new(
            kind,
            format!("Indicator add could not verify the requested study: {indicator}"),
        )
        .with_details(indicator_add_error_details(&data, indicator)));
    }

    Ok(serde_json::json!({
        "action": "add",
        "indicator": indicator,
        "entity_id": entity_id.expect("validated entity id"),
        "new_study_count": 1,
        "before_count": before_count.expect("validated before count"),
        "after_count": after_count.expect("validated after count"),
        "input_count": input_count.expect("validated input count"),
        "awaited": true,
        "name_verified": true,
        "inputs_verified": true
    }))
}

fn indicator_add_error_details(data: &Value, indicator: &str) -> Value {
    let mut details = Map::new();
    details.insert(
        "requested_indicator".to_string(),
        Value::String(indicator.to_string()),
    );
    const STAGES: &[&str] = &[
        "metainfo_unavailable",
        "indicator_resolution",
        "input_validation",
        "inserter_unavailable",
        "insert_rejected",
        "insert_not_awaitable",
        "insert_timeout",
        "study_count_mismatch",
        "readback_mismatch",
    ];
    let stage = data
        .get("error_stage")
        .and_then(Value::as_str)
        .filter(|stage| STAGES.contains(stage));
    details.insert(
        "error_stage".to_string(),
        Value::String(stage.unwrap_or("malformed_runtime_outcome").to_string()),
    );
    for key in [
        "candidate_count",
        "unmatched_input_count",
        "new_study_count",
        "cleanup_candidate_count",
    ] {
        if let Some(value) = data.get(key).and_then(Value::as_u64) {
            details.insert(key.to_string(), Value::from(value));
        }
    }
    for key in [
        "mutation_performed",
        "awaited",
        "name_verified",
        "inputs_verified",
        "cleanup_removed",
        "cleanup_absent",
    ] {
        if let Some(value) = data.get(key).and_then(Value::as_bool) {
            details.insert(key.to_string(), Value::Bool(value));
        }
    }
    details.insert(
        "next_action_hint".to_string(),
        Value::String(
            "Confirm the selected chart, exact indicator name, and input ids before retrying."
                .to_string(),
        ),
    );
    Value::Object(details)
}

pub async fn indicator_remove(
    runtime: &mut impl RuntimeEvaluator,
    entity_id: &str,
) -> Result<Value, AppError> {
    let entity_id_literal = js_string(entity_id)?;
    let data = runtime
        .evaluate(
            &format!(
                r#"
                (async function() {{
                    function sleep(ms) {{
                        return new Promise(function(resolve) {{ setTimeout(resolve, ms); }});
                    }}
                    function getStudyByIdSafe(chart, entityId) {{
                        try {{ return chart.getStudyById(entityId); }} catch(e) {{ return null; }}
                    }}
                    function studyName(study) {{
                        try {{
                            if (typeof study.name === "function") return study.name();
                            if (typeof study.title === "function") return study.title();
                            return study.name || study.title || null;
                        }} catch(e) {{
                            return null;
                        }}
                    }}

                    var chart = {CHART_API};
                    var entityId = {entity_id_literal};
                    var beforeStudies = chart.getAllStudies();
                    var beforeStudy = getStudyByIdSafe(chart, entityId);
                    if (!beforeStudy) return {{ error: "Study not found: " + entityId }};
                    var name = studyName(beforeStudy);
                    chart.removeEntity(entityId);
                    await sleep(500);
                    var afterStudy = getStudyByIdSafe(chart, entityId);
                    var afterStudies = chart.getAllStudies();
                    return {{
                        action: "remove",
                        entity_id: entityId,
                        indicator: name,
                        removed: !afterStudy,
                        before_count: beforeStudies.length,
                        after_count: afterStudies.length
                    }};
                }})()
                "#
            ),
            true,
        )
        .await?;

    if let Some(message) = data.get("error").and_then(Value::as_str) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            message.to_string(),
        ));
    }
    if data.get("removed").and_then(Value::as_bool) != Some(true) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!("Indicator remove did not remove study: {entity_id}"),
        )
        .with_details(data));
    }

    Ok(data)
}

pub async fn indicator_toggle(
    runtime: &mut impl RuntimeEvaluator,
    entity_id: &str,
    visible: bool,
) -> Result<Value, AppError> {
    let entity_id_literal = js_string(entity_id)?;
    let data = runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    function getStudyByIdSafe(chart, entityId) {{
                        try {{ return chart.getStudyById(entityId); }} catch(e) {{ return null; }}
                    }}

                    var chart = {CHART_API};
                    var entityId = {entity_id_literal};
                    var study = getStudyByIdSafe(chart, entityId);
                    if (!study) return {{ error: "Study not found: " + entityId }};
                    var previousVisible = null;
                    try {{ previousVisible = study.isVisible(); }} catch(e) {{}}
                    study.setVisible({visible});
                    var actualVisible = study.isVisible();
                    return {{
                        action: "toggle",
                        entity_id: entityId,
                        requested_visible: {visible},
                        previous_visible: previousVisible,
                        visible: actualVisible
                    }};
                }})()
                "#
            ),
            false,
        )
        .await?;

    if let Some(message) = data.get("error").and_then(Value::as_str) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            message.to_string(),
        ));
    }
    if data.get("visible").and_then(Value::as_bool) != Some(visible) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!("Indicator visibility did not change for study: {entity_id}"),
        )
        .with_details(data));
    }

    Ok(data)
}

pub async fn indicator_set(
    runtime: &mut impl RuntimeEvaluator,
    entity_id: &str,
    inputs: &Value,
) -> Result<Value, AppError> {
    require_non_empty_object(inputs)?;
    let entity_id_literal = js_string(entity_id)?;
    let inputs_json = serde_json::to_string(inputs)
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))?;

    let data = runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    function getStudyByIdSafe(chart, entityId) {{
                        try {{ return chart.getStudyById(entityId); }} catch(e) {{ return null; }}
                    }}

                    var chart = {CHART_API};
                    var entityId = {entity_id_literal};
                    var study = getStudyByIdSafe(chart, entityId);
                    if (!study) return {{ error: "Study not found: " + entityId }};
                    var currentInputs = study.getInputValues();
                    var overrides = {inputs_json};
                    var updatedInputs = {{}};
                    var unmatchedInputs = Object.assign({{}}, overrides);

                    for (var i = 0; i < currentInputs.length; i++) {{
                        var input = currentInputs[i];
                        if (input && Object.prototype.hasOwnProperty.call(overrides, input.id)) {{
                            input.value = overrides[input.id];
                            updatedInputs[input.id] = overrides[input.id];
                            delete unmatchedInputs[input.id];
                        }}
                    }}

                    if (Object.keys(updatedInputs).length === 0) {{
                        return {{
                            error: "No matching input ids found for study: " + entityId,
                            updated_inputs: updatedInputs,
                            unmatched_inputs: unmatchedInputs,
                            available_inputs: currentInputs.map(function(input) {{ return input && input.id; }}).filter(Boolean)
                        }};
                    }}

                    study.setInputValues(currentInputs);
                    return {{
                        action: "set",
                        entity_id: entityId,
                        updated_inputs: updatedInputs,
                        unmatched_inputs: unmatchedInputs
                    }};
                }})()
                "#
            ),
            false,
        )
        .await?;

    if let Some(message) = data.get("error").and_then(Value::as_str) {
        return Err(AppError::new(ErrorKind::Validation, message.to_string()).with_details(data));
    }

    Ok(data)
}

fn require_non_empty_object(value: &Value) -> Result<&Map<String, Value>, AppError> {
    let Some(object) = value.as_object() else {
        return Err(AppError::new(
            ErrorKind::Validation,
            "--inputs must be a JSON object",
        ));
    };
    if object.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "--inputs must not be empty",
        ));
    }
    Ok(object)
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use serde_json::json;

    use super::super::test_support::FakeRuntime;
    use super::*;
    use tradingview_core::ErrorKind;

    #[test]
    fn parse_indicator_inputs_requires_non_empty_object() {
        assert!(parse_indicator_inputs(r#"{"length":20}"#).is_ok());

        let err = parse_indicator_inputs("[]").unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);

        let err = parse_indicator_inputs("{}").unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);

        let err = parse_indicator_inputs("{").unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);
    }

    #[test]
    fn parse_indicator_add_inputs_rejects_composite_values() {
        assert!(parse_indicator_add_inputs(r#"{"length":21,"enabled":true}"#).is_ok());
        assert!(parse_indicator_add_inputs(r#"{"__proto__":null}"#).is_ok());
        assert!(
            parse_indicator_add_inputs(
                r#"{"positive":9007199254740991,"negative":-9007199254740991}"#
            )
            .is_ok()
        );

        for raw in [r#"{"levels":[1,2]}"#, r#"{"style":{"color":"red"}}"#] {
            let err = parse_indicator_add_inputs(raw).unwrap_err();
            assert_eq!(err.kind, ErrorKind::Validation);
        }

        for raw in [
            r#"{"length":9007199254740992}"#,
            r#"{"length":9007199254740993}"#,
            r#"{"length":-9007199254740992}"#,
            r#"{"length":1e20}"#,
        ] {
            let err = parse_indicator_add_inputs(raw).unwrap_err();
            assert_eq!(err.kind, ErrorKind::Validation);
            assert!(err.message.contains("losslessly representable"));
        }
    }

    #[tokio::test]
    async fn indicator_add_serializes_name_and_inputs() {
        let requested = "Volume'); window.bad = true; ('";
        let payload = json!({
            "action": "add",
            "indicator": requested,
            "success": true,
            "mutation_performed": true,
            "entity_id": "abc123",
            "new_study_count": 1,
            "before_count": 1,
            "after_count": 2,
            "input_count": 1,
            "awaited": true,
            "name_verified": true,
            "inputs_verified": true
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);
        let inputs = json!({"length": 20});

        let result = indicator_add(&mut runtime, requested, Some(&inputs))
            .await
            .unwrap();

        assert_eq!(result["action"], "add");
        assert_eq!(result["indicator"], requested);
        assert_eq!(result["entity_id"], "abc123");
        assert_eq!(result["before_count"], 1);
        assert_eq!(result["after_count"], 2);
        assert_eq!(result["input_count"], 1);
        assert!(runtime.evaluated[0].0.contains("createStudyInserter.call"));
        assert!(!runtime.evaluated[0].0.contains("chart.createStudy("));
        assert!(
            runtime.evaluated[0]
                .0
                .contains("\"Volume'); window.bad = true; ('\"")
        );
        assert!(runtime.evaluated[0].0.contains("JSON.parse"));
        assert!(runtime.evaluated[0].0.contains(r#"\"length\":20"#));
        assert!(runtime.evaluated[0].1);
    }

    #[tokio::test]
    async fn indicator_add_requires_new_entity_id() {
        let mut runtime = FakeRuntime::new([json!({
            "action": "add",
            "indicator": "Missing",
            "entity_id": null,
            "new_study_count": 0,
            "mutation_performed": false,
            "error_stage": "indicator_resolution",
            "candidate_count": 0
        })]);

        let err = indicator_add(&mut runtime, "Missing", None)
            .await
            .unwrap_err();

        assert_eq!(err.kind, ErrorKind::Validation);
    }

    #[tokio::test]
    async fn indicator_add_rejects_unverified_or_multiple_studies() {
        for payload in [
            json!({
                "entity_id": "abc123",
                "new_study_count": 2,
                "awaited": true,
                "name_verified": true,
                "inputs_verified": true,
                "mutation_performed": true,
                "error_stage": "study_count_mismatch"
            }),
            json!({
                "entity_id": "abc123",
                "new_study_count": 1,
                "awaited": true,
                "name_verified": false,
                "inputs_verified": true,
                "mutation_performed": true,
                "error_stage": "readback_mismatch"
            }),
        ] {
            let mut runtime = FakeRuntime::new([payload]);
            let err = indicator_add(&mut runtime, "Volume", Some(&json!({"length": 21})))
                .await
                .unwrap_err();
            assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
        }

        let mut runtime = FakeRuntime::new([json!({
            "action": "add",
            "indicator": "Volume",
            "success": true,
            "mutation_performed": true,
            "entity_id": "abc123",
            "new_study_count": 1,
            "before_count": 1,
            "after_count": 2,
            "input_count": 0,
            "awaited": true,
            "name_verified": true,
            "inputs_verified": true
        })]);
        let err = indicator_add(&mut runtime, "Volume", Some(&json!({"length": 21})))
            .await
            .unwrap_err();
        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);

        for marker in [
            json!({"candidate_count": 0}),
            json!({"unmatched_input_count": 1}),
            json!({"error": "failure-only"}),
        ] {
            let mut payload = json!({
                "action": "add",
                "indicator": "Volume",
                "success": true,
                "mutation_performed": true,
                "entity_id": "abc123",
                "new_study_count": 1,
                "before_count": 1,
                "after_count": 2,
                "input_count": 0,
                "awaited": true,
                "name_verified": true,
                "inputs_verified": true
            });
            payload
                .as_object_mut()
                .expect("fixture payload is an object")
                .extend(marker.as_object().expect("marker is an object").clone());
            let mut runtime = FakeRuntime::new([payload]);
            let err = indicator_add(&mut runtime, "Volume", None)
                .await
                .unwrap_err();
            assert_eq!(
                err.details.unwrap()["error_stage"],
                "malformed_runtime_outcome"
            );
        }
    }

    #[tokio::test]
    async fn indicator_add_sanitizes_runtime_and_payload_failures() {
        let private = "private-runtime-value";
        let mut runtime = FakeRuntime::new([]).with_evaluate_app_error(
            AppError::new(ErrorKind::InternalApiUnavailable, private)
                .with_details(json!({"raw": private})),
        );
        let err = indicator_add(&mut runtime, "Volume", None)
            .await
            .unwrap_err();
        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
        assert!(!err.message.contains(private));
        assert!(!err.details.unwrap().to_string().contains(private));

        let mut runtime = FakeRuntime::new([json!({
            "error_stage": "readback_mismatch",
            "mutation_performed": true,
            "new_study_count": 1,
            "awaited": true,
            "name_verified": false,
            "inputs_verified": true,
            "raw": private,
            "script_id": private
        })]);
        let err = indicator_add(&mut runtime, "Volume", None)
            .await
            .unwrap_err();
        assert!(!err.details.unwrap().to_string().contains(private));

        for payload in [
            json!({
                "success": false,
                "mutation_performed": true,
                "action": "remove",
                "indicator": "Other",
                "entity_id": "abc123",
                "new_study_count": 1,
                "before_count": {"raw": private},
                "after_count": 2,
                "input_count": 0,
                "awaited": true,
                "name_verified": true,
                "inputs_verified": true
            }),
            json!({
                "error_stage": {"raw": private},
                "candidate_count": {"raw": private},
                "mutation_performed": {"raw": private}
            }),
        ] {
            let mut runtime = FakeRuntime::new([payload]);
            let err = indicator_add(&mut runtime, "Volume", None)
                .await
                .unwrap_err();
            let details = err.details.unwrap();
            assert_eq!(details["error_stage"], "malformed_runtime_outcome");
            assert!(!details.to_string().contains(private));
        }
    }

    fn execute_indicator_add_expression(expression: &str, options: &str) -> Value {
        let script = format!(
            r#"
const options = {options};
let studies = [];
let instances = {{}};
let inventoryReads = 0;
let factoryCalls = 0;
let configureCalls = 0;
let insertCalls = 0;
let removeCalls = 0;
const chart = {{
  getAllStudies: function() {{
    inventoryReads++;
    if (options.shrinkOnThirdInventory && inventoryReads === 3) studies = studies.slice(0, 1);
    return studies.slice();
  }},
  getStudyById: function(id) {{ return instances[id] || null; }},
  removeEntity: function(id) {{ removeCalls++; studies = studies.filter(function(row) {{ return row.id !== id; }}); delete instances[id]; }},
  studyMetaIntoRepository: function() {{
    return {{ getInternalMetaInfoArray: function() {{
      return options.zeroMeta ? [] : options.duplicateMeta ? [meta, meta] : [meta];
    }} }};
  }},
  _chartWidget: {{ model: function() {{ return model; }} }}
}};
const meta = {{ description: 'Volume', id: 'volume-meta', is_price_study: false, inputs: [{{ id: 'length' }}] }};
const model = {{
  createStudyInserter: function() {{
    factoryCalls++;
    return {{
      setForceOverlay: function() {{ configureCalls++; }},
      insert: function(provider) {{
        insertCalls++;
        if (options.nonThenable) return {{}};
        if (options.rejectInsert) return Promise.reject(new Error('private rejection'));
        if (options.timeout) return new Promise(function() {{}});
        return Promise.resolve(provider()).then(function(payload) {{
          if (options.zeroStudy) return;
          const count = options.duplicateStudy ? 2 : 1;
          for (let i = 0; i < count; i++) {{
            const id = 'study-' + i;
            studies.push({{ id: id, name: options.nameMismatch ? 'Other' : 'Volume' }});
            instances[id] = {{ getInputValues: function() {{
              return [{{ id: 'length', value: options.inputMismatch ? 20 : payload.inputs.length }}];
            }} }};
          }}
        }});
      }}
    }};
  }}
}};
global.window = global;
window.TradingViewApi = {{ _activeChartWidgetWV: {{ value: function() {{ return chart; }} }} }};
if (options.timeout) global.setTimeout = function(callback) {{ Promise.resolve().then(callback); return 1; }};
Promise.resolve({expression}).then(function(result) {{
  process.stdout.write(JSON.stringify({{
    result: result,
    observations: {{ inventoryReads, factoryCalls, configureCalls, insertCalls, removeCalls }}
  }}));
  process.exit(0);
}}).catch(function(error) {{
  process.stderr.write(String(error && error.stack || error));
  process.exit(1);
}});
"#
        );
        let output = Command::new("node")
            .args(["-e", &script])
            .output()
            .expect("Node.js is required to execute the indicator insertion fixture");
        assert!(
            output.status.success(),
            "indicator insertion fixture failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout)
            .expect("indicator insertion fixture should return JSON")
    }

    #[tokio::test]
    #[ignore = "run through scripts/check-indicator-insertion-js-contract.py with pinned Node.js"]
    async fn javascript_indicator_add_contract_verifies_immediate_name_and_inputs() {
        let payload = json!({
            "action": "add",
            "indicator": "Volume",
            "success": true,
            "mutation_performed": true,
            "entity_id": "fixture",
            "new_study_count": 1,
            "before_count": 0,
            "after_count": 1,
            "input_count": 1,
            "awaited": true,
            "name_verified": true,
            "inputs_verified": true
        });
        let mut runtime = FakeRuntime::new([payload]);
        indicator_add(&mut runtime, "Volume", Some(&json!({"length": 21})))
            .await
            .unwrap();
        let expression = &runtime.evaluated[0].0;

        let success_run = execute_indicator_add_expression(expression, "{}");
        let success = &success_run["result"];
        assert_eq!(success["new_study_count"], 1);
        assert_eq!(success["awaited"], true);
        assert_eq!(success["name_verified"], true);
        assert_eq!(success["inputs_verified"], true);
        assert_eq!(success_run["observations"]["factoryCalls"], 1);
        assert_eq!(success_run["observations"]["configureCalls"], 1);
        assert_eq!(success_run["observations"]["insertCalls"], 1);

        for options in [
            "{ nameMismatch: true }",
            "{ inputMismatch: true }",
            "{ duplicateStudy: true }",
        ] {
            let run = execute_indicator_add_expression(expression, options);
            let failure = &run["result"];
            assert_ne!(failure.get("error_stage"), None);
            assert_ne!(
                failure.get("entity_id"),
                Some(&Value::String("study-0".into()))
            );
            assert_eq!(run["observations"]["factoryCalls"], 1);
            assert_eq!(run["observations"]["configureCalls"], 1);
            assert_eq!(run["observations"]["insertCalls"], 1);
        }

        let name_run = execute_indicator_add_expression(expression, "{ nameMismatch: true }");
        let name_failure = &name_run["result"];
        assert_eq!(name_failure["error_stage"], "readback_mismatch");
        assert_eq!(name_failure["cleanup_removed"], true);
        assert_eq!(name_failure["cleanup_absent"], true);

        let duplicate_run = execute_indicator_add_expression(
            expression,
            "{ duplicateStudy: true, shrinkOnThirdInventory: true }",
        );
        let duplicate = &duplicate_run["result"];
        assert_eq!(duplicate["error_stage"], "study_count_mismatch");
        assert_eq!(duplicate["cleanup_candidate_count"], 2);
        assert_eq!(duplicate["cleanup_removed"], false);
        assert_eq!(duplicate_run["observations"]["inventoryReads"], 2);
        assert_eq!(duplicate_run["observations"]["removeCalls"], 0);
        assert_eq!(duplicate_run["observations"]["factoryCalls"], 1);
        assert_eq!(duplicate_run["observations"]["configureCalls"], 1);
        assert_eq!(duplicate_run["observations"]["insertCalls"], 1);

        for (options, stage, expected_reads, expected_removes, expected_calls) in [
            ("{ zeroMeta: true }", "indicator_resolution", 0, 0, 0),
            ("{ duplicateMeta: true }", "indicator_resolution", 0, 0, 0),
            ("{ nonThenable: true }", "insert_not_awaitable", 2, 0, 1),
            ("{ rejectInsert: true }", "insert_rejected", 2, 0, 1),
            ("{ timeout: true }", "insert_timeout", 1, 0, 1),
            ("{ zeroStudy: true }", "study_count_mismatch", 2, 0, 1),
        ] {
            let run = execute_indicator_add_expression(expression, options);
            assert_eq!(run["result"]["error_stage"], stage);
            assert_eq!(run["observations"]["inventoryReads"], expected_reads);
            assert_eq!(run["observations"]["removeCalls"], expected_removes);
            assert_eq!(run["observations"]["factoryCalls"], expected_calls);
            assert_eq!(run["observations"]["configureCalls"], expected_calls);
            assert_eq!(run["observations"]["insertCalls"], expected_calls);
        }

        for key in ["__proto__", "constructor", "toString"] {
            let special_inputs = json!({(key): null});
            let special_payload = json!({
                "action": "add",
                "indicator": "Volume",
                "success": true,
                "mutation_performed": true,
                "entity_id": "fixture",
                "new_study_count": 1,
                "before_count": 0,
                "after_count": 1,
                "input_count": 1,
                "awaited": true,
                "name_verified": true,
                "inputs_verified": true
            });
            let mut runtime = FakeRuntime::new([special_payload]);
            indicator_add(&mut runtime, "Volume", Some(&special_inputs))
                .await
                .unwrap();
            let run = execute_indicator_add_expression(&runtime.evaluated[0].0, "{}");
            assert_eq!(run["result"]["error_stage"], "input_validation");
            assert_eq!(run["result"]["unmatched_input_count"], 1);
            assert_eq!(run["observations"]["factoryCalls"], 0);
            assert_eq!(run["observations"]["insertCalls"], 0);
        }
    }

    #[tokio::test]
    async fn indicator_remove_maps_missing_study_to_internal_api_error() {
        let mut runtime = FakeRuntime::new([json!({"error": "Study not found: missing"})]);

        let err = indicator_remove(&mut runtime, "missing").await.unwrap_err();

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
    }

    #[tokio::test]
    async fn indicator_remove_requires_post_delete_absence() {
        let mut runtime = FakeRuntime::new([json!({
            "action": "remove",
            "entity_id": "abc123",
            "removed": false
        })]);

        let err = indicator_remove(&mut runtime, "abc123").await.unwrap_err();

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
    }

    #[tokio::test]
    async fn indicator_toggle_returns_observed_visibility() {
        let payload = json!({
            "action": "toggle",
            "entity_id": "abc123",
            "requested_visible": false,
            "previous_visible": true,
            "visible": false
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);

        let result = indicator_toggle(&mut runtime, "abc123", false)
            .await
            .unwrap();

        assert_eq!(result, payload);
        assert!(runtime.evaluated[0].0.contains("setVisible(false)"));
    }

    #[tokio::test]
    async fn indicator_set_returns_updated_and_unmatched_inputs() {
        let payload = json!({
            "action": "set",
            "entity_id": "abc123",
            "updated_inputs": {"length": 20},
            "unmatched_inputs": {"unknown": 1}
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);
        let inputs = json!({"length": 20, "unknown": 1});

        let result = indicator_set(&mut runtime, "abc123", &inputs)
            .await
            .unwrap();

        assert_eq!(result, payload);
        assert!(runtime.evaluated[0].0.contains("setInputValues"));
        assert!(runtime.evaluated[0].0.contains("\"length\":20"));
    }

    #[tokio::test]
    async fn indicator_set_errors_when_no_input_ids_match() {
        let mut runtime = FakeRuntime::new([json!({
            "error": "No matching input ids found for study: abc123",
            "updated_inputs": {},
            "unmatched_inputs": {"unknown": 1},
            "available_inputs": ["length"]
        })]);
        let inputs = json!({"unknown": 1});

        let err = indicator_set(&mut runtime, "abc123", &inputs)
            .await
            .unwrap_err();

        assert_eq!(err.kind, ErrorKind::Validation);
        assert!(err.details.is_some());
    }
}
