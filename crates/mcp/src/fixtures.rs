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
                "refresh_token": if refresh {
                    "synthetic-new-refresh"
                } else {
                    "synthetic-refresh"
                },
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
    if (mode == "slow-initialize-headers" && request["method"] == "initialize")
        || (mode == "slow-catalog-headers" && request["method"] == "tools/list")
    {
        let mut response = Response::status(200);
        response.delay = Duration::from_secs(5);
        return response;
    }
    if mode.starts_with("alert-history-") {
        if request["method"] == "tools/list" {
            return Response::json(json!({"jsonrpc": "2.0", "id": id, "result": {"tools": [{
                "name": "get_alerts_log", "inputSchema": {
                    "type": "object", "properties": {
                        "symbol": {"type": "string"}, "days": {"type": "integer"},
                        "limit": {"type": "integer"}
                    }, "required": ["symbol"]
                }
            }]}}));
        }
        if request["method"] == "tools/call" {
            if mode == "alert-history-429" {
                return Response::status(429);
            }
            let mut value = json!({"success": true, "days": 7, "count": 1, "events": [{
                "tv_alert_id": 12, "fire_id": 34, "symbol": "NASDAQ:EXAMPLE",
                "fired_at": "2000-02-29T12:34:56Z", "message": "private event text",
                "webhook": null
            }]});
            match mode {
                "alert-history-empty" => {
                    value["events"] = json!([]);
                    value["count"] = json!(0);
                }
                "alert-history-mismatch" => value["events"][0]["symbol"] = json!("NYSE:OTHER"),
                "alert-history-invalid" => value["events"][0]["tv_alert_id"] = Value::Null,
                "alert-history-provider-error" => {
                    value = json!({"success": false, "error": "private"})
                }
                _ => {}
            }
            return Response::json(json!({"jsonrpc": "2.0", "id": id, "result": {
                "content": [], "structuredContent": value
            }}));
        }
    }
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
            let mut tools = vec![json!({
                "name": if mode == "numeric-nullable-schema" {
                    "mcp-tv-get-ohlcv"
                } else {
                    "get_ohlcv"
                },
                "inputSchema": schema
            })];
            for tool in [
                crate::tools::Tool::EconomicSymbols,
                crate::tools::Tool::EconomicData,
                crate::tools::Tool::EconomicCalendar,
                crate::tools::Tool::Dividends,
                crate::tools::Tool::News,
                crate::tools::Tool::Story,
                crate::tools::Tool::Documents,
                crate::tools::Tool::Document,
                crate::tools::Tool::Financials,
                crate::tools::Tool::FinancialHistory,
                crate::tools::Tool::Forecasts,
                crate::tools::Tool::Earnings,
                crate::tools::Tool::Search,
                crate::tools::Tool::Columns,
                crate::tools::Tool::Symbol,
                crate::tools::Tool::Symbols,
                crate::tools::Tool::Screener,
                crate::tools::Tool::Watchlists,
                crate::tools::Tool::Watchlist,
                crate::tools::Tool::Alerts,
                crate::tools::Tool::AlertDetails,
                crate::tools::Tool::CreateAlert,
                crate::tools::Tool::UpdateAlert,
                crate::tools::Tool::StopAlerts,
                crate::tools::Tool::RestartAlerts,
                crate::tools::Tool::DeleteAlerts,
                crate::tools::Tool::CreateWatchlist,
                crate::tools::Tool::UpdateWatchlist,
                crate::tools::Tool::AddWatchlist,
                crate::tools::Tool::RemoveWatchlist,
                crate::tools::Tool::DeleteWatchlist,
            ] {
                let properties: serde_json::Map<_, _> = tool
                    .fields()
                    .iter()
                    .map(|(name, kind)| {
                        (
                            (*name).into(),
                            if *kind == "array" {
                                json!({
                                    "type": ["null", "array"],
                                    "items": {
                                        "type": if matches!(
                                            tool,
                                            crate::tools::Tool::AlertDetails
                                                | crate::tools::Tool::StopAlerts
                                                | crate::tools::Tool::RestartAlerts
                                                | crate::tools::Tool::DeleteAlerts
                                        ) {
                                            "integer"
                                        } else {
                                            "string"
                                        }
                                    }
                                })
                            } else {
                                json!({"type": kind})
                            },
                        )
                    })
                    .collect();
                tools.push(json!({
                    "name": tool.names()[0],
                    "inputSchema": {
                        "type": "object", "properties": properties,
                        "required": if mode == "schema-change" {
                            vec!["new_required"]
                        } else {
                            vec![]
                        }
                    }
                }));
            }
            json!({"tools": tools})
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
            let mut result = json!({
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
            let args = &request["params"]["arguments"];
            match request["params"]["name"].as_str() {
                Some("mcp-tv-get-economic-symbols") => {
                    result["structuredContent"] = json!({
                        "success": true, "count": 1,
                        "symbols": [{
                            "symbol": "ECONOMICS:USEXAMPLE",
                            "description": "Synthetic indicator",
                            "category": "prce"
                        }]
                    });
                }
                Some("mcp-tv-get-economic-data") => {
                    result["structuredContent"] = json!({
                        "success": true, "symbol": args["symbol"], "count": 2,
                        "series": [
                            {"date": "2026-01-01", "value": 0},
                            {"date": "2026-02-01", "value": null}
                        ],
                        "unit": "%", "scale": 1
                    });
                }
                Some("mcp-tv-get-economic-calendar") => {
                    result["structuredContent"] = json!({
                        "status": "ok",
                        "result": [{
                            "id": "synthetic-event",
                            "date": "2026-01-01T12:00:00Z",
                            "actual": 0,
                            "forecast": null
                        }]
                    });
                }
                Some("mcp-tv-get-dividends-calendar") => {
                    result["structuredContent"] = json!({
                        "success": true,
                        "count": 1,
                        "data": [{
                            "symbol": "NASDAQ:EXAMPLE",
                            "dividend_amount_recent": 0,
                            "dividend_amount_upcoming": null
                        }]
                    });
                }
                Some("mcp-tv-get-news") => {
                    result["structuredContent"] = json!({"success": true, "data": {
                        "headlines": [{
                            "id": "urn:newsml:example:one",
                            "title": "Example",
                            "permission": "restricted",
                            "paywall": true
                        }],
                        "count": 1,
                        "offset": args["offset"],
                        "has_more": true,
                        "next_offset": 1,
                        "total_available": 2
                    }});
                }
                Some("mcp-tv-get-news-story") => {
                    result["structuredContent"] = json!({
                        "id": args["id"], "permission": "restricted", "ast_description": null
                    });
                }
                Some("mcp-tv-get-documents") => {
                    result["structuredContent"] = json!({"items": [{
                        "id": "synthetic-document", "title": "Example filing", "reported": 1000,
                        "views": [{"id": "opaque-view", "type": "summary"}]
                    }], "total": 1});
                }
                Some("mcp-tv-get-document-view") => {
                    result["structuredContent"] = json!({"id": args["view_id"], "astDescription": {
                        "type": "root",
                        "children": [{
                            "type": "paragraph",
                            "children": ["Synthetic text"]
                        }]
                    }});
                }
                Some("mcp-tv-get-financials") => {
                    result["structuredContent"] = json!({"success": true, "data": {
                        "name": "EXAMPLE", "total_revenue_ttm": 0, "net_income_ttm": null
                    }});
                }
                Some("mcp-tv-get-financial-history") => {
                    result["structuredContent"] = json!({
                        "success": true, "symbol": "NASDAQ:EXAMPLE", "period": args["period"],
                        "labels": ["FY2025 Q1"],
                        "series": {"revenue": [{"value": 0, "yoy_pct": null}]}
                    });
                }
                Some("mcp-tv-get-forecasts") => {
                    result["structuredContent"] = json!({"success": true, "data": {
                        "symbol": "NASDAQ:EXAMPLE", "currency": "EUR",
                        "analyst_rating": {"recommendation": "provider-opinion"},
                        "price_targets": {"average": null}, "estimates": {"eps_next_quarter": 0}
                    }});
                }
                Some("mcp-tv-get-earnings-calendar") => {
                    result["structuredContent"] = json!({"success": true, "data": {
                        "count": 1,
                        "earnings": [{
                            "symbol": "NASDAQ:EXAMPLE",
                            "release_date": "2026-05-01"
                        }]
                    }});
                }
                Some(
                    "mcp-tv-create-alert"
                    | "mcp-tv-update-alert"
                    | "mcp-tv-stop-alerts"
                    | "mcp-tv-restart-alerts"
                    | "mcp-tv-delete-alert",
                ) => {
                    result["structuredContent"] = if mode == "mutation-no-id" {
                        json!({"success": true})
                    } else {
                        json!({"success": true, "alert_id": 12})
                    };
                }
                Some(
                    "mcp-watchlist-create-watchlist"
                    | "mcp-watchlist-update-watchlist"
                    | "mcp-watchlist-add-to-watchlist"
                    | "mcp-watchlist-remove-from-watchlist"
                    | "mcp-watchlist-delete-watchlist",
                ) => {
                    result["structuredContent"] = if mode == "mutation-no-id" {
                        json!({"success": true})
                    } else {
                        json!({"success": true, "watchlist": {"id": 12}})
                    };
                }
                Some("mcp-watchlist-list-watchlists") => {
                    result["structuredContent"] = if mode == "watchlist-deleted" {
                        json!({"watchlists": []})
                    } else {
                        json!({
                            "watchlists": [{"id": 12, "name": "Example", "symbols": ["NASDAQ:EXAMPLE"]}]
                        })
                    };
                }
                Some("mcp-watchlist-get-watchlist") => {
                    if mode == "readback-429" {
                        let mut reply = Response::status(429);
                        reply.extra = "Retry-After: 123\r\n".into();
                        return reply;
                    }
                    result["structuredContent"] = json!({
                        "watchlist": {"id": 12, "name": "Example", "symbols": ["NASDAQ:EXAMPLE"]}
                    });
                }
                Some("mcp-tv-list-alerts" | "mcp-tv-get-alerts") => {
                    if mode == "readback-429" {
                        let mut reply = Response::status(429);
                        reply.extra = "Retry-After: 123\r\n".into();
                        return reply;
                    }
                    result["structuredContent"] = json!({
                        "success": true,
                        "alerts": [{"alert_id": 12, "symbol": "NASDAQ:EXAMPLE", "active": false}]
                    });
                    if mode == "alert-active" || mode == "alert-sse" {
                        result["structuredContent"] = json!({"success": true, "alerts": [{
                            "alert_id": 12, "symbol": "NASDAQ:EXAMPLE", "active": true,
                            "name": "Example", "condition_type": "greater", "threshold": 100.0,
                            "resolution": "1D", "auto_deactivate": false,
                            "email": false, "mobile_push": false, "popup": false
                        }]});
                    } else if mode == "alert-deleted" {
                        result["structuredContent"] = json!({"alerts": []});
                    }
                }
                Some("mcp-tv-search-symbols") => {
                    result["structuredContent"] = json!({
                        "data": {
                            "count": 1,
                            "symbols": [{
                                "symbol": "NASDAQ:EXAMPLE",
                                "description": "Example Corp",
                                "type": "stock",
                                "exchange": "NASDAQ"
                            }]
                        }
                    })
                }
                Some("mcp-tv-get-screener-columns") => {
                    result["structuredContent"] =
                        if args.get("search").is_none() && args.get("group").is_none() {
                            json!({
                                "count": 1,
                                "groups": [{"group": "market", "count": 1, "columns": ["volume"]}]
                            })
                        } else {
                            json!({
                                "count": 1,
                                "columns": [{
                                    "name": "volume",
                                    "description": "Volume",
                                    "group": "market",
                                    "markets": ["stock"],
                                    "variants": ["volume|5"]
                                }]
                            })
                        };
                }
                Some("mcp-tv-run-screener") => {
                    result["structuredContent"] = json!({
                        "success": true,
                        "data": {
                            "rows": [{"symbol": "NASDAQ:EXAMPLE", "close": 0, "volume": null}],
                            "totalCount": 10
                        }
                    });
                }
                Some("mcp-tv-get-symbol-data-batch") => {
                    let symbols = args["symbols"].as_array().unwrap();
                    let mut rows = serde_json::Map::new();
                    rows.insert(
                        symbols[0].as_str().unwrap().into(),
                        json!({"close": 0, "volume": null}),
                    );
                    result["structuredContent"] = json!({
                        "success": true,
                        "count": 1,
                        "data": rows,
                        "missing": [{"symbol": symbols[1], "reason": "synthetic unavailable"}],
                        "missing_count": 1
                    });
                }
                Some("mcp-tv-get-symbol-data") => {
                    result["structuredContent"] = json!({
                        "symbol": args["symbol"], "data": {"close": 10.0, "volume": null}
                    })
                }
                _ => {}
            }
            if mode == "application-429" {
                result["structuredContent"] = json!({
                    "success": false,
                    "error": "Synthetic upstream status 429; synthetic-secret"
                });
            }
            let rpc = json!({"jsonrpc": "2.0", "id": id, "result": result});
            if mode == "sse" || mode == "alert-sse" {
                let mut reply = Response::json(json!({}));
                reply.mime = "text/event-stream";
                reply.body = format!("data: {rpc}\n\n").into_bytes();
                return reply;
            }
            if mode == "stall" {
                let mut reply = Response::json(rpc);
                reply.delay = Duration::from_secs(60);
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
    let guard = Admission::acquire(&dir, Instant::now() + Duration::from_secs(10))
        .await
        .unwrap();
    // Response/deadline tests start their clock after local directory setup.
    let deadline = Instant::now() + Duration::from_secs(seconds);
    let budget = Arc::new(Mutex::new(Budget::open(&dir, true).unwrap()));
    let http = Http::new(server.endpoints.clone(), deadline, budget.clone()).unwrap();
    let store = Store::memory(server.endpoints.clone(), deadline);
    (root, guard, budget, http, store)
}

#[tokio::test]
async fn protocol_diagnostics_separate_initialization_and_catalog_timeouts() {
    for (mode, waiting_method) in [
        ("slow-initialize-headers", "initialize"),
        ("slow-catalog-headers", "tools/list"),
    ] {
        let server = Server::start(mode).await;
        let (_root, _guard, _budget, http, _store) = context(&server, 1).await;
        let outcome = crate::transport::inspect_tools(
            &http,
            "synthetic-secret-token".into(),
            &[(
                crate::tools::Tool::Bars,
                tradingview_model::mcp_bars::Request::new("NASDAQ:EXAMPLE", "1D", 20)
                    .unwrap()
                    .arguments(),
            )],
        )
        .await;
        assert_eq!(outcome, Err(Failure::Timeout));
        let diagnostics = http.diagnostics();
        assert_eq!(
            diagnostics["protocol"][waiting_method]["phase"],
            "await_headers"
        );
        assert!(diagnostics["protocol"][waiting_method]["status"].is_null());
        if waiting_method == "tools/list" {
            assert_eq!(
                diagnostics["protocol"]["initialize"]["phase"],
                "json_complete"
            );
            assert_eq!(
                diagnostics["protocol"]["notifications/initialized"]["phase"],
                "accepted"
            );
        }
        assert_eq!(server.calls("tools/call"), 0);
        let encoded = diagnostics.to_string();
        for secret in [
            "synthetic-secret-token",
            "NASDAQ:EXAMPLE",
            "params",
            "arguments",
        ] {
            assert!(!encoded.contains(secret));
        }
    }
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
        let (_root, _guard, _budget, http, _store) = context(&server, 30).await;
        let result = if mode == "stall" {
            let read =
                tokio::spawn(
                    async move { proof::read(&http, "synthetic-access".into(), "1D").await },
                );
            // Establish that this is a response wait, not a slow TCP/SDK setup.
            tokio::time::timeout(Duration::from_secs(10), async {
                while server.calls("tools/call") == 0 {
                    sleep(Duration::from_millis(1)).await;
                }
            })
            .await
            .unwrap();
            tokio::time::pause();
            tokio::time::advance(Duration::from_secs(31)).await;
            let result = read.await.unwrap();
            tokio::time::resume();
            result
        } else {
            proof::read(&http, "synthetic-access".into(), "1D").await
        };
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
    assert_eq!(auth.refresh().await.unwrap_err(), Failure::CredentialWrite);
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
async fn intraday_service_sends_exact_intervals_once_and_preserves_partiality() {
    use crate::client::{Operation, execute};
    use tradingview_model::mcp_bars::Request as BarsRequest;

    for timeframe in ["1m", "5m", "15m", "30m", "1h", "4h"] {
        let server = Server::start("json").await;
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
            .find(|(key, _)| key == "state")
            .unwrap()
            .1
            .into_owned();
        auth.exchange("synthetic-code", &state, None).await.unwrap();

        let data = execute(
            Operation::Bars(BarsRequest::new("NASDAQ:EXAMPLE", timeframe, 20).unwrap()),
            &mut guard,
            store,
            http,
            budget,
            true,
        )
        .await
        .unwrap();
        assert_eq!(data["request"]["timeframe"], timeframe);
        assert_eq!(data["client_observation"]["count_status"], "short");
        assert_eq!(
            data["client_observation"]["calendar_coverage"],
            "unconfirmed"
        );
        assert!(data["bars"][0]["volume"].is_null());
        assert_eq!(server.calls("tools/call"), 1);

        let requests = server.requests.lock().unwrap();
        let call = requests
            .iter()
            .filter_map(|request| serde_json::from_slice::<Value>(&request.body).ok())
            .find(|value| value["method"] == "tools/call")
            .unwrap();
        assert_eq!(
            call["params"]["arguments"],
            json!({
                "symbol": "NASDAQ:EXAMPLE",
                "interval": timeframe,
                "count": 20,
                "summary": false
            })
        );
    }
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

#[tokio::test]
async fn authenticated_read_reuses_the_validated_record_without_reopening_storage() {
    use crate::client::{Operation, execute};
    use tradingview_model::mcp_bars::Request as BarsRequest;

    let server = Server::start("json").await;
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
        .find(|(key, _)| key == "state")
        .unwrap()
        .1
        .into_owned();
    auth.exchange("synthetic-code", &state, None).await.unwrap();

    // Match the public command: preflight reads storage, then SDK restore/token
    // access the same operation snapshot. Reopening storage would fail here.
    let reader = store.next_operation();
    reader.fail_loads_after(1);
    assert!(reader.load_record().await.unwrap().is_some());
    let result = execute(
        Operation::Bars(BarsRequest::new("NASDAQ:AAPL", "1D", 20).unwrap()),
        &mut guard,
        reader.clone(),
        http,
        budget,
        true,
    )
    .await
    .unwrap();
    assert_eq!(result["contract_version"], "mcp_bars.v1");
    assert_eq!(reader.storage_loads(), 1);
    assert_eq!(server.calls("tools/call"), 1);
    assert_eq!(
        reader.next_operation().load_record().await.unwrap_err(),
        Failure::CredentialRead
    );
}

#[tokio::test]
async fn symbol_data_commands_share_authentication_and_preserve_faults_without_replay() {
    use crate::client::{Operation, execute};
    use tradingview_model::mcp_data::Request as DataRequest;

    for mode in [
        "json",
        "sse",
        "429",
        "401",
        "malformed",
        "tool-error",
        "schema-change",
    ] {
        for request in [
            DataRequest::screener(tradingview_model::mcp_data::ScreenerOptions {
                limit: 3,
                columns: vec!["close".into(), "volume".into()],
                ..Default::default()
            })
            .unwrap(),
            DataRequest::search("Example", None).unwrap(),
            DataRequest::columns(None, None, Some("volume")).unwrap(),
            DataRequest::columns(None, None, None).unwrap(),
            DataRequest::symbols(
                &["NASDAQ:EXAMPLE".into(), "NYSE:MISSING".into()],
                &["close".into(), "volume".into()],
            )
            .unwrap(),
            DataRequest::symbol(
                "NASDAQ:EXAMPLE",
                &["close".into(), "volume".into(), "market_cap_basic".into()],
            )
            .unwrap(),
        ] {
            let server = Server::start(mode).await;
            let (_root, mut guard, budget, http, store) = context(&server, 30).await;
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
                .find(|(key, _)| key == "state")
                .unwrap()
                .1
                .into_owned();
            auth.exchange("synthetic-code", &state, None).await.unwrap();
            let result = execute(
                Operation::Data(request.clone()),
                &mut guard,
                store,
                http,
                budget,
                true,
            )
            .await;
            assert_eq!(
                server.calls("tools/call"),
                if mode == "schema-change" { 0 } else { 1 }
            );
            if matches!(mode, "json" | "sse") {
                let data = result.unwrap();
                assert_eq!(data["source"], "tradingview_mcp");
                assert_eq!(data["request"], request.arguments());
                assert_eq!(data["transport"]["tool_attempts"], 1);
                if request.kind() == tradingview_model::mcp_data::Kind::Screener {
                    assert_eq!(data["contract_version"], "mcp_screener.v1");
                    assert_eq!(data["client_observation"]["coverage_status"], "limited");
                    assert_eq!(data["items"][0]["fields"]["close"], 0);
                    assert!(data["items"][0]["fields"]["volume"].is_null());
                }
                if request.kind() == tradingview_model::mcp_data::Kind::Symbols {
                    assert_eq!(data["contract_version"], "mcp_symbols.v1");
                    assert_eq!(data["items"][0]["status"], "returned");
                    assert_eq!(data["items"][1]["status"], "missing");
                    assert_eq!(data["client_observation"]["symbols_status"], "partial");
                }
                if request.kind() == tradingview_model::mcp_data::Kind::Symbol {
                    assert_eq!(data["client_observation"]["fields_status"], "incomplete");
                    assert!(data["fields"]["volume"].is_null());
                    assert_eq!(
                        data["client_observation"]["missing_fields"]
                            .as_array()
                            .unwrap()
                            .len(),
                        2
                    );
                }
            } else {
                let details = result.unwrap_err().details.unwrap();
                assert_eq!(details["contract_version"], "mcp_error.v1");
                assert_eq!(
                    details["tool_attempts"],
                    if mode == "schema-change" { 0 } else { 1 }
                );
                assert!(!details.to_string().contains("synthetic-access"));
                if mode == "401" {
                    assert_eq!(details["code"], "auth_refreshed_retry_required");
                }
            }
        }
    }
}

#[tokio::test]
async fn account_read_service_preserves_contracts_and_never_replays_failures() {
    use crate::client::{Operation, execute};
    use tradingview_model::mcp_account::{Kind, Request};

    for request in [
        Request::watchlists(),
        Request::watchlist("12").unwrap(),
        Request::alerts(Some("NASDAQ:EXAMPLE"), Some(false)).unwrap(),
        Request::alert_details(&[12, 10]).unwrap(),
    ] {
        for mode in [
            "json",
            "sse",
            "401",
            "429",
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
                .find(|(key, _)| key == "state")
                .unwrap()
                .1
                .into_owned();
            auth.exchange("synthetic-code", &state, None).await.unwrap();

            let result = execute(
                Operation::Account(request.clone()),
                &mut guard,
                store,
                http,
                budget,
                true,
            )
            .await;
            assert_eq!(
                server.calls("tools/call"),
                if mode == "schema-change" { 0 } else { 1 }
            );
            if matches!(mode, "json" | "sse") {
                let data = result.unwrap();
                assert_eq!(data["source"], "tradingview_mcp");
                assert_eq!(data["request"], request.arguments());
                assert_eq!(data["transport"]["tool_attempts"], 1);
                if request.kind() == Kind::AlertDetails {
                    assert_eq!(data["items"][1]["status"], "unreported");
                    assert!(data["items"][1]["alert"].is_null());
                }
                if request.kind() == Kind::Watchlist {
                    assert_eq!(data["watchlist"]["id"], "12");
                }
                let requests = server.requests.lock().unwrap();
                let call = requests
                    .iter()
                    .filter_map(|request| serde_json::from_slice::<Value>(&request.body).ok())
                    .find(|value| value["method"] == "tools/call")
                    .unwrap();
                assert_eq!(call["params"]["arguments"], request.arguments());
            } else {
                let details = result.unwrap_err().details.unwrap();
                assert_eq!(details["contract_version"], "mcp_error.v1");
                assert!(!details.to_string().contains("synthetic-access"));
                if mode == "401" {
                    assert_eq!(details["code"], "auth_refreshed_retry_required");
                }
            }
        }
    }
}

#[tokio::test]
async fn watchlist_changes_dispatch_once_and_separate_readback_from_mutation() {
    use crate::client::{Operation, execute};
    use tradingview_model::mcp_account::WatchlistMutation;

    for (request, mode, expected) in [
        (
            WatchlistMutation::create("Example", &["NASDAQ:EXAMPLE".into()]).unwrap(),
            "json",
            "matched",
        ),
        (
            WatchlistMutation::update("12", Some("Example"), None).unwrap(),
            "sse",
            "matched",
        ),
        (
            WatchlistMutation::symbols("12", &["NASDAQ:EXAMPLE".into()], false).unwrap(),
            "json",
            "matched",
        ),
        (
            WatchlistMutation::symbols("12", &["NYSE:OTHER".into()], true).unwrap(),
            "json",
            "matched",
        ),
        (
            WatchlistMutation::delete("12").unwrap(),
            "watchlist-deleted",
            "not_reported",
        ),
        (
            WatchlistMutation::delete("12").unwrap(),
            "json",
            "still_present",
        ),
        (
            WatchlistMutation::create("Example", &[]).unwrap(),
            "mutation-no-id",
            "not_performed",
        ),
        (
            WatchlistMutation::update("12", Some("After"), None).unwrap(),
            "json",
            "mismatch",
        ),
        (
            WatchlistMutation::update("12", Some("Example"), None).unwrap(),
            "readback-429",
            "failed",
        ),
    ] {
        let server = Server::start(mode).await;
        let (_root, mut guard, budget, http, store) = context(&server, 8).await;
        fixture_login(&http, &store, &budget).await;
        let result = execute(
            Operation::WatchlistMutation(request),
            &mut guard,
            store,
            http,
            budget,
            true,
        )
        .await
        .unwrap();
        assert_eq!(result["contract_version"], "mcp_watchlist_mutation.v1");
        assert_eq!(result["mutation"]["tool_attempts"], 1);
        assert_eq!(result["mutation"]["status"], "response_received");
        assert_eq!(result["readback"]["status"], expected);
        assert_eq!(
            server.calls("tools/call"),
            if expected == "not_performed" { 1 } else { 2 }
        );
        if expected == "failed" {
            assert_eq!(result["readback"]["error"]["code"], "rate_limited");
        }
    }
}

#[tokio::test]
async fn failed_account_changes_do_not_refresh_or_replay() {
    use crate::client::{Operation, execute};
    use tradingview_model::mcp_account::WatchlistMutation;

    for alert in [false, true] {
        for mode in [
            "401",
            "429",
            "500",
            "malformed",
            "tool-error",
            "schema-change",
            "stall",
        ] {
            let server = Server::start(mode).await;
            let (_root, mut guard, budget, http, store) = context(&server, 8).await;
            fixture_login(&http, &store, &budget).await;
            let counts = budget.clone();
            let request = WatchlistMutation::create("Example", &[]).unwrap();
            let operation = if alert {
                Operation::AlertMutation(
                    tradingview_model::mcp_account::AlertMutation::state(
                        tradingview_model::mcp_account::AlertAction::Stop,
                        &[12],
                    )
                    .unwrap(),
                )
            } else {
                Operation::WatchlistMutation(request)
            };
            let task = tokio::spawn(async move {
                execute(operation, &mut guard, store, http, budget, true).await
            });
            if mode == "stall" {
                tokio::time::timeout(Duration::from_secs(3), async {
                    while server.calls("tools/call") == 0 {
                        sleep(Duration::from_millis(1)).await;
                    }
                })
                .await
                .unwrap();
                tokio::time::pause();
                tokio::time::advance(Duration::from_secs(31)).await;
            }
            let result = task.await.unwrap();
            if mode == "stall" {
                tokio::time::resume();
            }
            let details = result.unwrap_err().details.unwrap();
            assert_eq!(
                details["mutation"]["status"],
                if mode == "schema-change" {
                    "not_attempted"
                } else {
                    "outcome_unknown"
                }
            );
            assert_eq!(
                server.calls("tools/call"),
                if mode == "schema-change" { 0 } else { 1 }
            );
            assert_eq!(counts.lock().unwrap().counts().refresh, 0);
            assert_eq!(details["mutation"]["automatic_retry"], false);
        }
    }
}

async fn fixture_login(http: &Http, store: &Store, budget: &Arc<Mutex<Budget>>) {
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
        .find(|(key, _)| key == "state")
        .unwrap()
        .1
        .into_owned();
    auth.exchange("synthetic-code", &state, None).await.unwrap();
}

#[tokio::test]
async fn alert_changes_separate_received_reply_and_per_target_readback() {
    use crate::client::{Operation, execute};
    use tradingview_model::mcp_account::{AlertAction, AlertMutation, AlertSettings};

    let create = || {
        AlertMutation::create(
            "NASDAQ:EXAMPLE",
            100.0,
            "greater",
            "1D",
            AlertSettings {
                name: Some("Example".into()),
                ..Default::default()
            },
        )
        .unwrap()
    };
    for (request, mode, expected) in [
        (create(), "alert-active", "matched"),
        (
            AlertMutation::update(
                12,
                AlertSettings {
                    email: Some(false),
                    ..Default::default()
                },
            )
            .unwrap(),
            "alert-sse",
            "matched",
        ),
        (
            AlertMutation::state(AlertAction::Stop, &[12]).unwrap(),
            "json",
            "matched",
        ),
        (
            AlertMutation::state(AlertAction::Restart, &[12]).unwrap(),
            "alert-active",
            "matched",
        ),
        (
            AlertMutation::state(AlertAction::Stop, &[12, 13]).unwrap(),
            "json",
            "unconfirmed",
        ),
        (
            AlertMutation::state(AlertAction::Stop, &[12]).unwrap(),
            "alert-active",
            "mismatch",
        ),
        (
            AlertMutation::state(AlertAction::Delete, &[12]).unwrap(),
            "alert-deleted",
            "not_reported",
        ),
        (
            AlertMutation::state(AlertAction::Delete, &[12]).unwrap(),
            "json",
            "mismatch",
        ),
        (create(), "mutation-no-id", "not_performed"),
        (create(), "readback-429", "failed"),
    ] {
        let server = Server::start(mode).await;
        let (_root, mut guard, budget, http, store) = context(&server, 8).await;
        fixture_login(&http, &store, &budget).await;
        let result = execute(
            Operation::AlertMutation(request),
            &mut guard,
            store,
            http,
            budget,
            true,
        )
        .await
        .unwrap();
        assert_eq!(result["contract_version"], "mcp_alert_mutation.v1");
        assert_eq!(result["mutation"]["tool_attempts"], 1);
        assert_eq!(result["mutation"]["status"], "response_received");
        assert_eq!(result["readback"]["status"], expected);
        assert_eq!(
            server.calls("tools/call"),
            if expected == "not_performed" { 1 } else { 2 }
        );
        if expected == "failed" {
            assert_eq!(result["readback"]["error"]["code"], "rate_limited");
        }
    }
}

#[tokio::test]
async fn financial_service_keeps_source_values_and_failure_dispatch_counts() {
    use crate::client::{Operation, execute};
    use tradingview_model::mcp_financials::Request;

    for request in [
        Request::snapshot("NASDAQ:EXAMPLE", "ttm", &[]).unwrap(),
        Request::history("NASDAQ:EXAMPLE", "fq", Some("2025-01-01"), None).unwrap(),
        Request::forecasts("NASDAQ:EXAMPLE").unwrap(),
        Request::earnings(&["NASDAQ:EXAMPLE".into(), "NYSE:OTHER".into()], None, None).unwrap(),
    ] {
        for mode in [
            "json",
            "sse",
            "401",
            "429",
            "500",
            "malformed",
            "tool-error",
            "schema-change",
        ] {
            let server = Server::start(mode).await;
            let (_root, mut guard, budget, http, store) = context(&server, 8).await;
            fixture_login(&http, &store, &budget).await;
            let result = execute(
                Operation::Financial(request.clone()),
                &mut guard,
                store,
                http,
                budget,
                true,
            )
            .await;
            assert_eq!(
                server.calls("tools/call"),
                if mode == "schema-change" { 0 } else { 1 }
            );
            if matches!(mode, "json" | "sse") {
                let data = result.unwrap();
                assert_eq!(data["source"], "tradingview_mcp");
                assert_eq!(data["transport"]["tool_attempts"], 1);
                assert_eq!(data["client_observation"]["completeness"], "unconfirmed");
            } else {
                let details = result.unwrap_err().details.unwrap();
                assert_eq!(
                    details["tool_attempts"],
                    if mode == "schema-change" { 0 } else { 1 }
                );
                assert!(!details.to_string().contains("synthetic-secret"));
            }
        }
    }
}

#[tokio::test]
async fn research_service_keeps_references_access_and_single_dispatch_failures() {
    use crate::client::{Operation, execute};
    use tradingview_model::mcp_research::{DocumentOptions, Request};

    for request in [
        Request::news("NASDAQ:EXAMPLE", "en", 2, 0).unwrap(),
        Request::story("urn:newsml:example:one", "en", "non_pro", None).unwrap(),
        Request::documents("NASDAQ:EXAMPLE", DocumentOptions::default()).unwrap(),
        Request::document("opaque-view").unwrap(),
    ] {
        for mode in [
            "json",
            "sse",
            "401",
            "429",
            "500",
            "malformed",
            "schema-change",
            "tool-error",
        ] {
            let server = Server::start(mode).await;
            let (_root, mut guard, budget, http, store) = context(&server, 8).await;
            fixture_login(&http, &store, &budget).await;
            let result = execute(
                Operation::Research(request.clone()),
                &mut guard,
                store,
                http,
                budget,
                true,
            )
            .await;
            assert_eq!(
                server.calls("tools/call"),
                if mode == "schema-change" { 0 } else { 1 }
            );
            if matches!(mode, "json" | "sse") {
                let data = result.unwrap();
                assert_eq!(data["source"], "tradingview_mcp");
                assert_eq!(data["transport"]["tool_attempts"], 1);
                assert_eq!(data["client_observation"]["completeness"], "unconfirmed");
            } else {
                let details = result.unwrap_err().details.unwrap();
                assert_eq!(
                    details["tool_attempts"],
                    if mode == "schema-change" { 0 } else { 1 }
                );
                assert!(!details.to_string().contains("synthetic-secret"));
            }
        }
    }
}

#[tokio::test]
async fn economic_service_keeps_nulls_and_single_dispatch_failures() {
    use crate::client::{Operation, execute};
    use tradingview_model::mcp_economics::{CalendarOptions, DividendOptions, Request};

    for request in [
        Request::symbols(Some("US"), None, None).unwrap(),
        Request::series("ECONOMICS:USEXAMPLE", None, None).unwrap(),
        Request::calendar(CalendarOptions::default()).unwrap(),
        Request::dividends(DividendOptions {
            symbols: vec!["NASDAQ:EXAMPLE".into(), "NYSE:OTHER".into()],
            ..Default::default()
        })
        .unwrap(),
        Request::dividends(DividendOptions {
            market: Some("america".into()),
            limit: Some(2),
            ..Default::default()
        })
        .unwrap(),
    ] {
        for mode in [
            "json",
            "sse",
            "401",
            "429",
            "500",
            "malformed",
            "schema-change",
            "tool-error",
        ] {
            let server = Server::start(mode).await;
            let (_root, mut guard, budget, http, store) = context(&server, 8).await;
            fixture_login(&http, &store, &budget).await;
            let result = execute(
                Operation::Economic(request.clone()),
                &mut guard,
                store,
                http,
                budget,
                true,
            )
            .await;
            assert_eq!(
                server.calls("tools/call"),
                if mode == "schema-change" { 0 } else { 1 }
            );
            if matches!(mode, "json" | "sse") {
                let data = result.unwrap();
                assert_eq!(data["source"], "tradingview_mcp");
                assert_eq!(data["transport"]["tool_attempts"], 1);
                assert_eq!(data["client_observation"]["completeness"], "unconfirmed");
            } else {
                let details = result.unwrap_err().details.unwrap();
                assert_eq!(
                    details["tool_attempts"],
                    if mode == "schema-change" { 0 } else { 1 }
                );
                assert!(!details.to_string().contains("synthetic-secret"));
            }
        }
    }
}

#[tokio::test]
async fn dividend_service_distinguishes_http_throttling_from_application_errors() {
    use crate::client::{Operation, execute};
    use tradingview_model::mcp_economics::{DividendOptions, Request};

    for market_mode in [false, true] {
        for mode in ["429", "application-429", "401"] {
            let server = Server::start(mode).await;
            let (_root, mut guard, budget, http, store) = context(&server, 8).await;
            fixture_login(&http, &store, &budget).await;
            let request = Request::dividends(if market_mode {
                DividendOptions {
                    market: Some("america".into()),
                    limit: Some(2),
                    ..Default::default()
                }
            } else {
                DividendOptions {
                    symbols: vec!["NASDAQ:EXAMPLE".into()],
                    ..Default::default()
                }
            })
            .unwrap();
            let error = execute(
                Operation::Economic(request),
                &mut guard,
                store,
                http,
                budget,
                true,
            )
            .await
            .unwrap_err();
            assert!(!format!("{error:?}").contains("synthetic-secret"));
            let details = error.details.unwrap();
            assert_eq!(server.calls("tools/call"), 1);
            assert_eq!(details["tool_attempts"], 1);
            match mode {
                "429" => {
                    assert_eq!(details["code"], "rate_limited");
                    assert_eq!(details["retry_after_seconds"], 123);
                    assert_eq!(details["retry_after_evidence"], "http_header");
                    assert_eq!(guard.check_cooldown(), Err(Failure::RateLimited));
                }
                "application-429" => {
                    assert_eq!(details["code"], "provider_error");
                    assert!(details.get("retry_after_seconds").is_none());
                    assert!(details.get("local_cooldown_seconds").is_none());
                    assert_eq!(guard.check_cooldown(), Ok(()));
                }
                "401" => {
                    assert_eq!(details["code"], "auth_refreshed_retry_required");
                    assert_eq!(
                        details["next_action"],
                        "repeat the same explicit tv mcp command"
                    );
                }
                _ => unreachable!(),
            }
        }
    }
}

#[tokio::test]
async fn alert_history_service_preserves_outcomes_without_replay() {
    for (mode, error_code) in [
        ("alert-history-valid", None),
        ("alert-history-empty", None),
        ("alert-history-mismatch", Some("invalid_response")),
        ("alert-history-invalid", Some("invalid_response")),
        ("alert-history-provider-error", Some("provider_error")),
        ("alert-history-429", Some("rate_limited")),
    ] {
        let server = Server::start(mode).await;
        let (_root, mut guard, budget, http, store) = context(&server, 8).await;
        fixture_login(&http, &store, &budget).await;
        let request =
            tradingview_model::mcp_account::Request::alert_history("NASDAQ:EXAMPLE", 7, 100)
                .unwrap();
        let result = crate::client::execute(
            crate::Operation::Account(request),
            &mut guard,
            store,
            http,
            budget,
            true,
        )
        .await;
        assert_eq!(server.calls("tools/call"), 1);
        match error_code {
            Some(code) => {
                let details = result.unwrap_err().details.unwrap();
                assert_eq!(details["code"], code);
                assert_eq!(details["tool_attempts"], 1);
            }
            None => {
                let data = result.unwrap();
                assert_eq!(data["contract_version"], "mcp_alert_history.v1");
                assert_eq!(
                    data["returned_count"],
                    if mode == "alert-history-empty" { 0 } else { 1 }
                );
                assert_eq!(data["coverage"], "unconfirmed");
                assert!(!data.to_string().contains("private"));
            }
        }
    }
}
