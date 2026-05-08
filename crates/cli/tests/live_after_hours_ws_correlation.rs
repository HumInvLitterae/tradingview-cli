use std::{
    collections::HashMap,
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
const DEFAULT_CAPTURE_DURATION_MS: u64 = 30_000;
const DEFAULT_SAMPLE_INTERVAL_MS: u64 = 2_000;

#[derive(Debug, Clone, Deserialize)]
struct CdpTarget {
    id: String,
    #[serde(default)]
    url: String,
    #[serde(rename = "webSocketDebuggerUrl")]
    web_socket_debugger_url: Option<String>,
}

#[derive(Debug, Default)]
struct CorrelationStats {
    websocket_created: u64,
    websocket_frames_received: u64,
    websocket_frames_sent: u64,
    event_count: u64,
    candidates: Vec<FrameCandidate>,
}

#[derive(Debug, Clone)]
struct FrameCandidate {
    kind: &'static str,
    source: String,
    byte_len: usize,
    contains_symbol: bool,
    contains_qualified_symbol: bool,
    contains_expected_price: bool,
    contains_after_token: bool,
    numeric_tokens: Vec<String>,
}

#[derive(Debug, Clone)]
struct VisibleSample {
    elapsed_ms: u128,
    phase: Option<String>,
    after_price: Option<String>,
    regular_price: Option<String>,
    expected_seen: Option<bool>,
    numeric_tokens: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
struct ProbeNeedles<'a> {
    symbol: &'a str,
    qualified_symbol: &'a str,
    expected_price: Option<&'a str>,
}

#[tokio::test]
#[ignore = "requires a running TradingView Desktop CDP session and TV_LIVE_AFTER_HOURS_WS_CORRELATION_SMOKE=1"]
async fn after_hours_ws_correlation_live_smoke() {
    if std::env::var("TV_LIVE_AFTER_HOURS_WS_CORRELATION_SMOKE")
        .ok()
        .as_deref()
        != Some("1")
    {
        panic!(
            "live after-hours WebSocket correlation smoke is gated; set TV_LIVE_AFTER_HOURS_WS_CORRELATION_SMOKE=1 and run with --ignored"
        );
    }

    let symbol = env_string("TV_LIVE_AFTER_HOURS_SYMBOL").unwrap_or_else(|| DEFAULT_SYMBOL.into());
    let qualified_symbol = env_string("TV_LIVE_AFTER_HOURS_QUALIFIED_SYMBOL")
        .unwrap_or_else(|| DEFAULT_QUALIFIED_SYMBOL.into());
    let expected_phase = env_string("TV_LIVE_AFTER_HOURS_EXPECT_PHASE");
    let expected_visible_price = env_string("TV_LIVE_AFTER_HOURS_EXPECT_VISIBLE_PRICE");
    let duration = env_duration_ms(
        "TV_LIVE_AFTER_HOURS_WS_CORRELATION_DURATION_MS",
        DEFAULT_CAPTURE_DURATION_MS,
    );
    let sample_interval = env_duration_ms(
        "TV_LIVE_AFTER_HOURS_WS_CORRELATION_SAMPLE_MS",
        DEFAULT_SAMPLE_INTERVAL_MS,
    );
    let target = resolve_chart_target().await;

    println!(
        "after-hours WebSocket correlation smoke: symbol={} qualified={} target_id={} expected_phase={} expected_visible_price={} duration_ms={} sample_ms={}",
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
        sample_interval.as_millis(),
    );

    let mut stream = connect_target(&target).await;
    let mut next_id = 1_u64;
    let mut request_urls = HashMap::new();
    let mut stats = CorrelationStats::default();
    let needles = ProbeNeedles {
        symbol: &symbol,
        qualified_symbol: &qualified_symbol,
        expected_price: expected_visible_price.as_deref(),
    };

    call_cdp(
        &mut stream,
        &mut next_id,
        "Network.enable",
        json!({}),
        &mut request_urls,
        &mut stats,
        needles,
    )
    .await
    .expect("Network.enable should succeed");

    let started = Instant::now();
    let mut samples = vec![
        evaluate_visible_panel(
            &mut stream,
            &mut next_id,
            &mut request_urls,
            &mut stats,
            needles,
            started.elapsed().as_millis(),
        )
        .await,
    ];
    capture_with_visible_sampling(
        &mut stream,
        &mut next_id,
        duration,
        sample_interval,
        &mut request_urls,
        &mut stats,
        needles,
        &mut samples,
        started,
    )
    .await;
    samples.push(
        evaluate_visible_panel(
            &mut stream,
            &mut next_id,
            &mut request_urls,
            &mut stats,
            needles,
            started.elapsed().as_millis(),
        )
        .await,
    );

    if let Some(expected) = expected_visible_price.as_deref()
        && !samples.iter().any(|sample| {
            sample.expected_seen == Some(true) || sample.after_price.as_deref() == Some(expected)
        })
    {
        panic!(
            "visible panel did not contain expected price {}: samples={} network={}",
            expected,
            sample_summary(&samples),
            network_summary(&stats, &samples)
        );
    }

    if let Some(expected_phase) = expected_phase.as_deref() {
        let observed = samples.iter().find_map(|sample| sample.phase.as_deref());
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
        "ok visible_samples={} network={} elapsed_ms={}",
        sample_summary(&samples),
        network_summary(&stats, &samples),
        started.elapsed().as_millis()
    );
}

async fn connect_target(target: &CdpTarget) -> WebSocketStream<MaybeTlsStream<TcpStream>> {
    let (stream, _) = connect_async(
        target
            .web_socket_debugger_url
            .as_deref()
            .expect("selected target should include a webSocketDebuggerUrl"),
    )
    .await
    .expect("CDP websocket should connect");
    stream
}

async fn resolve_chart_target() -> CdpTarget {
    let targets = fetch_targets().await;
    if let Some(target_id) = env_string("TV_LIVE_AFTER_HOURS_TARGET_ID") {
        return targets
            .into_iter()
            .find(|target| target.id == target_id)
            .unwrap_or_else(|| panic!("provided target id was not found in CDP target list"));
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
    stats: &mut CorrelationStats,
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
    stats: &mut CorrelationStats,
    needles: ProbeNeedles<'_>,
    elapsed_ms: u128,
) -> VisibleSample {
    let expression = minimal_visible_panel_expression(needles.symbol, needles.expected_price);
    let value = call_cdp(
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
    .unwrap_or(Value::Null);
    VisibleSample {
        elapsed_ms,
        phase: value
            .get("phase")
            .and_then(Value::as_str)
            .map(str::to_string),
        after_price: value
            .get("visible_after_market_price")
            .and_then(Value::as_str)
            .map(str::to_string),
        regular_price: value
            .get("visible_regular_price")
            .and_then(Value::as_str)
            .map(str::to_string),
        expected_seen: value
            .get("expected_visible_price_seen")
            .and_then(Value::as_bool),
        numeric_tokens: value
            .get("numeric_candidates")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
    }
}

#[allow(clippy::too_many_arguments)]
async fn capture_with_visible_sampling(
    stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    next_id: &mut u64,
    duration: Duration,
    sample_interval: Duration,
    request_urls: &mut HashMap<String, String>,
    stats: &mut CorrelationStats,
    needles: ProbeNeedles<'_>,
    samples: &mut Vec<VisibleSample>,
    started: Instant,
) {
    let deadline = Instant::now() + duration;
    let mut next_sample = Instant::now() + sample_interval;
    while let Some(remaining) = deadline.checked_duration_since(Instant::now()) {
        if Instant::now() >= next_sample {
            samples.push(
                evaluate_visible_panel(
                    stream,
                    next_id,
                    request_urls,
                    stats,
                    needles,
                    started.elapsed().as_millis(),
                )
                .await,
            );
            next_sample = Instant::now() + sample_interval;
            continue;
        }
        let wait = remaining.min(
            next_sample
                .checked_duration_since(Instant::now())
                .unwrap_or_else(|| Duration::from_millis(1)),
        );
        let message = tokio::time::timeout(wait, stream.next()).await;
        let Ok(Some(Ok(message))) = message else {
            continue;
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
    stats: &mut CorrelationStats,
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
                FrameCandidate::from_text(
                    if method.ends_with("Received") {
                        "ws_received"
                    } else {
                        "ws_sent"
                    },
                    source,
                    payload,
                    needles,
                ),
            );
        }
        _ => {}
    }
}

fn maybe_push_candidate(stats: &mut CorrelationStats, candidate: FrameCandidate) {
    if stats.candidates.len() >= 48 {
        return;
    }
    if candidate.contains_symbol
        || candidate.contains_qualified_symbol
        || candidate.contains_expected_price
        || candidate.contains_after_token
        || !candidate.numeric_tokens.is_empty()
    {
        stats.candidates.push(candidate);
    }
}

impl FrameCandidate {
    fn from_text(
        kind: &'static str,
        source: String,
        text: &str,
        needles: ProbeNeedles<'_>,
    ) -> Self {
        let lower = text.to_lowercase();
        Self {
            kind,
            source,
            byte_len: text.len(),
            contains_symbol: !needles.symbol.is_empty() && text.contains(needles.symbol),
            contains_qualified_symbol: !needles.qualified_symbol.is_empty()
                && text.contains(needles.qualified_symbol),
            contains_expected_price: needles
                .expected_price
                .map(|expected| !expected.is_empty() && text.contains(expected))
                .unwrap_or(false),
            contains_after_token: contains_after_token(&lower) || contains_after_token(text),
            numeric_tokens: decimal_tokens(text).into_iter().take(8).collect(),
        }
    }
}

fn minimal_visible_panel_expression(symbol: &str, expected_price: Option<&str>) -> String {
    let symbol_json = serde_json::to_string(symbol).expect("symbol serializes");
    let expected_json = serde_json::to_string(&expected_price).expect("expected price serializes");
    format!(
        r#"(() => {{
  const symbol = {symbol_json};
  const expectedPrice = {expected_json};
  function textOf(el) {{
    return String(el && (el.innerText || el.textContent) || "").replace(/\s+/g, " ").trim();
  }}
  function decimalTokens(text) {{
    const matches = String(text || "").match(/\b\d{{1,4}}(?:,\d{{3}})*(?:\.\d+)?\b/g) || [];
    return Array.from(new Set(matches));
  }}
  const detail = document.querySelector('[data-test-id-widget-type="detail"]');
  const text = textOf(detail);
  const afterRegex = /アフター\s*マーケット|after[-\s]?market|post[-\s]?market/i;
  const afterIndex = afterRegex.exec(text)?.index ?? -1;
  const beforeAfter = afterIndex >= 0 ? text.slice(Math.max(0, afterIndex - 180), afterIndex) : "";
  const beforeUsd = Array.from(beforeAfter.matchAll(/(\d{{1,4}}(?:,\d{{3}})*\.\d+)\s*USD/g)).map(match => match[1]);
  const regular = Array.from(text.matchAll(/(\d{{1,4}}(?:,\d{{3}})*\.\d+)\s*USD/g)).map(match => match[1])[0] || null;
  return {{
    symbol,
    detail_found: Boolean(detail),
    symbol_seen: text.includes(symbol),
    phase: afterIndex >= 0 ? "post-market" : null,
    visible_regular_price: regular,
    visible_after_market_price: beforeUsd.length ? beforeUsd[beforeUsd.length - 1] : null,
    expected_visible_price: expectedPrice,
    expected_visible_price_seen: expectedPrice ? text.includes(expectedPrice) : null,
    numeric_candidates: decimalTokens(text).slice(0, 16),
  }};
}})()"#
    )
}

fn network_summary(stats: &CorrelationStats, samples: &[VisibleSample]) -> String {
    let visible_prices = visible_after_prices(samples);
    let exact_matches = exact_candidate_matches(stats, &visible_prices);
    format!(
        "events={} ws_created={} ws_received={} ws_sent={} candidates={} visible_prices={} exact_matches={} top={}",
        stats.event_count,
        stats.websocket_created,
        stats.websocket_frames_received,
        stats.websocket_frames_sent,
        stats.candidates.len(),
        visible_prices.join(","),
        exact_matches.join(","),
        stats
            .candidates
            .iter()
            .take(10)
            .map(candidate_summary)
            .collect::<Vec<_>>()
            .join("|")
    )
}

fn exact_candidate_matches(stats: &CorrelationStats, visible_prices: &[String]) -> Vec<String> {
    let mut matches = vec![];
    for candidate in &stats.candidates {
        for price in visible_prices {
            if candidate.numeric_tokens.iter().any(|token| token == price) {
                let item = format!("{}:{}", candidate.kind, price);
                if !matches.contains(&item) {
                    matches.push(item);
                }
            }
        }
    }
    matches
}

fn candidate_summary(candidate: &FrameCandidate) -> String {
    format!(
        "{}:{} bytes={} symbol={} qualified={} expected_price={} after_token={} nums={}",
        candidate.kind,
        candidate.source,
        candidate.byte_len,
        candidate.contains_symbol,
        candidate.contains_qualified_symbol,
        candidate.contains_expected_price,
        candidate.contains_after_token,
        candidate.numeric_tokens.join(",")
    )
}

fn sample_summary(samples: &[VisibleSample]) -> String {
    samples
        .iter()
        .take(20)
        .map(|sample| {
            format!(
                "{}ms:phase={} regular={} after={} expected_seen={} nums={}",
                sample.elapsed_ms,
                sample.phase.as_deref().unwrap_or("<none>"),
                sample.regular_price.as_deref().unwrap_or("<none>"),
                sample.after_price.as_deref().unwrap_or("<none>"),
                sample
                    .expected_seen
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "null".to_string()),
                sample
                    .numeric_tokens
                    .iter()
                    .take(6)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(",")
            )
        })
        .collect::<Vec<_>>()
        .join("|")
}

fn visible_after_prices(samples: &[VisibleSample]) -> Vec<String> {
    let mut values = vec![];
    for sample in samples {
        if let Some(value) = sample.after_price.as_ref()
            && !values.contains(value)
        {
            values.push(value.clone());
        }
    }
    values
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

fn decimal_tokens(text: &str) -> Vec<String> {
    let mut tokens = vec![];
    for token in text.split(|ch: char| !ch.is_ascii_digit() && ch != '.' && ch != ',') {
        let token = token.trim_matches(',');
        if token.contains('.') && token.chars().any(|ch| ch.is_ascii_digit()) {
            let normalized = token.replace(',', "");
            if !tokens.contains(&normalized) {
                tokens.push(normalized);
            }
        }
    }
    tokens
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
        .filter(|value| *value > 0 && *value <= 120_000)
        .map(Duration::from_millis)
        .unwrap_or_else(|| Duration::from_millis(default_ms))
}

#[test]
fn decimal_tokens_extracts_public_safe_values() {
    assert_eq!(
        decimal_tokens("RKLB 105.47 after 105.67 volume 1,234.50"),
        vec!["105.47", "105.67", "1234.50"]
    );
}

#[test]
fn exact_candidate_matches_visible_prices() {
    let stats = CorrelationStats {
        candidates: vec![FrameCandidate::from_text(
            "ws_received",
            "wss://example.test/socket".to_string(),
            "price=105.67 symbol=RKLB",
            ProbeNeedles {
                symbol: "RKLB",
                qualified_symbol: "NASDAQ:RKLB",
                expected_price: Some("105.67"),
            },
        )],
        ..Default::default()
    };
    assert_eq!(
        exact_candidate_matches(&stats, &["105.67".to_string()]),
        vec!["ws_received:105.67"]
    );
}

#[test]
fn phase_matching_accepts_hyphenated_aliases() {
    assert!(phase_matches_expected("postmarket", "post-market"));
    assert!(phase_matches_expected("premarket", "pre-market"));
    assert!(!phase_matches_expected("postmarket", "regular"));
}
