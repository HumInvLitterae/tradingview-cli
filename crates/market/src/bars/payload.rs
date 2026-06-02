use serde_json::{Map, Value, json};
use tradingview_core::{AppError, ErrorKind};

use super::{
    protocol::bar_to_value,
    types::{
        BARS_CONTRACT_VERSION, BARS_SOURCE, BarsAvailabilityState, BarsRequest, BarsRequestMode,
        BarsResult, BarsWaitSummary,
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
    let range_coverage_status = range_coverage_status(request, &result);
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
        "request_mode": request.request_mode_name(),
        "requested_symbol": request.requested_symbol,
        "resolved_symbol": request.symbol,
        "symbol": request.symbol,
        "symbol_resolution": request.symbol_resolution.to_value(),
        "timeframe": request.timeframe,
        "requested_count": request.count,
        "bar_count": bar_count,
        "requested_range": request.requested_range_value(),
        "range_alignment": request.range_alignment_value(),
        "range_fetch_summary": result.fetch_summary.to_value(),
        "returned_range": {
            "timeframe": request.timeframe,
            "first_time": first_time,
            "last_time": last_time,
            "bar_count": bar_count,
            "time_order": "ascending",
        },
        "observed_range": {
            "first_time": result.observed_first_time,
            "last_time": result.observed_last_time,
        },
        "range_coverage_status": range_coverage_status,
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
            "range_fetch_summary": result.fetch_summary.to_value(),
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
        "request_mode": request.request_mode_name(),
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
        "request_mode".to_string(),
        Value::String(request.request_mode_name().to_string()),
    );
    details.insert(
        "requested_symbol".to_string(),
        Value::String(request.requested_symbol.clone()),
    );
    details.insert(
        "resolved_symbol".to_string(),
        Value::String(request.symbol.clone()),
    );
    details.insert("symbol".to_string(), Value::String(request.symbol.clone()));
    details.insert(
        "symbol_resolution".to_string(),
        request.symbol_resolution.to_value(),
    );
    details.insert(
        "timeframe".to_string(),
        Value::String(request.timeframe.clone()),
    );
    details.insert(
        "requested_count".to_string(),
        Value::Number((request.count as u64).into()),
    );
    details.insert(
        "requested_range".to_string(),
        request.requested_range_value(),
    );
    details.insert(
        "range_alignment".to_string(),
        request.range_alignment_value(),
    );
    details.insert(
        "range_fetch_summary".to_string(),
        super::types::BarsFetchSummary::empty_for_request(request).to_value(),
    );
    details.insert(
        "requested_timeframe".to_string(),
        Value::String(request.timeframe.clone()),
    );

    if let Some(extra) = extra.as_object() {
        for (key, value) in extra {
            details.insert(key.clone(), value.clone());
        }
    }

    Value::Object(details)
}

fn range_coverage_status(request: &BarsRequest, result: &BarsResult) -> &'static str {
    let BarsRequestMode::DateRange { from, to } = &request.mode else {
        return if result.bars.len() == request.count {
            "complete"
        } else {
            "partial"
        };
    };
    let Some(observed_first_time) = result.observed_first_time else {
        return "partial";
    };
    let Some(observed_last_time) = result.observed_last_time else {
        return "partial";
    };
    if result.bars.len() == request.count
        && result
            .bars
            .last()
            .is_some_and(|returned_last| returned_last.time < to.timestamp)
    {
        return "partial";
    }
    if observed_first_time <= from.timestamp && observed_last_time >= to.timestamp {
        "complete"
    } else {
        "partial"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bars::{
        types::{
            Bar, BarsFetchSummary, BarsFetchSummaryInput, BarsSymbolResolution, BarsWaitSummary,
            DEFAULT_TIMEOUT_MS,
        },
        validation::{
            validate_bars_range_request, validate_bars_range_request_with_resolution,
            validate_bars_request,
        },
    };

    fn test_wait_summary(request: &BarsRequest) -> BarsWaitSummary {
        let mut summary = BarsWaitSummary::new(request);
        summary.websocket_messages_seen = 3;
        summary.websocket_packets_seen = 4;
        summary.update_messages_seen = 2;
        summary.series_completed_seen = true;
        summary
    }

    fn test_result(
        request: &BarsRequest,
        bars: Vec<Bar>,
        completed: bool,
        wait_summary: BarsWaitSummary,
        observed_first_time: Option<i64>,
        observed_last_time: Option<i64>,
    ) -> BarsResult {
        let count = bars.len();
        BarsResult {
            bars,
            completed,
            wait_summary,
            fetch_summary: BarsFetchSummary::new(
                request,
                BarsFetchSummaryInput {
                    request_more_count: 0,
                    observed_count: count,
                    filtered_count: count,
                    returned_count: count,
                    completed,
                    observed_first_time,
                    observed_last_time,
                },
            ),
            observed_first_time,
            observed_last_time,
        }
    }

    #[test]
    fn bars_payload_contains_stable_source_contract() {
        let request = validate_bars_request("NASDAQ:AAPL", "1d", 5).unwrap();
        let payload = bars_payload(
            &request,
            test_result(
                &request,
                vec![
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
                true,
                test_wait_summary(&request),
                Some(1),
                Some(2),
            ),
            42,
        );

        assert_eq!(payload["contract_version"], BARS_CONTRACT_VERSION);
        assert_eq!(payload["source"], BARS_SOURCE);
        assert_eq!(payload["source_category"], "desktop_free_read");
        assert_eq!(payload["requires_desktop"], false);
        assert_eq!(payload["non_mutating"], true);
        assert_eq!(payload["request_mode"], "recent_count");
        assert_eq!(payload["requested_symbol"], "NASDAQ:AAPL");
        assert_eq!(payload["resolved_symbol"], "NASDAQ:AAPL");
        assert_eq!(payload["symbol"], "NASDAQ:AAPL");
        assert_eq!(
            payload["symbol_resolution"]["resolution_source"],
            "input_exchange_qualified"
        );
        assert_eq!(
            payload["symbol_resolution"]["resolution_status"],
            "input_exchange_qualified"
        );
        assert!(payload["requested_range"].is_null());
        assert!(payload["range_alignment"].is_null());
        assert_eq!(payload["returned_range"]["first_time"], 1);
        assert_eq!(payload["returned_range"]["last_time"], 2);
        assert_eq!(payload["range_coverage_status"], "partial");
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
        assert_eq!(payload["range_fetch_summary"]["fetch_window_count"], 1);
        assert_eq!(payload["range_fetch_summary"]["request_more_count"], 0);
        assert_eq!(payload["range_fetch_summary"]["initial_fetch_count"], 5);
        assert_eq!(payload["range_fetch_summary"]["requested_count_cap"], 5);
        assert_eq!(payload["range_fetch_summary"]["observed_count"], 2);
        assert_eq!(payload["range_fetch_summary"]["filtered_count"], 2);
        assert_eq!(payload["range_fetch_summary"]["returned_count"], 2);
        assert_eq!(payload["range_fetch_summary"]["range_truncated"], false);
        assert_eq!(
            payload["range_fetch_summary"]["range_truncation_reason"],
            "none"
        );
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
    fn bars_payload_preserves_requested_and_resolved_symbol() {
        let request = validate_bars_range_request_with_resolution(
            "AAPL",
            "NASDAQ:AAPL",
            BarsSymbolResolution::symbol_search("AAPL", "NASDAQ:AAPL", 3),
            "1D",
            "2020-01-01",
            "2020-01-31",
            500,
        )
        .unwrap();
        let payload = bars_payload(
            &request,
            test_result(
                &request,
                vec![Bar {
                    time: 1_577_836_800,
                    open: 10.0,
                    high: 12.0,
                    low: 9.0,
                    close: 11.0,
                    volume: 100.0,
                }],
                true,
                test_wait_summary(&request),
                Some(1_577_836_800),
                Some(1_580_428_800),
            ),
            42,
        );

        assert_eq!(payload["requested_symbol"], "AAPL");
        assert_eq!(payload["resolved_symbol"], "NASDAQ:AAPL");
        assert_eq!(payload["symbol"], "NASDAQ:AAPL");
        assert_eq!(payload["symbol_resolution"]["input_symbol"], "AAPL");
        assert_eq!(
            payload["symbol_resolution"]["resolved_symbol"],
            "NASDAQ:AAPL"
        );
        assert_eq!(
            payload["symbol_resolution"]["resolution_source"],
            "symbol_search_rest"
        );
        assert_eq!(
            payload["symbol_resolution"]["resolution_status"],
            "resolved"
        );
        assert_eq!(payload["symbol_resolution"]["candidate_count"], 3);
        assert_eq!(payload["range_alignment"]["timeframe"], "1D");
        assert_eq!(payload["range_fetch_summary"]["requested_count_cap"], 500);
    }

    #[test]
    fn bars_payload_marks_full_count_coverage_complete() {
        let request = validate_bars_request("NASDAQ:AAPL", "1d", 1).unwrap();
        let payload = bars_payload(
            &request,
            test_result(
                &request,
                vec![Bar {
                    time: 1,
                    open: 10.0,
                    high: 12.0,
                    low: 9.0,
                    close: 11.0,
                    volume: 100.0,
                }],
                true,
                test_wait_summary(&request),
                Some(1),
                Some(1),
            ),
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
            test_result(
                &request,
                vec![Bar {
                    time: 1,
                    open: 10.0,
                    high: 12.0,
                    low: 9.0,
                    close: 11.0,
                    volume: 100.0,
                }],
                false,
                wait_summary,
                Some(1),
                Some(1),
            ),
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
    fn bars_payload_reports_date_range_readback() {
        let request =
            validate_bars_range_request("NASDAQ:AAPL", "1D", "2020-01-01", "2020-01-31", 500)
                .unwrap();
        let payload = bars_payload(
            &request,
            test_result(
                &request,
                vec![
                    Bar {
                        time: 1_577_836_800,
                        open: 10.0,
                        high: 12.0,
                        low: 9.0,
                        close: 11.0,
                        volume: 100.0,
                    },
                    Bar {
                        time: 1_580_428_800,
                        open: 11.0,
                        high: 13.0,
                        low: 10.0,
                        close: 12.0,
                        volume: 200.0,
                    },
                ],
                true,
                test_wait_summary(&request),
                Some(1_577_836_800),
                Some(1_580_428_800),
            ),
            42,
        );

        assert_eq!(payload["request_mode"], "date_range");
        assert_eq!(payload["requested_count"], 500);
        assert_eq!(payload["requested_range"]["from"], "2020-01-01");
        assert_eq!(payload["requested_range"]["to"], "2020-01-31");
        assert_eq!(payload["requested_range"]["from_time"], 1_577_836_800);
        assert_eq!(payload["requested_range"]["to_time"], 1_580_428_800);
        assert_eq!(
            payload["requested_range"]["to_time_exclusive"],
            1_580_515_200
        );
        assert_eq!(payload["returned_range"]["first_time"], 1_577_836_800);
        assert_eq!(payload["returned_range"]["last_time"], 1_580_428_800);
        assert_eq!(payload["returned_range"]["bar_count"], 2);
        assert_eq!(payload["observed_range"]["first_time"], 1_577_836_800);
        assert_eq!(payload["observed_range"]["last_time"], 1_580_428_800);
        assert_eq!(payload["range_coverage_status"], "complete");
        assert_eq!(payload["range_alignment"]["timeframe"], "1D");
        assert_eq!(payload["range_fetch_summary"]["initial_fetch_count"], 500);
        assert_eq!(payload["range_fetch_summary"]["requested_count_cap"], 500);
        assert_eq!(payload["range_fetch_summary"]["observed_count"], 2);
        assert_eq!(payload["range_fetch_summary"]["filtered_count"], 2);
        assert_eq!(payload["range_fetch_summary"]["returned_count"], 2);
        assert_eq!(payload["range_fetch_summary"]["range_truncated"], false);
        assert_eq!(
            payload["range_fetch_summary"]["range_truncation_reason"],
            "none"
        );
        assert_eq!(
            payload["range_alignment"]["bar_timestamp_semantics"],
            "period_start"
        );
        assert_eq!(
            payload["range_alignment"]["range_filter_policy"],
            "timestamp_within_requested_range"
        );
        assert_eq!(
            payload["range_alignment"]["requested_range_interpretation"],
            "inclusive_calendar_dates"
        );
        assert_eq!(payload["source_availability"]["request_mode"], "date_range");
    }

    #[test]
    fn bars_payload_reports_weekly_monthly_range_alignment() {
        for timeframe in ["1W", "1M"] {
            let request = validate_bars_range_request(
                "NASDAQ:AAPL",
                timeframe,
                "2020-01-01",
                "2020-03-31",
                500,
            )
            .unwrap();
            let payload = bars_payload(
                &request,
                test_result(
                    &request,
                    vec![Bar {
                        time: 1_577_836_800,
                        open: 10.0,
                        high: 12.0,
                        low: 9.0,
                        close: 11.0,
                        volume: 100.0,
                    }],
                    true,
                    test_wait_summary(&request),
                    Some(1_577_836_800),
                    Some(1_585_699_200),
                ),
                42,
            );

            assert_eq!(payload["request_mode"], "date_range");
            assert_eq!(payload["range_alignment"]["timeframe"], timeframe);
            assert_eq!(
                payload["range_alignment"]["bar_timestamp_semantics"],
                "period_start"
            );
            assert_eq!(
                payload["range_alignment"]["range_filter_policy"],
                "timestamp_within_requested_range"
            );
            assert_eq!(
                payload["range_alignment"]["requested_range_interpretation"],
                "inclusive_calendar_dates"
            );
            assert_eq!(payload["range_coverage_status"], "complete");
        }
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
        assert_eq!(details["request_mode"], "recent_count");
        assert_eq!(details["requested_symbol"], "NASDAQ:AAPL");
        assert_eq!(details["resolved_symbol"], "NASDAQ:AAPL");
        assert_eq!(details["symbol"], "NASDAQ:AAPL");
        assert_eq!(
            details["symbol_resolution"]["resolution_source"],
            "input_exchange_qualified"
        );
        assert!(details["requested_range"].is_null());
        assert!(details["range_alignment"].is_null());
        assert_eq!(details["requested_timeframe"], "1D");
        assert_eq!(details["range_fetch_summary"]["fetch_window_count"], 1);
        assert_eq!(details["range_fetch_summary"]["request_more_count"], 0);
        assert_eq!(details["range_fetch_summary"]["initial_fetch_count"], 5);
        assert_eq!(details["range_fetch_summary"]["requested_count_cap"], 5);
        assert_eq!(details["range_fetch_summary"]["observed_count"], 0);
        assert_eq!(details["range_fetch_summary"]["filtered_count"], 0);
        assert_eq!(details["range_fetch_summary"]["returned_count"], 0);
        assert_eq!(details["range_fetch_summary"]["range_truncated"], false);
        assert_eq!(
            details["range_fetch_summary"]["range_truncation_reason"],
            "none"
        );
        assert_eq!(details["availability_status"], "unavailable");
        assert_eq!(details["completed"], false);
        assert!(details.get("raw_frame").is_none());
        assert!(details.get("raw_payload").is_none());
    }

    #[test]
    fn bars_error_details_contains_range_alignment_for_date_range() {
        let request =
            validate_bars_range_request("NASDAQ:AAPL", "1W", "2020-01-01", "2020-03-31", 500)
                .unwrap();
        let details = bars_error_details(
            &request,
            json!({
                "availability_status": "unavailable",
                "completed": false,
            }),
        );

        assert_eq!(details["contract_version"], BARS_CONTRACT_VERSION);
        assert_eq!(details["requested_symbol"], "NASDAQ:AAPL");
        assert_eq!(details["resolved_symbol"], "NASDAQ:AAPL");
        assert_eq!(details["symbol"], "NASDAQ:AAPL");
        assert_eq!(details["requested_timeframe"], "1W");
        assert_eq!(details["request_mode"], "date_range");
        assert_eq!(details["requested_range"]["from"], "2020-01-01");
        assert_eq!(details["range_alignment"]["timeframe"], "1W");
        assert_eq!(
            details["range_alignment"]["bar_timestamp_semantics"],
            "period_start"
        );
        assert_eq!(
            details["range_alignment"]["range_filter_policy"],
            "timestamp_within_requested_range"
        );
        assert_eq!(
            details["range_alignment"]["requested_range_interpretation"],
            "inclusive_calendar_dates"
        );
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
