use std::{
    process::Command,
    time::{Duration, Instant},
};

use serde_json::Value;

const DEFAULT_SYMBOLS: &str = "NASDAQ:AAPL,NYSE:IONQ";

#[test]
#[ignore = "requires TradingView scanner availability and TV_LIVE_SNAPSHOT_SMOKE=1"]
fn snapshot_live_smoke() {
    if std::env::var("TV_LIVE_SNAPSHOT_SMOKE").ok().as_deref() != Some("1") {
        panic!("live snapshot smoke is gated; set TV_LIVE_SNAPSHOT_SMOKE=1 and run with --ignored");
    }

    let tv = env!("CARGO_BIN_EXE_tv");
    let symbols = csv_env("TV_LIVE_SNAPSHOT_SYMBOLS", DEFAULT_SYMBOLS);
    let groups = optional_csv_env("TV_LIVE_SNAPSHOT_GROUPS");
    let fields = optional_csv_env("TV_LIVE_SNAPSHOT_FIELDS");
    let runs = positive_env("TV_LIVE_SNAPSHOT_RUNS").unwrap_or(1);

    println!(
        "snapshot live smoke: symbols={} groups={} fields={} runs={}",
        symbols.join(","),
        display_csv(&groups),
        display_csv(&fields),
        runs
    );

    let mut checked = 0usize;
    let mut slowest: Option<(String, Duration)> = None;
    for run in 1..=runs {
        for symbol in &symbols {
            let started = Instant::now();
            let output = run_snapshot(tv, symbol, &groups, &fields);
            let elapsed = started.elapsed();
            let envelope = parse_output(symbol, output, elapsed);
            assert_snapshot_success(symbol, &envelope, elapsed);
            checked += 1;
            if slowest
                .as_ref()
                .is_none_or(|(_, previous)| elapsed > *previous)
            {
                slowest = Some((symbol.clone(), elapsed));
            }
            println!(
                "ok run={} symbol={} sections={} errors={} elapsed_ms={}",
                run,
                symbol,
                section_summary(&envelope),
                envelope
                    .pointer("/data/errors")
                    .and_then(Value::as_array)
                    .map(|errors| errors.len().to_string())
                    .unwrap_or_else(|| "<missing>".to_string()),
                elapsed.as_millis()
            );
        }
    }

    if let Some((symbol, elapsed)) = slowest {
        println!(
            "snapshot live smoke passed: checked={} slowest_symbol={} slowest_elapsed_ms={}",
            checked,
            symbol,
            elapsed.as_millis()
        );
    }
}

fn run_snapshot(
    tv: &str,
    symbol: &str,
    groups: &[String],
    fields: &[String],
) -> std::process::Output {
    let mut command = Command::new(tv);
    command.args(["snapshot", symbol]);
    for group in groups {
        command.args(["--group", group]);
    }
    for field in fields {
        command.args(["--field", field]);
    }
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
                "snapshot live smoke returned non-JSON output: requested_symbol={} status={} elapsed_ms={} stdout_bytes={} stderr_bytes={}",
                symbol,
                output.status,
                elapsed.as_millis(),
                output.stdout.len(),
                output.stderr.len()
            )
        });

    if !output.status.success() {
        panic!(
            "snapshot live smoke command failed: requested_symbol={} status={} elapsed_ms={} summary={}",
            symbol,
            output.status,
            elapsed.as_millis(),
            summarize_envelope(&parsed)
        );
    }

    parsed
}

fn assert_snapshot_success(symbol: &str, envelope: &Value, elapsed: Duration) {
    let data = envelope.get("data").unwrap_or(&Value::Null);
    let sections = data.get("sections").unwrap_or(&Value::Null);
    let errors = data.get("errors").and_then(Value::as_array);
    let next_action_hints = data.get("next_action_hints").and_then(Value::as_array);
    let missing_evidence = data.get("missing_evidence").and_then(Value::as_array);
    let follow_up_hints = data.get("follow_up_hints").and_then(Value::as_array);
    let summary = data.get("summary").unwrap_or(&Value::Null);

    if envelope.get("success").and_then(Value::as_bool) != Some(true)
        || envelope.get("command").and_then(Value::as_str) != Some("snapshot")
        || data.get("contract_version").and_then(Value::as_str) != Some("snapshot.v1")
        || data.get("source").and_then(Value::as_str) != Some("snapshot_desktop_free")
        || data.get("source_category").and_then(Value::as_str) != Some("desktop_free_read")
        || data.get("requires_desktop").and_then(Value::as_bool) != Some(false)
        || data.get("non_mutating").and_then(Value::as_bool) != Some(true)
        || data.get("requested_symbol").and_then(Value::as_str) != Some(symbol)
        || summary
            .get("coverage_status")
            .and_then(Value::as_str)
            .is_none()
        || summary.get("field_coverage").is_none()
        || errors.is_none()
        || missing_evidence.is_none()
        || follow_up_hints.is_none()
        || next_action_hints.is_none()
    {
        panic!(
            "snapshot live smoke metadata validation failed: requested_symbol={} elapsed_ms={} summary={}",
            symbol,
            elapsed.as_millis(),
            summarize_envelope(envelope)
        );
    }
    if !matches!(
        summary.get("coverage_status").and_then(Value::as_str),
        Some("complete" | "partial")
    ) {
        panic!(
            "snapshot live smoke invalid coverage status: requested_symbol={} elapsed_ms={} summary={}",
            symbol,
            elapsed.as_millis(),
            summarize_envelope(envelope)
        );
    }
    let follow_up_hints = follow_up_hints.unwrap();
    if !follow_up_hints
        .iter()
        .any(|hint| valid_follow_up_hint(hint, "chart_quote"))
        || !follow_up_hints
            .iter()
            .any(|hint| valid_follow_up_hint(hint, "observe_chart"))
        || !follow_up_hints
            .iter()
            .any(|hint| valid_follow_up_hint(hint, "screenshot"))
    {
        panic!(
            "snapshot live smoke missing follow-up hints: requested_symbol={} elapsed_ms={} summary={}",
            symbol,
            elapsed.as_millis(),
            summarize_envelope(envelope)
        );
    }

    let mut ok_count = 0usize;
    for section in ["quote", "info", "fundamentals"] {
        let value = sections.get(section).unwrap_or_else(|| {
            panic!(
                "snapshot live smoke missing section: requested_symbol={} section={} elapsed_ms={} summary={}",
                symbol,
                section,
                elapsed.as_millis(),
                summarize_envelope(envelope)
            )
        });
        if assert_section_shape(section, value, symbol, envelope, elapsed) {
            ok_count += 1;
        }
    }

    if ok_count == 0 {
        panic!(
            "snapshot live smoke had no successful sections: requested_symbol={} elapsed_ms={} summary={}",
            symbol,
            elapsed.as_millis(),
            summarize_envelope(envelope)
        );
    }
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
                    "snapshot live smoke invalid successful section: requested_symbol={} section={} elapsed_ms={} summary={}",
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
                    "snapshot live smoke invalid failed section: requested_symbol={} section={} elapsed_ms={} summary={}",
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
                "snapshot live smoke section missing ok flag: requested_symbol={} section={} elapsed_ms={} summary={}",
                symbol,
                section,
                elapsed.as_millis(),
                summarize_envelope(envelope)
            );
        }
    }
}

fn valid_follow_up_hint(hint: &Value, kind: &str) -> bool {
    hint.get("kind").and_then(Value::as_str) == Some(kind)
        && hint.get("command").and_then(Value::as_str).is_some()
        && hint.get("reason").and_then(Value::as_str).is_some()
        && hint.get("requires_desktop").and_then(Value::as_bool) == Some(true)
        && hint.get("source_category").and_then(Value::as_str) == Some("desktop_backed_read")
        && hint.get("non_mutating").and_then(Value::as_bool) == Some(true)
        && hint.get("evidence_role").and_then(Value::as_str).is_some()
        && hint.get("auto_execute").and_then(Value::as_bool) == Some(false)
}

fn summarize_envelope(envelope: &Value) -> String {
    let data = envelope.get("data").unwrap_or(&Value::Null);
    let error = envelope.get("error").unwrap_or(&Value::Null);
    format!(
        "success={} command={} kind={} message={} requested={} contract={} source={} category={} coverage={} sections={} errors={} missing_evidence={} hints={} next_hints={}",
        bool_field(envelope, "success"),
        string_field(envelope, "command").unwrap_or("<missing>"),
        string_field(error, "kind").unwrap_or("<none>"),
        string_field(error, "message").unwrap_or("<none>"),
        string_field(data, "requested_symbol").unwrap_or("<missing>"),
        string_field(data, "contract_version").unwrap_or("<missing>"),
        string_field(data, "source").unwrap_or("<missing>"),
        string_field(data, "source_category").unwrap_or("<missing>"),
        data.pointer("/summary/coverage_status")
            .and_then(Value::as_str)
            .unwrap_or("<missing>"),
        section_summary(envelope),
        data.get("errors")
            .and_then(Value::as_array)
            .map(|errors| errors.len().to_string())
            .unwrap_or_else(|| "<missing>".to_string()),
        data.get("missing_evidence")
            .and_then(Value::as_array)
            .map(|items| items.len().to_string())
            .unwrap_or_else(|| "<missing>".to_string()),
        data.get("follow_up_hints")
            .and_then(Value::as_array)
            .map(|hints| hints.len().to_string())
            .unwrap_or_else(|| "<missing>".to_string()),
        data.get("next_action_hints")
            .and_then(Value::as_array)
            .map(|hints| hints.len().to_string())
            .unwrap_or_else(|| "<missing>".to_string()),
    )
}

fn section_summary(envelope: &Value) -> String {
    let sections = envelope.pointer("/data/sections").unwrap_or(&Value::Null);
    ["quote", "info", "fundamentals"]
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
        .join(",")
}

fn csv_env(name: &str, default: &str) -> Vec<String> {
    let source = std::env::var(name).unwrap_or_else(|_| default.to_string());
    parse_csv(name, &source)
}

fn optional_csv_env(name: &str) -> Vec<String> {
    std::env::var(name)
        .ok()
        .map(|value| parse_csv(name, &value))
        .unwrap_or_default()
}

fn parse_csv(name: &str, source: &str) -> Vec<String> {
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

fn display_csv(values: &[String]) -> String {
    if values.is_empty() {
        "<none>".to_string()
    } else {
        values.join(",")
    }
}

fn string_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn bool_field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_bool)
        .map(|value| value.to_string())
        .unwrap_or_else(|| "<missing>".to_string())
}
