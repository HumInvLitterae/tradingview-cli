//! Loopback-only synthetic OAuth/MCP server. No provider or OS store access.

use crate::{
    Failure,
    admission::Admission,
    auth::Auth,
    budget::{Budget, RequestClass},
    credentials::Store,
    http::{Endpoints, Http},
    proof,
};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    task::JoinHandle,
    time::{Instant, sleep},
};

#[derive(Clone)]
struct Request {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}
struct Response {
    status: u16,
    body: Vec<u8>,
    mime: &'static str,
    extra: String,
    delay: Duration,
}

impl Response {
    fn json(value: Value) -> Self {
        Self {
            status: 200,
            body: serde_json::to_vec(&value).unwrap(),
            mime: "application/json",
            extra: String::new(),
            delay: Duration::ZERO,
        }
    }

    fn status(status: u16) -> Self {
        Self {
            status,
            ..Self::json(json!({"error": "synthetic-provider-error"}))
        }
    }
}
struct Server {
    endpoints: Endpoints,
    requests: Arc<Mutex<Vec<Request>>>,
    task: JoinHandle<()>,
}

impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl Server {
    async fn start(mode: &'static str) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let endpoints = Endpoints {
            resource: format!("{base}/mcp"),
            issuer: base.clone(),
            resource_metadata: format!("{base}/resource"),
            authorization_metadata: format!("{base}/metadata"),
            authorize: format!("{base}/authorize"),
            token: format!("{base}/token"),
            register: format!("{base}/register"),
        };
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = requests.clone();
        let routes = endpoints.clone();
        let task = tokio::spawn(async move {
            let mut children = tokio::task::JoinSet::new();
            loop {
                let (mut socket, _) = listener.accept().await.unwrap();
                let captured = captured.clone();
                let routes = routes.clone();
                children.spawn(async move {
                    let mut bytes = Vec::new();
                    loop {
                        let Ok(byte) = socket.read_u8().await else {
                            return;
                        };
                        bytes.push(byte);
                        if bytes.ends_with(b"\r\n\r\n") {
                            break;
                        }
                        if bytes.len() > 16384 {
                            return;
                        }
                    }

                    let head = String::from_utf8(bytes).unwrap();
                    let mut lines = head.split("\r\n");
                    let mut first = lines.next().unwrap().split_whitespace();
                    let method = first.next().unwrap().to_owned();
                    let path = first.next().unwrap().to_owned();
                    let headers: HashMap<_, _> = lines
                        .filter_map(|line| line.split_once(':'))
                        .map(|(k, v)| (k.to_ascii_lowercase(), v.trim().to_owned()))
                        .collect();
                    let len = headers
                        .get("content-length")
                        .map(|v| v.parse::<usize>().unwrap())
                        .unwrap_or(0);
                    if len > 65536 {
                        return;
                    }
                    let mut body = vec![0; len];
                    if socket.read_exact(&mut body).await.is_err() {
                        return;
                    }

                    let request = Request {
                        method,
                        path,
                        headers,
                        body,
                    };
                    captured.lock().unwrap().push(request.clone());
                    let response = respond(&routes, &request, mode);
                    sleep(response.delay).await;
                    let head = format!(
                        "HTTP/1.1 {} Fixture\r\n\
                         Content-Type: {}\r\n\
                         Content-Length: {}\r\n\
                         Connection: close\r\n{}\r\n",
                        response.status,
                        response.mime,
                        response.body.len(),
                        response.extra
                    );
                    let _ = socket.write_all(head.as_bytes()).await;
                    let _ = socket.write_all(&response.body).await;
                });
            }
        });
        Self {
            endpoints,
            requests,
            task,
        }
    }

    fn calls(&self, method: &str) -> usize {
        self.requests
            .lock()
            .unwrap()
            .iter()
            .filter(|r| {
                serde_json::from_slice::<Value>(&r.body)
                    .ok()
                    .and_then(|v| v.get("method").cloned())
                    .and_then(|v| v.as_str().map(str::to_owned))
                    .as_deref()
                    == Some(method)
            })
            .count()
    }
}

fn respond(ep: &Endpoints, r: &Request, mode: &str) -> Response {
    match r.path.as_str() {
        "/resource" => {
            return Response::json(json!({
                "resource": if mode == "bad-resource" {
                    "https://example.invalid/mcp"
                } else {
                    &ep.resource
                },
                "authorization_servers": [ep.issuer]
            }));
        }
        "/metadata" => {
            return Response::json(json!({
                "issuer": ep.issuer,
                "authorization_endpoint": ep.authorize,
                "token_endpoint": ep.token,
                "registration_endpoint": ep.register,
                "response_types_supported": ["code"],
                "code_challenge_methods_supported": ["S256"],
                "token_endpoint_auth_methods_supported": ["none"],
                "scopes_supported": ["mcp:read", "mcp:tools"]
            }));
        }
        "/register" => {
            let request: Value = serde_json::from_slice(&r.body).unwrap();
            return Response::json(
                json!({"client_id": "synthetic-client", "redirect_uris": request["redirect_uris"]}),
            );
        }
        "/token" => {
            if mode == "token-redirect" {
                let mut reply = Response::status(302);
                reply.extra = "Location: https://example.invalid/steal\r\n".into();
                return reply;
            }
            let fields: HashMap<_, _> = reqwest::Url::parse(&format!(
                "http://example.invalid/?{}",
                String::from_utf8_lossy(&r.body)
            ))
            .unwrap()
            .query_pairs()
            .into_owned()
            .collect();
            assert_eq!(fields.get("resource"), Some(&ep.resource));
            let refresh = fields.get("grant_type").map(String::as_str) == Some("refresh_token");
            if !refresh {
                assert!(fields.get("code_verifier").is_some_and(|v| v.len() >= 43));
            }
            let scope = if mode == "broad-grant" {
                "mcp:tools"
            } else {
                "mcp:read"
            };
            return Response::json(json!({
                "access_token": if refresh { "synthetic-new-access" } else { "synthetic-access" },
                "token_type": "Bearer",
                "refresh_token": if refresh { "synthetic-new-refresh" } else { "synthetic-refresh" },
                "expires_in": 3600,
                "scope": scope
            }));
        }
        _ => {}
    }
    if r.method == "GET" || r.method == "DELETE" {
        return Response::status(405);
    }
    let request: Value = serde_json::from_slice(&r.body).unwrap();
    let id = &request["id"];
    let result = match request["method"].as_str().unwrap() {
        "initialize" => {
            json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "synthetic-server", "version": "1"}
            })
        }
        "notifications/initialized" | "notifications/cancelled" => return Response::status(202),
        "tools/list" => {
            let mut schema = json!({
                "type": "object",
                "properties": {
                    "symbol": {"type": "string"},
                    "interval": {"type": "string"},
                    "count": {"type": "integer"},
                    "summary": {"type": "boolean"}
                },
                "required": ["symbol"]
            });
            if mode == "schema-change" {
                schema["required"] = json!(["symbol", "new_required"]);
            }
            if mode == "numeric-nullable-schema" {
                schema["properties"]["count"]["type"] = json!("number");
                schema["properties"]["interval"] =
                    json!({"anyOf": [{"type": "string"}, {"type": "null"}]});
            }
            json!({
                "tools": [{
                    "name": if mode == "numeric-nullable-schema" {
                        "mcp-tv-get-ohlcv"
                    } else {
                        "get_ohlcv"
                    },
                    "inputSchema": schema
                }]
            })
        }
        "tools/call" => {
            assert_eq!(
                r.headers.get("authorization").map(String::as_str),
                Some("Bearer synthetic-access")
            );
            let status = match mode {
                "401" => 401,
                "429" => 429,
                "500" => 500,
                "404" => 404,
                _ => 200,
            };
            if status != 200 {
                let mut reply = Response::status(status);
                if status == 429 {
                    reply.extra = "Retry-After: 123\r\n".into();
                }
                return reply;
            }
            if mode == "malformed" {
                let mut reply = Response::json(json!({}));
                reply.body = b"synthetic-secret-invalid-json".to_vec();
                return reply;
            }
            if mode == "oversized" {
                let mut reply = Response::json(json!({}));
                reply.body = vec![b' '; 8 * 1024 * 1024 + 1];
                return reply;
            }
            if mode == "redirect" {
                let mut reply = Response::status(302);
                reply.extra = format!("Location: {}/unexpected\r\n", ep.issuer);
                return reply;
            }
            if mode == "mrtr" {
                return Response::json(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "resultType": "input_required",
                        "inputRequests": [],
                        "requestState": "synthetic"
                    }
                }));
            }
            let result = json!({
                "content": [],
                "structuredContent": {
                    "bars": [{
                        "t": 1704153600,
                        "o": 1.0,
                        "h": 3.0,
                        "l": 0.5,
                        "c": 2.0,
                        "v": null
                    }]
                },
                "isError": mode == "tool-error"
            });
            let rpc = json!({"jsonrpc": "2.0", "id": id, "result": result});
            if mode == "sse" {
                let mut reply = Response::json(json!({}));
                reply.mime = "text/event-stream";
                reply.body = format!("data: {rpc}\n\n").into_bytes();
                return reply;
            }
            if mode == "stall" {
                let mut reply = Response::json(rpc);
                reply.delay = Duration::from_secs(5);
                return reply;
            }
            result
        }
        _ => panic!("unexpected synthetic method"),
    };
    let mut response = Response::json(json!({"jsonrpc": "2.0", "id": id, "result": result}));
    if mode == "session-json" && request["method"] == "initialize" {
        response.extra = "Mcp-Session-Id: synthetic-session\r\n".into();
    }
    response
}

async fn context(
    server: &Server,
    seconds: u64,
) -> (
    tempfile::TempDir,
    Admission,
    Arc<Mutex<Budget>>,
    Http,
    Store,
) {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("state");
    let deadline = Instant::now() + Duration::from_secs(seconds);
    let guard = Admission::acquire(&dir, deadline).await.unwrap();
    let budget = Arc::new(Mutex::new(Budget::open(&dir, true).unwrap()));
    let http = Http::new(server.endpoints.clone(), deadline, budget.clone()).unwrap();
    let store = Store::memory(server.endpoints.clone(), deadline);
    (root, guard, budget, http, store)
}

#[tokio::test]
async fn three_timeframes_share_discovery_and_never_repeat_a_tool_request() {
    let server = Server::start("numeric-nullable-schema").await;
    let (_root, mut guard, budget, http, _store) = context(&server, 5).await;
    let report = proof::read_intervals(
        &http,
        "synthetic-access".into(),
        &["1D", "1W", "M"],
        Some(&mut guard),
    )
    .await
    .unwrap();
    assert_eq!(report["all_reads_completed"], true);
    assert_eq!(report["observations"].as_array().unwrap().len(), 3);
    assert_eq!(server.calls("initialize"), 1);
    assert_eq!(server.calls("tools/list"), 1);
    assert_eq!(server.calls("tools/call"), 3);
    assert!(
        server
            .requests
            .lock()
            .unwrap()
            .iter()
            .filter_map(|r| serde_json::from_slice::<Value>(&r.body).ok())
            .filter(|v| v["method"] == "tools/call")
            .all(|v| v["params"]["name"] == "mcp-tv-get-ohlcv")
    );
    assert_eq!(budget.lock().unwrap().counts().tools, 3);
    assert_eq!(
        guard
            .before_tool(Instant::now() + Duration::from_millis(20))
            .await
            .unwrap_err(),
        Failure::Timeout
    );

    let (_root, _guard, _budget, http, _store) = context(&server, 5).await;
    let report = proof::read_intervals(&http, "synthetic-access".into(), &["1D", "1D"], None)
        .await
        .unwrap();
    assert_eq!(report["all_reads_completed"], false);
    assert_eq!(report["failure"], "budget_exhausted");
    assert_eq!(report["observations"].as_array().unwrap().len(), 1);
    assert_eq!(server.calls("tools/call"), 4);
}

#[tokio::test]
async fn sdk_json_and_sse_success_preserve_partiality_without_raw_payload_output() {
    for mode in ["json", "sse", "session-json"] {
        let server = Server::start(mode).await;
        let (_root, _guard, budget, http, _store) = context(&server, 3).await;
        let result = proof::read(&http, "synthetic-access".into(), "1D")
            .await
            .unwrap();
        assert_eq!(result["bar_count"], 1);
        assert_eq!(result["count_status"], "short");
        assert_eq!(result["missing_volume_count"], 1);
        assert!(!result.to_string().contains("synthetic-access"));
        assert_eq!(server.calls("tools/call"), 1);
        assert_eq!(budget.lock().unwrap().counts().tools, 1);
        assert_eq!(
            budget.lock().unwrap().counts().protocol as usize,
            server.requests.lock().unwrap().len()
        );
    }
}

#[tokio::test]
async fn real_sdk_transport_does_not_replay_faults_or_reinitialize_sessions() {
    for (mode, expected) in [
        ("401", Failure::AuthRequired),
        ("429", Failure::RateLimited),
        ("500", Failure::ProviderError),
        ("404", Failure::ProviderError),
        ("malformed", Failure::InvalidResponse),
        ("redirect", Failure::ProviderError),
        ("oversized", Failure::ResponseTooLarge),
        ("tool-error", Failure::ProviderError),
    ] {
        let server = Server::start(mode).await;
        let (_root, _guard, _budget, http, _store) = context(&server, 3).await;
        let error = proof::read(&http, "synthetic-access".into(), "1D")
            .await
            .unwrap_err();
        assert_eq!(error, expected, "fixture {mode}");
        assert_eq!(server.calls("tools/call"), 1);
        assert_eq!(server.calls("initialize"), 1);
        assert!(!format!("{error:?}").contains("synthetic-secret"));
        assert!(
            !server
                .requests
                .lock()
                .unwrap()
                .iter()
                .any(|r| r.path == "/unexpected")
        );
        if mode == "429" {
            assert_eq!(http.cooldown(), Some(123));
        }
    }
}

#[tokio::test]
async fn deadline_stops_stalled_response_and_schema_drift_prevents_tool_dispatch() {
    for mode in ["stall", "schema-change"] {
        let server = Server::start(mode).await;
        let (_root, _guard, _budget, http, _store) = context(&server, 1).await;
        let result = proof::read(&http, "synthetic-access".into(), "1D").await;
        assert_eq!(
            result.unwrap_err(),
            if mode == "stall" {
                Failure::Timeout
            } else {
                Failure::SchemaChanged
            }
        );
        assert_eq!(
            server.calls("tools/call"),
            if mode == "stall" { 1 } else { 0 }
        );
    }
}

#[tokio::test]
async fn sdk_oauth_pkce_state_rotation_and_fresh_manager_restore() {
    let server = Server::start("json").await;
    let (_root, _guard, budget, http, store) = context(&server, 5).await;
    let mut auth = Auth::discover(http.clone(), store.clone(), budget.clone())
        .await
        .unwrap();
    let url = auth
        .register("http://127.0.0.1:12345/callback")
        .await
        .unwrap();
    let fields: HashMap<_, _> = reqwest::Url::parse(&url)
        .unwrap()
        .query_pairs()
        .into_owned()
        .collect();
    assert_eq!(fields["code_challenge_method"], "S256");
    assert_eq!(fields["scope"], "mcp:read");
    assert_eq!(fields["resource"], server.endpoints.resource);
    assert!(
        auth.exchange("synthetic-code", "wrong-state", None)
            .await
            .is_err()
    );
    assert!(
        auth.exchange(
            "synthetic-code",
            &fields["state"],
            Some("https://example.invalid")
        )
        .await
        .is_err()
    );
    assert_eq!(budget.lock().unwrap().counts().exchange, 0);
    auth.exchange("synthetic-code", &fields["state"], None)
        .await
        .unwrap();
    let mut restarted = Auth::discover(http, store.clone(), budget.clone())
        .await
        .unwrap();
    restarted.restore().await.unwrap();
    assert_eq!(restarted.token().await.unwrap(), "synthetic-access");
    restarted.refresh().await.unwrap();
    assert_eq!(restarted.token().await.unwrap(), "synthetic-new-access");
    let saved = serde_json::to_value(store.load_record().await.unwrap().unwrap()).unwrap();
    assert_eq!(
        saved["token_response"]["refresh_token"],
        "synthetic-new-refresh"
    );
    assert_eq!(budget.lock().unwrap().counts().refresh, 1);
    restarted.refresh().await.unwrap();
    assert_eq!(budget.lock().unwrap().counts().refresh, 2);
    budget.lock().unwrap().credential_continuity().unwrap();
}

#[tokio::test]
async fn oauth_origin_broader_grant_redirect_and_uncertain_refresh_fail_closed() {
    for mode in ["bad-resource", "broad-grant", "token-redirect"] {
        let server = Server::start(mode).await;
        let (_root, _guard, budget, http, store) = context(&server, 5).await;
        let discovered = Auth::discover(http, store.clone(), budget.clone()).await;
        if mode == "bad-resource" {
            assert!(matches!(discovered, Err(Failure::BindingMismatch)));
            continue;
        }
        let mut auth = discovered.unwrap();
        let url = auth
            .register("http://127.0.0.1:12345/callback")
            .await
            .unwrap();
        let state = reqwest::Url::parse(&url)
            .unwrap()
            .query_pairs()
            .find(|(k, _)| k == "state")
            .unwrap()
            .1
            .into_owned();
        assert!(auth.exchange("synthetic-code", &state, None).await.is_err());
        assert!(store.load_record().await.unwrap().is_none());
        assert!(budget.lock().unwrap().credential_continuity().is_err());
    }
    let server = Server::start("json").await;
    let (_root, _guard, budget, _http, _store) = context(&server, 5).await;
    budget
        .lock()
        .unwrap()
        .charge(RequestClass::Refresh)
        .unwrap();
    assert_eq!(
        budget.lock().unwrap().credential_continuity().unwrap_err(),
        Failure::AuthRequired
    );
}

#[tokio::test]
async fn successful_server_rotation_with_failed_save_requires_reauthorization() {
    let server = Server::start("json").await;
    let (_root, _guard, budget, http, store) = context(&server, 5).await;
    let mut auth = Auth::discover(http.clone(), store.clone(), budget.clone())
        .await
        .unwrap();
    let url = auth
        .register("http://127.0.0.1:12345/callback")
        .await
        .unwrap();
    let state = reqwest::Url::parse(&url)
        .unwrap()
        .query_pairs()
        .find(|(k, _)| k == "state")
        .unwrap()
        .1
        .into_owned();
    auth.exchange("synthetic-code", &state, None).await.unwrap();
    store.fail_saves();
    assert_eq!(
        auth.refresh().await.unwrap_err(),
        Failure::StorageUnavailable
    );
    let mut restarted = Auth::discover(http, store, budget.clone()).await.unwrap();
    assert_eq!(
        restarted.restore().await.unwrap_err(),
        Failure::AuthRequired
    );
    assert_eq!(budget.lock().unwrap().counts().refresh, 1);
}

#[tokio::test]
async fn rejected_tool_refreshes_once_without_replaying_and_prepares_next_invocation() {
    let server = Server::start("401").await;
    let (_root, mut guard, budget, http, store) = context(&server, 5).await;
    let mut auth = Auth::discover(http, store.clone(), budget.clone())
        .await
        .unwrap();
    let url = auth
        .register("http://127.0.0.1:12345/callback")
        .await
        .unwrap();
    let state = reqwest::Url::parse(&url)
        .unwrap()
        .query_pairs()
        .find(|(k, _)| k == "state")
        .unwrap()
        .1
        .into_owned();
    auth.exchange("synthetic-code", &state, None).await.unwrap();
    assert_eq!(
        proof::authenticated_read(&mut auth, &mut guard, "1D")
            .await
            .unwrap_err(),
        Failure::AuthRefreshedRetryRequired
    );
    assert_eq!(server.calls("tools/call"), 1);
    assert_eq!(budget.lock().unwrap().counts().refresh, 1);
    let saved = serde_json::to_value(store.load_record().await.unwrap().unwrap()).unwrap();
    assert_eq!(
        saved["token_response"]["access_token"],
        "synthetic-new-access"
    );
}

#[tokio::test]
async fn public_service_shapes_wire_data_and_preserves_typed_failures() {
    use crate::client::{Operation, execute};
    use tradingview_model::mcp_bars::Request as BarsRequest;
    for mode in [
        "json",
        "sse",
        "429",
        "401",
        "malformed",
        "tool-error",
        "schema-change",
    ] {
        let server = Server::start(mode).await;
        let (_root, mut guard, budget, http, store) = context(&server, 5).await;
        let mut auth = Auth::discover(http.clone(), store.clone(), budget.clone())
            .await
            .unwrap();
        let url = auth
            .register("http://127.0.0.1:12345/callback")
            .await
            .unwrap();
        let state = reqwest::Url::parse(&url)
            .unwrap()
            .query_pairs()
            .find(|(k, _)| k == "state")
            .unwrap()
            .1
            .into_owned();
        auth.exchange("synthetic-code", &state, None).await.unwrap();
        let result = execute(
            Operation::Bars(BarsRequest::new("NASDAQ:AAPL", "1D", 20).unwrap()),
            &mut guard,
            store,
            http,
            budget,
            true,
        )
        .await;
        if matches!(mode, "json" | "sse") {
            let data = result.unwrap();
            assert_eq!(data["contract_version"], "mcp_bars.v1");
            assert_eq!(data["client_observation"]["count_status"], "short");
            assert!(data["bars"][0]["volume"].is_null());
            assert_eq!(
                data["provider_observation"]["adjustment"]["evidence"],
                "unconfirmed"
            );
        } else {
            let error = result.unwrap_err();
            let details = error.details.unwrap();
            assert_eq!(details["contract_version"], "mcp_error.v1");
            assert_eq!(
                details["tool_attempts"],
                if mode == "schema-change" { 0 } else { 1 }
            );
            assert!(!details.to_string().contains("synthetic-access"));
            assert!(!error.message.contains("synthetic-provider"));
            if mode == "429" {
                assert_eq!(details["code"], "rate_limited");
                assert_eq!(details["retry_after_seconds"], 123);
                assert_eq!(details["retry_after_evidence"], "http_header");
            }
            if mode == "401" {
                assert_eq!(details["code"], "auth_refreshed_retry_required");
                assert_eq!(server.calls("tools/call"), 1);
            }
        }
    }
}
