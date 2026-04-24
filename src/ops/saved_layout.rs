use serde_json::{Value, json};

use crate::{cdp::RuntimeEvaluator, error::AppError};

pub async fn saved_layout_list(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            r#"
            new Promise(function(resolve) {
                try {
                    var api = window.TradingViewApi;
                    if (!api || typeof api.getSavedCharts !== 'function') {
                        resolve({
                            layout_count: 0,
                            source: 'internal_api',
                            layouts: [],
                            error: 'getSavedCharts is unavailable'
                        });
                        return;
                    }

                    var settled = false;
                    function finish(payload) {
                        if (settled) return;
                        settled = true;
                        resolve(payload);
                    }

                    api.getSavedCharts(function(charts) {
                        if (!Array.isArray(charts)) {
                            finish({
                                layout_count: 0,
                                source: 'internal_api',
                                layouts: [],
                                error: 'getSavedCharts returned no data'
                            });
                            return;
                        }

                        var layouts = charts.map(function(chart) {
                            return {
                                id: chart.id || chart.chartId || null,
                                name: chart.name || chart.title || 'Untitled',
                                symbol: chart.symbol || null,
                                resolution: chart.resolution || null,
                                modified: chart.timestamp || chart.modified || null
                            };
                        });

                        finish({
                            layout_count: layouts.length,
                            source: 'internal_api',
                            layouts: layouts
                        });
                    });

                    setTimeout(function() {
                        finish({
                            layout_count: 0,
                            source: 'internal_api',
                            layouts: [],
                            error: 'getSavedCharts timed out'
                        });
                    }, 5000);
                } catch (error) {
                    resolve({
                        layout_count: 0,
                        source: 'internal_api',
                        layouts: [],
                        error: error && error.message ? error.message : String(error)
                    });
                }
            })
            "#,
            true,
        )
        .await
        .map(normalize_saved_layout_list_payload)
}

fn normalize_saved_layout_list_payload(data: Value) -> Value {
    let layouts = data
        .get("layouts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut payload = json!({
        "layout_count": data
            .get("layout_count")
            .and_then(Value::as_u64)
            .unwrap_or(layouts.len() as u64),
        "source": data
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or("internal_api"),
        "layouts": layouts,
    });

    if let Some(error) = data.get("error").cloned() {
        payload["error"] = error;
    }

    payload
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::test_support::FakeRuntime;
    use super::*;

    #[tokio::test]
    async fn saved_layout_list_returns_normalized_payload() {
        let payload = json!({
            "layout_count": 2,
            "source": "internal_api",
            "layouts": [
                {
                    "id": "chart-1",
                    "name": "Swing Layout",
                    "symbol": "NASDAQ:AAPL",
                    "resolution": "1D",
                    "modified": 1777000000
                },
                {
                    "id": "chart-2",
                    "name": "Intraday",
                    "symbol": null,
                    "resolution": "15",
                    "modified": null
                }
            ]
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);

        let result = saved_layout_list(&mut runtime).await.unwrap();

        assert_eq!(result, payload);
        assert!(runtime.evaluated[0].0.contains("getSavedCharts"));
        assert!(!runtime.evaluated[0].0.contains("loadChartFromServer"));
        assert!(runtime.evaluated[0].1);
    }

    #[tokio::test]
    async fn saved_layout_list_preserves_error_payload() {
        let mut runtime = FakeRuntime::new([json!({
            "layout_count": 0,
            "source": "internal_api",
            "layouts": [],
            "error": "getSavedCharts timed out"
        })]);

        let result = saved_layout_list(&mut runtime).await.unwrap();

        assert_eq!(result["layout_count"], 0);
        assert_eq!(result["source"], "internal_api");
        assert_eq!(result["layouts"].as_array().unwrap().len(), 0);
        assert_eq!(result["error"], "getSavedCharts timed out");
    }

    #[tokio::test]
    async fn saved_layout_list_defaults_malformed_payload_to_empty_list() {
        let mut runtime = FakeRuntime::new([json!({
            "source": "internal_api"
        })]);

        let result = saved_layout_list(&mut runtime).await.unwrap();

        assert_eq!(result["layout_count"], 0);
        assert_eq!(result["source"], "internal_api");
        assert_eq!(result["layouts"].as_array().unwrap().len(), 0);
    }
}
