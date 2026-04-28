use serde_json::{Value, json};

use tradingview_core::{AppError, ErrorKind};

pub(super) fn normalize_replay_action(data: Value) -> Result<Value, AppError> {
    if data.get("ok").and_then(Value::as_bool) == Some(false) {
        let message = data
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("TradingView replay operation failed");
        let kind = match data.get("error_kind").and_then(Value::as_str) {
            Some("validation") => ErrorKind::Validation,
            _ => ErrorKind::InternalApiUnavailable,
        };
        return Err(AppError::new(kind, message).with_details(data));
    }

    if data.get("action").is_none() {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "TradingView replay operation did not return an action",
        )
        .with_details(data));
    }

    let mut payload = data;
    if let Some(object) = payload.as_object_mut() {
        object.remove("ok");
    }
    Ok(payload)
}

pub(super) fn normalize_replay_status(data: Value) -> Result<Value, AppError> {
    if data.get("ok").and_then(Value::as_bool) == Some(false) {
        let message = data
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("TradingView replay API is not available");
        return Err(AppError::new(ErrorKind::InternalApiUnavailable, message).with_details(data));
    }

    if data.get("is_replay_available").is_none() {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "TradingView replay status did not return replay state",
        )
        .with_details(data));
    }

    Ok(json!({
        "is_replay_available": data.get("is_replay_available").cloned().unwrap_or(Value::Null),
        "is_replay_started": data.get("is_replay_started").cloned().unwrap_or(Value::Null),
        "is_autoplay_started": data.get("is_autoplay_started").cloned().unwrap_or(Value::Null),
        "replay_mode": data.get("replay_mode").cloned().unwrap_or(Value::Null),
        "current_date": data.get("current_date").cloned().unwrap_or(Value::Null),
        "autoplay_delay": data.get("autoplay_delay").cloned().unwrap_or(Value::Null),
        "position": data.get("position").cloned().unwrap_or(Value::Null),
        "realized_pnl": data.get("realized_pnl").cloned().unwrap_or(Value::Null),
        "source": data.get("source").cloned().unwrap_or_else(|| json!("internal_api")),
    }))
}
