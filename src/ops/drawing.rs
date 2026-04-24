use serde_json::Value;

use crate::{
    cdp::RuntimeEvaluator,
    error::{AppError, ErrorKind},
};

use super::common::{CHART_API, js_string, require_finite};

#[derive(Debug, Clone)]
pub struct DrawingPoint {
    pub time: f64,
    pub price: f64,
}

#[derive(Debug, Clone)]
pub struct DrawingShapeRequest {
    pub shape_type: String,
    pub point: DrawingPoint,
    pub point2: Option<DrawingPoint>,
    pub text: Option<String>,
    pub overrides: Option<Value>,
}

pub fn parse_drawing_overrides(raw: &str) -> Result<Value, AppError> {
    let value: Value = serde_json::from_str(raw).map_err(|err| {
        AppError::new(
            ErrorKind::Validation,
            format!("--overrides must be a JSON object: {err}"),
        )
    })?;

    if !value.is_object() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "--overrides must be a JSON object",
        ));
    }

    Ok(value)
}

pub async fn drawing_shape(
    runtime: &mut impl RuntimeEvaluator,
    request: DrawingShapeRequest,
) -> Result<Value, AppError> {
    require_finite(request.point.time, "time")?;
    require_finite(request.point.price, "price")?;
    if let Some(point2) = &request.point2 {
        require_finite(point2.time, "time2")?;
        require_finite(point2.price, "price2")?;
    }

    let shape_literal = js_string(&request.shape_type)?;
    let text_literal = js_string(request.text.as_deref().unwrap_or(""))?;
    let overrides_json = request
        .overrides
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))?
        .unwrap_or_else(|| "{}".to_string());

    let point2_expression = match request.point2 {
        Some(point2) => format!(
            "{{ time: {time}, price: {price} }}",
            time = point2.time,
            price = point2.price
        ),
        None => "null".to_string(),
    };

    let data = runtime
        .evaluate(
            &format!(
                r#"
                (async function() {{
                    function sleep(ms) {{
                        return new Promise(function(resolve) {{ setTimeout(resolve, ms); }});
                    }}
                    function shapeIds(api) {{
                        return api.getAllShapes().map(function(shape) {{ return shape.id; }});
                    }}

                    var api = {CHART_API};
                    var before = shapeIds(api);
                    var point = {{ time: {time}, price: {price} }};
                    var point2 = {point2_expression};
                    var options = {{
                        shape: {shape_literal},
                        overrides: {overrides_json},
                        text: {text_literal}
                    }};

                    if (point2) {{
                        api.createMultipointShape([point, point2], options);
                    }} else {{
                        api.createShape(point, options);
                    }}

                    await sleep(300);
                    var after = shapeIds(api);
                    var newIds = after.filter(function(id) {{ return before.indexOf(id) === -1; }});
                    return {{
                        action: "shape",
                        shape: {shape_literal},
                        entity_id: newIds.length > 0 ? newIds[0] : null,
                        new_shape_count: newIds.length,
                        before_count: before.length,
                        after_count: after.length,
                        point: point,
                        point2: point2,
                        text: {text_literal},
                        override_count: Object.keys({overrides_json}).length
                    }};
                }})()
                "#,
                time = request.point.time,
                price = request.point.price,
            ),
            true,
        )
        .await?;

    if data.get("entity_id").and_then(Value::as_str).is_none() {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!(
                "Drawing shape did not create a new entity: {}",
                request.shape_type
            ),
        )
        .with_details(data));
    }

    Ok(data)
}

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

pub async fn drawing_remove(
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

                    var api = {CHART_API};
                    var entityId = {entity_id_literal};
                    var before = api.getAllShapes();
                    var beforeShape = getShapeByIdSafe(api, entityId);
                    if (!beforeShape) {{
                        return {{
                            error: "Shape not found: " + entityId,
                            available: before.map(function(shape) {{ return shape.id; }})
                        }};
                    }}

                    api.removeEntity(entityId);
                    var after = api.getAllShapes();
                    var afterShape = getShapeByIdSafe(api, entityId);
                    return {{
                        action: "remove",
                        entity_id: entityId,
                        removed: !afterShape,
                        before_count: before.length,
                        remaining_shapes: after.length
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
    if data.get("removed").and_then(Value::as_bool) != Some(true) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!("Drawing remove did not remove entity: {entity_id}"),
        )
        .with_details(data));
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::test_support::FakeRuntime;
    use super::*;
    use crate::error::ErrorKind;

    #[test]
    fn parse_drawing_overrides_requires_json_object() {
        assert!(parse_drawing_overrides(r#"{"color":"red"}"#).is_ok());
        assert!(parse_drawing_overrides("{}").is_ok());

        let err = parse_drawing_overrides("[]").unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);

        let err = parse_drawing_overrides("{").unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);
    }

    #[tokio::test]
    async fn drawing_shape_serializes_user_inputs() {
        let payload = json!({
            "action": "shape",
            "shape": "horizontal_line",
            "entity_id": "shape123",
            "new_shape_count": 1,
            "before_count": 0,
            "after_count": 1
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);
        let request = DrawingShapeRequest {
            shape_type: "horizontal_line'); window.bad = true; ('".to_string(),
            point: DrawingPoint {
                time: 1700000000.0,
                price: 100.5,
            },
            point2: None,
            text: Some("hello'); window.bad = true; ('".to_string()),
            overrides: Some(json!({"linecolor": "red"})),
        };

        let result = drawing_shape(&mut runtime, request).await.unwrap();

        assert_eq!(result, payload);
        assert!(runtime.evaluated[0].0.contains("createShape"));
        assert!(
            runtime.evaluated[0]
                .0
                .contains("\"horizontal_line'); window.bad = true; ('\"")
        );
        assert!(
            runtime.evaluated[0]
                .0
                .contains("\"hello'); window.bad = true; ('\"")
        );
        assert!(runtime.evaluated[0].0.contains("\"linecolor\":\"red\""));
        assert!(runtime.evaluated[0].1);
    }

    #[tokio::test]
    async fn drawing_shape_supports_second_point() {
        let payload = json!({
            "action": "shape",
            "shape": "trend_line",
            "entity_id": "shape123",
            "new_shape_count": 1
        });
        let mut runtime = FakeRuntime::new([payload]);
        let request = DrawingShapeRequest {
            shape_type: "trend_line".to_string(),
            point: DrawingPoint {
                time: 1700000000.0,
                price: 100.0,
            },
            point2: Some(DrawingPoint {
                time: 1700000600.0,
                price: 101.0,
            }),
            text: None,
            overrides: None,
        };

        let result = drawing_shape(&mut runtime, request).await.unwrap();

        assert_eq!(result["entity_id"], "shape123");
        assert!(runtime.evaluated[0].0.contains("createMultipointShape"));
    }

    #[tokio::test]
    async fn drawing_shape_rejects_non_finite_values() {
        let mut runtime = FakeRuntime::new([]);
        let request = DrawingShapeRequest {
            shape_type: "horizontal_line".to_string(),
            point: DrawingPoint {
                time: f64::NAN,
                price: 100.0,
            },
            point2: None,
            text: None,
            overrides: None,
        };

        let err = drawing_shape(&mut runtime, request).await.unwrap_err();

        assert_eq!(err.kind, ErrorKind::Validation);
        assert!(runtime.evaluated.is_empty());
    }

    #[tokio::test]
    async fn drawing_shape_requires_new_entity_id() {
        let mut runtime = FakeRuntime::new([json!({
            "action": "shape",
            "shape": "horizontal_line",
            "entity_id": null,
            "new_shape_count": 0
        })]);
        let request = DrawingShapeRequest {
            shape_type: "horizontal_line".to_string(),
            point: DrawingPoint {
                time: 1700000000.0,
                price: 100.0,
            },
            point2: None,
            text: None,
            overrides: None,
        };

        let err = drawing_shape(&mut runtime, request).await.unwrap_err();

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
    }

    #[tokio::test]
    async fn drawing_list_returns_runtime_payload() {
        let payload = json!({
            "count": 1,
            "shapes": [{"id": "shape123", "name": "Horizontal Line"}]
        });
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

    #[tokio::test]
    async fn drawing_remove_requires_post_delete_absence() {
        let mut runtime = FakeRuntime::new([json!({
            "action": "remove",
            "entity_id": "shape123",
            "removed": false
        })]);

        let err = drawing_remove(&mut runtime, "shape123").await.unwrap_err();

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
    }

    #[tokio::test]
    async fn drawing_remove_returns_success_payload() {
        let payload = json!({
            "action": "remove",
            "entity_id": "shape123",
            "removed": true,
            "before_count": 2,
            "remaining_shapes": 1
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);

        let result = drawing_remove(&mut runtime, "shape123").await.unwrap();

        assert_eq!(result, payload);
        assert!(runtime.evaluated[0].0.contains("removeEntity"));
    }
}
