use serde_json::{Value, json};

use tradingview_core::{AppError, ErrorKind};

const VALID_AUTOPLAY_DELAYS: [u64; 9] = [100, 143, 200, 300, 1000, 2000, 3000, 5000, 10000];
pub const MAX_REPLAY_LOG_STEPS: u64 = 100;

pub fn validate_replay_date(date: &str) -> Result<(), AppError> {
    parse_replay_date_ms(date).map(|_| ())
}

pub fn validate_replay_autoplay_speed(speed: u64) -> Result<(), AppError> {
    if speed == 0 || VALID_AUTOPLAY_DELAYS.contains(&speed) {
        return Ok(());
    }

    Err(AppError::new(
        ErrorKind::Validation,
        format!(
            "Invalid replay autoplay delay: {speed}ms. Use 0 or one of: {}.",
            VALID_AUTOPLAY_DELAYS
                .iter()
                .map(u64::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    )
    .with_details(json!({
        "speed": speed,
        "supported": VALID_AUTOPLAY_DELAYS,
    })))
}

pub fn validate_replay_trade_action(action: &str) -> Result<(), AppError> {
    match action {
        "buy" | "sell" | "close" => Ok(()),
        _ => Err(AppError::new(
            ErrorKind::Validation,
            "Invalid replay trade action. Use buy, sell, or close.",
        )
        .with_details(json!({
            "action": action,
            "supported": ["buy", "sell", "close"],
        }))),
    }
}

pub fn validate_replay_log_steps(steps: u64) -> Result<(), AppError> {
    if (1..=MAX_REPLAY_LOG_STEPS).contains(&steps) {
        return Ok(());
    }

    Err(AppError::new(
        ErrorKind::Validation,
        format!("replay log steps must be between 1 and {MAX_REPLAY_LOG_STEPS}"),
    )
    .with_details(json!({
        "field": "steps",
        "value": steps,
        "minimum": 1,
        "maximum": MAX_REPLAY_LOG_STEPS,
    })))
}

pub fn parse_replay_date_ms(date: &str) -> Result<i64, AppError> {
    let trimmed = date.trim();
    let parts = trimmed.split('-').collect::<Vec<_>>();
    if parts.len() != 3 || parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
        return Err(invalid_replay_date(date));
    }

    let year = parts[0]
        .parse::<i32>()
        .map_err(|_| invalid_replay_date(date))?;
    let month = parts[1]
        .parse::<u32>()
        .map_err(|_| invalid_replay_date(date))?;
    let day = parts[2]
        .parse::<u32>()
        .map_err(|_| invalid_replay_date(date))?;
    if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
        return Err(invalid_replay_date(date));
    }

    let days = days_from_civil(year, month, day);
    Ok(days * 86_400_000)
}

fn invalid_replay_date(date: &str) -> AppError {
    AppError::new(
        ErrorKind::Validation,
        format!("Invalid replay date: {date}. Use YYYY-MM-DD."),
    )
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = year - i32::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = (year - era * 400) as u32;
    let month = month as i32;
    let day = day as i32;
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era =
        year_of_era as i32 * 365 + year_of_era as i32 / 4 - year_of_era as i32 / 100 + day_of_year;
    (era * 146_097 + day_of_era - 719_468) as i64
}

pub fn normalize_replay_action(data: Value) -> Result<Value, AppError> {
    normalize_replay_operation(data, "replay_operation")
}

pub fn normalize_replay_operation(data: Value, default_operation: &str) -> Result<Value, AppError> {
    if data.get("ok").and_then(Value::as_bool) == Some(false) {
        let message = data
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("TradingView replay operation failed")
            .to_string();
        let kind = match data.get("error_kind").and_then(Value::as_str) {
            Some("validation") => ErrorKind::Validation,
            _ => ErrorKind::InternalApiUnavailable,
        };
        let details = enrich_replay_action_payload(data, default_operation);
        return Err(AppError::new(kind, message).with_details(details));
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
    Ok(enrich_replay_action_payload(payload, default_operation))
}

pub fn normalize_replay_status(data: Value) -> Result<Value, AppError> {
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

    let mut payload = json!({
        "is_replay_available": data.get("is_replay_available").cloned().unwrap_or(Value::Null),
        "is_replay_started": data.get("is_replay_started").cloned().unwrap_or(Value::Null),
        "is_autoplay_started": data.get("is_autoplay_started").cloned().unwrap_or(Value::Null),
        "replay_mode": data.get("replay_mode").cloned().unwrap_or(Value::Null),
        "current_date": data.get("current_date").cloned().unwrap_or(Value::Null),
        "autoplay_delay": data.get("autoplay_delay").cloned().unwrap_or(Value::Null),
        "position": data.get("position").cloned().unwrap_or(Value::Null),
        "realized_pnl": data.get("realized_pnl").cloned().unwrap_or(Value::Null),
        "source": data.get("source").cloned().unwrap_or_else(|| json!("internal_api")),
        "source_category": "desktop_backed_read",
        "requires_desktop": true,
        "non_mutating": true,
        "replay_context": replay_context_from(&data),
    });
    if let Some(chart_context) = data.get("chart_context") {
        payload["chart_context"] = chart_context.clone();
    }
    Ok(payload)
}

fn enrich_replay_action_payload(mut payload: Value, default_operation: &str) -> Value {
    let operation = payload
        .get("operation")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| {
            payload
                .get("action")
                .and_then(Value::as_str)
                .map(|action| replay_operation_from_action(Some(action)))
                .unwrap_or_else(|| default_operation.to_string())
        });
    let replay_context = replay_context_from(&payload);

    if let Some(object) = payload.as_object_mut() {
        object
            .entry("source".to_string())
            .or_insert_with(|| json!("internal_api"));
        object
            .entry("source_category".to_string())
            .or_insert_with(|| json!("desktop_backed_operation"));
        object
            .entry("requires_desktop".to_string())
            .or_insert_with(|| json!(true));
        object
            .entry("non_mutating".to_string())
            .or_insert_with(|| json!(false));
        object
            .entry("operation".to_string())
            .or_insert_with(|| json!(operation));
        object
            .entry("replay_context".to_string())
            .or_insert(replay_context);
    }
    payload
}

fn replay_operation_from_action(action: Option<&str>) -> String {
    match action {
        Some("started") => "replay_start",
        Some("step") => "replay_step",
        Some("replay_stopped" | "already_stopped") => "replay_stop",
        Some("autoplay") => "replay_autoplay",
        Some("buy" | "sell" | "close") => "replay_trade",
        _ => "replay_operation",
    }
    .to_string()
}

fn replay_context_from(data: &Value) -> Value {
    json!({
        "is_replay_available": data.get("is_replay_available").cloned().unwrap_or(Value::Null),
        "is_replay_started": data
            .get("is_replay_started")
            .or_else(|| data.get("replay_started"))
            .cloned()
            .unwrap_or(Value::Null),
        "is_autoplay_started": data
            .get("is_autoplay_started")
            .or_else(|| data.get("autoplay_active"))
            .cloned()
            .unwrap_or(Value::Null),
        "replay_mode": data.get("replay_mode").cloned().unwrap_or(Value::Null),
        "current_date": data.get("current_date").cloned().unwrap_or(Value::Null),
        "autoplay_delay": data
            .get("autoplay_delay")
            .or_else(|| data.get("delay_ms"))
            .cloned()
            .unwrap_or(Value::Null),
        "position": data.get("position").cloned().unwrap_or(Value::Null),
        "realized_pnl": data.get("realized_pnl").cloned().unwrap_or(Value::Null),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_start_rejects_invalid_date_before_evaluating() {
        let error = validate_replay_date("2026-02-31").unwrap_err();
        assert_eq!(error.kind, ErrorKind::Validation);
    }

    #[test]
    fn replay_date_parser_returns_unix_millis() {
        assert_eq!(parse_replay_date_ms("1970-01-01").unwrap(), 0);
        assert_eq!(parse_replay_date_ms("1970-01-02").unwrap(), 86_400_000);
        assert!(parse_replay_date_ms("2024-02-29").is_ok());
        assert_eq!(
            parse_replay_date_ms("2023-02-29").unwrap_err().kind,
            ErrorKind::Validation
        );
    }

    #[test]
    fn replay_autoplay_rejects_invalid_speed_before_evaluating() {
        let error = validate_replay_autoplay_speed(123).unwrap_err();
        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(error.message.contains("Invalid replay autoplay delay"));
    }

    #[test]
    fn replay_autoplay_accepts_known_delays() {
        for delay in [0, 100, 143, 200, 300, 1000, 2000, 3000, 5000, 10000] {
            assert!(validate_replay_autoplay_speed(delay).is_ok());
        }
    }

    #[test]
    fn replay_trade_rejects_invalid_action_before_evaluating() {
        let error = validate_replay_trade_action("hold").unwrap_err();
        assert_eq!(error.kind, ErrorKind::Validation);
    }

    #[test]
    fn replay_trade_accepts_supported_actions() {
        for action in ["buy", "sell", "close"] {
            assert!(validate_replay_trade_action(action).is_ok());
        }
    }

    #[test]
    fn replay_action_normalization_removes_ok_flag() {
        let payload = normalize_replay_action(json!({
            "ok": true,
            "action": "step",
            "current_date": 1767225600000_i64,
            "source": "internal_api"
        }))
        .unwrap();

        assert_eq!(payload["action"], "step");
        assert_eq!(payload["operation"], "replay_step");
        assert_eq!(payload["source_category"], "desktop_backed_operation");
        assert_eq!(payload["requires_desktop"], true);
        assert_eq!(payload["non_mutating"], false);
        assert_eq!(payload["replay_context"]["current_date"], 1767225600000_i64);
        assert!(payload.get("ok").is_none());
    }

    #[test]
    fn replay_action_normalization_maps_errors() {
        let error = normalize_replay_operation(
            json!({
                "ok": false,
                "error_kind": "validation",
                "message": "Replay is not started. Use replay start first."
            }),
            "replay_step",
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        let details = error.details.unwrap();
        assert_eq!(details["operation"], "replay_step");
        assert_eq!(details["source_category"], "desktop_backed_operation");
        assert_eq!(details["non_mutating"], false);
    }

    #[test]
    fn replay_status_normalization_returns_practical_fields() {
        let payload = normalize_replay_status(json!({
            "is_replay_available": true,
            "is_replay_started": true,
            "is_autoplay_started": false,
            "replay_mode": "replay",
            "current_date": 1767225600000_i64,
            "autoplay_delay": 1000,
            "position": {"side": "long"},
            "realized_pnl": 12.5
        }))
        .unwrap();

        assert_eq!(payload["is_replay_available"], true);
        assert_eq!(payload["source"], "internal_api");
        assert_eq!(payload["source_category"], "desktop_backed_read");
        assert_eq!(payload["requires_desktop"], true);
        assert_eq!(payload["non_mutating"], true);
        assert_eq!(payload["replay_context"]["current_date"], 1767225600000_i64);
        assert_eq!(payload["position"]["side"], "long");
    }

    #[test]
    fn replay_status_normalization_rejects_unexpected_payload() {
        let error = normalize_replay_status(json!({"source": "internal_api"})).unwrap_err();
        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
    }
}
