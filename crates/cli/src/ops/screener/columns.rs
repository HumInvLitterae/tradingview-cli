use serde_json::{Map, Value, json};

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};

use super::{
    engine::{
        SCREENER_SOURCE, ScreenerMutationSession, ensure_dialog_open, expanded_expression,
        fetch_active_screener_storage_config, normalize_columns, read_screener_state,
        read_screener_with_restore, require_active_screen_title, value_bool,
    },
    validation::{
        ScreenerColumnAddRequest, ScreenerColumnSelector, is_test_screener_screen_name,
        validate_screener_column_reorder_request,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScreenerColumnTarget {
    index: usize,
    name: String,
}

#[derive(Clone, Debug, PartialEq)]
struct ScreenerStorageColumnTarget {
    index: usize,
    id: String,
    name: Option<String>,
    params: Value,
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

pub async fn screener_columns_config(
    runtime: &mut impl RuntimeEvaluator,
) -> Result<Value, AppError> {
    let state = read_screener_state(runtime, None).await?;
    ensure_dialog_open(&state)?;
    let visible_columns = column_targets_from_state(&state);
    let screen_title = require_active_screen_title(&state)?;
    let config = fetch_active_screener_storage_config(runtime, &screen_title).await?;
    let columns = storage_columns_from_config(&config, &visible_columns);

    Ok(json!({
        "source": SCREENER_SOURCE,
        "action": "columns_config",
        "scope": "screen_storage_api",
        "screen_title": screen_title,
        "screen_id": config.get("screen_id").cloned().unwrap_or(Value::Null),
        "active_column_set": config.get("active_column_set").cloned().unwrap_or(Value::Null),
        "column_count": columns.len(),
        "columns": storage_column_targets_payload(&columns),
    }))
}

pub async fn screener_columns_actions(
    runtime: &mut impl RuntimeEvaluator,
) -> Result<Value, AppError> {
    let mut session = ScreenerMutationSession::open(runtime).await?;
    let state = read_screener_state(session.runtime, None).await?;
    ensure_dialog_open(&state)?;
    let columns = column_targets_from_state(&state);
    let actions = read_column_actions(session.runtime).await?;
    let close_result = session.restore().await;
    close_result?;

    Ok(json!({
        "source": SCREENER_SOURCE,
        "action": "columns_actions",
        "open": value_bool(&state, "open"),
        "screen_title": state.get("screen_title").cloned().unwrap_or(Value::Null),
        "column_count": columns.len(),
        "columns": column_targets_payload(&columns),
        "opened_for_read": session.opened_for_mutation,
        "restored_open_state": session.restored_open_state,
        "settings_button_found": value_bool(&actions, "settings_button_found"),
        "settings_opened": value_bool(&actions, "settings_opened"),
        "categories": actions.get("categories").cloned().unwrap_or_else(|| json!([])),
        "header_menu_actions": actions.get("header_menu_actions").cloned().unwrap_or_else(|| json!([])),
        "remove_supported": value_bool(&actions, "remove_supported"),
        "reset_supported": value_bool(&actions, "reset_supported"),
        "unavailable_reason": actions.get("unavailable_reason").cloned().unwrap_or(Value::Null),
    }))
}

pub async fn screener_columns_remove(
    runtime: &mut impl RuntimeEvaluator,
    selector: ScreenerColumnSelector,
    dry_run: bool,
) -> Result<Value, AppError> {
    let before_state = read_screener_state(runtime, None).await?;
    ensure_dialog_open(&before_state)?;
    let visible_columns = column_targets_from_state(&before_state);
    let visible_target = resolve_column_target(&visible_columns, &selector)?;
    let screen_title = require_active_screen_title(&before_state)?;
    let before_config = fetch_active_screener_storage_config(runtime, &screen_title).await?;
    let before_columns = storage_columns_from_config(&before_config, &visible_columns);
    ensure_storage_column_index(&before_columns, visible_target.index)?;
    let target = before_columns[visible_target.index].clone();
    let expected_after_columns = remove_storage_column(&before_columns, target.index);

    if dry_run {
        return Ok(json!({
            "source": SCREENER_SOURCE,
            "action": "columns_remove",
            "scope": "screen_storage_api",
            "dry_run": true,
            "removed": false,
            "screen_title": screen_title,
            "screen_id": before_config.get("screen_id").cloned().unwrap_or(Value::Null),
            "before_column_count": before_columns.len(),
            "after_column_count": expected_after_columns.len(),
            "target_column": storage_column_target_payload(&target),
            "columns": storage_column_targets_payload(&before_columns),
            "after_columns": storage_column_targets_payload(&expected_after_columns),
        }));
    }

    ensure_test_screener_screen_for_column_mutation(&screen_title, "remove")?;
    let save_result =
        save_screener_storage_columns(runtime, &before_config, &expected_after_columns).await?;
    let after_config = fetch_active_screener_storage_config(runtime, &screen_title).await?;
    let after_columns = storage_columns_from_config(&after_config, &visible_columns);
    if !storage_column_order_matches(&after_columns, &expected_after_columns) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener storage columns did not match after remove",
        )
        .with_details(json!({
            "source": SCREENER_SOURCE,
            "action": "columns_remove",
            "scope": "screen_storage_api",
            "target_column": storage_column_target_payload(&target),
            "expected_columns": storage_column_targets_payload(&expected_after_columns),
            "after_columns": storage_column_targets_payload(&after_columns),
            "save_result": save_result,
        })));
    }

    Ok(json!({
        "source": SCREENER_SOURCE,
        "action": "columns_remove",
        "scope": "screen_storage_api",
        "dry_run": false,
        "removed": true,
        "screen_title": screen_title,
        "screen_id": before_config.get("screen_id").cloned().unwrap_or(Value::Null),
        "target_column": storage_column_target_payload(&target),
        "before_column_count": before_columns.len(),
        "after_column_count": after_columns.len(),
        "columns": storage_column_targets_payload(&expected_after_columns),
        "save_result": save_result,
    }))
}

pub async fn screener_columns_add(
    runtime: &mut impl RuntimeEvaluator,
    request: ScreenerColumnAddRequest,
) -> Result<Value, AppError> {
    let before_state = read_screener_state(runtime, None).await?;
    ensure_dialog_open(&before_state)?;
    let visible_columns = column_targets_from_state(&before_state);
    let screen_title = require_active_screen_title(&before_state)?;
    let before_config = fetch_active_screener_storage_config(runtime, &screen_title).await?;
    let before_columns = storage_columns_from_config(&before_config, &visible_columns);
    let expected_after_columns = add_storage_column(&before_columns, &request)?;
    let inserted_index = request
        .after_index
        .map(|index| index + 1)
        .unwrap_or(before_columns.len());
    let target = expected_after_columns[inserted_index].clone();

    if request.dry_run {
        return Ok(json!({
            "source": SCREENER_SOURCE,
            "action": "columns_add",
            "scope": "screen_storage_api",
            "dry_run": true,
            "added": false,
            "screen_title": screen_title,
            "screen_id": before_config.get("screen_id").cloned().unwrap_or(Value::Null),
            "after_index": request.after_index,
            "inserted_index": inserted_index,
            "before_column_count": before_columns.len(),
            "after_column_count": expected_after_columns.len(),
            "target_column": storage_column_target_payload(&target),
            "columns": storage_column_targets_payload(&before_columns),
            "after_columns": storage_column_targets_payload(&expected_after_columns),
        }));
    }

    ensure_test_screener_screen_for_column_mutation(&screen_title, "add")?;
    let save_result =
        save_screener_storage_columns(runtime, &before_config, &expected_after_columns).await?;
    let after_config = fetch_active_screener_storage_config(runtime, &screen_title).await?;
    let after_columns = storage_columns_from_config(&after_config, &visible_columns);
    if !storage_column_order_matches(&after_columns, &expected_after_columns) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener storage columns did not match after add",
        )
        .with_details(json!({
            "source": SCREENER_SOURCE,
            "action": "columns_add",
            "scope": "screen_storage_api",
            "target_column": storage_column_target_payload(&target),
            "expected_columns": storage_column_targets_payload(&expected_after_columns),
            "after_columns": storage_column_targets_payload(&after_columns),
            "save_result": save_result,
        })));
    }

    Ok(json!({
        "source": SCREENER_SOURCE,
        "action": "columns_add",
        "scope": "screen_storage_api",
        "dry_run": false,
        "added": true,
        "screen_title": screen_title,
        "screen_id": before_config.get("screen_id").cloned().unwrap_or(Value::Null),
        "after_index": request.after_index,
        "inserted_index": inserted_index,
        "target_column": storage_column_target_payload(&target),
        "before_column_count": before_columns.len(),
        "after_column_count": expected_after_columns.len(),
        "columns": storage_column_targets_payload(&expected_after_columns),
        "save_result": save_result,
    }))
}

pub async fn screener_columns_reorder(
    runtime: &mut impl RuntimeEvaluator,
    from_index: usize,
    to_index: usize,
    dry_run: bool,
) -> Result<Value, AppError> {
    let (from_index, to_index) = validate_screener_column_reorder_request(from_index, to_index)?;
    let before_state = read_screener_state(runtime, None).await?;
    ensure_dialog_open(&before_state)?;
    let visible_columns = column_targets_from_state(&before_state);
    let screen_title = require_active_screen_title(&before_state)?;
    let before_config = fetch_active_screener_storage_config(runtime, &screen_title).await?;
    let before_columns = storage_columns_from_config(&before_config, &visible_columns);
    ensure_storage_column_index(&before_columns, from_index)?;
    ensure_storage_column_index(&before_columns, to_index)?;
    let target = before_columns[from_index].clone();
    let expected_after_columns = reorder_storage_columns(&before_columns, from_index, to_index);

    if dry_run {
        return Ok(json!({
            "source": SCREENER_SOURCE,
            "action": "columns_reorder",
            "scope": "screen_storage_api",
            "dry_run": true,
            "reordered": false,
            "screen_title": screen_title,
            "screen_id": before_config.get("screen_id").cloned().unwrap_or(Value::Null),
            "from_index": from_index,
            "to_index": to_index,
            "target_column": storage_column_target_payload(&target),
            "before_column_count": before_columns.len(),
            "after_column_count": expected_after_columns.len(),
            "columns": storage_column_targets_payload(&before_columns),
            "after_columns": storage_column_targets_payload(&expected_after_columns),
        }));
    }

    ensure_test_screener_screen_for_column_mutation(&screen_title, "reorder")?;
    let save_result =
        save_screener_storage_columns(runtime, &before_config, &expected_after_columns).await?;
    let after_config = fetch_active_screener_storage_config(runtime, &screen_title).await?;
    let after_columns = storage_columns_from_config(&after_config, &visible_columns);
    if !storage_column_order_matches(&after_columns, &expected_after_columns) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener storage columns did not match after reorder",
        )
        .with_details(json!({
            "source": SCREENER_SOURCE,
            "action": "columns_reorder",
            "scope": "screen_storage_api",
            "expected_columns": storage_column_targets_payload(&expected_after_columns),
            "after_columns": storage_column_targets_payload(&after_columns),
            "save_result": save_result,
        })));
    }

    Ok(json!({
        "source": SCREENER_SOURCE,
        "action": "columns_reorder",
        "scope": "screen_storage_api",
        "dry_run": false,
        "reordered": true,
        "screen_title": screen_title,
        "screen_id": before_config.get("screen_id").cloned().unwrap_or(Value::Null),
        "from_index": from_index,
        "to_index": to_index,
        "target_column": storage_column_target_payload(&target),
        "before_column_count": before_columns.len(),
        "after_column_count": after_columns.len(),
        "columns": storage_column_targets_payload(&expected_after_columns),
        "save_result": save_result,
    }))
}

fn column_targets_from_state(state: &Value) -> Vec<ScreenerColumnTarget> {
    state
        .get("columns")
        .and_then(Value::as_array)
        .map(|columns| {
            columns
                .iter()
                .enumerate()
                .filter_map(|(index, column)| {
                    column.as_str().map(str::trim).and_then(|name| {
                        if name.is_empty() {
                            None
                        } else {
                            Some(ScreenerColumnTarget {
                                index,
                                name: name.to_string(),
                            })
                        }
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn resolve_column_target(
    columns: &[ScreenerColumnTarget],
    selector: &ScreenerColumnSelector,
) -> Result<ScreenerColumnTarget, AppError> {
    match selector {
        ScreenerColumnSelector::Index(index) => columns
            .iter()
            .find(|column| column.index == *index)
            .cloned()
            .ok_or_else(|| {
                AppError::new(
                    ErrorKind::Validation,
                    format!("No visible Screener column found at index {index}"),
                )
                .with_details(json!({ "columns": column_targets_payload(columns) }))
            }),
        ScreenerColumnSelector::Name(name) => {
            let needle = name.to_lowercase();
            let matches = columns
                .iter()
                .filter(|column| column.name.to_lowercase().contains(&needle))
                .cloned()
                .collect::<Vec<_>>();
            match matches.len() {
                0 => Err(AppError::new(
                    ErrorKind::Validation,
                    format!("No visible Screener column matched name {name:?}"),
                )
                .with_details(json!({ "columns": column_targets_payload(columns) }))),
                1 => Ok(matches[0].clone()),
                _ => Err(AppError::new(
                    ErrorKind::Validation,
                    format!("Screener column name {name:?} matched multiple columns"),
                )
                .with_details(json!({ "matches": column_targets_payload(&matches) }))),
            }
        }
    }
}

fn column_target_payload(column: &ScreenerColumnTarget) -> Value {
    json!({
        "index": column.index,
        "name": column.name,
    })
}

fn column_targets_payload(columns: &[ScreenerColumnTarget]) -> Vec<Value> {
    columns.iter().map(column_target_payload).collect()
}

fn ensure_test_screener_screen_for_column_mutation(
    screen_title: &str,
    operation: &str,
) -> Result<(), AppError> {
    if is_test_screener_screen_name(screen_title) {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorKind::Validation,
            format!(
                "Screener columns {operation} mutation is limited to test screen names containing CLI-Test or テスト"
            ),
        )
        .with_details(json!({ "screen_title": screen_title })))
    }
}

fn storage_columns_from_config(
    config: &Value,
    visible_columns: &[ScreenerColumnTarget],
) -> Vec<ScreenerStorageColumnTarget> {
    config
        .get("columns")
        .and_then(Value::as_array)
        .map(|columns| {
            columns
                .iter()
                .enumerate()
                .filter_map(|(index, column)| {
                    let id = column.get("id").and_then(Value::as_str)?.trim();
                    if id.is_empty() {
                        return None;
                    }
                    let params = column
                        .get("params")
                        .cloned()
                        .unwrap_or_else(|| Value::Object(Map::new()));
                    Some(ScreenerStorageColumnTarget {
                        index,
                        id: id.to_string(),
                        name: visible_columns.get(index).map(|column| column.name.clone()),
                        params,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn storage_column_target_payload(column: &ScreenerStorageColumnTarget) -> Value {
    json!({
        "index": column.index,
        "id": column.id,
        "name": column.name,
        "name_source": column.name.as_ref().map(|_| "visible_column_index"),
        "params": column.params,
    })
}

fn storage_column_targets_payload(columns: &[ScreenerStorageColumnTarget]) -> Vec<Value> {
    columns.iter().map(storage_column_target_payload).collect()
}

fn storage_column_update_payload(columns: &[ScreenerStorageColumnTarget]) -> Vec<Value> {
    columns
        .iter()
        .map(|column| {
            json!({
                "id": column.id,
                "params": column.params,
            })
        })
        .collect()
}

fn ensure_storage_column_index(
    columns: &[ScreenerStorageColumnTarget],
    index: usize,
) -> Result<(), AppError> {
    if index < columns.len() {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorKind::Validation,
            format!("No saved Screener column found at index {index}"),
        )
        .with_details(json!({
            "column_count": columns.len(),
            "columns": storage_column_targets_payload(columns),
        })))
    }
}

fn remove_storage_column(
    columns: &[ScreenerStorageColumnTarget],
    index: usize,
) -> Vec<ScreenerStorageColumnTarget> {
    columns
        .iter()
        .enumerate()
        .filter(|(column_index, _)| *column_index != index)
        .map(|(_, column)| column.clone())
        .enumerate()
        .map(|(index, mut column)| {
            column.index = index;
            column
        })
        .collect()
}

fn add_storage_column(
    columns: &[ScreenerStorageColumnTarget],
    request: &ScreenerColumnAddRequest,
) -> Result<Vec<ScreenerStorageColumnTarget>, AppError> {
    if let Some(after_index) = request.after_index {
        ensure_storage_column_index(columns, after_index)?;
    }
    let insert_index = request
        .after_index
        .map(|index| index + 1)
        .unwrap_or(columns.len());
    let mut added = columns.to_vec();
    added.insert(
        insert_index,
        ScreenerStorageColumnTarget {
            index: insert_index,
            id: request.id.clone(),
            name: None,
            params: request.params.clone(),
        },
    );
    Ok(added
        .into_iter()
        .enumerate()
        .map(|(index, mut column)| {
            column.index = index;
            column
        })
        .collect())
}

fn reorder_storage_columns(
    columns: &[ScreenerStorageColumnTarget],
    from_index: usize,
    to_index: usize,
) -> Vec<ScreenerStorageColumnTarget> {
    let mut reordered = columns.to_vec();
    let column = reordered.remove(from_index);
    reordered.insert(to_index, column);
    reordered
        .into_iter()
        .enumerate()
        .map(|(index, mut column)| {
            column.index = index;
            column
        })
        .collect()
}

fn storage_column_order_matches(
    actual: &[ScreenerStorageColumnTarget],
    expected: &[ScreenerStorageColumnTarget],
) -> bool {
    actual.len() == expected.len()
        && actual
            .iter()
            .zip(expected)
            .all(|(actual, expected)| actual.id == expected.id && actual.params == expected.params)
}

async fn save_screener_storage_columns(
    runtime: &mut impl RuntimeEvaluator,
    config: &Value,
    columns: &[ScreenerStorageColumnTarget],
) -> Result<Value, AppError> {
    let mut screen = config.get("storage_screen").cloned().ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener active screen storage payload was not available",
        )
        .with_details(config.clone())
    })?;
    let screen_object = screen.as_object_mut().ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener active screen storage payload was not an object",
        )
        .with_details(config.clone())
    })?;
    screen_object.insert(
        "default_custom_column_set".to_string(),
        Value::Array(storage_column_update_payload(columns)),
    );
    screen_object.insert(
        "active_column_set".to_string(),
        Value::String("custom".to_string()),
    );
    let screen_json = serde_json::to_string(&screen).map_err(|error| {
        AppError::new(
            ErrorKind::Internal,
            format!("Failed to serialize Screener storage payload: {error}"),
        )
    })?;

    let result = runtime
        .evaluate(
            &expanded_expression(&format!(
                r#"
                (async function() {{
                    REPLACE_HELPERS
                    var initData = window.initData || {{}};
                    var storageUrl = initData.SCREENER_STORAGE_URL;
                    if (!storageUrl) {{
                        return {{
                            saved: false,
                            reason: 'missing_screener_storage_init_data'
                        }};
                    }}
                    var screen = {screen_json};
                    var base = String(storageUrl).replace(/\/$/, '') + '/api/v2/screens/';
                    var response = await fetch(base + encodeURIComponent(String(screen.id)) + '/', {{
                        method: 'PUT',
                        credentials: 'include',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify(screen)
                    }});
                    var body = await response.json().catch(function() {{ return null; }});
                    return {{
                        saved: response.ok,
                        status: response.status,
                        status_text: response.statusText,
                        screen_id: String(screen.id || ''),
                        column_count: Array.isArray(screen.default_custom_column_set)
                            ? screen.default_custom_column_set.length
                            : null,
                        response_column_count: body && Array.isArray(body.default_custom_column_set)
                            ? body.default_custom_column_set.length
                            : null,
                        response_title: body && body.title ? String(body.title) : null
                    }};
                }})()
                "#
            )),
            true,
        )
        .await?;

    if value_bool(&result, "saved") {
        Ok(result)
    } else {
        Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener storage column save request failed",
        )
        .with_details(result))
    }
}

async fn read_column_actions(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &expanded_expression(SCREENER_COLUMNS_ACTIONS_EXPRESSION),
            true,
        )
        .await
}

const SCREENER_COLUMNS_ACTIONS_EXPRESSION: &str = r#"
(async function() {
    function sleep(ms) {
        return new Promise(function(resolve) { setTimeout(resolve, ms); });
    }
    REPLACE_HELPERS
    function actionText(text) {
        if (/^昇順で並べ替え$|^Sort ascending$/i.test(text)) return 'sort_ascending';
        if (/^降順で並べ替え$|^Sort descending$/i.test(text)) return 'sort_descending';
        if (/^左に移動$|^Move left$/i.test(text)) return 'move_left';
        if (/^右に移動$|^Move right$/i.test(text)) return 'move_right';
        if (/^先頭に移動$|^Move to beginning$/i.test(text)) return 'move_first';
        if (/^末尾に移動$|^Move to end$/i.test(text)) return 'move_last';
        if (/削除|Remove|非表示|Hide/i.test(text)) return 'remove';
        if (/リセット|デフォルト|Reset|Default/i.test(text)) return 'reset';
        return null;
    }
    function mouseContextClick(el) {
        var rect = el.getBoundingClientRect();
        var x = rect.left + rect.width / 2;
        var y = rect.top + rect.height / 2;
        ['mouseover', 'mousedown', 'mouseup', 'contextmenu'].forEach(function(type) {
            el.dispatchEvent(new MouseEvent(type, {
                bubbles: true,
                cancelable: true,
                clientX: x,
                clientY: y,
                button: 2,
                buttons: type === 'mousedown' ? 2 : 0,
                view: window
            }));
        });
    }
    function collectActionTexts(root) {
        var seen = {};
        var actions = [];
        var nodes = Array.from(root.querySelectorAll('button, [role="menuitem"], [role="option"], [role="button"], div, span')).filter(visible);
        nodes.forEach(function(el) {
            var text = textOf(el);
            var kind = actionText(text);
            if (!kind || seen[kind + ':' + text]) return;
            seen[kind + ':' + text] = true;
            actions.push({
                index: actions.length,
                text: text,
                kind: kind,
                enabled: !(el.disabled || el.getAttribute('aria-disabled') === 'true')
            });
        });
        return actions;
    }
    function findColumnSettingsButton() {
        var table = visibleElements('table').find(function(candidate) {
            return candidate.querySelector('th');
        });
        if (!table) return null;
        var headers = Array.from(table.querySelectorAll('th')).filter(visible);
        for (var i = headers.length - 1; i >= 0; i--) {
            var header = headers[i];
            var text = textOf(header);
            var buttons = Array.from(header.querySelectorAll('button, [role="button"], [title], [aria-label]')).filter(visible);
            var explicit = buttons.find(function(button) {
                var label = [button.getAttribute('title'), button.getAttribute('aria-label'), textOf(button)].filter(Boolean).join(' ');
                return /カラムの設定|Column settings/i.test(label);
            });
            if (explicit) return explicit;
            if (!text && buttons.length) return buttons[buttons.length - 1];
        }
        return null;
    }
    function findColumnSettingsPanel() {
        var panels = visibleElements('[role="dialog"], [class*="popover"], [class*="portal"], [class*="menu"]');
        return panels.find(function(panel) {
            var text = textOf(panel);
            return /カラム|Column/i.test(text) &&
                (/銘柄情報|マーケットデータ|テクニカル|Fundamental|Technical|Market/i.test(text) ||
                 /検索|Search/i.test(text));
        }) || null;
    }
    function collectColumnCategories(panel) {
        var seen = {};
        var categories = [];
        var nodes = Array.from(panel.querySelectorAll('[role="option"], button, [role="button"], div, span')).filter(visible);
        nodes.forEach(function(el) {
            var text = textOf(el);
            if (!text || text.length > 80) return;
            var match = text.match(/^(銘柄情報|マーケットデータ|テクニカル|ファンダメンタル|評価|成長率|マージン|配当|Symbol info|Market data|Technical|Fundamental|Ratings|Growth|Margins|Dividends)\s*([0-9]+)?$/i);
            var key = match && match[1] ? match[1].toLowerCase() : text;
            if (!match || seen[key]) return;
            seen[key] = true;
            categories.push({
                index: categories.length,
                text: text,
                count: match[2] ? Number(match[2]) : null
            });
        });
        return categories;
    }
    var state = readScreenerState(0);
    var settingsButton = findColumnSettingsButton();
    var settingsOpened = false;
    var categories = [];
    if (settingsButton) {
        mouseClick(settingsButton);
        for (var i = 0; i < 10; i++) {
            var panel = findColumnSettingsPanel();
            if (panel) {
                settingsOpened = true;
                categories = collectColumnCategories(panel);
                break;
            }
            await sleep(100);
        }
    }
    if (settingsButton && settingsOpened) {
        mouseClick(settingsButton);
        await sleep(50);
    }

    var headerMenuActions = [];
    var allActions = headerMenuActions;
    var removeSupported = allActions.some(function(action) { return action.kind === 'remove'; });
    var resetSupported = allActions.some(function(action) { return action.kind === 'reset'; });
    return {
        source: 'ui_screener_dialog',
        action: 'columns_actions',
        open: !!state.open,
        screen_title: state.screen_title || null,
        settings_button_found: !!settingsButton,
        settings_opened: settingsOpened,
        categories: categories,
        header_menu_actions: headerMenuActions,
        remove_supported: removeSupported,
        reset_supported: resetSupported,
        unavailable_reason: removeSupported ? null : 'visible_column_remove_action_not_found'
    };
})()
"#;

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::super::test_support::FakeRuntime;
    use super::super::validation::validate_screener_column_add_request;
    use super::*;

    fn storage_config(title: &str, columns: Vec<Value>) -> Value {
        json!({
            "storage_available": true,
            "fetch_ok": true,
            "title_matches": true,
            "screen_id": "screen-test",
            "screen_title": title,
            "active_column_set": "custom",
            "storage_screen": {
                "id": "screen-test",
                "title": title,
                "active_column_set": "custom",
                "default_custom_column_set": columns,
                "filters": []
            },
            "columns": columns
        })
    }

    fn storage_column(id: &str) -> Value {
        json!({ "id": id, "params": {} })
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
    async fn screener_columns_actions_reports_detected_actions() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "米国株（テスト用）",
                "columns": ["Symbol", "Price", "Change %"],
                "column_count": 3
            }),
            json!({
                "settings_button_found": true,
                "settings_opened": true,
                "categories": [
                    { "index": 0, "text": "銘柄情報26", "count": 26 },
                    { "index": 1, "text": "テクニカル39", "count": 39 }
                ],
                "header_menu_actions": [
                    { "index": 0, "text": "左に移動", "kind": "move_left", "enabled": true }
                ],
                "remove_supported": false,
                "reset_supported": false,
                "unavailable_reason": "visible_column_remove_action_not_found"
            }),
        ]);

        let result = screener_columns_actions(&mut runtime).await.unwrap();

        assert_eq!(result["action"], "columns_actions");
        assert_eq!(result["screen_title"], "米国株（テスト用）");
        assert_eq!(result["column_count"], 3);
        assert_eq!(result["settings_opened"], true);
        assert_eq!(result["categories"].as_array().unwrap().len(), 2);
        assert_eq!(result["header_menu_actions"][0]["kind"], "move_left");
        assert_eq!(result["remove_supported"], false);
    }

    #[tokio::test]
    async fn screener_columns_config_returns_storage_columns() {
        let mut runtime = FakeRuntime::new([
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "米国株（テスト用）",
                "columns": ["Symbol", "Price", "Change %"],
                "column_count": 3
            }),
            storage_config(
                "米国株（テスト用）",
                vec![
                    storage_column("TickerUniversal"),
                    storage_column("Price"),
                    json!({ "id": "Change", "params": { "resolution": "TimeResolution1D" } }),
                ],
            ),
        ]);

        let result = screener_columns_config(&mut runtime).await.unwrap();

        assert_eq!(result["action"], "columns_config");
        assert_eq!(result["scope"], "screen_storage_api");
        assert_eq!(result["screen_title"], "米国株（テスト用）");
        assert_eq!(result["column_count"], 3);
        assert_eq!(result["columns"][0]["id"], "TickerUniversal");
        assert_eq!(result["columns"][1]["name"], "Price");
        assert_eq!(
            result["columns"][2]["params"]["resolution"],
            "TimeResolution1D"
        );
    }

    #[tokio::test]
    async fn screener_columns_remove_dry_run_returns_target_without_mutation() {
        let mut runtime = FakeRuntime::new([
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "米国株（テスト用）",
                "columns": ["Symbol", "Price", "Change %"],
                "column_count": 3
            }),
            storage_config(
                "米国株（テスト用）",
                vec![
                    storage_column("TickerUniversal"),
                    storage_column("Price"),
                    storage_column("Change"),
                ],
            ),
        ]);

        let result = screener_columns_remove(
            &mut runtime,
            ScreenerColumnSelector::Name("Change".to_string()),
            true,
        )
        .await
        .unwrap();

        assert_eq!(result["action"], "columns_remove");
        assert_eq!(result["scope"], "screen_storage_api");
        assert_eq!(result["dry_run"], true);
        assert_eq!(result["removed"], false);
        assert_eq!(result["target_column"]["index"], 2);
        assert_eq!(result["target_column"]["name"], "Change %");
        assert_eq!(result["target_column"]["id"], "Change");
        assert_eq!(result["after_column_count"], 2);
        assert_eq!(runtime.mouse_events.len(), 0);
    }

    #[tokio::test]
    async fn screener_columns_remove_saves_storage_and_post_checks() {
        let mut runtime = FakeRuntime::new([
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "CLI-Test1",
                "columns": ["Symbol", "Price", "Change %"],
                "column_count": 3
            }),
            storage_config(
                "CLI-Test1",
                vec![
                    storage_column("TickerUniversal"),
                    storage_column("Price"),
                    storage_column("Change"),
                ],
            ),
            json!({ "saved": true, "screen_id": "screen-test", "column_count": 2 }),
            storage_config(
                "CLI-Test1",
                vec![storage_column("TickerUniversal"), storage_column("Price")],
            ),
        ]);

        let result = screener_columns_remove(&mut runtime, ScreenerColumnSelector::Index(2), false)
            .await
            .unwrap();

        assert_eq!(result["action"], "columns_remove");
        assert_eq!(result["dry_run"], false);
        assert_eq!(result["removed"], true);
        assert_eq!(result["screen_title"], "CLI-Test1");
        assert_eq!(result["target_column"]["id"], "Change");
        assert_eq!(result["before_column_count"], 3);
        assert_eq!(result["after_column_count"], 2);
        assert_eq!(result["columns"][1]["id"], "Price");
        assert!(runtime.evaluated.iter().any(|(expression, _)| {
            expression.contains("default_custom_column_set")
                && expression.contains("TickerUniversal")
                && !expression.contains("\"Change\"")
        }));
    }

    #[tokio::test]
    async fn screener_columns_remove_refuses_non_test_screen_mutation() {
        let mut runtime = FakeRuntime::new([
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "Production",
                "columns": ["Symbol", "Price"],
                "column_count": 2
            }),
            storage_config(
                "Production",
                vec![storage_column("TickerUniversal"), storage_column("Price")],
            ),
        ]);

        let error = screener_columns_remove(&mut runtime, ScreenerColumnSelector::Index(1), false)
            .await
            .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
    }

    #[tokio::test]
    async fn screener_columns_add_dry_run_returns_expected_order() {
        let mut runtime = FakeRuntime::new([
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "米国株（テスト用）",
                "columns": ["Symbol", "Price"],
                "column_count": 2
            }),
            storage_config(
                "米国株（テスト用）",
                vec![storage_column("TickerUniversal"), storage_column("Price")],
            ),
        ]);
        let request = validate_screener_column_add_request(
            "Change",
            Some(r#"{"resolution":"TimeResolution1D"}"#),
            Some(1),
            true,
        )
        .unwrap();

        let result = screener_columns_add(&mut runtime, request).await.unwrap();

        assert_eq!(result["action"], "columns_add");
        assert_eq!(result["scope"], "screen_storage_api");
        assert_eq!(result["dry_run"], true);
        assert_eq!(result["added"], false);
        assert_eq!(result["inserted_index"], 2);
        assert_eq!(result["target_column"]["id"], "Change");
        assert_eq!(
            result["target_column"]["params"]["resolution"],
            "TimeResolution1D"
        );
        assert_eq!(result["after_column_count"], 3);
        assert_eq!(result["after_columns"][2]["id"], "Change");
    }

    #[tokio::test]
    async fn screener_columns_add_rejects_out_of_range_after_index() {
        let mut runtime = FakeRuntime::new([
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "米国株（テスト用）",
                "columns": ["Symbol", "Price"],
                "column_count": 2
            }),
            storage_config(
                "米国株（テスト用）",
                vec![storage_column("TickerUniversal"), storage_column("Price")],
            ),
        ]);
        let request = validate_screener_column_add_request("Change", None, Some(2), true).unwrap();

        let error = screener_columns_add(&mut runtime, request)
            .await
            .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
    }

    #[tokio::test]
    async fn screener_columns_add_saves_storage_and_post_checks() {
        let mut runtime = FakeRuntime::new([
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "CLI-Test1",
                "columns": ["Symbol", "Price"],
                "column_count": 2
            }),
            storage_config(
                "CLI-Test1",
                vec![storage_column("TickerUniversal"), storage_column("Price")],
            ),
            json!({ "saved": true, "screen_id": "screen-test", "column_count": 3 }),
            storage_config(
                "CLI-Test1",
                vec![
                    storage_column("TickerUniversal"),
                    storage_column("Price"),
                    json!({ "id": "Change", "params": { "resolution": "TimeResolution1D" } }),
                ],
            ),
        ]);
        let request = validate_screener_column_add_request(
            "Change",
            Some(r#"{"resolution":"TimeResolution1D"}"#),
            Some(1),
            false,
        )
        .unwrap();

        let result = screener_columns_add(&mut runtime, request).await.unwrap();

        assert_eq!(result["action"], "columns_add");
        assert_eq!(result["dry_run"], false);
        assert_eq!(result["added"], true);
        assert_eq!(result["screen_title"], "CLI-Test1");
        assert_eq!(result["inserted_index"], 2);
        assert_eq!(result["target_column"]["id"], "Change");
        assert_eq!(result["before_column_count"], 2);
        assert_eq!(result["after_column_count"], 3);
        assert_eq!(result["columns"][2]["id"], "Change");
        assert!(runtime.evaluated.iter().any(|(expression, _)| {
            expression.contains("default_custom_column_set")
                && expression.contains("\"Change\"")
                && expression.contains("TimeResolution1D")
        }));
    }

    #[tokio::test]
    async fn screener_columns_add_refuses_non_test_screen_mutation() {
        let mut runtime = FakeRuntime::new([
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "Production",
                "columns": ["Symbol", "Price"],
                "column_count": 2
            }),
            storage_config(
                "Production",
                vec![storage_column("TickerUniversal"), storage_column("Price")],
            ),
        ]);
        let request = validate_screener_column_add_request("Change", None, None, false).unwrap();

        let error = screener_columns_add(&mut runtime, request)
            .await
            .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
    }

    #[tokio::test]
    async fn screener_columns_reorder_dry_run_returns_expected_order() {
        let mut runtime = FakeRuntime::new([
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "米国株（テスト用）",
                "columns": ["Symbol", "Price", "Change %"],
                "column_count": 3
            }),
            storage_config(
                "米国株（テスト用）",
                vec![
                    storage_column("TickerUniversal"),
                    storage_column("Price"),
                    storage_column("Change"),
                ],
            ),
        ]);

        let result = screener_columns_reorder(&mut runtime, 2, 1, true)
            .await
            .unwrap();

        assert_eq!(result["action"], "columns_reorder");
        assert_eq!(result["scope"], "screen_storage_api");
        assert_eq!(result["dry_run"], true);
        assert_eq!(result["reordered"], false);
        assert_eq!(result["target_column"]["id"], "Change");
        assert_eq!(result["after_columns"][1]["id"], "Change");
        assert_eq!(result["after_columns"][2]["id"], "Price");
    }

    #[tokio::test]
    async fn screener_columns_reorder_saves_storage_and_post_checks() {
        let mut runtime = FakeRuntime::new([
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "CLI-Test1",
                "columns": ["Symbol", "Price", "Change %"],
                "column_count": 3
            }),
            storage_config(
                "CLI-Test1",
                vec![
                    storage_column("TickerUniversal"),
                    storage_column("Price"),
                    storage_column("Change"),
                ],
            ),
            json!({ "saved": true, "screen_id": "screen-test", "column_count": 3 }),
            storage_config(
                "CLI-Test1",
                vec![
                    storage_column("TickerUniversal"),
                    storage_column("Change"),
                    storage_column("Price"),
                ],
            ),
        ]);

        let result = screener_columns_reorder(&mut runtime, 2, 1, false)
            .await
            .unwrap();

        assert_eq!(result["action"], "columns_reorder");
        assert_eq!(result["dry_run"], false);
        assert_eq!(result["reordered"], true);
        assert_eq!(result["target_column"]["id"], "Change");
        assert_eq!(result["columns"][1]["id"], "Change");
        assert_eq!(result["columns"][2]["id"], "Price");
        assert!(runtime.evaluated.iter().any(|(expression, _)| {
            expression.contains("\"Change\"") && expression.contains("\"Price\"")
        }));
    }

    #[tokio::test]
    async fn screener_columns_remove_rejects_ambiguous_name() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "columns": ["Price", "Price Change", "Change %"],
                "column_count": 3
            }),
        ]);

        let error = screener_columns_remove(
            &mut runtime,
            ScreenerColumnSelector::Name("Price".to_string()),
            true,
        )
        .await
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(error.details.is_some());
    }
}
