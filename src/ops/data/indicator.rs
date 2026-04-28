use serde_json::Value;

use crate::cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};

use super::super::common::{CHART_API, js_string};

pub async fn study_values(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var chart = {CHART_API};
                    var sources = chart._chartWidget.model().model().dataSources();
                    var results = [];
                    for (var si = 0; si < sources.length; si++) {{
                        var s = sources[si];
                        if (!s.metaInfo) continue;
                        try {{
                            var meta = s.metaInfo();
                            var name = meta.description || meta.shortDescription || "";
                            if (!name) continue;
                            var values = {{}};
                            try {{
                                var dwv = s.dataWindowView();
                                if (dwv) {{
                                    var items = dwv.items();
                                    if (items) {{
                                        for (var i = 0; i < items.length; i++) {{
                                            var item = items[i];
                                            if (item._value && item._value !== "∅" && item._title) values[item._title] = item._value;
                                        }}
                                    }}
                                }}
                            }} catch(e) {{}}
                            if (Object.keys(values).length > 0) results.push({{ name: name, values: values }});
                        }} catch(e) {{}}
                    }}
                    return {{ study_count: results.length, studies: results }};
                }})()
                "#
            ),
            false,
        )
        .await
}

pub async fn data_indicator(
    runtime: &mut impl RuntimeEvaluator,
    entity_id: &str,
) -> Result<Value, AppError> {
    let entity_id_literal = js_string(entity_id)?;
    let data = runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var api = {CHART_API};
                    var entityId = {entity_id_literal};
                    var study = api.getStudyById(entityId);
                    if (!study) return {{ error: "Study not found: " + entityId }};
                    var result = {{ entity_id: entityId, visible: null, inputs: null }};
                    try {{ result.visible = study.isVisible(); }} catch(e) {{}}
                    try {{ result.inputs = study.getInputValues(); }} catch(e) {{ result.inputs_error = e.message; }}
                    if (Array.isArray(result.inputs)) {{
                        result.inputs = result.inputs.filter(function(input) {{
                            if (!input) return false;
                            if (input.id === "text" && typeof input.value === "string" && input.value.length > 200) return false;
                            if (typeof input.value === "string" && input.value.length > 500) return false;
                            return true;
                        }});
                    }}
                    return result;
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

    Ok(data)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::super::test_support::FakeRuntime;
    use super::*;
    use tradingview_core::ErrorKind;

    #[tokio::test]
    async fn study_values_returns_runtime_payload() {
        let payload = json!({
            "study_count": 1,
            "studies": [{"name": "Relative Strength", "values": {"RS": "98"}}]
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);

        let result = study_values(&mut runtime).await.unwrap();

        assert_eq!(result, payload);
        assert!(runtime.evaluated[0].0.contains("dataWindowView"));
    }

    #[tokio::test]
    async fn data_indicator_serializes_entity_id_and_filters_large_inputs() {
        let payload = json!({
            "entity_id": "eFu1Ot",
            "visible": true,
            "inputs": [{"id": "length", "value": 20}]
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);

        let result = data_indicator(&mut runtime, "eFu1Ot'); window.bad = true; ('")
            .await
            .unwrap();

        assert_eq!(result, payload);
        assert!(
            runtime.evaluated[0]
                .0
                .contains("\"eFu1Ot'); window.bad = true; ('\"")
        );
        assert!(runtime.evaluated[0].0.contains("getInputValues"));
    }

    #[tokio::test]
    async fn data_indicator_maps_missing_study_to_internal_api_error() {
        let mut runtime = FakeRuntime::new([json!({"error": "Study not found: missing"})]);

        let err = data_indicator(&mut runtime, "missing").await.unwrap_err();

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
    }
}
