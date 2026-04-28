use serde_json::{Value, json};
use tradingview_core::{AppError, ErrorKind};

use crate::normalize::strip_em;

const SYMBOL_SEARCH_URL: &str = "https://symbol-search.tradingview.com/symbol_search/v3/";

pub async fn symbol_search(query: &str) -> Result<Value, AppError> {
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
    Ok(normalize_symbol_search_response(query, &value))
}

pub(crate) fn normalize_symbol_search_response(query: &str, value: &Value) -> Value {
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
            json!({
                "symbol": symbol,
                "description": description,
                "exchange": exchange,
                "type": symbol_type,
                "full_name": full_name,
            })
        })
        .collect::<Vec<_>>();

    json!({
        "query": query,
        "source": "rest_api",
        "count": results.len(),
        "results": results,
    })
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
        assert_eq!(result["count"], 1);
        assert_eq!(result["results"][0]["symbol"], "AAPL");
        assert_eq!(result["results"][0]["description"], "Apple Inc");
        assert_eq!(result["results"][0]["full_name"], "NASDAQ:AAPL");
    }
}
