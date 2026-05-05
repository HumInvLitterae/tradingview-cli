use std::{
    process::Command,
    time::{Duration, Instant},
};

use serde_json::Value;

const DEFAULT_SYMBOLS: &str = "NASDAQ:AAPL,NYSE:IONQ";
const DEFAULT_TIMEFRAME: &str = "1D";
const DEFAULT_COUNT: usize = 5;

#[test]
#[ignore = "requires TradingView WebSocket availability and TV_LIVE_BARS_SMOKE=1"]
fn experimental_bars_live_smoke() {
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

fn run_bars(tv: &str, symbol: &str, timeframe: &str, count: usize) -> std::process::Output {
    Command::new(tv)
        .env("TV_EXPERIMENTAL_BARS", "1")
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

    if envelope.get("success").and_then(Value::as_bool) != Some(true)
        || envelope.get("command").and_then(Value::as_str) != Some("bars")
        || data.get("source").and_then(Value::as_str) != Some("experimental_tradingview_ws")
        || data.get("experimental").and_then(Value::as_bool) != Some(true)
        || string_field(data, "requested_symbol") != Some(symbol)
        || string_field(data, "symbol") != Some(symbol)
        || string_field(data, "timeframe") != Some(expected_timeframe)
        || data.get("requested_count").and_then(Value::as_u64) != Some(expected_count as u64)
        || bar_count == 0
        || bar_count as usize != bars.len()
        || bar_count > expected_count as u64
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
