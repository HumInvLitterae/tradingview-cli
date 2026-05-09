use std::{collections::HashMap, time::Duration};

use serde_json::{Value, json};
use tradingview_cdp::CdpClient;
use tradingview_core::{AppError, ErrorKind};

#[cfg(not(test))]
const QUOTE_DATA_WAIT: Duration = Duration::from_millis(3_500);
#[cfg(test)]
const QUOTE_DATA_WAIT: Duration = Duration::from_millis(50);
const EVENT_POLL_TIMEOUT: Duration = Duration::from_millis(250);
const QUOTE_DATA_CONTRACT_VERSION: &str = "quote_data.v1";

pub async fn quote_data(runtime: &mut CdpClient, symbol: &str) -> Result<Value, AppError> {
    let requested_symbol = symbol.trim();
    if requested_symbol.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "quote-data source requires SYMBOL",
        ));
    }

    runtime.call_method("Network.enable", json!({})).await?;
    let mut observer = QuoteDataObserver::new(requested_symbol);
    let started = std::time::Instant::now();
    while started.elapsed() < QUOTE_DATA_WAIT {
        let remaining = QUOTE_DATA_WAIT.saturating_sub(started.elapsed());
        let timeout = remaining.min(EVENT_POLL_TIMEOUT);
        let event = match runtime.next_event(timeout).await {
            Ok(Some(event)) => event,
            Ok(None) => continue,
            Err(err) if err.kind == ErrorKind::Timeout => continue,
            Err(err) => return Err(err),
        };
        if let Some(candidate) = observer.handle_event(&event) {
            let elapsed = started.elapsed();
            let wait_summary = observer.summary(elapsed);
            return Ok(success_payload(
                requested_symbol,
                candidate,
                elapsed,
                wait_summary,
            ));
        }
    }

    Err(unavailable_error(
        requested_symbol,
        observer.summary(started.elapsed()),
    ))
}

#[derive(Debug, Clone, PartialEq)]
struct QuoteDataCandidate {
    symbol: String,
    rtc: Value,
    rtc_time: Value,
    rch: Value,
    rchp: Value,
    current_session: Value,
    market_phase: Value,
    update_mode: Value,
}

#[derive(Debug)]
struct QuoteDataObserver<'a> {
    requested_symbol: &'a str,
    quote_session_symbols: HashMap<String, Vec<String>>,
    websocket_events_seen: u64,
    websocket_frames_seen: u64,
    qsd_messages_seen: u64,
    qsd_with_rtc_seen: u64,
    matching_symbol_qsd_seen: u64,
    matching_symbol_without_rtc_seen: u64,
    matching_qsd_messages_seen: u64,
    quote_session_symbol_mappings_seen: u64,
}

impl<'a> QuoteDataObserver<'a> {
    fn new(requested_symbol: &'a str) -> Self {
        Self {
            requested_symbol,
            quote_session_symbols: HashMap::new(),
            websocket_events_seen: 0,
            websocket_frames_seen: 0,
            qsd_messages_seen: 0,
            qsd_with_rtc_seen: 0,
            matching_symbol_qsd_seen: 0,
            matching_symbol_without_rtc_seen: 0,
            matching_qsd_messages_seen: 0,
            quote_session_symbol_mappings_seen: 0,
        }
    }

    fn handle_event(&mut self, event: &Value) -> Option<QuoteDataCandidate> {
        let method = event.get("method").and_then(Value::as_str)?;
        if method.starts_with("Network.webSocket") {
            self.websocket_events_seen += 1;
        }
        match method {
            "Network.webSocketFrameReceived" | "Network.webSocketFrameSent" => {
                self.websocket_frames_seen += 1;
                let payload = event
                    .pointer("/params/response/payloadData")
                    .and_then(Value::as_str)
                    .or_else(|| event.pointer("/params/payloadData").and_then(Value::as_str))?;
                if method == "Network.webSocketFrameSent" {
                    self.update_quote_session_symbols(payload);
                }
                self.quote_data_candidate(payload)
            }
            _ => None,
        }
    }

    fn update_quote_session_symbols(&mut self, payload: &str) {
        for text in tradingview_socket_messages(payload) {
            let Ok(value) = serde_json::from_str::<Value>(&text) else {
                continue;
            };
            let Some(method) = value.get("m").and_then(Value::as_str) else {
                continue;
            };
            if method != "quote_add_symbols" && method != "quote_remove_symbols" {
                continue;
            }
            let Some(session) = value.pointer("/p/0").and_then(Value::as_str) else {
                continue;
            };
            let symbols = value
                .get("p")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .skip(1)
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let entry = self
                .quote_session_symbols
                .entry(session.to_string())
                .or_default();
            if method == "quote_add_symbols" {
                self.quote_session_symbol_mappings_seen += symbols.len() as u64;
                for symbol in symbols {
                    if !entry.iter().any(|existing| existing == &symbol) {
                        entry.push(symbol);
                    }
                }
            } else {
                entry.retain(|existing| !symbols.iter().any(|symbol| symbol == existing));
            }
        }
    }

    fn quote_data_candidate(&mut self, payload: &str) -> Option<QuoteDataCandidate> {
        for text in tradingview_socket_messages(payload) {
            let Ok(value) = serde_json::from_str::<Value>(&text) else {
                continue;
            };
            if value.get("m").and_then(Value::as_str) != Some("qsd") {
                continue;
            }
            self.qsd_messages_seen += 1;
            let session = value
                .pointer("/p/0")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let Some(item) = value.pointer("/p/1") else {
                continue;
            };
            let values = item.get("v").unwrap_or(&Value::Null);
            let symbol = item
                .get("n")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .trim();
            let mapped_symbol = self.mapped_single_symbol(session);
            let item_matches_request = self.symbol_matches_item(symbol, values)
                || mapped_symbol
                    .as_deref()
                    .is_some_and(|mapped| symbol_matches(mapped, self.requested_symbol));
            let Some(rtc) = values.get("rtc").filter(|value| !value.is_null()) else {
                if item_matches_request {
                    self.matching_symbol_qsd_seen += 1;
                    self.matching_symbol_without_rtc_seen += 1;
                }
                continue;
            };
            self.qsd_with_rtc_seen += 1;
            if !item_matches_request {
                continue;
            }
            self.matching_symbol_qsd_seen += 1;
            self.matching_qsd_messages_seen += 1;
            return Some(QuoteDataCandidate {
                symbol: if symbol_matches(symbol, self.requested_symbol) {
                    symbol.to_string()
                } else {
                    mapped_symbol.unwrap_or_else(|| self.requested_symbol.to_string())
                },
                rtc: rtc.clone(),
                rtc_time: values.get("rtc_time").cloned().unwrap_or(Value::Null),
                rch: values.get("rch").cloned().unwrap_or(Value::Null),
                rchp: values.get("rchp").cloned().unwrap_or(Value::Null),
                current_session: values
                    .get("current_session")
                    .cloned()
                    .unwrap_or(Value::Null),
                market_phase: values
                    .get("market-status")
                    .and_then(|status| status.get("phase"))
                    .cloned()
                    .unwrap_or(Value::Null),
                update_mode: values.get("update_mode").cloned().unwrap_or(Value::Null),
            });
        }
        None
    }

    fn symbol_matches_item(&self, symbol: &str, values: &Value) -> bool {
        symbol_matches(symbol, self.requested_symbol)
            || ["pro_name", "original_name"]
                .iter()
                .filter_map(|key| values.get(*key).and_then(Value::as_str))
                .any(|candidate| symbol_matches(candidate, self.requested_symbol))
            || values
                .get("base_name")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .any(|candidate| symbol_matches(candidate, self.requested_symbol))
    }

    fn mapped_single_symbol(&self, session: &str) -> Option<String> {
        self.quote_session_symbols
            .get(session)
            .filter(|symbols| symbols.len() == 1)
            .and_then(|symbols| symbols.first())
            .cloned()
    }

    fn summary(&self, elapsed: Duration) -> Value {
        json!({
            "bounded_wait_ms": QUOTE_DATA_WAIT.as_millis(),
            "elapsed_ms": elapsed.as_millis(),
            "websocket_events_seen": self.websocket_events_seen,
            "websocket_frames_seen": self.websocket_frames_seen,
            "qsd_messages_seen": self.qsd_messages_seen,
            "qsd_with_rtc_seen": self.qsd_with_rtc_seen,
            "matching_symbol_qsd_seen": self.matching_symbol_qsd_seen,
            "matching_symbol_without_rtc_seen": self.matching_symbol_without_rtc_seen,
            "matching_qsd_messages_seen": self.matching_qsd_messages_seen,
            "quote_session_symbol_mappings_seen": self.quote_session_symbol_mappings_seen,
            "raw_frame_included": false,
        })
    }
}

fn success_payload(
    requested_symbol: &str,
    candidate: QuoteDataCandidate,
    elapsed: Duration,
    wait_summary: Value,
) -> Value {
    let session_readback = session_readback(
        candidate.market_phase.clone(),
        candidate.current_session.clone(),
    );
    json!({
        "contract_version": QUOTE_DATA_CONTRACT_VERSION,
        "source": "desktop_quote_data_ws",
        "source_category": "desktop_backed_read",
        "requires_desktop": true,
        "non_mutating": true,
        "requested_symbol": requested_symbol,
        "observed_symbol": candidate.symbol,
        "price_source": "tradingview_quote_data_qsd",
        "price_session": "unknown",
        "scanner_extended_hours_included": false,
        "chart_main_series_included": false,
        "bounded_wait_ms": QUOTE_DATA_WAIT.as_millis(),
        "elapsed_ms": elapsed.as_millis(),
        "source_availability": source_availability(true, true, wait_summary, None, false, None),
        "quote_data": {
            "rtc": candidate.rtc,
            "rtc_time": candidate.rtc_time,
            "rch": candidate.rch,
            "rchp": candidate.rchp,
            "current_session": candidate.current_session.clone(),
            "market_phase": candidate.market_phase.clone(),
            "update_mode": candidate.update_mode,
            "session_readback": session_readback,
        },
        "note": "quote-data source is a Desktop-backed WebSocket readback and is not scanner extended_hours or chart main-series quote",
    })
}

fn unavailable_error(requested_symbol: &str, wait_summary: Value) -> AppError {
    let unavailable_reason = unavailable_reason_from_summary(&wait_summary);
    let source_availability = source_availability(
        false,
        false,
        wait_summary.clone(),
        Some(unavailable_reason),
        true,
        Some(next_action_for_unavailable(unavailable_reason)),
    );
    AppError::new(
        ErrorKind::InternalApiUnavailable,
        "TradingView quote-data WebSocket did not provide qsd.rtc for the requested symbol within the bounded wait",
    )
    .with_details(json!({
        "contract_version": QUOTE_DATA_CONTRACT_VERSION,
        "source": "desktop_quote_data_ws",
        "source_category": "desktop_backed_read",
        "requires_desktop": true,
        "non_mutating": true,
        "requested_symbol": requested_symbol,
        "observed_symbol": Value::Null,
        "price_source": "tradingview_quote_data_qsd",
        "scanner_extended_hours_included": false,
        "chart_main_series_included": false,
        "source_availability": source_availability,
        "wait_summary": wait_summary,
        "next_action_hint": "Retry while the selected TradingView page is streaming the requested symbol, or use `--source scanner` for scanner REST extended_hours when delayed scanner data is acceptable.",
    }))
}

fn source_availability(
    available: bool,
    rtc_observed: bool,
    wait_summary: Value,
    unavailable_reason: Option<&str>,
    timed_out: bool,
    next_action: Option<&str>,
) -> Value {
    json!({
        "available": available,
        "status": if available { "available" } else { "unavailable" },
        "rtc_observed": rtc_observed,
        "unavailable_reason": unavailable_reason,
        "timed_out": timed_out,
        "next_action": next_action,
        "raw_frame_included": false,
        "wait_summary": wait_summary,
    })
}

fn session_readback(market_phase: Value, current_session: Value) -> Value {
    let market_phase_normalized =
        normalize_session_value(market_phase.as_str().unwrap_or_default());
    let current_session_normalized =
        normalize_session_value(current_session.as_str().unwrap_or_default());
    json!({
        "market_phase": market_phase,
        "market_phase_normalized": market_phase_normalized,
        "current_session": current_session,
        "current_session_normalized": current_session_normalized,
        "session_source": "tradingview_quote_data_fields",
        "session_inferred": false,
    })
}

fn normalize_session_value(value: &str) -> Value {
    let normalized = value
        .trim()
        .to_ascii_lowercase()
        .replace(['-', '_', ' '], "");
    if normalized.is_empty() {
        Value::Null
    } else {
        json!(normalized)
    }
}

fn unavailable_reason_from_summary(wait_summary: &Value) -> &'static str {
    if wait_summary
        .get("websocket_events_seen")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        == 0
    {
        "no_websocket_events"
    } else if wait_summary
        .get("websocket_frames_seen")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        == 0
    {
        "no_websocket_frames"
    } else if wait_summary
        .get("qsd_messages_seen")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        == 0
    {
        "no_qsd_messages"
    } else if wait_summary
        .get("matching_symbol_qsd_seen")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        == 0
    {
        "no_matching_symbol"
    } else {
        "no_rtc"
    }
}

fn next_action_for_unavailable(unavailable_reason: &str) -> &'static str {
    match unavailable_reason {
        "no_matching_symbol" => "check_desktop_streaming_symbol",
        "no_rtc" => "use_scanner_if_delayed_rest_ok",
        _ => "retry_quote_data",
    }
}

fn tradingview_socket_messages(payload: &str) -> Vec<String> {
    if !payload.contains("~m~") {
        return vec![payload.to_string()];
    }
    let mut messages = vec![];
    let mut index = 0;
    while let Some(prefix_offset) = payload[index..].find("~m~") {
        let prefix_start = index + prefix_offset;
        let len_start = prefix_start + 3;
        let Some(len_end_offset) = payload[len_start..].find("~m~") else {
            break;
        };
        let len_end = len_start + len_end_offset;
        let Ok(message_len) = payload[len_start..len_end].parse::<usize>() else {
            index = len_end + 3;
            continue;
        };
        let message_start = len_end + 3;
        let message_end = message_start.saturating_add(message_len);
        if message_end > payload.len() {
            break;
        }
        messages.push(payload[message_start..message_end].to_string());
        index = message_end;
    }
    messages
}

fn symbol_matches(candidate: &str, requested: &str) -> bool {
    let candidate = candidate.trim();
    if candidate.is_empty() {
        return false;
    }
    bare_symbol(candidate) == bare_symbol(requested)
}

fn bare_symbol(symbol: &str) -> String {
    symbol
        .split(':')
        .next_back()
        .unwrap_or(symbol)
        .trim()
        .to_ascii_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn qsd_event(payload: &str) -> Value {
        json!({
            "method": "Network.webSocketFrameReceived",
            "params": {
                "response": {
                    "payloadData": payload
                }
            }
        })
    }

    fn websocket_created_event() -> Value {
        json!({
            "method": "Network.webSocketCreated",
            "params": {
                "requestId": "public-test"
            }
        })
    }

    fn unavailable_details_from_summary(summary: Value) -> Value {
        unavailable_error("NASDAQ:RKLB", summary).details.unwrap()
    }

    #[test]
    fn session_readback_normalizes_spelling_without_inference() {
        let readback = session_readback(json!("post-market"), json!("pre market"));

        assert_eq!(readback["market_phase"], "post-market");
        assert_eq!(readback["market_phase_normalized"], "postmarket");
        assert_eq!(readback["current_session"], "pre market");
        assert_eq!(readback["current_session_normalized"], "premarket");
        assert_eq!(readback["session_inferred"], false);

        let unknown = session_readback(Value::Null, json!(""));
        assert_eq!(unknown["market_phase_normalized"], Value::Null);
        assert_eq!(unknown["current_session_normalized"], Value::Null);
    }

    #[test]
    fn quote_data_observer_extracts_rtc_readback() {
        let mut observer = QuoteDataObserver::new("NASDAQ:RKLB");
        let event = qsd_event(
            r#"{"m":"qsd","p":["qs",{"n":"NASDAQ:RKLB","s":"ok","v":{"rtc":104.55,"rtc_time":1778278370,"rch":-0.92,"rchp":-0.87,"current_session":"post_market","market-status":{"phase":"post-market"},"update_mode":"streaming"}}]}"#,
        );

        let candidate = observer.handle_event(&event).unwrap();

        assert_eq!(candidate.symbol, "NASDAQ:RKLB");
        assert_eq!(candidate.rtc, json!(104.55));
        assert_eq!(candidate.rtc_time, json!(1778278370));
        assert_eq!(candidate.current_session, json!("post_market"));
        assert_eq!(candidate.market_phase, json!("post-market"));
    }

    #[test]
    fn quote_data_observer_uses_single_symbol_mapping_without_raw_frame() {
        let mut observer = QuoteDataObserver::new("NASDAQ:RKLB");
        let sent = json!({
            "method": "Network.webSocketFrameSent",
            "params": {
                "response": {
                    "payloadData": r#"{"m":"quote_add_symbols","p":["qs_rklb","NASDAQ:RKLB"]}"#
                }
            }
        });
        observer.handle_event(&sent);
        let event = qsd_event(r#"{"m":"qsd","p":["qs_rklb",{"s":"ok","v":{"rtc":104.55}}]}"#);

        let candidate = observer.handle_event(&event).unwrap();

        assert_eq!(candidate.symbol, "NASDAQ:RKLB");
        assert_eq!(candidate.rtc, json!(104.55));
    }

    #[test]
    fn quote_data_observer_rejects_unattributed_deltas() {
        let mut observer = QuoteDataObserver::new("NASDAQ:RKLB");
        let event = qsd_event(r#"{"m":"qsd","p":["qs_watchlist",{"s":"ok","v":{"rtc":399.95}}]}"#);

        assert!(observer.handle_event(&event).is_none());
        assert_eq!(observer.summary(Duration::ZERO)["qsd_messages_seen"], 1);
        assert_eq!(
            observer.summary(Duration::ZERO)["matching_qsd_messages_seen"],
            0
        );
        assert_eq!(
            observer.summary(Duration::ZERO)["matching_symbol_qsd_seen"],
            0
        );
    }

    #[test]
    fn quote_data_observer_counts_matching_qsd_without_rtc() {
        let mut observer = QuoteDataObserver::new("NASDAQ:RKLB");
        let event = qsd_event(
            r#"{"m":"qsd","p":["qs",{"n":"NASDAQ:RKLB","s":"ok","v":{"current_session":"post_market"}}]}"#,
        );

        assert!(observer.handle_event(&event).is_none());
        let summary = observer.summary(Duration::ZERO);
        assert_eq!(summary["qsd_messages_seen"], 1);
        assert_eq!(summary["qsd_with_rtc_seen"], 0);
        assert_eq!(summary["matching_symbol_qsd_seen"], 1);
        assert_eq!(summary["matching_symbol_without_rtc_seen"], 1);
        assert_eq!(summary["matching_qsd_messages_seen"], 0);
    }

    #[test]
    fn quote_data_observer_counts_qsd_with_rtc_before_symbol_match() {
        let mut observer = QuoteDataObserver::new("NASDAQ:RKLB");
        let event =
            qsd_event(r#"{"m":"qsd","p":["qs",{"n":"NASDAQ:AAPL","s":"ok","v":{"rtc":199.5}}]}"#);

        assert!(observer.handle_event(&event).is_none());
        let summary = observer.summary(Duration::ZERO);
        assert_eq!(summary["qsd_messages_seen"], 1);
        assert_eq!(summary["qsd_with_rtc_seen"], 1);
        assert_eq!(summary["matching_symbol_qsd_seen"], 0);
        assert_eq!(summary["matching_qsd_messages_seen"], 0);
    }

    #[test]
    fn quote_data_observer_counts_quote_session_symbol_mappings() {
        let mut observer = QuoteDataObserver::new("NASDAQ:RKLB");
        let sent = json!({
            "method": "Network.webSocketFrameSent",
            "params": {
                "response": {
                    "payloadData": r#"{"m":"quote_add_symbols","p":["qs_rklb","NASDAQ:RKLB","NASDAQ:AAPL"]}"#
                }
            }
        });

        observer.handle_event(&sent);

        assert_eq!(
            observer.summary(Duration::ZERO)["quote_session_symbol_mappings_seen"],
            2
        );
    }

    #[test]
    fn success_payload_keeps_quote_data_separate_from_extended_hours() {
        let payload = success_payload(
            "NASDAQ:RKLB",
            QuoteDataCandidate {
                symbol: "NASDAQ:RKLB".to_string(),
                rtc: json!(104.55),
                rtc_time: Value::Null,
                rch: Value::Null,
                rchp: Value::Null,
                current_session: json!("post_market"),
                market_phase: json!("post-market"),
                update_mode: json!("streaming"),
            },
            Duration::from_millis(12),
            json!({
                "bounded_wait_ms": 50,
                "elapsed_ms": 12,
                "websocket_events_seen": 1,
                "websocket_frames_seen": 1,
                "qsd_messages_seen": 1,
                "qsd_with_rtc_seen": 1,
                "matching_symbol_qsd_seen": 1,
                "matching_symbol_without_rtc_seen": 0,
                "matching_qsd_messages_seen": 1,
                "quote_session_symbol_mappings_seen": 0,
                "raw_frame_included": false,
            }),
        );

        assert_eq!(payload["contract_version"], QUOTE_DATA_CONTRACT_VERSION);
        assert_eq!(payload["source"], "desktop_quote_data_ws");
        assert_eq!(payload["quote_data"]["rtc"], json!(104.55));
        assert_eq!(payload["source_availability"]["available"], true);
        assert_eq!(payload["source_availability"]["status"], "available");
        assert_eq!(payload["source_availability"]["rtc_observed"], true);
        assert_eq!(
            payload["source_availability"]["unavailable_reason"],
            Value::Null
        );
        assert_eq!(payload["source_availability"]["timed_out"], false);
        assert_eq!(payload["source_availability"]["next_action"], Value::Null);
        assert_eq!(payload["source_availability"]["raw_frame_included"], false);
        assert_eq!(
            payload["source_availability"]["wait_summary"]["matching_qsd_messages_seen"],
            1
        );
        assert_eq!(
            payload["quote_data"]["session_readback"]["market_phase_normalized"],
            "postmarket"
        );
        assert_eq!(
            payload["quote_data"]["session_readback"]["current_session_normalized"],
            "postmarket"
        );
        assert_eq!(
            payload["quote_data"]["session_readback"]["session_source"],
            "tradingview_quote_data_fields"
        );
        assert_eq!(
            payload["quote_data"]["session_readback"]["session_inferred"],
            false
        );
        assert!(payload.get("extended_hours").is_none());
        assert_eq!(payload["chart_main_series_included"], false);
        assert_eq!(payload["scanner_extended_hours_included"], false);
    }

    #[test]
    fn unavailable_error_is_public_safe() {
        let error = unavailable_error(
            "NASDAQ:RKLB",
            json!({
                "bounded_wait_ms": 50,
                "websocket_events_seen": 1,
                "websocket_frames_seen": 1,
                "qsd_messages_seen": 1,
                "qsd_with_rtc_seen": 0,
                "matching_symbol_qsd_seen": 1,
                "matching_symbol_without_rtc_seen": 1,
                "matching_qsd_messages_seen": 0,
                "quote_session_symbol_mappings_seen": 0,
                "raw_frame_included": false,
            }),
        );
        let details = error.details.unwrap();

        assert_eq!(details["contract_version"], QUOTE_DATA_CONTRACT_VERSION);
        assert_eq!(details["source"], "desktop_quote_data_ws");
        assert_eq!(details["wait_summary"]["raw_frame_included"], false);
        assert_eq!(details["source_availability"]["available"], false);
        assert_eq!(details["source_availability"]["status"], "unavailable");
        assert_eq!(details["source_availability"]["rtc_observed"], false);
        assert_eq!(
            details["source_availability"]["unavailable_reason"],
            "no_rtc"
        );
        assert_eq!(details["source_availability"]["timed_out"], true);
        assert_eq!(
            details["source_availability"]["next_action"],
            "use_scanner_if_delayed_rest_ok"
        );
        assert_eq!(details["source_availability"]["raw_frame_included"], false);
        assert_eq!(
            details["source_availability"]["wait_summary"]["raw_frame_included"],
            false
        );
        assert!(details.get("raw").is_none());
    }

    #[test]
    fn unavailable_reason_reports_no_websocket_events() {
        let details = unavailable_details_from_summary(
            QuoteDataObserver::new("NASDAQ:RKLB").summary(Duration::ZERO),
        );

        assert_eq!(
            details["source_availability"]["unavailable_reason"],
            "no_websocket_events"
        );
        assert_eq!(
            details["source_availability"]["next_action"],
            "retry_quote_data"
        );
    }

    #[test]
    fn unavailable_reason_reports_no_websocket_frames() {
        let mut observer = QuoteDataObserver::new("NASDAQ:RKLB");
        observer.handle_event(&websocket_created_event());

        let details = unavailable_details_from_summary(observer.summary(Duration::ZERO));

        assert_eq!(
            details["source_availability"]["unavailable_reason"],
            "no_websocket_frames"
        );
        assert_eq!(
            details["source_availability"]["next_action"],
            "retry_quote_data"
        );
    }

    #[test]
    fn unavailable_reason_reports_no_qsd_messages() {
        let mut observer = QuoteDataObserver::new("NASDAQ:RKLB");
        observer.handle_event(&qsd_event(r#"{"m":"timescale_update","p":["cs",{}]}"#));

        let details = unavailable_details_from_summary(observer.summary(Duration::ZERO));

        assert_eq!(
            details["source_availability"]["unavailable_reason"],
            "no_qsd_messages"
        );
        assert_eq!(
            details["source_availability"]["next_action"],
            "retry_quote_data"
        );
    }

    #[test]
    fn unavailable_reason_reports_no_matching_symbol() {
        let mut observer = QuoteDataObserver::new("NASDAQ:RKLB");
        observer.handle_event(&qsd_event(
            r#"{"m":"qsd","p":["qs",{"n":"NASDAQ:AAPL","s":"ok","v":{"rtc":199.5}}]}"#,
        ));

        let details = unavailable_details_from_summary(observer.summary(Duration::ZERO));

        assert_eq!(
            details["source_availability"]["unavailable_reason"],
            "no_matching_symbol"
        );
        assert_eq!(
            details["source_availability"]["next_action"],
            "check_desktop_streaming_symbol"
        );
    }

    #[test]
    fn unavailable_reason_reports_no_rtc() {
        let mut observer = QuoteDataObserver::new("NASDAQ:RKLB");
        observer.handle_event(&qsd_event(
            r#"{"m":"qsd","p":["qs",{"n":"NASDAQ:RKLB","s":"ok","v":{"current_session":"regular"}}]}"#,
        ));

        let details = unavailable_details_from_summary(observer.summary(Duration::ZERO));

        assert_eq!(
            details["source_availability"]["unavailable_reason"],
            "no_rtc"
        );
        assert_eq!(
            details["source_availability"]["next_action"],
            "use_scanner_if_delayed_rest_ok"
        );
    }
}
