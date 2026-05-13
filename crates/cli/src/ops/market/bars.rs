use serde_json::Value;
use tradingview_core::AppError;

pub async fn bars(symbol: &str, timeframe: &str, count: usize) -> Result<Value, AppError> {
    tradingview_market::bars_symbol(symbol, timeframe, count).await
}
