use serde_json::Value;

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};

use super::super::common::{CHART_API, js_string};

pub async fn drawing_list(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    function shapeName(shape) {{
                        try {{
                            if (typeof shape.name === "function") return shape.name();
                            if (typeof shape.title === "function") return shape.title();
                            return shape.name || shape.title || null;
                        }} catch(e) {{
                            return null;
                        }}
                    }}

                    var api = {CHART_API};
                    var all = api.getAllShapes();
                    var shapes = all.map(function(shape) {{
                        return {{
                            id: shape.id,
                            name: shapeName(shape)
                        }};
                    }});
                    return {{
                        count: shapes.length,
                        shapes: shapes
                    }};
                }})()
                "#
            ),
            false,
        )
        .await
}

pub async fn drawing_get(
    runtime: &mut impl RuntimeEvaluator,
    entity_id: &str,
) -> Result<Value, AppError> {
    let entity_id_literal = js_string(entity_id)?;
    let data = runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    function getShapeByIdSafe(api, entityId) {{
                        try {{ return api.getShapeById(entityId); }} catch(e) {{ return null; }}
                    }}
                    function shapeName(shape) {{
                        try {{
                            if (typeof shape.name === "function") return shape.name();
                            if (typeof shape.title === "function") return shape.title();
                            return shape.name || shape.title || null;
                        }} catch(e) {{
                            return null;
                        }}
                    }}

                    var api = {CHART_API};
                    var entityId = {entity_id_literal};
                    var shape = getShapeByIdSafe(api, entityId);
                    if (!shape) return {{ error: "Shape not found: " + entityId }};
                    var props = {{ entity_id: entityId, name: shapeName(shape) }};
                    var methods = [];
                    try {{
                        for (var key in shape) {{
                            if (typeof shape[key] === "function") methods.push(key);
                        }}
                        props.available_methods = methods;
                    }} catch(e) {{}}
                    try {{ props.points = shape.getPoints(); }} catch(e) {{ props.points_error = e.message; }}
                    try {{ props.properties = shape.getProperties(); }} catch(e) {{
                        try {{ props.properties = shape.properties(); }} catch(e2) {{ props.properties_error = e2.message; }}
                    }}
                    try {{ props.visible = shape.isVisible(); }} catch(e) {{}}
                    try {{ props.locked = shape.isLocked(); }} catch(e) {{}}
                    try {{ props.selectable = shape.isSelectionEnabled(); }} catch(e) {{}}
                    return props;
                }})()
                "#
            ),
            false,
        )
        .await?;

    if let Some(message) = data.get("error").and_then(Value::as_str) {
        return Err(AppError::new(ErrorKind::Validation, message.to_string()));
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::ops::test_support::FakeRuntime;

    use super::*;
    use tradingview_core::ErrorKind;

    #[tokio::test]
    async fn drawing_list_returns_runtime_payload() {
        let payload =
            json!({"count": 1, "shapes": [{"id": "shape123", "name": "Horizontal Line"}]});
        let mut runtime = FakeRuntime::new([payload.clone()]);

        let result = drawing_list(&mut runtime).await.unwrap();

        assert_eq!(result, payload);
        assert!(runtime.evaluated[0].0.contains("getAllShapes"));
    }

    #[tokio::test]
    async fn drawing_get_maps_missing_shape_to_validation() {
        let mut runtime = FakeRuntime::new([json!({"error": "Shape not found: missing"})]);

        let err = drawing_get(&mut runtime, "missing").await.unwrap_err();

        assert_eq!(err.kind, ErrorKind::Validation);
    }
}
