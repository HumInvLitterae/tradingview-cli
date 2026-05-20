use serde_json::Value;
use serde_json::json;
use tradingview_core::{AppError, ErrorKind};

const DEFAULT_RECENT_COUNT: usize = 100;
const DEFAULT_RANGE_COUNT_CAP: usize = 500;

pub async fn bars(
    symbol: &str,
    timeframe: &str,
    count: Option<usize>,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<Value, AppError> {
    match (from, to) {
        (None, None) => {
            tradingview_market::bars_symbol(
                symbol,
                timeframe,
                count.unwrap_or(DEFAULT_RECENT_COUNT),
            )
            .await
        }
        (Some(from), Some(to)) => {
            tradingview_market::bars_symbol_range(
                symbol,
                timeframe,
                from,
                to,
                count.unwrap_or(DEFAULT_RANGE_COUNT_CAP),
            )
            .await
        }
        (from, to) => Err(AppError::new(
            ErrorKind::Validation,
            "bars date-range mode requires both --from and --to",
        )
        .with_details(json!({
            "from_provided": from.is_some(),
            "to_provided": to.is_some(),
        }))),
    }
}
