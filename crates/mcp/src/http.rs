//! One-dispatch HTTP adapter. Never use the SDK AuthClient refresh/replay wrapper.

use crate::{
    Failure, Result,
    budget::{Budget, RequestClass},
    sse,
};
use futures_util::StreamExt;
use http::{HeaderName, HeaderValue, Method, StatusCode};
use rmcp::{
    model::ClientJsonRpcMessage,
    transport::{
        auth::{OAuthHttpClient, OAuthHttpClientFuture, OAuthHttpRequest},
        common::client_side_sse::BoxedSseResponse,
        streamable_http_client::{
            StreamableHttpClient, StreamableHttpError, StreamableHttpPostResponse,
        },
    },
};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::time::{Instant, timeout_at};

#[derive(Clone)]
pub(crate) struct Endpoints {
    pub resource: String,
    pub issuer: String,
    pub resource_metadata: String,
    pub authorization_metadata: String,
    pub authorize: String,
    pub token: String,
    pub register: String,
}

impl Endpoints {
    pub fn tradingview() -> Self {
        Self {
            resource: "https://mcp.tradingview.com/mcp".into(),
            issuer: "https://www.tradingview.com".into(),
            resource_metadata:
                "https://mcp.tradingview.com/.well-known/oauth-protected-resource/mcp".into(),
            authorization_metadata:
                "https://www.tradingview.com/.well-known/oauth-authorization-server".into(),
            authorize: "https://www.tradingview.com/mcp/oauth/authorize".into(),
            token: "https://www.tradingview.com/mcp/oauth/token".into(),
            register: "https://www.tradingview.com/mcp/oauth/register".into(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct Http {
    client: reqwest::Client,
    pub endpoints: Endpoints,
    pub deadline: Instant,
    budget: Arc<Mutex<Budget>>,
    fault: Arc<Mutex<Option<Failure>>>,
    cooldown: Arc<Mutex<Option<u64>>>,
    tool_sent: Arc<Mutex<HashSet<String>>>,
    diagnostics: Arc<Mutex<Value>>,
}

impl Http {
    pub fn new(
        endpoints: Endpoints,
        deadline: Instant,
        budget: Arc<Mutex<Budget>>,
    ) -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(concat!("tv/", env!("CARGO_PKG_VERSION")))
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .referer(false)
            .connect_timeout(Duration::from_secs(5))
            .build()
            .map_err(|_| Failure::Connection)?;
        Ok(Self {
            client,
            endpoints,
            deadline,
            budget,
            fault: Arc::new(Mutex::new(None)),
            cooldown: Arc::new(Mutex::new(None)),
            tool_sent: Arc::new(Mutex::new(HashSet::new())),
            diagnostics: Arc::new(Mutex::new(serde_json::json!({}))),
        })
    }

    pub fn diagnostics(&self) -> Value {
        self.diagnostics
            .lock()
            .map(|v| v.clone())
            .unwrap_or(Value::Null)
    }

    // Private proof diagnostics only: closed protocol names, phases and statuses.
    // Keep each operation separate because the SDK may also open an SSE stream.
    fn describe_protocol(&self, method: &str, phase: &str, status: Option<StatusCode>) {
        if !matches!(
            method,
            "initialize"
                | "notifications/initialized"
                | "notifications/cancelled"
                | "tools/list"
                | "tools/call"
                | "event_stream"
                | "delete_session"
        ) {
            return;
        }
        if let Ok(mut diagnostics) = self.diagnostics.lock() {
            if !diagnostics["protocol"].is_object() {
                diagnostics["protocol"] = serde_json::json!({});
            }
            diagnostics["protocol"][method] = serde_json::json!({
                "phase": phase,
                "status": status.map(|value| value.as_u16()),
                "remaining_ms": self.deadline.saturating_duration_since(Instant::now()).as_millis()
            });
        }
    }

    pub(crate) fn describe_catalog(&self, tools: &[rmcp::model::Tool]) {
        if let Ok(mut diagnostics) = self.diagnostics.lock() {
            diagnostics["tool_count_on_page"] = serde_json::json!(tools.len());
            // Tool identifiers only, never descriptions, schemas or arguments.
            // Omit opaque numeric/hex identifiers from diagnostic output.
            let names: Vec<_> = tools
                .iter()
                .take(40)
                .filter_map(|tool| {
                    let name = tool.name.as_ref();
                    (name.len() <= 128
                        && name
                            .bytes()
                            .all(|b| b.is_ascii_alphabetic() || b"_-.:/".contains(&b)))
                    .then_some(name)
                })
                .collect();
            diagnostics["tool_names_without_opaque_identifiers"] = serde_json::json!(names);
            diagnostics["ohlcv_suffix_present"] =
                serde_json::json!(tools.iter().any(|t| t.name.ends_with("get_ohlcv")));
            // Public names from the published tool documentation, not arbitrary
            // names/descriptions supplied by a server or account-local values.
            for name in [
                "get_ohlcv",
                "get_symbol_data",
                "run_screener",
                "list_watchlists",
                "search_symbols",
            ] {
                diagnostics["documented_names_present"][name] =
                    serde_json::json!(tools.iter().any(|t| t.name == name));
            }
        }
    }

    pub(crate) fn describe_ohlcv_schema(&self, schema: Option<&serde_json::Map<String, Value>>) {
        if let Ok(mut diagnostics) = self.diagnostics.lock() {
            diagnostics["stage"] = serde_json::json!("tool_schema");
            diagnostics["get_ohlcv_present"] = serde_json::json!(schema.is_some());
            if let Some(schema) = schema {
                let properties = schema.get("properties").and_then(Value::as_object);
                // Closed field names and JSON type names only; no descriptions,
                // examples, provider identifiers or arbitrary schema values.
                for name in ["symbol", "interval", "count", "summary"] {
                    let field = properties.and_then(|p| p.get(name));
                    let kind = field.and_then(|v| v.get("type")).and_then(Value::as_str);
                    diagnostics["ohlcv_input_types"][name] = serde_json::json!(match kind {
                        Some(
                            "string" | "number" | "integer" | "boolean" | "array" | "object"
                            | "null",
                        ) => kind,
                        _ => None,
                    });
                    diagnostics["ohlcv_input_any_of"][name] =
                        serde_json::json!(field.is_some_and(|v| v.get("anyOf").is_some()));
                }
            }
        }
    }

    pub fn failure(&self) -> Failure {
        self.fault
            .lock()
            .ok()
            .and_then(|v| *v)
            .unwrap_or(Failure::InvalidResponse)
    }

    pub(crate) fn tool_attempts(&self) -> Result<u32> {
        Ok(self
            .budget
            .lock()
            .map_err(|_| Failure::LocalState)?
            .counts()
            .tools)
    }

    pub fn cooldown(&self) -> Option<u64> {
        self.cooldown.lock().ok().and_then(|v| *v)
    }

    fn remember(&self, error: Failure) -> Failure {
        if let Ok(mut fault) = self.fault.lock() {
            fault.get_or_insert(error);
        }
        error
    }

    fn charge(&self, class: RequestClass) -> Result<()> {
        if Instant::now() >= self.deadline {
            return Err(self.remember(Failure::Timeout));
        }
        self.budget
            .lock()
            .map_err(|_| Failure::LocalState)?
            .charge(class)
            .map_err(|e| self.remember(e))
    }

    async fn send(&self, request: reqwest::RequestBuilder) -> Result<reqwest::Response> {
        let response = timeout_at(self.deadline, request.send())
            .await
            .map_err(|_| self.remember(Failure::Timeout))?
            .map_err(|error| {
                self.remember(if error.is_timeout() {
                    Failure::Timeout
                } else {
                    Failure::Connection
                })
            })?;
        if let Ok(mut value) = self.diagnostics.lock() {
            value["last_http_status"] = serde_json::json!(response.status().as_u16());
            value["browser_challenge"] = serde_json::json!(
                response
                    .headers()
                    .get("cf-mitigated")
                    .and_then(|v| v.to_str().ok())
                    == Some("challenge")
            );
            value["html_response"] = serde_json::json!(
                response
                    .headers()
                    .get(http::header::CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok())
                    .is_some_and(|v| v.starts_with("text/html"))
            );
            if response
                .headers()
                .get(http::header::WWW_AUTHENTICATE)
                .and_then(|v| v.to_str().ok())
                .is_some_and(|v| v.contains("scope=\"mcp:tools\""))
            {
                value["scope_hint"] = serde_json::json!("mcp:tools");
            }
        }
        Ok(response)
    }

    pub async fn bytes(&self, mut response: reqwest::Response) -> Result<Vec<u8>> {
        let result = timeout_at(self.deadline, async {
            let mut bytes = Vec::new();
            while let Some(chunk) = response.chunk().await.map_err(|e| {
                if e.is_timeout() {
                    Failure::Timeout
                } else {
                    Failure::Connection
                }
            })? {
                if bytes.len().saturating_add(chunk.len()) > sse::MAX_RESPONSE_BYTES {
                    return Err(Failure::ResponseTooLarge);
                }
                bytes.extend_from_slice(&chunk);
            }
            Ok(bytes)
        })
        .await
        .map_err(|_| self.remember(Failure::Timeout))?;
        result.map_err(|e| self.remember(e))
    }

    fn status(&self, response: &reqwest::Response) -> Result<()> {
        let error = match response.status() {
            StatusCode::UNAUTHORIZED => Some(Failure::AuthRequired),
            StatusCode::FORBIDDEN => Some(Failure::AccessDenied),
            StatusCode::TOO_MANY_REQUESTS => {
                let wait =
                    retry_after_seconds(response.headers(), crate::admission::now_ms()? / 1000);
                *self.cooldown.lock().map_err(|_| Failure::LocalState)? = Some(wait);
                if let Ok(mut diagnostics) = self.diagnostics.lock() {
                    let observed = response.headers().contains_key(http::header::RETRY_AFTER)
                        && wait != u64::MAX;
                    diagnostics["retry_after_seconds"] = if observed {
                        serde_json::json!(wait)
                    } else {
                        Value::Null
                    };
                    diagnostics["retry_after_evidence"] = serde_json::json!(if observed {
                        "http_header"
                    } else {
                        "unconfirmed"
                    });
                }
                Some(Failure::RateLimited)
            }
            status if !status.is_success() => Some(Failure::ProviderError),
            _ => None,
        };
        error.map_or(Ok(()), |e| Err(self.remember(e)))
    }

    pub async fn metadata(&self, uri: &str) -> Result<Value> {
        if uri != self.endpoints.resource_metadata && uri != self.endpoints.authorization_metadata {
            return Err(Failure::BindingMismatch);
        }
        self.charge(RequestClass::Metadata)?;
        if let Ok(mut diagnostics) = self.diagnostics.lock() {
            diagnostics["stage"] = serde_json::json!(if uri == self.endpoints.resource_metadata {
                "resource_metadata"
            } else {
                "authorization_metadata"
            });
        }
        let response = self
            .send(
                self.client
                    .get(uri)
                    .header(http::header::ACCEPT, "application/json"),
            )
            .await?;
        self.status(&response)?;
        serde_json::from_slice(&self.bytes(response).await?)
            .map_err(|_| self.remember(Failure::InvalidResponse))
    }

    fn protocol_request(
        &self,
        method: Method,
        uri: &str,
        session: Option<&str>,
        token: Option<String>,
        headers: HashMap<HeaderName, HeaderValue>,
    ) -> Result<reqwest::RequestBuilder> {
        if uri != self.endpoints.resource {
            return Err(self.remember(Failure::BindingMismatch));
        }
        let token = token.ok_or(Failure::AuthRequired)?;
        let mut request = self
            .client
            .request(method, uri)
            .bearer_auth(token)
            .header(http::header::ACCEPT, "application/json, text/event-stream");
        if let Some(session) = session {
            request = request.header("mcp-session-id", session);
        }
        for (key, value) in headers {
            if key == http::header::AUTHORIZATION
                || key == http::header::HOST
                || key == http::header::COOKIE
            {
                return Err(self.remember(Failure::BindingMismatch));
            }
            request = request.header(key, value);
        }
        Ok(request)
    }

    fn events(&self, response: reqwest::Response, limit: usize) -> BoxedSseResponse {
        let deadline = self.deadline;
        let client = self.clone();
        let raw = futures_util::stream::unfold(
            (response.bytes_stream(), false),
            move |(mut stream, ended)| {
                let client = client.clone();
                async move {
                    if ended {
                        return None;
                    }
                    let (item, ended) = match timeout_at(deadline, stream.next()).await {
                        Ok(Some(Ok(bytes))) => (Ok(bytes), false),
                        Ok(Some(Err(_))) => (Err(client.remember(Failure::Connection)), true),
                        Ok(None) => return None,
                        Err(_) => (Err(client.remember(Failure::Timeout)), true),
                    };
                    Some((item, (stream, ended)))
                }
            },
        );
        let client = self.clone();
        Box::pin(
            sse::decode(raw, limit.min(sse::MAX_RESPONSE_BYTES)).map(move |event| {
                if let Err(ref error) = event {
                    let fault = match error {
                        rmcp::transport::streamable_http_client::SseError::Body(inner) => inner
                            .downcast_ref::<Failure>()
                            .copied()
                            .unwrap_or(Failure::InvalidResponse),
                        _ => Failure::InvalidResponse,
                    };
                    client.remember(fault);
                }
                event
            }),
        )
    }

    async fn post(
        &self,
        uri: Arc<str>,
        message: ClientJsonRpcMessage,
        session: Option<Arc<str>>,
        token: Option<String>,
        headers: HashMap<HeaderName, HeaderValue>,
        limit: usize,
    ) -> Result<StreamableHttpPostResponse> {
        let value = serde_json::to_value(&message).map_err(|_| Failure::InvalidResponse)?;
        let method = value.get("method").and_then(Value::as_str);
        let class = match method {
            Some(
                "initialize"
                | "notifications/initialized"
                | "tools/list"
                | "notifications/cancelled",
            ) => RequestClass::Protocol,
            Some("tools/call") => {
                let name = value
                    .pointer("/params/name")
                    .and_then(Value::as_str)
                    .ok_or(Failure::UnsupportedCapability)?;
                let tool = crate::tools::Tool::from_name(name)?;
                let args = value
                    .pointer("/params/arguments")
                    .ok_or(Failure::InvalidResponse)?;
                tool.validate_arguments(args)?;
                let mut sent = self.tool_sent.lock().map_err(|_| Failure::LocalState)?;
                if !sent.insert(format!("{tool:?}:{args}")) {
                    return Err(self.remember(Failure::BudgetExhausted));
                }
                RequestClass::Tool
            }
            // Do not dispatch replies to server-triggered actions or other tools.
            _ => return Err(self.remember(Failure::UnsupportedCapability)),
        };
        let request = self
            .protocol_request(Method::POST, &uri, session.as_deref(), token, headers)?
            .json(&message);
        self.charge(class)?;
        let method = method.ok_or(Failure::UnsupportedCapability)?;
        self.describe_protocol(method, "await_headers", None);
        let response = self.send(request).await?;
        let status = response.status();
        self.describe_protocol(method, "headers_received", Some(status));
        self.status(&response)?;
        if status == StatusCode::ACCEPTED {
            self.describe_protocol(method, "accepted", Some(status));
            return Ok(StreamableHttpPostResponse::Accepted);
        }
        let session = response
            .headers()
            .get("mcp-session-id")
            .map(|v| v.to_str().map(str::to_owned))
            .transpose()
            .map_err(|_| Failure::InvalidResponse)?;
        let mime = response
            .headers()
            .get(http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .to_owned();
        match mime.as_str() {
            "application/json" => {
                self.describe_protocol(method, "json_body", Some(status));
                let message = serde_json::from_slice(&self.bytes(response).await?)
                    .map_err(|_| self.remember(Failure::InvalidResponse))?;
                self.describe_protocol(method, "json_complete", Some(status));
                Ok(StreamableHttpPostResponse::Json(message, session))
            }
            "text/event-stream" => {
                self.describe_protocol(method, "sse_stream", Some(status));
                Ok(StreamableHttpPostResponse::Sse(
                    self.events(response, limit),
                    session,
                ))
            }
            _ => Err(self.remember(Failure::InvalidResponse)),
        }
    }
}

impl StreamableHttpClient for Http {
    type Error = Failure;
    async fn post_message(
        &self,
        uri: Arc<str>,
        message: ClientJsonRpcMessage,
        session: Option<Arc<str>>,
        token: Option<String>,
        headers: HashMap<HeaderName, HeaderValue>,
    ) -> std::result::Result<StreamableHttpPostResponse, StreamableHttpError<Failure>> {
        self.post(
            uri,
            message,
            session,
            token,
            headers,
            sse::MAX_RESPONSE_BYTES,
        )
        .await
        .map_err(StreamableHttpError::Client)
    }

    async fn post_message_with_max_sse_event_size(
        &self,
        uri: Arc<str>,
        message: ClientJsonRpcMessage,
        session: Option<Arc<str>>,
        token: Option<String>,
        headers: HashMap<HeaderName, HeaderValue>,
        limit: usize,
    ) -> std::result::Result<StreamableHttpPostResponse, StreamableHttpError<Failure>> {
        self.post(uri, message, session, token, headers, limit)
            .await
            .map_err(StreamableHttpError::Client)
    }

    async fn get_stream(
        &self,
        uri: Arc<str>,
        session: Option<Arc<str>>,
        last_event_id: Option<String>,
        token: Option<String>,
        headers: HashMap<HeaderName, HeaderValue>,
    ) -> std::result::Result<BoxedSseResponse, StreamableHttpError<Failure>> {
        let result = async {
            if last_event_id.is_some() {
                return Err(Failure::UnsupportedCapability);
            }
            let request =
                self.protocol_request(Method::GET, &uri, session.as_deref(), token, headers)?;
            self.charge(RequestClass::Protocol)?;
            self.describe_protocol("event_stream", "await_headers", None);
            let response = self.send(request).await?;
            self.describe_protocol("event_stream", "headers_received", Some(response.status()));
            if response.status() == StatusCode::METHOD_NOT_ALLOWED {
                return Ok(None);
            }
            self.status(&response)?;
            if !response
                .headers()
                .get(http::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .is_some_and(|s| s.split(';').next() == Some("text/event-stream"))
            {
                return Err(Failure::InvalidResponse);
            }
            Ok(Some(self.events(response, sse::MAX_RESPONSE_BYTES)))
        }
        .await;
        match result {
            Ok(Some(stream)) => Ok(stream),
            Ok(None) => Err(StreamableHttpError::ServerDoesNotSupportSse),
            Err(e) => Err(StreamableHttpError::Client(self.remember(e))),
        }
    }

    async fn delete_session(
        &self,
        uri: Arc<str>,
        session: Arc<str>,
        token: Option<String>,
        headers: HashMap<HeaderName, HeaderValue>,
    ) -> std::result::Result<(), StreamableHttpError<Failure>> {
        let result = async {
            let request =
                self.protocol_request(Method::DELETE, &uri, Some(&session), token, headers)?;
            self.charge(RequestClass::Protocol)?;
            self.describe_protocol("delete_session", "await_headers", None);
            let response = self.send(request).await?;
            self.describe_protocol(
                "delete_session",
                "headers_received",
                Some(response.status()),
            );
            if response.status() == StatusCode::METHOD_NOT_ALLOWED {
                return Ok(());
            }
            self.status(&response)
        }
        .await;
        result.map_err(StreamableHttpError::Client)
    }
}

impl OAuthHttpClient for Http {
    fn execute(&self, request: OAuthHttpRequest) -> OAuthHttpClientFuture<'_> {
        Box::pin(async move {
            let (parts, body) = request.request.into_parts();
            let uri = parts.uri.to_string();
            if parts.method != Method::POST {
                return Err(Box::new(Failure::BindingMismatch) as _);
            }
            let class = if uri == self.endpoints.register {
                RequestClass::Registration
            } else if uri == self.endpoints.token {
                let fields: HashMap<_, _> = reqwest::Url::parse(&format!(
                    "https://example.invalid/?{}",
                    String::from_utf8_lossy(&body)
                ))
                .map_err(|_| Failure::InvalidResponse)?
                .query_pairs()
                .into_owned()
                .collect();
                if fields.get("resource") != Some(&self.endpoints.resource)
                    || fields
                        .get("scope")
                        .is_some_and(|v| v.split_whitespace().any(|s| s != "mcp:read"))
                {
                    return Err(Box::new(Failure::BindingMismatch) as _);
                }
                match fields.get("grant_type").map(String::as_str) {
                    Some("authorization_code") => RequestClass::Exchange,
                    Some("refresh_token") => RequestClass::Refresh,
                    _ => return Err(Box::new(Failure::UnsupportedCapability) as _),
                }
            } else {
                return Err(Box::new(Failure::BindingMismatch) as _);
            };
            let registration = if uri == self.endpoints.register {
                Some(serde_json::from_slice::<Value>(&body).map_err(|_| Failure::InvalidResponse)?)
            } else {
                None
            };
            self.charge(class)?;
            let response = self
                .send(self.client.post(uri).headers(parts.headers).body(body))
                .await?;
            // Return bounded OAuth errors to SDK; expose only our closed Failure
            // vocabulary to callers, never the SDK's body-bearing error strings.
            let status = response.status();
            if status.is_redirection() {
                return Err(Box::new(self.remember(Failure::BindingMismatch)) as _);
            }
            if status == StatusCode::UNAUTHORIZED
                || status == StatusCode::FORBIDDEN
                || status == StatusCode::TOO_MANY_REQUESTS
                || status.is_server_error()
            {
                let _ = self.status(&response);
            }
            let headers = response.headers().clone();
            let body = self.bytes(response).await?;
            if !status.is_success()
                && let Ok(value) = serde_json::from_slice::<Value>(&body)
                && let Some(code) = value.get("error").and_then(Value::as_str).filter(|s| {
                    matches!(
                        *s,
                        "invalid_scope"
                            | "invalid_grant"
                            | "invalid_client"
                            | "access_denied"
                            | "unauthorized_client"
                            | "unsupported_grant_type"
                    )
                })
            {
                if let Ok(mut diagnostics) = self.diagnostics.lock() {
                    diagnostics["oauth_error"] = serde_json::json!(code);
                }
                if code == "invalid_scope" || code == "access_denied" {
                    self.remember(Failure::AccessDenied);
                }
            }
            if status.is_success() {
                let value: Value = serde_json::from_slice(&body)
                    .map_err(|_| self.remember(Failure::InvalidResponse))?;
                if let Some(request) = registration {
                    if value
                        .get("client_id")
                        .and_then(Value::as_str)
                        .is_none_or(str::is_empty)
                    {
                        return Err(Box::new(self.remember(Failure::InvalidResponse)) as _);
                    }
                    if value
                        .get("redirect_uris")
                        .is_some_and(|uris| Some(uris) != request.get("redirect_uris"))
                    {
                        return Err(Box::new(self.remember(Failure::BindingMismatch)) as _);
                    }
                    if value
                        .get("scope")
                        .and_then(Value::as_str)
                        .is_some_and(|v| v.split_whitespace().any(|s| s != "mcp:read"))
                    {
                        return Err(Box::new(self.remember(Failure::AccessDenied)) as _);
                    }
                } else {
                    if value
                        .get("access_token")
                        .and_then(Value::as_str)
                        .is_none_or(str::is_empty)
                        || !value
                            .get("token_type")
                            .and_then(Value::as_str)
                            .is_some_and(|t| t.eq_ignore_ascii_case("bearer"))
                        || value
                            .get("expires_in")
                            .is_some_and(|v| v.as_u64().is_none())
                        || value
                            .get("refresh_token")
                            .is_some_and(|v| v.as_str().is_none_or(str::is_empty))
                    {
                        return Err(Box::new(self.remember(Failure::InvalidResponse)) as _);
                    }
                }
            }
            let mut output = http::Response::builder().status(status).body(body)?;
            *output.headers_mut() = headers;
            Ok(output)
        })
    }
}

// RFC 9110 delta-seconds or IMF-fixdate. An unrecognized server hint blocks
// this proof's remaining reads rather than retrying earlier than it may allow.
fn retry_after_seconds(headers: &http::HeaderMap, now: u64) -> u64 {
    let Some(header) = headers.get(http::header::RETRY_AFTER) else {
        return 60;
    };
    let Ok(text) = header.to_str() else {
        return u64::MAX;
    };
    if !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit()) {
        return text.parse().unwrap_or(u64::MAX);
    }
    http_date_seconds(text)
        .map(|date| date.saturating_sub(now))
        .unwrap_or(u64::MAX)
}

fn http_date_seconds(text: &str) -> Option<u64> {
    let fields: Vec<_> = text.split_whitespace().collect();
    if fields.len() != 6 || fields[5] != "GMT" || !fields[0].ends_with(',') {
        return None;
    }
    let day = fields[1].parse::<u64>().ok()?;
    let month = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ]
    .iter()
    .position(|m| *m == fields[2])?;
    let year = fields[3].parse::<u64>().ok()?;
    if !(1970..=9999).contains(&year) {
        return None;
    }
    let clock: Vec<_> = fields[4]
        .split(':')
        .map(str::parse::<u64>)
        .collect::<std::result::Result<_, _>>()
        .ok()?;
    if clock.len() != 3 || clock[0] > 23 || clock[1] > 59 || clock[2] > 59 {
        return None;
    }
    let leap = |year: u64| {
        year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
    };
    let mut months = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    if leap(year) {
        months[1] = 29;
    }
    if day == 0 || day > months[month] {
        return None;
    }
    let days = (1970..year)
        .map(|y| if leap(y) { 366 } else { 365 })
        .sum::<u64>()
        + months[..month].iter().sum::<u64>()
        + day
        - 1;
    Some(days * 86400 + clock[0] * 3600 + clock[1] * 60 + clock[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_after_date_delta_and_unknown_are_not_shortened() {
        let mut headers = http::HeaderMap::new();
        assert_eq!(retry_after_seconds(&headers, 0), 60);
        headers.insert(http::header::RETRY_AFTER, HeaderValue::from_static("120"));
        assert_eq!(retry_after_seconds(&headers, 0), 120);
        headers.insert(
            http::header::RETRY_AFTER,
            HeaderValue::from_static("Thu, 01 Jan 1970 00:02:00 GMT"),
        );
        assert_eq!(retry_after_seconds(&headers, 30), 90);
        headers.insert(
            http::header::RETRY_AFTER,
            HeaderValue::from_static("unknown"),
        );
        assert_eq!(retry_after_seconds(&headers, 30), u64::MAX);
    }
}
