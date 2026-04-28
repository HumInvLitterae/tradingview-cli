use serde_json::{Value, json};

use tradingview_core::{AppError, ErrorKind};

fn sanitize_alert_condition_value(condition: &Value) -> Value {
    let Some(object) = condition.as_object() else {
        return condition.clone();
    };

    let mut sanitized = serde_json::Map::new();
    for key in ["type", "alert_cond_id", "frequency", "resolution", "symbol"] {
        if let Some(value) = object.get(key).cloned() {
            sanitized.insert(key.to_string(), value);
        }
    }
    if let Some(value) = object.get("alertCondId").cloned() {
        sanitized
            .entry("alert_cond_id".to_string())
            .or_insert(value);
    }
    if let Some(value) = object.get("operator").cloned() {
        sanitized.insert("operator".to_string(), value);
    }
    if let Some(value) = object.get("value").cloned() {
        sanitized.insert("value".to_string(), value);
    }

    if let Some(series) = object.get("series").and_then(Value::as_array) {
        let has_study_series = series
            .iter()
            .any(|item| item.get("type").and_then(Value::as_str) == Some("study"));
        sanitized.insert("series_count".to_string(), json!(series.len()));
        sanitized.insert("has_study_series".to_string(), json!(has_study_series));
    }

    Value::Object(sanitized)
}

fn sanitize_public_alert_value(alert: &Value) -> Value {
    let Some(object) = alert.as_object() else {
        return Value::Null;
    };

    let condition = object
        .get("condition")
        .map(sanitize_alert_condition_value)
        .unwrap_or(Value::Null);
    let message = object
        .get("message")
        .or_else(|| object.get("description"))
        .cloned()
        .unwrap_or_else(|| json!(""));

    json!({
        "alert_id": object.get("alert_id").or_else(|| object.get("id")).cloned().unwrap_or(Value::Null),
        "symbol": object.get("symbol").cloned().unwrap_or(Value::Null),
        "type": object.get("type").cloned().unwrap_or(Value::Null),
        "message": message,
        "active": object.get("active").cloned().unwrap_or(Value::Bool(true)),
        "condition": condition,
        "resolution": object.get("resolution").or_else(|| object.get("interval")).cloned().unwrap_or(Value::Null),
        "created": object.get("created").or_else(|| object.get("create_time")).cloned().unwrap_or(Value::Null),
        "last_fired": object.get("last_fired").or_else(|| object.get("last_fire_time")).cloned().unwrap_or(Value::Null),
        "expiration": object.get("expiration").or_else(|| object.get("expire_time")).cloned().unwrap_or(Value::Null),
    })
}

fn sanitize_public_alert_array(value: Option<&Value>) -> Value {
    value
        .and_then(Value::as_array)
        .map(|alerts| {
            Value::Array(
                alerts
                    .iter()
                    .map(sanitize_public_alert_value)
                    .collect::<Vec<_>>(),
            )
        })
        .unwrap_or_else(|| json!([]))
}

pub(super) fn sanitize_alert_payload(mut data: Value) -> Value {
    if let Some(object) = data.as_object_mut() {
        if object.contains_key("alerts") {
            object.insert(
                "alerts".to_string(),
                sanitize_public_alert_array(object.get("alerts")),
            );
        }
        if object.contains_key("target_alerts") {
            object.insert(
                "target_alerts".to_string(),
                sanitize_public_alert_array(object.get("target_alerts")),
            );
        }
        if let Some(matched_alert) = object.get("matched_alert").cloned() {
            object.insert(
                "matched_alert".to_string(),
                sanitize_public_alert_value(&matched_alert),
            );
        }
    }
    data
}

pub(super) fn normalize_alert_list_payload(data: Value) -> Value {
    let data = sanitize_alert_payload(data);
    let alerts = data
        .get("alerts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut payload = json!({
        "alert_count": data
            .get("alert_count")
            .and_then(Value::as_u64)
            .unwrap_or(alerts.len() as u64),
        "source": data
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or("internal_api"),
        "alerts": alerts,
    });

    if let Some(error) = data.get("error").cloned() {
        payload["error"] = error;
    }

    payload
}

pub(super) fn normalize_alert_create_payload(data: Value) -> Result<Value, AppError> {
    let data = sanitize_alert_payload(data);
    if let Some(message) = data.get("error").and_then(Value::as_str) {
        let kind = match data.get("error_kind").and_then(Value::as_str) {
            Some("validation") => ErrorKind::Validation,
            _ => ErrorKind::InternalApiUnavailable,
        };
        return Err(AppError::new(kind, message.to_string()).with_details(data));
    }

    if !data
        .get("price_set")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Alert price input could not be set",
        )
        .with_details(data));
    }

    if !data
        .get("created")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Alert create button could not be clicked",
        )
        .with_details(data));
    }

    Ok(json!({
        "price": data.get("price").cloned().unwrap_or(Value::Null),
        "condition": data
            .get("condition")
            .and_then(Value::as_str)
            .unwrap_or("crossing"),
        "message": data
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("(none)"),
        "price_set": true,
        "source": data
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or("dom_fallback"),
        "created": true,
        "opened": data
            .get("opened")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        "open_selector": data.get("open_selector").cloned().unwrap_or(Value::Null),
        "message_set": data
            .get("message_set")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        "alert_id": data.get("alert_id").cloned().unwrap_or(Value::Null),
        "symbol": data.get("symbol").cloned().unwrap_or(Value::Null),
        "resolution": data.get("resolution").cloned().unwrap_or(Value::Null),
        "condition_type": data.get("condition_type").cloned().unwrap_or(Value::Null),
        "before_count": data.get("before_count").cloned().unwrap_or(Value::Null),
        "after_count": data.get("after_count").cloned().unwrap_or(Value::Null),
        "matched_alert": data.get("matched_alert").cloned().unwrap_or(Value::Null),
    }))
}

pub(super) fn normalize_indicator_alert_create_payload(data: Value) -> Result<Value, AppError> {
    let data = sanitize_alert_payload(data);
    if let Some(message) = data.get("error").and_then(Value::as_str) {
        let kind = match data.get("error_kind").and_then(Value::as_str) {
            Some("validation") => ErrorKind::Validation,
            _ => ErrorKind::InternalApiUnavailable,
        };
        return Err(AppError::new(kind, message.to_string()).with_details(data));
    }

    if !data
        .get("created")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Indicator alert create did not confirm a created alert",
        )
        .with_details(data));
    }

    Ok(json!({
        "action": data
            .get("action")
            .and_then(Value::as_str)
            .unwrap_or("create_indicator"),
        "dry_run": false,
        "created": true,
        "source": data
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or("indicator_alert_api"),
        "alert_id": data.get("alert_id").cloned().unwrap_or(Value::Null),
        "symbol": data.get("symbol").cloned().unwrap_or(Value::Null),
        "resolution": data.get("resolution").cloned().unwrap_or(Value::Null),
        "message": data.get("message").cloned().unwrap_or(Value::Null),
        "before_count": data.get("before_count").cloned().unwrap_or(Value::Null),
        "after_count": data.get("after_count").cloned().unwrap_or(Value::Null),
        "script": data.get("script").cloned().unwrap_or(Value::Null),
        "condition": data.get("condition").cloned().unwrap_or(Value::Null),
        "input_metadata": data.get("input_metadata").cloned().unwrap_or(Value::Null),
        "matched_alert": data.get("matched_alert").cloned().unwrap_or(Value::Null),
    }))
}

pub(super) fn alert_api_error_allows_fallback(error: &AppError) -> bool {
    error
        .details
        .as_ref()
        .and_then(|details| details.get("api_fallback_allowed"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

pub(super) fn normalize_alert_delete_payload(data: Value) -> Result<Value, AppError> {
    let data = sanitize_alert_payload(data);
    if let Some(message) = data.get("error").and_then(Value::as_str) {
        let kind = match data.get("error_kind").and_then(Value::as_str) {
            Some("validation") => ErrorKind::Validation,
            _ => ErrorKind::InternalApiUnavailable,
        };
        return Err(AppError::new(kind, message.to_string()).with_details(data));
    }

    if !data
        .get("deleted")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Alert delete did not remove the requested alert",
        )
        .with_details(data));
    }

    Ok(json!({
        "alert_id": data.get("alert_id").cloned().unwrap_or(Value::Null),
        "deleted": true,
        "source": data
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or("internal_api"),
        "before_count": data.get("before_count").cloned().unwrap_or(Value::Null),
        "after_count": data.get("after_count").cloned().unwrap_or(Value::Null),
        "matched_before": data
            .get("matched_before")
            .and_then(Value::as_bool)
            .unwrap_or(true),
        "matched_after": data
            .get("matched_after")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        "matched_alert": data.get("matched_alert").cloned().unwrap_or(Value::Null),
    }))
}

pub(super) fn normalize_alert_delete_all_payload(data: Value) -> Result<Value, AppError> {
    let data = sanitize_alert_payload(data);
    if let Some(message) = data.get("error").and_then(Value::as_str) {
        let kind = match data.get("error_kind").and_then(Value::as_str) {
            Some("validation") => ErrorKind::Validation,
            _ => ErrorKind::InternalApiUnavailable,
        };
        return Err(AppError::new(kind, message.to_string()).with_details(data));
    }

    if data
        .get("action")
        .and_then(Value::as_str)
        .is_some_and(|action| action == "delete_all")
        && !data
            .get("deleted")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Alert delete --all did not remove all target alerts",
        )
        .with_details(data));
    }

    Ok(json!({
        "action": data.get("action").cloned().unwrap_or_else(|| json!("delete_all")),
        "dry_run": data.get("dry_run").and_then(Value::as_bool).unwrap_or(false),
        "deleted": data.get("deleted").and_then(Value::as_bool).unwrap_or(false),
        "source": data.get("source").and_then(Value::as_str).unwrap_or("internal_api"),
        "before_count": data.get("before_count").cloned().unwrap_or(Value::Null),
        "after_count": data.get("after_count").cloned().unwrap_or(Value::Null),
        "target_alert_ids": data.get("target_alert_ids").cloned().unwrap_or_else(|| json!([])),
        "target_alerts": data.get("target_alerts").cloned().unwrap_or_else(|| json!([])),
        "remaining_target_alert_ids": data
            .get("remaining_target_alert_ids")
            .cloned()
            .unwrap_or_else(|| json!([])),
    }))
}
