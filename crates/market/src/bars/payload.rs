use serde_json::{Map, Value, json};
use tradingview_core::{AppError, ErrorKind};

use super::{
    protocol::bar_to_value,
    types::{
        BARS_CONTRACT_VERSION, BARS_SOURCE, BarsAvailabilityState, BarsRequest, BarsResult,
        BarsWaitSummary,
    },
};

pub(super) fn bars_payload(request: &BarsRequest, result: BarsResult, elapsed_ms: u64) -> Value {
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

pub(super) fn no_bars_error(
    request: &BarsRequest,
    result: &BarsResult,
    elapsed_ms: u64,
) -> AppError {
    let source_availability = bars_source_availability(
        request,
        BarsAvailabilityState::unavailable(
            "timeout_no_bars",
            0,
            result.completed,
            !result.completed,
        ),
        &result.wait_summary,
        elapsed_ms,
    );

    AppError::new(
        ErrorKind::InternalApiUnavailable,
        "Bars request completed without returning bars",
    )
    .with_details(bars_error_details(
        request,
        json!({
            "availability_status": "unavailable",
            "completed": result.completed,
            "elapsed_ms": elapsed_ms,
            "source_availability": source_availability,
            "next_action_hint": "The browserless historical bars source did not return bars inside the bounded request. Retry later or use `tv ohlcv` against a selected chart target when chart-backed bars are acceptable.",
        }),
    ))
}

pub(super) fn bars_source_availability(
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

pub(super) fn bars_error_details(request: &BarsRequest, extra: Value) -> Value {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bars::{
        types::{Bar, BarsWaitSummary, DEFAULT_TIMEOUT_MS},
        validation::validate_bars_request,
    };

    fn test_wait_summary(request: &BarsRequest) -> BarsWaitSummary {
        let mut summary = BarsWaitSummary::new(request);
        summary.websocket_messages_seen = 3;
        summary.websocket_packets_seen = 4;
        summary.update_messages_seen = 2;
        summary.series_completed_seen = true;
        summary
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
