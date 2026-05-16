use std::process::Command;

use serde_json::Value;

#[test]
#[ignore = "requires a running TradingView Desktop CDP session and TV_LIVE_OBSERVE_CHART_SMOKE=1"]
fn observe_chart_jsonl_live_smoke() {
    if std::env::var("TV_LIVE_OBSERVE_CHART_SMOKE").ok().as_deref() != Some("1") {
        panic!(
            "live observe chart smoke is gated; set TV_LIVE_OBSERVE_CHART_SMOKE=1 and run with --ignored"
        );
    }

    let tv = env!("CARGO_BIN_EXE_tv");
    let target_id = optional_env("TV_LIVE_OBSERVE_CHART_TARGET_ID");
    let duration_ms = positive_env("TV_LIVE_OBSERVE_CHART_DURATION_MS").unwrap_or(3000);
    let heartbeat_ms = positive_env("TV_LIVE_OBSERVE_CHART_HEARTBEAT_MS").unwrap_or(1000);
    let max_events = positive_env("TV_LIVE_OBSERVE_CHART_MAX_EVENTS");

    println!(
        "observe chart live smoke: duration_ms={} heartbeat_ms={} max_events={} target_id={}",
        duration_ms,
        heartbeat_ms,
        max_events
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<none>".to_string()),
        if target_id.is_some() {
            "<provided>"
        } else {
            "<default>"
        }
    );

    let output = run_observe_chart(
        tv,
        target_id.as_deref(),
        duration_ms,
        heartbeat_ms,
        max_events,
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout_events = parse_jsonl(&stdout, "stdout");
    let stderr_events = parse_jsonl(&stderr, "stderr");

    if !output.status.success() {
        panic!(
            "observe chart live smoke command failed: status={} stdout_events={} stderr_summary={}",
            output.status,
            stdout_events.len(),
            summarize_error_events(&stderr_events)
        );
    }

    assert!(
        !stdout_events.is_empty(),
        "observe chart live smoke emitted no stdout JSONL events"
    );
    assert_readiness_event(&stdout_events[0], stdout_events.len(), stderr_events.len());

    let mut sample_count = 0_u64;
    let mut heartbeat_count = 0_u64;
    let mut summary_count = 0_u64;
    for event in stdout_events.iter().skip(1) {
        assert_eq!(
            event.get("command").and_then(Value::as_str),
            Some("observe"),
            "observe chart live smoke emitted a non-observe event summary={}",
            summarize_event(event)
        );
        match event.pointer("/data/_event").and_then(Value::as_str) {
            Some("sample") => {
                sample_count += 1;
                assert_sample_event(event, sample_count, heartbeat_count);
            }
            Some("heartbeat") => {
                heartbeat_count += 1;
                assert_heartbeat_event(event, sample_count, heartbeat_count);
            }
            Some("summary") => {
                summary_count += 1;
                assert_eq!(
                    summary_count,
                    1,
                    "observe chart live smoke emitted more than one summary event summary={}",
                    summarize_event(event)
                );
                assert_summary_event(event, sample_count, heartbeat_count);
            }
            other => panic!(
                "observe chart live smoke emitted unexpected event_type={:?} summary={}",
                other,
                summarize_event(event)
            ),
        }
    }
    assert_eq!(
        summary_count, 1,
        "observe chart live smoke did not emit a final summary event"
    );
    assert_eq!(
        stdout_events
            .last()
            .and_then(|event| event.pointer("/data/_event"))
            .and_then(Value::as_str),
        Some("summary"),
        "observe chart live smoke summary event was not last: last={}",
        summarize_event(stdout_events.last().unwrap())
    );

    if let Some(max_events) = max_events {
        assert!(
            sample_count <= max_events,
            "observe chart live smoke emitted too many sample events: samples={} max_events={} heartbeats={}",
            sample_count,
            max_events,
            heartbeat_count
        );
    }

    println!(
        "observe chart live smoke passed: stdout_events={} samples={} heartbeats={} summaries={} stderr_events={}",
        stdout_events.len(),
        sample_count,
        heartbeat_count,
        summary_count,
        stderr_events.len()
    );
}

fn run_observe_chart(
    tv: &str,
    target_id: Option<&str>,
    duration_ms: u64,
    heartbeat_ms: u64,
    max_events: Option<u64>,
) -> std::process::Output {
    let mut command = Command::new(tv);
    if let Some(target_id) = target_id {
        command.args(["--target-id", target_id]);
    }
    command.args([
        "observe",
        "chart",
        "--duration-ms",
        &duration_ms.to_string(),
        "--heartbeat-ms",
        &heartbeat_ms.to_string(),
    ]);
    if let Some(max_events) = max_events {
        command.args(["--max-events", &max_events.to_string()]);
    }
    command
        .output()
        .expect("test-built tv binary should execute")
}

fn assert_readiness_event(event: &Value, stdout_count: usize, stderr_count: usize) {
    let success = event.get("success").and_then(Value::as_bool);
    let command = event.get("command").and_then(Value::as_str);
    let event_type = event.pointer("/data/_event").and_then(Value::as_str);
    let contract = event
        .pointer("/data/contract_version")
        .and_then(Value::as_str);
    let observe = event.pointer("/data/_observe").and_then(Value::as_str);
    if success != Some(true)
        || command != Some("observe")
        || event_type != Some("readiness")
        || contract != Some("observe_chart.v1")
        || observe != Some("chart")
    {
        panic!(
            "observe chart live smoke first event was not readiness: stdout_events={} stderr_events={} summary={}",
            stdout_count,
            stderr_count,
            summarize_event(event)
        );
    }
}

fn assert_sample_event(event: &Value, sample_count: u64, heartbeat_count: u64) {
    let data = event.get("data").unwrap_or(&Value::Null);
    if data.get("_stream").and_then(Value::as_str) != Some("bars")
        || data.get("_observe").and_then(Value::as_str) != Some("chart")
        || data.get("contract_version").and_then(Value::as_str) != Some("observe_chart.v1")
        || data.get("source").and_then(Value::as_str) != Some("desktop_chart_stream")
        || data.get("source_category").and_then(Value::as_str) != Some("desktop_backed_read")
        || data.get("requires_desktop").and_then(Value::as_bool) != Some(true)
        || data.get("non_mutating").and_then(Value::as_bool) != Some(true)
    {
        panic!(
            "observe chart live smoke sample event failed metadata validation: samples={} heartbeats={} summary={}",
            sample_count,
            heartbeat_count,
            summarize_event(event)
        );
    }
}

fn assert_heartbeat_event(event: &Value, sample_count: u64, heartbeat_count: u64) {
    let data = event.get("data").unwrap_or(&Value::Null);
    let reported_samples = data.get("sample_count").and_then(Value::as_u64);
    if data.get("_stream").and_then(Value::as_str) != Some("bars")
        || data.get("_observe").and_then(Value::as_str) != Some("chart")
        || data.get("contract_version").and_then(Value::as_str) != Some("observe_chart.v1")
        || data.get("source").and_then(Value::as_str) != Some("desktop_chart_stream")
        || data.get("source_category").and_then(Value::as_str) != Some("desktop_backed_read")
        || data.get("requires_desktop").and_then(Value::as_bool) != Some(true)
        || data.get("non_mutating").and_then(Value::as_bool) != Some(true)
        || data.get("elapsed_ms").and_then(Value::as_u64).is_none()
        || reported_samples.is_none()
        || reported_samples.unwrap_or(u64::MAX) != sample_count
    {
        panic!(
            "observe chart live smoke heartbeat event failed metadata validation: samples={} heartbeats={} summary={}",
            sample_count,
            heartbeat_count,
            summarize_event(event)
        );
    }
}

fn assert_summary_event(event: &Value, sample_count: u64, heartbeat_count: u64) {
    let data = event.get("data").unwrap_or(&Value::Null);
    let reported_samples = data.get("sample_count").and_then(Value::as_u64);
    let reported_heartbeats = data.get("heartbeat_count").and_then(Value::as_u64);
    let end_reason = data.get("end_reason").and_then(Value::as_str);
    if data.get("_stream").and_then(Value::as_str) != Some("bars")
        || data.get("_observe").and_then(Value::as_str) != Some("chart")
        || data.get("contract_version").and_then(Value::as_str) != Some("observe_chart.v1")
        || data.get("source").and_then(Value::as_str) != Some("desktop_chart_stream")
        || data.get("source_category").and_then(Value::as_str) != Some("desktop_backed_read")
        || data.get("requires_desktop").and_then(Value::as_bool) != Some(true)
        || data.get("non_mutating").and_then(Value::as_bool) != Some(true)
        || data.get("elapsed_ms").and_then(Value::as_u64).is_none()
        || reported_samples != Some(sample_count)
        || reported_heartbeats != Some(heartbeat_count)
        || !matches!(
            end_reason,
            Some("duration_elapsed" | "max_events_reached" | "completed")
        )
    {
        panic!(
            "observe chart live smoke summary event failed metadata validation: samples={} heartbeats={} summary={}",
            sample_count,
            heartbeat_count,
            summarize_event(event)
        );
    }
}

fn parse_jsonl(source: &str, stream_name: &str) -> Vec<Value> {
    source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            Some(serde_json::from_str::<Value>(trimmed).unwrap_or_else(|_| {
                panic!(
                    "observe chart live smoke emitted non-JSONL {} line {} bytes={}",
                    stream_name,
                    index + 1,
                    trimmed.len()
                )
            }))
        })
        .collect()
}

fn summarize_event(event: &Value) -> String {
    let data = event.get("data").unwrap_or(&Value::Null);
    let error = event.get("error").unwrap_or(&Value::Null);
    format!(
        "success={} command={} event={} stream={} source={} category={} ready={} sample_count={} heartbeat_count={} elapsed_ms={} end_reason={} error_kind={} error_message={}",
        bool_summary(event.get("success")),
        string_summary(event.get("command")),
        string_summary(data.get("_event")),
        string_summary(data.get("_stream")),
        string_summary(data.get("source")),
        string_summary(data.get("source_category")),
        bool_summary(data.get("ready")),
        number_summary(data.get("sample_count")),
        number_summary(data.get("heartbeat_count")),
        number_summary(data.get("elapsed_ms")),
        string_summary(data.get("end_reason")),
        string_summary(error.get("kind")),
        string_summary(error.get("message")),
    )
}

fn summarize_error_events(events: &[Value]) -> String {
    if events.is_empty() {
        return "<none>".to_string();
    }
    events
        .iter()
        .map(summarize_event)
        .collect::<Vec<_>>()
        .join("; ")
}

fn optional_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn positive_env(name: &str) -> Option<u64> {
    optional_env(name).map(|value| {
        let parsed = value
            .parse::<u64>()
            .unwrap_or_else(|_| panic!("{name} must be a positive integer"));
        assert!(parsed > 0, "{name} must be positive");
        parsed
    })
}

fn string_summary(value: Option<&Value>) -> String {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("<missing>")
        .to_string()
}

fn bool_summary(value: Option<&Value>) -> String {
    value
        .and_then(Value::as_bool)
        .map(|value| value.to_string())
        .unwrap_or_else(|| "<missing>".to_string())
}

fn number_summary(value: Option<&Value>) -> String {
    value
        .and_then(Value::as_u64)
        .map(|value| value.to_string())
        .unwrap_or_else(|| "<missing>".to_string())
}
