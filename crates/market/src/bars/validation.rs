use std::time::Duration;

use serde_json::json;
use tradingview_core::{AppError, ErrorKind};

use super::types::{
    BarsDate, BarsRequest, BarsRequestMode, BarsSymbolResolution, DEFAULT_TIMEOUT_MS,
    MAX_DATE_RANGE_BAR_COUNT, MAX_RECENT_BAR_COUNT,
};

const DATE_RANGE_TIMEFRAMES: &[&str] = &["1", "5", "15", "30", "60", "1D", "1W", "1M"];

#[cfg(test)]
pub(super) fn validate_bars_request(
    symbol: &str,
    timeframe: &str,
    count: usize,
) -> Result<BarsRequest, AppError> {
    let symbol = symbol.trim();
    validate_bars_request_with_resolution(
        symbol,
        symbol,
        BarsSymbolResolution::input_exchange_qualified(symbol),
        timeframe,
        count,
    )
}

pub(super) fn validate_bars_request_with_resolution(
    requested_symbol: &str,
    resolved_symbol: &str,
    symbol_resolution: BarsSymbolResolution,
    timeframe: &str,
    count: usize,
) -> Result<BarsRequest, AppError> {
    validate_bars_request_inner(
        requested_symbol,
        resolved_symbol,
        symbol_resolution,
        timeframe,
        count,
        MAX_RECENT_BAR_COUNT,
        BarsRequestMode::RecentCount,
    )
}

#[cfg(test)]
pub(super) fn validate_bars_range_request(
    symbol: &str,
    timeframe: &str,
    from: &str,
    to: &str,
    count_cap: usize,
) -> Result<BarsRequest, AppError> {
    let symbol = symbol.trim();
    validate_bars_range_request_with_resolution(
        symbol,
        symbol,
        BarsSymbolResolution::input_exchange_qualified(symbol),
        timeframe,
        from,
        to,
        count_cap,
    )
}

pub(super) fn validate_bars_range_request_with_resolution(
    requested_symbol: &str,
    resolved_symbol: &str,
    symbol_resolution: BarsSymbolResolution,
    timeframe: &str,
    from: &str,
    to: &str,
    count_cap: usize,
) -> Result<BarsRequest, AppError> {
    let timeframe = normalize_timeframe(timeframe)?;
    if !DATE_RANGE_TIMEFRAMES.contains(&timeframe.as_str()) {
        return Err(AppError::new(
            ErrorKind::Validation,
            "bars date-range mode currently supports only 1-minute, 5-minute, 15-minute, 30-minute, 60-minute, daily, weekly, and monthly timeframes",
        )
        .with_details(json!({
            "requested_timeframe": timeframe,
            "supported_timeframes": DATE_RANGE_TIMEFRAMES,
        })));
    }

    let from = parse_bars_date(from, "from")?;
    let to = parse_bars_date(to, "to")?;
    if from.timestamp > to.timestamp {
        return Err(AppError::new(
            ErrorKind::Validation,
            "bars --from must be earlier than or equal to --to",
        )
        .with_details(json!({
            "from": from.date,
            "to": to.date,
            "from_time": from.timestamp,
            "to_time": to.timestamp,
        })));
    }

    validate_bars_request_inner(
        requested_symbol,
        resolved_symbol,
        symbol_resolution,
        &timeframe,
        count_cap,
        MAX_DATE_RANGE_BAR_COUNT,
        BarsRequestMode::DateRange { from, to },
    )
}

fn validate_bars_request_inner(
    requested_symbol: &str,
    resolved_symbol: &str,
    symbol_resolution: BarsSymbolResolution,
    timeframe: &str,
    count: usize,
    max_count: usize,
    mode: BarsRequestMode,
) -> Result<BarsRequest, AppError> {
    let requested_symbol = requested_symbol.trim();
    if requested_symbol.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "bars symbol must not be empty",
        ));
    }
    let resolved_symbol = resolved_symbol.trim();
    if resolved_symbol.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "bars resolved symbol must not be empty",
        ));
    }
    if !resolved_symbol.contains(':') {
        return Err(AppError::new(
            ErrorKind::Validation,
            "bars symbol must be exchange-qualified, for example NASDAQ:AAPL",
        )
        .with_details(json!({
            "requested_symbol": requested_symbol,
            "resolved_symbol": resolved_symbol,
            "expected_format": "EXCHANGE:SYMBOL",
        })));
    }

    let timeframe = normalize_timeframe(timeframe)?;
    if count == 0 || count > max_count {
        return Err(AppError::new(
            ErrorKind::Validation,
            format!("bars count must be between 1 and {max_count}"),
        )
        .with_details(json!({
            "minimum": 1,
            "maximum": max_count,
            "requested_count": count,
        })));
    }

    Ok(BarsRequest {
        requested_symbol: requested_symbol.to_string(),
        symbol: resolved_symbol.to_string(),
        symbol_resolution,
        timeframe,
        count,
        timeout: Duration::from_millis(DEFAULT_TIMEOUT_MS),
        mode,
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

fn parse_bars_date(input: &str, field: &str) -> Result<BarsDate, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(invalid_bars_date(input, field));
    }
    let mut parts = trimmed.split('-');
    let Some(year) = parts.next().and_then(|part| part.parse::<i32>().ok()) else {
        return Err(invalid_bars_date(input, field));
    };
    let Some(month) = parts.next().and_then(|part| part.parse::<u32>().ok()) else {
        return Err(invalid_bars_date(input, field));
    };
    let Some(day) = parts.next().and_then(|part| part.parse::<u32>().ok()) else {
        return Err(invalid_bars_date(input, field));
    };
    if parts.next().is_some() || !valid_ymd(year, month, day) {
        return Err(invalid_bars_date(input, field));
    }
    let days = days_from_civil(year, month, day);
    Ok(BarsDate {
        date: format!("{year:04}-{month:02}-{day:02}"),
        timestamp: days * 86_400,
    })
}

fn invalid_bars_date(input: &str, field: &str) -> AppError {
    AppError::new(
        ErrorKind::Validation,
        format!("Invalid bars --{field} date: {input}. Use YYYY-MM-DD."),
    )
    .with_details(json!({
        "field": field,
        "requested_date": input,
        "expected_format": "YYYY-MM-DD",
    }))
}

fn valid_ymd(year: i32, month: u32, day: u32) -> bool {
    if !(1..=12).contains(&month) || day == 0 {
        return false;
    }
    day <= days_in_month(year, month)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = year - i32::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let month = month as i32;
    let day = day as i32;
    let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    (era * 146_097 + doe - 719_468) as i64
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

    #[test]
    fn validate_range_accepts_intraday_daily_weekly_monthly_dates_and_count_cap() {
        for timeframe in ["1", "1m"] {
            let request = validate_bars_range_request(
                "NASDAQ:AAPL",
                timeframe,
                "2020-01-01",
                "2020-03-31",
                5000,
            )
            .unwrap();
            assert_eq!(request.timeframe, "1");
            assert_eq!(request.count, 5000);
            assert_eq!(request.request_mode_name(), "date_range");
        }

        let request =
            validate_bars_range_request("NASDAQ:AAPL", "5", "2020-01-01", "2020-03-31", 1000)
                .unwrap();
        assert_eq!(request.timeframe, "5");
        assert_eq!(request.count, 1000);
        assert_eq!(request.request_mode_name(), "date_range");

        let request =
            validate_bars_range_request("NASDAQ:AAPL", "15", "2020-01-01", "2020-03-31", 1000)
                .unwrap();
        assert_eq!(request.timeframe, "15");
        assert_eq!(request.count, 1000);
        assert_eq!(request.request_mode_name(), "date_range");

        let request =
            validate_bars_range_request("NASDAQ:AAPL", "30m", "2020-01-01", "2020-03-31", 1000)
                .unwrap();
        assert_eq!(request.timeframe, "30");
        assert_eq!(request.count, 1000);
        assert_eq!(request.request_mode_name(), "date_range");

        let request =
            validate_bars_range_request("NASDAQ:AAPL", "1h", "2020-01-01", "2020-03-31", 1000)
                .unwrap();
        assert_eq!(request.timeframe, "60");
        assert_eq!(request.count, 1000);
        assert_eq!(request.request_mode_name(), "date_range");

        let request =
            validate_bars_range_request("NASDAQ:AAPL", "1d", "2020-01-01", "2020-03-31", 5000)
                .unwrap();
        assert_eq!(request.timeframe, "1D");
        assert_eq!(request.count, 5000);
        assert_eq!(request.request_mode_name(), "date_range");
        assert_eq!(
            request.date_range_bounds(),
            Some((1_577_836_800, 1_585_699_200))
        );

        let request =
            validate_bars_range_request("NASDAQ:AAPL", "1w", "2020-01-01", "2020-03-31", 501)
                .unwrap();
        assert_eq!(request.timeframe, "1W");
        assert_eq!(request.count, 501);
        assert_eq!(request.request_mode_name(), "date_range");

        let request =
            validate_bars_range_request("NASDAQ:AAPL", "1M", "2020-01-01", "2020-03-31", 500)
                .unwrap();
        assert_eq!(request.timeframe, "1M");
        assert_eq!(request.request_mode_name(), "date_range");

        let err =
            validate_bars_range_request("NASDAQ:AAPL", "1D", "2020-01-01", "2020-03-31", 5001)
                .unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);
        assert_eq!(
            err.details
                .as_ref()
                .and_then(|details| details.get("maximum"))
                .and_then(|value| value.as_u64()),
            Some(5000)
        );
    }

    #[test]
    fn validate_range_rejects_invalid_dates_and_unsupported_intraday_timeframes() {
        let err = validate_bars_range_request("NASDAQ:AAPL", "1D", "2023-02-29", "2023-03-01", 500)
            .unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);

        let err = validate_bars_range_request("NASDAQ:AAPL", "1D", "2020-03-31", "2020-01-01", 500)
            .unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);

        for timeframe in ["3", "45", "120", "180", "240"] {
            let err = validate_bars_range_request(
                "NASDAQ:AAPL",
                timeframe,
                "2020-01-01",
                "2020-03-31",
                500,
            )
            .unwrap_err();
            assert_eq!(err.kind, ErrorKind::Validation);
            assert_eq!(
                err.details
                    .as_ref()
                    .and_then(|details| details.get("supported_timeframes")),
                Some(&serde_json::json!([
                    "1", "5", "15", "30", "60", "1D", "1W", "1M"
                ]))
            );
        }
    }
}
