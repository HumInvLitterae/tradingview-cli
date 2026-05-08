use std::{
    process::Command,
    thread,
    time::{Duration, Instant},
};

use serde_json::Value;

const DEFAULT_SYMBOL: &str = "OKLO";
const DEFAULT_QUALIFIED_SYMBOL: &str = "NYSE:OKLO";
const PROBE_STATE_NAME: &str = "__tvQuoteSessionExtendedHoursProbe";

const QUOTE_SESSION_FIELDS: &[&str] = &[
    "last_price",
    "lp",
    "close",
    "change",
    "change_percent",
    "open_price",
    "high_price",
    "low_price",
    "volume",
    "regular_close",
    "prev_close_price",
    "premarket_open",
    "premarket_high",
    "premarket_low",
    "premarket_close",
    "premarket_volume",
    "postmarket_open",
    "postmarket_high",
    "postmarket_low",
    "postmarket_close",
    "postmarket_volume",
    "market-status",
    "session-premarket",
    "session-postmarket",
    "session-regular",
    "update_mode",
    "delay_seconds",
    "lp_time",
    "rt-update-time",
];

#[test]
#[ignore = "requires a running TradingView Desktop CDP session and TV_LIVE_QUOTE_SESSION_SMOKE=1"]
fn quote_session_extended_hours_live_smoke() {
    if std::env::var("TV_LIVE_QUOTE_SESSION_SMOKE").ok().as_deref() != Some("1") {
        panic!(
            "live quote session smoke is gated; set TV_LIVE_QUOTE_SESSION_SMOKE=1 and run with --ignored"
        );
    }

    let tv = env!("CARGO_BIN_EXE_tv");
    let symbol =
        env_string("TV_LIVE_QUOTE_SESSION_SYMBOL").unwrap_or_else(|| DEFAULT_SYMBOL.into());
    let qualified_symbol = env_string("TV_LIVE_QUOTE_SESSION_QUALIFIED_SYMBOL")
        .unwrap_or_else(|| DEFAULT_QUALIFIED_SYMBOL.into());
    let expected_phase = env_string("TV_LIVE_QUOTE_SESSION_EXPECT_PHASE");
    let target_id = env_string("TV_LIVE_QUOTE_SESSION_TARGET_ID")
        .unwrap_or_else(|| resolve_single_chart_target(tv));
    let chart_symbol = env_string("TV_LIVE_QUOTE_SESSION_CHART_SYMBOL")
        .or_else(|| read_current_chart_symbol(tv, &target_id));
    let probe_symbols = probe_symbols(&qualified_symbol, chart_symbol.as_deref());

    println!(
        "quote session extended-hours smoke: symbol={} qualified={} chart_symbol={} target_id={} expected_phase={}",
        symbol,
        qualified_symbol,
        chart_symbol.as_deref().unwrap_or("<unavailable>"),
        if std::env::var("TV_LIVE_QUOTE_SESSION_TARGET_ID")
            .ok()
            .is_some()
        {
            "<provided>"
        } else {
            "<auto>"
        },
        expected_phase.as_deref().unwrap_or("<none>")
    );

    let started = Instant::now();
    let scanner = run_scanner_quote(tv, &symbol);
    let scanner_envelope = parse_command_json("scanner quote", scanner, started.elapsed());
    assert_scanner_quote(&symbol, &scanner_envelope, started.elapsed());

    start_quote_session_probe(tv, &target_id, &probe_symbols);
    thread::sleep(Duration::from_millis(5_500));
    let probe_envelope = read_quote_session_probe(tv, &target_id);
    let probe = probe_envelope.pointer("/data/result").unwrap_or_else(|| {
        panic!(
            "quote session probe result missing: {}",
            summarize_probe(&probe_envelope)
        )
    });
    assert_probe_result(&probe_symbols, expected_phase.as_deref(), probe);

    println!(
        "ok scanner={} quote_session={} elapsed_ms={}",
        scanner_summary(&scanner_envelope),
        selected_probe_summary(probe),
        started.elapsed().as_millis()
    );
}

fn run_scanner_quote(tv: &str, symbol: &str) -> std::process::Output {
    Command::new(tv)
        .args(["quote", symbol, "--source", "scanner"])
        .output()
        .expect("test-built tv binary should execute")
}

fn resolve_single_chart_target(tv: &str) -> String {
    let output = Command::new(tv)
        .arg("readiness")
        .output()
        .expect("test-built tv binary should execute");
    let envelope = parse_command_json("readiness", output, Duration::ZERO);
    let targets = envelope
        .pointer("/data/chart_targets")
        .and_then(Value::as_array)
        .unwrap_or(&vec![])
        .clone();
    match targets.as_slice() {
        [target] => target
            .get("id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| {
                panic!(
                    "readiness returned one chart target without a usable id; set TV_LIVE_QUOTE_SESSION_TARGET_ID"
                )
            }),
        [] => panic!(
            "readiness found no chart target; open TradingView Desktop or set TV_LIVE_QUOTE_SESSION_TARGET_ID"
        ),
        _ => panic!(
            "readiness found multiple chart targets; set TV_LIVE_QUOTE_SESSION_TARGET_ID for the intended chart"
        ),
    }
}

fn read_current_chart_symbol(tv: &str, target_id: &str) -> Option<String> {
    let output = Command::new(tv)
        .args(["--target-id", target_id, "quote"])
        .output()
        .expect("test-built tv binary should execute");
    let envelope = parse_command_json("current chart quote", output, Duration::ZERO);
    if !envelope
        .get("success")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return None;
    }
    envelope
        .pointer("/data/chart_symbol")
        .or_else(|| envelope.pointer("/data/observed_symbol"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn probe_symbols(qualified_symbol: &str, chart_symbol: Option<&str>) -> Vec<String> {
    let mut symbols = vec![qualified_symbol.to_string()];
    if let Some(chart_symbol) = chart_symbol {
        push_unique(&mut symbols, chart_symbol.to_string());
        if chart_symbol.contains(':') {
            push_unique(&mut symbols, symbol_ext(chart_symbol, "regular"));
            push_unique(&mut symbols, symbol_ext(chart_symbol, "extended"));
            push_unique(&mut symbols, symbol_ext(chart_symbol, "premarket"));
            push_unique(&mut symbols, symbol_ext(chart_symbol, "postmarket"));
        }
    }
    symbols
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn symbol_ext(symbol: &str, session: &str) -> String {
    format!(
        "={{\"adjustment\":\"splits\",\"currency-id\":\"USD\",\"session\":\"{}\",\"symbol\":\"{}\"}}",
        session, symbol
    )
}

fn start_quote_session_probe(tv: &str, target_id: &str, symbols: &[String]) {
    let fields_json =
        serde_json::to_string(QUOTE_SESSION_FIELDS).expect("quote session fields serialize");
    let symbols_json = serde_json::to_string(symbols).expect("probe symbols serialize");
    let script = format!(
        r#"(() => {{
  const qs = window.getQuoteSessionInstance && window.getQuoteSessionInstance();
  if (!qs) return {{ started: false, error: "missing_quote_session" }};
  const fields = {fields_json};
  const symbols = {symbols_json};
  const sub = "codex_quote_session_probe_" + Date.now();
  const originalFields = qs.options && Array.isArray(qs.options.fields) ? qs.options.fields.slice() : null;
  const state = {{ started: true, done: false, sub, requested_symbols: symbols, requested_fields: fields, updates: {{}}, errors: [] }};
  function select(update) {{
    const vals = update && update.values || {{}};
    const selected = {{}};
    for (const key of fields) {{
      if (Object.prototype.hasOwnProperty.call(vals, key)) selected[key] = vals[key];
    }}
    return {{ symbolname: update && update.symbolname || null, status: update && update.status || null, selected }};
  }}
  function cb(update) {{
    const name = update && update.symbolname || "<unknown>";
    state.updates[name] = select(update);
  }}
  function cleanup() {{
    try {{ qs.unsubscribe(sub, symbols, cb); }} catch (e) {{ state.errors.push("unsubscribe:" + String(e && e.message || e)); }}
    try {{ if (originalFields) {{ qs.options.fields = originalFields; qs.setFields(); }} }} catch (e) {{ state.errors.push("restore_fields:" + String(e && e.message || e)); }}
    state.done = true;
    state.update_count = Object.keys(state.updates).length;
  }}
  try {{
    qs.options = qs.options || {{}};
    qs.options.fields = Array.from(new Set([...(originalFields || []), ...fields]));
    qs.setFields();
    qs.subscribe(sub, symbols, cb);
  }} catch (e) {{
    state.errors.push("subscribe:" + String(e && e.message || e));
    state.done = true;
  }}
  window.{PROBE_STATE_NAME} = state;
  setTimeout(cleanup, 4500);
  return {{ started: state.started, field_count: fields.length, symbol_count: symbols.length }};
}})()"#
    );
    let output = run_ui_eval(tv, target_id, &script);
    let envelope = parse_command_json("quote session probe start", output, Duration::ZERO);
    if envelope
        .pointer("/data/result/started")
        .and_then(Value::as_bool)
        != Some(true)
    {
        panic!(
            "quote session probe did not start: {}",
            summarize_probe(&envelope)
        );
    }
}

fn read_quote_session_probe(tv: &str, target_id: &str) -> Value {
    let script = format!("(() => window.{PROBE_STATE_NAME} || null)()");
    let output = run_ui_eval(tv, target_id, &script);
    parse_command_json("quote session probe read", output, Duration::ZERO)
}

fn run_ui_eval(tv: &str, target_id: &str, script: &str) -> std::process::Output {
    Command::new(tv)
        .env("TV_ALLOW_UNSAFE_UI_EVAL", "1")
        .args(["--target-id", target_id, "ui", "eval", script])
        .output()
        .expect("test-built tv binary should execute")
}

fn parse_command_json(label: &str, output: std::process::Output, elapsed: Duration) -> Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let parsed = serde_json::from_str::<Value>(&stdout)
        .or_else(|_| serde_json::from_str::<Value>(&stderr))
        .unwrap_or_else(|_| {
            panic!(
                "{label} returned non-JSON output: status={} elapsed_ms={} stdout_bytes={} stderr_bytes={}",
                output.status,
                elapsed.as_millis(),
                output.stdout.len(),
                output.stderr.len()
            )
        });
    if !output.status.success() {
        panic!(
            "{label} failed: status={} elapsed_ms={} summary={}",
            output.status,
            elapsed.as_millis(),
            summarize_probe(&parsed)
        );
    }
    parsed
}

fn assert_scanner_quote(symbol: &str, envelope: &Value, elapsed: Duration) {
    let data = envelope.get("data").unwrap_or(&Value::Null);
    if envelope.get("success").and_then(Value::as_bool) != Some(true)
        || envelope.get("command").and_then(Value::as_str) != Some("quote")
        || data.get("source").and_then(Value::as_str) != Some("scanner_scan_rest")
        || data.get("requires_desktop").and_then(Value::as_bool) != Some(false)
        || data.pointer("/extended_hours/premarket").is_none()
        || data.pointer("/extended_hours/postmarket").is_none()
    {
        panic!(
            "scanner quote validation failed: symbol={} elapsed_ms={} summary={}",
            symbol,
            elapsed.as_millis(),
            summarize_probe(envelope)
        );
    }
}

fn assert_probe_result(symbols: &[String], expected_phase: Option<&str>, probe: &Value) {
    let errors = probe
        .get("errors")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if probe.get("started").and_then(Value::as_bool) != Some(true)
        || probe.get("done").and_then(Value::as_bool) != Some(true)
        || !errors.is_empty()
    {
        panic!("quote session probe failed: {}", summarize_probe(probe));
    }

    let updates = probe
        .get("updates")
        .and_then(Value::as_object)
        .unwrap_or_else(|| {
            panic!(
                "quote session probe missing updates: {}",
                summarize_probe(probe)
            )
        });
    if updates.is_empty() {
        panic!("quote session probe returned no updates");
    }
    if !symbols.iter().any(|symbol| updates.contains_key(symbol)) {
        panic!(
            "quote session probe did not return any requested symbol update: requested_count={} update_count={}",
            symbols.len(),
            updates.len()
        );
    }

    let mut phase_seen = false;
    let mut extended_field_seen = false;
    for update in updates.values() {
        let selected = update.get("selected").unwrap_or(&Value::Null);
        let phase = selected
            .get("market-status")
            .and_then(|status| status.get("phase"))
            .and_then(Value::as_str);
        if expected_phase.is_some_and(|expected| phase == Some(expected)) {
            phase_seen = true;
        }
        if selected.get("premarket_close").is_some() || selected.get("postmarket_close").is_some() {
            extended_field_seen = true;
        }
    }
    if let Some(expected) = expected_phase {
        assert!(
            phase_seen,
            "quote session probe did not observe expected phase {}: {}",
            expected,
            selected_probe_summary(probe)
        );
    }
    assert!(
        extended_field_seen,
        "quote session probe did not observe premarket/postmarket fields: {}",
        selected_probe_summary(probe)
    );
}

fn scanner_summary(envelope: &Value) -> String {
    let data = envelope.get("data").unwrap_or(&Value::Null);
    format!(
        "last={} update_mode={} delay_seconds={} premarket_close={} postmarket_close={}",
        display_value(data.get("last")),
        string_field(data, "update_mode").unwrap_or("<missing>"),
        display_value(data.get("delay_seconds")),
        display_value(data.pointer("/extended_hours/premarket/close")),
        display_value(data.pointer("/extended_hours/postmarket/close")),
    )
}

fn selected_probe_summary(probe: &Value) -> String {
    let updates = probe
        .get("updates")
        .and_then(Value::as_object)
        .map(|updates| {
            updates
                .iter()
                .map(|(symbol, update)| {
                    let selected = update.get("selected").unwrap_or(&Value::Null);
                    let phase = selected
                        .get("market-status")
                        .and_then(|status| status.get("phase"))
                        .and_then(Value::as_str)
                        .unwrap_or("<missing>");
                    format!(
                        "{}:phase={} last={} pre={} post={} mode={}",
                        compact_symbol(symbol),
                        phase,
                        display_value(selected.get("last_price")),
                        display_value(selected.get("premarket_close")),
                        display_value(selected.get("postmarket_close")),
                        string_field(selected, "update_mode").unwrap_or("<missing>")
                    )
                })
                .collect::<Vec<_>>()
                .join(";")
        })
        .unwrap_or_else(|| "<missing-updates>".to_string());
    format!(
        "done={} update_count={} updates={}",
        probe
            .get("done")
            .and_then(Value::as_bool)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<missing>".to_string()),
        probe
            .get("update_count")
            .and_then(Value::as_u64)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<missing>".to_string()),
        updates
    )
}

fn summarize_probe(value: &Value) -> String {
    let data = value.get("data").unwrap_or(value);
    let result = data.get("result").unwrap_or(data);
    let error = value.get("error").unwrap_or(&Value::Null);
    format!(
        "success={} kind={} message={} probe={}",
        value
            .get("success")
            .and_then(Value::as_bool)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<unknown>".to_string()),
        string_field(error, "kind").unwrap_or("<none>"),
        string_field(error, "message").unwrap_or("<none>"),
        selected_probe_summary(result)
    )
}

fn env_string(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn string_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn display_value(value: Option<&Value>) -> String {
    match value {
        Some(Value::Number(number)) => number.to_string(),
        Some(Value::String(text)) if !text.trim().is_empty() => text.trim().to_string(),
        Some(Value::Bool(value)) => value.to_string(),
        Some(Value::Null) | None => "null".to_string(),
        Some(_) => "<complex>".to_string(),
    }
}

fn compact_symbol(symbol: &str) -> String {
    if symbol.len() <= 32 {
        symbol.to_string()
    } else {
        format!("{}...", &symbol[..32])
    }
}
