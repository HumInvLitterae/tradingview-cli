use serde_json::{Value, json};
use tradingview_core::{AppError, ErrorKind};

use crate::{
    normalize::{bare_symbol, split_exchange_symbol},
    search::symbol_search,
    types::SymbolInfo,
};

const DESKTOP_FREE_READ_CATEGORY: &str = "desktop_free_read";

pub async fn symbol_info(symbol: &str) -> Result<Value, AppError> {
    serde_json::to_value(symbol_info_typed(symbol).await?)
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))
}

/// Resolves Desktop-free symbol metadata through TradingView symbol search.
///
/// This is the typed Rust API. Use [`symbol_info`] only when preserving the
/// CLI-compatible JSON payload shape is required.
pub async fn symbol_info_typed(symbol: &str) -> Result<SymbolInfo, AppError> {
    let requested_symbol = symbol.trim();
    if requested_symbol.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "info symbol must not be empty",
        ));
    }
    let (_, requested_name) = split_exchange_symbol(requested_symbol);
    let search = symbol_search(&requested_name).await?;
    let target = resolve_symbol_search_match(requested_symbol, &search)?;
    Ok(symbol_info_from_search_result_typed(
        requested_symbol,
        &target,
    ))
}

pub(crate) fn resolve_symbol_search_match(
    requested_symbol: &str,
    search: &Value,
) -> Result<Value, AppError> {
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

#[cfg(test)]
pub(crate) fn symbol_info_from_search_result(requested_symbol: &str, target: &Value) -> Value {
    serde_json::to_value(symbol_info_from_search_result_typed(
        requested_symbol,
        target,
    ))
    .expect("symbol info response should serialize")
}

pub(crate) fn symbol_info_from_search_result_typed(
    requested_symbol: &str,
    target: &Value,
) -> SymbolInfo {
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
    SymbolInfo {
        symbol: symbol.to_string(),
        full_name: full_name.clone(),
        exchange: exchange.to_string(),
        description: target.get("description").cloned().unwrap_or(Value::Null),
        symbol_type: target.get("type").cloned().unwrap_or(Value::Null),
        pro_name: full_name,
        typespecs: Value::Null,
        resolution: Value::Null,
        chart_type: Value::Null,
        source: "symbol_search_rest".to_string(),
        source_category: DESKTOP_FREE_READ_CATEGORY.to_string(),
        requires_desktop: false,
        non_mutating: true,
        requested_symbol: requested_symbol.to_string(),
    }
}

pub(crate) fn preferred_symbol_candidates(requested_symbol: &str, search: &Value) -> Vec<Value> {
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

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
        assert_eq!(result["source_category"], "desktop_free_read");
        assert_eq!(result["requires_desktop"], false);
        assert_eq!(result["non_mutating"], true);
        assert_eq!(result["requested_symbol"], "IONQ");
    }

    #[test]
    fn symbol_info_from_search_result_typed_returns_current_info_shape() {
        let target = json!({
            "symbol": "IONQ",
            "description": "IonQ, Inc.",
            "exchange": "NYSE",
            "type": "stock",
            "full_name": "NYSE:IONQ"
        });

        let result = symbol_info_from_search_result_typed("IONQ", &target);

        assert_eq!(result.symbol, "IONQ");
        assert_eq!(result.full_name, "NYSE:IONQ");
        assert_eq!(result.exchange, "NYSE");
        assert_eq!(result.description, json!("IonQ, Inc."));
        assert_eq!(result.symbol_type, json!("stock"));
        assert_eq!(result.source, "symbol_search_rest");
        assert_eq!(result.source_category, "desktop_free_read");
        assert!(!result.requires_desktop);
        assert!(result.non_mutating);
    }
}
