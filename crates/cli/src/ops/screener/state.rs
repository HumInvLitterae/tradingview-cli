use serde_json::{Map, Value, json};
use tokio::time::{Duration, sleep};

use tradingview_cdp::{RuntimeEvaluator, Target, TransportConfig};
use tradingview_core::{AppError, ErrorKind};

use super::{
    super::ui::ui_keyboard,
    engine::{
        SCREENER_SOURCE, ensure_dialog_open, expanded_expression, read_screener_state,
        read_screener_with_restore, value_bool,
    },
    validation::validate_screener_limit,
};

const FULL_PAGE_SCREENER_URL: &str = "https://www.tradingview.com/screener/";
const FULL_PAGE_TARGET_WAIT_ATTEMPTS: usize = 10;
const FULL_PAGE_TARGET_WAIT_MS: u64 = 250;

pub async fn screener_status(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    let state = read_screener_state(runtime, None).await?;
    Ok(status_payload(&state))
}

pub async fn screener_open(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    let result = runtime
        .evaluate(&expanded_expression(SCREENER_OPEN_EXPRESSION), true)
        .await?;
    ensure_button_available(&result)?;
    ensure_dialog_open(&result)?;
    Ok(with_action(result, "open"))
}

pub async fn screener_open_full_page(config: &TransportConfig) -> Result<Value, AppError> {
    let targets = tradingview_cdp::fetch_targets(config).await?;
    if let Some(target) = first_screener_target(&targets) {
        activate_screener_target(config, target).await?;
        return Ok(full_page_open_payload(target, false, true));
    }

    let created_target =
        tradingview_cdp::new_target_url(config, FULL_PAGE_SCREENER_URL)
            .await
            .map_err(|error| {
                AppError::new(
                    error.kind,
                    "Full-page Stock Screener target could not be created through local CDP",
                )
                .with_details(json!({
                    "source": SCREENER_SOURCE,
                    "action": "open_full_page",
                    "full_page": true,
                    "target_url": FULL_PAGE_SCREENER_URL,
                    "creation_error": error.to_string(),
                    "creation_error_details": error.details,
                    "next_action_hint": "Open the Stock Screener as a TradingView Desktop tab manually, then rerun `tv screener open --full-page` to reuse it and get target_cli_args.",
                }))
            })?;
    let target = wait_for_screener_target(config, &created_target.id).await?;
    activate_screener_target(config, &target).await?;
    Ok(full_page_open_payload(&target, true, false))
}

pub async fn screener_get(
    runtime: &mut impl RuntimeEvaluator,
    limit: Option<usize>,
) -> Result<Value, AppError> {
    let limit = validate_screener_limit(limit)?;
    let read = read_screener_with_restore(runtime, Some(limit)).await?;

    let mut payload = object_payload(read.state)?;
    payload.insert(
        "source".to_string(),
        Value::String(SCREENER_SOURCE.to_string()),
    );
    payload.insert("limit".to_string(), Value::from(limit));
    payload.insert(
        "opened_for_read".to_string(),
        Value::Bool(read.opened_for_read),
    );
    payload.insert(
        "restored_open_state".to_string(),
        Value::Bool(read.restored_open_state),
    );
    Ok(Value::Object(payload))
}

pub async fn screener_close(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    let before = read_screener_state(runtime, None).await?;
    if !value_bool(&before, "open") {
        return Ok(json!({
            "source": SCREENER_SOURCE,
            "action": "already_closed",
            "open": false,
            "closed": false,
        }));
    }

    ui_keyboard(runtime, "Escape", false, false, false, false).await?;
    let result = runtime
        .evaluate(&expanded_expression(SCREENER_WAIT_CLOSED_EXPRESSION), true)
        .await?;
    if value_bool(&result, "open") {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Stock Screener dialog did not close after Escape",
        )
        .with_details(result));
    }

    Ok(json!({
        "source": SCREENER_SOURCE,
        "action": "close",
        "open": false,
        "closed": true,
    }))
}

fn status_payload(state: &Value) -> Value {
    json!({
        "source": SCREENER_SOURCE,
        "open": value_bool(state, "open"),
        "button_found": value_bool(state, "button_found"),
        "dialog_title": state.get("dialog_title").cloned().unwrap_or(Value::Null),
        "screen_title": state.get("screen_title").cloned().unwrap_or(Value::Null),
        "column_count": state.get("column_count").cloned().unwrap_or(Value::from(0)),
        "filter_count": state.get("filter_count").cloned().unwrap_or(Value::from(0)),
        "visible_row_count": state.get("visible_row_count").cloned().unwrap_or(Value::from(0)),
    })
}

fn first_screener_target(targets: &[Target]) -> Option<&Target> {
    targets
        .iter()
        .find(|target| tradingview_cdp::is_screener_target(target))
}

async fn wait_for_screener_target(
    config: &TransportConfig,
    preferred_target_id: &str,
) -> Result<Target, AppError> {
    for _ in 0..FULL_PAGE_TARGET_WAIT_ATTEMPTS {
        let targets = tradingview_cdp::fetch_targets(config).await?;
        if let Some(target) = targets
            .iter()
            .find(|target| {
                target.id == preferred_target_id && tradingview_cdp::is_screener_target(target)
            })
            .cloned()
        {
            return Ok(target);
        }
        if let Some(target) = first_screener_target(&targets).cloned() {
            return Ok(target);
        }
        sleep(Duration::from_millis(FULL_PAGE_TARGET_WAIT_MS)).await;
    }

    Err(AppError::new(
        ErrorKind::InternalApiUnavailable,
        "Full-page Stock Screener target did not appear after opening",
    )
    .with_details(json!({
        "source": SCREENER_SOURCE,
        "action": "open_full_page",
        "full_page": true,
        "created_target_id": preferred_target_id,
        "target_url": FULL_PAGE_SCREENER_URL,
        "wait_attempts": FULL_PAGE_TARGET_WAIT_ATTEMPTS,
    })))
}

async fn activate_screener_target(
    config: &TransportConfig,
    target: &Target,
) -> Result<(), AppError> {
    let response = reqwest::get(config.activate_url(&target.id))
        .await
        .map_err(|err| AppError::new(ErrorKind::Connection, err.to_string()))?;
    if !response.status().is_success() {
        return Err(AppError::new(
            ErrorKind::Connection,
            format!("CDP target activation returned HTTP {}", response.status()),
        )
        .with_details(json!({
            "target_id": target.id,
            "url": target.url,
        })));
    }
    Ok(())
}

fn full_page_open_payload(target: &Target, created: bool, reused: bool) -> Value {
    json!({
        "source": SCREENER_SOURCE,
        "action": "open_full_page",
        "full_page": true,
        "created": created,
        "reused": reused,
        "target_id": target.id,
        "target_cli_args": tradingview_cdp::target_cli_args(&target.id),
        "title": tradingview_cdp::target_title_for_handoff(target),
        "url": tradingview_cdp::target_url_for_handoff(target),
    })
}

fn ensure_button_available(value: &Value) -> Result<(), AppError> {
    if value_bool(value, "button_found") {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Stock Screener button not found",
        )
        .with_details(value.clone()))
    }
}

fn with_action(value: Value, action: &str) -> Value {
    match object_payload(value) {
        Ok(mut object) => {
            object.insert(
                "source".to_string(),
                Value::String(SCREENER_SOURCE.to_string()),
            );
            object.insert("action".to_string(), Value::String(action.to_string()));
            Value::Object(object)
        }
        Err(_) => json!({
            "source": SCREENER_SOURCE,
            "action": action,
        }),
    }
}

fn object_payload(value: Value) -> Result<Map<String, Value>, AppError> {
    value.as_object().cloned().ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Unexpected Stock Screener response shape",
        )
    })
}

const SCREENER_OPEN_EXPRESSION: &str = r#"
(async function() {
    function sleep(ms) {
        return new Promise(function(resolve) { setTimeout(resolve, ms); });
    }
    REPLACE_HELPERS
    var before = readScreenerState(0);
    if (!before.button_found) return before;
    if (before.open) return Object.assign({ opened: false, already_open: true }, before);
    var button = document.querySelector('[data-name="screener-dialog-button"]');
    if (!button) return before;
    mouseClick(button);
    for (var i = 0; i < 20; i++) {
        await sleep(200);
        var state = readScreenerState(0);
        if (state.open) return Object.assign({ opened: true, already_open: false }, state);
    }
    return Object.assign({ opened: true, already_open: false }, readScreenerState(0));
})()
"#;

const SCREENER_WAIT_CLOSED_EXPRESSION: &str = r#"
(async function() {
    function sleep(ms) {
        return new Promise(function(resolve) { setTimeout(resolve, ms); });
    }
    REPLACE_HELPERS
    for (var i = 0; i < 10; i++) {
        var state = readScreenerState(0);
        if (!state.open) return state;
        await sleep(150);
    }
    return readScreenerState(0);
})()
"#;

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::super::test_support::FakeRuntime;
    use super::*;
    use tradingview_core::ErrorKind;

    fn target(id: &str, title: &str, url: &str) -> Target {
        Target {
            id: id.to_string(),
            title: title.to_string(),
            kind: "page".to_string(),
            url: url.to_string(),
            web_socket_debugger_url: Some(format!("ws://127.0.0.1/devtools/page/{id}")),
        }
    }

    #[tokio::test]
    async fn screener_status_maps_closed_state() {
        let mut runtime = FakeRuntime::new([json!({
            "button_found": true,
            "open": false,
            "column_count": 0,
            "filter_count": 0,
            "visible_row_count": 0
        })]);

        let result = screener_status(&mut runtime).await.unwrap();

        assert_eq!(result["source"], SCREENER_SOURCE);
        assert_eq!(result["open"], false);
        assert_eq!(result["button_found"], true);
    }

    #[tokio::test]
    async fn screener_get_returns_rows_and_restores_closed_state() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": false }),
            json!({ "button_found": true, "open": true, "opened": true }),
            json!({
                "button_found": true,
                "open": true,
                "dialog_title": "株式スクリーナー",
                "screen_title": "米国株",
                "columns": ["シンボル", "価格"],
                "column_count": 2,
                "filters": [{ "text": "指数" }],
                "filter_count": 1,
                "rows": [{
                    "cells": ["AAPL", "200.00"],
                    "field_values": { "シンボル": "AAPL", "価格": "200.00" },
                    "text": "AAPL 200.00"
                }],
                "row_count": 1,
                "visible_row_count": 1
            }),
            json!({ "button_found": true, "open": true }),
            json!({ "button_found": true, "open": false }),
        ]);

        let result = screener_get(&mut runtime, Some(1)).await.unwrap();

        assert_eq!(result["source"], SCREENER_SOURCE);
        assert_eq!(result["opened_for_read"], true);
        assert_eq!(result["restored_open_state"], false);
        assert_eq!(result["rows"][0]["field_values"]["シンボル"], "AAPL");
        assert_eq!(runtime.key_events.len(), 2);
    }

    #[tokio::test]
    async fn screener_get_keeps_initial_open_state() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "rows": [],
                "row_count": 0,
                "visible_row_count": 0
            }),
        ]);

        let result = screener_get(&mut runtime, Some(5)).await.unwrap();

        assert_eq!(result["opened_for_read"], false);
        assert_eq!(result["restored_open_state"], true);
        assert!(runtime.key_events.is_empty());
    }

    #[tokio::test]
    async fn screener_close_is_noop_when_already_closed() {
        let mut runtime = FakeRuntime::new([json!({ "button_found": true, "open": false })]);

        let result = screener_close(&mut runtime).await.unwrap();

        assert_eq!(result["action"], "already_closed");
        assert!(runtime.key_events.is_empty());
    }

    #[tokio::test]
    async fn screener_open_maps_missing_button_to_internal_api_error() {
        let mut runtime = FakeRuntime::new([json!({ "button_found": false, "open": false })]);

        let error = screener_open(&mut runtime).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
    }

    #[tokio::test]
    async fn screener_open_rejects_toolbar_only_false_positive() {
        let mut runtime = FakeRuntime::new([json!({
            "button_found": true,
            "open": false,
            "panel_root_found": false,
            "filter_count": 0,
            "column_count": 0
        })]);

        let error = screener_open(&mut runtime).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert!(error.details.is_some());
    }

    #[test]
    fn first_screener_target_ignores_chart_targets() {
        let targets = vec![
            target("chart", "Chart", "https://www.tradingview.com/chart/abc"),
            target(
                "screener",
                "Screener",
                "https://www.tradingview.com/screener/",
            ),
        ];

        let selected = first_screener_target(&targets).unwrap();

        assert_eq!(selected.id, "screener");
    }

    #[test]
    fn full_page_open_payload_contains_target_handoff() {
        let target = target(
            "target-1",
            "Stock Screener",
            "https://www.tradingview.com/screener/",
        );

        let payload = full_page_open_payload(&target, true, false);

        assert_eq!(payload["source"], SCREENER_SOURCE);
        assert_eq!(payload["action"], "open_full_page");
        assert_eq!(payload["full_page"], true);
        assert_eq!(payload["created"], true);
        assert_eq!(payload["reused"], false);
        assert_eq!(payload["target_id"], "target-1");
        assert_eq!(
            payload["target_cli_args"],
            json!(["--target-id", "target-1"])
        );
    }
}
