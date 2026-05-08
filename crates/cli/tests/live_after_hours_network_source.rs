use std::{
    collections::HashMap,
    process::Command,
    time::{Duration, Instant},
};

use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{Error as WsError, Message},
};

const DEFAULT_SYMBOL: &str = "RKLB";
const DEFAULT_QUALIFIED_SYMBOL: &str = "NASDAQ:RKLB";
const DEFAULT_CAPTURE_DURATION_MS: u64 = 15_000;

#[derive(Debug, Clone, Deserialize)]
struct CdpTarget {
    id: String,
    #[serde(default)]
    url: String,
    #[serde(rename = "webSocketDebuggerUrl")]
    web_socket_debugger_url: Option<String>,
}

#[derive(Debug, Default)]
struct NetworkStats {
    websocket_created: u64,
    websocket_frames_received: u64,
    websocket_frames_sent: u64,
    response_count: u64,
    event_count: u64,
    candidates: Vec<CandidateSummary>,
}

#[derive(Debug, Clone)]
struct CandidateSummary {
    kind: &'static str,
    source: String,
    byte_len: usize,
    contains_symbol: bool,
    contains_qualified_symbol: bool,
    contains_expected_price: bool,
    contains_after_token: bool,
    decimal_token_count: usize,
}

#[derive(Debug, Clone, Copy)]
struct ProbeNeedles<'a> {
    symbol: &'a str,
    qualified_symbol: &'a str,
    expected_price: Option<&'a str>,
}

#[tokio::test]
#[ignore = "requires a running TradingView Desktop CDP session and TV_LIVE_AFTER_HOURS_NETWORK_SMOKE=1"]
async fn after_hours_network_source_live_smoke() {
    if std::env::var("TV_LIVE_AFTER_HOURS_NETWORK_SMOKE")
        .ok()
        .as_deref()
        != Some("1")
    {
        panic!(
            "live after-hours network source smoke is gated; set TV_LIVE_AFTER_HOURS_NETWORK_SMOKE=1 and run with --ignored"
        );
    }

    let tv = env!("CARGO_BIN_EXE_tv");
    let symbol = env_string("TV_LIVE_AFTER_HOURS_SYMBOL").unwrap_or_else(|| DEFAULT_SYMBOL.into());
    let qualified_symbol = env_string("TV_LIVE_AFTER_HOURS_QUALIFIED_SYMBOL")
        .unwrap_or_else(|| DEFAULT_QUALIFIED_SYMBOL.into());
    let expected_phase = env_string("TV_LIVE_AFTER_HOURS_EXPECT_PHASE");
    let expected_visible_price = env_string("TV_LIVE_AFTER_HOURS_EXPECT_VISIBLE_PRICE");
    let duration = env_duration_ms(
        "TV_LIVE_AFTER_HOURS_NETWORK_DURATION_MS",
        DEFAULT_CAPTURE_DURATION_MS,
    );
    let target = resolve_chart_target().await;

    println!(
        "after-hours network source smoke: symbol={} qualified={} target_id={} expected_phase={} expected_visible_price={} duration_ms={}",
        symbol,
        qualified_symbol,
        if std::env::var("TV_LIVE_AFTER_HOURS_TARGET_ID")
            .ok()
            .is_some()
        {
            "<provided>"
        } else {
            "<auto>"
        },
        expected_phase.as_deref().unwrap_or("<none>"),
        expected_visible_price.as_deref().unwrap_or("<none>"),
        duration.as_millis(),
    );

    let scanner_summary = scanner_quote_summary(tv, &symbol);
    let started = Instant::now();
    let (mut stream, _) = connect_async(
        target
            .web_socket_debugger_url
            .as_deref()
            .expect("selected target should include a webSocketDebuggerUrl"),
    )
    .await
    .expect("CDP websocket should connect");
    let mut next_id = 1_u64;
    let mut request_urls = HashMap::new();
    let mut stats = NetworkStats::default();

    call_cdp(
        &mut stream,
        &mut next_id,
        "Network.enable",
        json!({}),
        &mut request_urls,
        &mut stats,
        ProbeNeedles {
            symbol: &symbol,
            qualified_symbol: &qualified_symbol,
            expected_price: expected_visible_price.as_deref(),
        },
    )
    .await
    .expect("Network.enable should succeed");

    let visible_before = evaluate_visible_panel(
        &mut stream,
        &mut next_id,
        &mut request_urls,
        &mut stats,
        ProbeNeedles {
            symbol: &symbol,
            qualified_symbol: &qualified_symbol,
            expected_price: expected_visible_price.as_deref(),
        },
    )
    .await;

    capture_events(
        &mut stream,
        duration,
        &mut request_urls,
        &mut stats,
        ProbeNeedles {
            symbol: &symbol,
            qualified_symbol: &qualified_symbol,
            expected_price: expected_visible_price.as_deref(),
        },
    )
    .await;

    let visible_after = evaluate_visible_panel(
        &mut stream,
        &mut next_id,
        &mut request_urls,
        &mut stats,
        ProbeNeedles {
            symbol: &symbol,
            qualified_symbol: &qualified_symbol,
            expected_price: expected_visible_price.as_deref(),
        },
    )
    .await;

    if let Some(expected) = expected_visible_price.as_deref()
        && visible_after
            .get("expected_visible_price_seen")
            .and_then(Value::as_bool)
            != Some(true)
        && visible_before
            .get("expected_visible_price_seen")
            .and_then(Value::as_bool)
            != Some(true)
    {
        panic!(
            "visible panel did not contain expected price {}: before={} after={} network={}",
            expected,
            visible_summary(&visible_before),
            visible_summary(&visible_after),
            network_summary(&stats)
        );
    }

    if let Some(expected_phase) = expected_phase.as_deref() {
        let observed = visible_after
            .get("phase")
            .and_then(Value::as_str)
            .or_else(|| visible_before.get("phase").and_then(Value::as_str));
        if let Some(observed) = observed
            && !phase_matches_expected(expected_phase, observed)
        {
            println!(
                "phase_result=not_yet_in_expected_phase expected={} observed={}",
                expected_phase, observed
            );
        }
    }

    println!(
        "ok scanner={} visible_before={} visible_after={} network={} elapsed_ms={}",
        scanner_summary,
        visible_summary(&visible_before),
        visible_summary(&visible_after),
        network_summary(&stats),
        started.elapsed().as_millis()
    );
}

async fn resolve_chart_target() -> CdpTarget {
    let targets = fetch_targets().await;
    if let Some(target_id) = env_string("TV_LIVE_AFTER_HOURS_TARGET_ID") {
        return targets
            .into_iter()
            .find(|target| target.id == target_id)
            .unwrap_or_else(|| {
                panic!(
                    "provided target id was not found in CDP target list; choose a current chart target"
                )
            });
    }
    let chart_targets = targets
        .into_iter()
        .filter(|target| target.url.to_lowercase().contains("tradingview.com/chart"))
        .collect::<Vec<_>>();
    match chart_targets.as_slice() {
        [target] => target.clone(),
        [] => panic!("no chart target found; open TradingView Desktop chart"),
        _ => panic!(
            "multiple chart targets found; set TV_LIVE_AFTER_HOURS_TARGET_ID for the intended chart"
        ),
    }
}

async fn fetch_targets() -> Vec<CdpTarget> {
    let host = std::env::var("TV_CDP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("TV_CDP_PORT").unwrap_or_else(|_| "9222".to_string());
    let url = format!("http://{host}:{port}/json/list");
    reqwest::get(&url)
        .await
        .expect("CDP target list should be reachable")
        .json::<Vec<CdpTarget>>()
        .await
        .expect("CDP target list should parse")
}

async fn call_cdp(
    stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    next_id: &mut u64,
    method: &str,
    params: Value,
    request_urls: &mut HashMap<String, String>,
    stats: &mut NetworkStats,
    needles: ProbeNeedles<'_>,
) -> Result<Value, WsError> {
    let id = *next_id;
    *next_id += 1;
    stream
        .send(Message::Text(
            json!({ "id": id, "method": method, "params": params })
                .to_string()
                .into(),
        ))
        .await?;
    while let Some(message) = stream.next().await {
        let message = message?;
        let Some(value) = message_text_json(message) else {
            continue;
        };
        if value.get("id").and_then(Value::as_u64) == Some(id) {
            return Ok(value);
        }
        handle_cdp_event(&value, request_urls, stats, needles);
    }
    Ok(Value::Null)
}

async fn evaluate_visible_panel(
    stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    next_id: &mut u64,
    request_urls: &mut HashMap<String, String>,
    stats: &mut NetworkStats,
    needles: ProbeNeedles<'_>,
) -> Value {
    let expression = visible_panel_expression(needles.symbol, needles.expected_price);
    call_cdp(
        stream,
        next_id,
        "Runtime.evaluate",
        json!({
            "expression": expression,
            "returnByValue": true,
            "awaitPromise": false,
        }),
        request_urls,
        stats,
        needles,
    )
    .await
    .expect("Runtime.evaluate should succeed")
    .pointer("/result/result/value")
    .cloned()
    .unwrap_or(Value::Null)
}

async fn capture_events(
    stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    duration: Duration,
    request_urls: &mut HashMap<String, String>,
    stats: &mut NetworkStats,
    needles: ProbeNeedles<'_>,
) {
    let deadline = Instant::now() + duration;
    while let Some(remaining) = deadline.checked_duration_since(Instant::now()) {
        let message = tokio::time::timeout(remaining, stream.next()).await;
        let Ok(Some(Ok(message))) = message else {
            break;
        };
        let Some(value) = message_text_json(message) else {
            continue;
        };
        handle_cdp_event(&value, request_urls, stats, needles);
    }
}

fn message_text_json(message: Message) -> Option<Value> {
    match message {
        Message::Text(text) => serde_json::from_str(&text).ok(),
        Message::Binary(bytes) => serde_json::from_slice(&bytes).ok(),
        _ => None,
    }
}

fn handle_cdp_event(
    value: &Value,
    request_urls: &mut HashMap<String, String>,
    stats: &mut NetworkStats,
    needles: ProbeNeedles<'_>,
) {
    let Some(method) = value.get("method").and_then(Value::as_str) else {
        return;
    };
    stats.event_count += 1;
    let params = value.get("params").unwrap_or(&Value::Null);
    match method {
        "Network.webSocketCreated" => {
            stats.websocket_created += 1;
            if let (Some(request_id), Some(url)) = (
                params.get("requestId").and_then(Value::as_str),
                params.get("url").and_then(Value::as_str),
            ) {
                request_urls.insert(request_id.to_string(), sanitize_url(url));
            }
        }
        "Network.webSocketFrameReceived" | "Network.webSocketFrameSent" => {
            if method.ends_with("Received") {
                stats.websocket_frames_received += 1;
            } else {
                stats.websocket_frames_sent += 1;
            }
            let payload = params
                .pointer("/response/payloadData")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let request_id = params
                .get("requestId")
                .and_then(Value::as_str)
                .unwrap_or("<unknown>");
            let source = request_urls
                .get(request_id)
                .cloned()
                .unwrap_or_else(|| "<unknown-ws>".to_string());
            maybe_push_candidate(
                stats,
                CandidateSummary::from_text(
                    if method.ends_with("Received") {
                        "ws_received"
                    } else {
                        "ws_sent"
                    },
                    source,
                    payload,
                    needles.symbol,
                    needles.qualified_symbol,
                    needles.expected_price,
                ),
            );
        }
        "Network.responseReceived" => {
            stats.response_count += 1;
            let response = params.get("response").unwrap_or(&Value::Null);
            let url = response
                .get("url")
                .and_then(Value::as_str)
                .map(sanitize_url)
                .unwrap_or_else(|| "<unknown-response>".to_string());
            let resource_type = params.get("type").and_then(Value::as_str).unwrap_or("-");
            maybe_push_candidate(
                stats,
                CandidateSummary::from_text(
                    "response",
                    format!("{} type={}", url, resource_type),
                    &url,
                    needles.symbol,
                    needles.qualified_symbol,
                    needles.expected_price,
                ),
            );
        }
        _ => {}
    }
}

fn maybe_push_candidate(stats: &mut NetworkStats, candidate: CandidateSummary) {
    if stats.candidates.len() >= 20 {
        return;
    }
    if candidate.contains_symbol
        || candidate.contains_qualified_symbol
        || candidate.contains_expected_price
        || candidate.contains_after_token
    {
        stats.candidates.push(candidate);
    }
}

impl CandidateSummary {
    fn from_text(
        kind: &'static str,
        source: String,
        text: &str,
        symbol: &str,
        qualified_symbol: &str,
        expected_price: Option<&str>,
    ) -> Self {
        let lower = text.to_lowercase();
        Self {
            kind,
            source,
            byte_len: text.len(),
            contains_symbol: !symbol.is_empty() && text.contains(symbol),
            contains_qualified_symbol: !qualified_symbol.is_empty()
                && text.contains(qualified_symbol),
            contains_expected_price: expected_price
                .map(|expected| !expected.is_empty() && text.contains(expected))
                .unwrap_or(false),
            contains_after_token: contains_after_token(&lower) || contains_after_token(text),
            decimal_token_count: decimal_token_count(text),
        }
    }
}

fn visible_panel_expression(symbol: &str, expected_price: Option<&str>) -> String {
    let symbol_json = serde_json::to_string(symbol).expect("symbol serializes");
    let expected_json = serde_json::to_string(&expected_price).expect("expected price serializes");
    format!(
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
  function decimalTokens(text) {{
    const matches = String(text || "").match(/\b\d{{1,4}}(?:,\d{{3}})*(?:\.\d+)?\b/g) || [];
    return Array.from(new Set(matches));
  }}
  const nodes = Array.from(document.querySelectorAll("aside, section, div, span"));
  const rightTexts = [];
  for (const el of nodes) {{
    if (!visible(el)) continue;
    const rect = el.getBoundingClientRect();
    if (rect.left < window.innerWidth * 0.58) continue;
    const text = normalizedText(el);
    if (!text || text.length > 280) continue;
    if (text.includes(symbol) || /アフター|マーケット|After|Market|Post|USD|最終更新/i.test(text)) {{
      if (!rightTexts.includes(text)) rightTexts.push(text);
    }}
  }}
  const panelText = rightTexts.join("\n");
  const afterRegex = /アフター\s*マーケット|after[-\s]?market|post[-\s]?market/i;
  const afterIndex = afterRegex.exec(panelText)?.index ?? -1;
  const beforeAfterLabel = afterIndex >= 0 ? panelText.slice(Math.max(0, afterIndex - 220), afterIndex) : "";
  const afterSnippet = rightTexts.find(text => afterRegex.test(text) && /\d{{1,4}}(?:,\d{{3}})*\.\d+\s*USD/.test(text)) || "";
  const snippetUsdMatches = Array.from(afterSnippet.matchAll(/(\d{{1,4}}(?:,\d{{3}})*\.\d+)\s*USD/g)).map(match => match[1]);
  const beforeUsdMatches = Array.from(beforeAfterLabel.matchAll(/(\d{{1,4}}(?:,\d{{3}})*\.\d+)\s*USD/g)).map(match => match[1]);
  const afterPrice = snippetUsdMatches.length
    ? snippetUsdMatches[snippetUsdMatches.length - 1]
    : (beforeUsdMatches.length ? beforeUsdMatches[beforeUsdMatches.length - 1] : null);
  const phaseText = rightTexts.find(text => /アフター|after[-\s]?market|post[-\s]?market/i.test(text)) || "";
  return {{
    symbol,
    symbol_seen: panelText.includes(symbol),
    after_market_label_seen: afterIndex >= 0,
    visible_after_market_price: afterPrice,
    expected_visible_price: expectedPrice,
    expected_visible_price_seen: expectedPrice ? panelText.includes(expectedPrice) : null,
    phase: /アフター|after|post/i.test(phaseText) ? "post-market" : null,
    numeric_candidates: decimalTokens(panelText).slice(0, 12),
  }};
}})()"#
    )
}

fn scanner_quote_summary(tv: &str, symbol: &str) -> String {
    let output = Command::new(tv)
        .args(["quote", symbol, "--source", "scanner"])
        .output()
        .expect("test-built tv binary should execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let value = serde_json::from_str::<Value>(&stdout).unwrap_or(Value::Null);
    let data = value.get("data").unwrap_or(&Value::Null);
    format!(
        "last={} close={} update_mode={} delay_seconds={} postmarket_close={}",
        display_value(data.get("last")),
        display_value(data.get("close")),
        data.get("update_mode")
            .and_then(Value::as_str)
            .unwrap_or("<missing>"),
        display_value(data.get("delay_seconds")),
        display_value(data.pointer("/extended_hours/postmarket/close")),
    )
}

fn network_summary(stats: &NetworkStats) -> String {
    format!(
        "events={} ws_created={} ws_received={} ws_sent={} responses={} candidates={}",
        stats.event_count,
        stats.websocket_created,
        stats.websocket_frames_received,
        stats.websocket_frames_sent,
        stats.response_count,
        stats
            .candidates
            .iter()
            .take(12)
            .map(candidate_summary)
            .collect::<Vec<_>>()
            .join("|")
    )
}

fn candidate_summary(candidate: &CandidateSummary) -> String {
    format!(
        "{}:{} bytes={} symbol={} qualified={} expected_price={} after_token={} decimals={}",
        candidate.kind,
        candidate.source,
        candidate.byte_len,
        candidate.contains_symbol,
        candidate.contains_qualified_symbol,
        candidate.contains_expected_price,
        candidate.contains_after_token,
        candidate.decimal_token_count
    )
}

fn visible_summary(value: &Value) -> String {
    format!(
        "symbol_seen={} after_label={} after_price={} expected_price={} expected_seen={} numbers={}",
        display_value(value.get("symbol_seen")),
        display_value(value.get("after_market_label_seen")),
        display_value(value.get("visible_after_market_price")),
        display_value(value.get("expected_visible_price")),
        display_value(value.get("expected_visible_price_seen")),
        display_array(value.get("numeric_candidates")),
    )
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

fn sanitize_url(raw: &str) -> String {
    reqwest::Url::parse(raw)
        .ok()
        .map(|url| {
            format!(
                "{}://{}{}",
                url.scheme(),
                url.host_str().unwrap_or("<host>"),
                url.path()
            )
        })
        .unwrap_or_else(|| "<unparseable-url>".to_string())
}

fn contains_after_token(text: &str) -> bool {
    text.contains("postmarket")
        || text.contains("post-market")
        || text.contains("aftermarket")
        || text.contains("after-market")
        || text.contains("アフター")
        || text.contains("マーケット")
}

fn decimal_token_count(text: &str) -> usize {
    text.split(|ch: char| !ch.is_ascii_digit() && ch != '.' && ch != ',')
        .filter(|token| {
            let token = token.trim_matches(',');
            token.contains('.') && token.chars().any(|ch| ch.is_ascii_digit())
        })
        .count()
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

fn env_string(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn env_duration_ms(name: &str, default_ms: u64) -> Duration {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0 && *value <= 60_000)
        .map(Duration::from_millis)
        .unwrap_or_else(|| Duration::from_millis(default_ms))
}

#[test]
fn sanitize_url_removes_query_and_fragment() {
    assert_eq!(
        sanitize_url("wss://example.test/socket.io/websocket?session=secret#frag"),
        "wss://example.test/socket.io/websocket"
    );
}

#[test]
fn candidate_detection_reports_tokens_without_raw_payload() {
    let candidate = CandidateSummary::from_text(
        "ws_received",
        "wss://example.test/socket".to_string(),
        "symbol=NASDAQ:RKLB price=110.17 phase=post-market",
        "RKLB",
        "NASDAQ:RKLB",
        Some("110.17"),
    );
    assert!(candidate.contains_symbol);
    assert!(candidate.contains_qualified_symbol);
    assert!(candidate.contains_expected_price);
    assert!(candidate.contains_after_token);
    assert_eq!(candidate.decimal_token_count, 1);
}

#[test]
fn phase_matching_accepts_tradingview_extended_hours_aliases() {
    assert!(phase_matches_expected("postmarket", "post-market"));
    assert!(phase_matches_expected("premarket", "pre-market"));
    assert!(!phase_matches_expected("postmarket", "regular"));
}
