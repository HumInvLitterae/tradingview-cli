use serde_json::{Value, json};

use tradingview_cdp::{
    self as transport, CdpClient, RuntimeEvaluator, TargetSelection, TransportConfig,
};
use tradingview_core::AppError;

use super::common::{CHART_API, merge_object};

pub async fn status(config: &TransportConfig) -> Result<Value, AppError> {
    let targets = transport::fetch_targets(config).await?;
    if config.target_id.is_some() {
        let target = transport::discover_target(config).await?;
        let target_handoff = target_with_handoff(&target);
        let mut data = json!({
            "connected": true,
            "cdp_connected": true,
            "target_id": target.id,
            "target_url": target.url,
            "target_title": target.title,
            "target_cli_args": transport::target_cli_args(&target.id),
            "target_selected_by": "cli_option",
            "cdp_host": config.host,
            "cdp_port": config.port,
            "chart_symbol": "unknown",
            "chart_resolution": "unknown",
            "chart_type": null,
            "api_available": false,
            "desktop_readiness": desktop_readiness_summary(
                config,
                &targets,
                "selected",
                Some(target_handoff),
                "The requested target was selected. Use the returned target_cli_args for follow-up chart-dependent commands."
            ),
        });
        let mut runtime = CdpClient::connect(&target).await?;
        if let Ok(chart) = chart_status(&mut runtime).await {
            merge_object(&mut data, chart);
        }
        return Ok(data);
    }
    let data = match transport::select_target(&targets) {
        TargetSelection::Selected(target) => {
            let target_handoff = target_with_handoff(&target);
            let mut data = json!({
                "connected": true,
                "cdp_connected": true,
                "target_id": target.id,
                "target_url": target.url,
                "target_title": target.title,
                "target_cli_args": transport::target_cli_args(&target.id),
                "target_selected_by": "auto",
                "cdp_host": config.host,
                "cdp_port": config.port,
                "chart_symbol": "unknown",
                "chart_resolution": "unknown",
                "chart_type": null,
                "api_available": false,
                "desktop_readiness": desktop_readiness_summary(
                    config,
                    &targets,
                    "selected",
                    Some(target_handoff),
                    "A single chart-compatible target was selected. Use target_cli_args when running follow-up chart-dependent commands."
                ),
            });
            let mut runtime = CdpClient::connect(&target).await?;
            if let Ok(chart) = chart_status(&mut runtime).await {
                merge_object(&mut data, chart);
            }
            data
        }
        TargetSelection::None => json!({
            "connected": false,
            "cdp_connected": false,
            "cdp_host": config.host,
            "cdp_port": config.port,
            "chart_symbol": "unknown",
            "chart_resolution": "unknown",
            "chart_type": null,
            "api_available": false,
            "error": "No TradingView chart target found",
            "candidates": targets_with_handoff(&targets),
            "desktop_readiness": desktop_readiness_summary(
                config,
                &targets,
                "none",
                None,
                "Run `tv tab list` to inspect available CDP targets. Open or select a TradingView chart target before running chart-dependent commands."
            ),
        }),
        TargetSelection::Ambiguous(candidates) => json!({
            "connected": false,
            "cdp_connected": false,
            "cdp_host": config.host,
            "cdp_port": config.port,
            "chart_symbol": "unknown",
            "chart_resolution": "unknown",
            "chart_type": null,
            "api_available": false,
            "error": "Multiple TradingView chart targets found",
            "candidates": targets_with_handoff(&candidates),
            "desktop_readiness": desktop_readiness_summary(
                config,
                &targets,
                "ambiguous",
                None,
                "Run `tv tab list`, choose the intended chart target, then retry as `tv --target-id <ID> <command>`."
            ),
        }),
    };
    Ok(data)
}

fn desktop_readiness_summary(
    config: &TransportConfig,
    targets: &[transport::Target],
    target_selection: &str,
    selected_target: Option<Value>,
    next_action_hint: &str,
) -> Value {
    let chart_targets = targets
        .iter()
        .filter(|target| is_chart_target(target))
        .collect::<Vec<_>>();
    let screener_targets = targets
        .iter()
        .filter(|target| transport::is_screener_target(target))
        .collect::<Vec<_>>();
    let app_window_targets = targets
        .iter()
        .filter(|target| transport::is_app_window_target(target))
        .collect::<Vec<_>>();
    json!({
        "cdp_endpoint": {
            "host": config.host,
            "port": config.port,
        },
        "target_selection": target_selection,
        "target_count": targets.len(),
        "chart_target_count": chart_targets.len(),
        "screener_target_count": screener_targets.len(),
        "app_window_target_count": app_window_targets.len(),
        "selected_target": selected_target,
        "chart_targets": targets_with_handoff_refs(&chart_targets),
        "screener_targets": targets_with_handoff_refs(&screener_targets),
        "app_window_targets": targets_with_handoff_refs(&app_window_targets),
        "next_action_hint": next_action_hint,
    })
}

fn is_chart_target(target: &transport::Target) -> bool {
    target.kind == "page" && target.url.to_lowercase().contains("tradingview.com/chart")
}

fn target_with_handoff(target: &transport::Target) -> Value {
    json!({
        "id": target.id,
        "title": transport::target_title_for_handoff(target),
        "type": target.kind,
        "url": transport::target_url_for_handoff(target),
        "target_cli_args": transport::target_cli_args(&target.id),
    })
}

fn targets_with_handoff(targets: &[transport::Target]) -> Vec<Value> {
    targets.iter().map(target_with_handoff).collect()
}

fn targets_with_handoff_refs(targets: &[&transport::Target]) -> Vec<Value> {
    targets
        .iter()
        .map(|target| target_with_handoff(target))
        .collect()
}

async fn chart_status(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var result = {{
                        chart_symbol: "unknown",
                        chart_resolution: "unknown",
                        chart_type: null,
                        api_available: false
                    }};
                    try {{
                        var chart = {CHART_API};
                        result.chart_symbol = chart.symbol();
                        result.chart_resolution = chart.resolution();
                        result.chart_type = chart.chartType();
                        result.api_available = true;
                    }} catch(e) {{
                        result.api_error = e && e.message ? e.message : String(e);
                    }}
                    return result;
                }})()
                "#
            ),
            false,
        )
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(id: &str, kind: &str, url: &str, title: &str) -> transport::Target {
        transport::Target {
            id: id.to_string(),
            title: title.to_string(),
            kind: kind.to_string(),
            url: url.to_string(),
            web_socket_debugger_url: None,
        }
    }

    #[test]
    fn desktop_readiness_summary_counts_target_kinds() {
        let config = TransportConfig {
            host: "127.0.0.1".to_string(),
            port: 9222,
            target_id: None,
        };
        let targets = vec![
            target(
                "chart",
                "page",
                "https://www.tradingview.com/chart/abc",
                "AAPL",
            ),
            target(
                "screener",
                "page",
                "https://www.tradingview.com/screener/",
                "Screener",
            ),
            target(
                "window",
                "page",
                "file:///Users/example/TradingView.app/Contents/Resources/app.asar/app/window/index.html",
                "index.html",
            ),
        ];

        let summary = desktop_readiness_summary(&config, &targets, "selected", None, "next action");

        assert_eq!(summary["target_count"], 3);
        assert_eq!(summary["chart_target_count"], 1);
        assert_eq!(summary["screener_target_count"], 1);
        assert_eq!(summary["app_window_target_count"], 1);
        assert_eq!(
            summary["app_window_targets"][0]["url"],
            "file://<tradingview-app-window>"
        );
        assert_eq!(
            summary["chart_targets"][0]["target_cli_args"],
            json!(["--target-id", "chart"])
        );
    }
}
