use std::{
    collections::BTreeMap,
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::{Value, json};
use tokio_tungstenite::tungstenite::Message;
use tradingview_core::{AppError, ErrorKind};

use super::types::Bar;

#[derive(Debug, PartialEq)]
pub(super) enum WsPacket {
    Message(Value),
    Ping(i64),
}

pub(super) fn session_id(prefix: &str) -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    format!("{prefix}{:x}{:x}", std::process::id(), millis)
}

pub(super) fn frame(payload: &str) -> String {
    format!("~m~{}~m~{}", payload.len(), payload)
}

pub(super) fn pong_frame(value: i64) -> String {
    frame(&format!("~h~{value}"))
}

pub(super) fn parse_packets(message: Message) -> Result<Vec<WsPacket>, AppError> {
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

pub(super) fn parse_bars_from_message(message: &Value) -> Vec<Bar> {
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

pub(super) fn merge_bars(existing: &mut Vec<Bar>, incoming: Vec<Bar>) {
    let mut map = existing
        .drain(..)
        .map(|bar| (bar.time, bar))
        .collect::<BTreeMap<_, _>>();
    for bar in incoming {
        map.insert(bar.time, bar);
    }
    *existing = map.into_values().collect();
}

pub(super) fn bar_to_value(bar: Bar) -> Value {
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

    fn decode_single_frame(raw: &str) -> (usize, &str) {
        let rest = raw
            .strip_prefix("~m~")
            .expect("frame should start with the message marker");
        let (length, payload) = rest
            .split_once("~m~")
            .expect("frame should contain the payload marker");
        (
            length.parse::<usize>().expect("frame length should parse"),
            payload,
        )
    }

    #[test]
    fn pong_frames_use_canonical_payload_and_exact_byte_lengths() {
        for value in [7, 42, 1_234_567_890] {
            let pong = pong_frame(value);
            let expected_payload = format!("~h~{value}");
            let (declared_length, payload) = decode_single_frame(&pong);

            assert_eq!(pong, frame(&expected_payload));
            assert_eq!(declared_length, payload.len());
            assert_eq!(payload, expected_payload);
        }
    }

    #[test]
    fn parses_canonical_legacy_and_bare_heartbeat_packets() {
        let canonical = frame("~h~42");
        let legacy = frame("~h~43~");
        let packets = parse_text_packets(&format!("{canonical}{legacy}~h~44")).unwrap();

        assert_eq!(
            packets,
            vec![WsPacket::Ping(42), WsPacket::Ping(43), WsPacket::Ping(44)]
        );
    }

    #[test]
    fn rejects_truncated_and_invalid_framed_heartbeat_packets() {
        for raw in ["~m~5~m~~h~4".to_string(), frame("~h~"), frame("~h~abc")] {
            let error = parse_text_packets(&raw).unwrap_err();
            assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
            assert!(!error.message.is_empty());
            assert!(error.details.is_none());
        }
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
}
