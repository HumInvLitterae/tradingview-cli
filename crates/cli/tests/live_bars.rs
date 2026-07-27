use std::{
    io::Read,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use serde::Serialize;
use serde_json::Value;
use tempfile::NamedTempFile;

const DEFAULT_SYMBOLS: &str = "NASDAQ:AAPL,NYSE:IONQ";
const DEFAULT_TIMEFRAME: &str = "1D";
const DEFAULT_COUNT: usize = 5;
const RANGE_CHILD_TIMEOUT: Duration = Duration::from_secs(15);

#[test]
#[ignore = "requires TradingView WebSocket availability and TV_LIVE_BARS_SMOKE=1"]
fn bars_live_smoke() {
    if std::env::var("TV_LIVE_BARS_SMOKE").ok().as_deref() != Some("1") {
        panic!("live bars smoke is gated; set TV_LIVE_BARS_SMOKE=1 and run with --ignored");
    }

    let tv = env!("CARGO_BIN_EXE_tv");
    let symbols = live_symbols();
    let timeframe = live_timeframe();
    let expected_timeframe = normalized_timeframe(&timeframe);
    let count = live_count();
    let runs = live_runs();

    println!(
        "bars live smoke: symbols={} timeframe={} count={} runs={}",
        symbols.join(","),
        timeframe,
        count,
        runs
    );

    let mut checked = 0usize;
    let mut slowest: Option<(String, Duration)> = None;
    for run in 1..=runs {
        for symbol in &symbols {
            let started = Instant::now();
            let output = run_bars(tv, symbol, &timeframe, count);
            let elapsed = started.elapsed();
            let envelope = parse_output(symbol, output, elapsed);
            assert_bars_success(symbol, &expected_timeframe, count, &envelope, elapsed);
            checked += 1;
            if slowest
                .as_ref()
                .is_none_or(|(_, previous)| elapsed > *previous)
            {
                slowest = Some((symbol.clone(), elapsed));
            }
            let data = envelope.get("data").unwrap_or(&Value::Null);
            println!(
                "ok run={} symbol={} timeframe={} bars={} completed={} elapsed_ms={}",
                run,
                symbol,
                string_field(data, "timeframe").unwrap_or("<missing>"),
                data.get("bar_count").and_then(Value::as_u64).unwrap_or(0),
                data.pointer("/data_quality/completed")
                    .and_then(Value::as_bool)
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "<missing>".to_string()),
                elapsed.as_millis()
            );
        }
    }

    if let Some((symbol, elapsed)) = slowest {
        println!(
            "bars live smoke passed: checked={} slowest_symbol={} slowest_elapsed_ms={}",
            checked,
            symbol,
            elapsed.as_millis()
        );
    }
}

#[derive(Clone, Copy)]
struct RangeCase {
    label: &'static str,
    from_env: &'static str,
    to_env: &'static str,
    count: usize,
    minimum_request_more: u64,
}

const RANGE_CASES: [RangeCase; 3] = [
    RangeCase {
        label: "single_window",
        from_env: "TV_LIVE_BARS_RANGE_SINGLE_FROM",
        to_env: "TV_LIVE_BARS_RANGE_SINGLE_TO",
        count: 500,
        minimum_request_more: 0,
    },
    RangeCase {
        label: "additional_window",
        from_env: "TV_LIVE_BARS_RANGE_PAGED_FROM",
        to_env: "TV_LIVE_BARS_RANGE_PAGED_TO",
        count: 5000,
        minimum_request_more: 1,
    },
    RangeCase {
        label: "closure_boundary",
        from_env: "TV_LIVE_BARS_RANGE_CLOSURE_FROM",
        to_env: "TV_LIVE_BARS_RANGE_CLOSURE_TO",
        count: 500,
        minimum_request_more: 0,
    },
];

#[derive(Serialize)]
struct RangeCaseSummary {
    case: &'static str,
    requested: u64,
    completed: bool,
    bar_count: u64,
    coverage_status: String,
    fetch_window_count: u64,
    request_more_count: u64,
    range_truncated: bool,
    truncation_reason: String,
    elapsed_ms: u128,
}

#[derive(Serialize)]
struct RangeSmokeSummary {
    requested: usize,
    completed: usize,
    cases: Vec<RangeCaseSummary>,
}

#[test]
#[ignore = "requires TradingView WebSocket availability, explicit range inputs, owner approval, and TV_LIVE_BARS_RANGE_SMOKE=1"]
fn one_minute_date_range_live_smoke() {
    if std::env::var("TV_LIVE_BARS_RANGE_SMOKE").ok().as_deref() != Some("1") {
        panic!(
            "one-minute range smoke is gated; set TV_LIVE_BARS_RANGE_SMOKE=1 and run with --ignored"
        );
    }

    let symbol = required_exchange_qualified_symbol("TV_LIVE_BARS_RANGE_SYMBOL");
    let configured = RANGE_CASES
        .iter()
        .map(|case| {
            (
                *case,
                required_env(case.from_env),
                required_env(case.to_env),
            )
        })
        .collect::<Vec<_>>();
    let tv = env!("CARGO_BIN_EXE_tv");
    let mut cases = Vec::with_capacity(configured.len());

    for (case, from, to) in configured {
        let started = Instant::now();
        let output = run_bars_range(tv, &symbol, &from, &to, case.count);
        let elapsed = started.elapsed();
        let envelope = parse_range_output(output, elapsed);
        cases.push(assert_range_success(case, &envelope, elapsed));
    }

    let summary = RangeSmokeSummary {
        requested: RANGE_CASES.len(),
        completed: cases.len(),
        cases,
    };
    assert_eq!(summary.requested, summary.completed);
    let serialized = serde_json::to_string(&summary).expect("range summary should serialize");
    assert!(!serialized.contains(&symbol));
    for case in RANGE_CASES {
        assert!(!serialized.contains(&required_env(case.from_env)));
        assert!(!serialized.contains(&required_env(case.to_env)));
    }
    println!("{serialized}");
}

fn run_bars_range(
    tv: &str,
    symbol: &str,
    from: &str,
    to: &str,
    count: usize,
) -> std::process::Output {
    let mut stdout_file = NamedTempFile::new().expect("range smoke stdout file should open");
    let mut stderr_file = NamedTempFile::new().expect("range smoke stderr file should open");
    let mut child = Command::new(tv)
        .args([
            "bars",
            symbol,
            "--timeframe",
            "1",
            "--from",
            from,
            "--to",
            to,
            "--count",
            &count.to_string(),
        ])
        .stdout(Stdio::from(
            stdout_file
                .reopen()
                .expect("range smoke stdout should reopen"),
        ))
        .stderr(Stdio::from(
            stderr_file
                .reopen()
                .expect("range smoke stderr should reopen"),
        ))
        .spawn()
        .expect("test-built tv binary should execute");
    let deadline = Instant::now() + RANGE_CHILD_TIMEOUT;
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .expect("range smoke child should be readable")
        {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("one-minute range smoke child exceeded its fixed deadline");
        }
        thread::sleep(Duration::from_millis(25));
    };

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    stdout_file
        .read_to_end(&mut stdout)
        .expect("range smoke stdout should be readable");
    stderr_file
        .read_to_end(&mut stderr)
        .expect("range smoke stderr should be readable");
    std::process::Output {
        status,
        stdout,
        stderr,
    }
}

fn parse_range_output(output: std::process::Output, elapsed: Duration) -> Value {
    let parsed = serde_json::from_slice::<Value>(&output.stdout)
        .or_else(|_| serde_json::from_slice::<Value>(&output.stderr))
        .unwrap_or_else(|_| {
            panic!(
                "one-minute range smoke returned non-JSON output: status={} elapsed_ms={} stdout_bytes={} stderr_bytes={}",
                output.status,
                elapsed.as_millis(),
                output.stdout.len(),
                output.stderr.len()
            )
        });
    assert!(
        output.status.success(),
        "one-minute range smoke command failed: status={} elapsed_ms={} kind={}",
        output.status,
        elapsed.as_millis(),
        parsed
            .pointer("/error/kind")
            .and_then(Value::as_str)
            .unwrap_or("invalid")
    );
    parsed
}

fn assert_range_success(case: RangeCase, envelope: &Value, elapsed: Duration) -> RangeCaseSummary {
    let data = envelope.get("data").unwrap_or(&Value::Null);
    let bar_count = required_u64(data, "/bar_count");
    let fetch_window_count = required_u64(data, "/range_fetch_summary/fetch_window_count");
    let request_more_count = required_u64(data, "/range_fetch_summary/request_more_count");
    let filtered_count = required_u64(data, "/range_fetch_summary/filtered_count");
    let returned_count = required_u64(data, "/range_fetch_summary/returned_count");
    let range_truncated = required_bool(data, "/range_fetch_summary/range_truncated");
    let truncation_reason = required_string(data, "/range_fetch_summary/range_truncation_reason");
    let coverage_status = required_string(data, "/range_coverage_status");
    let completed = required_bool(data, "/data_quality/completed");
    let partial_result = required_bool(data, "/data_quality/partial_result");
    let timed_out = required_bool(data, "/source_availability/timed_out");
    let wait_completed = required_bool(data, "/source_availability/wait_summary/completed");
    let series_completed_seen = required_bool(
        data,
        "/source_availability/wait_summary/series_completed_seen",
    );

    assert_eq!(envelope.get("success").and_then(Value::as_bool), Some(true));
    assert_eq!(
        envelope.get("command").and_then(Value::as_str),
        Some("bars")
    );
    assert_eq!(
        data.get("contract_version").and_then(Value::as_str),
        Some("bars.v1")
    );
    assert_eq!(
        data.get("request_mode").and_then(Value::as_str),
        Some("date_range")
    );
    assert_eq!(data.get("timeframe").and_then(Value::as_str), Some("1"));
    assert_eq!(
        data.pointer("/range_alignment/bar_timestamp_semantics")
            .and_then(Value::as_str),
        Some("period_start")
    );
    assert_eq!(fetch_window_count, request_more_count + 1);
    assert!(request_more_count >= case.minimum_request_more);
    if case.label == "single_window" {
        assert_eq!(request_more_count, 0);
    }
    assert_eq!(bar_count, returned_count);
    assert!(returned_count <= filtered_count);
    assert!(matches!(coverage_status.as_str(), "complete" | "partial"));
    assert!(matches!(
        truncation_reason.as_str(),
        "none" | "count_cap" | "timeout" | "source_exhausted"
    ));
    assert!(
        range_classification_is_consistent(
            case,
            completed,
            &coverage_status,
            range_truncated,
            &truncation_reason,
            timed_out,
            wait_completed,
            series_completed_seen,
            partial_result,
            bar_count,
        ),
        "one-minute range smoke returned contradictory aggregate classification"
    );

    RangeCaseSummary {
        case: case.label,
        requested: 1,
        completed,
        bar_count,
        coverage_status,
        fetch_window_count,
        request_more_count,
        range_truncated,
        truncation_reason,
        elapsed_ms: elapsed.as_millis(),
    }
}

fn required_env(key: &str) -> String {
    let value = std::env::var(key)
        .unwrap_or_else(|_| panic!("{key} must be set"))
        .trim()
        .to_string();
    assert!(!value.is_empty(), "{key} must not be empty");
    value
}

fn required_exchange_qualified_symbol(key: &str) -> String {
    let value = required_env(key);
    assert!(
        is_exchange_qualified_symbol(&value),
        "{key} must contain exactly one colon with non-empty exchange and symbol"
    );
    value
}

fn is_exchange_qualified_symbol(value: &str) -> bool {
    let Some((exchange, symbol)) = value.split_once(':') else {
        return false;
    };
    !exchange.is_empty() && !symbol.is_empty() && !symbol.contains(':')
}

#[allow(clippy::too_many_arguments)]
fn range_classification_is_consistent(
    case: RangeCase,
    completed: bool,
    coverage_status: &str,
    range_truncated: bool,
    truncation_reason: &str,
    timed_out: bool,
    wait_completed: bool,
    series_completed_seen: bool,
    partial_result: bool,
    bar_count: u64,
) -> bool {
    let truncation_consistent = match truncation_reason {
        "none" => !range_truncated && coverage_status == "complete",
        "count_cap" | "timeout" | "source_exhausted" => {
            range_truncated && coverage_status == "partial"
        }
        _ => false,
    };
    let completion_consistent = timed_out == !completed
        && wait_completed == completed
        && (!completed || series_completed_seen)
        && (completed || truncation_reason == "timeout");
    let result_count_consistent = partial_result == (bar_count != case.count as u64);

    truncation_consistent && completion_consistent && result_count_consistent
}

fn required_u64(value: &Value, pointer: &str) -> u64 {
    value
        .pointer(pointer)
        .and_then(Value::as_u64)
        .unwrap_or_else(|| panic!("range smoke missing aggregate integer at {pointer}"))
}

fn required_bool(value: &Value, pointer: &str) -> bool {
    value
        .pointer(pointer)
        .and_then(Value::as_bool)
        .unwrap_or_else(|| panic!("range smoke missing aggregate boolean at {pointer}"))
}

fn required_string(value: &Value, pointer: &str) -> String {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| panic!("range smoke missing aggregate string at {pointer}"))
        .to_string()
}

#[test]
fn one_minute_range_smoke_matrix_is_exactly_bounded() {
    assert_eq!(RANGE_CASES.len(), 3);
    assert_eq!(
        RANGE_CASES
            .iter()
            .map(|case| case.label)
            .collect::<Vec<_>>(),
        ["single_window", "additional_window", "closure_boundary"]
    );
    assert_eq!(RANGE_CASES[0].count, 500);
    assert_eq!(RANGE_CASES[1].count, 5000);
    assert_eq!(RANGE_CASES[1].minimum_request_more, 1);
    assert_eq!(RANGE_CHILD_TIMEOUT, Duration::from_secs(15));
}

#[test]
fn one_minute_range_symbol_gate_requires_exact_exchange_qualification() {
    for invalid in ["", "AAPL", ":", "NASDAQ:", ":AAPL", "NASDAQ:AAPL:EXTRA"] {
        assert!(!is_exchange_qualified_symbol(invalid), "{invalid}");
    }
    assert!(is_exchange_qualified_symbol("NASDAQ:AAPL"));
}

#[test]
fn one_minute_range_classification_rejects_contradictions() {
    let closure = RANGE_CASES[2];
    assert!(range_classification_is_consistent(
        closure, true, "complete", false, "none", false, true, true, true, 10,
    ));
    assert!(range_classification_is_consistent(
        closure, false, "partial", true, "timeout", true, false, false, true, 10,
    ));
    assert!(!range_classification_is_consistent(
        closure, false, "complete", false, "none", false, false, false, true, 10,
    ));
    assert!(!range_classification_is_consistent(
        closure,
        true,
        "complete",
        true,
        "source_exhausted",
        false,
        true,
        true,
        true,
        10,
    ));
}

#[test]
fn one_minute_range_summary_is_aggregate_only() {
    let summary = RangeSmokeSummary {
        requested: 3,
        completed: 3,
        cases: vec![RangeCaseSummary {
            case: "single_window",
            requested: 1,
            completed: true,
            bar_count: 10,
            coverage_status: "complete".to_string(),
            fetch_window_count: 1,
            request_more_count: 0,
            range_truncated: false,
            truncation_reason: "none".to_string(),
            elapsed_ms: 42,
        }],
    };
    let serialized = serde_json::to_string(&summary).unwrap();
    for private in [
        "NASDAQ:PRIVATE",
        "2020-01-01",
        "\"bars\"",
        "\"prices\"",
        "raw_payload",
        "ws://",
    ] {
        assert!(!serialized.contains(private));
    }
}

fn run_bars(tv: &str, symbol: &str, timeframe: &str, count: usize) -> std::process::Output {
    Command::new(tv)
        .args([
            "bars",
            symbol,
            "--timeframe",
            timeframe,
            "--count",
            &count.to_string(),
        ])
        .output()
        .expect("test-built tv binary should execute")
}

fn parse_output(symbol: &str, output: std::process::Output, elapsed: Duration) -> Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let parsed = serde_json::from_str::<Value>(&stdout)
        .or_else(|_| serde_json::from_str::<Value>(&stderr))
        .unwrap_or_else(|_| {
            panic!(
                "bars live smoke returned non-JSON output: requested_symbol={} status={} elapsed_ms={} stdout_bytes={} stderr_bytes={}",
                symbol,
                output.status,
                elapsed.as_millis(),
                output.stdout.len(),
                output.stderr.len()
            )
        });

    if !output.status.success() {
        panic!(
            "bars live smoke command failed: requested_symbol={} status={} elapsed_ms={} summary={}",
            symbol,
            output.status,
            elapsed.as_millis(),
            summarize_envelope(&parsed)
        );
    }

    parsed
}

fn assert_bars_success(
    symbol: &str,
    expected_timeframe: &str,
    expected_count: usize,
    envelope: &Value,
    elapsed: Duration,
) {
    let data = envelope.get("data").unwrap_or(&Value::Null);
    let bars = data
        .get("bars")
        .and_then(Value::as_array)
        .unwrap_or_else(|| {
            panic!(
                "bars live smoke missing bars array: requested_symbol={} elapsed_ms={} summary={}",
                symbol,
                elapsed.as_millis(),
                summarize_envelope(envelope)
            )
        });
    let bar_count = data.get("bar_count").and_then(Value::as_u64).unwrap_or(0);
    let first_time = bars
        .first()
        .and_then(|bar| bar.get("time"))
        .and_then(Value::as_i64);
    let last_time = bars
        .last()
        .and_then(|bar| bar.get("time"))
        .and_then(Value::as_i64);
    let requested_count_fulfilled = bar_count == expected_count as u64;
    let expected_coverage_status = if requested_count_fulfilled {
        "complete"
    } else {
        "partial"
    };
    let completed = data
        .pointer("/data_quality/completed")
        .and_then(Value::as_bool);
    let expected_timed_out = completed.map(|value| !value);

    if envelope.get("success").and_then(Value::as_bool) != Some(true)
        || envelope.get("command").and_then(Value::as_str) != Some("bars")
        || data.get("contract_version").and_then(Value::as_str) != Some("bars.v1")
        || data.get("source").and_then(Value::as_str) != Some("tradingview_bars_ws")
        || data.get("source_category").and_then(Value::as_str) != Some("desktop_free_read")
        || data.get("requires_desktop").and_then(Value::as_bool) != Some(false)
        || data.get("non_mutating").and_then(Value::as_bool) != Some(true)
        || string_field(data, "requested_symbol") != Some(symbol)
        || string_field(data, "symbol") != Some(symbol)
        || string_field(data, "timeframe") != Some(expected_timeframe)
        || data.get("requested_count").and_then(Value::as_u64) != Some(expected_count as u64)
        || bar_count == 0
        || bar_count as usize != bars.len()
        || bar_count > expected_count as u64
        || data
            .pointer("/summary/requested_count")
            .and_then(Value::as_u64)
            != Some(expected_count as u64)
        || data.pointer("/summary/bar_count").and_then(Value::as_u64) != Some(bar_count)
        || data.pointer("/summary/first_time").and_then(Value::as_i64) != first_time
        || data.pointer("/summary/last_time").and_then(Value::as_i64) != last_time
        || data.pointer("/summary/time_order").and_then(Value::as_str) != Some("ascending")
        || data
            .pointer("/summary/requested_count_fulfilled")
            .and_then(Value::as_bool)
            != Some(requested_count_fulfilled)
        || data
            .pointer("/summary/coverage_status")
            .and_then(Value::as_str)
            != Some(expected_coverage_status)
        || data.pointer("/range/timeframe").and_then(Value::as_str) != Some(expected_timeframe)
        || data.pointer("/range/first_time").and_then(Value::as_i64) != first_time
        || data.pointer("/range/last_time").and_then(Value::as_i64) != last_time
        || data.pointer("/range/bar_count").and_then(Value::as_u64) != Some(bar_count)
        || data
            .pointer("/range_fetch_summary/fetch_window_count")
            .and_then(Value::as_u64)
            != Some(1)
        || data
            .pointer("/range_fetch_summary/request_more_count")
            .and_then(Value::as_u64)
            != Some(0)
        || data
            .pointer("/range_fetch_summary/initial_fetch_count")
            .and_then(Value::as_u64)
            != Some(expected_count as u64)
        || data
            .pointer("/range_fetch_summary/requested_count_cap")
            .and_then(Value::as_u64)
            != Some(expected_count as u64)
        || data
            .pointer("/range_fetch_summary/observed_count")
            .and_then(Value::as_u64)
            != Some(bar_count)
        || data
            .pointer("/range_fetch_summary/filtered_count")
            .and_then(Value::as_u64)
            != Some(bar_count)
        || data
            .pointer("/range_fetch_summary/returned_count")
            .and_then(Value::as_u64)
            != Some(bar_count)
        || data
            .pointer("/range_fetch_summary/range_truncated")
            .and_then(Value::as_bool)
            != Some(false)
        || data
            .pointer("/range_fetch_summary/range_truncation_reason")
            .and_then(Value::as_str)
            != Some("none")
        || data
            .pointer("/data_quality/realtime_guarantee")
            .and_then(Value::as_bool)
            != Some(false)
        || data
            .pointer("/data_quality/entitlement_checked")
            .and_then(Value::as_bool)
            != Some(false)
        || data
            .pointer("/data_quality/completed")
            .and_then(Value::as_bool)
            .is_none()
        || data
            .pointer("/data_quality/elapsed_ms")
            .and_then(Value::as_u64)
            .is_none()
        || data
            .pointer("/data_quality/partial_result")
            .and_then(Value::as_bool)
            != Some(!requested_count_fulfilled)
        || data
            .pointer("/source_availability/available")
            .and_then(Value::as_bool)
            != Some(true)
        || data
            .pointer("/source_availability/status")
            .and_then(Value::as_str)
            != Some("available")
        || !data
            .pointer("/source_availability/unavailable_reason")
            .is_some_and(Value::is_null)
        || data
            .pointer("/source_availability/requested_count")
            .and_then(Value::as_u64)
            != Some(expected_count as u64)
        || data
            .pointer("/source_availability/bar_count")
            .and_then(Value::as_u64)
            != Some(bar_count)
        || data
            .pointer("/source_availability/requested_count_fulfilled")
            .and_then(Value::as_bool)
            != Some(requested_count_fulfilled)
        || data
            .pointer("/source_availability/timed_out")
            .and_then(Value::as_bool)
            != expected_timed_out
        || data
            .pointer("/source_availability/raw_frame_included")
            .and_then(Value::as_bool)
            != Some(false)
        || data
            .pointer("/source_availability/wait_summary/timeout_ms")
            .and_then(Value::as_u64)
            .is_none()
        || data
            .pointer("/source_availability/wait_summary/elapsed_ms")
            .and_then(Value::as_u64)
            .is_none()
        || data
            .pointer("/source_availability/wait_summary/completed")
            .and_then(Value::as_bool)
            != completed
        || data
            .pointer("/source_availability/wait_summary/websocket_messages_seen")
            .and_then(Value::as_u64)
            .is_none()
        || data
            .pointer("/source_availability/wait_summary/websocket_packets_seen")
            .and_then(Value::as_u64)
            .is_none()
        || data
            .pointer("/source_availability/wait_summary/update_messages_seen")
            .and_then(Value::as_u64)
            .is_none()
        || data
            .pointer("/source_availability/wait_summary/series_completed_seen")
            .and_then(Value::as_bool)
            .is_none()
        || data
            .pointer("/source_availability/wait_summary/error_messages_seen")
            .and_then(Value::as_u64)
            .is_none()
        || data
            .pointer("/source_availability/wait_summary/bars_observed_count")
            .and_then(Value::as_u64)
            != Some(bar_count)
        || data
            .pointer("/source_availability/wait_summary/raw_frame_included")
            .and_then(Value::as_bool)
            != Some(false)
    {
        panic!(
            "bars live smoke validation failed: requested_symbol={} elapsed_ms={} summary={}",
            symbol,
            elapsed.as_millis(),
            summarize_envelope(envelope)
        );
    }

    for (index, bar) in bars.iter().enumerate() {
        assert_bar(symbol, index, bar, envelope, elapsed);
    }
}

fn assert_bar(symbol: &str, index: usize, bar: &Value, envelope: &Value, elapsed: Duration) {
    if bar.get("time").and_then(Value::as_i64).is_none()
        || number_field(bar, "open").is_none()
        || number_field(bar, "high").is_none()
        || number_field(bar, "low").is_none()
        || number_field(bar, "close").is_none()
        || number_field(bar, "volume").is_none()
    {
        panic!(
            "bars live smoke bar validation failed: requested_symbol={} bar_index={} elapsed_ms={} summary={}",
            symbol,
            index,
            elapsed.as_millis(),
            summarize_envelope(envelope)
        );
    }
}

fn summarize_envelope(envelope: &Value) -> String {
    let data = envelope.get("data").unwrap_or(&Value::Null);
    let error = envelope.get("error").unwrap_or(&Value::Null);
    format!(
        "success={} kind={} message={} requested={} symbol={} timeframe={} requested_count={} bar_count={} completed={} elapsed_ms={}",
        envelope
            .get("success")
            .and_then(Value::as_bool)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<missing>".to_string()),
        string_field(error, "kind").unwrap_or("<none>"),
        string_field(error, "message").unwrap_or("<none>"),
        string_field(data, "requested_symbol").unwrap_or("<missing>"),
        string_field(data, "symbol").unwrap_or("<missing>"),
        string_field(data, "timeframe").unwrap_or("<missing>"),
        data.get("requested_count")
            .and_then(Value::as_u64)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<missing>".to_string()),
        data.get("bar_count")
            .and_then(Value::as_u64)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<missing>".to_string()),
        data.pointer("/data_quality/completed")
            .and_then(Value::as_bool)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<missing>".to_string()),
        data.pointer("/data_quality/elapsed_ms")
            .and_then(Value::as_u64)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<missing>".to_string()),
    )
}

fn live_symbols() -> Vec<String> {
    let source =
        std::env::var("TV_LIVE_BARS_SYMBOLS").unwrap_or_else(|_| DEFAULT_SYMBOLS.to_string());
    let symbols: Vec<String> = source
        .split(',')
        .map(str::trim)
        .filter(|symbol| !symbol.is_empty())
        .map(str::to_string)
        .collect();
    assert!(
        !symbols.is_empty(),
        "TV_LIVE_BARS_SYMBOLS did not contain any non-empty symbols"
    );
    for symbol in &symbols {
        assert!(
            symbol.contains(':'),
            "TV_LIVE_BARS_SYMBOLS must contain exchange-qualified symbols"
        );
    }
    symbols
}

fn live_timeframe() -> String {
    std::env::var("TV_LIVE_BARS_TIMEFRAME")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_TIMEFRAME.to_string())
}

fn live_count() -> usize {
    let value = std::env::var("TV_LIVE_BARS_COUNT").unwrap_or_else(|_| DEFAULT_COUNT.to_string());
    let count = value
        .trim()
        .parse::<usize>()
        .expect("TV_LIVE_BARS_COUNT must be a positive integer");
    assert!(count > 0, "TV_LIVE_BARS_COUNT must be positive");
    assert!(count <= 500, "TV_LIVE_BARS_COUNT must be at most 500");
    count
}

fn live_runs() -> usize {
    let value = std::env::var("TV_LIVE_BARS_RUNS").unwrap_or_else(|_| "1".to_string());
    let runs = value
        .trim()
        .parse::<usize>()
        .expect("TV_LIVE_BARS_RUNS must be a positive integer");
    assert!(runs > 0, "TV_LIVE_BARS_RUNS must be positive");
    runs
}

fn normalized_timeframe(timeframe: &str) -> String {
    match timeframe.trim() {
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
    }
    .to_string()
}

fn string_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn number_field<'a>(value: &'a Value, key: &str) -> Option<&'a serde_json::Number> {
    value.get(key).and_then(Value::as_number)
}
