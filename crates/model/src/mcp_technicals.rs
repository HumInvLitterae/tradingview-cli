//! Official single-timeframe snapshots; no local indicator calculation.

use serde_json::{Value, json};
use tradingview_core::{AppError, ErrorKind};

pub const TIMEFRAMES: &[&str] = &["1m", "5m", "15m", "30m", "1h", "2h", "4h", "1D", "1W", "1M"];
const GROUPS: &[(&str, &[&str])] = &[
    (
        "oscillators",
        &[
            "rsi",
            "stoch_k",
            "stoch_d",
            "cci20",
            "adx",
            "macd",
            "macd_signal",
            "momentum",
            "ao",
        ],
    ),
    (
        "moving_averages",
        &[
            "ema10", "ema20", "ema30", "ema50", "ema100", "ema200", "sma10", "sma20", "sma30",
            "sma50", "sma100", "sma200", "vwma", "hullma9",
        ],
    ),
];

#[derive(Clone, Debug)]
pub struct Request {
    symbol: String,
    timeframe: String,
}

impl Request {
    pub fn new(symbol: &str, timeframe: &str) -> Result<Self, AppError> {
        crate::mcp_bars::validate_symbol(symbol)?;
        if !TIMEFRAMES.contains(&timeframe) {
            return Err(crate::mcp_bars::unsupported("timeframe"));
        }
        Ok(Self {
            symbol: symbol.into(),
            timeframe: timeframe.into(),
        })
    }

    pub fn arguments(&self) -> Value {
        json!({"symbol": self.symbol, "interval": self.timeframe})
    }
}

fn invalid(reason: &str) -> AppError {
    AppError::new(
        ErrorKind::InternalApiUnavailable,
        "TradingView MCP response is invalid",
    )
    .with_details(
        json!({"contract_version": "mcp_error.v1", "source": "tradingview_mcp",
            "code": "invalid_response", "reason": reason}),
    )
}

fn scalar(value: Option<&Value>, numeric: bool) -> Result<Value, AppError> {
    let value = value.cloned().unwrap_or(Value::Null);
    let valid = if numeric {
        value.as_f64().is_some_and(f64::is_finite)
    } else {
        value.as_str().is_some_and(|s| !s.is_empty())
    };
    if value.is_null() || valid {
        Ok(value)
    } else {
        Err(invalid("invalid_technical_value"))
    }
}

fn evidence(value: Value) -> Value {
    json!({"evidence": if value.is_null() { "unconfirmed" } else { "provider_response" }, "value": value})
}

pub fn normalize(request: &Request, value: Value, received_ms: u64) -> Result<Value, AppError> {
    if value.get("success") == Some(&json!(false)) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "TradingView did not return a successful result",
        )
        .with_details(json!({
            "contract_version": "mcp_error.v1", "source": "tradingview_mcp",
            "code": "provider_error", "reason": "provider_failure",
            "provider_error_hints": crate::mcp_error::provider_error_hints(value.get("error"))
        })));
    }
    if value.get("success") != Some(&json!(true)) {
        return Err(invalid("invalid_success_flag"));
    }
    let data = value
        .get("data")
        .and_then(Value::as_object)
        .ok_or_else(|| invalid("missing_technical_data"))?;
    let symbol = scalar(data.get("symbol"), false)?;
    let interval = scalar(data.get("interval"), false)?;
    if !symbol.is_null() && symbol != request.symbol {
        return Err(invalid("symbol_mismatch"));
    }
    if !interval.is_null() && interval != request.timeframe {
        return Err(invalid("interval_mismatch"));
    }
    let mut indicators = Vec::new();
    for (group, names) in GROUPS {
        let fields = data
            .get(*group)
            .and_then(Value::as_object)
            .ok_or_else(|| invalid("missing_indicator_group"))?;
        for name in *names {
            indicators.push(json!({"name": name, "value": scalar(fields.get(*name), true)?}));
        }
    }
    let fields = data
        .get("summary")
        .and_then(Value::as_object)
        .ok_or_else(|| invalid("missing_summary"))?;
    let summary = json!({
        "recommendation": scalar(fields.get("recommendation"), false)?,
        "value": scalar(fields.get("value"), true)?,
        "ma": scalar(fields.get("ma"), false)?,
        "other": scalar(fields.get("other"), false)?
    });
    let available = indicators.iter().any(|item| !item["value"].is_null())
        || summary
            .as_object()
            .unwrap()
            .values()
            .any(|value| !value.is_null());
    let unknown = evidence(Value::Null);
    Ok(json!({
        "contract_version": "mcp_technicals.v1", "source": "tradingview_mcp",
        "source_category": "desktop_free_read", "requires_desktop": false,
        "request": {"symbol": request.symbol, "timeframe": request.timeframe},
        "status": if available { "available" } else { "empty" },
        "indicators": indicators, "summary": summary,
        "provider_observation": {
            "symbol": evidence(symbol), "interval": evidence(interval), "data_as_of": unknown,
            "delay_seconds": unknown, "adjustment": unknown, "session": unknown, "finality": unknown
        },
        "client_observation": {"received_at": crate::mcp_bars::received_at(received_ms)?},
        "transport": {"status": "succeeded", "tool_attempts": 1}
    }))
}

#[cfg(test)]
mod tests;
