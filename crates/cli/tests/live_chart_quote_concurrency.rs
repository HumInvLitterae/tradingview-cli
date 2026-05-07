use std::{
    process::{Child, Command, Output},
    time::{Duration, Instant},
};

use serde_json::Value;

const DEFAULT_SYMBOLS: &str = "PLUG,AAPL,MSFT,IONQ,MU,PLUG";

#[test]
#[ignore = "requires a running TradingView Desktop CDP session and TV_LIVE_CHART_QUOTE_CONCURRENCY_SMOKE=1"]
fn chart_quote_concurrency_live_smoke() {
    if std::env::var("TV_LIVE_CHART_QUOTE_CONCURRENCY_SMOKE")
        .ok()
        .as_deref()
        != Some("1")
    {
        panic!(
            "live chart quote concurrency smoke is gated; set TV_LIVE_CHART_QUOTE_CONCURRENCY_SMOKE=1 and run with --ignored"
        );
    }

    let tv = env!("CARGO_BIN_EXE_tv");
    let symbols = live_symbols();
    let runs = live_runs();
    let width = live_width();
    let target_id = std::env::var("TV_LIVE_CHART_QUOTE_CONCURRENCY_TARGET_ID")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    println!(
        "chart quote concurrency smoke: symbols={}, runs={}, width={}, target_id={}",
        symbols.join(","),
        runs,
        width,
        if target_id.is_some() {
            "<provided>"
        } else {
            "<default>"
        }
    );

    let mut checked = 0usize;
    let total_started = Instant::now();
    for run in 1..=runs {
        for batch in symbols.chunks(width) {
            let children: Vec<RunningQuote> = batch
                .iter()
                .map(|symbol| spawn_chart_quote(tv, target_id.as_deref(), symbol, run))
                .collect();

            for child in children {
                let completed = child.wait();
                let envelope = parse_output(&completed);
                assert_chart_quote_success(&completed, &envelope);
                checked += 1;
                let data = envelope.get("data").unwrap_or(&Value::Null);
                println!(
                    "ok run={} symbol={} observed={} chart={} stable_samples={} restored={} elapsed_ms={}",
                    completed.run,
                    completed.symbol,
                    string_field(data, "observed_symbol").unwrap_or("<missing>"),
                    string_field(data, "chart_symbol").unwrap_or("<missing>"),
                    data.pointer("/freshness_check/stable_samples_seen")
                        .and_then(Value::as_u64)
                        .unwrap_or(0),
                    data.get("restored")
                        .and_then(Value::as_bool)
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "<missing>".to_string()),
                    completed.elapsed.as_millis(),
                );
            }
        }
    }

    println!(
        "chart quote concurrency smoke passed: checked={} total_elapsed_ms={}",
        checked,
        total_started.elapsed().as_millis()
    );
}

struct RunningQuote {
    run: usize,
    symbol: String,
    started: Instant,
    child: Child,
}

impl RunningQuote {
    fn wait(self) -> CompletedQuote {
        let output = self
            .child
            .wait_with_output()
            .expect("test-built tv binary should finish");
        CompletedQuote {
            run: self.run,
            symbol: self.symbol,
            elapsed: self.started.elapsed(),
            output,
        }
    }
}

struct CompletedQuote {
    run: usize,
    symbol: String,
    elapsed: Duration,
    output: Output,
}

fn spawn_chart_quote(tv: &str, target_id: Option<&str>, symbol: &str, run: usize) -> RunningQuote {
    let mut command = Command::new(tv);
    if let Some(target_id) = target_id {
        command.args(["--target-id", target_id]);
    }
    command.args(["quote", symbol, "--source", "chart"]);
    let started = Instant::now();
    let child = command.spawn().expect("test-built tv binary should start");
    RunningQuote {
        run,
        symbol: symbol.to_string(),
        started,
        child,
    }
}

fn parse_output(completed: &CompletedQuote) -> Value {
    let stdout = String::from_utf8_lossy(&completed.output.stdout);
    let stderr = String::from_utf8_lossy(&completed.output.stderr);
    let parsed = serde_json::from_str::<Value>(&stdout)
        .or_else(|_| serde_json::from_str::<Value>(&stderr))
        .unwrap_or_else(|_| {
            panic!(
                "quote concurrency smoke returned non-JSON output: run={} requested_symbol={} status={} elapsed_ms={} stdout_bytes={} stderr_bytes={}",
                completed.run,
                completed.symbol,
                completed.output.status,
                completed.elapsed.as_millis(),
                completed.output.stdout.len(),
                completed.output.stderr.len()
            )
        });

    if !completed.output.status.success() {
        panic!(
            "quote concurrency smoke command failed: run={} requested_symbol={} status={} elapsed_ms={} summary={}",
            completed.run,
            completed.symbol,
            completed.output.status,
            completed.elapsed.as_millis(),
            summarize_envelope(&parsed)
        );
    }

    parsed
}

fn assert_chart_quote_success(completed: &CompletedQuote, envelope: &Value) {
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
        || bare_symbol(requested) != bare_symbol(&completed.symbol)
        || bare_symbol(observed) != bare_symbol(&completed.symbol)
        || bare_symbol(chart) != bare_symbol(&completed.symbol)
        || !freshness_passed
        || stable_samples < required_stable_samples
        || !restored
    {
        panic!(
            "quote concurrency smoke validation failed: run={} requested_symbol={} elapsed_ms={} summary={}",
            completed.run,
            completed.symbol,
            completed.elapsed.as_millis(),
            summarize_envelope(envelope)
        );
    }
}

fn summarize_envelope(envelope: &Value) -> String {
    let data = envelope.get("data").unwrap_or(&Value::Null);
    let error = envelope.get("error").unwrap_or(&Value::Null);
    format!(
        "success={} kind={} message={} requested={} observed={} chart={} restored={} freshness_passed={} stable_samples={} switch_performed={}",
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
        data.get("switch_performed")
            .and_then(Value::as_bool)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<missing>".to_string()),
    )
}

fn live_symbols() -> Vec<String> {
    let source = std::env::var("TV_LIVE_CHART_QUOTE_CONCURRENCY_SYMBOLS")
        .unwrap_or_else(|_| DEFAULT_SYMBOLS.to_string());
    let symbols: Vec<String> = source
        .split(',')
        .map(str::trim)
        .filter(|symbol| !symbol.is_empty())
        .map(str::to_string)
        .collect();
    assert!(
        !symbols.is_empty(),
        "TV_LIVE_CHART_QUOTE_CONCURRENCY_SYMBOLS did not contain any non-empty symbols"
    );
    symbols
}

fn live_runs() -> usize {
    let value =
        std::env::var("TV_LIVE_CHART_QUOTE_CONCURRENCY_RUNS").unwrap_or_else(|_| "1".to_string());
    let runs = value
        .trim()
        .parse::<usize>()
        .expect("TV_LIVE_CHART_QUOTE_CONCURRENCY_RUNS must be a positive integer");
    assert!(
        runs > 0,
        "TV_LIVE_CHART_QUOTE_CONCURRENCY_RUNS must be positive"
    );
    runs
}

fn live_width() -> usize {
    let value =
        std::env::var("TV_LIVE_CHART_QUOTE_CONCURRENCY_WIDTH").unwrap_or_else(|_| "2".to_string());
    let width = value
        .trim()
        .parse::<usize>()
        .expect("TV_LIVE_CHART_QUOTE_CONCURRENCY_WIDTH must be a positive integer");
    assert!(
        width > 0,
        "TV_LIVE_CHART_QUOTE_CONCURRENCY_WIDTH must be positive"
    );
    width
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
