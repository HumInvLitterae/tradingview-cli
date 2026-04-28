use serde_json::Value;

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};

use super::super::common::{CHART_API, js_string, require_finite};
use super::validation::{DrawingPositionRequest, DrawingShapeRequest, validate_position_request};

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

fn optional_number_expression(value: Option<f64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_string())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::ops::test_support::FakeRuntime;

    use super::super::validation::{DrawingPoint, DrawingPositionRequest, PositionDirection};
    use super::*;
    use tradingview_core::ErrorKind;

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
        let payload = json!({"action": "shape", "shape": "trend_line", "entity_id": "shape123", "new_shape_count": 1});
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
        let mut runtime = FakeRuntime::new([
            json!({"action": "shape", "shape": "horizontal_line", "entity_id": null, "new_shape_count": 0}),
        ]);
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
    async fn drawing_position_returns_entity_and_tick_levels() {
        let payload = json!({
            "action": "position", "direction": "long", "shape": "long_position", "entity_id": "pos123",
            "new_shape_count": 1, "before_count": 2, "after_count": 3, "entry_price": 100.0,
            "stop_loss": 90.0, "take_profit": 120.0, "entry_time": 1700000000,
            "stop_level": 1000, "profit_level": 2000, "risk_reward_ratio": 2.0,
            "account_size": 10000.0, "risk": 1.0, "lot_size": 0.5, "source": "chart_api"
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
        let mut runtime =
            FakeRuntime::new([json!({"error": "Could not determine pricescale from symbol info"})]);
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
        let mut runtime = FakeRuntime::new([
            json!({"action": "position", "entity_id": null, "new_shape_count": 0}),
        ]);
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
}
