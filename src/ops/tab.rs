use std::collections::HashSet;

use serde::Serialize;
use serde_json::{Value, json};

use crate::{
    cdp::{CdpClient, RuntimeEvaluator},
    transport::{self, Target, TransportConfig},
};
use tradingview_core::{AppError, ErrorKind};

const TAB_NEW_WAIT_MS: u64 = 2_000;
const TAB_CLOSE_WAIT_MS: u64 = 1_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ChartTab {
    index: usize,
    id: String,
    title: String,
    url: String,
    chart_id: Option<String>,
    target_cli_args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct AppTab {
    index: usize,
    title: String,
    active: bool,
    closable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ScreenerTarget {
    index: usize,
    id: String,
    title: String,
    url: String,
    target_cli_args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct AppWindowTarget {
    id: String,
    title: String,
    url: String,
    target_cli_args: Vec<String>,
}

pub async fn tab_list(config: &TransportConfig) -> Result<Value, AppError> {
    let targets = transport::fetch_targets(config).await?;
    let tabs = chart_tabs_from_targets(&targets);
    let screener_targets = screener_targets_from_targets(&targets);
    let app_window_targets = app_window_targets_from_targets(&targets);
    let app_tabs = app_tabs_from_targets(&targets).await;

    Ok(json!({
        "tab_count": tabs.len(),
        "tabs": tabs,
        "screener_target_count": screener_targets.len(),
        "screener_targets": screener_targets,
        "app_window_target_count": app_window_targets.len(),
        "app_window_targets": app_window_targets,
        "app_tab_count": app_tabs.len(),
        "app_tabs": app_tabs,
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

    Ok(tab_switch_payload(tab))
}

fn tab_switch_payload(tab: &ChartTab) -> Value {
    json!({
        "action": "switched",
        "index": tab.index,
        "tab_id": tab.id,
        "target_id": tab.id,
        "target_cli_args": target_cli_args(&tab.id),
        "next_command_hint": format!("tv --target-id {} <command>", tab.id),
        "chart_id": tab.chart_id,
        "title": tab.title,
        "url": tab.url,
    })
}

pub async fn tab_new(config: &TransportConfig, from: Option<usize>) -> Result<Value, AppError> {
    let targets = transport::fetch_targets(config).await?;
    let tabs_before = chart_tabs_from_targets(&targets);
    let source = resolve_source_tab(&tabs_before, from)?.clone();
    let app_target = app_window_target(&targets)?;
    let mut app_runtime = CdpClient::connect(app_target).await?;
    let app_tabs_before = read_app_tabs(&mut app_runtime).await?;

    activate_tab(config, &source).await?;
    click_new_app_tab(&mut app_runtime).await?;
    wait_for_tab_update(TAB_NEW_WAIT_MS).await;

    let app_tabs_after = read_app_tabs(&mut app_runtime).await?;
    if app_tabs_after.len() <= app_tabs_before.len() {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "TradingView did not open a new app tab",
        )
        .with_details(json!({
            "tabs_before": app_tabs_before.len(),
            "tabs_after": app_tabs_after.len(),
            "chart_tabs_before": tabs_before.len(),
            "source_index": source.index,
        })));
    }

    let targets_after = transport::fetch_targets(config).await?;
    let tabs_after = chart_tabs_from_targets(&targets_after);
    let new_app_tabs = new_app_tabs(&app_tabs_before, &app_tabs_after);

    let new_tabs = new_tabs(&tabs_before, &tabs_after);

    Ok(json!({
        "action": "new_tab_opened",
        "source_index": source.index,
        "source_tab": source,
        "tabs_before": app_tabs_before.len(),
        "tabs_after": app_tabs_after.len(),
        "app_tabs_before": app_tabs_before.len(),
        "app_tabs_after": app_tabs_after.len(),
        "new_app_tabs": new_app_tabs,
        "chart_tabs_before": tabs_before.len(),
        "chart_tabs_after": tabs_after.len(),
        "new_tabs": new_tabs,
    }))
}

pub async fn tab_close(config: &TransportConfig, index: usize) -> Result<Value, AppError> {
    let targets = transport::fetch_targets(config).await?;
    let chart_tabs_before = chart_tabs_from_targets(&targets);
    let app_target = app_window_target(&targets)?;
    let mut app_runtime = CdpClient::connect(app_target).await?;
    let app_tabs_before = read_app_tabs(&mut app_runtime).await?;
    let closed = validate_close_target(&app_tabs_before, index)?.clone();

    click_close_app_tab(&mut app_runtime, index).await?;
    wait_for_tab_update(TAB_CLOSE_WAIT_MS).await;

    let app_tabs_after = read_app_tabs(&mut app_runtime).await?;
    if app_tabs_after.len() >= app_tabs_before.len() {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "TradingView did not close the requested app tab",
        )
        .with_details(json!({
            "tabs_before": app_tabs_before.len(),
            "tabs_after": app_tabs_after.len(),
            "closed_index": closed.index,
        })));
    }

    let targets_after = transport::fetch_targets(config).await?;
    let chart_tabs_after = chart_tabs_from_targets(&targets_after);

    Ok(json!({
        "action": "tab_closed",
        "closed_index": closed.index,
        "closed_tab": closed,
        "tabs_before": app_tabs_before.len(),
        "tabs_after": app_tabs_after.len(),
        "app_tabs_before": app_tabs_before.len(),
        "app_tabs_after": app_tabs_after.len(),
        "chart_tabs_before": chart_tabs_before.len(),
        "chart_tabs_after": chart_tabs_after.len(),
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
            target_cli_args: target_cli_args(&target.id),
        })
        .collect()
}

fn screener_targets_from_targets(targets: &[Target]) -> Vec<ScreenerTarget> {
    targets
        .iter()
        .filter(|target| target.kind == "page")
        .filter(|target| is_screener_url(&target.url))
        .enumerate()
        .map(|(index, target)| ScreenerTarget {
            index,
            id: target.id.clone(),
            title: clean_title(&target.title),
            url: target.url.clone(),
            target_cli_args: target_cli_args(&target.id),
        })
        .collect()
}

fn is_screener_url(url: &str) -> bool {
    url.to_lowercase().contains("tradingview.com/screener")
}

fn app_window_targets_from_targets(targets: &[Target]) -> Vec<AppWindowTarget> {
    targets
        .iter()
        .filter(|target| transport::is_app_window_target(target))
        .map(|target| AppWindowTarget {
            id: target.id.clone(),
            title: transport::target_title_for_handoff(target),
            url: transport::target_url_for_handoff(target),
            target_cli_args: target_cli_args(&target.id),
        })
        .collect()
}

async fn activate_tab(config: &TransportConfig, tab: &ChartTab) -> Result<(), AppError> {
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
            "index": tab.index,
        })));
    }
    Ok(())
}

fn app_window_target(targets: &[Target]) -> Result<&Target, AppError> {
    targets
        .iter()
        .find(|target| transport::is_app_window_target(target))
        .ok_or_else(|| {
            AppError::new(
                ErrorKind::InternalApiUnavailable,
                "TradingView app window target was not found",
            )
        })
}

async fn app_tabs_from_targets(targets: &[Target]) -> Vec<AppTab> {
    let Some(target) = targets
        .iter()
        .find(|target| transport::is_app_window_target(target))
    else {
        return Vec::new();
    };

    let Ok(mut runtime) = CdpClient::connect(target).await else {
        return Vec::new();
    };

    read_app_tabs(&mut runtime).await.unwrap_or_default()
}

fn resolve_source_tab(tabs: &[ChartTab], from: Option<usize>) -> Result<&ChartTab, AppError> {
    match from {
        Some(index) => tabs.get(index).ok_or_else(|| {
            AppError::new(
                ErrorKind::Validation,
                format!("Tab index {index} out of range"),
            )
            .with_details(json!({
                "tab_count": tabs.len(),
                "requested_index": index,
            }))
        }),
        None if tabs.len() == 1 => Ok(&tabs[0]),
        None if tabs.is_empty() => Err(AppError::new(
            ErrorKind::Validation,
            "No TradingView chart tabs available to open from",
        )
        .with_details(json!({
            "tab_count": tabs.len(),
        }))),
        None => Err(AppError::new(
            ErrorKind::Validation,
            "Multiple TradingView chart tabs are open; pass --from <INDEX>",
        )
        .with_details(json!({
            "tab_count": tabs.len(),
        }))),
    }
}

fn validate_close_target(tabs: &[AppTab], index: usize) -> Result<&AppTab, AppError> {
    if tabs.len() <= 1 {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Refusing to close the last TradingView app tab",
        )
        .with_details(json!({
            "tab_count": tabs.len(),
            "requested_index": index,
        })));
    }

    tabs.get(index).ok_or_else(|| {
        AppError::new(
            ErrorKind::Validation,
            format!("Tab index {index} out of range"),
        )
        .with_details(json!({
            "tab_count": tabs.len(),
            "requested_index": index,
        }))
    })
}

async fn read_app_tabs(runtime: &mut impl RuntimeEvaluator) -> Result<Vec<AppTab>, AppError> {
    let value = runtime
        .evaluate(
            r#"
            (function() {
                return Array.from(document.querySelectorAll(".tabs-container .tab")).map(function(tab, index) {
                    var title = "";
                    var titleNode = tab.querySelector(".tab-title");
                    if (titleNode) title = (titleNode.textContent || "").trim();
                    return {
                        index: index,
                        title: title,
                        active: tab.classList.contains("active"),
                        closable: !!tab.querySelector(".tab-close-button-container button")
                    };
                });
            })()
            "#,
            false,
        )
        .await?;
    app_tabs_from_value(&value)
}

fn app_tabs_from_value(value: &Value) -> Result<Vec<AppTab>, AppError> {
    let rows = value.as_array().ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "TradingView app tabs payload was not an array",
        )
        .with_details(value.clone())
    })?;

    rows.iter()
        .map(|row| {
            Ok(AppTab {
                index: row.get("index").and_then(Value::as_u64).unwrap_or(0) as usize,
                title: row
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                active: row.get("active").and_then(Value::as_bool).unwrap_or(false),
                closable: row
                    .get("closable")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            })
        })
        .collect()
}

async fn click_new_app_tab(runtime: &mut impl RuntimeEvaluator) -> Result<(), AppError> {
    let clicked = runtime
        .evaluate(
            r#"
            (function() {
                var button = document.querySelector("button.create-new-tab-button");
                if (!button) return false;
                button.click();
                return true;
            })()
            "#,
            false,
        )
        .await?
        .as_bool()
        .unwrap_or(false);

    if clicked {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "TradingView create-new-tab button was not found",
        ))
    }
}

async fn click_close_app_tab(
    runtime: &mut impl RuntimeEvaluator,
    index: usize,
) -> Result<(), AppError> {
    let result = runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var tabs = Array.from(document.querySelectorAll(".tabs-container .tab"));
                    var tab = tabs[{index}];
                    if (!tab) return {{ clicked: false, reason: "missing_tab" }};
                    var titleNode = tab.querySelector(".tab-title");
                    var button = tab.querySelector(".tab-close-button-container button");
                    if (!button) {{
                        return {{
                            clicked: false,
                            reason: "missing_close_button",
                            title: titleNode ? (titleNode.textContent || "").trim() : ""
                        }};
                    }}
                    button.click();
                    return {{
                        clicked: true,
                        title: titleNode ? (titleNode.textContent || "").trim() : ""
                    }};
                }})()
                "#
            ),
            false,
        )
        .await?;

    if result
        .get("clicked")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "TradingView app tab close button was not found",
        )
        .with_details(result))
    }
}

async fn wait_for_tab_update(milliseconds: u64) {
    tokio::time::sleep(std::time::Duration::from_millis(milliseconds)).await;
}

fn new_tabs(before: &[ChartTab], after: &[ChartTab]) -> Vec<ChartTab> {
    let before_ids = before
        .iter()
        .map(|tab| tab.id.as_str())
        .collect::<HashSet<_>>();
    after
        .iter()
        .filter(|tab| !before_ids.contains(tab.id.as_str()))
        .cloned()
        .collect()
}

fn new_app_tabs(before: &[AppTab], after: &[AppTab]) -> Vec<AppTab> {
    after.iter().skip(before.len()).cloned().collect()
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

fn target_cli_args(target_id: &str) -> Vec<String> {
    vec!["--target-id".to_string(), target_id.to_string()]
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

    fn app_tab(index: usize, title: &str) -> AppTab {
        AppTab {
            index,
            title: title.to_string(),
            active: index == 0,
            closable: true,
        }
    }

    fn chart_tab(index: usize, id: &str) -> ChartTab {
        ChartTab {
            index,
            id: id.to_string(),
            title: format!("tab {id}"),
            url: format!("https://www.tradingview.com/chart/{id}"),
            chart_id: Some(id.to_string()),
            target_cli_args: target_cli_args(id),
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
        assert_eq!(tabs[0].target_cli_args, target_cli_args("a"));
        assert_eq!(tabs[1].index, 1);
        assert_eq!(tabs[1].chart_id.as_deref(), Some("efgh5678"));
    }

    #[test]
    fn screener_targets_include_explicit_handoff() {
        let targets = vec![
            target(
                "chart",
                "page",
                "https://www.tradingview.com/chart/abcd1234/",
                "Live stock charts on AAPL",
            ),
            target(
                "screener",
                "page",
                "https://www.tradingview.com/screener/qq4NFtlO/",
                "US stocks test copy",
            ),
            target(
                "window",
                "page",
                "file:///TradingView.app/Contents/Resources/app.asar/app/window/index.html",
                "index.html",
            ),
            target(
                "worker",
                "worker",
                "https://www.tradingview.com/screener/worker",
                "worker",
            ),
        ];

        let screener_targets = screener_targets_from_targets(&targets);

        assert_eq!(screener_targets.len(), 1);
        assert_eq!(screener_targets[0].id, "screener");
        assert_eq!(
            screener_targets[0].target_cli_args,
            target_cli_args("screener")
        );
    }

    #[test]
    fn app_window_targets_include_explicit_handoff() {
        let targets = vec![
            target(
                "window",
                "page",
                "file:///TradingView.app/Contents/Resources/app.asar/app/window/index.html",
                "index.html",
            ),
            target(
                "chart",
                "page",
                "https://www.tradingview.com/chart/abcd1234/",
                "Live stock charts on AAPL",
            ),
        ];

        let app_window_targets = app_window_targets_from_targets(&targets);

        assert_eq!(app_window_targets.len(), 1);
        assert_eq!(app_window_targets[0].id, "window");
        assert_eq!(app_window_targets[0].title, "TradingView app window");
        assert_eq!(app_window_targets[0].url, "file://<tradingview-app-window>");
        assert_eq!(
            app_window_targets[0].target_cli_args,
            target_cli_args("window")
        );
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

    #[test]
    fn resolve_source_tab_defaults_only_when_single_chart_tab_exists() {
        let single = vec![chart_tab(0, "a")];
        assert_eq!(resolve_source_tab(&single, None).unwrap().id, "a");

        let multiple = vec![chart_tab(0, "a"), chart_tab(1, "b")];
        let error = resolve_source_tab(&multiple, None).unwrap_err();
        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(
            error
                .message
                .contains("Multiple TradingView chart tabs are open")
        );
    }

    #[test]
    fn resolve_source_tab_validates_explicit_index() {
        let tabs = vec![chart_tab(0, "a")];
        assert_eq!(resolve_source_tab(&tabs, Some(0)).unwrap().id, "a");

        let error = resolve_source_tab(&tabs, Some(1)).unwrap_err();
        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(error.message.contains("out of range"));
    }

    #[test]
    fn tab_switch_payload_includes_next_target_handoff() {
        let tab = chart_tab(2, "target-2");

        let payload = tab_switch_payload(&tab);

        assert_eq!(payload["action"], "switched");
        assert_eq!(payload["tab_id"], "target-2");
        assert_eq!(payload["target_id"], "target-2");
        assert_eq!(
            payload["target_cli_args"],
            json!(["--target-id", "target-2"])
        );
        assert!(payload.get("target_env").is_none());
        assert_eq!(
            payload["next_command_hint"],
            "tv --target-id target-2 <command>"
        );
        assert_eq!(payload["chart_id"], "target-2");
    }

    #[test]
    fn validate_close_target_rejects_last_tab_and_out_of_range_index() {
        let single = vec![app_tab(0, "a")];
        let error = validate_close_target(&single, 0).unwrap_err();
        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(error.message.contains("last TradingView app tab"));

        let multiple = vec![app_tab(0, "a"), app_tab(1, "b")];
        assert_eq!(validate_close_target(&multiple, 1).unwrap().title, "b");
        let error = validate_close_target(&multiple, 2).unwrap_err();
        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(error.message.contains("out of range"));
    }

    #[test]
    fn new_tabs_returns_targets_not_seen_before() {
        let before = vec![chart_tab(0, "a")];
        let after = vec![chart_tab(0, "a"), chart_tab(1, "b")];

        let created = new_tabs(&before, &after);

        assert_eq!(created.len(), 1);
        assert_eq!(created[0].id, "b");
    }

    #[test]
    fn app_tabs_from_value_parses_tab_rows() {
        let tabs = app_tabs_from_value(&json!([
            {"index": 0, "title": "LWLG", "active": true, "closable": true},
            {"index": 1, "title": "New Tab", "active": false, "closable": true}
        ]))
        .unwrap();

        assert_eq!(tabs.len(), 2);
        assert_eq!(tabs[0].title, "LWLG");
        assert!(tabs[0].active);
        assert_eq!(tabs[1].index, 1);
    }

    #[test]
    fn new_app_tabs_returns_rows_appended_after_new_tab_click() {
        let before = vec![app_tab(0, "LWLG")];
        let after = vec![app_tab(0, "LWLG"), app_tab(1, "New Tab")];

        let created = new_app_tabs(&before, &after);

        assert_eq!(created.len(), 1);
        assert_eq!(created[0].title, "New Tab");
    }
}
