use serde_json::{Map, Value, json};

use crate::{
    cdp::RuntimeEvaluator,
    error::{AppError, ErrorKind},
};

use super::ui::ui_keyboard;

const SCREENER_SOURCE: &str = "ui_screener_dialog";
const DEFAULT_SCREENER_LIMIT: usize = 20;
const MAX_SCREENER_LIMIT: usize = 100;

pub fn validate_screener_limit(limit: Option<usize>) -> Result<usize, AppError> {
    match limit {
        Some(0) => Err(AppError::new(
            ErrorKind::Validation,
            "--limit must be greater than 0",
        )),
        Some(limit) => Ok(limit.min(MAX_SCREENER_LIMIT)),
        None => Ok(DEFAULT_SCREENER_LIMIT),
    }
}

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

pub async fn screener_screens_active(
    runtime: &mut impl RuntimeEvaluator,
) -> Result<Value, AppError> {
    let read = read_screener_with_restore(runtime, None).await?;
    Ok(json!({
        "source": SCREENER_SOURCE,
        "open": value_bool(&read.state, "open"),
        "opened_for_read": read.opened_for_read,
        "restored_open_state": read.restored_open_state,
        "dialog_title": read.state.get("dialog_title").cloned().unwrap_or(Value::Null),
        "screen_title": read.state.get("screen_title").cloned().unwrap_or(Value::Null),
    }))
}

pub async fn screener_filters_list(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    let read = read_screener_with_restore(runtime, None).await?;
    let filters = normalize_filters(read.state.get("filters"));
    Ok(json!({
        "source": SCREENER_SOURCE,
        "open": value_bool(&read.state, "open"),
        "opened_for_read": read.opened_for_read,
        "restored_open_state": read.restored_open_state,
        "filter_count": filters.len(),
        "filters": filters,
    }))
}

pub async fn screener_columns_list(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    let read = read_screener_with_restore(runtime, None).await?;
    let columns = normalize_columns(read.state.get("columns"));
    Ok(json!({
        "source": SCREENER_SOURCE,
        "open": value_bool(&read.state, "open"),
        "opened_for_read": read.opened_for_read,
        "restored_open_state": read.restored_open_state,
        "column_count": columns.len(),
        "columns": columns,
    }))
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

async fn read_screener_state(
    runtime: &mut impl RuntimeEvaluator,
    limit: Option<usize>,
) -> Result<Value, AppError> {
    let expression = match limit {
        Some(limit) => expanded_expression(&screener_read_expression(limit)),
        None => screener_state_expression(0),
    };
    let state = runtime.evaluate(&expression, true).await?;
    Ok(state)
}

struct ScreenerReadState {
    state: Value,
    opened_for_read: bool,
    restored_open_state: bool,
}

async fn read_screener_with_restore(
    runtime: &mut impl RuntimeEvaluator,
    limit: Option<usize>,
) -> Result<ScreenerReadState, AppError> {
    let initial = read_screener_state(runtime, None).await?;
    let restored_open_state = value_bool(&initial, "open");
    let mut opened_for_read = false;

    if !restored_open_state {
        screener_open(runtime).await?;
        opened_for_read = true;
    }

    let read_result = read_screener_state(runtime, limit).await;
    let close_result = if opened_for_read {
        Some(screener_close(runtime).await)
    } else {
        None
    };

    let state = read_result?;
    if let Some(Err(err)) = close_result {
        return Err(err);
    }
    ensure_dialog_open(&state)?;

    Ok(ScreenerReadState {
        state,
        opened_for_read,
        restored_open_state,
    })
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

fn ensure_dialog_open(value: &Value) -> Result<(), AppError> {
    if value_bool(value, "open") {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Stock Screener dialog is not open",
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

fn value_bool(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn normalize_filters(filters: Option<&Value>) -> Vec<Value> {
    filters
        .and_then(Value::as_array)
        .map(|filters| {
            filters
                .iter()
                .enumerate()
                .map(|(index, filter)| {
                    json!({
                        "index": index,
                        "text": filter.get("text").cloned().unwrap_or(Value::Null),
                        "data_name": filter.get("data_name").cloned().unwrap_or(Value::Null),
                        "visible": filter.get("visible").and_then(Value::as_bool).unwrap_or(false),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn normalize_columns(columns: Option<&Value>) -> Vec<Value> {
    columns
        .and_then(Value::as_array)
        .map(|columns| {
            columns
                .iter()
                .enumerate()
                .filter_map(|(index, column)| {
                    column.as_str().map(|name| {
                        json!({
                            "index": index,
                            "name": name,
                        })
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn screener_state_expression(limit: usize) -> String {
    format!(
        r#"
        (function() {{
            {SCREENER_HELPERS}
            return readScreenerState({limit});
        }})()
        "#
    )
}

fn screener_read_expression(limit: usize) -> String {
    format!(
        r#"
        (async function() {{
            function sleep(ms) {{
                return new Promise(function(resolve) {{ setTimeout(resolve, ms); }});
            }}
            REPLACE_HELPERS
            for (var i = 0; i < 20; i++) {{
                var state = readScreenerState({limit});
                var hasTextRow = state.rows.some(function(row) {{
                    return row.text || row.cells.some(function(cell) {{ return cell; }});
                }});
                if (!state.open || hasTextRow || state.visible_row_count === 0) return state;
                await sleep(200);
            }}
            return readScreenerState({limit});
        }})()
        "#
    )
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
    button.click();
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

const SCREENER_HELPERS: &str = r#"
function visible(el) {
    if (!el || !el.getBoundingClientRect) return false;
    var rect = el.getBoundingClientRect();
    return rect.width > 0 && rect.height > 0;
}
function textOf(el) {
    return (el && (el.textContent || el.innerText) || '').replace(/\s+/g, ' ').trim();
}
function visibleElements(selector) {
    return Array.from(document.querySelectorAll(selector)).filter(visible);
}
function readScreenerState(limit) {
    var button = document.querySelector('[data-name="screener-dialog-button"]');
    var screenerDataElements = visibleElements('[data-name*="screener"]');
    var classElements = visibleElements('[class*="screener"]');
    var heading = Array.from(document.querySelectorAll('h1, h2, h3'))
        .find(function(el) {
            var text = textOf(el);
            return visible(el) && text.length <= 120 && /screener|スクリーナー/i.test(text);
        }) || Array.from(document.querySelectorAll('button, div, span'))
        .find(function(el) {
            var text = textOf(el);
            return visible(el) && text.length <= 120 && /screener|スクリーナー/i.test(text);
        }) || null;
    var table = visibleElements('table')[0] || null;
    var open = !!(table || heading || screenerDataElements.some(function(el) {
        return el !== button && (el.getAttribute('data-name') || '').indexOf('screener') >= 0;
    }));
    var filters = visibleElements('[data-name^="screener-filter-pill-"]').map(function(el) {
        return {
            text: textOf(el),
            data_name: el.getAttribute('data-name') || null,
            visible: visible(el)
        };
    });
    var columns = table ? Array.from(table.querySelectorAll('th')).map(textOf).filter(function(text) {
        return text.length > 0;
    }) : [];
    var rows = [];
    if (table && limit > 0) {
        rows = Array.from(table.querySelectorAll('tbody tr')).slice(0, limit).map(function(row) {
            var cells = Array.from(row.querySelectorAll('td, th')).map(textOf);
            var fieldValues = {};
            columns.forEach(function(column, index) {
                if (column && index < cells.length) fieldValues[column] = cells[index];
            });
            return {
                cells: cells,
                field_values: fieldValues,
                text: textOf(row).substring(0, 500)
            };
        });
    }
    var visibleRowCount = table ? table.querySelectorAll('tbody tr').length : 0;
    return {
        button_found: !!button,
        open: open,
        dialog_title: heading ? textOf(heading) : null,
        screen_title: (document.querySelector('[data-name="screener-topbar-screen-title"]') ? textOf(document.querySelector('[data-name="screener-topbar-screen-title"]')) : null),
        filters: filters,
        filter_count: filters.length,
        columns: columns,
        column_count: columns.length,
        rows: rows,
        row_count: rows.length,
        visible_row_count: visibleRowCount,
        table_found: !!table,
        class_match_count: classElements.length,
        data_name_match_count: screenerDataElements.length
    };
}
"#;

fn expanded_expression(template: &str) -> String {
    template.replace("REPLACE_HELPERS", SCREENER_HELPERS)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::test_support::FakeRuntime;
    use super::*;

    #[test]
    fn validate_screener_limit_defaults_clamps_and_rejects_zero() {
        assert_eq!(validate_screener_limit(None).unwrap(), 20);
        assert_eq!(validate_screener_limit(Some(3)).unwrap(), 3);
        assert_eq!(validate_screener_limit(Some(500)).unwrap(), 100);

        let error = validate_screener_limit(Some(0)).unwrap_err();
        assert_eq!(error.kind, ErrorKind::Validation);
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
    async fn screener_screens_active_returns_titles() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "dialog_title": "Stock Screener",
                "screen_title": "My Screen"
            }),
        ]);

        let result = screener_screens_active(&mut runtime).await.unwrap();

        assert_eq!(result["source"], SCREENER_SOURCE);
        assert_eq!(result["dialog_title"], "Stock Screener");
        assert_eq!(result["screen_title"], "My Screen");
        assert_eq!(result["opened_for_read"], false);
        assert_eq!(result["restored_open_state"], true);
    }

    #[tokio::test]
    async fn screener_filters_list_indexes_filters_and_restores_closed_state() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": false }),
            json!({ "button_found": true, "open": true, "opened": true }),
            json!({
                "button_found": true,
                "open": true,
                "filters": [
                    { "text": "Market cap", "data_name": "screener-filter-pill-market_cap", "visible": true },
                    { "text": "Exchange", "data_name": "screener-filter-pill-exchange", "visible": true }
                ],
                "filter_count": 2
            }),
            json!({ "button_found": true, "open": true }),
            json!({ "button_found": true, "open": false }),
        ]);

        let result = screener_filters_list(&mut runtime).await.unwrap();

        assert_eq!(result["source"], SCREENER_SOURCE);
        assert_eq!(result["filter_count"], 2);
        assert_eq!(result["filters"][0]["index"], 0);
        assert_eq!(result["filters"][0]["text"], "Market cap");
        assert_eq!(
            result["filters"][1]["data_name"],
            "screener-filter-pill-exchange"
        );
        assert_eq!(result["opened_for_read"], true);
        assert_eq!(result["restored_open_state"], false);
        assert_eq!(runtime.key_events.len(), 2);
    }

    #[tokio::test]
    async fn screener_columns_list_indexes_columns_and_keeps_open_state() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "columns": ["Symbol", "Price", "Change %"],
                "column_count": 3
            }),
        ]);

        let result = screener_columns_list(&mut runtime).await.unwrap();

        assert_eq!(result["source"], SCREENER_SOURCE);
        assert_eq!(result["column_count"], 3);
        assert_eq!(result["columns"][0]["index"], 0);
        assert_eq!(result["columns"][2]["name"], "Change %");
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
}
