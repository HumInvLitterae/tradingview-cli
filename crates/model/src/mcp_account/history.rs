//! Source-honest alert history; no free-text or webhook payload passthrough.

use super::{Request, invalid_response};
use serde_json::{Value, json};
use tradingview_core::AppError;

pub(super) fn normalize(
    request: &Request,
    value: &Value,
    received_ms: u64,
) -> Result<Value, AppError> {
    if value.get("success") != Some(&Value::Bool(true)) {
        return Err(invalid_response("history_success"));
    }
    let rows = value
        .get("events")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_response("history_events"))?;
    let limit = request.arguments["limit"].as_u64().unwrap();
    if rows.len() as u64 > limit {
        return Err(invalid_response("history_limit"));
    }
    if let Some(count) = value.get("count")
        && count.as_u64() != Some(rows.len() as u64)
    {
        return Err(invalid_response("history_count"));
    }
    if let Some(days) = value.get("days")
        && days != &request.arguments["days"]
    {
        return Err(invalid_response("history_days"));
    }
    let events = rows
        .iter()
        .map(|row| {
            let id = row
                .get("tv_alert_id")
                .and_then(Value::as_u64)
                .filter(|id| *id > 0 && *id <= i64::MAX as u64)
                .ok_or_else(|| invalid_response("history_alert_id"))?;
            let symbol = row
                .get("symbol")
                .and_then(Value::as_str)
                .ok_or_else(|| invalid_response("history_symbol"))?;
            if Some(symbol) != request.arguments["symbol"].as_str() {
                return Err(invalid_response("history_symbol_mismatch"));
            }
            let fired_at = row
                .get("fired_at")
                .and_then(Value::as_str)
                .and_then(crate::mcp_dates::utc_seconds_to_unix)
                .ok_or_else(|| invalid_response("history_fired_at"))?;
            // fire_id identifies the event, not the alert. bar_time is not fire time.
            // Only a null webhook was observed: never infer delivery from its presence.
            Ok(json!({
                "alert_id": id,
                "symbol": symbol,
                "fired_at_unix_seconds": fired_at,
                "webhook_delivery_status": null
            }))
        })
        .collect::<Result<Vec<_>, AppError>>()?;
    Ok(json!({
        "contract_version": "mcp_alert_history.v1",
        "source": "tradingview_mcp",
        "requested": request.arguments,
        "retrieved_at_unix_ms": received_ms,
        "events": events,
        "returned_count": rows.len(),
        "limit_reached": rows.len() as u64 == limit,
        "coverage": "unconfirmed"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response() -> Value {
        json!({"success": true, "days": 7, "count": 1, "events": [{
            "tv_alert_id": 12, "fire_id": 34, "symbol": "NASDAQ:EXAMPLE",
            "fired_at": "2000-02-29T12:34:56Z", "bar_time": "2000-02-28T00:00:00Z",
            "message": "private message", "name": null, "resolution": "1D",
            "webhook": {"url": "https://example.invalid/private", "status": "sent"}
        }]})
    }

    #[test]
    fn history_uses_alert_identity_and_fire_time_without_private_fields() {
        let request = Request::alert_history("NASDAQ:EXAMPLE", 7, 100).unwrap();
        let output = super::super::normalize(&request, response(), 1700000000000).unwrap();
        assert_eq!(output["events"][0]["alert_id"], 12);
        assert_eq!(output["events"][0]["fired_at_unix_seconds"], 951827696i64);
        assert!(output["events"][0]["webhook_delivery_status"].is_null());
        assert_eq!(output["requested"], request.arguments());
        assert_eq!(output["coverage"], "unconfirmed");
        for secret in ["private", "fire_id", "bar_time", "resolution", "sent"] {
            assert!(!output.to_string().contains(secret));
        }
    }

    #[test]
    fn history_distinguishes_empty_saturation_and_provider_failure() {
        let request = Request::alert_history("NASDAQ:EXAMPLE", 7, 1).unwrap();
        assert_eq!(
            normalize(&request, &response(), 0).unwrap()["limit_reached"],
            true
        );
        let empty = normalize(
            &request,
            &json!({"success": true, "events": [], "count": 0}),
            0,
        )
        .unwrap();
        assert_eq!(empty["returned_count"], 0);
        assert_eq!(empty["limit_reached"], false);
        assert_eq!(empty["coverage"], "unconfirmed");
        let error =
            super::super::normalize(&request, json!({"success": false, "error": "private"}), 0)
                .unwrap_err();
        assert_eq!(error.details.unwrap()["code"], "provider_error");
    }

    #[test]
    fn history_rejects_contradictions_instead_of_filling_missing_values() {
        let request = Request::alert_history("NASDAQ:EXAMPLE", 7, 100).unwrap();
        for (path, invalid) in [
            ("/count", json!(2)),
            ("/days", json!(8)),
            ("/events/0/tv_alert_id", Value::Null),
            ("/events/0/symbol", json!("NYSE:OTHER")),
            ("/events/0/fired_at", json!("2000-02-30T00:00:00Z")),
        ] {
            let mut value = response();
            *value.pointer_mut(path).unwrap() = invalid;
            assert!(normalize(&request, &value, 0).is_err(), "{path}");
        }
        let mut over_limit = response();
        over_limit["events"] = json!([response()["events"][0], response()["events"][0]]);
        assert!(
            normalize(
                &Request::alert_history("NASDAQ:EXAMPLE", 7, 1).unwrap(),
                &over_limit,
                0
            )
            .is_err()
        );
        for (symbol, days, limit) in [
            ("EXAMPLE", 7, 100),
            ("NASDAQ:EXAMPLE", 0, 100),
            ("NASDAQ:EXAMPLE", 7, 0),
            ("NASDAQ:EXAMPLE", 7, 2001),
        ] {
            assert!(Request::alert_history(symbol, days, limit).is_err());
        }
    }
}
