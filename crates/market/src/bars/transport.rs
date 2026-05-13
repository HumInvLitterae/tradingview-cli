use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::time::Instant;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Message, client::IntoClientRequest},
};
use tradingview_core::{AppError, ErrorKind};

use super::{
    payload::{bars_error_details, bars_source_availability},
    protocol::{
        WsPacket, frame, merge_bars, parse_bars_from_message, parse_packets, pong_frame, session_id,
    },
    types::{BarsAvailabilityState, BarsRequest, BarsResult, BarsWaitSummary},
};

const WS_ENDPOINT: &str = "wss://data.tradingview.com/socket.io/websocket?type=chart";

pub(super) async fn fetch_bars_ws(request: &BarsRequest) -> Result<BarsResult, AppError> {
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
