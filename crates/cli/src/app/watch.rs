use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};
use tradingview_core::{AppError, ErrorBody, ErrorEnvelope, ErrorKind, SuccessEnvelope};
use tradingview_market::quote_symbols_typed;

use crate::{
    app::output::{print_jsonl_stderr, print_jsonl_stdout},
    cli::{WatchCommand, WatchCompareOptions},
};

const WATCH_COMPARE_CONTRACT_VERSION: &str = "watch_compare.v1";
const WATCH_COMPARE_SOURCE: &str = "scanner_scan_rest";
const WATCH_COMPARE_SOURCE_CATEGORY: &str = "desktop_free_read";
const WATCH_COMPARE_LABEL: &str = "compare";

const MIN_WATCH_INTERVAL_MS: u64 = 1000;
const DEFAULT_WATCH_INTERVAL_MS: u64 = 5000;
const DEFAULT_WATCH_DURATION_MS: u64 = 30000;
const MAX_WATCH_DURATION_MS: u64 = 300000;
const DEFAULT_WATCH_HEARTBEAT_MS: u64 = 10000;
const MAX_WATCH_SYMBOLS: usize = 25;

#[derive(Debug, Clone, PartialEq, Eq)]
struct WatchCompareRequest {
    symbols: Vec<String>,
    interval_ms: u64,
    duration_ms: u64,
    max_events: Option<u64>,
    heartbeat_ms: u64,
}

#[derive(Debug, Default)]
struct WatchDedupe {
    last_sample: Option<Value>,
}

impl WatchDedupe {
    fn should_emit(&mut self, sample: &Value) -> bool {
        let comparable = comparable_watch_sample(sample);
        if self.last_sample.as_ref() == Some(&comparable) {
            return false;
        }
        self.last_sample = Some(comparable);
        true
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WatchEndReason {
    Completed,
    DurationElapsed,
    MaxEventsReached,
}

impl WatchEndReason {
    fn label(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::DurationElapsed => "duration_elapsed",
            Self::MaxEventsReached => "max_events_reached",
        }
    }
}

pub async fn run_watch_command(command: WatchCommand) -> Result<(), AppError> {
    let request = watch_request_from_command(command)?;
    run_watch_compare(request).await
}

async fn run_watch_compare(request: WatchCompareRequest) -> Result<(), AppError> {
    let readiness = watch_readiness(&request)?;
    let envelope = SuccessEnvelope::new("watch", readiness);
    print_jsonl_stdout(&envelope);

    let interval = Duration::from_millis(request.interval_ms);
    let duration = Duration::from_millis(request.duration_ms);
    let heartbeat = Duration::from_millis(request.heartbeat_ms);
    let started_at = Instant::now();
    let mut last_output_at = started_at;
    let mut next_sample_at = started_at;
    let mut next_heartbeat_at = started_at + heartbeat;
    let mut sample_count = 0_u64;
    let mut heartbeat_count = 0_u64;
    let mut poll_count = 0_u64;
    let mut poll_error_count = 0_u64;
    let mut last_sample_ts = None;
    let mut last_resolved_count = 0_u64;
    let mut last_error_count = 0_u64;
    let mut dedupe = WatchDedupe::default();

    let end_reason = loop {
        let now = Instant::now();
        if now.duration_since(started_at) >= duration {
            break WatchEndReason::DurationElapsed;
        }

        if now >= next_sample_at {
            poll_count += 1;
            match watch_sample(
                &request,
                started_at.elapsed().as_millis() as u64,
                poll_count,
            )
            .await
            {
                Ok(sample) => {
                    last_resolved_count = sample["resolved_count"].as_u64().unwrap_or(0);
                    last_error_count = sample["error_count"].as_u64().unwrap_or(0);
                    if dedupe.should_emit(&sample) {
                        sample_count += 1;
                        last_sample_ts = sample["_ts"].as_u64();
                        let envelope = SuccessEnvelope::new("watch", sample);
                        print_jsonl_stdout(&envelope);
                        last_output_at = Instant::now();
                        next_heartbeat_at = last_output_at + heartbeat;
                        if request
                            .max_events
                            .is_some_and(|max_events| sample_count >= max_events)
                        {
                            break WatchEndReason::MaxEventsReached;
                        }
                    }
                }
                Err(err) => {
                    poll_error_count += 1;
                    let envelope = ErrorEnvelope::new("watch", ErrorBody::from(err));
                    print_jsonl_stderr(&envelope);
                }
            }
            next_sample_at = Instant::now() + interval;
        }

        if Instant::now() >= next_heartbeat_at && last_output_at.elapsed() >= heartbeat {
            let payload = watch_heartbeat(
                &request,
                started_at.elapsed().as_millis() as u64,
                sample_count,
                poll_count,
                poll_error_count,
                last_sample_ts,
            )?;
            let envelope = SuccessEnvelope::new("watch", payload);
            print_jsonl_stdout(&envelope);
            heartbeat_count += 1;
            last_output_at = Instant::now();
            next_heartbeat_at = last_output_at + heartbeat;
        }

        let mut sleep_until = next_sample_at.min(next_heartbeat_at);
        sleep_until = sleep_until.min(started_at + duration);
        let sleep_duration = sleep_until.saturating_duration_since(Instant::now());
        if sleep_duration.is_zero() {
            tokio::task::yield_now().await;
            continue;
        }
        tokio::time::sleep(sleep_duration).await;
    };

    let payload = watch_summary(
        &request,
        started_at.elapsed().as_millis() as u64,
        sample_count,
        heartbeat_count,
        poll_count,
        poll_error_count,
        last_sample_ts,
        last_resolved_count,
        last_error_count,
        end_reason,
    )?;
    let envelope = SuccessEnvelope::new("watch", payload);
    print_jsonl_stdout(&envelope);
    Ok(())
}

fn watch_request_from_command(command: WatchCommand) -> Result<WatchCompareRequest, AppError> {
    match command {
        WatchCommand::Compare { symbols, options } => watch_compare_request(symbols, options),
    }
}

fn watch_compare_request(
    symbols: Vec<String>,
    options: WatchCompareOptions,
) -> Result<WatchCompareRequest, AppError> {
    let symbols = normalize_watch_symbols(symbols)?;
    validate_minimum("interval", options.interval, MIN_WATCH_INTERVAL_MS)?;
    validate_minimum("heartbeat_ms", options.heartbeat_ms, MIN_WATCH_INTERVAL_MS)?;
    validate_duration(options.duration_ms)?;
    validate_positive_optional("max_events", options.max_events)?;

    Ok(WatchCompareRequest {
        symbols,
        interval_ms: options.interval,
        duration_ms: options.duration_ms,
        max_events: options.max_events,
        heartbeat_ms: options.heartbeat_ms,
    })
}

fn normalize_watch_symbols(symbols: Vec<String>) -> Result<Vec<String>, AppError> {
    if symbols.len() < 2 {
        return Err(AppError::new(
            ErrorKind::Validation,
            "watch compare requires at least two symbols",
        )
        .with_details(json!({
            "minimum": 2,
            "requested_count": symbols.len(),
        })));
    }
    if symbols.len() > MAX_WATCH_SYMBOLS {
        return Err(AppError::new(
            ErrorKind::Validation,
            format!("watch compare accepts at most {MAX_WATCH_SYMBOLS} symbols"),
        )
        .with_details(json!({
            "maximum": MAX_WATCH_SYMBOLS,
            "requested_count": symbols.len(),
        })));
    }

    let mut normalized = Vec::with_capacity(symbols.len());
    for symbol in symbols {
        let symbol = symbol.trim();
        if symbol.is_empty() {
            return Err(AppError::new(
                ErrorKind::Validation,
                "watch compare symbol must not be empty",
            ));
        }
        normalized.push(symbol.to_string());
    }
    Ok(normalized)
}

fn validate_minimum(field: &str, value: u64, minimum: u64) -> Result<(), AppError> {
    if value < minimum {
        return Err(AppError::new(
            ErrorKind::Validation,
            format!("{field} must be at least {minimum}ms"),
        )
        .with_details(json!({
            "field": field,
            "value": value,
            "minimum": minimum,
        })));
    }
    Ok(())
}

fn validate_duration(value: u64) -> Result<(), AppError> {
    if value == 0 || value > MAX_WATCH_DURATION_MS {
        return Err(AppError::new(
            ErrorKind::Validation,
            format!("duration_ms must be between 1 and {MAX_WATCH_DURATION_MS}"),
        )
        .with_details(json!({
            "field": "duration_ms",
            "value": value,
            "minimum": 1,
            "maximum": MAX_WATCH_DURATION_MS,
        })));
    }
    Ok(())
}

fn validate_positive_optional(field: &str, value: Option<u64>) -> Result<(), AppError> {
    if value == Some(0) {
        return Err(AppError::new(
            ErrorKind::Validation,
            format!("{field} must be greater than zero"),
        )
        .with_details(json!({
            "field": field,
            "value": 0,
        })));
    }
    Ok(())
}

async fn watch_sample(
    request: &WatchCompareRequest,
    elapsed_ms: u64,
    poll_index: u64,
) -> Result<Value, AppError> {
    let batch = quote_symbols_typed(request.symbols.clone()).await?;
    let mut payload = serde_json::to_value(batch)
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))?;
    let Some(object) = payload.as_object_mut() else {
        return Err(AppError::new(
            ErrorKind::Internal,
            "Watch compare sample payload was not an object",
        ));
    };
    add_watch_metadata(object, "sample")?;
    object.insert("elapsed_ms".to_string(), json!(elapsed_ms));
    object.insert("poll_index".to_string(), json!(poll_index));
    Ok(payload)
}

fn watch_readiness(request: &WatchCompareRequest) -> Result<Value, AppError> {
    let mut payload = json!({
        "validated_symbols": request.symbols,
        "symbol_count": request.symbols.len(),
        "interval_ms": request.interval_ms,
        "duration_ms": request.duration_ms,
        "max_events": request.max_events,
        "heartbeat_ms": request.heartbeat_ms,
        "default_interval_ms": DEFAULT_WATCH_INTERVAL_MS,
        "default_duration_ms": DEFAULT_WATCH_DURATION_MS,
        "default_heartbeat_ms": DEFAULT_WATCH_HEARTBEAT_MS,
    });
    let Some(object) = payload.as_object_mut() else {
        return Err(AppError::new(
            ErrorKind::Internal,
            "Watch compare readiness payload was not an object",
        ));
    };
    add_watch_metadata(object, "readiness")?;
    Ok(payload)
}

fn watch_heartbeat(
    request: &WatchCompareRequest,
    elapsed_ms: u64,
    sample_count: u64,
    poll_count: u64,
    poll_error_count: u64,
    last_sample_ts: Option<u64>,
) -> Result<Value, AppError> {
    let mut payload = json!({
        "elapsed_ms": elapsed_ms,
        "sample_count": sample_count,
        "poll_count": poll_count,
        "poll_error_count": poll_error_count,
        "last_sample_ts": last_sample_ts,
        "interval_ms": request.interval_ms,
        "duration_ms": request.duration_ms,
        "max_events": request.max_events,
        "heartbeat_ms": request.heartbeat_ms,
    });
    let Some(object) = payload.as_object_mut() else {
        return Err(AppError::new(
            ErrorKind::Internal,
            "Watch compare heartbeat payload was not an object",
        ));
    };
    add_watch_metadata(object, "heartbeat")?;
    Ok(payload)
}

#[allow(clippy::too_many_arguments)]
fn watch_summary(
    request: &WatchCompareRequest,
    elapsed_ms: u64,
    sample_count: u64,
    heartbeat_count: u64,
    poll_count: u64,
    poll_error_count: u64,
    last_sample_ts: Option<u64>,
    last_resolved_count: u64,
    last_error_count: u64,
    end_reason: WatchEndReason,
) -> Result<Value, AppError> {
    let mut payload = json!({
        "elapsed_ms": elapsed_ms,
        "sample_count": sample_count,
        "heartbeat_count": heartbeat_count,
        "poll_count": poll_count,
        "poll_error_count": poll_error_count,
        "last_sample_ts": last_sample_ts,
        "last_resolved_count": last_resolved_count,
        "last_error_count": last_error_count,
        "interval_ms": request.interval_ms,
        "duration_ms": request.duration_ms,
        "max_events": request.max_events,
        "heartbeat_ms": request.heartbeat_ms,
        "end_reason": end_reason.label(),
    });
    let Some(object) = payload.as_object_mut() else {
        return Err(AppError::new(
            ErrorKind::Internal,
            "Watch compare summary payload was not an object",
        ));
    };
    add_watch_metadata(object, "summary")?;
    Ok(payload)
}

fn add_watch_metadata(
    object: &mut serde_json::Map<String, Value>,
    event: &str,
) -> Result<(), AppError> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))?
        .as_millis() as u64;
    object.insert("_watch".to_string(), json!(WATCH_COMPARE_LABEL));
    object.insert("_event".to_string(), json!(event));
    object.insert("_ts".to_string(), json!(ts));
    object.insert(
        "contract_version".to_string(),
        json!(WATCH_COMPARE_CONTRACT_VERSION),
    );
    object.insert("source".to_string(), json!(WATCH_COMPARE_SOURCE));
    object.insert(
        "source_category".to_string(),
        json!(WATCH_COMPARE_SOURCE_CATEGORY),
    );
    object.insert("requires_desktop".to_string(), json!(false));
    object.insert("non_mutating".to_string(), json!(true));
    Ok(())
}

fn comparable_watch_sample(sample: &Value) -> Value {
    let mut comparable = sample.clone();
    if let Some(object) = comparable.as_object_mut() {
        object.remove("_ts");
        object.remove("_event");
        object.remove("elapsed_ms");
        object.remove("poll_index");
    }
    comparable
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> WatchCompareRequest {
        watch_compare_request(
            vec![" NASDAQ:AAPL ".to_string(), "NASDAQ:MSFT".to_string()],
            WatchCompareOptions {
                interval: 2000,
                duration_ms: 10000,
                max_events: Some(3),
                heartbeat_ms: 3000,
            },
        )
        .unwrap()
    }

    #[test]
    fn watch_compare_request_validates_symbols_and_controls() {
        let request = request();
        assert_eq!(
            request.symbols,
            vec!["NASDAQ:AAPL".to_string(), "NASDAQ:MSFT".to_string()]
        );
        assert_eq!(request.interval_ms, 2000);
        assert_eq!(request.duration_ms, 10000);
        assert_eq!(request.max_events, Some(3));
        assert_eq!(request.heartbeat_ms, 3000);

        assert_eq!(
            watch_compare_request(
                vec!["AAPL".to_string()],
                WatchCompareOptions {
                    interval: 5000,
                    duration_ms: 30000,
                    max_events: None,
                    heartbeat_ms: 10000,
                },
            )
            .unwrap_err()
            .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            watch_compare_request(
                vec!["AAPL".to_string(), " ".to_string()],
                WatchCompareOptions {
                    interval: 5000,
                    duration_ms: 30000,
                    max_events: None,
                    heartbeat_ms: 10000,
                },
            )
            .unwrap_err()
            .kind,
            ErrorKind::Validation
        );
    }

    #[test]
    fn watch_compare_request_rejects_large_symbol_sets_and_bad_controls() {
        let symbols = (0..26).map(|idx| format!("NASDAQ:T{idx}")).collect();
        let error = watch_compare_request(
            symbols,
            WatchCompareOptions {
                interval: 5000,
                duration_ms: 30000,
                max_events: None,
                heartbeat_ms: 10000,
            },
        )
        .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Validation);
        assert_eq!(error.details.unwrap()["maximum"], MAX_WATCH_SYMBOLS);

        for options in [
            WatchCompareOptions {
                interval: 999,
                duration_ms: 30000,
                max_events: None,
                heartbeat_ms: 10000,
            },
            WatchCompareOptions {
                interval: 5000,
                duration_ms: 0,
                max_events: None,
                heartbeat_ms: 10000,
            },
            WatchCompareOptions {
                interval: 5000,
                duration_ms: 300001,
                max_events: None,
                heartbeat_ms: 10000,
            },
            WatchCompareOptions {
                interval: 5000,
                duration_ms: 30000,
                max_events: Some(0),
                heartbeat_ms: 10000,
            },
            WatchCompareOptions {
                interval: 5000,
                duration_ms: 30000,
                max_events: None,
                heartbeat_ms: 999,
            },
        ] {
            assert_eq!(
                watch_compare_request(vec!["AAPL".to_string(), "MSFT".to_string()], options)
                    .unwrap_err()
                    .kind,
                ErrorKind::Validation
            );
        }
    }

    #[test]
    fn readiness_heartbeat_and_summary_use_watch_contract_metadata() {
        let request = request();
        let readiness = watch_readiness(&request).unwrap();
        assert_eq!(readiness["contract_version"], "watch_compare.v1");
        assert_eq!(readiness["_watch"], "compare");
        assert_eq!(readiness["_event"], "readiness");
        assert_eq!(readiness["source"], "scanner_scan_rest");
        assert_eq!(readiness["source_category"], "desktop_free_read");
        assert_eq!(readiness["requires_desktop"], false);
        assert_eq!(readiness["non_mutating"], true);
        assert_eq!(readiness["validated_symbols"][0], "NASDAQ:AAPL");

        let heartbeat = watch_heartbeat(&request, 5000, 2, 4, 1, Some(123)).unwrap();
        assert_eq!(heartbeat["contract_version"], "watch_compare.v1");
        assert_eq!(heartbeat["_event"], "heartbeat");
        assert_eq!(heartbeat["sample_count"], 2);
        assert_eq!(heartbeat["poll_count"], 4);
        assert_eq!(heartbeat["poll_error_count"], 1);

        let summary = watch_summary(
            &request,
            10000,
            3,
            1,
            5,
            0,
            Some(456),
            2,
            0,
            WatchEndReason::MaxEventsReached,
        )
        .unwrap();
        assert_eq!(summary["contract_version"], "watch_compare.v1");
        assert_eq!(summary["_event"], "summary");
        assert_eq!(summary["end_reason"], "max_events_reached");
        assert_eq!(summary["last_resolved_count"], 2);
        assert_eq!(summary["last_error_count"], 0);
    }

    #[test]
    fn watch_dedupe_ignores_event_metadata() {
        let mut dedupe = WatchDedupe::default();
        let first = json!({
            "_ts": 1,
            "_event": "sample",
            "elapsed_ms": 10,
            "poll_index": 1,
            "items": [{"requested_symbol": "AAPL", "ok": true}],
        });
        let second = json!({
            "_ts": 2,
            "_event": "sample",
            "elapsed_ms": 20,
            "poll_index": 2,
            "items": [{"requested_symbol": "AAPL", "ok": true}],
        });
        let changed = json!({
            "_ts": 3,
            "_event": "sample",
            "elapsed_ms": 30,
            "poll_index": 3,
            "items": [{"requested_symbol": "AAPL", "ok": false}],
        });

        assert!(dedupe.should_emit(&first));
        assert!(!dedupe.should_emit(&second));
        assert!(dedupe.should_emit(&changed));
    }

    #[test]
    fn completed_end_reason_label_is_stable() {
        assert_eq!(WatchEndReason::Completed.label(), "completed");
        assert_eq!(WatchEndReason::DurationElapsed.label(), "duration_elapsed");
    }
}
