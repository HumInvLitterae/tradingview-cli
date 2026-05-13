use std::time::Duration;

use serde_json::{Value, json};

pub(super) const BARS_CONTRACT_VERSION: &str = "bars.v1";
pub(super) const BARS_SOURCE: &str = "tradingview_bars_ws";
pub(super) const DEFAULT_TIMEOUT_MS: u64 = 10_000;
pub(super) const MAX_BAR_COUNT: usize = 500;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct Bar {
    pub(super) time: i64,
    pub(super) open: f64,
    pub(super) high: f64,
    pub(super) low: f64,
    pub(super) close: f64,
    pub(super) volume: f64,
}

#[derive(Debug)]
pub(super) struct BarsRequest {
    pub(super) symbol: String,
    pub(super) timeframe: String,
    pub(super) count: usize,
    pub(super) timeout: Duration,
}

#[derive(Debug)]
pub(super) struct BarsResult {
    pub(super) bars: Vec<Bar>,
    pub(super) completed: bool,
    pub(super) wait_summary: BarsWaitSummary,
}

#[derive(Debug, Clone, Default)]
pub(super) struct BarsWaitSummary {
    pub(super) timeout_ms: u64,
    pub(super) websocket_messages_seen: u64,
    pub(super) websocket_packets_seen: u64,
    pub(super) update_messages_seen: u64,
    pub(super) series_completed_seen: bool,
    pub(super) error_messages_seen: u64,
}

#[derive(Debug, Clone)]
pub(super) struct BarsAvailabilityState<'a> {
    pub(super) available: bool,
    pub(super) unavailable_reason: Option<&'a str>,
    pub(super) bar_count: usize,
    pub(super) completed: bool,
    pub(super) timed_out: bool,
}

impl BarsAvailabilityState<'_> {
    pub(super) fn available(bar_count: usize, completed: bool, timed_out: bool) -> Self {
        Self {
            available: true,
            unavailable_reason: None,
            bar_count,
            completed,
            timed_out,
        }
    }

    pub(super) fn unavailable(
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
    pub(super) fn new(request: &BarsRequest) -> Self {
        Self {
            timeout_ms: request.timeout.as_millis() as u64,
            ..Self::default()
        }
    }

    pub(super) fn to_value(
        &self,
        elapsed_ms: u64,
        completed: bool,
        bars_observed_count: usize,
    ) -> Value {
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
