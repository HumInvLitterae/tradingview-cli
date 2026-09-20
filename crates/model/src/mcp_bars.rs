//! I/O-free request and output contract for the explicit official MCP command.

use serde_json::{Value, json};
use tradingview_core::{AppError, ErrorKind};

#[derive(Clone, Debug)]
pub struct Request {
    symbol: String,
    timeframe: String,
    count: u32,
}

impl Request {
    pub fn new(symbol: &str, timeframe: &str, count: u32) -> Result<Self, AppError> {
        validate_symbol(symbol)?;
        if !matches!(
            timeframe,
            "1m" | "5m" | "15m" | "30m" | "1h" | "4h" | "1D" | "1W" | "1M"
        ) {
            return Err(unsupported("timeframe"));
        }
        if !(1..=5000).contains(&count) {
            return Err(error(
                ErrorKind::Validation,
                "invalid_request",
                "Count must be between 1 and 5000",
                "count",
            ));
        }
        Ok(Self {
            symbol: symbol.to_owned(),
            timeframe: timeframe.to_owned(),
            count,
        })
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn timeframe(&self) -> &str {
        &self.timeframe
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn interval(&self) -> &str {
        if self.timeframe == "1M" {
            "M"
        } else {
            &self.timeframe
        }
    }

    pub fn arguments(&self) -> Value {
        json!({
            "symbol": self.symbol,
            "interval": self.interval(),
            "count": self.count,
            "summary": false
        })
    }
}

pub(crate) fn validate_symbol(symbol: &str) -> Result<(), AppError> {
    let Some((exchange, ticker)) = symbol.split_once(':') else {
        return Err(error(
            ErrorKind::Validation,
            "invalid_request",
            "Use an exchange-qualified symbol",
            "symbol",
        ));
    };
    if exchange.is_empty()
        || ticker.is_empty()
        || symbol.len() > 128
        || !exchange
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        || !ticker
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._!-".contains(&b))
    {
        return Err(error(
            ErrorKind::Validation,
            "invalid_request",
            "Invalid exchange-qualified symbol",
            "symbol",
        ));
    }
    Ok(())
}

pub fn unsupported(feature: &str) -> AppError {
    let mut result = error(
        ErrorKind::Validation,
        "unsupported_capability",
        "This MCP operation does not support the requested capability",
        feature,
    );
    result.details.as_mut().unwrap()["feature"] = json!(feature);
    result
}

fn error(kind: ErrorKind, code: &str, message: &str, reason: &str) -> AppError {
    AppError::new(kind, message).with_details(json!({
        "contract_version": "mcp_error.v1",
        "source": "tradingview_mcp",
        "code": code,
        "reason": reason,
        "tool_attempts": 0
    }))
}

fn invalid(reason: &str) -> AppError {
    error(
        ErrorKind::InternalApiUnavailable,
        "invalid_response",
        "TradingView MCP response is invalid",
        reason,
    )
}

fn evidence(value: Value) -> Value {
    json!({
        "evidence": if value.is_null() { "unconfirmed" } else { "provider_response" },
        "value": value
    })
}

pub fn normalize(request: &Request, value: Value, received_ms: u64) -> Result<Value, AppError> {
    let object = value
        .as_object()
        .ok_or_else(|| invalid("unsupported_wrapper"))?;
    match object.get("success") {
        Some(Value::Bool(false)) => {
            return Err(error(
                ErrorKind::InternalApiUnavailable,
                "provider_error",
                "TradingView did not return a successful result",
                "provider_failure",
            ));
        }
        Some(Value::Bool(true)) | None => {}
        _ => return Err(invalid("invalid_success_flag")),
    }
    let symbol = optional_string(object.get("symbol"))?;
    let interval = optional_string(object.get("interval"))?;
    if symbol
        .as_deref()
        .is_some_and(|symbol| symbol != request.symbol())
    {
        return Err(invalid("symbol_mismatch"));
    }
    if interval
        .as_deref()
        .is_some_and(|interval| interval != request.interval())
    {
        return Err(invalid("interval_mismatch"));
    }
    let rows = object
        .get("bars")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid("missing_bars"))?;
    if rows.len() > request.count as usize {
        return Err(invalid("excess_rows"));
    }
    if let Some(count) = object.get("count")
        && count.as_u64() != Some(rows.len() as u64)
    {
        return Err(invalid("count_mismatch"));
    }
    let mut bars = Vec::with_capacity(rows.len());
    let mut missing = Vec::new();
    let mut first = None;
    let mut last = None;
    for (index, row) in rows.iter().enumerate() {
        let time = row
            .get("t")
            .and_then(Value::as_i64)
            .filter(|t| (0..=253402300799).contains(t))
            .ok_or_else(|| invalid("invalid_bar_time"))?;
        if last.is_some_and(|previous| time <= previous) {
            return Err(invalid("nonascending_time"));
        }
        let number = |field: &str| {
            row.get(field)
                .and_then(Value::as_f64)
                .filter(|v| v.is_finite())
                .ok_or_else(|| invalid("invalid_ohlcv"))
        };
        let (open, high, low, close) = (number("o")?, number("h")?, number("l")?, number("c")?);
        if high < open.max(close).max(low) || low > open.min(close) {
            return Err(invalid("invalid_ohlc_bounds"));
        }
        let volume = if row.get("v").is_none_or(Value::is_null) {
            missing.push(json!({"bar_index": index, "field": "volume"}));
            Value::Null
        } else {
            let v = number("v")?;
            if v < 0.0 {
                return Err(invalid("invalid_volume"));
            }
            row["v"].clone()
        };
        first.get_or_insert(time);
        last = Some(time);
        bars.push(json!({
            "time": time,
            "open": row["o"],
            "high": row["h"],
            "low": row["l"],
            "close": row["c"],
            "volume": volume,
            "finality": "unconfirmed"
        }));
    }
    let unknown = evidence(Value::Null);
    Ok(json!({
        "contract_version": "mcp_bars.v1",
        "source": "tradingview_mcp",
        "source_category": "desktop_free_read",
        "requires_desktop": false,
        "request": {
            "symbol": request.symbol,
            "timeframe": request.timeframe,
            "mode": "recent_count",
            "count": request.count
        },
        "provider_observation": {
            "symbol": evidence(json!(symbol)),
            "interval": evidence(json!(interval)),
            "data_as_of": unknown,
            "delay_seconds": unknown,
            "adjustment": unknown,
            "session": unknown,
            "timestamp_semantics": unknown
        },
        "client_observation": {
            "received_at": received_at(received_ms)?,
            "bar_count": bars.len(),
            "returned_range": {"first_time": first, "last_time": last},
            "time_order": if bars.is_empty() { "unconfirmed" } else { "ascending" },
            "identity_match": if symbol.is_some() { "matched" } else { "unconfirmed" },
            "interval_match": if interval.is_some() { "matched" } else { "unconfirmed" },
            "count_status": if bars.is_empty() {
                "empty"
            } else if bars.len() < request.count as usize {
                "short"
            } else {
                "met"
            },
            "calendar_coverage": "unconfirmed",
            "missing_fields": missing
        },
        "transport": {"status": "succeeded", "tool_attempts": 1},
        "bars": bars
    }))
}

fn optional_string(value: Option<&Value>) -> Result<Option<String>, AppError> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) if !s.is_empty() => Ok(Some(s.clone())),
        _ => Err(invalid("invalid_identity_field")),
    }
}

pub(crate) fn received_at(ms: u64) -> Result<String, AppError> {
    if ms > 253402300799999 {
        return Err(invalid("invalid_client_time"));
    }
    let mut days = ms / 86400000;
    let mut year = 1970;
    let leap = |year: u64| {
        year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
    };
    loop {
        let n = if leap(year) { 366 } else { 365 };
        if days < n {
            break;
        }
        days -= n;
        year += 1;
    }
    let lengths = [
        31,
        if leap(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 0;
    while days >= lengths[month] {
        days -= lengths[month];
        month += 1;
    }
    let seconds = ms / 1000 % 86400;
    Ok(format!(
        "{year:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        month + 1,
        days + 1,
        seconds / 3600,
        seconds / 60 % 60,
        seconds % 60,
        ms % 1000
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> Request {
        Request::new("NASDAQ:EXAMPLE", "1D", 2).unwrap()
    }

    fn row() -> Value {
        json!({"t": 1704153600, "o": 10, "h": 12, "l": 9, "c": 11, "v": null})
    }

    #[test]
    fn short_and_empty_results_preserve_nulls_and_unknown_conditions() {
        let result = normalize(
            &request(),
            json!({
                "success": true,
                "symbol": "NASDAQ:EXAMPLE",
                "interval": "1D",
                "count": 1,
                "bars": [row()]
            }),
            1704369600123,
        )
        .unwrap();
        assert_eq!(result["client_observation"]["count_status"], "short");
        assert_eq!(
            result["client_observation"]["received_at"],
            "2024-01-04T12:00:00.123Z"
        );
        assert!(result["bars"][0]["volume"].is_null());
        assert_eq!(
            result["client_observation"]["missing_fields"][0]["bar_index"],
            0
        );
        assert_eq!(
            result["provider_observation"]["adjustment"]["evidence"],
            "unconfirmed"
        );
        let empty = normalize(&request(), json!({"bars": []}), 0).unwrap();
        assert_eq!(empty["client_observation"]["count_status"], "empty");
        assert_eq!(empty["client_observation"]["identity_match"], "unconfirmed");
        assert!(empty["client_observation"]["returned_range"]["first_time"].is_null());
    }

    #[test]
    fn wrapper_failures_mismatches_and_malformed_rows_never_become_success() {
        for value in [
            json!({"success": false, "bars": []}),
            json!({"success": "yes", "bars": []}),
            json!({"bars": [], "symbol": "OTHER:EXAMPLE"}),
            json!({"bars": [], "interval": "1W"}),
            json!({"bars": [], "count": 1}),
            json!({"bars": [row(), row()]}),
            json!({"bars": [{"t": 1, "o": 1, "h": 0, "l": 2, "c": 1}]}),
            json!({"bars": [{"t": 1, "o": 1, "h": 1, "l": 1, "c": 1, "v": -1}]}),
        ] {
            assert!(normalize(&request(), value, 0).is_err());
        }
    }

    #[test]
    fn intraday_intervals_preserve_identity_and_do_not_imply_calendar_coverage() {
        for timeframe in ["1m", "5m", "15m", "30m", "1h", "4h"] {
            let request = Request::new("NASDAQ:EXAMPLE", timeframe, 2).unwrap();
            assert_eq!(request.arguments()["interval"], timeframe);
            let mut later = row();
            later["t"] = json!(1704326400);
            let output = normalize(
                &request,
                json!({
                    "symbol": "NASDAQ:EXAMPLE",
                    "interval": timeframe,
                    "bars": [row(), later]
                }),
                1704369600000,
            )
            .unwrap();

            assert_eq!(output["request"]["timeframe"], timeframe);
            assert_eq!(output["client_observation"]["count_status"], "met");
            assert_eq!(
                output["client_observation"]["calendar_coverage"],
                "unconfirmed"
            );
            assert_eq!(output["bars"][1]["finality"], "unconfirmed");
            assert!(output["provider_observation"]["session"]["value"].is_null());
            assert!(normalize(&request, json!({"interval": "M", "bars": [row()]}), 0).is_err());
        }

        for unsupported in ["1", "5", "60", "2m", "2h", "1H", "M"] {
            assert!(Request::new("NASDAQ:EXAMPLE", unsupported, 2).is_err());
        }
    }

    #[test]
    fn validates_request_and_retains_real_zero_volume() {
        for (symbol, timeframe, count) in [
            ("AAPL", "1D", 1),
            ("NASDAQ:EXAMPLE", "2h", 1),
            ("NASDAQ:EXAMPLE", "1D", 0),
            ("NASDAQ:EXAMPLE", "1D", 5001),
            ("NASDAQ:EXAMPLE:OTHER", "1D", 1),
        ] {
            assert!(Request::new(symbol, timeframe, count).is_err());
        }
        assert_eq!(
            Request::new("NASDAQ:EXAMPLE", "1M", 20).unwrap().interval(),
            "M"
        );
        let mut value = row();
        value["v"] = json!(0);
        let output = normalize(&request(), json!({"bars": [value]}), 0).unwrap();
        assert_eq!(output["bars"][0]["volume"], 0);
        assert_eq!(output["client_observation"]["missing_fields"], json!([]));
        assert_eq!(
            received_at(951782400000).unwrap(),
            "2000-02-29T00:00:00.000Z"
        );
    }
}
