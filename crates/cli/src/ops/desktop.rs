use serde::Serialize;
use serde_json::Value;
use tokio::time::{Duration, sleep};

use tradingview_cdp::{self as transport, CdpClient, CdpHttpSession, RuntimeEvaluator, Target};
use tradingview_core::{AppError, ErrorKind};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AppTab {
    pub(crate) index: usize,
    pub(crate) title: String,
    pub(crate) active: bool,
    pub(crate) closable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AppWindowTarget {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) url: String,
    pub(crate) target_cli_args: Vec<String>,
}

pub(crate) fn app_window_targets_from_targets(targets: &[Target]) -> Vec<AppWindowTarget> {
    targets
        .iter()
        .filter(|target| transport::is_app_window_target(target))
        .map(|target| AppWindowTarget {
            id: target.id.clone(),
            title: transport::target_title_for_handoff(target),
            url: transport::target_url_for_handoff(target),
            target_cli_args: transport::target_cli_args(&target.id)
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
        })
        .collect()
}

pub(crate) fn app_window_target(targets: &[Target]) -> Result<&Target, AppError> {
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

pub(crate) async fn app_tabs_from_targets(targets: &[Target]) -> Vec<AppTab> {
    let Ok(target) = app_window_target(targets) else {
        return Vec::new();
    };

    let Ok(mut runtime) = CdpClient::connect(target).await else {
        return Vec::new();
    };

    read_app_tabs(&mut runtime).await.unwrap_or_default()
}

pub(crate) async fn read_app_tabs(
    runtime: &mut impl RuntimeEvaluator,
) -> Result<Vec<AppTab>, AppError> {
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

pub(crate) async fn create_new_app_tab(session: &CdpHttpSession) -> Result<(), AppError> {
    let targets = session.fetch_targets().await?;
    let app_target = app_window_target(&targets)?;
    let mut runtime = CdpClient::connect(app_target).await?;
    click_create_new_app_tab(&mut runtime).await
}

pub(crate) async fn click_create_new_app_tab(
    runtime: &mut impl RuntimeEvaluator,
) -> Result<(), AppError> {
    let result = runtime
        .evaluate(CLICK_CREATE_NEW_APP_TAB_EXPRESSION, false)
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
            "TradingView create-new-tab button was not found",
        )
        .with_details(result))
    }
}

pub(crate) async fn click_close_app_tab(
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

pub(crate) async fn wait_for_app_tab_update(milliseconds: u64) {
    sleep(Duration::from_millis(milliseconds)).await;
}

pub(crate) fn new_app_tabs(before: &[AppTab], after: &[AppTab]) -> Vec<AppTab> {
    after.iter().skip(before.len()).cloned().collect()
}

pub(crate) async fn current_new_tab_target(
    session: &CdpHttpSession,
) -> Result<Option<Target>, AppError> {
    let targets = session.fetch_targets().await?;
    Ok(first_new_tab_target(&targets).cloned())
}

fn first_new_tab_target(targets: &[Target]) -> Option<&Target> {
    targets
        .iter()
        .find(|target| transport::is_new_tab_target(target))
}

pub(crate) async fn wait_for_new_tab_target(
    session: &CdpHttpSession,
    wait_attempts: usize,
    wait_ms: u64,
    mut failure_details: Value,
) -> Result<Target, AppError> {
    for _ in 0..wait_attempts {
        if let Some(target) = current_new_tab_target(session).await? {
            return Ok(target);
        }
        sleep(Duration::from_millis(wait_ms)).await;
    }

    if let Some(object) = failure_details.as_object_mut() {
        object.insert("wait_attempts".to_string(), Value::from(wait_attempts));
    }

    Err(AppError::new(
        ErrorKind::InternalApiUnavailable,
        "TradingView new app tab target did not appear",
    )
    .with_details(failure_details))
}

const CLICK_CREATE_NEW_APP_TAB_EXPRESSION: &str = r#"
(function() {
    var button = document.querySelector("button.create-new-tab-button");
    if (!button) return { clicked: false, reason: "missing_create_new_tab_button" };
    button.click();
    return { clicked: true };
})()
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
            vec!["--target-id".to_string(), "window".to_string()]
        );
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
