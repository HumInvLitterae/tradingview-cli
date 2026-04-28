use serde_json::Value;

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::AppError;

use super::payload::normalize_alert_list_payload;

pub async fn alert_list(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            r#"
            (async function() {
                try {
                    const response = await fetch('https://pricealerts.tradingview.com/list_alerts', {
                        credentials: 'include',
                        headers: {
                            'accept': 'application/json'
                        }
                    });

                    if (!response.ok) {
                        return {
                            alert_count: 0,
                            source: 'internal_api',
                            alerts: [],
                            error: 'HTTP ' + response.status + ': ' + response.statusText
                        };
                    }

                    const data = await response.json();
                    const rows = Array.isArray(data.r) ? data.r : [];
                    const alerts = rows.map(function(alert) {
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

                    return {
                        alert_count: alerts.length,
                        source: 'internal_api',
                        alerts: alerts
                    };
                } catch (error) {
                    return {
                        alert_count: 0,
                        source: 'internal_api',
                        alerts: [],
                        error: error && error.message ? error.message : String(error)
                    };
                }
            })()
            "#,
            true,
        )
        .await
        .map(normalize_alert_list_payload)
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
    async fn alert_list_preserves_api_error_payload() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "alert_count": 0,
            "source": "internal_api",
            "alerts": [],
            "error": "HTTP 403: Forbidden"
        })]));

        let data = alert_list(&mut runtime).await.unwrap();

        assert_eq!(data["alert_count"], 0);
        assert_eq!(data["alerts"].as_array().unwrap().len(), 0);
        assert_eq!(data["error"], "HTTP 403: Forbidden");
    }

    #[tokio::test]
    async fn alert_list_defaults_malformed_payload_to_empty_list() {
        let mut runtime = FakeRuntime::new(VecDeque::from([json!({
            "source": "internal_api"
        })]));

        let data = alert_list(&mut runtime).await.unwrap();

        assert_eq!(data["alert_count"], 0);
        assert_eq!(data["source"], "internal_api");
        assert_eq!(data["alerts"].as_array().unwrap().len(), 0);
    }
}
