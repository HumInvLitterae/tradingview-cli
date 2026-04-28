use serde_json::Value;

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionDirection {
    Long,
    Short,
}

impl PositionDirection {
    pub fn parse(value: &str) -> Result<Self, AppError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "long" => Ok(Self::Long),
            "short" => Ok(Self::Short),
            _ => Err(AppError::new(
                ErrorKind::Validation,
                "direction must be \"long\" or \"short\"",
            )),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Long => "long",
            Self::Short => "short",
        }
    }

    fn shape_name(self) -> &'static str {
        match self {
            Self::Long => "long_position",
            Self::Short => "short_position",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DrawingPositionRequest {
    pub direction: PositionDirection,
    pub entry_price: f64,
    pub stop_loss: f64,
    pub take_profit: f64,
    pub entry_time: Option<f64>,
    pub account_size: Option<f64>,
    pub risk: Option<f64>,
    pub lot_size: Option<f64>,
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

pub async fn drawing_position(
    runtime: &mut impl RuntimeEvaluator,
    request: DrawingPositionRequest,
) -> Result<Value, AppError> {
    validate_position_request(&request)?;

    let shape_name = request.direction.shape_name();
    let shape_literal = js_string(shape_name)?;
    let direction_literal = js_string(request.direction.as_str())?;
    let entry_time_expression = request
        .entry_time
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_string());
    let account_size_expression = optional_number_expression(request.account_size);
    let risk_expression = optional_number_expression(request.risk);
    let lot_size_expression = optional_number_expression(request.lot_size);

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
                    function positiveNumber(value) {{
                        return typeof value === "number" && isFinite(value) && value > 0;
                    }}

                    var api = {CHART_API};
                    var chart = api._chartWidget;
                    var mainSeries = chart.model().mainSeries();
                    var symbolInfo = mainSeries.symbolInfo && mainSeries.symbolInfo();
                    var pricescale = symbolInfo && symbolInfo.pricescale;
                    if (!positiveNumber(pricescale)) {{
                        return {{ error: "Could not determine pricescale from symbol info" }};
                    }}

                    var entryPrice = {entry_price};
                    var stopLoss = {stop_loss};
                    var takeProfit = {take_profit};
                    var entryTime = {entry_time_expression};
                    if (entryTime === null) {{
                        var range = api.getVisibleRange && api.getVisibleRange();
                        entryTime = range && typeof range.to === "number"
                            ? range.to
                            : Math.floor(Date.now() / 1000);
                    }}

                    var stopLevel = Math.round(Math.abs(entryPrice - stopLoss) * pricescale);
                    var profitLevel = Math.round(Math.abs(takeProfit - entryPrice) * pricescale);
                    var overrides = {{
                        stopLevel: stopLevel,
                        profitLevel: profitLevel
                    }};
                    var accountSize = {account_size_expression};
                    var risk = {risk_expression};
                    var lotSize = {lot_size_expression};
                    if (accountSize !== null) overrides.accountSize = accountSize;
                    if (risk !== null) overrides.risk = risk;
                    if (lotSize !== null) overrides.lotSize = lotSize;

                    var before = shapeIds(api);
                    api.createShape({{ time: entryTime, price: entryPrice }}, {{
                        shape: {shape_literal},
                        overrides: overrides
                    }});
                    await sleep(300);
                    var after = shapeIds(api);
                    var newIds = after.filter(function(id) {{ return before.indexOf(id) === -1; }});

                    return {{
                        action: "position",
                        direction: {direction_literal},
                        shape: {shape_literal},
                        entity_id: newIds.length > 0 ? newIds[0] : null,
                        new_shape_count: newIds.length,
                        before_count: before.length,
                        after_count: after.length,
                        entry_price: entryPrice,
                        stop_loss: stopLoss,
                        take_profit: takeProfit,
                        entry_time: entryTime,
                        stop_level: stopLevel,
                        profit_level: profitLevel,
                        risk_reward_ratio: stopLevel > 0 ? Math.round((profitLevel / stopLevel) * 100) / 100 : null,
                        account_size: accountSize,
                        risk: risk,
                        lot_size: lotSize,
                        source: "chart_api"
                    }};
                }})()
                "#,
                entry_price = request.entry_price,
                stop_loss = request.stop_loss,
                take_profit = request.take_profit,
            ),
            true,
        )
        .await?;

    if let Some(message) = data.get("error").and_then(Value::as_str) {
        return Err(
            AppError::new(ErrorKind::InternalApiUnavailable, message.to_string())
                .with_details(data),
        );
    }

    if data.get("entity_id").and_then(Value::as_str).is_none() {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Position drawing did not create a new entity",
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

pub fn validate_position_request(request: &DrawingPositionRequest) -> Result<(), AppError> {
    require_finite(request.entry_price, "entry_price")?;
    require_finite(request.stop_loss, "stop_loss")?;
    require_finite(request.take_profit, "take_profit")?;
    if let Some(entry_time) = request.entry_time {
        require_finite(entry_time, "entry_time")?;
    }
    validate_positive_optional(request.account_size, "account_size")?;
    validate_positive_optional(request.risk, "risk")?;
    validate_positive_optional(request.lot_size, "lot_size")?;

    match request.direction {
        PositionDirection::Long => {
            if request.stop_loss >= request.entry_price {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "long position: stop_loss must be below entry_price",
                ));
            }
            if request.take_profit <= request.entry_price {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "long position: take_profit must be above entry_price",
                ));
            }
        }
        PositionDirection::Short => {
            if request.stop_loss <= request.entry_price {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "short position: stop_loss must be above entry_price",
                ));
            }
            if request.take_profit >= request.entry_price {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "short position: take_profit must be below entry_price",
                ));
            }
        }
    }

    Ok(())
}

fn validate_positive_optional(value: Option<f64>, label: &str) -> Result<(), AppError> {
    if let Some(value) = value {
        require_finite(value, label)?;
        if value <= 0.0 {
            return Err(AppError::new(
                ErrorKind::Validation,
                format!("{label} must be greater than 0"),
            ));
        }
    }
    Ok(())
}

fn optional_number_expression(value: Option<f64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_string())
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

pub async fn drawing_clear(
    runtime: &mut impl RuntimeEvaluator,
    dry_run: bool,
) -> Result<Value, AppError> {
    let data = runtime
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
                    function shapeSummary(shape) {{
                        return {{
                            id: shape.id,
                            name: shapeName(shape)
                        }};
                    }}

                    var api = {CHART_API};
                    var before = api.getAllShapes();
                    var targets = before.map(shapeSummary);
                    var dryRun = {dry_run};
                    if (dryRun) {{
                        return {{
                            action: "dry_run",
                            dry_run: true,
                            before_count: before.length,
                            would_clear_count: before.length,
                            cleared_entities: targets,
                            source: "chart_api"
                        }};
                    }}

                    if (before.length === 0) {{
                        return {{
                            action: "noop",
                            cleared: false,
                            before_count: 0,
                            after_count: 0,
                            cleared_entities: [],
                            source: "chart_api"
                        }};
                    }}

                    api.removeAllShapes();
                    var after = api.getAllShapes();
                    return {{
                        action: "cleared",
                        cleared: after.length === 0,
                        before_count: before.length,
                        after_count: after.length,
                        cleared_entities: targets,
                        source: "chart_api"
                    }};
                }})()
                "#,
                dry_run = if dry_run { "true" } else { "false" },
            ),
            false,
        )
        .await?;

    if !dry_run && data.get("after_count").and_then(Value::as_u64) != Some(0) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Drawing clear did not remove all drawings",
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
    use tradingview_core::ErrorKind;

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

    #[test]
    fn position_direction_accepts_long_and_short_only() {
        assert_eq!(
            PositionDirection::parse("long").unwrap(),
            PositionDirection::Long
        );
        assert_eq!(
            PositionDirection::parse(" SHORT ").unwrap(),
            PositionDirection::Short
        );
        let err = PositionDirection::parse("up").unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);
    }

    #[test]
    fn validate_position_request_enforces_long_price_ordering() {
        let valid = DrawingPositionRequest {
            direction: PositionDirection::Long,
            entry_price: 100.0,
            stop_loss: 90.0,
            take_profit: 120.0,
            entry_time: None,
            account_size: None,
            risk: None,
            lot_size: None,
        };
        assert!(validate_position_request(&valid).is_ok());

        let mut invalid = valid.clone();
        invalid.stop_loss = 100.0;
        let err = validate_position_request(&invalid).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);

        let mut invalid = valid.clone();
        invalid.take_profit = 99.0;
        let err = validate_position_request(&invalid).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);
    }

    #[test]
    fn validate_position_request_enforces_short_price_ordering() {
        let valid = DrawingPositionRequest {
            direction: PositionDirection::Short,
            entry_price: 100.0,
            stop_loss: 110.0,
            take_profit: 80.0,
            entry_time: None,
            account_size: Some(10_000.0),
            risk: Some(1.0),
            lot_size: Some(0.5),
        };
        assert!(validate_position_request(&valid).is_ok());

        let mut invalid = valid.clone();
        invalid.stop_loss = 99.0;
        let err = validate_position_request(&invalid).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);

        let mut invalid = valid.clone();
        invalid.take_profit = 100.0;
        let err = validate_position_request(&invalid).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);
    }

    #[test]
    fn validate_position_request_rejects_non_finite_and_non_positive_inputs() {
        let mut request = DrawingPositionRequest {
            direction: PositionDirection::Long,
            entry_price: f64::NAN,
            stop_loss: 90.0,
            take_profit: 120.0,
            entry_time: None,
            account_size: None,
            risk: None,
            lot_size: None,
        };
        let err = validate_position_request(&request).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);

        request.entry_price = 100.0;
        request.risk = Some(0.0);
        let err = validate_position_request(&request).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);
    }

    #[tokio::test]
    async fn drawing_position_returns_entity_and_tick_levels() {
        let payload = json!({
            "action": "position",
            "direction": "long",
            "shape": "long_position",
            "entity_id": "pos123",
            "new_shape_count": 1,
            "before_count": 2,
            "after_count": 3,
            "entry_price": 100.0,
            "stop_loss": 90.0,
            "take_profit": 120.0,
            "entry_time": 1700000000,
            "stop_level": 1000,
            "profit_level": 2000,
            "risk_reward_ratio": 2.0,
            "account_size": 10000.0,
            "risk": 1.0,
            "lot_size": 0.5,
            "source": "chart_api"
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);
        let request = DrawingPositionRequest {
            direction: PositionDirection::Long,
            entry_price: 100.0,
            stop_loss: 90.0,
            take_profit: 120.0,
            entry_time: Some(1700000000.0),
            account_size: Some(10_000.0),
            risk: Some(1.0),
            lot_size: Some(0.5),
        };

        let result = drawing_position(&mut runtime, request).await.unwrap();

        assert_eq!(result, payload);
        assert!(runtime.evaluated[0].0.contains("pricescale"));
        assert!(runtime.evaluated[0].0.contains("\"long_position\""));
        assert!(runtime.evaluated[0].0.contains("accountSize"));
        assert!(runtime.evaluated[0].1);
    }

    #[tokio::test]
    async fn drawing_position_maps_missing_pricescale_to_internal_api_unavailable() {
        let mut runtime = FakeRuntime::new([json!({
            "error": "Could not determine pricescale from symbol info"
        })]);
        let request = DrawingPositionRequest {
            direction: PositionDirection::Short,
            entry_price: 100.0,
            stop_loss: 110.0,
            take_profit: 80.0,
            entry_time: None,
            account_size: None,
            risk: None,
            lot_size: None,
        };

        let err = drawing_position(&mut runtime, request).await.unwrap_err();

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
    }

    #[tokio::test]
    async fn drawing_position_requires_new_entity_id() {
        let mut runtime = FakeRuntime::new([json!({
            "action": "position",
            "entity_id": null,
            "new_shape_count": 0
        })]);
        let request = DrawingPositionRequest {
            direction: PositionDirection::Short,
            entry_price: 100.0,
            stop_loss: 110.0,
            take_profit: 80.0,
            entry_time: None,
            account_size: None,
            risk: None,
            lot_size: None,
        };

        let err = drawing_position(&mut runtime, request).await.unwrap_err();

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

    #[tokio::test]
    async fn drawing_clear_dry_run_returns_targets() {
        let payload = json!({
            "action": "dry_run",
            "dry_run": true,
            "before_count": 2,
            "would_clear_count": 2,
            "cleared_entities": [
                {"id": "shape1", "name": "Horizontal Line"},
                {"id": "shape2", "name": "Trend Line"}
            ],
            "source": "chart_api"
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);

        let result = drawing_clear(&mut runtime, true).await.unwrap();

        assert_eq!(result, payload);
        assert!(runtime.evaluated[0].0.contains("getAllShapes"));
        assert!(runtime.evaluated[0].0.contains("removeAllShapes"));
    }

    #[tokio::test]
    async fn drawing_clear_returns_success_payload() {
        let payload = json!({
            "action": "cleared",
            "cleared": true,
            "before_count": 2,
            "after_count": 0,
            "cleared_entities": [
                {"id": "shape1", "name": "Horizontal Line"},
                {"id": "shape2", "name": null}
            ],
            "source": "chart_api"
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);

        let result = drawing_clear(&mut runtime, false).await.unwrap();

        assert_eq!(result, payload);
        assert!(runtime.evaluated[0].0.contains("removeAllShapes"));
    }

    #[tokio::test]
    async fn drawing_clear_returns_noop_when_empty() {
        let payload = json!({
            "action": "noop",
            "cleared": false,
            "before_count": 0,
            "after_count": 0,
            "cleared_entities": [],
            "source": "chart_api"
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);

        let result = drawing_clear(&mut runtime, false).await.unwrap();

        assert_eq!(result, payload);
    }

    #[tokio::test]
    async fn drawing_clear_requires_empty_post_delete_state() {
        let mut runtime = FakeRuntime::new([json!({
            "action": "cleared",
            "cleared": false,
            "before_count": 2,
            "after_count": 1,
            "cleared_entities": [
                {"id": "shape1", "name": "Horizontal Line"},
                {"id": "shape2", "name": "Trend Line"}
            ],
            "source": "chart_api"
        })]);

        let err = drawing_clear(&mut runtime, false).await.unwrap_err();

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
    }
}
