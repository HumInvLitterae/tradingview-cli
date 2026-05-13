use std::time::Duration;

use serde_json::json;
use tradingview_core::{AppError, ErrorKind};

use super::types::{BarsRequest, DEFAULT_TIMEOUT_MS, MAX_BAR_COUNT};

pub(super) fn validate_bars_request(
    symbol: &str,
    timeframe: &str,
    count: usize,
) -> Result<BarsRequest, AppError> {
    let symbol = symbol.trim();
    if symbol.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "bars symbol must not be empty",
        ));
    }
    if !symbol.contains(':') {
        return Err(AppError::new(
            ErrorKind::Validation,
            "bars symbol must be exchange-qualified, for example NASDAQ:AAPL",
        )
        .with_details(json!({
            "requested_symbol": symbol,
            "expected_format": "EXCHANGE:SYMBOL",
        })));
    }

    let timeframe = normalize_timeframe(timeframe)?;
    if count == 0 || count > MAX_BAR_COUNT {
        return Err(AppError::new(
            ErrorKind::Validation,
            format!("bars count must be between 1 and {MAX_BAR_COUNT}"),
        )
        .with_details(json!({
            "minimum": 1,
            "maximum": MAX_BAR_COUNT,
            "requested_count": count,
        })));
    }

    Ok(BarsRequest {
        symbol: symbol.to_string(),
        timeframe,
        count,
        timeout: Duration::from_millis(DEFAULT_TIMEOUT_MS),
    })
}

fn normalize_timeframe(timeframe: &str) -> Result<String, AppError> {
    let trimmed = timeframe.trim();
    if trimmed.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "bars timeframe must not be empty",
        ));
    }
    let normalized = match trimmed {
        "1m" => "1",
        "3m" => "3",
        "5m" => "5",
        "15m" => "15",
        "30m" => "30",
        "45m" => "45",
        "1h" => "60",
        "2h" => "120",
        "3h" => "180",
        "4h" => "240",
        "1d" | "D" => "1D",
        "1w" | "W" => "1W",
        "1M" | "M" => "1M",
        other => other,
    };
    if !is_supported_timeframe(normalized) {
        return Err(AppError::new(
            ErrorKind::Validation,
            "unsupported bars timeframe",
        )
        .with_details(json!({
            "requested_timeframe": timeframe,
            "supported_timeframes": ["1", "3", "5", "15", "30", "45", "60", "120", "180", "240", "1D", "1W", "1M"],
        })));
    }
    Ok(normalized.to_string())
}

fn is_supported_timeframe(timeframe: &str) -> bool {
    matches!(
        timeframe,
        "1" | "3" | "5" | "15" | "30" | "45" | "60" | "120" | "180" | "240" | "1D" | "1W" | "1M"
    )
}

#[cfg(test)]
mod tests {
    use tradingview_core::ErrorKind;

    use super::*;

    #[test]
    fn validate_rejects_bare_symbol_and_out_of_range_count() {
        let err = validate_bars_request("AAPL", "1D", 5).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);
        let err = validate_bars_request("NASDAQ:AAPL", "1D", 501).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);
    }

    #[test]
    fn validate_accepts_supported_timeframe_aliases() {
        let request = validate_bars_request("NASDAQ:AAPL", "1d", 5).unwrap();
        assert_eq!(request.timeframe, "1D");
    }
}
