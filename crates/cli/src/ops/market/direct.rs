use serde_json::Value;

use tradingview_core::AppError;

pub async fn symbol_search(query: &str) -> Result<Value, AppError> {
    tradingview_market::symbol_search(query).await
}

pub async fn symbol_info_direct(symbol: &str) -> Result<Value, AppError> {
    tradingview_market::symbol_info(symbol).await
}

pub async fn fundamentals_symbol(
    symbol: &str,
    groups: Vec<String>,
    fields: Vec<String>,
) -> Result<Value, AppError> {
    tradingview_market::fundamentals_symbol_with_groups(symbol, groups, fields).await
}

pub async fn events_symbol(symbol: &str, event_type: &str) -> Result<Value, AppError> {
    tradingview_market::events_symbol(symbol, event_type).await
}

pub async fn snapshot_symbol(
    symbol: &str,
    groups: Vec<String>,
    fields: Vec<String>,
) -> Result<Value, AppError> {
    tradingview_market::snapshot_symbol(symbol, groups, fields).await
}

pub async fn compare_symbols(symbols: Vec<String>) -> Result<Value, AppError> {
    tradingview_market::compare_symbols(symbols).await
}

pub async fn quote_symbol(symbol: &str) -> Result<Value, AppError> {
    tradingview_market::quote_symbol(symbol).await
}

pub async fn quote_symbols(symbols: Vec<String>) -> Result<Value, AppError> {
    tradingview_market::quote_symbols(symbols).await
}
