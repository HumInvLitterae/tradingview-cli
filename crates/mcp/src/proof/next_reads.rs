//! Fixed, opt-in reads for qualifying the next public contracts.

use super::ProofOperation;
use crate::{Failure, Result, admission::Admission, auth::Auth, tools::Tool, transport};
use serde_json::{Value, json};

pub(super) async fn inspect(
    operation: ProofOperation,
    auth: &mut Auth,
    admission: &mut Admission,
) -> Result<Value> {
    auth.restore().await?;
    let token = auth.token().await?;
    if matches!(operation, ProofOperation::NextReadCatalog) {
        return transport::inspect_tools(
            &auth.http,
            token,
            &[
                (
                    Tool::TechnicalSnapshotProof,
                    json!({"symbol": "NASDAQ:AAPL", "interval": "1D"}),
                ),
                (
                    Tool::AlertHistory,
                    json!({"symbol": "NASDAQ:AAPL", "days": 7, "limit": 100}),
                ),
            ],
        )
        .await;
    }
    let (tool, arguments) = match operation {
        ProofOperation::TechnicalControlShape => (
            Tool::Symbol,
            json!({"symbol": "NASDAQ:AAPL", "columns": ["close", "volume", "market_cap_basic"]}),
        ),
        ProofOperation::AccountHistoryShape => {
            (Tool::AlertHistory, json!({"days": 7, "limit": 100}))
        }
        ProofOperation::AlertHistoryShape => (
            Tool::AlertHistory,
            json!({"symbol": "NASDAQ:AAPL", "days": 7, "limit": 100}),
        ),
        ProofOperation::TechnicalDailyShape
        | ProofOperation::TechnicalWeeklyShape
        | ProofOperation::TechnicalMonthlyShape
        | ProofOperation::TechnicalTwoHourShape => {
            let interval = match operation {
                ProofOperation::TechnicalWeeklyShape => "1W",
                ProofOperation::TechnicalMonthlyShape => "1M",
                ProofOperation::TechnicalTwoHourShape => "2h",
                _ => "1D",
            };
            (
                Tool::TechnicalSnapshotProof,
                json!({"symbol": "NASDAQ:AAPL", "interval": interval}),
            )
        }
        _ => return Err(Failure::UnsupportedCapability),
    };
    let mut responses = transport::call(
        &auth.http,
        token,
        tool,
        std::slice::from_ref(&arguments),
        Some(admission),
    )
    .await?;
    let value = transport::result_value(responses.pop().ok_or(Failure::InvalidResponse)??)?;
    Ok(observation(tool, &arguments, &value))
}

fn observation(tool: Tool, arguments: &Value, value: &Value) -> Value {
    let mut report = json!({
        "tool": tool.names()[0],
        "requested_interval": arguments.get("interval"),
        "provider_success": value.get("success").and_then(Value::as_bool),
        "shape": safe_shape(value, 0),
        "symbol_echo_matches": value.get("symbol").and_then(Value::as_str)
            .map(|symbol| symbol == "NASDAQ:AAPL"),
        "interval_echo_matches": value.get("interval").and_then(Value::as_str)
            .map(|interval| arguments.get("interval").and_then(Value::as_str) == Some(interval))
    });
    if tool == Tool::AlertHistory {
        report["event_field_types"] = history_fields(value);
    }
    if value.get("success") == Some(&Value::Bool(false)) {
        report["failure"] = json!(Failure::ProviderError);
        report["provider_error_hints"] = error_hints(value.get("error"));
    }
    report
}

// Only flat event-schema identifiers and primitive types, never values or
// nested maps whose keys could be account-local identifiers.
fn history_fields(value: &Value) -> Value {
    let Some(events) = value.get("events").and_then(Value::as_array) else {
        return Value::Null;
    };
    Value::Array(
        events
            .iter()
            .take(2)
            .map(|event| {
                let Some(fields) = event.as_object() else {
                    return json!("invalid_event");
                };
                let types = fields
                    .iter()
                    .filter(|(name, _)| {
                        name.len() <= 64
                            && name.starts_with(|c: char| c.is_ascii_alphabetic())
                            && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
                    })
                    .take(40)
                    .map(|(name, value)| {
                        let kind = match value {
                            Value::Object(_) => "object",
                            Value::Array(_) => "array",
                            Value::Number(_) => "number",
                            Value::String(_) => "string",
                            Value::Bool(_) => "boolean",
                            Value::Null => "null",
                        };
                        (name.clone(), json!(kind))
                    })
                    .collect::<serde_json::Map<_, _>>();
                let timestamp_pattern = event
                    .get("fired_at")
                    .and_then(Value::as_str)
                    .filter(|value| value.len() <= 64)
                    .map(|value| {
                        value
                            .chars()
                            .map(|c| {
                                if c.is_ascii_digit() {
                                    '#'
                                } else if "-:TtZz+. ".contains(c) {
                                    c
                                } else {
                                    '?'
                                }
                            })
                            .collect::<String>()
                    });
                json!({"fields": types, "fired_at_pattern": timestamp_pattern})
            })
            .collect(),
    )
}

// These are textual clues, not HTTP statuses or verified root causes.
// Never retain the provider's arbitrary error string or nested private values.
fn error_hints(error: Option<&Value>) -> Value {
    let text = error
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_ascii_lowercase();
    let groups: &[(&str, &[&str])] = &[
        ("rate_limit", &["429", "rate limit", "too many requests"]),
        // This is an error-text reference, not proof of a request made by this client.
        ("screener_endpoint", &["scanner.tradingview.com"]),
        (
            "authorization",
            &["401", "403", "unauthorized", "forbidden", "permission"],
        ),
        (
            "entitlement",
            &["subscription", "premium", "paid plan", "entitlement"],
        ),
        ("timeout", &["timeout", "timed out"]),
        (
            "symbol",
            &["invalid symbol", "unknown symbol", "symbol not found"],
        ),
        (
            "interval",
            &["unsupported interval", "invalid interval", "timeframe"],
        ),
        (
            "implementation",
            &[
                "typeerror",
                "nameerror",
                "keyerror",
                "attributeerror",
                "not defined",
                "unexpected keyword",
                "internal server",
            ],
        ),
        ("not_found", &["404", "not found"]),
    ];
    let matched: Vec<_> = groups
        .iter()
        .filter(|(_, needles)| needles.iter().any(|needle| text.contains(needle)))
        .map(|(name, _)| *name)
        .collect();
    json!({"textual_clues": matched, "root_cause": "unconfirmed"})
}

// Names are a closed observation vocabulary, not arbitrary provider/account keys.
// Unknown names and every scalar value are suppressed, including message bodies.
fn safe_shape(value: &Value, depth: usize) -> Value {
    if depth > 8 {
        return json!("nested");
    }
    match value {
        Value::Object(fields) => {
            let mut retained = serde_json::Map::new();
            for (name, value) in fields {
                if matches!(
                    name.as_str(),
                    "symbol"
                        | "close"
                        | "volume"
                        | "market_cap_basic"
                        | "result"
                        | "results"
                        | "technical_rating"
                        | "technical_ratings"
                        | "error"
                        | "error_message"
                        | "interval"
                        | "timeframe"
                        | "data"
                        | "success"
                        | "indicators"
                        | "oscillators"
                        | "moving_averages"
                        | "summary"
                        | "ratings"
                        | "recommendation"
                        | "Recommend.All"
                        | "Recommend.MA"
                        | "Recommend.Other"
                        | "RSI"
                        | "Stoch.K"
                        | "Stoch.D"
                        | "CCI20"
                        | "ADX"
                        | "ADX+DI"
                        | "ADX-DI"
                        | "MACD.macd"
                        | "MACD.signal"
                        | "Mom"
                        | "AO"
                        | "EMA10"
                        | "EMA20"
                        | "EMA30"
                        | "EMA50"
                        | "EMA100"
                        | "EMA200"
                        | "SMA10"
                        | "SMA20"
                        | "SMA30"
                        | "SMA50"
                        | "SMA100"
                        | "SMA200"
                        | "VWMA"
                        | "HullMA9"
                        | "name"
                        | "value"
                        | "timestamp"
                        | "time"
                        | "date"
                        | "updated_at"
                        | "as_of"
                        | "alerts"
                        | "events"
                        | "logs"
                        | "history"
                        | "alert_id"
                        | "id"
                        | "fire_time"
                        | "fire_timestamp"
                        | "created_at"
                        | "delivery_status"
                        | "webhook_sent"
                        | "fired_at"
                        | "triggered_at"
                        | "message"
                        | "webhook"
                        | "webhook_status"
                        | "webhook_delivery_status"
                        | "status"
                        | "total"
                        | "count"
                        | "limit"
                        | "days"
                        | "has_more"
                ) {
                    retained.insert(name.clone(), safe_shape(value, depth + 1));
                }
            }
            json!({"fields": retained, "unrecognized_field_count": fields.len() - retained.len()})
        }
        Value::Array(items) => json!({
            "array_length": items.len(),
            "sample_shapes": items.iter().take(2)
                .map(|item| safe_shape(item, depth + 1)).collect::<Vec<_>>()
        }),
        Value::Null => json!("null"),
        Value::Bool(_) => json!("boolean"),
        Value::Number(_) => json!("number"),
        Value::String(_) => json!("string"),
    }
}

pub(super) async fn verify_history_command(
    directory: &std::path::Path,
    worker: Option<&std::path::Path>,
    timeout_seconds: Option<u64>,
) -> Result<Value> {
    let worker = worker.ok_or(Failure::UnsupportedCapability)?;
    let request = tradingview_model::mcp_account::Request::alert_history("NASDAQ:AAPL", 7, 100)
        .map_err(|_| Failure::UnsupportedCapability)?;
    match crate::Client::with_paths(directory.to_owned(), worker.to_owned())
        .run_with_timeout(crate::Operation::Account(request), timeout_seconds)
        .await
    {
        Ok(data) => Ok(
            json!({"success": true, "contract": data["contract_version"],
            "returned_count": data["returned_count"], "coverage": data["coverage"]}),
        ),
        Err(error) => Ok(json!({"success": false, "error": error.details})),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_reads_history_schema_never_retains_event_values_or_nested_keys() {
        let schema = history_fields(&json!({"events": [{
            "alertId": 123456, "fired_at": "2026-01-02T03:04:05.123Z",
            "message": "private message", "details": {"private-key": "secret"},
            "123456": "opaque key", "private-key": "not a schema identifier"
        }]}));
        assert_eq!(schema[0]["fields"]["alertId"], "number");
        assert_eq!(schema[0]["fired_at_pattern"], "####-##-##T##:##:##.###Z");
        for forbidden in ["123456", "private", "secret", "2026", "opaque"] {
            assert!(!schema.to_string().contains(forbidden));
        }
    }

    #[test]
    fn next_reads_error_hints_do_not_retain_messages_or_claim_http_status() {
        let hints = error_hints(Some(&json!("429 rate limit for private-account")));
        assert_eq!(hints["textual_clues"], json!(["rate_limit"]));
        assert_eq!(hints["root_cause"], "unconfirmed");
        let upstream = error_hints(Some(&json!(
            "429 Client Error for https://scanner.tradingview.com/private?token=secret"
        )));
        assert_eq!(
            upstream["textual_clues"],
            json!(["rate_limit", "screener_endpoint"])
        );
        for private in ["https", "private", "token", "secret"] {
            assert!(!upstream.to_string().contains(private));
        }

        assert!(!hints.to_string().contains("private-account"));
        assert_eq!(
            error_hints(Some(&json!({"secret": "429"})))["textual_clues"],
            json!([])
        );
    }

    #[test]
    fn technical_control_preserves_shape_and_failure_without_values() {
        let arguments =
            json!({"symbol": "NASDAQ:AAPL", "columns": ["close", "volume", "market_cap_basic"]});
        Tool::Symbol.validate_arguments(&arguments).unwrap();
        let success = observation(
            Tool::Symbol,
            &arguments,
            &json!({
                "success": true, "symbol": "NASDAQ:AAPL", "data": {"close": 123.45, "volume": null}
            }),
        );
        assert_eq!(
            success["shape"]["fields"]["data"]["fields"]["close"],
            "number"
        );
        assert!(!success.to_string().contains("123.45"));
        let failure = observation(
            Tool::Symbol,
            &arguments,
            &json!({
                "success": false, "error": "429 https://scanner.tradingview.com/private"
            }),
        );
        assert_eq!(failure["failure"], "provider_error");
        assert!(!failure.to_string().contains("private"));
    }

    #[test]
    fn next_reads_provider_failure_is_not_successful_data() {
        let report = observation(
            Tool::TechnicalSnapshotProof,
            &json!({"symbol": "NASDAQ:AAPL", "interval": "1D"}),
            &json!({"success": false, "error": "private provider detail"}),
        );
        assert_eq!(report["provider_success"], false);
        assert_eq!(report["failure"], json!(Failure::ProviderError));
        assert_eq!(report["shape"]["fields"]["error"], "string");
        assert!(!report.to_string().contains("private provider detail"));
    }

    #[test]
    fn next_reads_shape_suppresses_values_and_unknown_keys() {
        let shape = safe_shape(
            &json!({
                "events": [{
                    "alert_id": 123456,
                    "message": "private message",
                    "webhook": "https://example.invalid/private",
                    "private-key": {"value": "secret"}
                }]
            }),
            0,
        );
        let row = &shape["fields"]["events"]["sample_shapes"][0];
        assert_eq!(row["fields"]["alert_id"], "number");
        assert_eq!(row["fields"]["message"], "string");
        assert_eq!(row["unrecognized_field_count"], 1);
        let encoded = shape.to_string();
        for forbidden in ["123456", "private", "secret", "https"] {
            assert!(!encoded.contains(forbidden));
        }
        assert_eq!(safe_shape(&json!([]), 0)["array_length"], 0);
    }

    #[test]
    fn next_reads_technical_scope_and_history_inputs_stay_validated() {
        for interval in ["1D", "1W", "1M", "2h"] {
            assert!(
                Tool::TechnicalSnapshotProof
                    .validate_arguments(&json!({"symbol": "NASDAQ:AAPL", "interval": interval}))
                    .is_ok()
            );
        }
        assert!(
            Tool::AlertHistory
                .validate_arguments(&json!({"symbol": "NASDAQ:AAPL", "days": 7, "limit": 100}))
                .is_ok()
        );
        assert!(
            Tool::AlertHistory
                .validate_arguments(&json!({"days": 7, "limit": 100}))
                .is_ok()
        );
        for (tool, arguments) in [
            (
                Tool::TechnicalSnapshotProof,
                json!({"symbol": "NASDAQ:OTHER", "interval": "1D"}),
            ),
            (
                Tool::TechnicalSnapshotProof,
                json!({"symbol": "NASDAQ:AAPL", "interval": "1m"}),
            ),
            (Tool::AlertHistory, json!({"days": 8, "limit": 100})),
            (
                Tool::AlertHistory,
                json!({"symbol": "NASDAQ:AAPL", "days": 0, "limit": 100}),
            ),
            (
                Tool::AlertHistory,
                json!({"symbol": "NASDAQ:AAPL", "days": 7, "limit": 2001}),
            ),
        ] {
            assert_eq!(
                tool.validate_arguments(&arguments),
                Err(Failure::UnsupportedCapability)
            );
        }
    }
}
