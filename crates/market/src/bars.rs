mod payload;
mod protocol;
mod transport;
mod types;
mod validation;

use serde_json::Value;
use tokio::time::Instant;
use tradingview_core::AppError;

use self::{
    payload::{bars_payload, no_bars_error},
    transport::fetch_bars_ws,
    validation::{validate_bars_range_request, validate_bars_request},
};

pub async fn bars_symbol(symbol: &str, timeframe: &str, count: usize) -> Result<Value, AppError> {
    let request = validate_bars_request(symbol, timeframe, count)?;
    bars_for_request(request).await
}

pub async fn bars_symbol_range(
    symbol: &str,
    timeframe: &str,
    from: &str,
    to: &str,
    count_cap: usize,
) -> Result<Value, AppError> {
    let request = validate_bars_range_request(symbol, timeframe, from, to, count_cap)?;
    bars_for_request(request).await
}

async fn bars_for_request(request: self::types::BarsRequest) -> Result<Value, AppError> {
    let started = Instant::now();
    let result = fetch_bars_ws(&request).await?;
    let elapsed_ms = started.elapsed().as_millis() as u64;

    if result.bars.is_empty() {
        return Err(no_bars_error(&request, &result, elapsed_ms));
    }

    Ok(bars_payload(&request, result, elapsed_ms))
}
