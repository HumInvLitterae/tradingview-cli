use serde_json::{Value, json};
use tradingview_core::{AppError, ErrorKind};

use crate::{
    info::preferred_symbol_candidates,
    normalize::{bare_symbol, split_exchange_symbol},
    search::symbol_search,
};

const QUOTE_SCAN_URL: &str = "https://scanner.tradingview.com/america/scan";
const QUOTE_SCAN_COLUMNS: &[&str] = &[
    "name",
    "description",
    "close",
    "open",
    "high",
    "low",
    "volume",
    "change",
    "exchange",
    "type",
    "subtype",
    "premarket_open",
    "premarket_high",
    "premarket_low",
    "premarket_close",
    "premarket_change",
    "premarket_change_abs",
    "premarket_gap",
    "premarket_volume",
    "postmarket_open",
    "postmarket_high",
    "postmarket_low",
    "postmarket_close",
    "postmarket_change",
    "postmarket_change_abs",
    "postmarket_volume",
];

pub async fn quote_symbol(symbol: &str) -> Result<Value, AppError> {
    let requested_symbol = symbol.trim();
    if requested_symbol.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "quote symbol must not be empty",
        ));
    }
    let value = quote_symbol_via_scanner(requested_symbol).await?;
    match normalize_scanner_quote_response(requested_symbol, &value) {
        Ok(quote) => Ok(quote),
        Err(err) if err.kind == ErrorKind::Validation => {
            Err(add_symbol_search_candidates(err, requested_symbol).await)
        }
        Err(err) => Err(err),
    }
}

async fn quote_symbol_via_scanner(symbol: &str) -> Result<Value, AppError> {
    let (exchange, name) = split_exchange_symbol(symbol);
    let mut filters = vec![json!({
        "left": "name",
        "operation": "equal",
        "right": name,
    })];
    if let Some(exchange) = exchange {
        filters.push(json!({
            "left": "exchange",
            "operation": "in_range",
            "right": [exchange],
        }));
    }
    let body = json!({
        "columns": QUOTE_SCAN_COLUMNS,
        "filter": filters,
        "range": [0, 2],
    });
    let response = reqwest::Client::new()
        .post(QUOTE_SCAN_URL)
        .json(&body)
        .send()
        .await
        .map_err(|err| AppError::new(ErrorKind::Connection, err.to_string()))?;

    let status = response.status();
    if !status.is_success() {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!("TradingView scanner quote API returned {status}"),
        ));
    }

    response
        .json::<Value>()
        .await
        .map_err(|err| AppError::new(ErrorKind::InternalApiUnavailable, err.to_string()))
}

fn normalize_scanner_quote_response(
    requested_symbol: &str,
    value: &Value,
) -> Result<Value, AppError> {
    let rows = value.get("data").and_then(Value::as_array).ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "TradingView scanner quote payload did not include data rows",
        )
    })?;
    if rows.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "TradingView scanner quote did not return the requested symbol",
        )
        .with_details(json!({
            "requested_symbol": requested_symbol,
            "resolution_error": "not_found",
            "source": "scanner_scan_rest",
        })));
    }
    if rows.len() > 1 {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Quote symbol is ambiguous; use EXCHANGE:SYMBOL",
        )
        .with_details(json!({
            "requested_symbol": requested_symbol,
            "candidate_count": rows.len(),
            "candidates": scanner_quote_candidates(rows),
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
                "TradingView scanner quote row did not include a symbol",
            )
            .with_details(row.clone())
        })?;
    if bare_symbol(full_symbol) != bare_symbol(requested_symbol) {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Scanner quote freshness check failed because the returned symbol did not match the requested symbol",
        )
        .with_details(json!({
            "requested_symbol": requested_symbol,
            "observed_symbol": full_symbol,
            "resolution_error": "symbol_mismatch",
            "freshness_check": {
                "kind": "requested_symbol_matches_observed_symbol",
                "passed": false,
            },
        })));
    }

    let values = row.get("d").and_then(Value::as_array).ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "TradingView scanner quote row did not include quote values",
        )
        .with_details(row.clone())
    })?;
    let field = |index: usize| values.get(index).cloned().unwrap_or(Value::Null);
    let close = field(2);
    Ok(json!({
        "symbol": full_symbol,
        "time": Value::Null,
        "last": close,
        "close": field(2),
        "open": field(3),
        "high": field(4),
        "low": field(5),
        "volume": field(6),
        "change": field(7),
        "description": field(1),
        "exchange": field(8),
        "type": field(9),
        "subtype": field(10),
        "extended_hours": {
            "premarket": {
                "open": field(11),
                "high": field(12),
                "low": field(13),
                "last": field(14),
                "close": field(14),
                "change_percent": field(15),
                "change_abs": field(16),
                "gap_percent": field(17),
                "volume": field(18),
            },
            "postmarket": {
                "open": field(19),
                "high": field(20),
                "low": field(21),
                "last": field(22),
                "close": field(22),
                "change_percent": field(23),
                "change_abs": field(24),
                "volume": field(25),
            },
        },
        "source": "scanner_scan_rest",
        "non_mutating": true,
        "requested_symbol": requested_symbol,
        "original_symbol": Value::Null,
        "observed_symbol": full_symbol,
        "switch_performed": false,
        "restored": true,
        "freshness_check": {
            "kind": "requested_symbol_matches_observed_symbol",
            "passed": true,
        },
    }))
}

async fn add_symbol_search_candidates(mut error: AppError, requested_symbol: &str) -> AppError {
    let Ok(search) = symbol_search(requested_symbol).await else {
        return error;
    };
    let candidates = preferred_symbol_candidates(requested_symbol, &search);
    if let Some(details) = error.details.as_mut().and_then(Value::as_object_mut) {
        details.insert("candidate_count".to_string(), json!(candidates.len()));
        details.insert("candidates".to_string(), Value::Array(candidates));
        details.insert("candidate_source".to_string(), json!("symbol_search_rest"));
    }
    error
}

fn scanner_quote_candidates(rows: &[Value]) -> Vec<Value> {
    rows.iter()
        .take(10)
        .filter_map(|row| {
            let symbol = row.get("s").and_then(Value::as_str)?;
            let values = row.get("d").and_then(Value::as_array);
            Some(json!({
                "full_name": symbol,
                "symbol": symbol.split(':').next_back().unwrap_or(symbol),
                "exchange": symbol.split(':').next().unwrap_or_default(),
                "description": values.and_then(|values| values.get(1)).cloned().unwrap_or(Value::Null),
                "type": values.and_then(|values| values.get(9)).cloned().unwrap_or(Value::Null),
            }))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn normalize_scanner_quote_response_returns_non_mutating_quote_payload() {
        let payload = json!({
            "totalCount": 1,
            "data": [{
                "s": "NASDAQ:AAPL",
                "d": [
                    "AAPL",
                    "Apple Inc.",
                    266.39,
                    266.09,
                    268.36,
                    265.07,
                    16427115,
                    -1.72,
                    "NASDAQ",
                    "stock",
                    "common",
                    269.81,
                    269.98,
                    267.85,
                    268.2,
                    -0.9271914594953977,
                    -2.509999999999991,
                    -0.3324590890620876,
                    174665,
                    null,
                    null,
                    null,
                    null,
                    null,
                    null,
                    null
                ]
            }]
        });

        let result = normalize_scanner_quote_response("AAPL", &payload).unwrap();

        assert_eq!(result["symbol"], "NASDAQ:AAPL");
        assert_eq!(result["last"], 266.39);
        assert_eq!(result["close"], 266.39);
        assert_eq!(result["description"], "Apple Inc.");
        assert_eq!(result["source"], "scanner_scan_rest");
        assert_eq!(result["non_mutating"], true);
        assert_eq!(result["switch_performed"], false);
        assert_eq!(result["restored"], true);
        assert_eq!(result["freshness_check"]["passed"], true);
        assert_eq!(result["extended_hours"]["premarket"]["open"], 269.81);
        assert_eq!(result["extended_hours"]["premarket"]["last"], 268.2);
        assert_eq!(result["extended_hours"]["premarket"]["close"], 268.2);
        assert_eq!(
            result["extended_hours"]["premarket"]["change_percent"],
            -0.9271914594953977
        );
        assert_eq!(
            result["extended_hours"]["premarket"]["change_abs"],
            -2.509999999999991
        );
        assert_eq!(
            result["extended_hours"]["premarket"]["gap_percent"],
            -0.3324590890620876
        );
        assert_eq!(result["extended_hours"]["premarket"]["volume"], 174665);
        assert_eq!(result["extended_hours"]["postmarket"]["last"], Value::Null);
        assert_eq!(
            result["extended_hours"]["postmarket"]["change_percent"],
            Value::Null
        );
    }

    #[test]
    fn normalize_scanner_quote_response_defaults_missing_extended_hours_to_null() {
        let payload = json!({
            "totalCount": 1,
            "data": [{
                "s": "NYSE:IONQ",
                "d": [
                    "IONQ",
                    "IonQ, Inc.",
                    43.1,
                    43.84,
                    44.26,
                    42.89,
                    12000000,
                    -1.68,
                    "NYSE",
                    "stock",
                    "common"
                ]
            }]
        });

        let result = normalize_scanner_quote_response("NYSE:IONQ", &payload).unwrap();

        assert_eq!(result["symbol"], "NYSE:IONQ");
        assert_eq!(result["last"], 43.1);
        assert_eq!(result["extended_hours"]["premarket"]["last"], Value::Null);
        assert_eq!(
            result["extended_hours"]["premarket"]["gap_percent"],
            Value::Null
        );
        assert_eq!(
            result["extended_hours"]["postmarket"]["volume"],
            Value::Null
        );
    }

    #[test]
    fn normalize_scanner_quote_response_rejects_ambiguous_symbol() {
        let payload = json!({
            "data": [
                {"s": "NASDAQ:ABC", "d": []},
                {"s": "NYSE:ABC", "d": []}
            ]
        });

        let error = normalize_scanner_quote_response("ABC", &payload).unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(error.message.contains("ambiguous"));
        assert_eq!(
            error.details.as_ref().unwrap()["resolution_error"],
            "ambiguous"
        );
        assert_eq!(
            error.details.as_ref().unwrap()["candidates"][0]["full_name"],
            "NASDAQ:ABC"
        );
    }

    #[test]
    fn normalize_scanner_quote_response_rejects_missing_symbol_as_validation() {
        let payload = json!({
            "data": []
        });

        let error = normalize_scanner_quote_response("NASDAQ:IONQ", &payload).unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert_eq!(
            error.details.as_ref().unwrap()["resolution_error"],
            "not_found"
        );
    }
}
