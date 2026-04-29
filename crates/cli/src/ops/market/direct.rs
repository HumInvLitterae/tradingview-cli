use serde_json::Value;

use tradingview_core::AppError;

pub async fn symbol_search(query: &str) -> Result<Value, AppError> {
    tradingview_market::symbol_search(query).await
}

pub async fn symbol_info_direct(symbol: &str) -> Result<Value, AppError> {
    tradingview_market::symbol_info(symbol).await
}

pub async fn quote_symbol(symbol: &str) -> Result<Value, AppError> {
    tradingview_market::quote_symbol(symbol).await
}

pub async fn quote_symbols(symbols: Vec<String>) -> Result<Value, AppError> {
    tradingview_market::quote_symbols(symbols).await
}
