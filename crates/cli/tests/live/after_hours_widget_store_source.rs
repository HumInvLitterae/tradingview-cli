use std::{process::Command, time::Duration};

use serde_json::Value;

const DEFAULT_SYMBOL: &str = "RKLB";

#[test]
#[ignore = "requires a running TradingView Desktop CDP session and TV_LIVE_AFTER_HOURS_WIDGET_STORE_SMOKE=1"]
fn after_hours_widget_store_live_smoke() {
    if std::env::var("TV_LIVE_AFTER_HOURS_WIDGET_STORE_SMOKE")
        .ok()
        .as_deref()
        != Some("1")
    {
        panic!(
            "live after-hours widget store smoke is gated; set TV_LIVE_AFTER_HOURS_WIDGET_STORE_SMOKE=1 and run with --ignored"
        );
    }

    let tv = env!("CARGO_BIN_EXE_tv");
    let symbol = env_string("TV_LIVE_AFTER_HOURS_SYMBOL").unwrap_or_else(|| DEFAULT_SYMBOL.into());
    let expected_phase = env_string("TV_LIVE_AFTER_HOURS_EXPECT_PHASE");
    let expected_visible_price = env_string("TV_LIVE_AFTER_HOURS_EXPECT_VISIBLE_PRICE");
    let target_id = env_string("TV_LIVE_AFTER_HOURS_TARGET_ID")
        .unwrap_or_else(|| resolve_single_chart_target(tv));

    println!(
        "after-hours widget store smoke: symbol={} target_id={} expected_phase={} expected_visible_price={}",
        symbol,
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

    let envelope = read_widget_store_summary(
        tv,
        &target_id,
        &symbol,
        expected_phase.as_deref(),
        expected_visible_price.as_deref(),
    );
    let summary = envelope.pointer("/data/result").unwrap_or_else(|| {
        panic!(
            "widget store summary missing: {}",
            summarize_result(&envelope)
        )
    });
    assert_widget_summary(summary, &symbol, expected_visible_price.as_deref());

    println!("ok {}", widget_summary(summary));
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

fn read_widget_store_summary(
    tv: &str,
    target_id: &str,
    symbol: &str,
    expected_phase: Option<&str>,
    expected_visible_price: Option<&str>,
) -> Value {
    let symbol_json = serde_json::to_string(symbol).expect("symbol serializes");
    let expected_phase_json =
        serde_json::to_string(&expected_phase).expect("expected phase serializes");
    let expected_price_json =
        serde_json::to_string(&expected_visible_price).expect("expected price serializes");
    let script = format!(
        r#"(() => {{
  const symbol = {symbol_json};
  const expectedPhase = {expected_phase_json};
  const expectedPrice = {expected_price_json};
  const MAX_HITS = 60;

  function visible(el) {{
    const rect = el.getBoundingClientRect && el.getBoundingClientRect();
    const style = window.getComputedStyle(el);
    return rect && rect.width > 0 && rect.height > 0 && style.visibility !== "hidden" && style.display !== "none";
  }}
  function normalizedText(el) {{
    return String(el && el.innerText || el && el.textContent || "").replace(/\s+/g, " ").trim();
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
      data_qa_id: attr(el, "data-qa-id"),
      data_test_id_widget_type: attr(el, "data-test-id-widget-type"),
      class_tokens: classTokens(el),
      text: normalizedText(el).slice(0, 120),
    }};
  }}
  function detailRoot(el) {{
    let current = el;
    for (let depth = 0; current && depth < 12; depth += 1, current = current.parentElement) {{
      if (attr(current, "data-test-id-widget-type") === "detail") return current;
      if (attr(current, "data-qa-id") === "details-element status") return current;
    }}
    return null;
  }}
  function rightSide(el) {{
    if (!el || !visible(el)) return false;
    const rect = el.getBoundingClientRect();
    return rect.left >= window.innerWidth * 0.58;
  }}
  function hasAfterLabel(text) {{
    return /アフター\s*マーケット|after[-\s]?market|post[-\s]?market/i.test(String(text || ""));
  }}
  function afterMarketRoot(nodes) {{
    const labelNode = nodes
      .filter(rightSide)
      .filter(el => {{
        const text = normalizedText(el);
        return text && text.length <= 240 && hasAfterLabel(text);
      }})
      .sort((a, b) => normalizedText(a).length - normalizedText(b).length)[0];
    let current = labelNode;
    for (let depth = 0; current && depth < 8; depth += 1, current = current.parentElement) {{
      const text = normalizedText(current);
      if (text && text.length <= 900 && hasAfterLabel(text) && /\b\d{{1,4}}(?:,\d{{3}})*\.\d+\b/.test(text)) {{
        return current;
      }}
    }}
    return detailRoot(labelNode);
  }}
  function likelyRightDetailNode(el) {{
    if (!el || !visible(el)) return false;
    if (!rightSide(el)) return false;
    if (/price-axis-container/i.test(String(el.className || ""))) return false;
    return Boolean(detailRoot(el));
  }}
  function decimalTokens(text) {{
    const matches = String(text || "").match(/\b\d{{1,4}}(?:,\d{{3}})*(?:\.\d+)?\b/g) || [];
    return Array.from(new Set(matches));
  }}
  function findMatchedPriceNode(nodes, expectedPrice) {{
    const priceRegex = /\b\d{{1,4}}(?:,\d{{3}})*\.\d+\b/;
    const root = afterMarketRoot(nodes);
    const sourceNodes = root ? Array.from(root.querySelectorAll("div, span, button, section, article")) : nodes;
    const candidates = sourceNodes
      .concat(nodes.filter(el => !root && rightSide(el)))
      .filter(el => root ? visible(el) : (rightSide(el) && visible(el)))
      .filter(el => {{
        if (/price-axis-container/i.test(String(el.className || ""))) return false;
        const text = normalizedText(el);
        if (!text || text.length > 240) return false;
        if (expectedPrice && !text.includes(expectedPrice)) return false;
        if (!priceRegex.test(text)) return false;
        const rootText = normalizedText(root || detailRoot(el));
        if (/アフター\s*マーケット|after[-\s]?market|post[-\s]?market|USD/i.test(rootText)) return true;
        return Boolean(symbol && text.includes(symbol));
      }})
      .sort((a, b) => {{
        const aText = normalizedText(a);
        const bText = normalizedText(b);
        const aScore = (/USD/i.test(aText) ? 0 : 1) + (/アフター|after|post/i.test(aText) ? 0 : 1);
        const bScore = (/USD/i.test(bText) ? 0 : 1) + (/アフター|after|post/i.test(bText) ? 0 : 1);
        return aScore - bScore || aText.length - bText.length;
      }});
    return candidates[0] || null;
  }}
  function findFiber(el) {{
    let current = el;
    for (let depth = 0; current && depth < 10; depth += 1, current = current.parentElement) {{
      const key = Object.getOwnPropertyNames(current).find(name => name.startsWith("__reactFiber"));
      if (key) return current[key];
    }}
    return null;
  }}
  function reactPropPayloads(el) {{
    const payloads = [];
    let current = el;
    for (let depth = 0; current && depth < 10; depth += 1, current = current.parentElement) {{
      const key = Object.getOwnPropertyNames(current).find(name => name.startsWith("__reactProps"));
      if (key) payloads.push({{ depth, value: current[key] }});
    }}
    return payloads;
  }}
  function componentName(fiber) {{
    if (!fiber) return null;
    const type = fiber.elementType || fiber.type;
    if (typeof type === "string") return type;
    return type && (type.displayName || type.name) || (fiber.tag != null ? "tag:" + fiber.tag : null);
  }}
  function containsNeedle(path, text) {{
    const value = String(text || "");
    const pathText = String(path || "");
    if (/url|priceFormatter|formatter/i.test(pathText) || /^https?:\/\//i.test(value)) return false;
    if (/^USD$/.test(value)) return false;
    return (
      (expectedPrice && value.includes(expectedPrice)) ||
      (symbol && value.includes(symbol)) ||
      /アフター|after|post|market|pre_market|post_market/i.test(value) ||
      (/price|last|close|session|market|change/i.test(pathText) && /\d/.test(value))
    );
  }}
  function safePrimitive(value) {{
    if (typeof value === "string" || typeof value === "number" || typeof value === "boolean") {{
      return String(value).slice(0, 96);
    }}
    return null;
  }}
  function pushHit(hits, source, component, path, value) {{
    if (hits.length >= MAX_HITS) return;
    const primitive = safePrimitive(value);
    if (primitive == null || !containsNeedle(path, primitive)) return;
    hits.push({{ source, component, path, value: primitive }});
  }}
  function scanObject(root, source, component, hits) {{
    if (!root || typeof root !== "object") return 0;
    const seen = new Set();
    const stack = [{{ path: source, value: root, depth: 0 }}];
    let visited = 0;
    while (stack.length && visited < 550 && hits.length < MAX_HITS) {{
      const item = stack.pop();
      const value = item.value;
      visited += 1;
      const primitive = safePrimitive(value);
      if (primitive != null) {{
        pushHit(hits, source, component, item.path, primitive);
        continue;
      }}
      if (!value || typeof value !== "object" || item.depth >= 6) continue;
      if (seen.has(value)) continue;
      seen.add(value);
      const keys = Object.keys(value).slice(0, 40);
      for (const key of keys.reverse()) {{
        if (/^_(owner|store|source|self)$/.test(key)) continue;
        const child = value[key];
        if (typeof child === "function") continue;
        stack.push({{ path: item.path + "." + key, value: child, depth: item.depth + 1 }});
      }}
    }}
    return visited;
  }}
  function fiberChain(fiber) {{
    const chain = [];
    let current = fiber;
    for (let depth = 0; current && depth < 24; depth += 1, current = current.return) {{
      const name = componentName(current);
      chain.push({{
        depth,
        tag: current.tag,
        component: name,
        has_props: Boolean(current.memoizedProps),
        has_state: Boolean(current.memoizedState),
        has_update_queue: Boolean(current.updateQueue),
        has_state_node: Boolean(current.stateNode),
      }});
    }}
    return chain;
  }}
  const nodes = Array.from(document.querySelectorAll("aside, section, div, span"));
  const matchedNode = findMatchedPriceNode(nodes, expectedPrice);
  const matchedText = normalizedText(matchedNode);
  const fiber = findFiber(matchedNode);
  const chain = fiberChain(fiber);
  const hits = [];
  let scannedNodes = 0;
  const reactProps = reactPropPayloads(matchedNode);
  for (const item of reactProps) {{
    scannedNodes += scanObject(item.value, "reactProps.depth" + item.depth, "dom-react-props", hits);
  }}
  let current = fiber;
  for (let depth = 0; current && depth < 24 && hits.length < MAX_HITS; depth += 1, current = current.return) {{
    const component = componentName(current) || "unknown";
    scannedNodes += scanObject(current.memoizedProps, "memoizedProps", component, hits);
    scannedNodes += scanObject(current.memoizedState, "memoizedState", component, hits);
    scannedNodes += scanObject(current.updateQueue, "updateQueue", component, hits);
    if (current.stateNode && current.stateNode !== matchedNode && !(current.stateNode instanceof Element)) {{
      scannedNodes += scanObject(current.stateNode, "stateNode", component, hits);
    }}
  }}
  const rightText = Array.from(document.querySelectorAll("aside, section, div, span"))
    .filter(visible)
    .filter(el => el.getBoundingClientRect().left >= window.innerWidth * 0.58)
    .map(normalizedText)
    .filter(Boolean)
    .join("\n");
  const afterLabelSeen = /アフター\s*マーケット|after[-\s]?market|post[-\s]?market/i.test(rightText);
  const observedPhase = afterLabelSeen ? "post-market" : null;
  const expectedPhaseMatches = expectedPhase ? String(expectedPhase).replace(/[^a-z0-9]/gi, "").toLowerCase() === String(observedPhase || "").replace(/[^a-z0-9]/gi, "").toLowerCase() : null;
  return {{
    symbol,
    expected_phase: expectedPhase,
    observed_phase: observedPhase,
    expected_phase_matches: expectedPhaseMatches,
    expected_visible_price: expectedPrice,
    expected_visible_price_seen: expectedPrice ? rightText.includes(expectedPrice) : null,
    matched_node_found: Boolean(matchedNode),
    matched_node: nodeSummary(matchedNode),
    matched_text: matchedText.slice(0, 120),
    numeric_candidates: decimalTokens(rightText).slice(0, 16),
    fiber_found: Boolean(fiber),
    fiber_chain: chain,
    hit_count: hits.length,
    hits,
    scanned_node_count: scannedNodes,
  }};
}})()"#
    );
    let output = run_ui_eval(tv, target_id, &script);
    parse_command_json("widget store summary", output, Duration::ZERO)
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
            summarize_result(&parsed)
        );
    }
    parsed
}

fn assert_widget_summary(summary: &Value, symbol: &str, expected_visible_price: Option<&str>) {
    if summary.get("symbol").and_then(Value::as_str) != Some(symbol)
        || summary.get("matched_node_found").and_then(Value::as_bool) != Some(true)
    {
        panic!("widget store summary did not find a visible right-panel price node: {summary}");
    }
    if let Some(expected) = expected_visible_price
        && summary
            .get("expected_visible_price_seen")
            .and_then(Value::as_bool)
            != Some(true)
    {
        panic!(
            "widget store summary did not contain expected visible price {}: {}",
            expected,
            widget_summary(summary)
        );
    }
}

fn widget_summary(summary: &Value) -> String {
    format!(
        "symbol={} phase={} expected_phase_match={} expected_price={} expected_seen={} matched={} fiber={} components={} hit_count={} hits={} numbers={}",
        string_field(summary, "symbol").unwrap_or("<missing>"),
        string_field(summary, "observed_phase").unwrap_or("<missing>"),
        display_value(summary.get("expected_phase_matches")),
        display_value(summary.get("expected_visible_price")),
        display_value(summary.get("expected_visible_price_seen")),
        display_value(summary.get("matched_node_found")),
        display_value(summary.get("fiber_found")),
        component_summary(summary.get("fiber_chain")),
        display_value(summary.get("hit_count")),
        hit_summary(summary.get("hits")),
        display_array(summary.get("numeric_candidates")),
    )
}

fn component_summary(value: Option<&Value>) -> String {
    value
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .take(20)
                .map(|value| {
                    format!(
                        "{}:{}",
                        display_value(value.get("depth")),
                        string_field(value, "component").unwrap_or("<component>")
                    )
                })
                .collect::<Vec<_>>()
                .join("|")
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "<none>".to_string())
}

fn hit_summary(value: Option<&Value>) -> String {
    value
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .take(12)
                .map(|value| {
                    format!(
                        "{}:{}:{}={}",
                        string_field(value, "source").unwrap_or("<source>"),
                        string_field(value, "component").unwrap_or("<component>"),
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

fn summarize_result(value: &Value) -> String {
    let data = value.get("data").unwrap_or(value);
    let result = data.get("result").unwrap_or(data);
    if result.get("fiber_found").is_some() {
        widget_summary(result)
    } else {
        format!(
            "success={} kind={} message={}",
            value
                .get("success")
                .and_then(Value::as_bool)
                .map(|value| value.to_string())
                .unwrap_or_else(|| "<missing>".to_string()),
            value
                .pointer("/error/kind")
                .and_then(Value::as_str)
                .unwrap_or("<none>"),
            value
                .pointer("/error/message")
                .and_then(Value::as_str)
                .unwrap_or("<none>"),
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
                .join(",")
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "<none>".to_string())
}

fn string_field<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value.get(field).and_then(Value::as_str)
}

fn env_string(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[test]
fn widget_summary_reports_components_and_hits() {
    let value = serde_json::json!({
        "symbol": "RKLB",
        "observed_phase": "post-market",
        "expected_phase_matches": true,
        "expected_visible_price": "110.17",
        "expected_visible_price_seen": true,
        "matched_node_found": true,
        "fiber_found": true,
        "fiber_chain": [{"depth": 0, "component": "span"}],
        "hit_count": 1,
        "hits": [{"source": "memoizedProps", "component": "span", "path": "memoizedProps.children", "value": "110.17"}],
        "numeric_candidates": ["110.17"]
    });
    let summary = widget_summary(&value);
    assert!(summary.contains("span"));
    assert!(summary.contains("110.17"));
}
