use serde_json::{Map, Value, json};
use tradingview_core::{AppError, ErrorKind};

use crate::{normalize::bare_symbol, types::Fundamentals};

const FUNDAMENTALS_SOURCE: &str = "scanner_fundamentals_rest";
const FUNDAMENTALS_MARKET: &str = "america";

#[cfg(test)]
pub(super) fn normalize_fundamentals_response(
    requested_symbol: &str,
    fields: &[String],
    value: &Value,
) -> Result<Value, AppError> {
    serde_json::to_value(normalize_fundamentals_response_typed(
        requested_symbol,
        fields,
        &[],
        value,
    )?)
    .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))
}

pub(super) fn normalize_fundamentals_response_typed(
    requested_symbol: &str,
    fields: &[String],
    groups: &[String],
    value: &Value,
) -> Result<Fundamentals, AppError> {
    let rows = value.get("data").and_then(Value::as_array).ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "TradingView scanner fundamentals payload did not include data rows",
        )
    })?;
    if rows.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "TradingView scanner fundamentals did not return the requested symbol",
        )
        .with_details(json!({
            "requested_symbol": requested_symbol,
            "resolution_error": "not_found",
            "source": FUNDAMENTALS_SOURCE,
        })));
    }
    if rows.len() > 1 {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Fundamentals symbol is ambiguous; use EXCHANGE:SYMBOL",
        )
        .with_details(json!({
            "requested_symbol": requested_symbol,
            "candidate_count": rows.len(),
            "candidates": scanner_fundamentals_candidates(rows),
            "resolution_error": "ambiguous",
        })));
    }

    let row = &rows[0];
    let full_symbol = row
        .get("s")
        .and_then(Value::as_str)
        .filter(|symbol| !symbol.trim().is_empty())
        .ok_or_else(|| {
            AppError::new(
                ErrorKind::InternalApiUnavailable,
                "TradingView scanner fundamentals row did not include a symbol",
            )
            .with_details(row.clone())
        })?;
    if bare_symbol(full_symbol) != bare_symbol(requested_symbol) {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Scanner fundamentals returned symbol did not match the requested symbol",
        )
        .with_details(json!({
            "requested_symbol": requested_symbol,
            "observed_symbol": full_symbol,
            "resolution_error": "symbol_mismatch",
            "source": FUNDAMENTALS_SOURCE,
        })));
    }

    let values = row.get("d").and_then(Value::as_array).ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "TradingView scanner fundamentals row did not include values",
        )
        .with_details(row.clone())
    })?;

    let mut field_values = Map::new();
    let mut missing_fields = Vec::new();
    for (index, field) in fields.iter().enumerate() {
        match values.get(index) {
            Some(value) => {
                field_values.insert(field.clone(), value.clone());
            }
            None => {
                field_values.insert(field.clone(), Value::Null);
                missing_fields.push(field.clone());
            }
        }
    }

    Ok(Fundamentals {
        source: FUNDAMENTALS_SOURCE.to_string(),
        requested_symbol: requested_symbol.to_string(),
        symbol: full_symbol.to_string(),
        observed_symbol: full_symbol.to_string(),
        market: FUNDAMENTALS_MARKET.to_string(),
        fields: fields.to_vec(),
        requested_groups: groups.to_vec(),
        field_values: Value::Object(field_values),
        missing_fields,
        non_mutating: true,
    })
}

fn scanner_fundamentals_candidates(rows: &[Value]) -> Vec<Value> {
    rows.iter()
        .take(10)
        .filter_map(|row| {
            let symbol = row.get("s").and_then(Value::as_str)?;
            Some(json!({
                "full_name": symbol,
                "symbol": symbol.split(':').next_back().unwrap_or(symbol),
                "exchange": symbol.split(':').next().unwrap_or_default(),
            }))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn normalize_fundamentals_response_returns_public_safe_payload() {
        let fields = vec![
            "name".to_string(),
            "description".to_string(),
            "price_earnings_ttm".to_string(),
            "earnings_release_next_date".to_string(),
            "earnings_release_next_time".to_string(),
        ];
        let payload = json!({
            "data": [{
                "s": "NASDAQ:AAPL",
                "d": ["AAPL", "Apple Inc.", 31.2, 1777852800, 1]
            }]
        });

        let result = normalize_fundamentals_response("AAPL", &fields, &payload).unwrap();

        assert_eq!(result["source"], "scanner_fundamentals_rest");
        assert_eq!(result["requested_symbol"], "AAPL");
        assert_eq!(result["symbol"], "NASDAQ:AAPL");
        assert_eq!(result["observed_symbol"], "NASDAQ:AAPL");
        assert_eq!(result["market"], "america");
        assert_eq!(result["fields"], json!(fields));
        assert!(result.get("requested_groups").is_none());
        assert_eq!(result["field_values"]["name"], "AAPL");
        assert_eq!(result["field_values"]["price_earnings_ttm"], 31.2);
        assert_eq!(
            result["field_values"]["earnings_release_next_date"],
            1777852800
        );
        assert_eq!(result["field_values"]["earnings_release_next_time"], 1);
        assert_eq!(result["missing_fields"], json!([]));
        assert_eq!(result["non_mutating"], true);
    }

    #[test]
    fn normalize_fundamentals_response_serializes_requested_groups_when_present() {
        let fields = vec!["earnings_release_next_date".to_string()];
        let groups = vec!["earnings".to_string()];
        let payload = json!({
            "data": [{
                "s": "NASDAQ:AAPL",
                "d": [1777852800]
            }]
        });

        let result = serde_json::to_value(
            normalize_fundamentals_response_typed("AAPL", &fields, &groups, &payload).unwrap(),
        )
        .unwrap();

        assert_eq!(result["requested_groups"], json!(["earnings"]));
        assert_eq!(result["fields"], json!(fields));
    }

    #[test]
    fn normalize_fundamentals_response_tracks_missing_value_slots() {
        let fields = vec!["name".to_string(), "earnings_release_next_date".to_string()];
        let payload = json!({
            "data": [{
                "s": "NYSE:IONQ",
                "d": ["IONQ"]
            }]
        });

        let result = normalize_fundamentals_response("NYSE:IONQ", &fields, &payload).unwrap();

        assert_eq!(
            result["field_values"]["earnings_release_next_date"],
            Value::Null
        );
        assert_eq!(
            result["missing_fields"],
            json!(["earnings_release_next_date"])
        );
    }

    #[test]
    fn normalize_fundamentals_response_rejects_ambiguous_symbol() {
        let fields = vec!["name".to_string()];
        let payload = json!({
            "data": [
                {"s": "NASDAQ:ABC", "d": ["ABC"]},
                {"s": "NYSE:ABC", "d": ["ABC"]}
            ]
        });

        let error = normalize_fundamentals_response("ABC", &fields, &payload).unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert_eq!(
            error.details.as_ref().unwrap()["resolution_error"],
            "ambiguous"
        );
    }

    #[test]
    fn normalize_fundamentals_response_rejects_symbol_mismatch() {
        let fields = vec!["name".to_string()];
        let payload = json!({
            "data": [{
                "s": "NASDAQ:MSFT",
                "d": ["MSFT"]
            }]
        });

        let error = normalize_fundamentals_response("NASDAQ:AAPL", &fields, &payload).unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert_eq!(
            error.details.as_ref().unwrap()["resolution_error"],
            "symbol_mismatch"
        );
    }
}
