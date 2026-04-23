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
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 9222,
        }
    }
}

impl TransportConfig {
    pub fn list_url(&self) -> String {
        format!("http://{}:{}/json/list", self.host, self.port)
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
    match select_target(&targets) {
        TargetSelection::Selected(target) => Ok(target),
        TargetSelection::None => Err(AppError::new(
            ErrorKind::Connection,
            "No TradingView chart target found",
        )
        .with_details(json!({ "targets": targets }))),
        TargetSelection::Ambiguous(targets) => Err(AppError::new(
            ErrorKind::TargetAmbiguous,
            "Multiple TradingView chart targets found",
        )
        .with_details(json!({ "targets": targets }))),
    }
}

fn page_targets_matching(targets: &[Target], predicate: impl Fn(&Target) -> bool) -> Vec<&Target> {
    targets
        .iter()
        .filter(|target| target.kind == "page")
        .filter(|target| predicate(target))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

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
