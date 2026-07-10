use std::collections::HashSet;

use serde::Serialize;
use serde_json::{Value, json};

use tradingview_cdp::{self as transport, CdpClient, CdpHttpSession, Target, TransportConfig};
use tradingview_core::{AppError, ErrorKind};

use super::common::{desktop_backed_read_metadata, merge_object};
use super::desktop::{
    AppTab, AppWindowTarget, app_tabs_from_targets, app_window_target,
    app_window_targets_from_targets, click_close_app_tab, click_create_new_app_tab, new_app_tabs,
    read_app_tabs, wait_for_app_tab_update,
};

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
struct ScreenerTarget {
    index: usize,
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

    Ok(tab_list_payload(
        config,
        tabs,
        screener_targets,
        app_window_targets,
        app_tabs,
    ))
}

fn tab_list_payload(
    config: &TransportConfig,
    tabs: Vec<ChartTab>,
    screener_targets: Vec<ScreenerTarget>,
    app_window_targets: Vec<AppWindowTarget>,
    app_tabs: Vec<AppTab>,
) -> Value {
    let mut payload = json!({
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
        "desktop_readiness": tab_readiness_summary(
            tabs.len(),
            screener_targets.len(),
            app_window_targets.len(),
        ),
    });
    merge_object(
        &mut payload,
        desktop_backed_read_metadata("desktop_target_list", true),
    );
    payload
}

pub async fn tab_switch(config: &TransportConfig, index: usize) -> Result<Value, AppError> {
    let session = CdpHttpSession::new(config)?;
    let targets = session.fetch_targets().await?;
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

    session.activate_target(&tab.id).await.map_err(|err| {
        err.with_details(json!({
            "tab_id": tab.id,
            "index": index,
        }))
    })?;

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
    let session = CdpHttpSession::new(config)?;
    let targets = session.fetch_targets().await?;
    let tabs_before = chart_tabs_from_targets(&targets);
    let source = resolve_source_tab(&tabs_before, from)?.clone();
    let app_target = app_window_target(&targets)?;
    let mut app_runtime = CdpClient::connect(app_target).await?;
    let app_tabs_before = read_app_tabs(&mut app_runtime).await?;

    activate_tab(&session, &source).await?;
    click_create_new_app_tab(&mut app_runtime).await?;
    wait_for_app_tab_update(TAB_NEW_WAIT_MS).await;

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

    let targets_after = session.fetch_targets().await?;
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
    let session = CdpHttpSession::new(config)?;
    let targets = session.fetch_targets().await?;
    let chart_tabs_before = chart_tabs_from_targets(&targets);
    let app_target = app_window_target(&targets)?;
    let mut app_runtime = CdpClient::connect(app_target).await?;
    let app_tabs_before = read_app_tabs(&mut app_runtime).await?;
    let closed = validate_close_target(&app_tabs_before, index)?.clone();

    click_close_app_tab(&mut app_runtime, index).await?;
    wait_for_app_tab_update(TAB_CLOSE_WAIT_MS).await;

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

    let targets_after = session.fetch_targets().await?;
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
        .filter(|target| transport::is_screener_target(target))
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

async fn activate_tab(session: &CdpHttpSession, tab: &ChartTab) -> Result<(), AppError> {
    session.activate_target(&tab.id).await.map_err(|err| {
        err.with_details(json!({
            "tab_id": tab.id,
            "index": tab.index,
        }))
    })
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

fn tab_readiness_summary(
    chart_target_count: usize,
    screener_target_count: usize,
    app_window_target_count: usize,
) -> Value {
    let target_selection = match chart_target_count {
        0 => "none",
        1 => "selected",
        _ => "ambiguous",
    };
    let next_action_hint = match chart_target_count {
        0 => {
            "No chart target is available. Open a TradingView chart tab, then rerun `tv tab list`."
        }
        1 => {
            "Use tabs[0].target_cli_args for chart-dependent commands when you need an explicit target."
        }
        _ => {
            "Multiple chart targets are available. Choose the intended tabs[].target_cli_args and pass `tv --target-id <ID> <command>`."
        }
    };
    json!({
        "target_selection": target_selection,
        "chart_target_count": chart_target_count,
        "screener_target_count": screener_target_count,
        "app_window_target_count": app_window_target_count,
        "next_action_hint": next_action_hint,
    })
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
    fn tab_readiness_summary_marks_ambiguous_chart_targets() {
        let summary = tab_readiness_summary(2, 1, 1);

        assert_eq!(summary["target_selection"], "ambiguous");
        assert_eq!(summary["chart_target_count"], 2);
        assert!(
            summary["next_action_hint"]
                .as_str()
                .unwrap()
                .contains("--target-id")
        );
    }

    #[test]
    fn tab_list_payload_includes_source_metadata() {
        let config = TransportConfig {
            host: "127.0.0.1".to_string(),
            port: 9222,
            target_id: None,
        };

        let payload =
            tab_list_payload(&config, vec![chart_tab(0, "chart")], vec![], vec![], vec![]);

        assert_eq!(payload["source"], "desktop_target_list");
        assert_eq!(payload["source_category"], "desktop_backed_read");
        assert_eq!(payload["requires_desktop"], true);
        assert_eq!(payload["non_mutating"], true);
        assert_eq!(payload["tab_count"], 1);
    }

    #[test]
    fn validate_close_target_rejects_last_tab_and_out_of_range_index() {
        let single = vec![AppTab {
            index: 0,
            title: "a".to_string(),
            active: true,
            closable: true,
        }];
        let error = validate_close_target(&single, 0).unwrap_err();
        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(error.message.contains("last TradingView app tab"));

        let multiple = vec![
            AppTab {
                index: 0,
                title: "a".to_string(),
                active: true,
                closable: true,
            },
            AppTab {
                index: 1,
                title: "b".to_string(),
                active: false,
                closable: true,
            },
        ];
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
}
