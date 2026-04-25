use serde_json::{Map, Value, json};

use crate::{
    cdp::RuntimeEvaluator,
    error::{AppError, ErrorKind},
};

use super::{common::js_string, ui::ui_keyboard};

const SCREENER_SOURCE: &str = "ui_screener_dialog";
const DEFAULT_SCREENER_LIMIT: usize = 20;
const MAX_SCREENER_LIMIT: usize = 100;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScreenerFilterSelector {
    Index(usize),
    Text(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScreenerFilterTarget {
    index: usize,
    text: String,
    data_name: String,
    visible: bool,
}

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

pub fn validate_screener_filter_selector(
    index: Option<usize>,
    text: Option<&str>,
) -> Result<ScreenerFilterSelector, AppError> {
    let text = text.map(str::trim).filter(|value| !value.is_empty());
    match (index, text) {
        (Some(_), Some(_)) => Err(AppError::new(
            ErrorKind::Validation,
            "--index and --text are mutually exclusive",
        )),
        (Some(index), None) => Ok(ScreenerFilterSelector::Index(index)),
        (None, Some(text)) => Ok(ScreenerFilterSelector::Text(text.to_string())),
        (None, None) => Err(AppError::new(
            ErrorKind::Validation,
            "Either --index or --text is required",
        )),
    }
}

pub fn validate_screener_filter_clear(dry_run: bool, confirm_clear: bool) -> Result<(), AppError> {
    if !dry_run && !confirm_clear {
        return Err(AppError::new(
            ErrorKind::Validation,
            "screener filters clear requires --confirm-clear unless --dry-run is used",
        ));
    }
    Ok(())
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

pub async fn screener_filters_remove(
    runtime: &mut impl RuntimeEvaluator,
    selector: ScreenerFilterSelector,
    dry_run: bool,
) -> Result<Value, AppError> {
    let mut session = ScreenerMutationSession::open(runtime).await?;
    let before_state = read_screener_state(session.runtime, None).await?;
    ensure_dialog_open(&before_state)?;
    let before_filters = filter_targets_from_state(&before_state);
    let target = resolve_filter_target(&before_filters, &selector)?;

    if dry_run {
        let close_result = session.restore().await;
        close_result?;
        return Ok(json!({
            "source": SCREENER_SOURCE,
            "action": "filter_remove",
            "dry_run": true,
            "removed": false,
            "open": value_bool(&before_state, "open"),
            "opened_for_mutation": session.opened_for_mutation,
            "restored_open_state": session.restored_open_state,
            "before_filter_count": before_filters.len(),
            "after_filter_count": before_filters.len(),
            "target_filter": filter_target_payload(&target),
        }));
    }

    click_filter_remove_button(session.runtime, &target).await?;
    let after_state =
        wait_for_filter_removed(session.runtime, &target.data_name, before_filters.len()).await?;
    let after_filters = filter_targets_from_state(&after_state);
    let close_result = session.restore().await;
    close_result?;

    Ok(json!({
        "source": SCREENER_SOURCE,
        "action": "filter_remove",
        "dry_run": false,
        "removed": true,
        "open": value_bool(&after_state, "open"),
        "opened_for_mutation": session.opened_for_mutation,
        "restored_open_state": session.restored_open_state,
        "before_filter_count": before_filters.len(),
        "after_filter_count": after_filters.len(),
        "target_filter": filter_target_payload(&target),
    }))
}

pub async fn screener_filters_clear(
    runtime: &mut impl RuntimeEvaluator,
    dry_run: bool,
    confirm_clear: bool,
) -> Result<Value, AppError> {
    validate_screener_filter_clear(dry_run, confirm_clear)?;

    let mut session = ScreenerMutationSession::open(runtime).await?;
    let before_state = read_screener_state(session.runtime, None).await?;
    ensure_dialog_open(&before_state)?;
    let before_filters = filter_targets_from_state(&before_state);
    let targets = before_filters
        .iter()
        .map(filter_target_payload)
        .collect::<Vec<_>>();

    if dry_run {
        let close_result = session.restore().await;
        close_result?;
        return Ok(json!({
            "source": SCREENER_SOURCE,
            "action": "filters_clear",
            "dry_run": true,
            "cleared": false,
            "open": value_bool(&before_state, "open"),
            "opened_for_mutation": session.opened_for_mutation,
            "restored_open_state": session.restored_open_state,
            "before_filter_count": before_filters.len(),
            "after_filter_count": before_filters.len(),
            "target_filters": targets,
            "removed_filters": [],
        }));
    }

    let mut removed_filters = Vec::new();
    let mut current_filters = before_filters;
    while let Some(target) = current_filters.first().cloned() {
        click_filter_remove_button(session.runtime, &target).await?;
        let state =
            wait_for_filter_removed(session.runtime, &target.data_name, current_filters.len())
                .await?;
        removed_filters.push(filter_target_payload(&target));
        current_filters = filter_targets_from_state(&state);
    }

    let after_state = read_screener_state(session.runtime, None).await?;
    let after_filters = filter_targets_from_state(&after_state);
    let close_result = session.restore().await;
    close_result?;

    Ok(json!({
        "source": SCREENER_SOURCE,
        "action": "filters_clear",
        "dry_run": false,
        "cleared": true,
        "open": value_bool(&after_state, "open"),
        "opened_for_mutation": session.opened_for_mutation,
        "restored_open_state": session.restored_open_state,
        "before_filter_count": targets.len(),
        "after_filter_count": after_filters.len(),
        "target_filters": targets,
        "removed_filters": removed_filters,
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

struct ScreenerMutationSession<'a, R: RuntimeEvaluator> {
    runtime: &'a mut R,
    opened_for_mutation: bool,
    restored_open_state: bool,
}

impl<'a, R: RuntimeEvaluator> ScreenerMutationSession<'a, R> {
    async fn open(runtime: &'a mut R) -> Result<Self, AppError> {
        let initial = read_screener_state(runtime, None).await?;
        let restored_open_state = value_bool(&initial, "open");
        let mut opened_for_mutation = false;

        if !restored_open_state {
            screener_open(runtime).await?;
            opened_for_mutation = true;
        }

        Ok(Self {
            runtime,
            opened_for_mutation,
            restored_open_state,
        })
    }

    async fn restore(&mut self) -> Result<(), AppError> {
        if self.opened_for_mutation {
            screener_close(self.runtime).await?;
        }
        Ok(())
    }
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

fn filter_targets_from_state(state: &Value) -> Vec<ScreenerFilterTarget> {
    state
        .get("filters")
        .and_then(Value::as_array)
        .map(|filters| {
            filters
                .iter()
                .enumerate()
                .filter_map(|(fallback_index, filter)| {
                    let data_name = filter.get("data_name").and_then(Value::as_str)?;
                    let text = filter
                        .get("text")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .trim();
                    Some(ScreenerFilterTarget {
                        index: filter
                            .get("index")
                            .and_then(Value::as_u64)
                            .and_then(|value| usize::try_from(value).ok())
                            .unwrap_or(fallback_index),
                        text: text.to_string(),
                        data_name: data_name.to_string(),
                        visible: filter
                            .get("visible")
                            .and_then(Value::as_bool)
                            .unwrap_or(false),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn resolve_filter_target(
    filters: &[ScreenerFilterTarget],
    selector: &ScreenerFilterSelector,
) -> Result<ScreenerFilterTarget, AppError> {
    match selector {
        ScreenerFilterSelector::Index(index) => filters
            .iter()
            .find(|filter| filter.index == *index)
            .cloned()
            .ok_or_else(|| {
                AppError::new(
                    ErrorKind::Validation,
                    format!("No visible Screener filter found at index {index}"),
                )
                .with_details(json!({ "filters": filter_targets_payload(filters) }))
            }),
        ScreenerFilterSelector::Text(text) => {
            let needle = text.to_lowercase();
            let matches = filters
                .iter()
                .filter(|filter| filter.text.to_lowercase().contains(&needle))
                .cloned()
                .collect::<Vec<_>>();
            match matches.len() {
                0 => Err(AppError::new(
                    ErrorKind::Validation,
                    format!("No visible Screener filter matched text {text:?}"),
                )
                .with_details(json!({ "filters": filter_targets_payload(filters) }))),
                1 => Ok(matches[0].clone()),
                _ => Err(AppError::new(
                    ErrorKind::Validation,
                    format!("Screener filter text {text:?} matched multiple filters"),
                )
                .with_details(json!({ "matches": filter_targets_payload(&matches) }))),
            }
        }
    }
}

fn filter_target_payload(filter: &ScreenerFilterTarget) -> Value {
    json!({
        "index": filter.index,
        "text": filter.text,
        "data_name": filter.data_name,
        "visible": filter.visible,
    })
}

fn filter_targets_payload(filters: &[ScreenerFilterTarget]) -> Vec<Value> {
    filters.iter().map(filter_target_payload).collect()
}

async fn click_filter_remove_button(
    runtime: &mut impl RuntimeEvaluator,
    target: &ScreenerFilterTarget,
) -> Result<(), AppError> {
    let data_name = js_string(&target.data_name)?;
    let result = runtime
        .evaluate(
            &expanded_expression(&format!(
                r#"
                (async function() {{
                    function sleep(ms) {{
                        return new Promise(function(resolve) {{ setTimeout(resolve, ms); }});
                    }}
                    REPLACE_HELPERS
                    function mouseClick(el) {{
                        var rect = el.getBoundingClientRect();
                        var x = rect.left + rect.width / 2;
                        var y = rect.top + rect.height / 2;
                        ['mouseover', 'mousedown', 'mouseup', 'click'].forEach(function(type) {{
                            el.dispatchEvent(new MouseEvent(type, {{
                                bubbles: true,
                                cancelable: true,
                                clientX: x,
                                clientY: y,
                                view: window
                            }}));
                        }});
                    }}
                    var pill = document.querySelector('[data-name=' + {data_name} + ']');
                    if (!pill || !visible(pill)) {{
                        return {{ found: false, removed: false, data_name: {data_name} }};
                    }}
                    mouseClick(pill);
                    for (var i = 0; i < 20; i++) {{
                        await sleep(100);
                        var buttons = Array.from(document.querySelectorAll('button')).filter(visible);
                        var remove = buttons.find(function(button) {{
                            return String(button.className || '').indexOf('removeButton') >= 0;
                        }});
                        if (remove) {{
                            setTimeout(function() {{ mouseClick(remove); }}, 0);
                            return {{ found: true, remove_button_found: true, click_scheduled: true, data_name: {data_name} }};
                        }}
                    }}
                    return {{ found: true, remove_button_found: false, clicked: false, data_name: {data_name} }};
                }})()
                "#
            )),
            true,
        )
        .await?;

    if !value_bool(&result, "found") {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener filter pill not found",
        )
        .with_details(result));
    }
    if !value_bool(&result, "remove_button_found") {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener filter remove button not found",
        )
        .with_details(result));
    }
    Ok(())
}

async fn wait_for_filter_removed(
    runtime: &mut impl RuntimeEvaluator,
    data_name: &str,
    before_count: usize,
) -> Result<Value, AppError> {
    let raw_data_name = data_name.to_string();
    let data_name = js_string(data_name)?;
    let result = runtime
        .evaluate(
            &expanded_expression(&format!(
                r#"
                (async function() {{
                    function sleep(ms) {{
                        return new Promise(function(resolve) {{ setTimeout(resolve, ms); }});
                    }}
                    REPLACE_HELPERS
                    for (var i = 0; i < 30; i++) {{
                        var state = readScreenerState(0);
                        var stillPresent = state.filters.some(function(filter) {{
                            return filter.data_name === {data_name};
                        }});
                        if (!stillPresent || state.filter_count < {before_count}) return state;
                        await sleep(150);
                    }}
                    return readScreenerState(0);
                }})()
                "#
            )),
            true,
        )
        .await?;

    let after_filters = filter_targets_from_state(&result);
    let still_present = after_filters
        .iter()
        .any(|filter| filter.data_name == raw_data_name);
    if still_present && after_filters.len() >= before_count {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener filter still appears after remove",
        )
        .with_details(result));
    }

    Ok(result)
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

    #[test]
    fn validate_screener_filter_selector_requires_one_target() {
        assert_eq!(
            validate_screener_filter_selector(Some(2), None).unwrap(),
            ScreenerFilterSelector::Index(2)
        );
        assert_eq!(
            validate_screener_filter_selector(None, Some(" PER ")).unwrap(),
            ScreenerFilterSelector::Text("PER".to_string())
        );
        assert_eq!(
            validate_screener_filter_selector(None, None)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_filter_selector(Some(0), Some("PER"))
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
    }

    #[test]
    fn validate_screener_filter_clear_requires_confirmation_for_mutation() {
        assert!(validate_screener_filter_clear(true, false).is_ok());
        assert!(validate_screener_filter_clear(false, true).is_ok());
        assert_eq!(
            validate_screener_filter_clear(false, false)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
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
    async fn screener_filters_remove_dry_run_returns_target_without_mutation() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "filters": [
                    { "index": 0, "text": "Market cap", "data_name": "screener-filter-pill-market_cap", "visible": true },
                    { "index": 1, "text": "PER", "data_name": "screener-filter-pill-pe", "visible": true }
                ],
                "filter_count": 2
            }),
        ]);

        let result = screener_filters_remove(
            &mut runtime,
            ScreenerFilterSelector::Text("per".to_string()),
            true,
        )
        .await
        .unwrap();

        assert_eq!(result["action"], "filter_remove");
        assert_eq!(result["dry_run"], true);
        assert_eq!(result["removed"], false);
        assert_eq!(result["target_filter"]["text"], "PER");
        assert_eq!(result["before_filter_count"], 2);
        assert_eq!(result["after_filter_count"], 2);
    }

    #[tokio::test]
    async fn screener_filters_remove_clicks_target_and_reports_counts() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "filters": [
                    { "index": 0, "text": "Market cap", "data_name": "screener-filter-pill-market_cap", "visible": true },
                    { "index": 1, "text": "PER", "data_name": "screener-filter-pill-pe", "visible": true }
                ],
                "filter_count": 2
            }),
            json!({ "found": true, "remove_button_found": true, "clicked": true }),
            json!({
                "button_found": true,
                "open": true,
                "filters": [
                    { "index": 0, "text": "Market cap", "data_name": "screener-filter-pill-market_cap", "visible": true }
                ],
                "filter_count": 1
            }),
        ]);

        let result = screener_filters_remove(&mut runtime, ScreenerFilterSelector::Index(1), false)
            .await
            .unwrap();

        assert_eq!(result["dry_run"], false);
        assert_eq!(result["removed"], true);
        assert_eq!(
            result["target_filter"]["data_name"],
            "screener-filter-pill-pe"
        );
        assert_eq!(result["before_filter_count"], 2);
        assert_eq!(result["after_filter_count"], 1);
    }

    #[tokio::test]
    async fn screener_filters_remove_rejects_ambiguous_text() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "filters": [
                    { "index": 0, "text": "Price", "data_name": "screener-filter-pill-price", "visible": true },
                    { "index": 1, "text": "Price EMA", "data_name": "screener-filter-pill-price_ema", "visible": true }
                ],
                "filter_count": 2
            }),
        ]);

        let error = screener_filters_remove(
            &mut runtime,
            ScreenerFilterSelector::Text("price".to_string()),
            true,
        )
        .await
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(error.details.is_some());
    }

    #[tokio::test]
    async fn screener_filters_clear_requires_confirmation_and_removes_all() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "filters": [
                    { "index": 0, "text": "Market cap", "data_name": "screener-filter-pill-market_cap", "visible": true },
                    { "index": 1, "text": "PER", "data_name": "screener-filter-pill-pe", "visible": true }
                ],
                "filter_count": 2
            }),
            json!({ "found": true, "remove_button_found": true, "clicked": true }),
            json!({
                "button_found": true,
                "open": true,
                "filters": [
                    { "index": 0, "text": "PER", "data_name": "screener-filter-pill-pe", "visible": true }
                ],
                "filter_count": 1
            }),
            json!({ "found": true, "remove_button_found": true, "clicked": true }),
            json!({
                "button_found": true,
                "open": true,
                "filters": [],
                "filter_count": 0
            }),
            json!({
                "button_found": true,
                "open": true,
                "filters": [],
                "filter_count": 0
            }),
        ]);

        let result = screener_filters_clear(&mut runtime, false, true)
            .await
            .unwrap();

        assert_eq!(result["action"], "filters_clear");
        assert_eq!(result["cleared"], true);
        assert_eq!(result["before_filter_count"], 2);
        assert_eq!(result["after_filter_count"], 0);
        assert_eq!(result["removed_filters"].as_array().unwrap().len(), 2);
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
