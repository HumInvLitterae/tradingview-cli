use serde_json::{Map, Value};

use crate::{
    cdp::RuntimeEvaluator,
    error::{AppError, ErrorKind},
};

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
                    function sleep(ms) {{
                        return new Promise(function(resolve) {{ setTimeout(resolve, ms); }});
                    }}
                    function studyIds(chart) {{
                        return chart.getAllStudies().map(function(study) {{ return study.id; }});
                    }}

                    var chart = {CHART_API};
                    var before = studyIds(chart);
                    var overrides = {inputs_json};
                    var inputArr = Object.keys(overrides).map(function(key) {{
                        return {{ id: key, value: overrides[key] }};
                    }});
                    chart.createStudy({indicator_literal}, false, false, inputArr);
                    await sleep(1500);
                    var after = studyIds(chart);
                    var newIds = after.filter(function(id) {{ return before.indexOf(id) === -1; }});
                    return {{
                        action: "add",
                        indicator: {indicator_literal},
                        entity_id: newIds.length > 0 ? newIds[0] : null,
                        new_study_count: newIds.length,
                        before_count: before.length,
                        after_count: after.length,
                        input_count: inputArr.length
                    }};
                }})()
                "#
            ),
            true,
        )
        .await?;

    if data.get("entity_id").and_then(Value::as_str).is_none() {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!("Indicator add did not create a new study: {indicator}"),
        )
        .with_details(data));
    }

    Ok(data)
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
    use serde_json::json;

    use super::super::test_support::FakeRuntime;
    use super::*;
    use crate::error::ErrorKind;

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

    #[tokio::test]
    async fn indicator_add_serializes_name_and_inputs() {
        let payload = json!({
            "action": "add",
            "indicator": "Volume",
            "entity_id": "abc123",
            "new_study_count": 1,
            "before_count": 1,
            "after_count": 2,
            "input_count": 1
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);
        let inputs = json!({"length": 20});

        let result = indicator_add(
            &mut runtime,
            "Volume'); window.bad = true; ('",
            Some(&inputs),
        )
        .await
        .unwrap();

        assert_eq!(result, payload);
        assert!(runtime.evaluated[0].0.contains("createStudy"));
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
            "new_study_count": 0
        })]);

        let err = indicator_add(&mut runtime, "Missing", None)
            .await
            .unwrap_err();

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
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
