use serde_json::{Value, json};

use crate::{
    cdp::{CdpClient, RuntimeEvaluator},
    error::AppError,
    transport::{self, TargetSelection, TransportConfig},
};

use super::common::{CHART_API, merge_object};

pub async fn status(config: &TransportConfig) -> Result<Value, AppError> {
    let targets = transport::fetch_targets(config).await?;
    if config.target_id.is_some() {
        let target = transport::discover_target(config).await?;
        let mut data = json!({
            "connected": true,
            "cdp_connected": true,
            "target_id": target.id,
            "target_url": target.url,
            "target_title": target.title,
            "target_selected_by": "TV_CDP_TARGET_ID",
            "cdp_host": config.host,
            "cdp_port": config.port,
            "chart_symbol": "unknown",
            "chart_resolution": "unknown",
            "chart_type": null,
            "api_available": false,
        });
        let mut runtime = CdpClient::connect(&target).await?;
        if let Ok(chart) = chart_status(&mut runtime).await {
            merge_object(&mut data, chart);
        }
        return Ok(data);
    }
    let data = match transport::select_target(&targets) {
        TargetSelection::Selected(target) => {
            let mut data = json!({
                "connected": true,
                "cdp_connected": true,
                "target_id": target.id,
                "target_url": target.url,
                "target_title": target.title,
                "cdp_host": config.host,
                "cdp_port": config.port,
                "chart_symbol": "unknown",
                "chart_resolution": "unknown",
                "chart_type": null,
                "api_available": false,
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
            "candidates": targets,
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
            "candidates": candidates,
        }),
    };
    Ok(data)
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
