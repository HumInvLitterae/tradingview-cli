use serde_json::Value;
use tradingview_core::{AppError, ErrorKind};

use crate::{
    normalize::strip_em,
    types::{SymbolSearchResponse, SymbolSearchResult},
};

const SYMBOL_SEARCH_URL: &str = "https://symbol-search.tradingview.com/symbol_search/v3/";
const DESKTOP_FREE_READ_CATEGORY: &str = "desktop_free_read";

pub async fn symbol_search(query: &str) -> Result<Value, AppError> {
    serde_json::to_value(search_symbols_typed(query).await?)
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))
}

/// Searches TradingView symbols without connecting to TradingView Desktop.
///
/// This is the typed Rust API. Use [`symbol_search`] only when preserving the
/// CLI-compatible JSON payload shape is required.
pub async fn search_symbols_typed(query: &str) -> Result<SymbolSearchResponse, AppError> {
    let url = reqwest::Url::parse_with_params(
        SYMBOL_SEARCH_URL,
        &[
            ("text", query),
            ("hl", "1"),
            ("exchange", ""),
            ("lang", "en"),
            ("search_type", ""),
            ("domain", "production"),
        ],
    )
    .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))?;
    let response = reqwest::Client::new()
        .get(url)
        .header("Origin", "https://www.tradingview.com")
        .header("Referer", "https://www.tradingview.com/")
        .send()
        .await
        .map_err(|err| AppError::new(ErrorKind::Connection, err.to_string()))?;

    let status = response.status();
    if !status.is_success() {
        return Err(AppError::new(
            ErrorKind::Connection,
            format!("Symbol search API returned {status}"),
        ));
    }

    let value = response
        .json::<Value>()
        .await
        .map_err(|err| AppError::new(ErrorKind::InternalApiUnavailable, err.to_string()))?;
    Ok(normalize_symbol_search_response_typed(query, &value))
}

#[cfg(test)]
pub(crate) fn normalize_symbol_search_response(query: &str, value: &Value) -> Value {
    serde_json::to_value(normalize_symbol_search_response_typed(query, value))
        .expect("symbol search response should serialize")
}

pub(crate) fn normalize_symbol_search_response_typed(
    query: &str,
    value: &Value,
) -> SymbolSearchResponse {
    let rows = value
        .get("symbols")
        .and_then(Value::as_array)
        .or_else(|| value.as_array())
        .cloned()
        .unwrap_or_default();
    let results = rows
        .into_iter()
        .take(15)
        .map(|row| {
            let symbol = strip_em(
                row.get("symbol")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
            );
            let description = strip_em(
                row.get("description")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
            );
            let exchange = row
                .get("exchange")
                .or_else(|| row.get("prefix"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let symbol_type = row
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let full_name = if exchange.is_empty() {
                symbol.clone()
            } else {
                format!("{exchange}:{symbol}")
            };
            SymbolSearchResult {
                symbol,
                description,
                exchange,
                symbol_type,
                full_name,
            }
        })
        .collect::<Vec<_>>();

    SymbolSearchResponse {
        query: query.to_string(),
        source: "rest_api".to_string(),
        source_category: DESKTOP_FREE_READ_CATEGORY.to_string(),
        requires_desktop: false,
        non_mutating: true,
        count: results.len(),
        results,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn normalize_symbol_search_response_handles_object_and_em_tags() {
        let response = json!({
            "symbols": [{
                "symbol": "<em>AAPL</em>",
                "description": "Apple <em>Inc</em>",
                "exchange": "NASDAQ",
                "type": "stock"
            }]
        });

        let result = normalize_symbol_search_response("AAPL", &response);

        assert_eq!(result["query"], "AAPL");
        assert_eq!(result["source"], "rest_api");
        assert_eq!(result["source_category"], "desktop_free_read");
        assert_eq!(result["requires_desktop"], false);
        assert_eq!(result["non_mutating"], true);
        assert_eq!(result["count"], 1);
        assert_eq!(result["results"][0]["symbol"], "AAPL");
        assert_eq!(result["results"][0]["description"], "Apple Inc");
        assert_eq!(result["results"][0]["full_name"], "NASDAQ:AAPL");
    }

    #[test]
    fn normalize_symbol_search_response_typed_returns_results() {
        let response = json!({
            "symbols": [{
                "symbol": "<em>AAPL</em>",
                "description": "Apple <em>Inc</em>",
                "exchange": "NASDAQ",
                "type": "stock"
            }]
        });

        let result = normalize_symbol_search_response_typed("AAPL", &response);

        assert_eq!(result.query, "AAPL");
        assert_eq!(result.source, "rest_api");
        assert_eq!(result.source_category, "desktop_free_read");
        assert!(!result.requires_desktop);
        assert!(result.non_mutating);
        assert_eq!(result.count, 1);
        assert_eq!(result.results[0].symbol, "AAPL");
        assert_eq!(result.results[0].symbol_type, "stock");
    }
}
