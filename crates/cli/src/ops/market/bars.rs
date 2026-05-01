use std::{
    collections::BTreeMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::time::Instant;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Message, client::IntoClientRequest},
};
use tradingview_core::{AppError, ErrorKind};

const BARS_SOURCE: &str = "experimental_tradingview_ws";
const EXPERIMENTAL_BARS_ENV: &str = "TV_EXPERIMENTAL_BARS";
const DEFAULT_TIMEOUT_MS: u64 = 10_000;
const MAX_BAR_COUNT: usize = 500;
const WS_ENDPOINT: &str = "wss://data.tradingview.com/socket.io/websocket?type=chart";

#[derive(Debug, Clone, PartialEq)]
struct Bar {
    time: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

pub async fn bars(symbol: &str, timeframe: &str, count: usize) -> Result<Value, AppError> {
    let request = validate_bars_request(symbol, timeframe, count)?;
    let started = Instant::now();
    let result = fetch_bars_ws(&request).await?;
    let elapsed_ms = started.elapsed().as_millis() as u64;

    if result.bars.is_empty() {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Experimental bars request completed without returning bars",
        )
        .with_details(json!({
            "source": BARS_SOURCE,
            "experimental": true,
            "requested_symbol": request.symbol,
            "timeframe": request.timeframe,
            "requested_count": request.count,
            "completed": result.completed,
            "elapsed_ms": elapsed_ms,
            "next_action_hint": "This lab-gated WebSocket path is not a stable TradingView API. Retry later or use `tv ohlcv` against a selected chart target.",
        })));
    }

    Ok(json!({
        "source": BARS_SOURCE,
        "experimental": true,
        "requested_symbol": request.symbol,
        "symbol": request.symbol,
        "timeframe": request.timeframe,
        "requested_count": request.count,
        "bar_count": result.bars.len(),
        "bars": result.bars.into_iter().map(bar_to_value).collect::<Vec<_>>(),
        "data_quality": {
            "realtime_guarantee": false,
            "entitlement_checked": false,
            "completed": result.completed,
            "elapsed_ms": elapsed_ms,
        },
        "warnings": [
            "experimental undocumented TradingView WebSocket read",
            "no realtime or entitlement guarantee",
            "use `tv ohlcv` for selected-chart/CDP bars"
        ],
    }))
}

#[derive(Debug)]
struct BarsRequest {
    symbol: String,
    timeframe: String,
    count: usize,
    timeout: Duration,
}

#[derive(Debug)]
struct BarsResult {
    bars: Vec<Bar>,
    completed: bool,
}

fn validate_bars_request(
    symbol: &str,
    timeframe: &str,
    count: usize,
) -> Result<BarsRequest, AppError> {
    if !experimental_bars_enabled() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Experimental bars are disabled. Set TV_EXPERIMENTAL_BARS=1 to enable.",
        )
        .with_details(json!({
            "required_env": EXPERIMENTAL_BARS_ENV,
            "experimental": true,
            "next_action_hint": "Run with `TV_EXPERIMENTAL_BARS=1 tv bars EXCHANGE:SYMBOL --timeframe 1D --count 100` if you accept the lab boundary.",
        })));
    }

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

fn experimental_bars_enabled() -> bool {
    matches!(
        std::env::var(EXPERIMENTAL_BARS_ENV).ok().as_deref(),
        Some("1" | "true" | "yes" | "on")
    )
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

async fn fetch_bars_ws(request: &BarsRequest) -> Result<BarsResult, AppError> {
    let mut ws_request = WS_ENDPOINT.into_client_request().map_err(|err| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!("Could not prepare TradingView WebSocket request: {err}"),
        )
    })?;
    ws_request.headers_mut().insert(
        "Origin",
        "https://www.tradingview.com"
            .parse()
            .expect("valid origin header"),
    );

    let (mut stream, _) = connect_async(ws_request).await.map_err(|err| {
        AppError::new(
            ErrorKind::Connection,
            format!("TradingView WebSocket connection failed: {err}"),
        )
    })?;

    let session_id = session_id("cs_");
    let symbol_key = "symbol_1";
    send_ws(
        &mut stream,
        "set_auth_token",
        json!(["unauthorized_user_token"]),
    )
    .await?;
    send_ws(&mut stream, "chart_create_session", json!([session_id, ""])).await?;
    send_ws(
        &mut stream,
        "resolve_symbol",
        json!([
            session_id,
            symbol_key,
            format!(
                "={}",
                json!({
                    "symbol": request.symbol,
                    "adjustment": "splits",
                })
            )
        ]),
    )
    .await?;
    send_ws(
        &mut stream,
        "create_series",
        json!([
            session_id,
            "s1",
            "s1",
            symbol_key,
            request.timeframe,
            request.count
        ]),
    )
    .await?;
    send_ws(
        &mut stream,
        "switch_timezone",
        json!([session_id, "Etc/UTC"]),
    )
    .await?;

    let deadline = Instant::now() + request.timeout;
    let mut bars = Vec::new();
    let mut completed = false;
    loop {
        let now = Instant::now();
        if now >= deadline {
            break;
        }
        let remaining = deadline.saturating_duration_since(now);
        let message = tokio::time::timeout(remaining, stream.next())
            .await
            .map_err(|_| AppError::new(ErrorKind::Timeout, "Experimental bars request timed out"))?
            .ok_or_else(|| AppError::new(ErrorKind::Connection, "TradingView WebSocket closed"))?
            .map_err(|err| {
                AppError::new(
                    ErrorKind::Connection,
                    format!("TradingView WebSocket read failed: {err}"),
                )
            })?;

        for packet in parse_packets(message)? {
            match packet {
                WsPacket::Ping(value) => {
                    stream
                        .send(Message::Text(pong_frame(value).into()))
                        .await
                        .map_err(|err| {
                            AppError::new(
                                ErrorKind::Connection,
                                format!("TradingView WebSocket pong failed: {err}"),
                            )
                        })?;
                }
                WsPacket::Message(value) => {
                    let method = value.get("m").and_then(Value::as_str).unwrap_or_default();
                    if method == "timescale_update" || method == "du" {
                        let update_bars = parse_bars_from_message(&value);
                        merge_bars(&mut bars, update_bars);
                    } else if method == "series_completed" {
                        completed = true;
                        let _ =
                            send_ws(&mut stream, "chart_delete_session", json!([session_id])).await;
                        let _ = stream.close(None).await;
                        return Ok(BarsResult { bars, completed });
                    } else if matches!(method, "symbol_error" | "series_error" | "protocol_error") {
                        let _ =
                            send_ws(&mut stream, "chart_delete_session", json!([session_id])).await;
                        return Err(AppError::new(
                            ErrorKind::InternalApiUnavailable,
                            "TradingView WebSocket returned an error for experimental bars",
                        )
                        .with_details(json!({
                            "source": BARS_SOURCE,
                            "experimental": true,
                            "method": method,
                            "requested_symbol": request.symbol,
                            "timeframe": request.timeframe,
                            "requested_count": request.count,
                        })));
                    }
                }
            }
        }
    }

    let _ = send_ws(&mut stream, "chart_delete_session", json!([session_id])).await;
    let _ = stream.close(None).await;
    Ok(BarsResult { bars, completed })
}

async fn send_ws<S>(stream: &mut S, method: &str, params: Value) -> Result<(), AppError>
where
    S: futures_util::Sink<Message> + Unpin,
    S::Error: std::fmt::Display,
{
    let payload = json!({ "m": method, "p": params }).to_string();
    stream
        .send(Message::Text(frame(&payload).into()))
        .await
        .map_err(|err| {
            AppError::new(
                ErrorKind::Connection,
                format!("TradingView WebSocket send failed: {err}"),
            )
        })
}

fn session_id(prefix: &str) -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    format!("{prefix}{:x}{:x}", std::process::id(), millis)
}

fn frame(payload: &str) -> String {
    format!("~m~{}~m~{}", payload.len(), payload)
}

fn pong_frame(value: i64) -> String {
    format!("~m~{}~m~~h~{}~", value.to_string().len() + 3, value)
}

#[derive(Debug, PartialEq)]
enum WsPacket {
    Message(Value),
    Ping(i64),
}

fn parse_packets(message: Message) -> Result<Vec<WsPacket>, AppError> {
    match message {
        Message::Text(text) => parse_text_packets(&text),
        Message::Binary(bytes) => {
            let text = String::from_utf8(bytes.to_vec()).map_err(|err| {
                AppError::new(
                    ErrorKind::InternalApiUnavailable,
                    format!("TradingView WebSocket returned non-UTF8 data: {err}"),
                )
            })?;
            parse_text_packets(&text)
        }
        Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => Ok(Vec::new()),
        Message::Close(_) => Err(AppError::new(
            ErrorKind::Connection,
            "TradingView WebSocket closed",
        )),
    }
}

fn parse_text_packets(raw: &str) -> Result<Vec<WsPacket>, AppError> {
    let mut packets = Vec::new();
    let mut index = 0;
    while index < raw.len() {
        let remaining = &raw[index..];
        if let Some(rest) = remaining.strip_prefix("~h~") {
            let digits = rest
                .chars()
                .take_while(|ch| ch.is_ascii_digit())
                .collect::<String>();
            if digits.is_empty() {
                break;
            }
            let value = digits.parse::<i64>().map_err(|err| {
                AppError::new(
                    ErrorKind::InternalApiUnavailable,
                    format!("Could not parse TradingView WebSocket ping: {err}"),
                )
            })?;
            packets.push(WsPacket::Ping(value));
            index += 3 + digits.len();
            continue;
        }
        if !remaining.starts_with("~m~") {
            break;
        }
        let len_start = index + 3;
        let Some(len_end_offset) = raw[len_start..].find("~m~") else {
            break;
        };
        let len_end = len_start + len_end_offset;
        let length = raw[len_start..len_end].parse::<usize>().map_err(|err| {
            AppError::new(
                ErrorKind::InternalApiUnavailable,
                format!("Could not parse TradingView WebSocket frame length: {err}"),
            )
        })?;
        let payload_start = len_end + 3;
        let payload_end = payload_start + length;
        if payload_end > raw.len() {
            return Err(AppError::new(
                ErrorKind::InternalApiUnavailable,
                "TradingView WebSocket frame was truncated",
            ));
        }
        let payload = &raw[payload_start..payload_end];
        if let Some(rest) = payload.strip_prefix("~h~") {
            let value = rest.trim_end_matches('~').parse::<i64>().map_err(|err| {
                AppError::new(
                    ErrorKind::InternalApiUnavailable,
                    format!("Could not parse TradingView WebSocket framed ping: {err}"),
                )
            })?;
            packets.push(WsPacket::Ping(value));
        } else {
            let value = serde_json::from_str::<Value>(payload).map_err(|err| {
                AppError::new(
                    ErrorKind::InternalApiUnavailable,
                    format!("Could not parse TradingView WebSocket JSON frame: {err}"),
                )
            })?;
            packets.push(WsPacket::Message(value));
        }
        index = payload_end;
    }
    Ok(packets)
}

fn parse_bars_from_message(message: &Value) -> Vec<Bar> {
    let Some(series_map) = message
        .get("p")
        .and_then(Value::as_array)
        .and_then(|params| params.get(1))
        .and_then(Value::as_object)
    else {
        return Vec::new();
    };

    let mut bars = Vec::new();
    for series_value in series_map.values() {
        if let Some(series_bars) = series_value.get("s").and_then(Value::as_array) {
            bars.extend(series_bars.iter().filter_map(parse_bar));
        } else if let Some(series_bars) = series_value.as_array() {
            bars.extend(series_bars.iter().filter_map(parse_bar));
        }
    }
    bars.sort_by_key(|bar| bar.time);
    bars
}

fn parse_bar(value: &Value) -> Option<Bar> {
    let values = value
        .get("v")
        .and_then(Value::as_array)
        .or_else(|| value.as_array())?;
    let time = normalize_time(values.first()?.as_f64()?);
    let open = values.get(1)?.as_f64()?;
    let third = values.get(2)?.as_f64()?;
    let fourth = values.get(3)?.as_f64()?;
    let fifth = values.get(4)?.as_f64()?;
    let volume = values.get(5).and_then(Value::as_f64).unwrap_or(0.0);
    Some(Bar {
        time,
        open,
        high: third,
        low: fourth,
        close: fifth,
        volume,
    })
}

fn normalize_time(time: f64) -> i64 {
    if time > 1_000_000_000_000.0 {
        (time / 1000.0).floor() as i64
    } else {
        time.floor() as i64
    }
}

fn merge_bars(existing: &mut Vec<Bar>, incoming: Vec<Bar>) {
    let mut map = existing
        .drain(..)
        .map(|bar| (bar.time, bar))
        .collect::<BTreeMap<_, _>>();
    for bar in incoming {
        map.insert(bar.time, bar);
    }
    *existing = map.into_values().collect();
}

fn bar_to_value(bar: Bar) -> Value {
    json!({
        "time": bar.time,
        "open": bar.open,
        "high": bar.high,
        "low": bar.low,
        "close": bar.close,
        "volume": bar.volume,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, OnceLock};

    use super::*;

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn parse_text_packets_handles_multiple_frames_and_ping() {
        let first = json!({"m": "timescale_update", "p": ["cs", {}]}).to_string();
        let second = json!({"m": "series_completed", "p": ["cs", "s1"]}).to_string();
        let raw = format!("{}~h~42{}", frame(&first), frame(&second));
        let packets = parse_text_packets(&raw).unwrap();
        assert!(matches!(packets[0], WsPacket::Message(_)));
        assert_eq!(packets[1], WsPacket::Ping(42));
        assert!(matches!(packets[2], WsPacket::Message(_)));
    }

    #[test]
    fn parse_bars_from_timescale_update_sorts_and_normalizes_time() {
        let message = json!({
            "m": "timescale_update",
            "p": [
                "cs_test",
                {
                    "s1": {
                        "s": [
                            {"i": 1, "v": [1714608000000.0, 10.0, 12.0, 9.0, 11.0, 1000.25]},
                            {"i": 0, "v": [1714521600.0, 8.0, 10.0, 7.5, 9.5, 900.0]}
                        ]
                    }
                }
            ]
        });
        let bars = parse_bars_from_message(&message);
        assert_eq!(bars.len(), 2);
        assert_eq!(bars[0].time, 1_714_521_600);
        assert_eq!(bars[1].time, 1_714_608_000);
        assert_eq!(bars[1].high, 12.0);
        assert_eq!(bars[1].low, 9.0);
    }

    #[test]
    fn merge_bars_replaces_duplicate_time() {
        let mut existing = vec![Bar {
            time: 1,
            open: 1.0,
            high: 2.0,
            low: 0.5,
            close: 1.5,
            volume: 10.0,
        }];
        merge_bars(
            &mut existing,
            vec![Bar {
                time: 1,
                open: 2.0,
                high: 3.0,
                low: 1.0,
                close: 2.5,
                volume: 20.0,
            }],
        );
        assert_eq!(existing.len(), 1);
        assert_eq!(existing[0].open, 2.0);
    }

    #[test]
    fn validate_rejects_disabled_gate_before_network() {
        let _guard = env_lock().lock().unwrap();
        unsafe {
            std::env::remove_var(EXPERIMENTAL_BARS_ENV);
        }
        let err = validate_bars_request("NASDAQ:AAPL", "1D", 5).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);
        assert_eq!(err.details.unwrap()["required_env"], EXPERIMENTAL_BARS_ENV);
    }

    #[test]
    fn validate_rejects_bare_symbol_and_out_of_range_count() {
        let _guard = env_lock().lock().unwrap();
        unsafe {
            std::env::set_var(EXPERIMENTAL_BARS_ENV, "1");
        }
        let err = validate_bars_request("AAPL", "1D", 5).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);
        let err = validate_bars_request("NASDAQ:AAPL", "1D", 501).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);
        unsafe {
            std::env::remove_var(EXPERIMENTAL_BARS_ENV);
        }
    }

    #[test]
    fn validate_accepts_supported_timeframe_aliases() {
        let _guard = env_lock().lock().unwrap();
        unsafe {
            std::env::set_var(EXPERIMENTAL_BARS_ENV, "1");
        }
        let request = validate_bars_request("NASDAQ:AAPL", "1d", 5).unwrap();
        assert_eq!(request.timeframe, "1D");
        unsafe {
            std::env::remove_var(EXPERIMENTAL_BARS_ENV);
        }
    }
}
