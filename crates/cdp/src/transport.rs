use std::time::Duration;

use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use tradingview_core::{AppError, ErrorKind};

const CDP_HTTP_CONNECT_TIMEOUT: Duration = Duration::from_secs(1);
const CDP_HTTP_TOTAL_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Target {
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub url: String,
    #[serde(rename = "webSocketDebuggerUrl")]
    pub web_socket_debugger_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetSelection {
    Selected(Target),
    None,
    Ambiguous(Vec<Target>),
}

#[derive(Debug, Clone)]
pub struct TransportConfig {
    pub host: String,
    pub port: u16,
    pub target_id: Option<String>,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 9222,
            target_id: None,
        }
    }
}

impl TransportConfig {
    #[allow(dead_code)]
    pub fn from_env() -> Result<Self, AppError> {
        Self::from_env_with_target_id(None)
    }

    pub fn from_env_with_target_id(target_id: Option<&str>) -> Result<Self, AppError> {
        let host = std::env::var("TV_CDP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = match std::env::var("TV_CDP_PORT") {
            Ok(value) => value.parse::<u16>().map_err(|err| {
                AppError::new(
                    ErrorKind::Validation,
                    format!("TV_CDP_PORT must be a valid port: {err}"),
                )
            })?,
            Err(_) => 9222,
        };
        let cli_target_id = target_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        Ok(Self {
            host,
            port,
            target_id: cli_target_id,
        })
    }

    pub fn list_url(&self) -> String {
        format!("http://{}:{}/json/list", self.host, self.port)
    }

    pub fn version_url(&self) -> String {
        format!("http://{}:{}/json/version", self.host, self.port)
    }

    pub fn activate_url(&self, target_id: &str) -> String {
        format!(
            "http://{}:{}/json/activate/{}",
            self.host, self.port, target_id
        )
    }

    pub fn new_target_url(&self, url: &str) -> String {
        let mut endpoint = Url::parse(&format!("http://{}:{}/json/new", self.host, self.port))
            .expect("local CDP new target URL should be valid");
        endpoint.set_query(Some(url));
        endpoint.to_string()
    }
}

/// Reuses one configured CDP HTTP client for a top-level Desktop workflow.
///
/// The type is intentionally opaque so callers do not need to own or expose a
/// `reqwest` client. Existing free functions remain available for one-off use.
#[derive(Clone)]
pub struct CdpHttpSession {
    config: TransportConfig,
    client: Client,
}

impl CdpHttpSession {
    pub fn new(config: &TransportConfig) -> Result<Self, AppError> {
        Self::with_timeouts(config, CDP_HTTP_CONNECT_TIMEOUT, CDP_HTTP_TOTAL_TIMEOUT)
    }

    fn with_timeouts(
        config: &TransportConfig,
        connect_timeout: Duration,
        total_timeout: Duration,
    ) -> Result<Self, AppError> {
        let client = Client::builder()
            .connect_timeout(connect_timeout)
            .timeout(total_timeout)
            .build()
            .map_err(|_| AppError::new(ErrorKind::Internal, "Could not build CDP HTTP client"))?;
        Ok(Self {
            config: config.clone(),
            client,
        })
    }

    pub async fn fetch_targets(&self) -> Result<Vec<Target>, AppError> {
        let response = self
            .client
            .get(self.config.list_url())
            .send()
            .await
            .map_err(|err| self.target_list_request_error(err))?;

        if !response.status().is_success() {
            return Err(remote_status_error("CDP target list", response.status()).with_details(json!({
                "operation": "CDP target list",
                "http_failure_class": "remote_status",
                "cdp_host": self.config.host,
                "cdp_port": self.config.port,
                "status": response.status().as_u16(),
                "next_action_hint": "Run `tv status` to confirm the CDP endpoint, or run `tv launch` to restart TradingView Desktop with remote debugging enabled.",
            })));
        }

        response
            .json::<Vec<Target>>()
            .await
            .map_err(|err| map_cdp_http_error(err, "CDP target list response"))
    }

    pub async fn new_target_url(&self, url: &str) -> Result<Target, AppError> {
        if url.trim().is_empty() {
            return Err(AppError::new(
                ErrorKind::Validation,
                "CDP target URL must not be empty",
            ));
        }

        let response = self
            .client
            .put(self.config.new_target_url(url))
            .send()
            .await
            .map_err(|err| map_cdp_http_error(err, "CDP target creation"))?;

        let status = response.status();
        if !status.is_success() {
            return Err(
                remote_status_error("CDP target creation", status).with_details(json!({
                    "operation": "CDP target creation",
                    "http_failure_class": "remote_status",
                    "status": status.as_u16(),
                })),
            );
        }

        let target = response
            .json::<Target>()
            .await
            .map_err(|err| map_cdp_http_error(err, "CDP target creation response"))?;
        if target.id.trim().is_empty() || target.kind != "page" {
            return Err(AppError::new(
                ErrorKind::InternalApiUnavailable,
                "CDP target creation returned an unusable target",
            )
            .with_details(json!({
                "operation": "CDP target creation response",
                "http_failure_class": "payload_shape",
                "target_kind": target.kind,
                "target_id_present": !target.id.trim().is_empty(),
            })));
        }
        Ok(target)
    }

    pub async fn activate_target(&self, target_id: &str) -> Result<(), AppError> {
        let response = self
            .client
            .get(self.config.activate_url(target_id))
            .send()
            .await
            .map_err(|err| map_cdp_http_error(err, "CDP target activation"))?;
        if !response.status().is_success() {
            return Err(remote_status_error(
                "CDP target activation",
                response.status(),
            ));
        }
        Ok(())
    }

    pub async fn version_json(&self) -> Result<Option<Value>, AppError> {
        self.version_json_with_timeout(CDP_HTTP_TOTAL_TIMEOUT).await
    }

    pub async fn version_json_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Option<Value>, AppError> {
        let response = match self
            .client
            .get(self.config.version_url())
            .timeout(timeout)
            .send()
            .await
        {
            Ok(response) => response,
            Err(error) if error.is_timeout() => {
                return Err(map_cdp_http_error(error, "CDP version probe"));
            }
            Err(_) => return Ok(None),
        };
        if !response.status().is_success() {
            return Ok(None);
        }
        response
            .json::<Value>()
            .await
            .map(Some)
            .map_err(|err| map_cdp_http_error(err, "CDP version response"))
    }

    pub async fn discover_target(&self) -> Result<Target, AppError> {
        discover_target_from_targets(&self.config, self.fetch_targets().await?)
    }

    fn target_list_request_error(&self, error: reqwest::Error) -> AppError {
        let error = map_cdp_http_error(error, "CDP target list request");
        let failure_class = if error.kind == ErrorKind::Timeout {
            "timeout"
        } else {
            "connection"
        };
        error.with_details(json!({
            "operation": "CDP target list request",
            "http_failure_class": failure_class,
            "cdp_host": self.config.host,
            "cdp_port": self.config.port,
            "next_action_hint": "Run `tv status` to confirm the CDP endpoint, or run `tv launch` to start TradingView Desktop with remote debugging enabled.",
        }))
    }
}

fn map_cdp_http_error(error: reqwest::Error, operation: &str) -> AppError {
    if error.is_timeout() {
        http_failure(ErrorKind::Timeout, operation, "timeout", "timed out")
    } else if error.is_decode() {
        http_failure(
            ErrorKind::InternalApiUnavailable,
            operation,
            "payload",
            "returned an unusable payload",
        )
    } else {
        http_failure(
            ErrorKind::Connection,
            operation,
            "connection",
            "failed during HTTP transport",
        )
    }
}

fn remote_status_error(operation: &str, status: reqwest::StatusCode) -> AppError {
    AppError::new(
        ErrorKind::InternalApiUnavailable,
        format!("{operation} returned HTTP {status}"),
    )
    .with_details(json!({
        "operation": operation,
        "http_failure_class": "remote_status",
        "status": status.as_u16(),
    }))
}

fn http_failure(kind: ErrorKind, operation: &str, failure_class: &str, message: &str) -> AppError {
    AppError::new(kind, format!("{operation} {message}")).with_details(json!({
        "operation": operation,
        "http_failure_class": failure_class,
    }))
}

pub async fn fetch_targets(config: &TransportConfig) -> Result<Vec<Target>, AppError> {
    CdpHttpSession::new(config)?.fetch_targets().await
}

pub async fn new_target_url(config: &TransportConfig, url: &str) -> Result<Target, AppError> {
    CdpHttpSession::new(config)?.new_target_url(url).await
}

pub fn select_target(targets: &[Target]) -> TargetSelection {
    let chart_targets = page_targets_matching(targets, |target| {
        target.url.to_lowercase().contains("tradingview.com/chart")
    });
    match chart_targets.as_slice() {
        [target] => return TargetSelection::Selected((*target).clone()),
        targets if targets.len() > 1 => {
            return TargetSelection::Ambiguous(
                targets.iter().map(|target| (*target).clone()).collect(),
            );
        }
        _ => {}
    }

    let tradingview_targets = page_targets_matching(targets, |target| {
        target.url.to_lowercase().contains("tradingview") && !is_app_window_target(target)
    });
    match tradingview_targets.as_slice() {
        [target] => TargetSelection::Selected((*target).clone()),
        [] => TargetSelection::None,
        targets => {
            TargetSelection::Ambiguous(targets.iter().map(|target| (*target).clone()).collect())
        }
    }
}

pub async fn discover_target(config: &TransportConfig) -> Result<Target, AppError> {
    CdpHttpSession::new(config)?.discover_target().await
}

fn discover_target_from_targets(
    config: &TransportConfig,
    targets: Vec<Target>,
) -> Result<Target, AppError> {
    if let Some(target_id) = config.target_id.as_deref() {
        return targets
            .iter()
            .find(|target| target.id == target_id)
            .cloned()
            .ok_or_else(|| {
                AppError::new(
                    ErrorKind::Validation,
                    format!("Target id did not match any CDP target: {target_id}"),
                )
                .with_details(json!({
                    "target_id": target_id,
                    "target_selected_by": "cli_option",
                    "targets": targets_with_handoff(&targets),
                }))
            });
    }
    match select_target(&targets) {
        TargetSelection::Selected(target) => Ok(target),
        TargetSelection::None => Err(AppError::new(
            ErrorKind::Connection,
            "No TradingView chart target found",
        )
        .with_details(json!({
            "next_action_hint": "Run `tv tab list` to inspect available CDP targets, then retry with `tv --target-id <ID> <command>` when a chart target is available.",
            "targets": targets_with_handoff(&targets),
            "app_window_targets": targets_with_handoff(&app_window_targets(&targets)),
        }))),
        TargetSelection::Ambiguous(targets) => Err(AppError::new(
            ErrorKind::TargetAmbiguous,
            "Multiple TradingView chart targets found",
        )
        .with_details(json!({
            "next_action_hint": "Run `tv tab list`, choose the intended target, then retry as `tv --target-id <ID> <command>`.",
            "targets": targets_with_handoff(&targets),
        }))),
    }
}

fn page_targets_matching(targets: &[Target], predicate: impl Fn(&Target) -> bool) -> Vec<&Target> {
    targets
        .iter()
        .filter(|target| target.kind == "page")
        .filter(|target| predicate(target))
        .collect()
}

pub fn is_app_window_target(target: &Target) -> bool {
    target.kind == "page" && target.url.contains("/app/window/index.html")
}

pub fn is_new_tab_target(target: &Target) -> bool {
    target.kind == "page"
        && (target.url.contains("/app/new-tab/index.html")
            || target.title.eq_ignore_ascii_case("new tab")
            || target.title == "新規タブ")
}

pub fn is_screener_target(target: &Target) -> bool {
    target.kind == "page"
        && target
            .url
            .to_lowercase()
            .contains("tradingview.com/screener")
}

fn app_window_targets(targets: &[Target]) -> Vec<Target> {
    targets
        .iter()
        .filter(|target| is_app_window_target(target))
        .cloned()
        .collect()
}

pub fn target_cli_args(target_id: &str) -> [&str; 2] {
    ["--target-id", target_id]
}

fn target_with_handoff(target: &Target) -> serde_json::Value {
    json!({
        "id": target.id,
        "title": target_title_for_handoff(target),
        "type": target.kind,
        "url": target_url_for_handoff(target),
        "webSocketDebuggerUrl": target.web_socket_debugger_url,
        "target_cli_args": target_cli_args(&target.id),
    })
}

pub fn target_title_for_handoff(target: &Target) -> String {
    if is_app_window_target(target) {
        "TradingView app window".to_string()
    } else {
        target.title.clone()
    }
}

pub fn target_url_for_handoff(target: &Target) -> String {
    if is_app_window_target(target) {
        "file://<tradingview-app-window>".to_string()
    } else {
        target.url.clone()
    }
}

fn targets_with_handoff(targets: &[Target]) -> Vec<serde_json::Value> {
    targets.iter().map(target_with_handoff).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::loopback_fixture_lock;
    use std::sync::{Mutex, OnceLock};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
        time::{Duration, sleep, timeout},
    };

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    fn target(id: &str, url: &str) -> Target {
        Target {
            id: id.to_string(),
            title: id.to_string(),
            kind: "page".to_string(),
            url: url.to_string(),
            web_socket_debugger_url: Some(format!("ws://localhost/devtools/page/{id}")),
        }
    }

    #[test]
    fn selects_single_chart_target_first() {
        let targets = vec![
            target("a", "https://example.com"),
            target("b", "https://www.tradingview.com/chart/abc"),
        ];

        assert!(matches!(
            select_target(&targets),
            TargetSelection::Selected(Target { id, .. }) if id == "b"
        ));
    }

    #[test]
    fn falls_back_to_tradingview_page_target() {
        let targets = vec![target("a", "https://www.tradingview.com/markets")];

        assert!(matches!(
            select_target(&targets),
            TargetSelection::Selected(Target { id, .. }) if id == "a"
        ));
    }

    #[test]
    fn does_not_select_app_window_as_chart_target() {
        let targets = vec![target(
            "window",
            "file:///TradingView.app/Contents/Resources/app.asar/app/window/index.html",
        )];

        assert_eq!(select_target(&targets), TargetSelection::None);
        assert!(is_app_window_target(&targets[0]));
    }

    #[test]
    fn target_handoff_sanitizes_app_window_file_url() {
        let value = target_with_handoff(&target(
            "window",
            "file:///Users/example/TradingView.app/Contents/Resources/app.asar/app/window/index.html",
        ));

        assert_eq!(value["title"], "TradingView app window");
        assert_eq!(value["url"], "file://<tradingview-app-window>");
    }

    #[test]
    fn reports_ambiguous_chart_targets() {
        let targets = vec![
            target("a", "https://www.tradingview.com/chart/a"),
            target("b", "https://www.tradingview.com/chart/b"),
        ];

        assert!(matches!(
            select_target(&targets),
            TargetSelection::Ambiguous(targets) if targets.len() == 2
        ));
    }

    #[test]
    fn config_reads_optional_target_id_from_cli_option() {
        let config = TransportConfig::from_env_with_target_id(Some("cli-target")).unwrap();
        assert_eq!(config.target_id.as_deref(), Some("cli-target"));
    }

    #[test]
    fn default_host_uses_ipv4_loopback() {
        let config = TransportConfig::default();

        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.list_url(), "http://127.0.0.1:9222/json/list");
        assert_eq!(
            config.new_target_url("https://www.tradingview.com/screener/"),
            "http://127.0.0.1:9222/json/new?https://www.tradingview.com/screener/"
        );
    }

    #[test]
    fn env_host_and_port_override_default_endpoint() {
        let _guard = env_lock();
        unsafe {
            std::env::set_var("TV_CDP_HOST", "localhost");
            std::env::set_var("TV_CDP_PORT", "9333");
        }

        let config = TransportConfig::from_env().unwrap();

        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 9333);
        assert_eq!(config.list_url(), "http://localhost:9333/json/list");

        unsafe {
            std::env::remove_var("TV_CDP_HOST");
            std::env::remove_var("TV_CDP_PORT");
        }
    }

    #[test]
    fn target_handoff_uses_cli_args() {
        let value = target_with_handoff(&target("target-1", "https://www.tradingview.com/chart/a"));

        assert_eq!(value["target_cli_args"], json!(["--target-id", "target-1"]));
        assert!(value.get("target_env").is_none());
    }

    #[test]
    fn ignores_non_page_targets() {
        let targets = vec![Target {
            id: "worker".to_string(),
            title: "worker".to_string(),
            kind: "service_worker".to_string(),
            url: "https://www.tradingview.com/chart/a".to_string(),
            web_socket_debugger_url: None,
        }];

        assert_eq!(select_target(&targets), TargetSelection::None);
    }

    #[test]
    fn recognizes_full_page_screener_targets() {
        assert!(is_screener_target(&target(
            "screener",
            "https://www.tradingview.com/screener/"
        )));
        assert!(!is_screener_target(&target(
            "chart",
            "https://www.tradingview.com/chart/abc"
        )));
    }

    #[test]
    fn recognizes_tradingview_desktop_new_tab_targets() {
        assert!(is_new_tab_target(&Target {
            id: "new-tab".to_string(),
            title: "新規タブ".to_string(),
            kind: "page".to_string(),
            url: "file:///TradingView.app/Contents/Resources/app.asar/app/new-tab/index.html"
                .to_string(),
            web_socket_debugger_url: None,
        }));
        assert!(!is_new_tab_target(&target(
            "window",
            "file:///TradingView.app/Contents/Resources/app.asar/app/window/index.html"
        )));
    }

    #[tokio::test]
    async fn stalled_target_list_maps_to_timeout() {
        let _fixture_guard = loopback_fixture_lock().await;
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0u8; 512];
            let _ = stream.read(&mut request).await;
            sleep(Duration::from_millis(200)).await;
        });
        let config = TransportConfig {
            host: address.ip().to_string(),
            port: address.port(),
            target_id: None,
        };
        let session = CdpHttpSession::with_timeouts(
            &config,
            Duration::from_millis(25),
            Duration::from_millis(50),
        )
        .unwrap();

        let error = session.fetch_targets().await.unwrap_err();
        assert_eq!(error.kind, ErrorKind::Timeout);
        server.await.unwrap();
    }

    #[tokio::test]
    async fn session_reuses_connection_for_repeated_target_reads() {
        let _fixture_guard = loopback_fixture_lock().await;
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            for _ in 0..2 {
                read_http_request(&mut stream).await;
                stream
                    .write_all(
                        b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: keep-alive\r\n\r\n[]",
                    )
                    .await
                    .unwrap();
            }
            assert!(
                timeout(Duration::from_millis(100), listener.accept())
                    .await
                    .is_err()
            );
        });
        let config = TransportConfig {
            host: address.ip().to_string(),
            port: address.port(),
            target_id: None,
        };
        let session = CdpHttpSession::with_timeouts(
            &config,
            Duration::from_millis(25),
            Duration::from_millis(250),
        )
        .unwrap();

        assert!(session.fetch_targets().await.unwrap().is_empty());
        assert!(session.fetch_targets().await.unwrap().is_empty());
        server.await.unwrap();
    }

    #[tokio::test]
    async fn target_list_remote_status_maps_to_internal_api_unavailable() {
        let _fixture_guard = loopback_fixture_lock().await;
        for (status_line, status) in [
            ("429 Too Many Requests", 429u16),
            ("500 Internal Server Error", 500u16),
        ] {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            let server = tokio::spawn(async move {
                let (mut stream, _) = listener.accept().await.unwrap();
                read_http_request(&mut stream).await;
                let response = format!(
                    "HTTP/1.1 {status_line}\r\nContent-Length: 6\r\nConnection: close\r\n\r\nsecret"
                );
                stream.write_all(response.as_bytes()).await.unwrap();
            });
            let config = TransportConfig {
                host: address.ip().to_string(),
                port: address.port(),
                target_id: None,
            };
            let error = CdpHttpSession::new(&config)
                .unwrap()
                .fetch_targets()
                .await
                .unwrap_err();
            assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
            assert_eq!(error.exit_code(), 3);
            assert_eq!(error.details.as_ref().unwrap()["status"], status);
            assert!(
                !serde_json::to_string(&error.details)
                    .unwrap()
                    .contains("secret")
            );
            server.await.unwrap();
        }
    }

    #[tokio::test]
    async fn target_list_malformed_json_maps_to_internal_api_unavailable() {
        let _fixture_guard = loopback_fixture_lock().await;
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            read_http_request(&mut stream).await;
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\nConnection: close\r\n\r\n{")
                .await
                .unwrap();
        });
        let config = TransportConfig {
            host: address.ip().to_string(),
            port: address.port(),
            target_id: None,
        };
        let error = CdpHttpSession::new(&config)
            .unwrap()
            .fetch_targets()
            .await
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.exit_code(), 3);
        assert_eq!(error.details.unwrap()["http_failure_class"], "payload");
        server.await.unwrap();
    }

    #[tokio::test]
    async fn target_list_connection_refusal_remains_connection() {
        let _fixture_guard = loopback_fixture_lock().await;
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        drop(listener);
        let config = TransportConfig {
            host: address.ip().to_string(),
            port: address.port(),
            target_id: None,
        };
        let error = CdpHttpSession::new(&config)
            .unwrap()
            .fetch_targets()
            .await
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Connection);
        assert_eq!(error.exit_code(), 2);
    }

    #[tokio::test]
    async fn version_probe_malformed_success_maps_to_internal_api_unavailable() {
        let _fixture_guard = loopback_fixture_lock().await;
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            read_http_request(&mut stream).await;
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\nConnection: close\r\n\r\n{")
                .await
                .unwrap();
        });
        let config = TransportConfig {
            host: address.ip().to_string(),
            port: address.port(),
            target_id: None,
        };
        let error = CdpHttpSession::new(&config)
            .unwrap()
            .version_json()
            .await
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.exit_code(), 3);
        server.await.unwrap();
    }

    #[tokio::test]
    async fn version_probe_connection_refusal_remains_not_ready() {
        let _fixture_guard = loopback_fixture_lock().await;
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        drop(listener);
        let config = TransportConfig {
            host: address.ip().to_string(),
            port: address.port(),
            target_id: None,
        };
        assert_eq!(
            CdpHttpSession::new(&config)
                .unwrap()
                .version_json()
                .await
                .unwrap(),
            None
        );
    }

    async fn read_http_request(stream: &mut tokio::net::TcpStream) {
        let mut request = Vec::new();
        let mut buffer = [0u8; 256];
        loop {
            let count = stream.read(&mut buffer).await.unwrap();
            assert!(count > 0, "client closed before a complete request");
            request.extend_from_slice(&buffer[..count]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") {
                return;
            }
        }
    }
}
