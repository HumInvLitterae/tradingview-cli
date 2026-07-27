use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::time::Instant;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{Message, client::IntoClientRequest},
};
use tradingview_core::{AppError, ErrorKind};

use super::{
    BarsFailureStage,
    payload::{bars_error_details, bars_source_availability},
    protocol::{
        WsPacket, frame, merge_bars, parse_bars_from_message, parse_packets, pong_frame, session_id,
    },
    types::{
        Bar, BarsAvailabilityState, BarsFetchSummary, BarsFetchSummaryInput, BarsRequest,
        BarsResult, BarsWaitSummary,
    },
    with_source_failure_stage,
};

const WS_ENDPOINT: &str = "wss://data.tradingview.com/socket.io/websocket?type=chart";

pub(super) async fn fetch_bars_ws(request: &BarsRequest) -> Result<BarsResult, AppError> {
    fetch_bars_ws_from_endpoint(request, WS_ENDPOINT).await
}

async fn fetch_bars_ws_from_endpoint(
    request: &BarsRequest,
    endpoint: &str,
) -> Result<BarsResult, AppError> {
    let started = Instant::now();
    let mut wait_summary = BarsWaitSummary::new(request);
    let mut ws_request = endpoint.into_client_request().map_err(|err| {
        with_source_failure_stage(
            AppError::new(
                ErrorKind::InternalApiUnavailable,
                format!("Could not prepare TradingView WebSocket request: {err}"),
            ),
            BarsFailureStage::RequestPrepare,
        )
    })?;
    ws_request.headers_mut().insert(
        "Origin",
        "https://www.tradingview.com"
            .parse()
            .expect("valid origin header"),
    );

    let mut stream = connect_ws(ws_request, request.timeout)
        .await
        .map_err(|err| {
            let unavailable_reason = if err.kind == ErrorKind::Timeout {
                "connection_timeout"
            } else {
                "connection_failed"
            };
            let timed_out = err.kind == ErrorKind::Timeout;
            with_source_failure_stage(
                err.with_details(bars_error_details(
                    request,
                    json!({
                        "availability_status": "unavailable",
                        "source_availability": bars_source_availability(
                            request,
                            BarsAvailabilityState::unavailable(unavailable_reason, 0, false, timed_out),
                            &wait_summary,
                            started.elapsed().as_millis() as u64,
                        ),
                    }),
                )),
                BarsFailureStage::WebSocketConnect,
            )
        })?;

    let session_id = session_id("cs_");
    let symbol_key = "symbol_1";
    let setup_deadline = Instant::now() + request.timeout;
    send_initial_setup(
        &mut stream,
        setup_deadline,
        &session_id,
        symbol_key,
        TransportDiagnostics {
            request,
            bars: &[],
            request_more_count: 0,
            wait_summary: &wait_summary,
            elapsed_ms: started.elapsed().as_millis() as u64,
        },
    )
    .await?;

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
                return Err(with_source_failure_stage(
                    AppError::new(
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
                    )),
                    BarsFailureStage::ResponseWait,
                ));
            }
            Ok(None) => {
                return Err(with_source_failure_stage(
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
                    BarsFailureStage::ResponseWait,
                ));
            }
            Err(_) => {
                if bars.is_empty() {
                    return Err(with_source_failure_stage(
                        AppError::new(ErrorKind::Timeout, "Bars request timed out").with_details(
                            bars_error_details(
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
                            ),
                        ),
                        BarsFailureStage::ResponseWait,
                    ));
                }
                break;
            }
        };

        let packets = parse_packets(message).map_err(|err| {
            wait_summary.error_messages_seen += 1;
            let stage = if err.kind == ErrorKind::Connection {
                BarsFailureStage::ResponseWait
            } else {
                BarsFailureStage::Protocol
            };
            with_source_failure_stage(
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
                )),
                stage,
            )
        })?;
        wait_summary.websocket_packets_seen += packets.len() as u64;

        for packet in packets {
            match packet {
                WsPacket::Ping(value) => {
                    send_heartbeat_pong(
                        &mut stream,
                        deadline,
                        value,
                        HeartbeatDiagnostics {
                            request,
                            bars: &bars,
                            request_more_count,
                            wait_summary: &wait_summary,
                            elapsed_ms: started.elapsed().as_millis() as u64,
                        },
                    )
                    .await?;
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
                            if should_stop_for_no_progress(last_more_oldest, oldest_time) {
                                break 'read_loop;
                            }
                            last_more_oldest = oldest_time;
                            completed = false;
                            send_pagination_request(
                                &mut stream,
                                deadline,
                                &session_id,
                                TransportDiagnostics {
                                    request,
                                    bars: &bars,
                                    request_more_count,
                                    wait_summary: &wait_summary,
                                    elapsed_ms: started.elapsed().as_millis() as u64,
                                },
                            )
                            .await?;
                            request_more_count += 1;
                            continue;
                        }
                        cleanup_ws(&mut stream, &session_id, request.timeout).await;
                        return Ok(finalize_result(
                            request,
                            bars,
                            completed,
                            wait_summary,
                            request_more_count,
                        ));
                    } else if matches!(method, "symbol_error" | "series_error" | "protocol_error") {
                        wait_summary.error_messages_seen += 1;
                        cleanup_ws(&mut stream, &session_id, request.timeout).await;
                        return Err(with_source_failure_stage(
                            AppError::new(
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
                            )),
                            BarsFailureStage::Protocol,
                        ));
                    }
                }
            }
        }
    }

    cleanup_ws(&mut stream, &session_id, request.timeout).await;
    Ok(finalize_result(
        request,
        bars,
        completed,
        wait_summary,
        request_more_count,
    ))
}

#[derive(Clone, Copy)]
struct TransportDiagnostics<'a> {
    request: &'a BarsRequest,
    bars: &'a [Bar],
    request_more_count: usize,
    wait_summary: &'a BarsWaitSummary,
    elapsed_ms: u64,
}

async fn send_initial_setup<S>(
    stream: &mut S,
    deadline: Instant,
    session_id: &str,
    symbol_key: &str,
    diagnostics: TransportDiagnostics<'_>,
) -> Result<(), AppError>
where
    S: futures_util::Sink<Message> + Unpin,
    S::Error: std::fmt::Display,
{
    let sends = [
        (
            "set_auth_token",
            json!(["unauthorized_user_token"]),
            BarsFailureStage::SessionSetup,
        ),
        (
            "chart_create_session",
            json!([session_id, ""]),
            BarsFailureStage::SessionSetup,
        ),
        (
            "resolve_symbol",
            json!([
                session_id,
                symbol_key,
                format!(
                    "={}",
                    json!({
                        "symbol": diagnostics.request.symbol,
                        "adjustment": "splits",
                    })
                )
            ]),
            BarsFailureStage::SeriesSetup,
        ),
        (
            "create_series",
            json!([
                session_id,
                "s1",
                "s1",
                symbol_key,
                diagnostics.request.timeframe,
                diagnostics.request.initial_fetch_count()
            ]),
            BarsFailureStage::SeriesSetup,
        ),
        (
            "switch_timezone",
            json!([session_id, "Etc/UTC"]),
            BarsFailureStage::SessionSetup,
        ),
    ];

    for (method, params, stage) in sends {
        send_ws(stream, deadline, method, params)
            .await
            .map_err(|error| with_transport_diagnostics(error, stage, diagnostics))?;
    }
    Ok(())
}

async fn send_pagination_request<S>(
    stream: &mut S,
    deadline: Instant,
    session_id: &str,
    diagnostics: TransportDiagnostics<'_>,
) -> Result<(), AppError>
where
    S: futures_util::Sink<Message> + Unpin,
    S::Error: std::fmt::Display,
{
    send_ws(
        stream,
        deadline,
        "request_more_data",
        json!([session_id, "s1", diagnostics.request.initial_fetch_count()]),
    )
    .await
    .map_err(|error| with_transport_diagnostics(error, BarsFailureStage::Pagination, diagnostics))
}

async fn send_ws<S>(
    stream: &mut S,
    deadline: Instant,
    method: &str,
    params: Value,
) -> Result<(), AppError>
where
    S: futures_util::Sink<Message> + Unpin,
    S::Error: std::fmt::Display,
{
    let payload = json!({ "m": method, "p": params }).to_string();
    send_message(
        stream,
        deadline,
        Message::Text(frame(&payload).into()),
        "TradingView WebSocket send",
    )
    .await
}

fn with_transport_diagnostics(
    error: AppError,
    stage: BarsFailureStage,
    diagnostics: TransportDiagnostics<'_>,
) -> AppError {
    let unavailable_reason = send_unavailable_reason(&error);
    let timed_out = error.kind == ErrorKind::Timeout;
    with_source_failure_stage(
        error.with_details(bars_error_details(
            diagnostics.request,
            json!({
                "availability_status": "unavailable",
                "source_availability": bars_source_availability(
                    diagnostics.request,
                    BarsAvailabilityState::unavailable(
                        unavailable_reason,
                        diagnostics.bars.len(),
                        false,
                        timed_out,
                    ),
                    diagnostics.wait_summary,
                    diagnostics.elapsed_ms,
                ),
                "range_fetch_summary": range_fetch_summary_for_bars(
                    diagnostics.request,
                    diagnostics.bars,
                    diagnostics.request_more_count,
                    false,
                ),
            }),
        )),
        stage,
    )
}

async fn connect_ws<R>(
    request: R,
    timeout: std::time::Duration,
) -> Result<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>, AppError>
where
    R: IntoClientRequest + Unpin,
{
    tokio::time::timeout(timeout, connect_async(request))
        .await
        .map_err(|_| {
            AppError::new(
                ErrorKind::Timeout,
                "TradingView WebSocket connection timed out",
            )
        })?
        .map(|(stream, _)| stream)
        .map_err(|err| {
            AppError::new(
                ErrorKind::Connection,
                format!("TradingView WebSocket connection failed: {err}"),
            )
        })
}

async fn send_message<S>(
    stream: &mut S,
    deadline: Instant,
    message: Message,
    operation: &str,
) -> Result<(), AppError>
where
    S: futures_util::Sink<Message> + Unpin,
    S::Error: std::fmt::Display,
{
    tokio::time::timeout_at(deadline, stream.send(message))
        .await
        .map_err(|_| AppError::new(ErrorKind::Timeout, format!("{operation} timed out")))?
        .map_err(|err| AppError::new(ErrorKind::Connection, format!("{operation} failed: {err}")))
}

type HeartbeatDiagnostics<'a> = TransportDiagnostics<'a>;

async fn send_heartbeat_pong<S>(
    stream: &mut S,
    deadline: Instant,
    value: i64,
    diagnostics: HeartbeatDiagnostics<'_>,
) -> Result<(), AppError>
where
    S: futures_util::Sink<Message> + Unpin,
    S::Error: std::fmt::Display,
{
    send_message(
        stream,
        deadline,
        Message::Text(pong_frame(value).into()),
        "TradingView WebSocket pong",
    )
    .await
    .map_err(|error| {
        with_transport_diagnostics(error, BarsFailureStage::HeartbeatSend, diagnostics)
    })
}

async fn cleanup_ws<S>(stream: &mut S, session_id: &str, timeout: std::time::Duration)
where
    S: futures_util::Sink<Message> + Unpin,
    S::Error: std::fmt::Display,
{
    let deadline = Instant::now() + timeout;
    let _ = send_ws(
        stream,
        deadline,
        "chart_delete_session",
        json!([session_id]),
    )
    .await;
    let _ = tokio::time::timeout_at(deadline, stream.close()).await;
}

fn send_unavailable_reason(error: &AppError) -> &'static str {
    if error.kind == ErrorKind::Timeout {
        "websocket_send_timeout"
    } else {
        "connection_failed"
    }
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

fn should_stop_for_no_progress(previous_oldest: Option<i64>, oldest: Option<i64>) -> bool {
    oldest.is_some() && oldest == previous_oldest
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
    use futures_util::Sink;
    use std::{
        convert::Infallible,
        pin::Pin,
        task::{Context, Poll},
    };
    use tokio::{net::TcpListener, time::Duration};
    use tokio_tungstenite::accept_async;

    use super::*;
    use crate::bars::validation::{validate_bars_range_request, validate_bars_request};

    struct NeverReadySink;

    #[derive(Default)]
    struct RecordingSink {
        messages: Vec<Message>,
    }

    struct FailNthSink {
        attempts: usize,
        fail_at: usize,
        messages: Vec<Message>,
    }

    #[derive(Debug)]
    struct HeartbeatProbeEvidence {
        heartbeat_count: usize,
        pong_count: usize,
        post_pong_request_more_count: usize,
        post_request_update_count: usize,
        post_request_completion_count: usize,
    }

    impl Sink<Message> for NeverReadySink {
        type Error = Infallible;

        fn poll_ready(
            self: Pin<&mut Self>,
            _context: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Pending
        }

        fn start_send(self: Pin<&mut Self>, _item: Message) -> Result<(), Self::Error> {
            unreachable!("pending sink must never accept an item")
        }

        fn poll_flush(
            self: Pin<&mut Self>,
            _context: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Pending
        }

        fn poll_close(
            self: Pin<&mut Self>,
            _context: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Pending
        }
    }

    impl Sink<Message> for RecordingSink {
        type Error = Infallible;

        fn poll_ready(
            self: Pin<&mut Self>,
            _context: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
            self.messages.push(item);
            Ok(())
        }

        fn poll_flush(
            self: Pin<&mut Self>,
            _context: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(
            self: Pin<&mut Self>,
            _context: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
    }

    impl Sink<Message> for FailNthSink {
        type Error = &'static str;

        fn poll_ready(
            self: Pin<&mut Self>,
            _context: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
            self.attempts += 1;
            if self.attempts == self.fail_at {
                return Err("fixed send failure");
            }
            self.messages.push(item);
            Ok(())
        }

        fn poll_flush(
            self: Pin<&mut Self>,
            _context: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(
            self: Pin<&mut Self>,
            _context: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
    }

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

    async fn scripted_endpoint(message: Option<Message>) -> (String, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut websocket = accept_async(stream).await.unwrap();
            for _ in 0..5 {
                websocket.next().await.unwrap().unwrap();
            }
            if let Some(message) = message {
                websocket.send(message).await.unwrap();
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
        });
        (format!("ws://{address}"), server)
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
    fn finalize_result_preserves_one_minute_range_boundaries() {
        let request =
            validate_bars_range_request("NASDAQ:AAPL", "1m", "2020-01-01", "2020-01-01", 5000)
                .unwrap();
        let from = 1_577_836_800;
        let to_exclusive = from + 86_400;
        let result = finalize_result(
            &request,
            vec![
                bar(from - 60),
                bar(from),
                bar(from + 60),
                bar(to_exclusive - 60),
                bar(to_exclusive),
            ],
            true,
            BarsWaitSummary::new(&request),
            1,
        );

        assert_eq!(request.timeframe, "1");
        assert_eq!(result.bars.len(), 3);
        assert_eq!(result.bars[0].time, from);
        assert_eq!(result.bars[1].time, from + 60);
        assert_eq!(result.bars[2].time, to_exclusive - 60);
        assert_eq!(result.fetch_summary.fetch_window_count, 2);
        assert_eq!(result.fetch_summary.filtered_count, 3);
        assert_eq!(result.fetch_summary.returned_count, 3);
        assert!(!result.fetch_summary.range_truncated);
        assert_eq!(result.fetch_summary.range_truncation_reason, "none");
    }

    #[test]
    fn one_minute_range_reports_timeout_and_source_exhaustion_without_synthetic_bars() {
        let request =
            validate_bars_range_request("NASDAQ:AAPL", "1", "2020-01-04", "2020-01-05", 500)
                .unwrap();

        let timeout = finalize_result(
            &request,
            vec![bar(1_578_182_400)],
            false,
            BarsWaitSummary::new(&request),
            1,
        );
        assert!(timeout.fetch_summary.range_truncated);
        assert_eq!(timeout.fetch_summary.range_truncation_reason, "timeout");

        let closure_shaped = finalize_result(
            &request,
            Vec::new(),
            true,
            BarsWaitSummary::new(&request),
            1,
        );
        assert!(closure_shaped.bars.is_empty());
        assert!(closure_shaped.fetch_summary.range_truncated);
        assert_eq!(
            closure_shaped.fetch_summary.range_truncation_reason,
            "source_exhausted"
        );
    }

    #[test]
    fn repeated_oldest_timestamp_stops_request_more_without_resetting_progress() {
        assert!(!should_stop_for_no_progress(None, Some(100)));
        assert!(!should_stop_for_no_progress(Some(100), Some(99)));
        assert!(should_stop_for_no_progress(Some(100), Some(100)));
        assert!(!should_stop_for_no_progress(Some(100), None));
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

    #[tokio::test]
    async fn stalled_websocket_handshake_maps_to_timeout() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (_stream, _) = listener.accept().await.unwrap();
            tokio::time::sleep(Duration::from_millis(200)).await;
        });

        let error = match connect_ws(format!("ws://{address}"), Duration::from_millis(50)).await {
            Ok(_) => panic!("stalled WebSocket handshake should time out"),
            Err(error) => error,
        };
        assert_eq!(error.kind, ErrorKind::Timeout);
        server.await.unwrap();
    }

    #[tokio::test]
    async fn production_fetch_path_classifies_request_connect_response_and_protocol_failures() {
        let mut request = validate_bars_request("NASDAQ:AAPL", "1D", 5).unwrap();
        request.timeout = Duration::from_millis(100);

        let error = fetch_bars_ws_from_endpoint(&request, "not a websocket endpoint")
            .await
            .unwrap_err();
        assert_eq!(
            error.details.as_ref().unwrap()["source_failure_stage"],
            "request_prepare"
        );

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (_stream, _) = listener.accept().await.unwrap();
            tokio::time::sleep(Duration::from_millis(200)).await;
        });
        let error = fetch_bars_ws_from_endpoint(&request, &format!("ws://{address}"))
            .await
            .unwrap_err();
        assert_eq!(
            error.details.as_ref().unwrap()["source_failure_stage"],
            "websocket_connect"
        );
        server.await.unwrap();

        let (endpoint, server) = scripted_endpoint(None).await;
        let error = fetch_bars_ws_from_endpoint(&request, &endpoint)
            .await
            .unwrap_err();
        assert_eq!(
            error.details.as_ref().unwrap()["source_failure_stage"],
            "response_wait"
        );
        server.await.unwrap();

        let (endpoint, server) =
            scripted_endpoint(Some(Message::Text(frame("not-json").into()))).await;
        let error = fetch_bars_ws_from_endpoint(&request, &endpoint)
            .await
            .unwrap_err();
        assert_eq!(
            error.details.as_ref().unwrap()["source_failure_stage"],
            "protocol"
        );
        server.await.unwrap();
    }

    #[tokio::test]
    async fn pending_send_maps_to_timeout() {
        let mut pending = NeverReadySink;
        let error = send_ws(
            &mut pending,
            Instant::now() + Duration::from_millis(50),
            "set_auth_token",
            json!(["unauthorized_user_token"]),
        )
        .await
        .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Timeout);
    }

    #[tokio::test]
    async fn initial_setup_preserves_send_order_and_exact_stage_mapping() {
        let request = validate_bars_request("NASDAQ:AAPL", "1D", 5).unwrap();
        let wait_summary = BarsWaitSummary::new(&request);

        for (fail_at, expected_stage) in [
            (1, "session_setup"),
            (2, "session_setup"),
            (3, "series_setup"),
            (4, "series_setup"),
            (5, "session_setup"),
        ] {
            let mut sink = FailNthSink {
                attempts: 0,
                fail_at,
                messages: Vec::new(),
            };
            let error = send_initial_setup(
                &mut sink,
                Instant::now() + Duration::from_secs(1),
                "cs_fixture",
                "symbol_1",
                TransportDiagnostics {
                    request: &request,
                    bars: &[],
                    request_more_count: 0,
                    wait_summary: &wait_summary,
                    elapsed_ms: 1,
                },
            )
            .await
            .unwrap_err();

            assert_eq!(sink.attempts, fail_at);
            assert_eq!(
                error.details.as_ref().unwrap()["source_failure_stage"],
                expected_stage
            );
            assert_eq!(error.details.as_ref().unwrap()["requested_count"], 5);
        }

        let mut sink = RecordingSink::default();
        send_initial_setup(
            &mut sink,
            Instant::now() + Duration::from_secs(1),
            "cs_fixture",
            "symbol_1",
            TransportDiagnostics {
                request: &request,
                bars: &[],
                request_more_count: 0,
                wait_summary: &wait_summary,
                elapsed_ms: 1,
            },
        )
        .await
        .unwrap();
        let frames = sink
            .messages
            .iter()
            .map(|message| message.to_text().unwrap())
            .collect::<Vec<_>>();
        for (frame, method) in frames.iter().zip([
            "set_auth_token",
            "chart_create_session",
            "resolve_symbol",
            "create_series",
            "switch_timezone",
        ]) {
            assert!(frame.contains(&format!("\"m\":\"{method}\"")));
        }
    }

    #[tokio::test]
    async fn pagination_failure_preserves_partial_diagnostics_and_stage() {
        let request =
            validate_bars_range_request("NASDAQ:AAPL", "1", "2020-01-01", "2020-01-02", 500)
                .unwrap();
        let bars = vec![bar(1_577_836_800)];
        let wait_summary = BarsWaitSummary::new(&request);
        let mut pending = NeverReadySink;

        let error = send_pagination_request(
            &mut pending,
            Instant::now() + Duration::from_millis(50),
            "cs_fixture",
            TransportDiagnostics {
                request: &request,
                bars: &bars,
                request_more_count: 1,
                wait_summary: &wait_summary,
                elapsed_ms: 25,
            },
        )
        .await
        .unwrap_err();
        let details = error.details.unwrap();
        assert_eq!(details["source_failure_stage"], "pagination");
        assert_eq!(details["source_availability"]["bar_count"], 1);
        assert_eq!(details["range_fetch_summary"]["request_more_count"], 1);
    }

    #[tokio::test]
    async fn pending_heartbeat_pong_preserves_bars_timeout_diagnostics() {
        let request =
            validate_bars_range_request("NASDAQ:AAPL", "1D", "2020-01-01", "2020-01-31", 500)
                .unwrap();
        let bars = vec![bar(1_577_836_800)];
        let wait_summary = BarsWaitSummary::new(&request);
        let mut pending = NeverReadySink;

        let error = send_heartbeat_pong(
            &mut pending,
            Instant::now() + Duration::from_millis(50),
            123,
            HeartbeatDiagnostics {
                request: &request,
                bars: &bars,
                request_more_count: 1,
                wait_summary: &wait_summary,
                elapsed_ms: 25,
            },
        )
        .await
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Timeout);
        let details = error.details.unwrap();
        assert_eq!(details["source"], "tradingview_bars_ws");
        assert_eq!(details["availability_status"], "unavailable");
        assert_eq!(
            details["source_availability"]["unavailable_reason"],
            "websocket_send_timeout"
        );
        assert_eq!(details["source_availability"]["bar_count"], 1);
        assert_eq!(details["source_availability"]["timed_out"], true);
        assert!(details["source_availability"].get("wait_summary").is_some());
        assert_eq!(details["range_fetch_summary"]["request_more_count"], 1);
        assert_eq!(details["source_failure_stage"], "heartbeat_send");
    }

    #[tokio::test]
    async fn heartbeat_pong_sends_one_canonical_frame() {
        let request = validate_bars_request("NASDAQ:AAPL", "1D", 5).unwrap();
        let wait_summary = BarsWaitSummary::new(&request);
        let mut sink = RecordingSink::default();

        send_heartbeat_pong(
            &mut sink,
            Instant::now() + Duration::from_secs(1),
            42,
            HeartbeatDiagnostics {
                request: &request,
                bars: &[],
                request_more_count: 0,
                wait_summary: &wait_summary,
                elapsed_ms: 0,
            },
        )
        .await
        .unwrap();

        assert_eq!(sink.messages.len(), 1);
        assert_eq!(sink.messages[0], Message::Text("~m~5~m~~h~42".into()));
    }

    #[tokio::test]
    #[ignore = "requires TradingView WebSocket availability and TV_LIVE_BARS_HEARTBEAT_SMOKE=1"]
    async fn canonical_heartbeat_pong_live_probe() {
        if std::env::var("TV_LIVE_BARS_HEARTBEAT_SMOKE")
            .ok()
            .as_deref()
            != Some("1")
        {
            panic!(
                "heartbeat live probe is gated; set TV_LIVE_BARS_HEARTBEAT_SMOKE=1 and run with --ignored"
            );
        }

        let evidence = run_canonical_heartbeat_live_probe()
            .await
            .expect("public-safe heartbeat probe should complete");
        assert!(evidence.heartbeat_count > 0);
        assert_eq!(evidence.pong_count, 1);
        assert_eq!(evidence.post_pong_request_more_count, 1);
        assert!(evidence.post_request_update_count + evidence.post_request_completion_count > 0);
        println!(
            "heartbeat live probe passed: heartbeat_count={} pong_count={} post_pong_request_more_count={} post_request_update_count={} post_request_completion_count={} connection_usable_after_pong=true",
            evidence.heartbeat_count,
            evidence.pong_count,
            evidence.post_pong_request_more_count,
            evidence.post_request_update_count,
            evidence.post_request_completion_count,
        );
    }

    async fn run_canonical_heartbeat_live_probe() -> Result<HeartbeatProbeEvidence, AppError> {
        let request = validate_bars_request("NASDAQ:AAPL", "1", 5)?;
        let mut ws_request = WS_ENDPOINT.into_client_request().map_err(|err| {
            AppError::new(
                ErrorKind::InternalApiUnavailable,
                format!("Could not prepare heartbeat probe request: {err}"),
            )
        })?;
        ws_request.headers_mut().insert(
            "Origin",
            "https://www.tradingview.com"
                .parse()
                .expect("valid origin header"),
        );
        let mut stream = connect_ws(ws_request, request.timeout).await?;
        let chart_session = session_id("cs_");
        let symbol_key = "symbol_1";
        let setup_deadline = Instant::now() + request.timeout;
        for (method, params) in [
            ("set_auth_token", json!(["unauthorized_user_token"])),
            ("chart_create_session", json!([chart_session, ""])),
            (
                "resolve_symbol",
                json!([
                    chart_session,
                    symbol_key,
                    format!(
                        "={}",
                        json!({"symbol": request.symbol, "adjustment": "splits"})
                    )
                ]),
            ),
            (
                "create_series",
                json!([
                    chart_session,
                    "s1",
                    "s1",
                    symbol_key,
                    request.timeframe,
                    request.initial_fetch_count()
                ]),
            ),
            ("switch_timezone", json!([chart_session, "Etc/UTC"])),
        ] {
            send_ws(&mut stream, setup_deadline, method, params).await?;
        }

        let deadline = Instant::now() + Duration::from_secs(45);
        let mut heartbeat_count = 0usize;
        let mut pong_count = 0usize;
        let mut post_pong_request_more_count = 0usize;
        let mut post_request_update_count = 0usize;
        let mut post_request_completion_count = 0usize;
        let mut initial_series_completed = false;
        let mut websocket_message_index = 0usize;
        let mut post_pong_request_message_index = None;

        while Instant::now() < deadline {
            let next = tokio::time::timeout_at(deadline, stream.next())
                .await
                .map_err(|_| {
                    AppError::new(ErrorKind::Timeout, "Heartbeat live probe timed out")
                        .with_details(json!({
                            "heartbeat_count": heartbeat_count,
                            "pong_count": pong_count,
                            "post_pong_request_more_count": post_pong_request_more_count,
                            "post_request_update_count": post_request_update_count,
                            "post_request_completion_count": post_request_completion_count,
                        }))
                })?;
            let Some(message) = next else {
                return Err(AppError::new(
                    ErrorKind::Connection,
                    "Heartbeat live probe connection closed",
                ));
            };
            let message = message.map_err(|err| {
                AppError::new(
                    ErrorKind::Connection,
                    format!("Heartbeat live probe read failed: {err}"),
                )
            })?;
            websocket_message_index += 1;

            for packet in parse_packets(message)? {
                match packet {
                    WsPacket::Ping(value) => {
                        heartbeat_count += 1;
                        if pong_count == 0 {
                            let candidate = frame(&format!("~h~{value}"));
                            send_message(
                                &mut stream,
                                deadline,
                                Message::Text(candidate.into()),
                                "Heartbeat live probe pong",
                            )
                            .await?;
                            pong_count = 1;
                            if initial_series_completed {
                                send_ws(
                                    &mut stream,
                                    deadline,
                                    "request_more_data",
                                    json!([chart_session, "s1", 5]),
                                )
                                .await?;
                                post_pong_request_more_count = 1;
                                post_pong_request_message_index = Some(websocket_message_index);
                            }
                        }
                    }
                    WsPacket::Message(value) => {
                        let method = value.get("m").and_then(Value::as_str).unwrap_or_default();
                        let received_after_request = post_pong_request_message_index
                            .is_some_and(|index| websocket_message_index > index);
                        if method == "series_completed" {
                            if received_after_request {
                                post_request_completion_count += 1;
                            }
                            initial_series_completed = true;
                            if pong_count > 0 && post_pong_request_more_count == 0 {
                                send_ws(
                                    &mut stream,
                                    deadline,
                                    "request_more_data",
                                    json!([chart_session, "s1", 5]),
                                )
                                .await?;
                                post_pong_request_more_count = 1;
                                post_pong_request_message_index = Some(websocket_message_index);
                            }
                        } else if received_after_request
                            && matches!(method, "timescale_update" | "du")
                        {
                            post_request_update_count += 1;
                        }

                        if heartbeat_count > 0
                            && pong_count == 1
                            && post_pong_request_more_count == 1
                            && post_request_update_count + post_request_completion_count > 0
                        {
                            cleanup_ws(&mut stream, &chart_session, request.timeout).await;
                            return Ok(HeartbeatProbeEvidence {
                                heartbeat_count,
                                pong_count,
                                post_pong_request_more_count,
                                post_request_update_count,
                                post_request_completion_count,
                            });
                        }
                    }
                }
            }
        }

        cleanup_ws(&mut stream, &chart_session, request.timeout).await;
        Err(
            AppError::new(ErrorKind::Timeout, "Heartbeat live probe timed out").with_details(
                json!({
                    "heartbeat_count": heartbeat_count,
                    "pong_count": pong_count,
                    "post_pong_request_more_count": post_pong_request_more_count,
                    "post_request_update_count": post_request_update_count,
                    "post_request_completion_count": post_request_completion_count,
                }),
            ),
        )
    }
}
