use std::{
    process::Command,
    time::{Duration, Instant},
};

use serde_json::Value;

const DEFAULT_SYMBOLS: &str = "NASDAQ:AAPL,NYSE:IONQ";

#[test]
#[ignore = "requires TradingView scanner availability and TV_LIVE_COMPARE_SMOKE=1"]
fn compare_live_smoke() {
    if std::env::var("TV_LIVE_COMPARE_SMOKE").ok().as_deref() != Some("1") {
        panic!("live compare smoke is gated; set TV_LIVE_COMPARE_SMOKE=1 and run with --ignored");
    }

    let tv = env!("CARGO_BIN_EXE_tv");
    let symbols = csv_env("TV_LIVE_COMPARE_SYMBOLS", DEFAULT_SYMBOLS);
    assert!(
        symbols.len() >= 2,
        "TV_LIVE_COMPARE_SYMBOLS must contain at least two non-empty values"
    );
    let runs = positive_env("TV_LIVE_COMPARE_RUNS").unwrap_or(1);

    println!(
        "compare live smoke: symbols={} runs={}",
        symbols.join(","),
        runs
    );

    let mut slowest: Option<Duration> = None;
    for run in 1..=runs {
        let started = Instant::now();
        let output = run_compare(tv, &symbols);
        let elapsed = started.elapsed();
        let envelope = parse_output(&symbols, output, elapsed);
        assert_compare_success(&symbols, &envelope, elapsed);
        if slowest.is_none_or(|previous| elapsed > previous) {
            slowest = Some(elapsed);
        }
        println!(
            "ok run={} items={} resolved={} errors={} sections={} elapsed_ms={}",
            run,
            symbols.len(),
            count_field(
                envelope.pointer("/data").unwrap_or(&Value::Null),
                "resolved_count"
            ),
            count_field(
                envelope.pointer("/data").unwrap_or(&Value::Null),
                "error_count"
            ),
            item_summary(&envelope),
            elapsed.as_millis()
        );
    }

    if let Some(elapsed) = slowest {
        println!(
            "compare live smoke passed: runs={} symbols={} slowest_elapsed_ms={}",
            runs,
            symbols.join(","),
            elapsed.as_millis()
        );
    }
}

fn run_compare(tv: &str, symbols: &[String]) -> std::process::Output {
    let mut command = Command::new(tv);
    command.arg("compare").args(symbols);
    command
        .output()
        .expect("test-built tv binary should execute")
}

fn parse_output(symbols: &[String], output: std::process::Output, elapsed: Duration) -> Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let parsed = serde_json::from_str::<Value>(&stdout)
        .or_else(|_| serde_json::from_str::<Value>(&stderr))
        .unwrap_or_else(|_| {
            panic!(
                "compare live smoke returned non-JSON output: symbols={} status={} elapsed_ms={} stdout_bytes={} stderr_bytes={}",
                symbols.join(","),
                output.status,
                elapsed.as_millis(),
                output.stdout.len(),
                output.stderr.len()
            )
        });

    if !output.status.success() {
        panic!(
            "compare live smoke command failed: symbols={} status={} elapsed_ms={} summary={}",
            symbols.join(","),
            output.status,
            elapsed.as_millis(),
            summarize_envelope(&parsed)
        );
    }

    parsed
}

fn assert_compare_success(symbols: &[String], envelope: &Value, elapsed: Duration) {
    let data = envelope.get("data").unwrap_or(&Value::Null);
    let items = data
        .get("items")
        .and_then(Value::as_array)
        .unwrap_or_else(|| {
            panic!(
                "compare live smoke missing items array: symbols={} elapsed_ms={} summary={}",
                symbols.join(","),
                elapsed.as_millis(),
                summarize_envelope(envelope)
            )
        });

    if envelope.get("success").and_then(Value::as_bool) != Some(true)
        || envelope.get("command").and_then(Value::as_str) != Some("compare")
        || data.get("source").and_then(Value::as_str) != Some("compare_desktop_free")
        || data.get("source_category").and_then(Value::as_str) != Some("desktop_free_read")
        || data.get("requires_desktop").and_then(Value::as_bool) != Some(false)
        || data.get("non_mutating").and_then(Value::as_bool) != Some(true)
        || data.get("requested_count").and_then(Value::as_u64) != Some(symbols.len() as u64)
        || items.len() != symbols.len()
        || data.get("errors").and_then(Value::as_array).is_none()
        || data
            .get("next_action_hints")
            .and_then(Value::as_array)
            .is_none()
    {
        panic!(
            "compare live smoke metadata validation failed: symbols={} elapsed_ms={} summary={}",
            symbols.join(","),
            elapsed.as_millis(),
            summarize_envelope(envelope)
        );
    }

    let mut ok_count = 0usize;
    for (index, expected_symbol) in symbols.iter().enumerate() {
        let item = &items[index];
        if item.get("requested_symbol").and_then(Value::as_str) != Some(expected_symbol.as_str()) {
            panic!(
                "compare live smoke item order validation failed: expected_symbol={} index={} elapsed_ms={} summary={}",
                expected_symbol,
                index,
                elapsed.as_millis(),
                summarize_envelope(envelope)
            );
        }
        if assert_item_shape(expected_symbol, item, envelope, elapsed) {
            ok_count += 1;
        }
    }

    if ok_count == 0 {
        panic!(
            "compare live smoke had no successful items: symbols={} elapsed_ms={} summary={}",
            symbols.join(","),
            elapsed.as_millis(),
            summarize_envelope(envelope)
        );
    }
}

fn assert_item_shape(symbol: &str, item: &Value, envelope: &Value, elapsed: Duration) -> bool {
    let sections = item.get("sections").unwrap_or(&Value::Null);
    if item.get("ok").and_then(Value::as_bool).is_none()
        || item.get("errors").and_then(Value::as_array).is_none()
        || item.get("missing_summary").is_none()
    {
        panic!(
            "compare live smoke invalid item shape: requested_symbol={} elapsed_ms={} summary={}",
            symbol,
            elapsed.as_millis(),
            summarize_envelope(envelope)
        );
    }

    let mut ok_sections = 0usize;
    for section in ["quote", "info", "fundamentals"] {
        let value = sections.get(section).unwrap_or_else(|| {
            panic!(
                "compare live smoke missing section: requested_symbol={} section={} elapsed_ms={} summary={}",
                symbol,
                section,
                elapsed.as_millis(),
                summarize_envelope(envelope)
            )
        });
        if assert_section_shape(section, value, symbol, envelope, elapsed) {
            ok_sections += 1;
        }
    }

    let item_ok = item.get("ok").and_then(Value::as_bool) == Some(true);
    if item_ok != (ok_sections > 0) {
        panic!(
            "compare live smoke item ok mismatch: requested_symbol={} ok_sections={} elapsed_ms={} summary={}",
            symbol,
            ok_sections,
            elapsed.as_millis(),
            summarize_envelope(envelope)
        );
    }

    item_ok
}

fn assert_section_shape(
    section: &str,
    value: &Value,
    symbol: &str,
    envelope: &Value,
    elapsed: Duration,
) -> bool {
    match value.get("ok").and_then(Value::as_bool) {
        Some(true) => {
            if value.get("data").is_none() || value.get("error").is_some() {
                panic!(
                    "compare live smoke invalid successful section: requested_symbol={} section={} elapsed_ms={} summary={}",
                    symbol,
                    section,
                    elapsed.as_millis(),
                    summarize_envelope(envelope)
                );
            }
            true
        }
        Some(false) => {
            let error = value.get("error").unwrap_or(&Value::Null);
            if value.get("data").is_some()
                || error.get("section").and_then(Value::as_str) != Some(section)
                || error.get("kind").and_then(Value::as_str).is_none()
                || error.get("message").and_then(Value::as_str).is_none()
            {
                panic!(
                    "compare live smoke invalid failed section: requested_symbol={} section={} elapsed_ms={} summary={}",
                    symbol,
                    section,
                    elapsed.as_millis(),
                    summarize_envelope(envelope)
                );
            }
            false
        }
        None => {
            panic!(
                "compare live smoke section missing ok flag: requested_symbol={} section={} elapsed_ms={} summary={}",
                symbol,
                section,
                elapsed.as_millis(),
                summarize_envelope(envelope)
            );
        }
    }
}

fn summarize_envelope(envelope: &Value) -> String {
    let data = envelope.get("data").unwrap_or(&Value::Null);
    let error = envelope.get("error").unwrap_or(&Value::Null);
    format!(
        "success={} command={} kind={} message={} source={} category={} requested_count={} resolved_count={} error_count={} items={} errors={} hints={}",
        bool_field(envelope, "success"),
        string_field(envelope, "command").unwrap_or("<missing>"),
        string_field(error, "kind").unwrap_or("<none>"),
        string_field(error, "message").unwrap_or("<none>"),
        string_field(data, "source").unwrap_or("<missing>"),
        string_field(data, "source_category").unwrap_or("<missing>"),
        count_field(data, "requested_count"),
        count_field(data, "resolved_count"),
        count_field(data, "error_count"),
        item_summary(envelope),
        data.get("errors")
            .and_then(Value::as_array)
            .map(|errors| errors.len().to_string())
            .unwrap_or_else(|| "<missing>".to_string()),
        data.get("next_action_hints")
            .and_then(Value::as_array)
            .map(|hints| hints.len().to_string())
            .unwrap_or_else(|| "<missing>".to_string()),
    )
}

fn item_summary(envelope: &Value) -> String {
    envelope
        .pointer("/data/items")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .map(|item| {
                    let symbol = string_field(item, "requested_symbol").unwrap_or("<missing>");
                    let sections = item.get("sections").unwrap_or(&Value::Null);
                    let states = ["quote", "info", "fundamentals"]
                        .iter()
                        .map(|section| {
                            let value = sections.get(*section).unwrap_or(&Value::Null);
                            match value.get("ok").and_then(Value::as_bool) {
                                Some(true) => format!("{section}:ok"),
                                Some(false) => {
                                    let kind = value
                                        .pointer("/error/kind")
                                        .and_then(Value::as_str)
                                        .unwrap_or("<missing>");
                                    format!("{section}:error({kind})")
                                }
                                None => format!("{section}:missing"),
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(",");
                    format!("{symbol}[{states}]")
                })
                .collect::<Vec<_>>()
                .join(";")
        })
        .unwrap_or_else(|| "<missing>".to_string())
}

fn csv_env(name: &str, default: &str) -> Vec<String> {
    let source = std::env::var(name).unwrap_or_else(|_| default.to_string());
    let values: Vec<String> = source
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect();
    assert!(
        !values.is_empty(),
        "{name} did not contain any non-empty values"
    );
    values
}

fn positive_env(name: &str) -> Option<usize> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(|value| {
            let parsed = value
                .parse::<usize>()
                .unwrap_or_else(|_| panic!("{name} must be a positive integer"));
            assert!(parsed > 0, "{name} must be positive");
            parsed
        })
}

fn string_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn count_field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_u64)
        .map(|value| value.to_string())
        .unwrap_or_else(|| "<missing>".to_string())
}

fn bool_field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_bool)
        .map(|value| value.to_string())
        .unwrap_or_else(|| "<missing>".to_string())
}
