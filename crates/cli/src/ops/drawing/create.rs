use serde_json::Value;

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};

use super::super::common::{CHART_API, js_string};
use super::validation::{
    DrawingPositionRequest, DrawingShapeRequest, validate_position_request, validate_shape_request,
};

pub async fn drawing_shape(
    runtime: &mut impl RuntimeEvaluator,
    request: DrawingShapeRequest,
) -> Result<Value, AppError> {
    validate_shape_request(&request)?;
    if request.point3.is_some() {
        return drawing_three_point_shape(runtime, request).await;
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

async fn drawing_three_point_shape(
    runtime: &mut impl RuntimeEvaluator,
    request: DrawingShapeRequest,
) -> Result<Value, AppError> {
    let points = [
        &request.point,
        request.point2.as_ref().expect("validated point2"),
        request.point3.as_ref().expect("validated point3"),
    ];
    let points_json = serde_json::to_string(
        &points
            .iter()
            .map(|point| serde_json::json!({"time": point.time, "price": point.price}))
            .collect::<Vec<_>>(),
    )
    .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))?;
    let overrides_json = serde_json::to_string(
        request
            .overrides
            .as_ref()
            .unwrap_or(&Value::Object(Default::default())),
    )
    .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))?;
    let expression = THREE_POINT_SHAPE_TEMPLATE
        .replace("__REQUESTED_POINTS__", &points_json)
        .replace("__OVERRIDES__", &overrides_json);

    let data = runtime.evaluate(&expression, true).await.map_err(|err| {
        AppError::new(err.kind, "Three-point drawing evaluation failed").with_details(
            serde_json::json!({
                "shape": "parallel_channel",
                "verification_status": "evaluation_failed",
                "next_action_hint": "Inspect the selected chart before retrying; the creation outcome is unknown."
            }),
        )
    })?;

    if !three_point_success_is_valid(&data, &request) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Three-point drawing could not be verified",
        )
        .with_details(three_point_failure_details(&data)));
    }

    Ok(data)
}

fn three_point_success_is_valid(data: &Value, request: &DrawingShapeRequest) -> bool {
    let Some(entity_id) = data.get("entity_id").and_then(Value::as_str) else {
        return false;
    };
    if entity_id.trim().is_empty()
        || data.get("action").and_then(Value::as_str) != Some("shape")
        || data.get("verified").and_then(Value::as_bool) != Some(true)
        || data.get("verification_status").and_then(Value::as_str) != Some("verified")
        || data.get("new_shape_count").and_then(Value::as_u64) != Some(1)
        || data.get("before_count").and_then(Value::as_u64).is_none()
        || data.get("after_count").and_then(Value::as_u64).is_none()
        || data.get("text").and_then(Value::as_str) != Some("")
        || data.get("override_count").and_then(Value::as_u64)
            != Some(request_override_count(request))
        || data.get("requested_point_count").and_then(Value::as_u64) != Some(3)
        || data.get("observed_point_count").and_then(Value::as_u64) != Some(3)
        || data.get("shape").and_then(Value::as_str) != Some("parallel_channel")
        || data.get("source").and_then(Value::as_str) != Some("chart_api")
        || data.get("source_category").and_then(Value::as_str) != Some("desktop_backed_operation")
        || data.get("requires_desktop").and_then(Value::as_bool) != Some(true)
        || data.get("non_mutating").and_then(Value::as_bool) != Some(false)
        || data.get("sticky_ambiguity").and_then(Value::as_bool) != Some(false)
        || !known_creation_signal(data.get("creation_signal").and_then(Value::as_str))
    {
        return false;
    }
    let candidate_ids = data.get("candidate_entity_ids").and_then(Value::as_array);
    if !candidate_ids
        .is_some_and(|ids| ids.len() == 1 && ids[0].as_str().is_some_and(|id| id == entity_id))
    {
        return false;
    }
    let Some(observed) = data.get("observed_points").and_then(Value::as_array) else {
        return false;
    };
    let expected = [
        &request.point,
        request.point2.as_ref().expect("validated point2"),
        request.point3.as_ref().expect("validated point3"),
    ];
    observed.len() == expected.len()
        && observed
            .iter()
            .zip(expected.iter().copied())
            .all(|(actual, expected)| point_value_matches(actual, expected))
        && ["point", "point2", "point3"]
            .into_iter()
            .zip(expected)
            .all(|(key, expected)| {
                data.get(key)
                    .is_some_and(|actual| point_echo_matches(actual, expected))
            })
}

fn request_override_count(request: &DrawingShapeRequest) -> u64 {
    request
        .overrides
        .as_ref()
        .and_then(Value::as_object)
        .map_or(0, |overrides| overrides.len() as u64)
}

fn point_value_matches(
    actual: &Value,
    expected: &tradingview_model::drawing::DrawingPoint,
) -> bool {
    actual.get("time").and_then(Value::as_f64) == Some(expected.time)
        && actual
            .get("price")
            .and_then(Value::as_f64)
            .is_some_and(|price| nearly_equal_price(price, expected.price))
}

fn point_echo_matches(actual: &Value, expected: &tradingview_model::drawing::DrawingPoint) -> bool {
    actual.get("time").and_then(Value::as_f64) == Some(expected.time)
        && actual.get("price").and_then(Value::as_f64) == Some(expected.price)
}

fn nearly_equal_price(actual: f64, expected: f64) -> bool {
    actual.is_finite()
        && expected.is_finite()
        && (actual - expected).abs()
            <= f64::EPSILON * 8.0 * actual.abs().max(expected.abs()).max(1.0)
}

fn three_point_failure_details(data: &Value) -> Value {
    let verification_status =
        normalized_verification_status(data.get("verification_status").and_then(Value::as_str));
    let creation_signal =
        normalized_creation_signal(data.get("creation_signal").and_then(Value::as_str));
    let new_shape_count = data
        .get("new_shape_count")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    serde_json::json!({
        "shape": "parallel_channel",
        "operation": "drawing_shape",
        "requested_point_count": 3,
        "verification_status": verification_status,
        "creation_signal": creation_signal,
        "new_shape_count": new_shape_count,
        "observed_point_count": data.get("observed_point_count").and_then(Value::as_u64).unwrap_or(0),
        "sticky_ambiguity": data.get("sticky_ambiguity").and_then(Value::as_bool).unwrap_or(false),
        "candidate_entity_ids": sanitized_candidate_ids(data, verification_status, new_shape_count),
        "source": "chart_api",
        "source_category": "desktop_backed_operation",
        "requires_desktop": true,
        "non_mutating": false,
        "next_action_hint": "Inspect the selected chart and remove any unintended drawing by exact entity ID before retrying."
    })
}

fn known_creation_signal(value: Option<&str>) -> bool {
    matches!(
        value,
        Some(
            "returned_non_thenable" | "fulfilled" | "rejected" | "threw" | "pending_at_observation"
        )
    )
}

fn normalized_creation_signal(value: Option<&str>) -> &'static str {
    match value {
        Some("returned_non_thenable") => "returned_non_thenable",
        Some("fulfilled") => "fulfilled",
        Some("rejected") => "rejected",
        Some("threw") => "threw",
        Some("pending_at_observation") => "pending_at_observation",
        _ => "invalid",
    }
}

fn normalized_verification_status(value: Option<&str>) -> &'static str {
    match value {
        Some("capability_unavailable") => "capability_unavailable",
        Some("inventory_failed") => "inventory_failed",
        Some("ambiguous_multiple_candidates") => "ambiguous_multiple_candidates",
        Some("candidate_id_invalid") => "candidate_id_invalid",
        Some("lookup_failed") => "lookup_failed",
        Some("identity_mismatch") => "identity_mismatch",
        Some("point_readback_failed") => "point_readback_failed",
        Some("point_readback_malformed") => "point_readback_malformed",
        Some("point_mismatch") => "point_mismatch",
        Some("deadline_no_candidate") => "deadline_no_candidate",
        Some("verified") => "verified",
        _ => "invalid_result",
    }
}

fn sanitized_candidate_ids(data: &Value, status: &str, count: u64) -> Vec<Value> {
    let permits_candidates = match status {
        "ambiguous_multiple_candidates" => count > 1,
        "lookup_failed"
        | "identity_mismatch"
        | "point_readback_failed"
        | "point_readback_malformed"
        | "point_mismatch" => count == 1,
        _ => false,
    };
    if !permits_candidates {
        return Vec::new();
    }
    data.get("candidate_entity_ids")
        .and_then(Value::as_array)
        .filter(|ids| ids.len() as u64 == count)
        .filter(|ids| {
            ids.iter()
                .all(|id| id.as_str().is_some_and(|id| !id.trim().is_empty()))
        })
        .cloned()
        .unwrap_or_default()
}

const THREE_POINT_SHAPE_TEMPLATE: &str = r#"
(async function() {
    const requestedPoints = __REQUESTED_POINTS__;
    const result = {
        action: "shape",
        shape: "parallel_channel",
        entity_id: null,
        new_shape_count: 0,
        before_count: 0,
        after_count: 0,
        candidate_entity_ids: [],
        point: requestedPoints[0],
        point2: requestedPoints[1],
        point3: requestedPoints[2],
        observed_points: null,
        observed_point_count: 0,
        requested_point_count: 3,
        verified: false,
        verification_status: "capability_unavailable",
        creation_signal: "threw",
        sticky_ambiguity: false,
        source: "chart_api",
        source_category: "desktop_backed_operation",
        requires_desktop: true,
        non_mutating: false,
        text: "",
        override_count: Object.keys(__OVERRIDES__).length,
    };
    const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
    const shapeId = (row) => {
        try { return row && typeof row.id === "string" ? row.id : null; }
        catch (_) { return null; }
    };
    const shapeIdentity = (row) => {
        try {
            const value = typeof row.name === "function" ? row.name() : row.name;
            return typeof value === "string" ? value : null;
        } catch (_) { return null; }
    };
    const inventory = (api) => {
        try {
            const rows = api.getAllShapes();
            if (!Array.isArray(rows)) return null;
            const ids = rows.map(shapeId);
            return ids.every((id) => id !== null) ? rows : null;
        } catch (_) { return null; }
    };
    const nearlyEqual = (actual, expected) => {
        const scale = Math.max(1, Math.abs(actual), Math.abs(expected));
        return Math.abs(actual - expected) <= Number.EPSILON * 8 * scale;
    };
    const inspectPoints = (points) => {
        try {
            const wellFormed = Array.isArray(points) && points.length === 3
                && points.every((point) => point
                    && typeof point.time === "number" && Number.isFinite(point.time)
                    && typeof point.price === "number" && Number.isFinite(point.price));
            return {
                wellFormed,
                exact: wellFormed && points.every((point, index) =>
                    point.time === requestedPoints[index].time
                    && nearlyEqual(point.price, requestedPoints[index].price)),
            };
        } catch (_) { return { wellFormed: false, exact: false }; }
    };

    let api;
    try { api = window.TradingViewApi?._activeChartWidgetWV?.value?.(); }
    catch (_) { return result; }
    if (!api || typeof api.createMultipointShape !== "function"
        || typeof api.getAllShapes !== "function"
        || typeof api.getShapeById !== "function") return result;
    const before = inventory(api);
    if (!before) { result.verification_status = "inventory_failed"; return result; }
    result.before_count = before.length;
    const beforeIds = new Set(before.map(shapeId));
    const startedAt = Date.now();
    try {
        const creation = api.createMultipointShape(requestedPoints, {
            shape: "parallel_channel",
            overrides: __OVERRIDES__,
            text: "",
        });
        if (creation && typeof creation.then === "function") {
            result.creation_signal = "pending_at_observation";
            Promise.resolve(creation).then(
                () => { result.creation_signal = "fulfilled"; },
                () => { result.creation_signal = "rejected"; },
            );
        } else {
            result.creation_signal = "returned_non_thenable";
        }
    } catch (_) {
        result.creation_signal = "threw";
    }

    while (Date.now() - startedAt < 3000) {
        const after = inventory(api);
        if (!after) { result.verification_status = "inventory_failed"; return result; }
        const candidates = after.filter((row) => !beforeIds.has(shapeId(row)));
        result.after_count = after.length;
        result.new_shape_count = candidates.length;
        result.candidate_entity_ids = candidates.map(shapeId).filter((id) => id !== null);
        if (candidates.length > 1) {
            result.sticky_ambiguity = true;
            result.verification_status = "ambiguous_multiple_candidates";
            return result;
        }
        if (candidates.length === 1) {
            const candidate = candidates[0];
            const candidateId = shapeId(candidate);
            if (candidateId === null) { result.verification_status = "candidate_id_invalid"; return result; }
            let found;
            try { found = api.getShapeById(candidateId); }
            catch (_) { result.verification_status = "lookup_failed"; return result; }
            if (!found) { await sleep(100); continue; }
            if (shapeIdentity(candidate) !== "parallel_channel") {
                result.verification_status = "identity_mismatch";
                return result;
            }
            let observed;
            try { observed = found.getPoints(); }
            catch (_) { result.verification_status = "point_readback_failed"; return result; }
            result.observed_point_count = Array.isArray(observed) ? observed.length : 0;
            const inspected = inspectPoints(observed);
            if (!inspected.wellFormed) {
                result.verification_status = "point_readback_malformed";
                return result;
            }
            if (!inspected.exact) { result.verification_status = "point_mismatch"; return result; }
            result.entity_id = candidateId;
            result.observed_points = observed;
            result.verified = true;
            result.verification_status = "verified";
            return result;
        }
        await sleep(100);
    }
    result.verification_status = "deadline_no_candidate";
    return result;
})()
"#;

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
            point3: None,
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
            point3: None,
            text: None,
            overrides: None,
        };

        let result = drawing_shape(&mut runtime, request).await.unwrap();

        assert_eq!(result["entity_id"], "shape123");
        assert!(runtime.evaluated[0].0.contains("createMultipointShape"));
    }

    #[tokio::test]
    async fn drawing_shape_verifies_native_parallel_channel() {
        let payload = valid_three_point_payload();
        let mut runtime = FakeRuntime::new([payload.clone()]);

        let result = drawing_shape(&mut runtime, parallel_channel_request())
            .await
            .unwrap();

        assert_eq!(result, payload);
        let expression = &runtime.evaluated[0].0;
        assert!(expression.contains("createMultipointShape(requestedPoints"));
        assert!(expression.contains("candidates.length > 1"));
        assert!(expression.contains("Date.now() - startedAt < 3000"));
        assert!(!expression.contains("removeEntity"));
    }

    #[tokio::test]
    async fn drawing_shape_requires_request_consistent_override_count() {
        let mut request = parallel_channel_request();
        request.overrides = Some(json!({"linecolor": "red"}));
        let mut payload = valid_three_point_payload();
        payload["override_count"] = json!(1);
        let mut runtime = FakeRuntime::new([payload]);

        drawing_shape(&mut runtime, request).await.unwrap();

        let mut payload = valid_three_point_payload();
        payload["override_count"] = json!(1);
        let mut runtime = FakeRuntime::new([payload]);
        let error = drawing_shape(&mut runtime, parallel_channel_request())
            .await
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
    }

    fn valid_three_point_payload() -> Value {
        json!({
            "action": "shape",
            "shape": "parallel_channel",
            "entity_id": "shape123",
            "new_shape_count": 1,
            "before_count": 2,
            "after_count": 3,
            "candidate_entity_ids": ["shape123"],
            "point": {"time": 100, "price": 10},
            "point2": {"time": 200, "price": 12},
            "point3": {"time": 100, "price": 8},
            "observed_points": [
                {"time": 100, "price": 10},
                {"time": 200, "price": 12},
                {"time": 100, "price": 8}
            ],
            "observed_point_count": 3,
            "requested_point_count": 3,
            "verified": true,
            "verification_status": "verified",
            "creation_signal": "rejected",
            "sticky_ambiguity": false,
            "text": "",
            "override_count": 0,
            "source": "chart_api",
            "source_category": "desktop_backed_operation",
            "requires_desktop": true,
            "non_mutating": false
        })
    }

    #[tokio::test]
    async fn drawing_shape_rejects_contradictory_three_point_success_payloads() {
        let mut cases = Vec::new();
        let mut payload = valid_three_point_payload();
        payload["sticky_ambiguity"] = Value::Bool(true);
        cases.push(payload);
        let mut payload = valid_three_point_payload();
        payload["candidate_entity_ids"] = json!(["shape123", "other"]);
        cases.push(payload);
        let mut payload = valid_three_point_payload();
        payload["candidate_entity_ids"] = json!(["other"]);
        cases.push(payload);
        let mut payload = valid_three_point_payload();
        payload["creation_signal"] = Value::String("private-signal".into());
        cases.push(payload);
        let mut payload = valid_three_point_payload();
        payload["action"] = Value::String("private-action".into());
        cases.push(payload);
        for key in ["point", "point2", "point3"] {
            let mut payload = valid_three_point_payload();
            payload[key] = Value::Null;
            cases.push(payload);
        }
        for key in [
            "action",
            "entity_id",
            "candidate_entity_ids",
            "creation_signal",
            "sticky_ambiguity",
            "point",
            "point2",
            "point3",
            "before_count",
            "after_count",
            "text",
            "override_count",
        ] {
            let mut payload = valid_three_point_payload();
            payload.as_object_mut().unwrap().remove(key);
            cases.push(payload);
        }
        for (key, invalid) in [
            ("before_count", json!("2")),
            ("after_count", json!(-1)),
            ("text", json!(false)),
            ("override_count", json!("0")),
        ] {
            let mut payload = valid_three_point_payload();
            payload[key] = invalid;
            cases.push(payload);
        }

        for payload in cases {
            let mut runtime = FakeRuntime::new([payload]);
            let error = drawing_shape(&mut runtime, parallel_channel_request())
                .await
                .unwrap_err();
            assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        }
    }

    #[tokio::test]
    async fn drawing_shape_sanitizes_unverified_three_point_result() {
        let mut runtime = FakeRuntime::new([json!({
            "verified": false,
            "verification_status": "point_mismatch",
            "creation_signal": "fulfilled",
            "new_shape_count": 1,
            "observed_point_count": 3,
            "sticky_ambiguity": false,
            "raw_source": "private-source",
            "target_id": "private-target"
        })]);

        let error = drawing_shape(&mut runtime, parallel_channel_request())
            .await
            .unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        let details = error.details.unwrap().to_string();
        assert!(details.contains("point_mismatch"));
        assert!(!details.contains("private-source"));
        assert!(!details.contains("private-target"));
    }

    #[test]
    fn three_point_failure_details_normalize_known_fields_and_candidate_handles() {
        let details = three_point_failure_details(&json!({
            "verification_status": "private-status",
            "creation_signal": "private-signal",
            "new_shape_count": 1,
            "candidate_entity_ids": ["private-target"],
        }));
        assert_eq!(details["verification_status"], "invalid_result");
        assert_eq!(details["creation_signal"], "invalid");
        assert_eq!(details["candidate_entity_ids"], json!([]));
        assert!(!details.to_string().contains("private"));

        let details = three_point_failure_details(&json!({
            "verification_status": "point_mismatch",
            "creation_signal": "rejected",
            "new_shape_count": 1,
            "candidate_entity_ids": ["shape123"],
        }));
        assert_eq!(details["candidate_entity_ids"], json!(["shape123"]));

        let details = three_point_failure_details(&json!({
            "verification_status": "point_mismatch",
            "creation_signal": "rejected",
            "new_shape_count": 2,
            "candidate_entity_ids": ["shape123", "other"],
        }));
        assert_eq!(details["candidate_entity_ids"], json!([]));
    }

    fn parallel_channel_request() -> DrawingShapeRequest {
        DrawingShapeRequest {
            shape_type: "parallel_channel".into(),
            point: DrawingPoint {
                time: 100.0,
                price: 10.0,
            },
            point2: Some(DrawingPoint {
                time: 200.0,
                price: 12.0,
            }),
            point3: Some(DrawingPoint {
                time: 100.0,
                price: 8.0,
            }),
            text: None,
            overrides: None,
        }
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
            point3: None,
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
            point3: None,
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
