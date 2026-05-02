use std::{
    process::Command,
    time::{Duration, Instant},
};

use serde_json::Value;

const DEFAULT_SYMBOLS: &str = "PLUG,AAPL,MSFT,IONQ,MU,PLUG";

#[test]
#[ignore = "requires a running TradingView Desktop CDP session and TV_LIVE_CHART_QUOTE_SMOKE=1"]
fn chart_quote_sequence_live_smoke() {
    if std::env::var("TV_LIVE_CHART_QUOTE_SMOKE").ok().as_deref() != Some("1") {
        panic!(
            "live chart quote smoke is gated; set TV_LIVE_CHART_QUOTE_SMOKE=1 and run with --ignored"
        );
    }

    let tv = env!("CARGO_BIN_EXE_tv");
    let symbols = live_symbols();
    let runs = live_runs();
    let target_id = std::env::var("TV_LIVE_CHART_QUOTE_TARGET_ID")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    println!(
        "chart quote live smoke: symbols={}, runs={}, target_id={}",
        symbols.join(","),
        runs,
        if target_id.is_some() {
            "<provided>"
        } else {
            "<default>"
        }
    );

    let mut slowest: Option<(String, Duration)> = None;
    let mut checked = 0usize;
    for run in 1..=runs {
        for symbol in &symbols {
            let started = Instant::now();
            let output = run_chart_quote(tv, target_id.as_deref(), symbol);
            let elapsed = started.elapsed();
            let envelope = parse_output(symbol, output, elapsed);
            assert_chart_quote_success(symbol, &envelope, elapsed);
            checked += 1;
            if slowest
                .as_ref()
                .is_none_or(|(_, previous)| elapsed > *previous)
            {
                slowest = Some((symbol.clone(), elapsed));
            }
            let data = envelope.get("data").unwrap_or(&Value::Null);
            println!(
                "ok run={} symbol={} observed={} chart={} stable_samples={} elapsed_ms={}",
                run,
                symbol,
                string_field(data, "observed_symbol").unwrap_or("<missing>"),
                string_field(data, "chart_symbol").unwrap_or("<missing>"),
                data.pointer("/freshness_check/stable_samples_seen")
                    .and_then(Value::as_u64)
                    .unwrap_or(0),
                elapsed.as_millis()
            );
        }
    }

    if let Some((symbol, elapsed)) = slowest {
        println!(
            "chart quote live smoke passed: checked={} slowest_symbol={} slowest_elapsed_ms={}",
            checked,
            symbol,
            elapsed.as_millis()
        );
    }
}

fn run_chart_quote(tv: &str, target_id: Option<&str>, symbol: &str) -> std::process::Output {
    let mut command = Command::new(tv);
    if let Some(target_id) = target_id {
        command.args(["--target-id", target_id]);
    }
    command.args(["quote", symbol, "--source", "chart"]);
    command
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
                "quote live smoke returned non-JSON output: requested_symbol={} status={} elapsed_ms={} stdout_bytes={} stderr_bytes={}",
                symbol,
                output.status,
                elapsed.as_millis(),
                output.stdout.len(),
                output.stderr.len()
            )
        });

    if !output.status.success() {
        panic!(
            "quote live smoke command failed: requested_symbol={} status={} elapsed_ms={} summary={}",
            symbol,
            output.status,
            elapsed.as_millis(),
            summarize_envelope(&parsed)
        );
    }

    parsed
}

fn assert_chart_quote_success(symbol: &str, envelope: &Value, elapsed: Duration) {
    let data = envelope.get("data").unwrap_or(&Value::Null);
    let requested = string_field(data, "requested_symbol").unwrap_or("<missing>");
    let observed = string_field(data, "observed_symbol").unwrap_or("<missing>");
    let chart = string_field(data, "chart_symbol").unwrap_or("<missing>");
    let restored = data
        .get("restored")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let freshness_passed = data
        .pointer("/freshness_check/passed")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let stable_samples = data
        .pointer("/freshness_check/stable_samples_seen")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let switch_performed = data
        .get("switch_performed")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let required_stable_samples = if switch_performed { 2 } else { 1 };

    if envelope.get("success").and_then(Value::as_bool) != Some(true)
        || bare_symbol(requested) != bare_symbol(symbol)
        || bare_symbol(observed) != bare_symbol(symbol)
        || bare_symbol(chart) != bare_symbol(symbol)
        || !freshness_passed
        || stable_samples < required_stable_samples
        || !restored
    {
        panic!(
            "quote live smoke validation failed: requested_symbol={} elapsed_ms={} summary={}",
            symbol,
            elapsed.as_millis(),
            summarize_envelope(envelope)
        );
    }
}

fn summarize_envelope(envelope: &Value) -> String {
    let data = envelope.get("data").unwrap_or(&Value::Null);
    let error = envelope.get("error").unwrap_or(&Value::Null);
    format!(
        "success={} kind={} message={} requested={} observed={} chart={} restored={} freshness_passed={} stable_samples={}",
        envelope
            .get("success")
            .and_then(Value::as_bool)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<missing>".to_string()),
        string_field(error, "kind").unwrap_or("<none>"),
        string_field(error, "message").unwrap_or("<none>"),
        string_field(data, "requested_symbol").unwrap_or("<missing>"),
        string_field(data, "observed_symbol").unwrap_or("<missing>"),
        string_field(data, "chart_symbol").unwrap_or("<missing>"),
        data.get("restored")
            .and_then(Value::as_bool)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<missing>".to_string()),
        data.pointer("/freshness_check/passed")
            .and_then(Value::as_bool)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<missing>".to_string()),
        data.pointer("/freshness_check/stable_samples_seen")
            .and_then(Value::as_u64)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<missing>".to_string()),
    )
}

fn live_symbols() -> Vec<String> {
    let source = std::env::var("TV_LIVE_CHART_QUOTE_SYMBOLS")
        .unwrap_or_else(|_| DEFAULT_SYMBOLS.to_string());
    let symbols: Vec<String> = source
        .split(',')
        .map(str::trim)
        .filter(|symbol| !symbol.is_empty())
        .map(str::to_string)
        .collect();
    assert!(
        !symbols.is_empty(),
        "TV_LIVE_CHART_QUOTE_SYMBOLS did not contain any non-empty symbols"
    );
    symbols
}

fn live_runs() -> usize {
    let value = std::env::var("TV_LIVE_CHART_QUOTE_RUNS").unwrap_or_else(|_| "1".to_string());
    let runs = value
        .trim()
        .parse::<usize>()
        .expect("TV_LIVE_CHART_QUOTE_RUNS must be a positive integer");
    assert!(runs > 0, "TV_LIVE_CHART_QUOTE_RUNS must be positive");
    runs
}

fn string_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn bare_symbol(symbol: &str) -> String {
    symbol
        .split(':')
        .next_back()
        .unwrap_or(symbol)
        .trim()
        .to_ascii_uppercase()
}
