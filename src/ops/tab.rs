use serde::Serialize;
use serde_json::{Value, json};

use crate::{
    error::{AppError, ErrorKind},
    transport::{self, Target, TransportConfig},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ChartTab {
    index: usize,
    id: String,
    title: String,
    url: String,
    chart_id: Option<String>,
}

pub async fn tab_list(config: &TransportConfig) -> Result<Value, AppError> {
    let targets = transport::fetch_targets(config).await?;
    let tabs = chart_tabs_from_targets(&targets);

    Ok(json!({
        "tab_count": tabs.len(),
        "tabs": tabs,
        "cdp_host": config.host,
        "cdp_port": config.port,
    }))
}

pub async fn tab_switch(config: &TransportConfig, index: usize) -> Result<Value, AppError> {
    let targets = transport::fetch_targets(config).await?;
    let tabs = chart_tabs_from_targets(&targets);
    let tab = tabs.get(index).ok_or_else(|| {
        AppError::new(
            ErrorKind::Validation,
            format!("Tab index {index} out of range"),
        )
        .with_details(json!({
            "tab_count": tabs.len(),
            "requested_index": index,
        }))
    })?;

    let response = reqwest::get(config.activate_url(&tab.id))
        .await
        .map_err(|err| AppError::new(ErrorKind::Connection, err.to_string()))?;
    if !response.status().is_success() {
        return Err(AppError::new(
            ErrorKind::Connection,
            format!("CDP target activation returned HTTP {}", response.status()),
        )
        .with_details(json!({
            "tab_id": tab.id,
            "index": index,
        })));
    }

    Ok(json!({
        "action": "switched",
        "index": tab.index,
        "tab_id": tab.id,
        "chart_id": tab.chart_id,
        "title": tab.title,
        "url": tab.url,
    }))
}

fn chart_tabs_from_targets(targets: &[Target]) -> Vec<ChartTab> {
    targets
        .iter()
        .filter(|target| target.kind == "page")
        .filter(|target| target.url.to_lowercase().contains("tradingview.com/chart"))
        .enumerate()
        .map(|(index, target)| ChartTab {
            index,
            id: target.id.clone(),
            title: clean_title(&target.title),
            url: target.url.clone(),
            chart_id: chart_id_from_url(&target.url),
        })
        .collect()
}

fn clean_title(title: &str) -> String {
    title
        .strip_prefix("Live stock, index, futures, Forex and Bitcoin charts on ")
        .or_else(|| title.strip_prefix("Live stock charts on "))
        .unwrap_or(title)
        .to_string()
}

fn chart_id_from_url(url: &str) -> Option<String> {
    let marker = "/chart/";
    let start = url.find(marker)? + marker.len();
    let rest = &url[start..];
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let chart_id = &rest[..end];
    (!chart_id.is_empty()).then(|| chart_id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(id: &str, kind: &str, url: &str, title: &str) -> Target {
        Target {
            id: id.to_string(),
            title: title.to_string(),
            kind: kind.to_string(),
            url: url.to_string(),
            web_socket_debugger_url: None,
        }
    }

    #[test]
    fn chart_tabs_from_targets_filters_and_indexes_chart_pages() {
        let targets = vec![
            target(
                "worker",
                "service_worker",
                "https://www.tradingview.com/chart/worker",
                "worker",
            ),
            target("other", "page", "https://example.com", "example"),
            target(
                "a",
                "page",
                "https://www.tradingview.com/chart/abcd1234/?symbol=NASDAQ:AAPL",
                "Live stock charts on AAPL",
            ),
            target(
                "b",
                "page",
                "https://www.tradingview.com/chart/efgh5678",
                "LWLG chart",
            ),
        ];

        let tabs = chart_tabs_from_targets(&targets);

        assert_eq!(tabs.len(), 2);
        assert_eq!(tabs[0].index, 0);
        assert_eq!(tabs[0].id, "a");
        assert_eq!(tabs[0].title, "AAPL");
        assert_eq!(tabs[0].chart_id.as_deref(), Some("abcd1234"));
        assert_eq!(tabs[1].index, 1);
        assert_eq!(tabs[1].chart_id.as_deref(), Some("efgh5678"));
    }

    #[test]
    fn chart_id_from_url_handles_missing_chart_id() {
        assert_eq!(
            chart_id_from_url("https://www.tradingview.com/chart/abcd1234/?symbol=NASDAQ:AAPL")
                .as_deref(),
            Some("abcd1234")
        );
        assert_eq!(
            chart_id_from_url("https://www.tradingview.com/markets"),
            None
        );
    }
}
