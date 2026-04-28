use serde_json::{Value, json};
use tradingview_core::{AppError, ErrorKind};

const SYMBOL_SEARCH_URL: &str = "https://symbol-search.tradingview.com/symbol_search/v3/";
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
];

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

pub async fn symbol_info(symbol: &str) -> Result<Value, AppError> {
    let requested_symbol = symbol.trim();
    if requested_symbol.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "info symbol must not be empty",
        ));
    }
    let search = symbol_search(requested_symbol).await?;
    let target = resolve_symbol_search_match(requested_symbol, &search)?;
    Ok(symbol_info_from_search_result(requested_symbol, &target))
}

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

fn resolve_symbol_search_match(requested_symbol: &str, search: &Value) -> Result<Value, AppError> {
    let results = search
        .get("results")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let (requested_exchange, requested_name) = split_exchange_symbol(requested_symbol);
    let matches = results
        .iter()
        .filter(|candidate| {
            let symbol = candidate
                .get("symbol")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let exchange = candidate
                .get("exchange")
                .and_then(Value::as_str)
                .unwrap_or_default();
            bare_symbol(symbol) == requested_name
                && match requested_exchange.as_ref() {
                    Some(requested) => exchange.eq_ignore_ascii_case(requested),
                    None => true,
                }
        })
        .cloned()
        .collect::<Vec<_>>();

    if requested_exchange.is_none()
        && let Some(candidate) = matches.first()
    {
        return Ok(candidate.clone());
    }

    match matches.as_slice() {
        [candidate] => Ok(candidate.clone()),
        [] => {
            let candidates = preferred_symbol_candidates(requested_symbol, search);
            Err(AppError::new(
                ErrorKind::Validation,
                "Symbol was not found; use one of the candidate symbols",
            )
            .with_details(json!({
                "requested_symbol": requested_symbol,
                "resolution_error": "not_found",
                "source": "symbol_search_rest",
                "candidate_count": candidates.len(),
                "candidates": candidates,
            })))
        }
        _ => Err(AppError::new(
            ErrorKind::Validation,
            "Symbol is ambiguous; use EXCHANGE:SYMBOL",
        )
        .with_details(json!({
            "requested_symbol": requested_symbol,
            "resolution_error": "ambiguous",
            "source": "symbol_search_rest",
            "candidate_count": matches.len(),
            "candidates": matches.into_iter().take(10).collect::<Vec<_>>(),
        }))),
    }
}

fn symbol_info_from_search_result(requested_symbol: &str, target: &Value) -> Value {
    let symbol = target
        .get("symbol")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let exchange = target
        .get("exchange")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let full_name = target
        .get("full_name")
        .and_then(Value::as_str)
        .map(str::to_string)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| {
            if exchange.is_empty() {
                symbol.to_string()
            } else {
                format!("{exchange}:{symbol}")
            }
        });
    json!({
        "symbol": symbol,
        "full_name": full_name,
        "exchange": exchange,
        "description": target.get("description").cloned().unwrap_or(Value::Null),
        "type": target.get("type").cloned().unwrap_or(Value::Null),
        "pro_name": full_name,
        "typespecs": Value::Null,
        "resolution": Value::Null,
        "chart_type": Value::Null,
        "source": "symbol_search_rest",
        "non_mutating": true,
        "requested_symbol": requested_symbol,
    })
}

fn preferred_symbol_candidates(requested_symbol: &str, search: &Value) -> Vec<Value> {
    let results = search
        .get("results")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let (_, requested_name) = split_exchange_symbol(requested_symbol);
    let same_symbol = results
        .iter()
        .filter(|candidate| {
            candidate
                .get("symbol")
                .and_then(Value::as_str)
                .is_some_and(|symbol| bare_symbol(symbol) == requested_name)
        })
        .take(10)
        .cloned()
        .collect::<Vec<_>>();
    if same_symbol.is_empty() {
        results.into_iter().take(10).collect()
    } else {
        same_symbol
    }
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

fn split_exchange_symbol(symbol: &str) -> (Option<String>, String) {
    let symbol = symbol.trim();
    match symbol.split_once(':') {
        Some((exchange, name)) if !exchange.trim().is_empty() && !name.trim().is_empty() => (
            Some(exchange.trim().to_ascii_uppercase()),
            name.trim().to_ascii_uppercase(),
        ),
        _ => (None, symbol.to_ascii_uppercase()),
    }
}

fn bare_symbol(symbol: &str) -> String {
    symbol
        .split(':')
        .next_back()
        .unwrap_or(symbol)
        .to_ascii_uppercase()
}

fn normalize_symbol_search_response(query: &str, value: &Value) -> Value {
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

fn strip_em(value: &str) -> String {
    value.replace("<em>", "").replace("</em>", "")
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

    #[test]
    fn resolve_symbol_search_match_accepts_unique_bare_symbol() {
        let search = json!({
            "results": [
                {
                    "symbol": "IONQ",
                    "description": "IonQ, Inc.",
                    "exchange": "NYSE",
                    "type": "stock",
                    "full_name": "NYSE:IONQ"
                },
                {
                    "symbol": "IONX",
                    "description": "Defiance Daily Target 2X Long IONQ ETF",
                    "exchange": "NASDAQ",
                    "type": "fund",
                    "full_name": "NASDAQ:IONX"
                }
            ]
        });

        let result = resolve_symbol_search_match("IONQ", &search).unwrap();

        assert_eq!(result["full_name"], "NYSE:IONQ");
    }

    #[test]
    fn resolve_symbol_search_match_uses_first_search_result_for_bare_symbol() {
        let search = json!({
            "results": [
                {
                    "symbol": "IONQ",
                    "description": "IonQ, Inc.",
                    "exchange": "NYSE",
                    "type": "stock",
                    "full_name": "NYSE:IONQ"
                },
                {
                    "symbol": "IONQ",
                    "description": "Leverage Shares 3x Long IONQ ETP",
                    "exchange": "LSE",
                    "type": "fund",
                    "full_name": "LSE:IONQ"
                }
            ]
        });

        let result = resolve_symbol_search_match("IONQ", &search).unwrap();

        assert_eq!(result["full_name"], "NYSE:IONQ");
    }

    #[test]
    fn resolve_symbol_search_match_rejects_exchange_mismatch_with_candidates() {
        let search = json!({
            "results": [
                {
                    "symbol": "IONQ",
                    "description": "IonQ, Inc.",
                    "exchange": "NYSE",
                    "type": "stock",
                    "full_name": "NYSE:IONQ"
                }
            ]
        });

        let error = resolve_symbol_search_match("NASDAQ:IONQ", &search).unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        let details = error.details.as_ref().unwrap();
        assert_eq!(details["resolution_error"], "not_found");
        assert_eq!(details["candidates"][0]["full_name"], "NYSE:IONQ");
    }

    #[test]
    fn symbol_info_from_search_result_returns_current_info_shape() {
        let target = json!({
            "symbol": "IONQ",
            "description": "IonQ, Inc.",
            "exchange": "NYSE",
            "type": "stock",
            "full_name": "NYSE:IONQ"
        });

        let result = symbol_info_from_search_result("IONQ", &target);

        assert_eq!(result["symbol"], "IONQ");
        assert_eq!(result["full_name"], "NYSE:IONQ");
        assert_eq!(result["exchange"], "NYSE");
        assert_eq!(result["description"], "IonQ, Inc.");
        assert_eq!(result["type"], "stock");
        assert_eq!(result["pro_name"], "NYSE:IONQ");
        assert_eq!(result["source"], "symbol_search_rest");
        assert_eq!(result["non_mutating"], true);
        assert_eq!(result["requested_symbol"], "IONQ");
    }

    #[test]
    fn bare_symbol_compares_exchange_prefixed_inputs() {
        assert_eq!(bare_symbol("NASDAQ:AAPL"), bare_symbol("AAPL"));
        assert_eq!(bare_symbol("nyse:brk.b"), "BRK.B");
    }

    #[test]
    fn split_exchange_symbol_normalizes_optional_exchange() {
        assert_eq!(
            split_exchange_symbol(" nasdaq:aapl "),
            (Some("NASDAQ".to_string()), "AAPL".to_string())
        );
        assert_eq!(split_exchange_symbol("AAPL"), (None, "AAPL".to_string()));
    }

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
                    "common"
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
