use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::json;

use tradingview_core::{AppError, ErrorKind};

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

pub async fn fetch_targets(config: &TransportConfig) -> Result<Vec<Target>, AppError> {
    let response = reqwest::get(config.list_url())
        .await
        .map_err(|err| AppError::new(ErrorKind::Connection, err.to_string()))?;

    if !response.status().is_success() {
        return Err(AppError::new(
            ErrorKind::Connection,
            format!("CDP target list returned HTTP {}", response.status()),
        ));
    }

    response
        .json::<Vec<Target>>()
        .await
        .map_err(|err| AppError::new(ErrorKind::Connection, err.to_string()))
}

pub async fn new_target_url(config: &TransportConfig, url: &str) -> Result<Target, AppError> {
    if url.trim().is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "CDP target URL must not be empty",
        ));
    }

    let response = reqwest::Client::new()
        .put(config.new_target_url(url))
        .send()
        .await
        .map_err(|err| AppError::new(ErrorKind::Connection, err.to_string()))?;

    let status = response.status();
    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(AppError::new(
            ErrorKind::Connection,
            format!("CDP target creation returned HTTP {status}"),
        )
        .with_details(json!({
            "url": url,
            "status": status.as_u16(),
            "body": text,
        })));
    }

    let target = response
        .json::<Target>()
        .await
        .map_err(|err| AppError::new(ErrorKind::Connection, err.to_string()))?;
    if target.id.trim().is_empty() || target.kind != "page" {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "CDP target creation returned an unusable target",
        )
        .with_details(json!({
            "url": url,
            "target": target,
        })));
    }
    Ok(target)
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
    let targets = fetch_targets(config).await?;
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
    use std::sync::{Mutex, OnceLock};

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
}
