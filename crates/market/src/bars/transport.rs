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
    types::{
        Bar, BarsAvailabilityState, BarsFetchSummary, BarsFetchSummaryInput, BarsRequest,
        BarsResult, BarsWaitSummary,
    },
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
            request.initial_fetch_count()
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
    let mut last_more_oldest: Option<i64> = None;
    let mut request_more_count = 0;
    'read_loop: loop {
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
                        "range_fetch_summary": range_fetch_summary_for_bars(
                            request,
                            &bars,
                            request_more_count,
                            false,
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
                                "range_fetch_summary": range_fetch_summary_for_bars(
                                    request,
                                    &bars,
                                    request_more_count,
                                    false,
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
                                "range_fetch_summary": range_fetch_summary_for_bars(
                                    request,
                                    &bars,
                                    request_more_count,
                                    false,
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
                    "range_fetch_summary": range_fetch_summary_for_bars(
                        request,
                        &bars,
                        request_more_count,
                        false,
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
                        if should_request_more(request, &bars) {
                            let oldest_time = bars.first().map(|bar| bar.time);
                            if oldest_time.is_some() && oldest_time == last_more_oldest {
                                break 'read_loop;
                            }
                            last_more_oldest = oldest_time;
                            completed = false;
                            send_ws(
                                &mut stream,
                                "request_more_data",
                                json!([session_id, "s1", request.initial_fetch_count()]),
                            )
                            .await
                            .map_err(|err| {
                                err.with_details(bars_error_details(
                                    request,
                                    json!({
                                    "availability_status": "unavailable",
                                    "source_availability": bars_source_availability(
                                        request,
                                        BarsAvailabilityState::unavailable(
                                            "connection_failed",
                                            bars.len(),
                                            false,
                                            false,
                                        ),
                                            &wait_summary,
                                            started.elapsed().as_millis() as u64,
                                        ),
                                        "range_fetch_summary": range_fetch_summary_for_bars(
                                            request,
                                            &bars,
                                            request_more_count,
                                            false,
                                        ),
                                    }),
                                ))
                            })?;
                            request_more_count += 1;
                            continue;
                        }
                        let _ =
                            send_ws(&mut stream, "chart_delete_session", json!([session_id])).await;
                        let _ = stream.close(None).await;
                        return Ok(finalize_result(
                            request,
                            bars,
                            completed,
                            wait_summary,
                            request_more_count,
                        ));
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
                                "range_fetch_summary": range_fetch_summary_for_bars(
                                    request,
                                    &bars,
                                    request_more_count,
                                    false,
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
    Ok(finalize_result(
        request,
        bars,
        completed,
        wait_summary,
        request_more_count,
    ))
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

fn should_request_more(request: &BarsRequest, bars: &[super::types::Bar]) -> bool {
    let Some((from_time, _)) = request.date_range_bounds() else {
        return false;
    };
    let Some(oldest_time) = bars.first().map(|bar| bar.time) else {
        return false;
    };
    oldest_time > from_time
}

fn finalize_result(
    request: &BarsRequest,
    mut bars: Vec<super::types::Bar>,
    completed: bool,
    wait_summary: BarsWaitSummary,
    request_more_count: usize,
) -> BarsResult {
    let observed_first_time = bars.first().map(|bar| bar.time);
    let observed_last_time = bars.last().map(|bar| bar.time);
    let observed_count = bars.len();
    let mut filtered_count = observed_count;
    if let Some((from_time, to_time_exclusive)) = request.date_range_bounds() {
        bars.retain(|bar| bar.time >= from_time && bar.time < to_time_exclusive);
        filtered_count = bars.len();
        if bars.len() > request.count {
            bars.truncate(request.count);
        }
    }
    let returned_count = bars.len();
    let fetch_summary = BarsFetchSummary::new(
        request,
        BarsFetchSummaryInput {
            request_more_count,
            observed_count,
            filtered_count,
            returned_count,
            completed,
            observed_first_time,
            observed_last_time,
        },
    );
    BarsResult {
        bars,
        completed,
        wait_summary,
        fetch_summary,
        observed_first_time,
        observed_last_time,
    }
}

fn range_fetch_summary_for_bars(
    request: &BarsRequest,
    bars: &[Bar],
    request_more_count: usize,
    completed: bool,
) -> Value {
    let observed_first_time = bars.first().map(|bar| bar.time);
    let observed_last_time = bars.last().map(|bar| bar.time);
    let observed_count = bars.len();
    let filtered_count = request
        .date_range_bounds()
        .map(|(from_time, to_time_exclusive)| {
            bars.iter()
                .filter(|bar| bar.time >= from_time && bar.time < to_time_exclusive)
                .count()
        })
        .unwrap_or(observed_count);
    let returned_count = filtered_count.min(request.count);

    BarsFetchSummary::new(
        request,
        BarsFetchSummaryInput {
            request_more_count,
            observed_count,
            filtered_count,
            returned_count,
            completed,
            observed_first_time,
            observed_last_time,
        },
    )
    .to_value()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bars::validation::{validate_bars_range_request, validate_bars_request};

    fn bar(time: i64) -> Bar {
        Bar {
            time,
            open: 10.0,
            high: 11.0,
            low: 9.0,
            close: 10.5,
            volume: 100.0,
        }
    }

    #[test]
    fn finalize_result_reports_count_cap_truncation_for_date_range() {
        let request =
            validate_bars_range_request("NASDAQ:AAPL", "1D", "2020-01-01", "2020-01-31", 1)
                .unwrap();
        let result = finalize_result(
            &request,
            vec![bar(1_577_836_800), bar(1_577_923_200)],
            true,
            BarsWaitSummary::new(&request),
            0,
        );

        assert_eq!(result.bars.len(), 1);
        assert_eq!(result.fetch_summary.fetch_window_count, 1);
        assert_eq!(result.fetch_summary.request_more_count, 0);
        assert_eq!(result.fetch_summary.initial_fetch_count, 500);
        assert_eq!(result.fetch_summary.requested_count_cap, 1);
        assert_eq!(result.fetch_summary.observed_count, 2);
        assert_eq!(result.fetch_summary.filtered_count, 2);
        assert_eq!(result.fetch_summary.returned_count, 1);
        assert!(result.fetch_summary.range_truncated);
        assert_eq!(result.fetch_summary.range_truncation_reason, "count_cap");
    }

    #[test]
    fn finalize_result_reports_request_more_fetch_windows() {
        let request =
            validate_bars_range_request("NASDAQ:AAPL", "1W", "2020-01-01", "2020-12-31", 500)
                .unwrap();
        let result = finalize_result(
            &request,
            vec![bar(1_577_836_800), bar(1_609_372_800)],
            true,
            BarsWaitSummary::new(&request),
            2,
        );

        assert_eq!(result.fetch_summary.fetch_window_count, 3);
        assert_eq!(result.fetch_summary.request_more_count, 2);
        assert_eq!(result.fetch_summary.observed_count, 2);
        assert_eq!(result.fetch_summary.filtered_count, 2);
        assert_eq!(result.fetch_summary.returned_count, 2);
        assert!(!result.fetch_summary.range_truncated);
        assert_eq!(result.fetch_summary.range_truncation_reason, "none");
    }

    #[test]
    fn finalize_result_leaves_count_only_intraday_summary_untruncated() {
        let request = validate_bars_request("NASDAQ:AAPL", "5", 2).unwrap();
        let result = finalize_result(
            &request,
            vec![bar(1), bar(2)],
            true,
            BarsWaitSummary::new(&request),
            0,
        );

        assert_eq!(result.fetch_summary.initial_fetch_count, 2);
        assert_eq!(result.fetch_summary.requested_count_cap, 2);
        assert!(!result.fetch_summary.range_truncated);
        assert_eq!(result.fetch_summary.range_truncation_reason, "none");
    }
}
