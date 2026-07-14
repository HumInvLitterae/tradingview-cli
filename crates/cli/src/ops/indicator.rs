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
                    function cleanup(chart, beforeIds) {{
                        var rows = studyRows(chart).filter(function(row) {{ return beforeIds.indexOf(row.id) === -1; }});
                        var result = {{ cleanup_candidate_count: rows.length, cleanup_removed: false, cleanup_absent: false }};
                        if (rows.length !== 1) return result;
                        try {{
                            chart.removeEntity(rows[0].id);
                            result.cleanup_removed = true;
                            result.cleanup_absent = studyRows(chart).every(function(row) {{ return row.id !== rows[0].id; }});
                        }} catch(e) {{}}
                        return result;
                    }}
                    function failure(stage, beforeIds, mutationPerformed, extra, skipCleanup) {{
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
                        if (mutationPerformed && !skipCleanup) Object.assign(result, cleanup(chart, beforeIds));
                        return result;
                    }}

                    var chart = {CHART_API};
                    var overrides = {inputs_json};
                    var requestedName = String({indicator_literal}).trim();
                    var repository = null;
                    var candidates = [];
                    try {{
                        repository = chart.studyMetaIntoRepository().getInternalMetaInfoArray();
                        candidates = repository.filter(function(meta) {{
                            return meta && typeof meta.description === "string" && meta.description.trim() === requestedName;
                        }});
                    }} catch(e) {{
                        return failure("metainfo_unavailable", [], false, {{ candidate_count: 0 }});
                    }}
                    if (candidates.length !== 1) {{
                        return failure("indicator_resolution", [], false, {{ candidate_count: candidates.length }});
                    }}
                    var meta = candidates[0];
                    if (typeof meta.id !== "string" || !meta.id || typeof meta.is_price_study !== "boolean" || !Array.isArray(meta.inputs)) {{
                        return failure("metainfo_unavailable", [], false, {{ candidate_count: 1 }});
                    }}
                    var definitions = {{}};
                    meta.inputs.forEach(function(input) {{ if (input && typeof input.id === "string") definitions[input.id] = true; }});
                    var requestedKeys = Object.keys(overrides);
                    var unmatched = requestedKeys.filter(function(key) {{ return !definitions[key]; }});
                    if (unmatched.length > 0) {{
                        return failure("input_validation", [], false, {{ unmatched_input_count: unmatched.length }});
                    }}

                    var beforeRows = studyRows(chart);
                    var beforeIds = beforeRows.map(function(row) {{ return row.id; }});
                    var model = null;
                    var inserter = null;
                    try {{
                        model = chart._chartWidget.model();
                        inserter = model.createStudyInserter.call(model, {{ type: "java", studyId: meta.id }}, []);
                        if (!inserter || typeof inserter.insert !== "function" || typeof inserter.setForceOverlay !== "function") {{
                            return failure("inserter_unavailable", beforeIds, false, null);
                        }}
                        inserter.setForceOverlay.call(inserter, meta.is_price_study);
                    }} catch(e) {{
                        return failure("inserter_unavailable", beforeIds, false, null);
                    }}

                    var insertion = null;
                    try {{
                        insertion = inserter.insert.call(inserter, function() {{
                            return Promise.resolve({{ inputs: overrides, parentSources: [] }});
                        }});
                    }} catch(e) {{
                        return failure("insert_rejected", beforeIds, true, null);
                    }}
                    if (!insertion || typeof insertion.then !== "function") {{
                        return failure("insert_not_awaitable", beforeIds, true, null);
                    }}
                    var timeoutToken = {{}};
                    try {{
                        await Promise.race([
                            insertion,
                            new Promise(function(_, reject) {{ setTimeout(function() {{ reject(timeoutToken); }}, 8000); }})
                        ]);
                    }} catch(e) {{
                        if (e === timeoutToken) return failure("insert_timeout", beforeIds, true, null, true);
                        return failure("insert_rejected", beforeIds, true, null, false);
                    }}

                    var afterRows = studyRows(chart);
                    var added = afterRows.filter(function(row) {{ return beforeIds.indexOf(row.id) === -1; }});
                    if (added.length !== 1) {{
                        return failure("study_count_mismatch", beforeIds, true, {{ new_study_count: added.length, awaited: true }});
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
                        return failure("readback_mismatch", beforeIds, true, {{
                            new_study_count: 1,
                            awaited: true,
                            name_verified: nameVerified,
                            inputs_verified: inputsVerified
                        }});
                    }}
                    return {{
                        action: "add",
                        indicator: {indicator_literal},
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

    if data.get("entity_id").and_then(Value::as_str).is_none()
        || data.get("new_study_count").and_then(Value::as_u64) != Some(1)
        || data.get("awaited").and_then(Value::as_bool) != Some(true)
        || data.get("name_verified").and_then(Value::as_bool) != Some(true)
        || data.get("inputs_verified").and_then(Value::as_bool) != Some(true)
    {
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
        "entity_id": data.get("entity_id").cloned().unwrap_or(Value::Null),
        "new_study_count": 1,
        "before_count": data.get("before_count").cloned().unwrap_or(Value::Null),
        "after_count": data.get("after_count").cloned().unwrap_or(Value::Null),
        "input_count": data.get("input_count").cloned().unwrap_or(Value::Null),
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
    for key in [
        "error_stage",
        "candidate_count",
        "unmatched_input_count",
        "mutation_performed",
        "new_study_count",
        "awaited",
        "name_verified",
        "inputs_verified",
        "cleanup_candidate_count",
        "cleanup_removed",
        "cleanup_absent",
    ] {
        if let Some(value) = data.get(key) {
            details.insert(key.to_string(), value.clone());
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

        for raw in [r#"{"levels":[1,2]}"#, r#"{"style":{"color":"red"}}"#] {
            let err = parse_indicator_add_inputs(raw).unwrap_err();
            assert_eq!(err.kind, ErrorKind::Validation);
        }
    }

    #[tokio::test]
    async fn indicator_add_serializes_name_and_inputs() {
        let requested = "Volume'); window.bad = true; ('";
        let payload = json!({
            "action": "add",
            "indicator": requested,
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

        assert_eq!(result, payload);
        assert!(runtime.evaluated[0].0.contains("createStudyInserter.call"));
        assert!(!runtime.evaluated[0].0.contains("chart.createStudy("));
        assert!(
            runtime.evaluated[0]
                .0
                .contains("\"Volume'); window.bad = true; ('\"")
        );
        assert!(runtime.evaluated[0].0.contains("\"length\":20"));
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
    }

    fn execute_indicator_add_expression(expression: &str, options: &str) -> Value {
        let script = format!(
            r#"
const options = {options};
let studies = [];
let instances = {{}};
const chart = {{
  getAllStudies: function() {{ return studies.slice(); }},
  getStudyById: function(id) {{ return instances[id] || null; }},
  removeEntity: function(id) {{ studies = studies.filter(function(row) {{ return row.id !== id; }}); delete instances[id]; }},
  studyMetaIntoRepository: function() {{
    return {{ getInternalMetaInfoArray: function() {{
      return options.duplicateMeta ? [meta, meta] : [meta];
    }} }};
  }},
  _chartWidget: {{ model: function() {{ return model; }} }}
}};
const meta = {{ description: 'Volume', id: 'volume-meta', is_price_study: false, inputs: [{{ id: 'length' }}] }};
const model = {{
  createStudyInserter: function() {{
    return {{
      setForceOverlay: function() {{}},
      insert: function(provider) {{
        if (options.rejectInsert) return Promise.reject(new Error('private rejection'));
        return Promise.resolve(provider()).then(function(payload) {{
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
Promise.resolve({expression}).then(function(result) {{
  process.stdout.write(JSON.stringify(result));
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
            "entity_id": "fixture",
            "new_study_count": 1,
            "awaited": true,
            "name_verified": true,
            "inputs_verified": true
        });
        let mut runtime = FakeRuntime::new([payload]);
        indicator_add(&mut runtime, "Volume", Some(&json!({"length": 21})))
            .await
            .unwrap();
        let expression = &runtime.evaluated[0].0;

        let success = execute_indicator_add_expression(expression, "{}");
        assert_eq!(success["new_study_count"], 1);
        assert_eq!(success["awaited"], true);
        assert_eq!(success["name_verified"], true);
        assert_eq!(success["inputs_verified"], true);

        for options in [
            "{ nameMismatch: true }",
            "{ inputMismatch: true }",
            "{ duplicateStudy: true }",
        ] {
            let failure = execute_indicator_add_expression(expression, options);
            assert_ne!(failure.get("error_stage"), None);
            assert_ne!(
                failure.get("entity_id"),
                Some(&Value::String("study-0".into()))
            );
        }

        let name_failure = execute_indicator_add_expression(expression, "{ nameMismatch: true }");
        assert_eq!(name_failure["error_stage"], "readback_mismatch");
        assert_eq!(name_failure["cleanup_removed"], true);
        assert_eq!(name_failure["cleanup_absent"], true);

        let duplicate = execute_indicator_add_expression(expression, "{ duplicateStudy: true }");
        assert_eq!(duplicate["error_stage"], "study_count_mismatch");
        assert_eq!(duplicate["cleanup_candidate_count"], 2);
        assert_eq!(duplicate["cleanup_removed"], false);
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
