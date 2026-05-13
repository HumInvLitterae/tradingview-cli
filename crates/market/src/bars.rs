use std::{
    collections::BTreeMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use futures_util::{SinkExt, StreamExt};
use serde_json::{Map, Value, json};
use tokio::time::Instant;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Message, client::IntoClientRequest},
};
use tradingview_core::{AppError, ErrorKind};

const BARS_CONTRACT_VERSION: &str = "bars.v1";
const BARS_SOURCE: &str = "tradingview_bars_ws";
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

pub async fn bars_symbol(symbol: &str, timeframe: &str, count: usize) -> Result<Value, AppError> {
    let request = validate_bars_request(symbol, timeframe, count)?;
    let started = Instant::now();
    let result = fetch_bars_ws(&request).await?;
    let elapsed_ms = started.elapsed().as_millis() as u64;

    if result.bars.is_empty() {
        let source_availability = bars_source_availability(
            &request,
            BarsAvailabilityState::unavailable(
                "timeout_no_bars",
                0,
                result.completed,
                !result.completed,
            ),
            &result.wait_summary,
            elapsed_ms,
        );
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Bars request completed without returning bars",
        )
        .with_details(bars_error_details(
            &request,
            json!({
                "availability_status": "unavailable",
                "completed": result.completed,
                "elapsed_ms": elapsed_ms,
                "source_availability": source_availability,
                "next_action_hint": "The browserless historical bars source did not return bars inside the bounded request. Retry later or use `tv ohlcv` against a selected chart target when chart-backed bars are acceptable.",
            }),
        )));
    }

    Ok(bars_payload(&request, result, elapsed_ms))
}

fn bars_payload(request: &BarsRequest, result: BarsResult, elapsed_ms: u64) -> Value {
    let bar_count = result.bars.len();
    let first_time = result
        .bars
        .first()
        .map(|bar| bar.time)
        .expect("bars_payload requires at least one bar");
    let last_time = result
        .bars
        .last()
        .map(|bar| bar.time)
        .expect("bars_payload requires at least one bar");
    let requested_count_fulfilled = bar_count == request.count;
    let coverage_status = if requested_count_fulfilled {
        "complete"
    } else {
        "partial"
    };
    let timed_out = !result.completed;
    let source_availability = bars_source_availability(
        request,
        BarsAvailabilityState::available(bar_count, result.completed, timed_out),
        &result.wait_summary,
        elapsed_ms,
    );

    json!({
        "contract_version": BARS_CONTRACT_VERSION,
        "source": BARS_SOURCE,
        "source_category": "desktop_free_read",
        "requires_desktop": false,
        "non_mutating": true,
        "requested_symbol": request.symbol,
        "symbol": request.symbol,
        "timeframe": request.timeframe,
        "requested_count": request.count,
        "bar_count": bar_count,
        "summary": {
            "requested_count": request.count,
            "bar_count": bar_count,
            "first_time": first_time,
            "last_time": last_time,
            "time_order": "ascending",
            "requested_count_fulfilled": requested_count_fulfilled,
            "coverage_status": coverage_status,
        },
        "source_availability": source_availability,
        "range": {
            "timeframe": request.timeframe,
            "first_time": first_time,
            "last_time": last_time,
            "bar_count": bar_count,
        },
        "bars": result.bars.into_iter().map(bar_to_value).collect::<Vec<_>>(),
        "data_quality": {
            "realtime_guarantee": false,
            "entitlement_checked": false,
            "completed": result.completed,
            "elapsed_ms": elapsed_ms,
            "partial_result": !requested_count_fulfilled,
        },
        "warnings": [
            "undocumented TradingView WebSocket read",
            "no realtime or entitlement guarantee",
            "use `tv ohlcv` for selected-chart/CDP bars"
        ],
    })
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
    wait_summary: BarsWaitSummary,
}

#[derive(Debug, Clone, Default)]
struct BarsWaitSummary {
    timeout_ms: u64,
    websocket_messages_seen: u64,
    websocket_packets_seen: u64,
    update_messages_seen: u64,
    series_completed_seen: bool,
    error_messages_seen: u64,
}

#[derive(Debug, Clone)]
struct BarsAvailabilityState<'a> {
    available: bool,
    unavailable_reason: Option<&'a str>,
    bar_count: usize,
    completed: bool,
    timed_out: bool,
}

impl BarsAvailabilityState<'_> {
    fn available(bar_count: usize, completed: bool, timed_out: bool) -> Self {
        Self {
            available: true,
            unavailable_reason: None,
            bar_count,
            completed,
            timed_out,
        }
    }

    fn unavailable(
        unavailable_reason: &'static str,
        bar_count: usize,
        completed: bool,
        timed_out: bool,
    ) -> Self {
        Self {
            available: false,
            unavailable_reason: Some(unavailable_reason),
            bar_count,
            completed,
            timed_out,
        }
    }
}

impl BarsWaitSummary {
    fn new(request: &BarsRequest) -> Self {
        Self {
            timeout_ms: request.timeout.as_millis() as u64,
            ..Self::default()
        }
    }

    fn to_value(&self, elapsed_ms: u64, completed: bool, bars_observed_count: usize) -> Value {
        json!({
            "timeout_ms": self.timeout_ms,
            "elapsed_ms": elapsed_ms,
            "completed": completed,
            "websocket_messages_seen": self.websocket_messages_seen,
            "websocket_packets_seen": self.websocket_packets_seen,
            "update_messages_seen": self.update_messages_seen,
            "series_completed_seen": self.series_completed_seen,
            "error_messages_seen": self.error_messages_seen,
            "bars_observed_count": bars_observed_count,
            "raw_frame_included": false,
        })
    }
}

fn validate_bars_request(
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

async fn fetch_bars_ws(request: &BarsRequest) -> Result<BarsResult, AppError> {
    let started = Instant::now();
    let mut wait_summary = BarsWaitSummary::new(request);
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
        .with_details(bars_error_details(
            request,
            json!({
                "availability_status": "unavailable",
                "source_availability": bars_source_availability(
                    request,
                    BarsAvailabilityState::unavailable("connection_failed", 0, false, false),
                    &wait_summary,
                    started.elapsed().as_millis() as u64,
                ),
            }),
        ))
    })?;

    let session_id = session_id("cs_");
    let symbol_key = "symbol_1";
    send_ws(
        &mut stream,
        "set_auth_token",
        json!(["unauthorized_user_token"]),
    )
    .await
    .map_err(|err| {
        err.with_details(bars_error_details(
            request,
            json!({
                "availability_status": "unavailable",
                "source_availability": bars_source_availability(
                    request,
                    BarsAvailabilityState::unavailable("connection_failed", 0, false, false),
                    &wait_summary,
                    started.elapsed().as_millis() as u64,
                ),
            }),
        ))
    })?;
    send_ws(&mut stream, "chart_create_session", json!([session_id, ""]))
        .await
        .map_err(|err| {
            err.with_details(bars_error_details(
                request,
                json!({
                    "availability_status": "unavailable",
                    "source_availability": bars_source_availability(
                        request,
                        BarsAvailabilityState::unavailable("connection_failed", 0, false, false),
                        &wait_summary,
                        started.elapsed().as_millis() as u64,
                    ),
                }),
            ))
        })?;
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
    .await
    .map_err(|err| {
        err.with_details(bars_error_details(
            request,
            json!({
                "availability_status": "unavailable",
                "source_availability": bars_source_availability(
                    request,
                    BarsAvailabilityState::unavailable("connection_failed", 0, false, false),
                    &wait_summary,
                    started.elapsed().as_millis() as u64,
                ),
            }),
        ))
    })?;
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
    .await
    .map_err(|err| {
        err.with_details(bars_error_details(
            request,
            json!({
                "availability_status": "unavailable",
                "source_availability": bars_source_availability(
                    request,
                    BarsAvailabilityState::unavailable("connection_failed", 0, false, false),
                    &wait_summary,
                    started.elapsed().as_millis() as u64,
                ),
            }),
        ))
    })?;
    send_ws(
        &mut stream,
        "switch_timezone",
        json!([session_id, "Etc/UTC"]),
    )
    .await
    .map_err(|err| {
        err.with_details(bars_error_details(
            request,
            json!({
                "availability_status": "unavailable",
                "source_availability": bars_source_availability(
                    request,
                    BarsAvailabilityState::unavailable("connection_failed", 0, false, false),
                    &wait_summary,
                    started.elapsed().as_millis() as u64,
                ),
            }),
        ))
    })?;

    let deadline = Instant::now() + request.timeout;
    let mut bars = Vec::new();
    let mut completed = false;
    loop {
        let now = Instant::now();
        if now >= deadline {
            break;
        }
        let remaining = deadline.saturating_duration_since(now);
        let message = match tokio::time::timeout(remaining, stream.next()).await {
            Ok(Some(Ok(message))) => {
                wait_summary.websocket_messages_seen += 1;
                message
            }
            Ok(Some(Err(err))) => {
                return Err(AppError::new(
                    ErrorKind::Connection,
                    format!("TradingView WebSocket read failed: {err}"),
                )
                .with_details(bars_error_details(
                    request,
                    json!({
                        "availability_status": "unavailable",
                        "source_availability": bars_source_availability(
                            request,
                            BarsAvailabilityState::unavailable(
                                "websocket_read_failed",
                                bars.len(),
                                false,
                                false,
                            ),
                            &wait_summary,
                            started.elapsed().as_millis() as u64,
                        ),
                    }),
                )));
            }
            Ok(None) => {
                return Err(
                    AppError::new(ErrorKind::Connection, "TradingView WebSocket closed")
                        .with_details(bars_error_details(
                            request,
                            json!({
                                "availability_status": "unavailable",
                                "source_availability": bars_source_availability(
                                    request,
                                    BarsAvailabilityState::unavailable(
                                        "websocket_closed",
                                        bars.len(),
                                        false,
                                        false,
                                    ),
                                    &wait_summary,
                                    started.elapsed().as_millis() as u64,
                                ),
                            }),
                        )),
                );
            }
            Err(_) => {
                if bars.is_empty() {
                    return Err(AppError::new(ErrorKind::Timeout, "Bars request timed out")
                        .with_details(bars_error_details(
                            request,
                            json!({
                                "availability_status": "unavailable",
                                "source_availability": bars_source_availability(
                                    request,
                                    BarsAvailabilityState::unavailable(
                                        "timeout_no_bars",
                                        0,
                                        false,
                                        true,
                                    ),
                                    &wait_summary,
                                    started.elapsed().as_millis() as u64,
                                ),
                            }),
                        )));
                }
                break;
            }
        };

        let packets = parse_packets(message).map_err(|err| {
            wait_summary.error_messages_seen += 1;
            err.with_details(bars_error_details(
                request,
                json!({
                    "availability_status": "unavailable",
                    "source_availability": bars_source_availability(
                        request,
                        BarsAvailabilityState::unavailable(
                            "protocol_error",
                            bars.len(),
                            false,
                            false,
                        ),
                        &wait_summary,
                        started.elapsed().as_millis() as u64,
                    ),
                }),
            ))
        })?;
        wait_summary.websocket_packets_seen += packets.len() as u64;

        for packet in packets {
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
                        wait_summary.update_messages_seen += 1;
                        let update_bars = parse_bars_from_message(&value);
                        merge_bars(&mut bars, update_bars);
                    } else if method == "series_completed" {
                        completed = true;
                        wait_summary.series_completed_seen = true;
                        let _ =
                            send_ws(&mut stream, "chart_delete_session", json!([session_id])).await;
                        let _ = stream.close(None).await;
                        return Ok(BarsResult {
                            bars,
                            completed,
                            wait_summary,
                        });
                    } else if matches!(method, "symbol_error" | "series_error" | "protocol_error") {
                        wait_summary.error_messages_seen += 1;
                        let _ =
                            send_ws(&mut stream, "chart_delete_session", json!([session_id])).await;
                        return Err(AppError::new(
                            ErrorKind::InternalApiUnavailable,
                            "TradingView WebSocket returned an error for bars",
                        )
                        .with_details(bars_error_details(
                            request,
                            json!({
                                "availability_status": "unavailable",
                                "method": method,
                                "source_availability": bars_source_availability(
                                    request,
                                    BarsAvailabilityState::unavailable(
                                        "protocol_error",
                                        bars.len(),
                                        false,
                                        false,
                                    ),
                                    &wait_summary,
                                    started.elapsed().as_millis() as u64,
                                ),
                            }),
                        )));
                    }
                }
            }
        }
    }

    let _ = send_ws(&mut stream, "chart_delete_session", json!([session_id])).await;
    let _ = stream.close(None).await;
    Ok(BarsResult {
        bars,
        completed,
        wait_summary,
    })
}

fn bars_source_availability(
    request: &BarsRequest,
    state: BarsAvailabilityState<'_>,
    wait_summary: &BarsWaitSummary,
    elapsed_ms: u64,
) -> Value {
    json!({
        "available": state.available,
        "status": if state.available { "available" } else { "unavailable" },
        "unavailable_reason": state.unavailable_reason,
        "requested_count": request.count,
        "bar_count": state.bar_count,
        "requested_count_fulfilled": state.bar_count == request.count,
        "timed_out": state.timed_out,
        "raw_frame_included": false,
        "wait_summary": wait_summary.to_value(elapsed_ms, state.completed, state.bar_count),
    })
}

fn bars_error_details(request: &BarsRequest, extra: Value) -> Value {
    let mut details = Map::new();
    details.insert(
        "contract_version".to_string(),
        Value::String(BARS_CONTRACT_VERSION.to_string()),
    );
    details.insert("source".to_string(), Value::String(BARS_SOURCE.to_string()));
    details.insert(
        "source_category".to_string(),
        Value::String("desktop_free_read".to_string()),
    );
    details.insert("requires_desktop".to_string(), Value::Bool(false));
    details.insert("non_mutating".to_string(), Value::Bool(true));
    details.insert(
        "requested_symbol".to_string(),
        Value::String(request.symbol.clone()),
    );
    details.insert(
        "timeframe".to_string(),
        Value::String(request.timeframe.clone()),
    );
    details.insert(
        "requested_count".to_string(),
        Value::Number((request.count as u64).into()),
    );

    if let Some(extra) = extra.as_object() {
        for (key, value) in extra {
            details.insert(key.clone(), value.clone());
        }
    }

    Value::Object(details)
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
    use super::*;

    fn test_wait_summary(request: &BarsRequest) -> BarsWaitSummary {
        let mut summary = BarsWaitSummary::new(request);
        summary.websocket_messages_seen = 3;
        summary.websocket_packets_seen = 4;
        summary.update_messages_seen = 2;
        summary.series_completed_seen = true;
        summary
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
    fn bars_payload_contains_stable_source_contract() {
        let request = validate_bars_request("NASDAQ:AAPL", "1d", 5).unwrap();
        let payload = bars_payload(
            &request,
            BarsResult {
                bars: vec![
                    Bar {
                        time: 1,
                        open: 10.0,
                        high: 12.0,
                        low: 9.0,
                        close: 11.0,
                        volume: 100.0,
                    },
                    Bar {
                        time: 2,
                        open: 11.0,
                        high: 13.0,
                        low: 10.0,
                        close: 12.0,
                        volume: 200.0,
                    },
                ],
                completed: true,
                wait_summary: test_wait_summary(&request),
            },
            42,
        );

        assert_eq!(payload["contract_version"], BARS_CONTRACT_VERSION);
        assert_eq!(payload["source"], BARS_SOURCE);
        assert_eq!(payload["source_category"], "desktop_free_read");
        assert_eq!(payload["requires_desktop"], false);
        assert_eq!(payload["non_mutating"], true);
        assert_eq!(payload["data_quality"]["realtime_guarantee"], false);
        assert_eq!(payload["data_quality"]["entitlement_checked"], false);
        assert_eq!(payload["data_quality"]["partial_result"], true);
        assert_eq!(payload["summary"]["requested_count"], 5);
        assert_eq!(payload["summary"]["bar_count"], 2);
        assert_eq!(payload["summary"]["first_time"], 1);
        assert_eq!(payload["summary"]["last_time"], 2);
        assert_eq!(payload["summary"]["time_order"], "ascending");
        assert_eq!(payload["summary"]["requested_count_fulfilled"], false);
        assert_eq!(payload["summary"]["coverage_status"], "partial");
        assert_eq!(payload["range"]["timeframe"], "1D");
        assert_eq!(payload["range"]["first_time"], 1);
        assert_eq!(payload["range"]["last_time"], 2);
        assert_eq!(payload["range"]["bar_count"], 2);
        assert_eq!(payload["source_availability"]["available"], true);
        assert_eq!(payload["source_availability"]["status"], "available");
        assert!(payload["source_availability"]["unavailable_reason"].is_null());
        assert_eq!(payload["source_availability"]["requested_count"], 5);
        assert_eq!(payload["source_availability"]["bar_count"], 2);
        assert_eq!(
            payload["source_availability"]["requested_count_fulfilled"],
            false
        );
        assert_eq!(payload["source_availability"]["timed_out"], false);
        assert_eq!(payload["source_availability"]["raw_frame_included"], false);
        assert_eq!(
            payload["source_availability"]["wait_summary"]["completed"],
            true
        );
        assert_eq!(
            payload["source_availability"]["wait_summary"]["websocket_messages_seen"],
            3
        );
        assert_eq!(
            payload["source_availability"]["wait_summary"]["websocket_packets_seen"],
            4
        );
        assert_eq!(
            payload["source_availability"]["wait_summary"]["update_messages_seen"],
            2
        );
        assert_eq!(
            payload["source_availability"]["wait_summary"]["series_completed_seen"],
            true
        );
        assert_eq!(
            payload["source_availability"]["wait_summary"]["bars_observed_count"],
            2
        );
        assert_eq!(
            payload["source_availability"]["wait_summary"]["raw_frame_included"],
            false
        );
        assert!(payload.get("experimental").is_none());
    }

    #[test]
    fn bars_payload_marks_full_count_coverage_complete() {
        let request = validate_bars_request("NASDAQ:AAPL", "1d", 1).unwrap();
        let payload = bars_payload(
            &request,
            BarsResult {
                bars: vec![Bar {
                    time: 1,
                    open: 10.0,
                    high: 12.0,
                    low: 9.0,
                    close: 11.0,
                    volume: 100.0,
                }],
                completed: true,
                wait_summary: test_wait_summary(&request),
            },
            42,
        );

        assert_eq!(payload["summary"]["requested_count"], 1);
        assert_eq!(payload["summary"]["bar_count"], 1);
        assert_eq!(payload["summary"]["requested_count_fulfilled"], true);
        assert_eq!(payload["summary"]["coverage_status"], "complete");
        assert_eq!(payload["data_quality"]["partial_result"], false);
        assert_eq!(
            payload["source_availability"]["requested_count_fulfilled"],
            true
        );
    }

    #[test]
    fn bars_payload_marks_partial_timeout_when_series_does_not_complete() {
        let request = validate_bars_request("NASDAQ:AAPL", "1d", 2).unwrap();
        let mut wait_summary = test_wait_summary(&request);
        wait_summary.series_completed_seen = false;
        let payload = bars_payload(
            &request,
            BarsResult {
                bars: vec![Bar {
                    time: 1,
                    open: 10.0,
                    high: 12.0,
                    low: 9.0,
                    close: 11.0,
                    volume: 100.0,
                }],
                completed: false,
                wait_summary,
            },
            42,
        );

        assert_eq!(payload["summary"]["coverage_status"], "partial");
        assert_eq!(payload["source_availability"]["available"], true);
        assert_eq!(payload["source_availability"]["timed_out"], true);
        assert_eq!(
            payload["source_availability"]["wait_summary"]["completed"],
            false
        );
        assert_eq!(
            payload["source_availability"]["wait_summary"]["series_completed_seen"],
            false
        );
    }

    #[test]
    fn bars_error_details_contains_stable_source_contract() {
        let request = validate_bars_request("NASDAQ:AAPL", "1d", 5).unwrap();
        let details = bars_error_details(
            &request,
            json!({
                "availability_status": "unavailable",
                "completed": false,
            }),
        );

        assert_eq!(details["contract_version"], BARS_CONTRACT_VERSION);
        assert_eq!(details["source"], BARS_SOURCE);
        assert_eq!(details["source_category"], "desktop_free_read");
        assert_eq!(details["requires_desktop"], false);
        assert_eq!(details["non_mutating"], true);
        assert_eq!(details["availability_status"], "unavailable");
        assert_eq!(details["completed"], false);
        assert!(details.get("raw_frame").is_none());
        assert!(details.get("raw_payload").is_none());
    }

    #[test]
    fn bars_source_availability_reports_no_bars_failure_without_raw_frames() {
        let request = validate_bars_request("NASDAQ:AAPL", "1d", 5).unwrap();
        let availability = bars_source_availability(
            &request,
            BarsAvailabilityState::unavailable("timeout_no_bars", 0, false, true),
            &BarsWaitSummary::new(&request),
            42,
        );

        assert_eq!(availability["available"], false);
        assert_eq!(availability["status"], "unavailable");
        assert_eq!(availability["unavailable_reason"], "timeout_no_bars");
        assert_eq!(availability["requested_count"], 5);
        assert_eq!(availability["bar_count"], 0);
        assert_eq!(availability["requested_count_fulfilled"], false);
        assert_eq!(availability["timed_out"], true);
        assert_eq!(availability["raw_frame_included"], false);
        assert_eq!(
            availability["wait_summary"]["timeout_ms"],
            DEFAULT_TIMEOUT_MS
        );
        assert_eq!(availability["wait_summary"]["elapsed_ms"], 42);
        assert_eq!(availability["wait_summary"]["completed"], false);
        assert_eq!(availability["wait_summary"]["bars_observed_count"], 0);
        assert_eq!(availability["wait_summary"]["raw_frame_included"], false);
        assert!(availability.get("raw_frame").is_none());
        assert!(availability.get("raw_payload").is_none());
    }
}
