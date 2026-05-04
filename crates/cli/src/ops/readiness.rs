use serde_json::{Value, json};

use tradingview_cdp::{self as transport, CdpClient, Target, TargetSelection, TransportConfig};
use tradingview_core::AppError;

use super::{ohlcv_bars, state};

const READINESS_SOURCE: &str = "desktop_readiness";
const READINESS_SOURCE_CATEGORY: &str = "desktop_backed_read";

pub async fn readiness(config: &TransportConfig) -> Result<Value, AppError> {
    let targets = transport::fetch_targets(config).await?;
    let selection = select_readiness_target(config, &targets);
    let mut payload = readiness_base_payload(config, &targets, &selection);

    let Some(target) = selection.selected_target() else {
        return Ok(payload);
    };

    match CdpClient::connect(target).await {
        Ok(mut runtime) => {
            let chart_readiness = match state(&mut runtime).await {
                Ok(chart_state) => chart_readiness_from_state(&chart_state),
                Err(err) => readiness_from_error("chart_state_read", err),
            };
            let bars_readiness = match ohlcv_bars(&mut runtime, Some(1)).await {
                Ok(bars) => bars_readiness_from_bars(&bars),
                Err(err) => readiness_from_error("ohlcv_bars_read", err),
            };
            let ready = is_ready(&chart_readiness) && is_ready(&bars_readiness);
            insert_readiness_details(&mut payload, chart_readiness, bars_readiness, ready);
        }
        Err(err) => {
            let chart_readiness = readiness_from_error("runtime_connect", err);
            let bars_readiness = skipped_bars_readiness("runtime_unavailable");
            insert_readiness_details(&mut payload, chart_readiness, bars_readiness, false);
        }
    }

    Ok(payload)
}

#[derive(Debug, Clone)]
enum ReadinessSelection<'a> {
    Selected {
        target: &'a Target,
        selected_by: &'static str,
    },
    None,
    Ambiguous(Vec<&'a Target>),
    MissingCliTarget(String),
}

impl<'a> ReadinessSelection<'a> {
    fn selected_target(&self) -> Option<&'a Target> {
        match self {
            Self::Selected { target, .. } => Some(target),
            Self::None | Self::Ambiguous(_) | Self::MissingCliTarget(_) => None,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Selected { selected_by, .. } => selected_by,
            Self::None => "none",
            Self::Ambiguous(_) => "ambiguous",
            Self::MissingCliTarget(_) => "cli_option",
        }
    }
}

fn select_readiness_target<'a>(
    config: &TransportConfig,
    targets: &'a [Target],
) -> ReadinessSelection<'a> {
    if let Some(target_id) = config.target_id.as_deref() {
        return targets
            .iter()
            .find(|target| target.id == target_id)
            .map(|target| ReadinessSelection::Selected {
                target,
                selected_by: "cli_option",
            })
            .unwrap_or_else(|| ReadinessSelection::MissingCliTarget(target_id.to_string()));
    }

    match transport::select_target(targets) {
        TargetSelection::Selected(target) => {
            let target_id = target.id;
            let target = targets
                .iter()
                .find(|candidate| candidate.id == target_id)
                .expect("selected target should come from target list");
            ReadinessSelection::Selected {
                target,
                selected_by: "selected",
            }
        }
        TargetSelection::None => ReadinessSelection::None,
        TargetSelection::Ambiguous(candidates) => {
            let candidate_ids = candidates
                .iter()
                .map(|target| target.id.as_str())
                .collect::<Vec<_>>();
            ReadinessSelection::Ambiguous(
                targets
                    .iter()
                    .filter(|target| candidate_ids.contains(&target.id.as_str()))
                    .collect(),
            )
        }
    }
}

fn readiness_base_payload(
    config: &TransportConfig,
    targets: &[Target],
    selection: &ReadinessSelection<'_>,
) -> Value {
    let selected_target = selection.selected_target().map(target_with_handoff);
    let chart_targets = chart_targets(targets);
    let screener_targets = screener_targets(targets);
    let app_window_targets = app_window_targets(targets);
    let next_action_hint = next_action_hint(selection);
    let chart_readiness = match selection {
        ReadinessSelection::None => missing_chart_readiness("no_chart_target"),
        ReadinessSelection::Ambiguous(_) => missing_chart_readiness("ambiguous_chart_target"),
        ReadinessSelection::MissingCliTarget(_) => missing_chart_readiness("target_id_not_found"),
        ReadinessSelection::Selected { .. } => missing_chart_readiness("not_checked"),
    };
    let bars_readiness = match selection {
        ReadinessSelection::Selected { .. } => skipped_bars_readiness("not_checked"),
        _ => skipped_bars_readiness("no_selected_chart_target"),
    };

    let mut payload = json!({
        "source": READINESS_SOURCE,
        "source_category": READINESS_SOURCE_CATEGORY,
        "requires_desktop": true,
        "non_mutating": true,
        "ready": false,
        "cdp": {
            "endpoint": {
                "host": config.host,
                "port": config.port,
            },
            "connected": true,
            "target_count": targets.len(),
        },
        "target_selection": selection.label(),
        "selected_target": selected_target,
        "chart_target_count": chart_targets.len(),
        "screener_target_count": screener_targets.len(),
        "app_window_target_count": app_window_targets.len(),
        "chart_targets": targets_with_handoff_refs(&chart_targets),
        "screener_targets": targets_with_handoff_refs(&screener_targets),
        "app_window_targets": targets_with_handoff_refs(&app_window_targets),
        "chart_readiness": chart_readiness,
        "bars_readiness": bars_readiness,
        "next_action_hint": next_action_hint,
        "screenshot_hint": "If structured readiness does not explain the visible state, run `tv screenshot --region chart --output <PATH>` and inspect the saved image.",
    });

    if let ReadinessSelection::Ambiguous(candidates) = selection {
        payload["ambiguous_targets"] = json!(targets_with_handoff_refs(candidates));
    }
    if let ReadinessSelection::MissingCliTarget(target_id) = selection {
        payload["requested_target_id"] = json!(target_id);
    }

    payload
}

fn insert_readiness_details(
    payload: &mut Value,
    chart_readiness: Value,
    bars_readiness: Value,
    ready: bool,
) {
    payload["ready"] = json!(ready);
    payload["chart_readiness"] = chart_readiness;
    payload["bars_readiness"] = bars_readiness;
    payload["next_action_hint"] = json!(if ready {
        "Desktop chart target is ready. Chart-dependent read commands such as `tv state`, `tv ohlcv --count 1`, and bounded `tv stream ...` can use the selected target."
    } else {
        "Desktop target was found, but chart or bars readiness is incomplete. Inspect chart_readiness and bars_readiness, then retry with target_cli_args or capture a chart screenshot if the visual state is unclear."
    });
}

fn chart_readiness_from_state(chart_state: &Value) -> Value {
    let mut readiness = chart_state
        .get("chart_readiness")
        .cloned()
        .unwrap_or_else(|| missing_chart_readiness("missing_chart_readiness"));
    let ready = readiness
        .get("chart_api_available")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        && readiness
            .get("bars_available")
            .and_then(Value::as_bool)
            .unwrap_or(false);
    if let Some(object) = readiness.as_object_mut() {
        object.insert("ready".to_string(), json!(ready));
        if let Some(symbol) = chart_state.get("symbol") {
            object.insert("symbol".to_string(), symbol.clone());
        }
        if let Some(resolution) = chart_state.get("resolution") {
            object.insert("resolution".to_string(), resolution.clone());
        }
    }
    readiness
}

fn bars_readiness_from_bars(bars: &Value) -> Value {
    let last_bar = bars
        .get("bars")
        .and_then(Value::as_array)
        .and_then(|bars| bars.last())
        .cloned();
    json!({
        "ready": true,
        "phase": "ohlcv_bars_read",
        "symbol": bars.get("symbol").cloned().unwrap_or(Value::Null),
        "resolution": bars.get("resolution").cloned().unwrap_or(Value::Null),
        "bar_count": bars.get("bar_count").cloned().unwrap_or(Value::Null),
        "last_bar": last_bar,
        "source": bars.get("source").cloned().unwrap_or(Value::Null),
    })
}

fn readiness_from_error(phase: &str, err: AppError) -> Value {
    json!({
        "ready": false,
        "phase": phase,
        "error": {
            "kind": err.kind,
            "message": err.message,
            "details": err.details,
        }
    })
}

fn missing_chart_readiness(reason: &str) -> Value {
    json!({
        "ready": false,
        "reason": reason,
        "chart_api_available": false,
        "bars_available": false,
    })
}

fn skipped_bars_readiness(reason: &str) -> Value {
    json!({
        "ready": false,
        "skipped": true,
        "reason": reason,
    })
}

fn is_ready(value: &Value) -> bool {
    value.get("ready").and_then(Value::as_bool).unwrap_or(false)
}

fn next_action_hint(selection: &ReadinessSelection<'_>) -> &'static str {
    match selection {
        ReadinessSelection::Selected { selected_by, .. } if *selected_by == "cli_option" => {
            "The requested target was selected. Inspect chart_readiness and bars_readiness before running follow-up chart-dependent commands."
        }
        ReadinessSelection::Selected { .. } => {
            "A single chart-compatible target was selected. Inspect chart_readiness and bars_readiness, or reuse selected_target.target_cli_args for follow-up commands."
        }
        ReadinessSelection::None => {
            "No chart target is available. Run `tv tab list`, open a TradingView chart tab, then rerun `tv readiness`."
        }
        ReadinessSelection::Ambiguous(_) => {
            "Multiple chart targets are available. Choose the intended target_cli_args and rerun `tv --target-id <ID> readiness`."
        }
        ReadinessSelection::MissingCliTarget(_) => {
            "The requested target id was not found. Run `tv tab list`, choose a current target_cli_args, then rerun `tv --target-id <ID> readiness`."
        }
    }
}

fn chart_targets(targets: &[Target]) -> Vec<&Target> {
    targets
        .iter()
        .filter(|target| {
            target.kind == "page" && target.url.to_lowercase().contains("tradingview.com/chart")
        })
        .collect()
}

fn screener_targets(targets: &[Target]) -> Vec<&Target> {
    targets
        .iter()
        .filter(|target| transport::is_screener_target(target))
        .collect()
}

fn app_window_targets(targets: &[Target]) -> Vec<&Target> {
    targets
        .iter()
        .filter(|target| transport::is_app_window_target(target))
        .collect()
}

fn target_with_handoff(target: &Target) -> Value {
    json!({
        "id": target.id,
        "title": transport::target_title_for_handoff(target),
        "type": target.kind,
        "url": transport::target_url_for_handoff(target),
        "target_cli_args": transport::target_cli_args(&target.id),
    })
}

fn targets_with_handoff_refs(targets: &[&Target]) -> Vec<Value> {
    targets
        .iter()
        .map(|target| target_with_handoff(target))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tradingview_core::ErrorKind;

    fn config(target_id: Option<&str>) -> TransportConfig {
        TransportConfig {
            host: "127.0.0.1".to_string(),
            port: 9222,
            target_id: target_id.map(str::to_string),
        }
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
    fn readiness_base_payload_reports_ambiguous_targets_without_failure() {
        let targets = vec![
            target("a", "https://www.tradingview.com/chart/a"),
            target("b", "https://www.tradingview.com/chart/b"),
        ];
        let selection = select_readiness_target(&config(None), &targets);
        let payload = readiness_base_payload(&config(None), &targets, &selection);

        assert_eq!(payload["source"], "desktop_readiness");
        assert_eq!(payload["source_category"], "desktop_backed_read");
        assert_eq!(payload["requires_desktop"], true);
        assert_eq!(payload["non_mutating"], true);
        assert_eq!(payload["ready"], false);
        assert_eq!(payload["target_selection"], "ambiguous");
        assert_eq!(payload["chart_target_count"], 2);
        assert_eq!(payload["ambiguous_targets"].as_array().unwrap().len(), 2);
        assert!(
            payload["next_action_hint"]
                .as_str()
                .unwrap()
                .contains("--target-id")
        );
    }

    #[test]
    fn readiness_base_payload_reports_missing_cli_target() {
        let targets = vec![target("a", "https://www.tradingview.com/chart/a")];
        let config = config(Some("missing"));
        let selection = select_readiness_target(&config, &targets);
        let payload = readiness_base_payload(&config, &targets, &selection);

        assert_eq!(payload["ready"], false);
        assert_eq!(payload["target_selection"], "cli_option");
        assert_eq!(payload["requested_target_id"], "missing");
        assert!(payload["selected_target"].is_null());
    }

    #[test]
    fn chart_and_bars_readiness_can_mark_payload_ready() {
        let state = json!({
            "symbol": "NASDAQ:AAPL",
            "resolution": "D",
            "chart_readiness": {
                "chart_api_available": true,
                "bars_available": true,
                "bar_index_state": {
                    "has_first_index": true,
                    "has_last_index": true,
                    "first_index": 1,
                    "last_index": 5
                }
            }
        });
        let bars = json!({
            "symbol": "NASDAQ:AAPL",
            "resolution": "D",
            "bar_count": 1,
            "source": "direct_bars",
            "bars": [{"time": 1, "open": 1, "high": 2, "low": 1, "close": 2, "volume": 10}]
        });
        let chart_readiness = chart_readiness_from_state(&state);
        let bars_readiness = bars_readiness_from_bars(&bars);

        assert!(is_ready(&chart_readiness));
        assert!(is_ready(&bars_readiness));
        assert_eq!(chart_readiness["symbol"], "NASDAQ:AAPL");
        assert_eq!(bars_readiness["last_bar"]["close"], 2);
    }

    #[test]
    fn readiness_from_error_preserves_public_error_details() {
        let err = AppError::new(ErrorKind::InternalApiUnavailable, "bars unavailable")
            .with_details(json!({"reason": "bars_empty"}));
        let readiness = readiness_from_error("ohlcv_bars_read", err);

        assert_eq!(readiness["ready"], false);
        assert_eq!(readiness["phase"], "ohlcv_bars_read");
        assert_eq!(readiness["error"]["kind"], "internal_api_unavailable");
        assert_eq!(readiness["error"]["details"]["reason"], "bars_empty");
    }
}
