use serde_json::{Map, Value, json};
use tradingview_core::{AppError, ErrorKind};

use crate::{
    info::preferred_symbol_candidates,
    normalize::{bare_symbol, split_exchange_symbol},
    search::symbol_search,
    types::Fundamentals,
};

const FUNDAMENTALS_SCAN_URL: &str = "https://scanner.tradingview.com/america/scan";
const FUNDAMENTALS_SOURCE: &str = "scanner_fundamentals_rest";
const FUNDAMENTALS_MARKET: &str = "america";

const DEFAULT_FUNDAMENTAL_FIELDS: &[&str] = &[
    "name",
    "description",
    "exchange",
    "sector",
    "industry",
    "market_cap_basic",
    "price_earnings_ttm",
    "earnings_per_share_basic_ttm",
    "dividend_yield_recent",
    "earnings_release_next_date",
    "earnings_release_next_time",
    "earnings_release_date",
];

const SUPPORTED_FUNDAMENTAL_FIELDS: &[&str] = &[
    "name",
    "description",
    "exchange",
    "type",
    "subtype",
    "sector",
    "industry",
    "market_cap_basic",
    "price_earnings_ttm",
    "price_earnings_forward_fy",
    "earnings_per_share_basic_ttm",
    "earnings_per_share_basic_fq",
    "earnings_per_share_fq",
    "earnings_per_share_forecast_next_fq",
    "earnings_per_share_forecast_next_fy",
    "revenue_forecast_next_fq",
    "revenue_forecast_next_fy",
    "total_revenue_ttm",
    "total_revenue_fq",
    "net_income_ttm",
    "net_income_fq",
    "dividend_yield_recent",
    "dividends_yield_current",
    "dividend_ex_date_recent",
    "dividend_ex_date_upcoming",
    "dividend_payment_date_recent",
    "dividend_payment_date_upcoming",
    "earnings_release_next_date",
    "earnings_release_date",
    "earnings_release_next_time",
    "earnings_release_next_calendar_date",
    "earnings_release_calendar_date",
    "earnings_release_next_trading_date_fy",
    "earnings_release_trading_date_fy",
    "earnings_publication_type_next_fq",
];

pub async fn fundamentals_symbol(symbol: &str, fields: Vec<String>) -> Result<Value, AppError> {
    serde_json::to_value(fundamentals_symbol_typed(symbol, fields).await?)
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))
}

/// Reads scanner-backed fundamental fields without connecting to TradingView Desktop.
///
/// This is the typed Rust API. Use [`fundamentals_symbol`] only when preserving
/// the CLI-compatible JSON payload shape is required.
pub async fn fundamentals_symbol_typed(
    symbol: &str,
    fields: Vec<String>,
) -> Result<Fundamentals, AppError> {
    let requested_symbol = symbol.trim();
    if requested_symbol.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "fundamentals symbol must not be empty",
        ));
    }
    let fields = normalize_fundamental_fields(fields)?;
    let value = fundamentals_symbol_via_scanner(requested_symbol, &fields).await?;
    match normalize_fundamentals_response_typed(requested_symbol, &fields, &value) {
        Ok(payload) => Ok(payload),
        Err(err) if err.kind == ErrorKind::Validation => {
            Err(add_symbol_search_candidates(err, requested_symbol).await)
        }
        Err(err) => Err(err),
    }
}

fn normalize_fundamental_fields(fields: Vec<String>) -> Result<Vec<String>, AppError> {
    if fields.is_empty() {
        return Ok(DEFAULT_FUNDAMENTAL_FIELDS
            .iter()
            .map(|field| (*field).to_string())
            .collect());
    }

    let mut normalized = Vec::with_capacity(fields.len());
    for field in fields {
        let field = field.trim();
        if field.is_empty() {
            return Err(AppError::new(
                ErrorKind::Validation,
                "--field must not be empty",
            ));
        }
        let supported = SUPPORTED_FUNDAMENTAL_FIELDS
            .iter()
            .copied()
            .find(|candidate| *candidate == field)
            .ok_or_else(|| {
                AppError::new(
                    ErrorKind::Validation,
                    format!("Unsupported fundamentals field: {field}"),
                )
                .with_details(json!({ "supported_fields": SUPPORTED_FUNDAMENTAL_FIELDS }))
            })?;
        if !normalized.iter().any(|value| value == supported) {
            normalized.push(supported.to_string());
        }
    }

    Ok(normalized)
}

async fn fundamentals_symbol_via_scanner(
    symbol: &str,
    fields: &[String],
) -> Result<Value, AppError> {
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
        "columns": fields,
        "filter": filters,
        "range": [0, 2],
    });
    let response = reqwest::Client::new()
        .post(FUNDAMENTALS_SCAN_URL)
        .json(&body)
        .send()
        .await
        .map_err(|err| AppError::new(ErrorKind::Connection, err.to_string()))?;

    let status = response.status();
    if !status.is_success() {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!("TradingView scanner fundamentals API returned {status}"),
        ));
    }

    response
        .json::<Value>()
        .await
        .map_err(|err| AppError::new(ErrorKind::InternalApiUnavailable, err.to_string()))
}

#[cfg(test)]
fn normalize_fundamentals_response(
    requested_symbol: &str,
    fields: &[String],
    value: &Value,
) -> Result<Value, AppError> {
    serde_json::to_value(normalize_fundamentals_response_typed(
        requested_symbol,
        fields,
        value,
    )?)
    .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))
}

fn normalize_fundamentals_response_typed(
    requested_symbol: &str,
    fields: &[String],
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
        field_values: Value::Object(field_values),
        missing_fields,
        non_mutating: true,
    })
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
    fn normalize_fundamental_fields_uses_curated_defaults() {
        let fields = normalize_fundamental_fields(Vec::new()).unwrap();

        assert!(fields.contains(&"name".to_string()));
        assert!(fields.contains(&"price_earnings_ttm".to_string()));
        assert!(fields.contains(&"earnings_release_next_date".to_string()));
        assert!(fields.contains(&"earnings_release_next_time".to_string()));
    }

    #[test]
    fn normalize_fundamental_fields_rejects_unknown_field() {
        let error = normalize_fundamental_fields(vec!["banana".to_string()]).unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(
            error.details.as_ref().unwrap()["supported_fields"]
                .as_array()
                .unwrap()
                .contains(&json!("earnings_release_next_date"))
        );
    }

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
