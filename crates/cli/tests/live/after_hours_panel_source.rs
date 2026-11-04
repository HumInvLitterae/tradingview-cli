use std::{
    process::Command,
    thread,
    time::{Duration, Instant},
};

use serde_json::Value;

const DEFAULT_SYMBOL: &str = "RKLB";
const DEFAULT_QUALIFIED_SYMBOL: &str = "NASDAQ:RKLB";
const PROBE_STATE_NAME: &str = "__tvAfterHoursPanelSourceProbe";

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
#[ignore = "requires a running TradingView Desktop CDP session and TV_LIVE_AFTER_HOURS_PANEL_SMOKE=1"]
fn after_hours_panel_source_live_smoke() {
    if std::env::var("TV_LIVE_AFTER_HOURS_PANEL_SMOKE")
        .ok()
        .as_deref()
        != Some("1")
    {
        panic!(
            "live after-hours panel source smoke is gated; set TV_LIVE_AFTER_HOURS_PANEL_SMOKE=1 and run with --ignored"
        );
    }

    let tv = env!("CARGO_BIN_EXE_tv");
    let symbol = env_string("TV_LIVE_AFTER_HOURS_SYMBOL").unwrap_or_else(|| DEFAULT_SYMBOL.into());
    let qualified_symbol = env_string("TV_LIVE_AFTER_HOURS_QUALIFIED_SYMBOL")
        .unwrap_or_else(|| DEFAULT_QUALIFIED_SYMBOL.into());
    let expected_phase = env_string("TV_LIVE_AFTER_HOURS_EXPECT_PHASE");
    let expected_visible_price = env_string("TV_LIVE_AFTER_HOURS_EXPECT_VISIBLE_PRICE");
    let target_id = env_string("TV_LIVE_AFTER_HOURS_TARGET_ID")
        .unwrap_or_else(|| resolve_single_chart_target(tv));
    let chart_symbol = read_current_chart_symbol(tv, &target_id);
    let probe_symbols = probe_symbols(&qualified_symbol, chart_symbol.as_deref());

    println!(
        "after-hours panel source smoke: symbol={} qualified={} chart_symbol={} target_id={} expected_phase={} expected_visible_price={}",
        symbol,
        qualified_symbol,
        chart_symbol.as_deref().unwrap_or("<unavailable>"),
        if std::env::var("TV_LIVE_AFTER_HOURS_TARGET_ID")
            .ok()
            .is_some()
        {
            "<provided>"
        } else {
            "<auto>"
        },
        expected_phase.as_deref().unwrap_or("<none>"),
        expected_visible_price.as_deref().unwrap_or("<none>")
    );

    let started = Instant::now();
    let scanner = run_scanner_quote(tv, &symbol);
    let scanner_envelope = parse_command_json("scanner quote", scanner, started.elapsed());
    assert_scanner_quote(&symbol, &scanner_envelope, started.elapsed());

    let chart = run_chart_quote(tv, &target_id);
    let chart_envelope = parse_command_json("chart quote", chart, started.elapsed());
    assert_chart_quote(&chart_envelope, started.elapsed());

    start_quote_session_probe(tv, &target_id, &probe_symbols);
    thread::sleep(Duration::from_millis(5_500));
    let probe_envelope = read_quote_session_probe(tv, &target_id);
    let probe = probe_envelope.pointer("/data/result").unwrap_or_else(|| {
        panic!(
            "quote session probe result missing: {}",
            summarize_probe(&probe_envelope)
        )
    });
    let phase_result = assert_probe_result(&probe_symbols, expected_phase.as_deref(), probe);

    let panel_envelope =
        read_visible_panel_summary(tv, &target_id, &symbol, expected_visible_price.as_deref());
    let panel = panel_envelope.pointer("/data/result").unwrap_or_else(|| {
        panic!(
            "visible panel summary missing: {}",
            summarize_probe(&panel_envelope)
        )
    });
    assert_panel_summary(panel, &symbol, expected_visible_price.as_deref());

    println!(
        "ok scanner={} chart={} quote_session={}{} visible_panel={} elapsed_ms={}",
        scanner_summary(&scanner_envelope),
        chart_summary(&chart_envelope),
        selected_probe_summary(probe),
        phase_result
            .map(|message| format!(" phase_result={message}"))
            .unwrap_or_default(),
        panel_summary(panel),
        started.elapsed().as_millis()
    );
}

fn run_scanner_quote(tv: &str, symbol: &str) -> std::process::Output {
    Command::new(tv)
        .args(["quote", symbol, "--source", "scanner"])
        .output()
        .expect("test-built tv binary should execute")
}

fn run_chart_quote(tv: &str, target_id: &str) -> std::process::Output {
    Command::new(tv)
        .args(["--target-id", target_id, "quote", "--source", "chart"])
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
                    "readiness returned one chart target without a usable id; set TV_LIVE_AFTER_HOURS_TARGET_ID"
                )
            }),
        [] => panic!(
            "readiness found no chart target; open TradingView Desktop or set TV_LIVE_AFTER_HOURS_TARGET_ID"
        ),
        _ => panic!(
            "readiness found multiple chart targets; set TV_LIVE_AFTER_HOURS_TARGET_ID for the intended chart"
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
  const sub = "codex_after_hours_panel_probe_" + Date.now();
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

fn read_visible_panel_summary(
    tv: &str,
    target_id: &str,
    symbol: &str,
    expected_visible_price: Option<&str>,
) -> Value {
    let symbol_json = serde_json::to_string(symbol).expect("symbol serializes");
    let expected_json =
        serde_json::to_string(&expected_visible_price).expect("expected price serializes");
    let script = format!(
        r#"(() => {{
  const symbol = {symbol_json};
  const expectedPrice = {expected_json};
  function visible(el) {{
    const rect = el.getBoundingClientRect && el.getBoundingClientRect();
    const style = window.getComputedStyle(el);
    return rect && rect.width > 0 && rect.height > 0 && style.visibility !== "hidden" && style.display !== "none";
  }}
  function normalizedText(el) {{
    return String(el && el.innerText || el && el.textContent || "").replace(/\s+/g, " ").trim();
  }}
  function uniquePush(values, value) {{
    if (value && !values.includes(value)) values.push(value);
  }}
  function decimalTokens(text) {{
    const matches = String(text || "").match(/\b\d{{1,4}}(?:,\d{{3}})*(?:\.\d+)?\b/g) || [];
    return Array.from(new Set(matches));
  }}
  function safeText(value, max) {{
    return String(value || "").replace(/\s+/g, " ").trim().slice(0, max);
  }}
  function attr(el, name) {{
    return el && el.getAttribute && el.getAttribute(name) || null;
  }}
  function classTokens(el) {{
    return String(el && el.className || "")
      .split(/\s+/)
      .filter(Boolean)
      .filter(token => /widget|detail|price|last|market|status|session/i.test(token))
      .slice(0, 8);
  }}
  function nodeSummary(el) {{
    if (!el) return null;
    return {{
      tag: String(el.tagName || "").toLowerCase(),
      data_name: attr(el, "data-name"),
      data_qa_id: attr(el, "data-qa-id"),
      data_test_id_widget_type: attr(el, "data-test-id-widget-type"),
      role: attr(el, "role"),
      aria_label: attr(el, "aria-label"),
      class_tokens: classTokens(el),
      text: safeText(normalizedText(el), 120),
    }};
  }}
  function ancestorChain(el) {{
    const chain = [];
    let current = el;
    for (let i = 0; current && i < 10; i += 1, current = current.parentElement) {{
      chain.push(nodeSummary(current));
    }}
    return chain.filter(Boolean);
  }}
  function findDetailsStatusNode(nodes) {{
    const status = nodes.find(el => attr(el, "data-qa-id") === "details-element status" && visible(el));
    if (status) return status;
    return nodes.find(el => /details-element status/i.test(String(attr(el, "data-qa-id") || "")) && visible(el)) || null;
  }}
  function findMatchedPriceNode(nodes, expectedPrice) {{
    const priceRegex = /\d{{1,4}}(?:,\d{{3}})*\.\d+\s*USD/;
    const candidates = nodes
      .filter(visible)
      .filter(el => {{
        const rect = el.getBoundingClientRect();
        if (rect.left < window.innerWidth * 0.58) return false;
        const text = normalizedText(el);
        if (!text || text.length > 180) return false;
        if (expectedPrice && !text.includes(expectedPrice)) return false;
        return priceRegex.test(text) || /price|lastPrice/i.test(String(el.className || ""));
      }})
      .sort((a, b) => normalizedText(a).length - normalizedText(b).length);
    return candidates[0] || null;
  }}
  function reactKeys(el) {{
    const names = [];
    let current = el;
    for (let depth = 0; current && depth < 8; depth += 1, current = current.parentElement) {{
      for (const key of Object.getOwnPropertyNames(current)) {{
        if (key.startsWith("__react")) uniquePush(names, key.replace(/\$.*/, "$"));
      }}
    }}
    return names.slice(0, 12);
  }}
  function fiberFromNode(el) {{
    let current = el;
    for (let depth = 0; current && depth < 8; depth += 1, current = current.parentElement) {{
      const key = Object.getOwnPropertyNames(current).find(name => name.startsWith("__reactFiber"));
      if (key) return current[key];
    }}
    return null;
  }}
  function componentName(fiber) {{
    if (!fiber) return null;
    const type = fiber.elementType || fiber.type;
    if (typeof type === "string") return type;
    return type && (type.displayName || type.name) || (fiber.tag != null ? "tag:" + fiber.tag : null);
  }}
  function componentChain(fiber) {{
    const names = [];
    let current = fiber;
    for (let depth = 0; current && depth < 14; depth += 1, current = current.return) {{
      uniquePush(names, componentName(current));
    }}
    return names.filter(Boolean).slice(0, 12);
  }}
  function propCandidates(fiber, expectedPrice) {{
    if (!fiber || !fiber.memoizedProps) return [];
    const hits = [];
    const seen = new Set();
    const stack = [{{ path: "memoizedProps", value: fiber.memoizedProps, depth: 0 }}];
    while (stack.length && hits.length < 10 && seen.size < 350) {{
      const item = stack.pop();
      const value = item.value;
      if (value && typeof value === "object") {{
        if (seen.has(value)) continue;
        seen.add(value);
      }}
      if (typeof value === "string" || typeof value === "number" || typeof value === "boolean") {{
        const text = String(value);
        if (
          (expectedPrice && text.includes(expectedPrice)) ||
          /アフター|after|post|USD|market/i.test(text)
        ) {{
          hits.push({{ path: item.path, value: text.slice(0, 80) }});
        }}
        continue;
      }}
      if (!value || typeof value !== "object" || item.depth >= 5) continue;
      const keys = Object.keys(value).slice(0, 30);
      for (const key of keys.reverse()) {{
        if (/^_(owner|store|source|self)$/.test(key)) continue;
        const child = value[key];
        if (typeof child === "function") continue;
        stack.push({{ path: item.path + "." + key, value: child, depth: item.depth + 1 }});
      }}
    }}
    return hits;
  }}
  const nodes = Array.from(document.querySelectorAll("aside, section, div, span"));
  const rightTexts = [];
  for (const el of nodes) {{
    if (!visible(el)) continue;
    const rect = el.getBoundingClientRect();
    if (rect.left < window.innerWidth * 0.58) continue;
    const text = normalizedText(el);
    if (!text || text.length > 280) continue;
    if (
      text.includes(symbol) ||
      /アフター|マーケット|After|Market|Post|USD|最終更新/i.test(text) ||
      decimalTokens(text).length > 0
    ) {{
      uniquePush(rightTexts, text);
    }}
  }}
  const panelText = rightTexts.join("\n");
  const afterRegex = /アフター\s*マーケット|after[-\s]?market|post[-\s]?market/i;
  const compactSnippets = rightTexts
    .filter(text => text.includes(symbol) || /アフター|マーケット|After|Market|Post|USD|最終更新/i.test(text))
    .slice(0, 12);
  const afterSnippet = compactSnippets.find(text => afterRegex.test(text) && /\d{{1,4}}(?:,\d{{3}})*\.\d+\s*USD/.test(text)) || "";
  const afterMatch = afterRegex.exec(panelText);
  const afterIndex = afterMatch ? afterMatch.index : -1;
  const beforeAfterLabel = afterIndex >= 0 ? panelText.slice(Math.max(0, afterIndex - 220), afterIndex) : "";
  const afterWindow = afterIndex >= 0 ? panelText.slice(Math.max(0, afterIndex - 220), Math.min(panelText.length, afterIndex + 220)) : "";
  const snippetUsdMatches = Array.from(afterSnippet.matchAll(/(\d{{1,4}}(?:,\d{{3}})*\.\d+)\s*USD/g)).map(match => match[1]);
  const beforeUsdMatches = Array.from(beforeAfterLabel.matchAll(/(\d{{1,4}}(?:,\d{{3}})*\.\d+)\s*USD/g)).map(match => match[1]);
  const afterPrice = snippetUsdMatches.length
    ? snippetUsdMatches[snippetUsdMatches.length - 1]
    : (beforeUsdMatches.length ? beforeUsdMatches[beforeUsdMatches.length - 1] : null);
  const regularMatches = Array.from(panelText.matchAll(/(\d{{1,4}}(?:,\d{{3}})*\.\d+)\s*USD/g)).map(match => match[1]);
  const detailStatusNode = findDetailsStatusNode(nodes);
  const matchedPriceNode = findMatchedPriceNode(nodes, expectedPrice || afterPrice);
  const matchedFiber = fiberFromNode(matchedPriceNode);
  return {{
    symbol,
    symbol_seen: panelText.includes(symbol),
    after_market_label_seen: afterIndex >= 0,
    visible_after_market_price: afterPrice,
    expected_visible_price: expectedPrice,
    expected_visible_price_seen: expectedPrice ? panelText.includes(expectedPrice) : null,
    regular_usd_candidates: Array.from(new Set(regularMatches)).slice(0, 8),
    numeric_candidates: decimalTokens(panelText).slice(0, 24),
    after_market_window_numbers: decimalTokens(afterWindow).slice(0, 12),
    snippet_count: compactSnippets.length,
    snippets: compactSnippets,
    low_level: {{
      matched_node_found: Boolean(matchedPriceNode),
      matched_node: nodeSummary(matchedPriceNode),
      detail_status_node_found: Boolean(detailStatusNode),
      detail_status_node: nodeSummary(detailStatusNode),
      ancestor_chain: ancestorChain(matchedPriceNode).slice(0, 8),
      react: {{
        react_key_count: reactKeys(matchedPriceNode).length,
        react_keys: reactKeys(matchedPriceNode),
        fiber_found: Boolean(matchedFiber),
        component_names: componentChain(matchedFiber),
        prop_candidates: propCandidates(matchedFiber, expectedPrice || afterPrice),
      }},
    }},
  }};
}})()"#
    );
    let output = run_ui_eval(tv, target_id, &script);
    parse_command_json("visible panel summary", output, Duration::ZERO)
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

fn assert_chart_quote(envelope: &Value, elapsed: Duration) {
    let data = envelope.get("data").unwrap_or(&Value::Null);
    if envelope.get("success").and_then(Value::as_bool) != Some(true)
        || envelope.get("command").and_then(Value::as_str) != Some("quote")
        || data.get("source").and_then(Value::as_str) != Some("chart_api")
        || data.get("requires_desktop").and_then(Value::as_bool) != Some(true)
        || data.get("session_boundary").is_none()
    {
        panic!(
            "chart quote validation failed: elapsed_ms={} summary={}",
            elapsed.as_millis(),
            summarize_probe(envelope)
        );
    }
}

fn assert_probe_result(
    symbols: &[String],
    expected_phase: Option<&str>,
    probe: &Value,
) -> Option<String> {
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

    let mut observed_phases = Vec::new();
    for update in updates.values() {
        let selected = update.get("selected").unwrap_or(&Value::Null);
        let phase = selected
            .get("market-status")
            .and_then(|status| status.get("phase"))
            .and_then(Value::as_str);
        if let Some(phase) = phase {
            push_unique(&mut observed_phases, phase.to_string());
        }
    }
    if let Some(expected) = expected_phase
        && !observed_phases
            .iter()
            .any(|phase| phase_matches_expected(expected, phase))
    {
        let observed = if observed_phases.is_empty() {
            "<missing>".to_string()
        } else {
            observed_phases.join(",")
        };
        return Some(format!(
            "not_yet_in_expected_phase expected={} observed={}",
            expected, observed
        ));
    }
    None
}

fn assert_panel_summary(panel: &Value, symbol: &str, expected_visible_price: Option<&str>) {
    if panel.get("symbol").and_then(Value::as_str) != Some(symbol)
        || panel.get("symbol_seen").and_then(Value::as_bool) != Some(true)
    {
        panic!("visible panel did not expose requested symbol: {panel}");
    }
    if let Some(expected) = expected_visible_price
        && panel
            .get("expected_visible_price_seen")
            .and_then(Value::as_bool)
            != Some(true)
    {
        panic!(
            "visible panel did not contain expected price {}: {}",
            expected,
            panel_summary(panel)
        );
    }
    if expected_visible_price.is_some()
        && panel
            .pointer("/low_level/matched_node_found")
            .and_then(Value::as_bool)
            != Some(true)
    {
        panic!(
            "visible panel found expected price but did not identify a matched low-level node: {}",
            panel_summary(panel)
        );
    }
}

fn phase_matches_expected(expected: &str, observed: &str) -> bool {
    normalize_phase(expected) == normalize_phase(observed)
}

fn normalize_phase(phase: &str) -> String {
    phase
        .trim()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn scanner_summary(envelope: &Value) -> String {
    let data = envelope.get("data").unwrap_or(&Value::Null);
    format!(
        "last={} close={} update_mode={} delay_seconds={} premarket_close={} postmarket_close={}",
        display_value(data.get("last")),
        display_value(data.get("close")),
        string_field(data, "update_mode").unwrap_or("<missing>"),
        display_value(data.get("delay_seconds")),
        display_value(data.pointer("/extended_hours/premarket/close")),
        display_value(data.pointer("/extended_hours/postmarket/close")),
    )
}

fn chart_summary(envelope: &Value) -> String {
    let data = envelope.get("data").unwrap_or(&Value::Null);
    format!(
        "symbol={} last={} close={} time={} session_status={}",
        string_field(data, "symbol").unwrap_or("<missing>"),
        display_value(data.get("last")),
        display_value(data.get("close")),
        display_value(data.get("time")),
        string_field(
            data.get("session_boundary").unwrap_or(&Value::Null),
            "extended_hours_status"
        )
        .unwrap_or("<missing>"),
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

fn panel_summary(panel: &Value) -> String {
    format!(
        "symbol_seen={} after_label={} after_price={} expected_price={} expected_seen={} usd_candidates={} after_window_numbers={} snippets={} low_level={}",
        bool_field(panel, "symbol_seen"),
        bool_field(panel, "after_market_label_seen"),
        display_value(panel.get("visible_after_market_price")),
        display_value(panel.get("expected_visible_price")),
        display_value(panel.get("expected_visible_price_seen")),
        display_array(panel.get("regular_usd_candidates")),
        display_array(panel.get("after_market_window_numbers")),
        display_array(panel.get("snippets")),
        low_level_summary(panel.get("low_level")),
    )
}

fn low_level_summary(value: Option<&Value>) -> String {
    let Some(value) = value else {
        return "<missing>".to_string();
    };
    let react = value.get("react").unwrap_or(&Value::Null);
    format!(
        "matched={} matched_node={} detail_status={} ancestors={} react_fiber={} components={} prop_candidates={}",
        bool_field(value, "matched_node_found"),
        node_summary(value.get("matched_node")),
        bool_field(value, "detail_status_node_found"),
        display_node_array(value.get("ancestor_chain")),
        bool_field(react, "fiber_found"),
        display_array(react.get("component_names")),
        display_prop_candidates(react.get("prop_candidates")),
    )
}

fn node_summary(value: Option<&Value>) -> String {
    let Some(value) = value else {
        return "<none>".to_string();
    };
    let tag = string_field(value, "tag").unwrap_or("<tag>");
    let data_name = string_field(value, "data_name").unwrap_or("-");
    let qa = string_field(value, "data_qa_id").unwrap_or("-");
    let widget = string_field(value, "data_test_id_widget_type").unwrap_or("-");
    let classes = display_array(value.get("class_tokens"));
    format!("{tag}/data={data_name}/qa={qa}/widget={widget}/class={classes}")
}

fn display_node_array(value: Option<&Value>) -> String {
    value
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .take(8)
                .map(|value| node_summary(Some(value)))
                .collect::<Vec<_>>()
                .join(">")
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "<none>".to_string())
}

fn display_prop_candidates(value: Option<&Value>) -> String {
    value
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .take(8)
                .map(|value| {
                    format!(
                        "{}={}",
                        string_field(value, "path").unwrap_or("<path>"),
                        string_field(value, "value").unwrap_or("<value>")
                    )
                    .replace(';', ",")
                    .replace('\n', " ")
                })
                .collect::<Vec<_>>()
                .join("|")
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "<none>".to_string())
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
            .unwrap_or_else(|| "<missing>".to_string()),
        string_field(error, "kind").unwrap_or("<none>"),
        string_field(error, "message").unwrap_or("<none>"),
        panel_safe_result(result)
    )
}

fn panel_safe_result(value: &Value) -> String {
    if value.get("visible_after_market_price").is_some() || value.get("snippets").is_some() {
        panel_summary(value)
    } else if value.get("updates").is_some() {
        selected_probe_summary(value)
    } else {
        format!(
            "started={} done={} update_count={}",
            display_value(value.get("started")),
            display_value(value.get("done")),
            display_value(value.get("update_count"))
        )
    }
}

fn display_value(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(value)) if !value.trim().is_empty() => value.trim().to_string(),
        Some(Value::Number(value)) => value.to_string(),
        Some(Value::Bool(value)) => value.to_string(),
        Some(Value::Null) | None => "null".to_string(),
        Some(other) => other.to_string(),
    }
}

fn display_array(value: Option<&Value>) -> String {
    value
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .take(12)
                .map(|value| match value {
                    Value::String(value) => value.replace(';', ",").replace('\n', " "),
                    other => other.to_string(),
                })
                .collect::<Vec<_>>()
                .join("|")
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "<none>".to_string())
}

fn string_field<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value.get(field).and_then(Value::as_str)
}

fn bool_field(value: &Value, field: &str) -> String {
    value
        .get(field)
        .and_then(Value::as_bool)
        .map(|value| value.to_string())
        .unwrap_or_else(|| "<missing>".to_string())
}

fn compact_symbol(symbol: &str) -> String {
    if symbol.len() <= 32 {
        symbol.to_string()
    } else {
        format!("{}...", &symbol[..32])
    }
}

fn env_string(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[test]
fn phase_matching_accepts_tradingview_extended_hours_aliases() {
    assert!(phase_matches_expected("postmarket", "post-market"));
    assert!(phase_matches_expected("post-market", "postmarket"));
    assert!(phase_matches_expected("premarket", "pre-market"));
    assert!(phase_matches_expected("pre-market", "premarket"));
}

#[test]
fn phase_matching_keeps_regular_distinct_from_extended_hours() {
    assert!(!phase_matches_expected("postmarket", "regular"));
    assert!(!phase_matches_expected("premarket", "regular"));
}
