use serde_json::Value;

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};

use super::{ALERT_LIST_READER, payload::normalize_alert_list_payload};

pub async fn alert_list(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    let expression = r#"
        (async function() {
            __ALERT_LIST_READER__
            const result = await __readAlertRows();
            if (!result.ok) return result;
            const alerts = result.alerts.map(function(alert) {
                return {
                    alert_id: alert.alert_id || alert.id || null,
                    symbol: alert.symbol || (alert.condition && alert.condition.symbol) || null,
                    type: alert.type || null,
                    message: alert.message || alert.description || '',
                    active: alert.active !== false,
                    condition: alert.condition || null,
                    resolution: alert.resolution || alert.interval || null,
                    created: alert.created || alert.create_time || null,
                    last_fired: alert.last_fired || alert.last_fire_time || null,
                    expiration: alert.expiration || alert.expire_time || null
                };
            });
            return { alert_count: alerts.length, source: 'internal_api', alerts };
        })()
    "#
    .replace("__ALERT_LIST_READER__", ALERT_LIST_READER);
    let data = runtime.evaluate(&expression, true).await?;
    let phase = if data.get("error").is_some() {
        if data.get("phase").and_then(Value::as_str) == Some("invalid_response") {
            Some("invalid_response")
        } else {
            Some("list_unavailable")
        }
    } else if !data
        .get("alerts")
        .and_then(Value::as_array)
        .is_some_and(|rows| rows.iter().all(Value::is_object))
    {
        Some("invalid_response")
    } else {
        None
    };
    if let Some(phase) = phase {
        return Err(
            AppError::new(ErrorKind::InternalApiUnavailable, "Alert list unavailable")
                .with_details(serde_json::json!({"source": "internal_api", "phase": phase})),
        );
    }
    Ok(normalize_alert_list_payload(data))
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use serde_json::json;

    use super::super::super::test_support::FakeRuntime;
    use super::*;

    #[tokio::test]
    async fn alert_list_returns_runtime_payload() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "alert_count": 1,
            "source": "internal_api",
            "alerts": [
                {
                    "alert_id": "alert-1",
                    "symbol": "NASDAQ:AAPL",
                    "type": "price",
                    "message": "Breakout",
                    "active": true,
                    "condition": { "operator": "greater" },
                    "resolution": "1D",
                    "created": 1777000000,
                    "last_fired": null,
                    "expiration": 1777600000
                }
            ]
        })]));

        let data = alert_list(&mut runtime).await.unwrap();

        assert_eq!(data["alert_count"], 1);
        assert_eq!(data["source"], "internal_api");
        assert_eq!(data["alerts"][0]["alert_id"], "alert-1");
        assert_eq!(data["alerts"][0]["symbol"], "NASDAQ:AAPL");
        assert!(runtime.evaluated[0].0.contains("list_alerts"));
        assert!(!runtime.evaluated[0].0.contains("content-type"));
        assert!(runtime.evaluated[0].1);
    }

    #[tokio::test]
    async fn alert_list_rejects_api_error_without_exposing_raw_text() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "alert_count": 0,
            "source": "internal_api",
            "alerts": [],
            "error": "HTTP 403: Forbidden"
        })]));

        let error = alert_list(&mut runtime).await.unwrap_err();
        assert_eq!(error.exit_code(), 3);
        assert_eq!(error.details.as_ref().unwrap()["phase"], "list_unavailable");
        assert!(!format!("{error:?}").contains("403"));
    }

    #[tokio::test]
    async fn alert_list_rejects_malformed_payload() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "source": "internal_api"
        })]));

        let error = alert_list(&mut runtime).await.unwrap_err();
        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.details.unwrap()["phase"], "invalid_response");
    }
}
