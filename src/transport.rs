use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::error::{AppError, ErrorKind};

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
    pub target_id_source: Option<TargetIdSource>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetIdSource {
    CliOption,
    Env,
}

impl TargetIdSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CliOption => "cli_option",
            Self::Env => "env",
        }
    }
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 9222,
            target_id: None,
            target_id_source: None,
        }
    }
}

impl TransportConfig {
    #[allow(dead_code)]
    pub fn from_env() -> Result<Self, AppError> {
        Self::from_env_with_target_id(None)
    }

    pub fn from_env_with_target_id(target_id: Option<&str>) -> Result<Self, AppError> {
        let host = std::env::var("TV_CDP_HOST").unwrap_or_else(|_| "localhost".to_string());
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
        let env_target_id = std::env::var("TV_CDP_TARGET_ID")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let (target_id, target_id_source) = if let Some(target_id) = cli_target_id {
            (Some(target_id), Some(TargetIdSource::CliOption))
        } else if let Some(target_id) = env_target_id {
            (Some(target_id), Some(TargetIdSource::Env))
        } else {
            (None, None)
        };
        Ok(Self {
            host,
            port,
            target_id,
            target_id_source,
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
        target.url.to_lowercase().contains("tradingview")
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
                    "target_selected_by": config.target_id_source.map(TargetIdSource::as_str),
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
        .with_details(json!({ "targets": targets_with_handoff(&targets) }))),
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

pub fn target_cli_args(target_id: &str) -> [&str; 2] {
    ["--target-id", target_id]
}

fn target_with_handoff(target: &Target) -> serde_json::Value {
    json!({
        "id": target.id,
        "title": target.title,
        "type": target.kind,
        "url": target.url,
        "webSocketDebuggerUrl": target.web_socket_debugger_url,
        "target_cli_args": target_cli_args(&target.id),
        "target_env": {
            "TV_CDP_TARGET_ID": target.id,
        },
    })
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
    fn config_reads_optional_target_id_from_env() {
        let _guard = env_lock();
        unsafe {
            std::env::set_var("TV_CDP_TARGET_ID", "target-1");
        }
        let config = TransportConfig::from_env().unwrap();
        assert_eq!(config.target_id.as_deref(), Some("target-1"));
        assert_eq!(config.target_id_source, Some(TargetIdSource::Env));
        unsafe {
            std::env::remove_var("TV_CDP_TARGET_ID");
        }
    }

    #[test]
    fn cli_target_id_overrides_env_target_id() {
        let _guard = env_lock();
        unsafe {
            std::env::set_var("TV_CDP_TARGET_ID", "env-target");
        }
        let config = TransportConfig::from_env_with_target_id(Some("cli-target")).unwrap();
        assert_eq!(config.target_id.as_deref(), Some("cli-target"));
        assert_eq!(config.target_id_source, Some(TargetIdSource::CliOption));
        unsafe {
            std::env::remove_var("TV_CDP_TARGET_ID");
        }
    }

    #[test]
    fn target_handoff_prefers_cli_args_but_keeps_env() {
        let value = target_with_handoff(&target("target-1", "https://www.tradingview.com/chart/a"));

        assert_eq!(value["target_cli_args"], json!(["--target-id", "target-1"]));
        assert_eq!(value["target_env"]["TV_CDP_TARGET_ID"], "target-1");
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
}
