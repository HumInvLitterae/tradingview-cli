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
                    Tool::AlertHistoryProof,
                    json!({"symbol": "NASDAQ:AAPL", "days": 7, "limit": 100}),
                ),
            ],
        )
        .await;
    }
    let (tool, arguments) = match operation {
        ProofOperation::AlertHistoryShape => (
            Tool::AlertHistoryProof,
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
    Ok(json!({
        "tool": tool.names()[0],
        "requested_interval": arguments.get("interval"),
        "shape": safe_shape(&value, 0),
        "symbol_echo_matches": value.get("symbol").and_then(Value::as_str)
            .map(|symbol| symbol == "NASDAQ:AAPL"),
        "interval_echo_matches": value.get("interval").and_then(Value::as_str)
            .map(|interval| arguments.get("interval").and_then(Value::as_str) == Some(interval))
    }))
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn next_reads_proof_requests_cannot_expand_the_approved_scope() {
        for interval in ["1D", "1W", "1M", "2h"] {
            assert!(
                Tool::TechnicalSnapshotProof
                    .validate_arguments(&json!({"symbol": "NASDAQ:AAPL", "interval": interval}))
                    .is_ok()
            );
        }
        assert!(
            Tool::AlertHistoryProof
                .validate_arguments(&json!({"symbol": "NASDAQ:AAPL", "days": 7, "limit": 100}))
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
            (Tool::AlertHistoryProof, json!({"days": 7, "limit": 100})),
            (
                Tool::AlertHistoryProof,
                json!({"symbol": "NASDAQ:AAPL", "days": 8, "limit": 100}),
            ),
            (
                Tool::AlertHistoryProof,
                json!({"symbol": "NASDAQ:AAPL", "days": 7, "limit": 101}),
            ),
        ] {
            assert_eq!(
                tool.validate_arguments(&arguments),
                Err(Failure::UnsupportedCapability)
            );
        }
    }
}
