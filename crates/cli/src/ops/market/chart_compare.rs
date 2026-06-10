use serde_json::{Map, Value, json};

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::AppError;

use super::quote;

const CHART_COMPARE_CONTRACT_VERSION: &str = "chart_compare.v1";

pub async fn chart_compare(
    runtime: &mut impl RuntimeEvaluator,
    symbols: Vec<String>,
) -> Result<Value, AppError> {
    let requested_symbols: Vec<String> = symbols
        .into_iter()
        .map(|symbol| symbol.trim().to_string())
        .collect();
    let chart_context_before = chart_context_from_quote(&quote(runtime, None).await?);

    let mut items = Vec::new();
    let mut stopped = false;
    for (requested_index, requested_symbol) in requested_symbols.iter().enumerate() {
        match quote(runtime, Some(requested_symbol)).await {
            Ok(chart_quote) => {
                items.push(json!({
                    "requested_index": requested_index,
                    "requested_symbol": requested_symbol,
                    "status": "ok",
                    "observed_symbol": chart_quote.get("observed_symbol").cloned().unwrap_or(Value::Null),
                    "switch_performed": chart_quote.get("switch_performed").cloned().unwrap_or(Value::Null),
                    "restored": chart_quote.get("restored").cloned().unwrap_or(Value::Null),
                    "chart_quote": chart_quote,
                }));
            }
            Err(err) => {
                let restored = sanitized_restore_status(err.details.as_ref());
                items.push(json!({
                    "requested_index": requested_index,
                    "requested_symbol": requested_symbol,
                    "status": "error",
                    "restored": restored,
                    "failure_details": sanitized_error_details(&err),
                }));
                stopped = true;
                break;
            }
        }
    }

    let chart_context_after = match quote(runtime, None).await {
        Ok(after) => chart_context_from_quote(&after),
        Err(err) => json!({
            "available": false,
            "failure_details": sanitized_error_details(&err),
        }),
    };
    let summary = chart_compare_summary(&requested_symbols, &items, stopped, &chart_context_after);

    Ok(json!({
        "contract_version": CHART_COMPARE_CONTRACT_VERSION,
        "operation": "chart_compare",
        "source": "chart_api",
        "source_category": "desktop_backed_operation",
        "requires_desktop": true,
        "non_mutating": false,
        "restore_policy": "restore_original_after_each_symbol",
        "requested_symbols": requested_symbols,
        "chart_context_before": chart_context_before,
        "chart_context_after": chart_context_after,
        "items": items,
        "summary": summary,
    }))
}

fn chart_compare_summary(
    requested_symbols: &[String],
    items: &[Value],
    stopped: bool,
    chart_context_after: &Value,
) -> Value {
    let ok_count = items
        .iter()
        .filter(|item| item.get("status").and_then(Value::as_str) == Some("ok"))
        .count();
    let error_count = items
        .iter()
        .filter(|item| item.get("status").and_then(Value::as_str) == Some("error"))
        .count();
    let restore_failures = items
        .iter()
        .filter(|item| item.get("restored").and_then(Value::as_bool) == Some(false))
        .count();
    let restore_status = if restore_failures > 0 {
        "restore_failed"
    } else if chart_context_after
        .get("available")
        .and_then(Value::as_bool)
        == Some(false)
    {
        "unknown"
    } else {
        "all_restored"
    };
    let end_reason = if stopped {
        "item_failed"
    } else {
        "all_symbols_read"
    };

    json!({
        "requested_count": requested_symbols.len(),
        "item_count": items.len(),
        "ok_count": ok_count,
        "error_count": error_count,
        "restore_failure_count": restore_failures,
        "restore_status": restore_status,
        "end_reason": end_reason,
        "completed": !stopped && items.len() == requested_symbols.len(),
    })
}

fn chart_context_from_quote(quote: &Value) -> Value {
    json!({
        "available": true,
        "symbol": quote.get("symbol").cloned().unwrap_or(Value::Null),
        "chart_symbol": quote.get("chart_symbol").cloned().unwrap_or_else(|| {
            quote.get("symbol").cloned().unwrap_or(Value::Null)
        }),
        "observed_symbol": quote.get("observed_symbol").cloned().unwrap_or_else(|| {
            quote.get("symbol").cloned().unwrap_or(Value::Null)
        }),
        "exchange": quote.get("exchange").cloned().unwrap_or(Value::Null),
        "description": quote.get("description").cloned().unwrap_or(Value::Null),
        "type": quote.get("type").cloned().unwrap_or(Value::Null),
        "time": quote.get("time").cloned().unwrap_or(Value::Null),
        "bar_index": quote.get("bar_index").cloned().unwrap_or(Value::Null),
        "source": quote.get("source").cloned().unwrap_or_else(|| json!("chart_api")),
        "source_category": quote.get("source_category").cloned().unwrap_or_else(|| {
            json!("desktop_backed_read")
        }),
        "requires_desktop": true,
        "non_mutating": true,
    })
}

fn sanitized_restore_status(details: Option<&Value>) -> Value {
    details
        .and_then(|details| details.get("restored"))
        .cloned()
        .unwrap_or(Value::Null)
}

fn sanitized_error_details(err: &AppError) -> Value {
    let mut details = match err.details.as_ref() {
        Some(Value::Object(map)) => sanitize_map(map),
        Some(_) | None => Map::new(),
    };
    details.insert("kind".to_string(), json!(err.kind));
    details.insert("message".to_string(), json!(err.message));
    details
        .entry("source".to_string())
        .or_insert_with(|| json!("chart_api"));
    details
        .entry("source_category".to_string())
        .or_insert_with(|| json!("desktop_backed_operation"));
    details
        .entry("requires_desktop".to_string())
        .or_insert_with(|| json!(true));
    details
        .entry("non_mutating".to_string())
        .or_insert_with(|| json!(false));
    Value::Object(details)
}

fn sanitize_map(map: &Map<String, Value>) -> Map<String, Value> {
    map.iter()
        .filter_map(|(key, value)| {
            if is_private_detail_key(key) {
                return None;
            }
            Some((key.clone(), sanitize_value(value)))
        })
        .collect()
}

fn sanitize_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(sanitize_map(map)),
        Value::Array(values) => Value::Array(values.iter().map(sanitize_value).collect()),
        _ => value.clone(),
    }
}

fn is_private_detail_key(key: &str) -> bool {
    matches!(
        key,
        "raw"
            | "raw_payload"
            | "raw_payloads"
            | "raw_dom"
            | "target_id"
            | "session_id"
            | "cookie"
            | "authorization"
            | "credential"
            | "credentials"
            | "local_path"
            | "absolute_path"
            | "account_local_metadata"
    )
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tradingview_core::ErrorKind;

    use crate::ops::test_support::FakeRuntime;

    use super::*;

    fn quote_payload(symbol: &str, time: i64, last: f64) -> Value {
        json!({
            "symbol": symbol,
            "chart_symbol": symbol,
            "time": time,
            "bar_index": time,
            "open": last - 1.0,
            "high": last + 1.0,
            "low": last - 2.0,
            "close": last,
            "last": last,
            "volume": (last * 100.0) as i64,
        })
    }

    #[tokio::test]
    async fn chart_compare_wraps_ordered_chart_quote_items() {
        let mut runtime = FakeRuntime::new([
            quote_payload("NASDAQ:AAPL", 1, 10.0),
            quote_payload("NASDAQ:AAPL", 1, 10.0),
            json!({"requested_symbol": "NASDAQ:MSFT", "observed_symbol": "NASDAQ:MSFT"}),
            quote_payload("NASDAQ:MSFT", 2, 42.0),
            quote_payload("NASDAQ:MSFT", 2, 42.0),
            json!({"requested_symbol": "NASDAQ:AAPL", "observed_symbol": "NASDAQ:AAPL"}),
            quote_payload("NASDAQ:AAPL", 1, 10.0),
            json!({"requested_symbol": "NASDAQ:NVDA", "observed_symbol": "NASDAQ:NVDA"}),
            quote_payload("NASDAQ:NVDA", 3, 100.0),
            quote_payload("NASDAQ:NVDA", 3, 100.0),
            json!({"requested_symbol": "NASDAQ:AAPL", "observed_symbol": "NASDAQ:AAPL"}),
            quote_payload("NASDAQ:AAPL", 1, 10.0),
        ]);

        let result = chart_compare(
            &mut runtime,
            vec!["NASDAQ:MSFT".to_string(), "NASDAQ:NVDA".to_string()],
        )
        .await
        .unwrap();

        assert_eq!(result["contract_version"], "chart_compare.v1");
        assert_eq!(result["operation"], "chart_compare");
        assert_eq!(result["source"], "chart_api");
        assert_eq!(result["source_category"], "desktop_backed_operation");
        assert_eq!(result["requires_desktop"], true);
        assert_eq!(result["non_mutating"], false);
        assert_eq!(result["requested_symbols"][0], "NASDAQ:MSFT");
        assert_eq!(result["requested_symbols"][1], "NASDAQ:NVDA");
        assert_eq!(result["chart_context_before"]["symbol"], "NASDAQ:AAPL");
        assert_eq!(result["chart_context_after"]["symbol"], "NASDAQ:AAPL");
        assert_eq!(result["items"][0]["requested_index"], 0);
        assert_eq!(result["items"][0]["status"], "ok");
        assert_eq!(
            result["items"][0]["chart_quote"]["requested_symbol"],
            "NASDAQ:MSFT"
        );
        assert_eq!(result["items"][0]["chart_quote"]["restored"], true);
        assert_eq!(result["items"][1]["requested_index"], 1);
        assert_eq!(result["items"][1]["status"], "ok");
        assert_eq!(result["summary"]["ok_count"], 2);
        assert_eq!(result["summary"]["error_count"], 0);
        assert_eq!(result["summary"]["restore_status"], "all_restored");
        assert_eq!(result["summary"]["end_reason"], "all_symbols_read");
        assert_eq!(result["summary"]["completed"], true);
    }

    #[tokio::test]
    async fn chart_compare_stops_on_item_error_and_sanitizes_failure_details() {
        let mut runtime = FakeRuntime::new([
            quote_payload("NASDAQ:AAPL", 1, 10.0),
            quote_payload("NASDAQ:AAPL", 1, 10.0),
            json!({"requested_symbol": "NASDAQ:MSFT", "observed_symbol": "NASDAQ:MSFT"}),
            quote_payload("NASDAQ:AAPL", 1, 10.0),
            quote_payload("NASDAQ:AAPL", 1, 10.0),
            quote_payload("NASDAQ:AAPL", 1, 10.0),
            quote_payload("NASDAQ:AAPL", 1, 10.0),
            json!({"requested_symbol": "NASDAQ:MSFT", "observed_symbol": "NASDAQ:MSFT"}),
            quote_payload("NASDAQ:AAPL", 1, 10.0),
            quote_payload("NASDAQ:AAPL", 1, 10.0),
            quote_payload("NASDAQ:AAPL", 1, 10.0),
            quote_payload("NASDAQ:AAPL", 1, 10.0),
            json!({"requested_symbol": "NASDAQ:AAPL", "observed_symbol": "NASDAQ:AAPL"}),
            quote_payload("NASDAQ:AAPL", 1, 10.0),
        ]);

        let result = chart_compare(
            &mut runtime,
            vec!["NASDAQ:MSFT".to_string(), "NASDAQ:NVDA".to_string()],
        )
        .await
        .unwrap();

        assert_eq!(result["items"].as_array().unwrap().len(), 1);
        assert_eq!(result["items"][0]["status"], "error");
        assert_eq!(
            result["items"][0]["failure_details"]["kind"],
            "internal_api_unavailable"
        );
        assert_eq!(result["items"][0]["failure_details"]["source"], "chart_api");
        assert!(result["items"][0]["failure_details"].get("raw").is_none());
        assert_eq!(result["summary"]["ok_count"], 0);
        assert_eq!(result["summary"]["error_count"], 1);
        assert_eq!(result["summary"]["end_reason"], "item_failed");
        assert_eq!(result["summary"]["completed"], false);
    }

    #[test]
    fn sanitizer_removes_nested_private_values() {
        let err = AppError::new(ErrorKind::InternalApiUnavailable, "failed").with_details(json!({
            "source": "chart_api",
            "raw": {"secret": true},
            "nested": {"target_id": "abc", "safe": true},
            "array": [{"raw_payload": "hidden", "safe": 1}],
        }));

        let sanitized = sanitized_error_details(&err);

        assert!(sanitized.get("raw").is_none());
        assert!(sanitized["nested"].get("target_id").is_none());
        assert_eq!(sanitized["nested"]["safe"], true);
        assert!(sanitized["array"][0].get("raw_payload").is_none());
        assert_eq!(sanitized["array"][0]["safe"], 1);
    }
}
