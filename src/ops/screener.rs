use serde_json::{Map, Value, json};
use tokio::time::{Duration, sleep};

use crate::{
    cdp::{MouseEvent, MouseEventType, RuntimeEvaluator},
    error::{AppError, ErrorKind},
};

use super::{
    common::{js_string, require_finite},
    ui::ui_keyboard,
};

const SCREENER_SOURCE: &str = "ui_screener_dialog";
const DEFAULT_SCREENER_LIMIT: usize = 20;
const MAX_SCREENER_LIMIT: usize = 100;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScreenerFilterSelector {
    Index(usize),
    Text(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScreenerFilterModifyRequest {
    pub selector: ScreenerFilterSelector,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub dry_run: bool,
    preset_label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScreenerColumnSelector {
    Index(usize),
    Name(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScreenerFilterTarget {
    index: usize,
    text: String,
    data_name: String,
    visible: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScreenerScreenTarget {
    index: usize,
    name: String,
    active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScreenerScreenAction {
    index: usize,
    text: String,
    kind: String,
    enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScreenerColumnTarget {
    index: usize,
    name: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ScreenMenuClickPoint {
    x: f64,
    y: f64,
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

pub fn validate_screener_filter_modify_request(
    index: Option<usize>,
    text: Option<&str>,
    min: Option<f64>,
    max: Option<f64>,
    dry_run: bool,
) -> Result<ScreenerFilterModifyRequest, AppError> {
    let selector = validate_screener_filter_selector(index, text)?;
    let preset_label = screener_filter_range_preset_label(min, max)?;
    Ok(ScreenerFilterModifyRequest {
        selector,
        min,
        max,
        dry_run,
        preset_label,
    })
}

pub fn validate_screener_column_selector(
    index: Option<usize>,
    name: Option<&str>,
) -> Result<ScreenerColumnSelector, AppError> {
    let name = name.map(str::trim).filter(|value| !value.is_empty());
    match (index, name) {
        (Some(_), Some(_)) => Err(AppError::new(
            ErrorKind::Validation,
            "--index and --name are mutually exclusive",
        )),
        (Some(index), None) => Ok(ScreenerColumnSelector::Index(index)),
        (None, Some(name)) => Ok(ScreenerColumnSelector::Name(name.to_string())),
        (None, None) => Err(AppError::new(
            ErrorKind::Validation,
            "Either --index or --name is required",
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

pub fn validate_screener_screen_name(name: &str) -> Result<String, AppError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "--name must not be empty",
        ));
    }
    Ok(name.to_string())
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

pub async fn screener_screens_actions(
    runtime: &mut impl RuntimeEvaluator,
) -> Result<Value, AppError> {
    let mut session = ScreenerMutationSession::open(runtime).await?;
    let state = read_screener_state(session.runtime, None).await?;
    ensure_dialog_open(&state)?;
    let result = session
        .runtime
        .evaluate(
            &expanded_expression(SCREENER_SCREEN_ACTIONS_EXPRESSION),
            true,
        )
        .await?;
    ensure_screen_menu_opened(&result)?;
    let actions = screen_actions_from_menu(&result);
    let save_actions = actions
        .iter()
        .filter(|action| action.kind == "save")
        .collect::<Vec<_>>();
    let opened_for_read = session.opened_for_mutation;
    let restored_open_state = session.restored_open_state;
    session.restore().await?;
    Ok(json!({
        "source": SCREENER_SOURCE,
        "scope": "screen_title_menu",
        "open": value_bool(&state, "open"),
        "opened_for_read": opened_for_read,
        "restored_open_state": restored_open_state,
        "screen_title": state.get("screen_title").cloned().unwrap_or(Value::Null),
        "action_count": actions.len(),
        "save_available": !save_actions.is_empty(),
        "save_enabled": save_actions.iter().any(|action| action.enabled),
        "actions": screen_actions_payload(&actions),
    }))
}

pub async fn screener_screens_list(
    runtime: &mut impl RuntimeEvaluator,
    catalog: bool,
) -> Result<Value, AppError> {
    let mut session = ScreenerMutationSession::open(runtime).await?;
    let state = read_screener_state(session.runtime, None).await?;
    ensure_dialog_open(&state)?;
    if catalog {
        open_screen_catalog_from_menu(session.runtime).await?;
        let result = session
            .runtime
            .evaluate(
                &expanded_expression(SCREENER_SCREEN_CATALOG_LIST_EXPRESSION),
                true,
            )
            .await?;
        ensure_screen_catalog_opened(&result)?;
        let screens = screen_targets_from_menu(&result);
        let opened_for_read = session.opened_for_mutation;
        let restored_open_state = session.restored_open_state;
        session.restore().await?;
        return Ok(json!({
            "source": SCREENER_SOURCE,
            "scope": "screen_catalog",
            "open": value_bool(&state, "open"),
            "opened_for_read": opened_for_read,
            "restored_open_state": restored_open_state,
            "screen_title": state.get("screen_title").cloned().unwrap_or(Value::Null),
            "screen_count": screens.len(),
            "screens": screen_targets_payload(&screens),
        }));
    }

    let result = session
        .runtime
        .evaluate(
            &expanded_expression(SCREENER_SCREEN_MENU_LIST_EXPRESSION),
            true,
        )
        .await?;
    ensure_screen_menu_opened(&result)?;
    let screens = screen_targets_from_menu(&result);
    let opened_for_read = session.opened_for_mutation;
    let restored_open_state = session.restored_open_state;
    session.restore().await?;
    Ok(json!({
        "source": SCREENER_SOURCE,
        "scope": "screen_title_menu",
        "open": value_bool(&state, "open"),
        "opened_for_read": opened_for_read,
        "restored_open_state": restored_open_state,
        "screen_title": state.get("screen_title").cloned().unwrap_or(Value::Null),
        "catalog_entry_found": value_bool(&result, "catalog_entry_found"),
        "screen_count": screens.len(),
        "screens": screen_targets_payload(&screens),
    }))
}

pub async fn screener_screens_switch(
    runtime: &mut impl RuntimeEvaluator,
    name: &str,
    dry_run: bool,
    catalog: bool,
) -> Result<Value, AppError> {
    let name = validate_screener_screen_name(name)?;
    let mut session = ScreenerMutationSession::open(runtime).await?;
    let before_state = read_screener_state(session.runtime, None).await?;
    ensure_dialog_open(&before_state)?;

    let scope = if catalog {
        "screen_catalog"
    } else {
        "screen_title_menu"
    };
    let screen_list = if catalog {
        open_screen_catalog_from_menu(session.runtime).await?;
        let result = session
            .runtime
            .evaluate(
                &expanded_expression(SCREENER_SCREEN_CATALOG_LIST_EXPRESSION),
                true,
            )
            .await?;
        ensure_screen_catalog_opened(&result)?;
        result
    } else {
        let result = session
            .runtime
            .evaluate(
                &expanded_expression(SCREENER_SCREEN_MENU_LIST_EXPRESSION),
                true,
            )
            .await?;
        ensure_screen_menu_opened(&result)?;
        result
    };
    let screens = screen_targets_from_menu(&screen_list);
    let target = resolve_screen_target(&screens, &name)?;
    let active_before = before_state
        .get("screen_title")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();

    if dry_run {
        let close_result = session.restore().await;
        close_result?;
        return Ok(json!({
            "source": SCREENER_SOURCE,
            "action": "screen_switch",
            "scope": scope,
            "dry_run": true,
            "switched": false,
            "already_active": active_before == name,
            "opened_for_mutation": session.opened_for_mutation,
            "restored_open_state": session.restored_open_state,
            "before_screen_title": active_before,
            "after_screen_title": active_before,
            "target_screen": screen_target_payload(&target),
            "screen_count": screens.len(),
            "screens": screen_targets_payload(&screens),
        }));
    }

    if active_before == name {
        let close_result = session.restore().await;
        close_result?;
        return Ok(json!({
            "source": SCREENER_SOURCE,
            "action": "screen_switch",
            "scope": scope,
            "dry_run": false,
            "switched": false,
            "already_active": true,
            "opened_for_mutation": session.opened_for_mutation,
            "restored_open_state": session.restored_open_state,
            "before_screen_title": active_before,
            "after_screen_title": name,
            "target_screen": screen_target_payload(&target),
            "screen_count": screens.len(),
            "screens": screen_targets_payload(&screens),
        }));
    }

    let click_result = if catalog {
        click_screen_catalog_target(session.runtime, &target).await
    } else {
        click_screen_menu_target(session.runtime, &target).await
    };
    if let Err(err) = click_result {
        let _ = session.restore().await;
        return Err(err);
    }
    let after_state_result = wait_for_screen_title(session.runtime, &name).await;
    let close_result = session.restore().await;
    let after_state = after_state_result?;
    let after_title = after_state
        .get("screen_title")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    close_result?;

    Ok(json!({
        "source": SCREENER_SOURCE,
        "action": "screen_switch",
        "scope": scope,
        "dry_run": false,
        "switched": true,
        "already_active": false,
        "opened_for_mutation": session.opened_for_mutation,
        "restored_open_state": session.restored_open_state,
        "before_screen_title": active_before,
        "after_screen_title": after_title,
        "target_screen": screen_target_payload(&target),
        "screen_count": screens.len(),
        "screens": screen_targets_payload(&screens),
    }))
}

pub async fn screener_screens_save(
    runtime: &mut impl RuntimeEvaluator,
    dry_run: bool,
) -> Result<Value, AppError> {
    let mut session = ScreenerMutationSession::open(runtime).await?;
    let before_state = read_screener_state(session.runtime, None).await?;
    ensure_dialog_open(&before_state)?;

    let result = session
        .runtime
        .evaluate(
            &expanded_expression(SCREENER_SCREEN_SAVE_POINT_EXPRESSION),
            true,
        )
        .await?;
    ensure_screen_menu_opened(&result)?;
    if !value_bool(&result, "found") {
        let _ = session.restore().await;
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener screen save action was not found",
        )
        .with_details(result));
    }
    if !value_bool(&result, "enabled") {
        let _ = session.restore().await;
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener screen save action is disabled",
        )
        .with_details(result));
    }

    let actions = screen_actions_from_menu(&result);
    let target_action = resolve_save_screen_action(&actions)?;
    let before_title = before_state
        .get("screen_title")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();

    if dry_run {
        let close_result = session.restore().await;
        close_result?;
        return Ok(json!({
            "source": SCREENER_SOURCE,
            "action": "screen_save",
            "scope": "screen_title_menu",
            "dry_run": true,
            "clicked": false,
            "save_requested": false,
            "confirmation": "dry_run",
            "opened_for_mutation": session.opened_for_mutation,
            "restored_open_state": session.restored_open_state,
            "screen_title": before_title,
            "target_action": screen_action_payload(&target_action),
            "action_count": actions.len(),
            "actions": screen_actions_payload(&actions),
        }));
    }

    let point = screen_menu_click_point(&result)?;
    dispatch_screen_menu_click(session.runtime, point).await?;
    let after_state = wait_for_screen_save_post_check(session.runtime, &before_title).await;
    let close_result = session.restore().await;
    let after_state = after_state?;
    close_result?;

    Ok(json!({
        "source": SCREENER_SOURCE,
        "action": "screen_save",
        "scope": "screen_title_menu",
        "dry_run": false,
        "clicked": true,
        "save_requested": true,
        "confirmation": "not_observable",
        "opened_for_mutation": session.opened_for_mutation,
        "restored_open_state": session.restored_open_state,
        "screen_title": before_title,
        "after_screen_title": after_state.get("screen_title").cloned().unwrap_or(Value::Null),
        "blocking_dialog_found": value_bool(&after_state, "blocking_dialog_found"),
        "target_action": screen_action_payload(&target_action),
        "action_count": actions.len(),
        "actions": screen_actions_payload(&actions),
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

pub async fn screener_filters_actions(
    runtime: &mut impl RuntimeEvaluator,
) -> Result<Value, AppError> {
    let mut session = ScreenerMutationSession::open(runtime).await?;
    let state = read_screener_state(session.runtime, None).await?;
    ensure_dialog_open(&state)?;
    let filters = filter_targets_from_state(&state);
    let actions = read_filter_actions(session.runtime).await?;
    let close_result = session.restore().await;
    close_result?;

    Ok(json!({
        "source": SCREENER_SOURCE,
        "action": "filters_actions",
        "open": value_bool(&state, "open"),
        "screen_title": state.get("screen_title").cloned().unwrap_or(Value::Null),
        "filter_count": filters.len(),
        "filters": filter_targets_payload(&filters),
        "opened_for_read": session.opened_for_mutation,
        "restored_open_state": session.restored_open_state,
        "add_button_found": value_bool(&actions, "add_button_found"),
        "add_supported": value_bool(&actions, "add_supported"),
        "numeric_modify_supported": value_bool(&actions, "numeric_modify_supported"),
        "candidate_filter": actions.get("candidate_filter").cloned().unwrap_or(Value::Null),
        "range_options": actions.get("range_options").cloned().unwrap_or_else(|| json!([])),
        "unavailable_reason": actions.get("unavailable_reason").cloned().unwrap_or(Value::Null),
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

pub async fn screener_filters_modify(
    runtime: &mut impl RuntimeEvaluator,
    request: ScreenerFilterModifyRequest,
) -> Result<Value, AppError> {
    let mut session = ScreenerMutationSession::open(runtime).await?;
    let before_state = read_screener_state(session.runtime, None).await?;
    ensure_dialog_open(&before_state)?;
    let before_filters = filter_targets_from_state(&before_state);
    let target = resolve_filter_target(&before_filters, &request.selector)?;
    let requested_range = filter_modify_range_payload(&request);

    if request.dry_run {
        let close_result = session.restore().await;
        close_result?;
        return Ok(json!({
            "source": SCREENER_SOURCE,
            "action": "filter_modify",
            "dry_run": true,
            "modified": false,
            "open": value_bool(&before_state, "open"),
            "opened_for_mutation": session.opened_for_mutation,
            "restored_open_state": session.restored_open_state,
            "before_filter_count": before_filters.len(),
            "after_filter_count": before_filters.len(),
            "target_filter": filter_target_payload(&target),
            "requested_range": requested_range,
        }));
    }

    if normalize_screener_text(&target.text).contains(&request.preset_label) {
        let close_result = session.restore().await;
        close_result?;
        return Ok(json!({
            "source": SCREENER_SOURCE,
            "action": "filter_modify",
            "dry_run": false,
            "modified": false,
            "already_matching": true,
            "open": value_bool(&before_state, "open"),
            "opened_for_mutation": session.opened_for_mutation,
            "restored_open_state": session.restored_open_state,
            "before_filter_count": before_filters.len(),
            "after_filter_count": before_filters.len(),
            "target_filter": filter_target_payload(&target),
            "after_filter": filter_target_payload(&target),
            "requested_range": requested_range,
        }));
    }

    click_filter_range_preset(session.runtime, &target, &request.preset_label).await?;
    let after_state =
        wait_for_filter_modified(session.runtime, &target.data_name, &request.preset_label).await?;
    let after_filters = filter_targets_from_state(&after_state);
    let after_target = after_filters
        .iter()
        .find(|filter| filter.data_name == target.data_name)
        .cloned();
    let close_result = session.restore().await;
    close_result?;

    Ok(json!({
        "source": SCREENER_SOURCE,
        "action": "filter_modify",
        "dry_run": false,
        "modified": true,
        "open": value_bool(&after_state, "open"),
        "opened_for_mutation": session.opened_for_mutation,
        "restored_open_state": session.restored_open_state,
        "before_filter_count": before_filters.len(),
        "after_filter_count": after_filters.len(),
        "target_filter": filter_target_payload(&target),
        "after_filter": after_target.as_ref().map(filter_target_payload).unwrap_or(Value::Null),
        "requested_range": requested_range,
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
    let mut session = ScreenerMutationSession::open(runtime).await?;
    let before_state = read_screener_state(session.runtime, None).await?;
    ensure_dialog_open(&before_state)?;
    let before_columns = column_targets_from_state(&before_state);
    let target = resolve_column_target(&before_columns, &selector)?;
    let actions = read_column_actions(session.runtime).await?;
    let remove_supported = value_bool(&actions, "remove_supported");

    if dry_run {
        let close_result = session.restore().await;
        close_result?;
        return Ok(json!({
            "source": SCREENER_SOURCE,
            "action": "columns_remove",
            "dry_run": true,
            "removed": false,
            "remove_supported": remove_supported,
            "unavailable_reason": actions.get("unavailable_reason").cloned().unwrap_or(Value::Null),
            "opened_for_mutation": session.opened_for_mutation,
            "restored_open_state": session.restored_open_state,
            "before_column_count": before_columns.len(),
            "after_column_count": before_columns.len(),
            "target_column": column_target_payload(&target),
            "columns": column_targets_payload(&before_columns),
        }));
    }

    let close_result = session.restore().await;
    close_result?;

    Err(AppError::new(
        ErrorKind::InternalApiUnavailable,
        "Screener column remove action was not found in the visible TradingView UI",
    )
    .with_details(json!({
        "source": SCREENER_SOURCE,
        "action": "columns_remove",
        "dry_run": false,
        "removed": false,
        "remove_supported": remove_supported,
        "unavailable_reason": actions.get("unavailable_reason").cloned().unwrap_or(Value::Null),
        "target_column": column_target_payload(&target),
        "columns": column_targets_payload(&before_columns),
    })))
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

fn filter_modify_range_payload(request: &ScreenerFilterModifyRequest) -> Value {
    json!({
        "min": request.min,
        "max": request.max,
        "preset_label": request.preset_label,
    })
}

fn screener_filter_range_preset_label(
    min: Option<f64>,
    max: Option<f64>,
) -> Result<String, AppError> {
    if min.is_none() && max.is_none() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Either --min or --max is required",
        ));
    }
    if let Some(value) = min {
        require_finite(value, "--min")?;
        if value < 0.0 {
            return Err(AppError::new(
                ErrorKind::Validation,
                "--min must be greater than or equal to 0",
            ));
        }
    }
    if let Some(value) = max {
        require_finite(value, "--max")?;
        if value <= 0.0 {
            return Err(AppError::new(
                ErrorKind::Validation,
                "--max must be greater than 0",
            ));
        }
    }

    match (min, max) {
        (Some(min), Some(max)) => {
            if !approximately(min, 0.0) || max <= min {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "Screener filter preset ranges currently support --min 0 --max <N>",
                ));
            }
            ensure_supported_filter_preset(max, &[3.0, 5.0, 10.0, 20.0, 30.0], "--max")?;
            Ok(format!("0% 〜 {}%", format_filter_percent(max)))
        }
        (Some(min), None) => {
            ensure_supported_filter_preset(
                min,
                &[
                    3.0, 5.0, 10.0, 15.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0,
                ],
                "--min",
            )?;
            Ok(format!("{}%以上", format_filter_percent(min)))
        }
        (None, Some(_)) => Err(AppError::new(
            ErrorKind::Validation,
            "Screener filter preset ranges do not currently support --max without --min",
        )),
        (None, None) => unreachable!(),
    }
}

fn ensure_supported_filter_preset(
    value: f64,
    supported: &[f64],
    label: &str,
) -> Result<(), AppError> {
    if supported
        .iter()
        .any(|supported| approximately(value, *supported))
    {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorKind::Validation,
            format!("{label} does not match a supported visible Screener preset"),
        )
        .with_details(json!({ "supported": supported })))
    }
}

fn approximately(left: f64, right: f64) -> bool {
    (left - right).abs() < 0.000_001
}

fn format_filter_percent(value: f64) -> String {
    if approximately(value.fract(), 0.0) {
        format!("{}", value.trunc() as i64)
    } else {
        format!("{value}")
    }
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

fn ensure_screen_menu_opened(value: &Value) -> Result<(), AppError> {
    if value_bool(value, "menu_opened") {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Stock Screener screen title menu did not open",
        )
        .with_details(value.clone()))
    }
}

fn ensure_screen_catalog_opened(value: &Value) -> Result<(), AppError> {
    if value_bool(value, "catalog_opened") {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Stock Screener screen catalog did not open",
        )
        .with_details(value.clone()))
    }
}

fn screen_targets_from_menu(value: &Value) -> Vec<ScreenerScreenTarget> {
    value
        .get("screens")
        .and_then(Value::as_array)
        .map(|screens| {
            screens
                .iter()
                .enumerate()
                .filter_map(|(fallback_index, screen)| {
                    let name = screen.get("name").and_then(Value::as_str)?.trim();
                    if name.is_empty() {
                        return None;
                    }
                    Some(ScreenerScreenTarget {
                        index: screen
                            .get("index")
                            .and_then(Value::as_u64)
                            .and_then(|value| usize::try_from(value).ok())
                            .unwrap_or(fallback_index),
                        name: name.to_string(),
                        active: screen
                            .get("active")
                            .and_then(Value::as_bool)
                            .unwrap_or(false),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn resolve_screen_target(
    screens: &[ScreenerScreenTarget],
    name: &str,
) -> Result<ScreenerScreenTarget, AppError> {
    let matches = screens
        .iter()
        .filter(|screen| screen.name == name)
        .cloned()
        .collect::<Vec<_>>();
    match matches.len() {
        0 => Err(AppError::new(
            ErrorKind::Validation,
            format!("No visible Screener screen matched name {name:?}"),
        )
        .with_details(json!({ "screens": screen_targets_payload(screens) }))),
        1 => Ok(matches[0].clone()),
        _ => Err(AppError::new(
            ErrorKind::Validation,
            format!("Screener screen name {name:?} matched multiple visible entries"),
        )
        .with_details(json!({ "matches": screen_targets_payload(&matches) }))),
    }
}

fn screen_target_payload(screen: &ScreenerScreenTarget) -> Value {
    json!({
        "index": screen.index,
        "name": screen.name,
        "active": screen.active,
    })
}

fn screen_targets_payload(screens: &[ScreenerScreenTarget]) -> Vec<Value> {
    screens.iter().map(screen_target_payload).collect()
}

fn screen_actions_from_menu(value: &Value) -> Vec<ScreenerScreenAction> {
    value
        .get("actions")
        .and_then(Value::as_array)
        .map(|actions| {
            actions
                .iter()
                .enumerate()
                .filter_map(|(fallback_index, action)| {
                    let text = action.get("text").and_then(Value::as_str)?.trim();
                    if text.is_empty() {
                        return None;
                    }
                    let kind = action
                        .get("kind")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown")
                        .trim();
                    Some(ScreenerScreenAction {
                        index: action
                            .get("index")
                            .and_then(Value::as_u64)
                            .and_then(|value| usize::try_from(value).ok())
                            .unwrap_or(fallback_index),
                        text: text.to_string(),
                        kind: kind.to_string(),
                        enabled: action
                            .get("enabled")
                            .and_then(Value::as_bool)
                            .unwrap_or(true),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn resolve_save_screen_action(
    actions: &[ScreenerScreenAction],
) -> Result<ScreenerScreenAction, AppError> {
    let matches = actions
        .iter()
        .filter(|action| action.kind == "save")
        .cloned()
        .collect::<Vec<_>>();
    match matches.len() {
        0 => Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "No visible Screener save action found",
        )
        .with_details(json!({ "actions": screen_actions_payload(actions) }))),
        1 => Ok(matches[0].clone()),
        _ => Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Multiple visible Screener save actions found",
        )
        .with_details(json!({ "matches": screen_actions_payload(&matches) }))),
    }
}

fn screen_action_payload(action: &ScreenerScreenAction) -> Value {
    json!({
        "index": action.index,
        "text": action.text,
        "kind": action.kind,
        "enabled": action.enabled,
    })
}

fn screen_actions_payload(actions: &[ScreenerScreenAction]) -> Vec<Value> {
    actions.iter().map(screen_action_payload).collect()
}

async fn open_screen_catalog_from_menu(
    runtime: &mut impl RuntimeEvaluator,
) -> Result<Value, AppError> {
    let result = runtime
        .evaluate(
            &expanded_expression(SCREENER_SCREEN_CATALOG_OPEN_POINT_EXPRESSION),
            true,
        )
        .await?;
    if !value_bool(&result, "found") {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener screen catalog open item was not found",
        )
        .with_details(result));
    }
    if value_bool(&result, "already_open") {
        return Ok(result);
    }
    let point = screen_menu_click_point(&result)?;
    dispatch_screen_menu_click(runtime, point).await?;
    Ok(result)
}

async fn click_screen_menu_target(
    runtime: &mut impl RuntimeEvaluator,
    target: &ScreenerScreenTarget,
) -> Result<(), AppError> {
    let target_name = js_string(&target.name)?;
    let result = runtime
        .evaluate(
            &expanded_expression(&format!(
                r#"
                (async function() {{
                    function sleep(ms) {{
                        return new Promise(function(resolve) {{ setTimeout(resolve, ms); }});
                    }}
                    REPLACE_HELPERS
                    var title = document.querySelector('[data-name="screener-topbar-screen-title"]');
                    if (!title || !visible(title)) {{
                        return {{ found: false, menu_opened: false, reason: 'title_not_found' }};
                    }}
                    mouseClick(title);
                    var menuState = null;
                    for (var i = 0; i < 10; i++) {{
                        var menu = findScreenerScreenMenu();
                        if (menu) {{
                            var activeTitle = textOf(title);
                            menuState = {{
                                menu_opened: true,
                                screen_title: activeTitle,
                                catalog_entry_found: /スクリーンを開く|Open screen/i.test(textOf(menu)),
                                screens: collectScreenerScreenEntries(menu, activeTitle),
                                menu: menu
                            }};
                            break;
                        }}
                        await sleep(150);
                    }}
                    if (!menuState || !menuState.menu_opened) {{
                        return {{ found: false, menu_opened: false, reason: 'menu_not_found' }};
                    }}
                    var targetName = {target_name};
                    var candidate = findScreenerScreenMenuItem(menuState.menu, targetName);
                    if (!candidate) {{
                        closeScreenerScreenMenu();
                        delete menuState.menu;
                        return Object.assign({{ found: false, reason: 'target_not_found', target_name: targetName }}, menuState);
                    }}
                    var rect = candidate.getBoundingClientRect();
                    var x = rect.left + rect.width / 2;
                    var y = rect.top + rect.height / 2;
                    delete menuState.menu;
                    return Object.assign({{
                        found: true,
                        target_name: targetName,
                        click_point: {{ x: x, y: y }}
                    }}, menuState);
                }})()
                "#
            )),
            true,
        )
        .await?;

    if !value_bool(&result, "found") {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener screen menu target was not found",
        )
        .with_details(result));
    }
    let point = screen_menu_click_point(&result)?;
    dispatch_screen_menu_click(runtime, point).await?;
    Ok(())
}

async fn click_screen_catalog_target(
    runtime: &mut impl RuntimeEvaluator,
    target: &ScreenerScreenTarget,
) -> Result<(), AppError> {
    open_screen_catalog_from_menu(runtime).await?;
    let target_name = js_string(&target.name)?;
    let result = runtime
        .evaluate(
            &expanded_expression(&format!(
                r#"
                (async function() {{
                    function sleep(ms) {{
                        return new Promise(function(resolve) {{ setTimeout(resolve, ms); }});
                    }}
                    REPLACE_HELPERS
                    var catalogState = null;
                    for (var i = 0; i < 20; i++) {{
                        var catalog = findScreenerScreenCatalog();
                        if (catalog) {{
                            var title = document.querySelector('[data-name="screener-topbar-screen-title"]');
                            var activeTitle = title && visible(title) ? textOf(title) : null;
                            catalogState = {{
                                catalog_opened: true,
                                screen_title: activeTitle,
                                screens: collectScreenerCatalogScreenEntries(catalog, activeTitle),
                                catalog: catalog
                            }};
                            break;
                        }}
                        await sleep(150);
                    }}
                    if (!catalogState || !catalogState.catalog_opened) {{
                        return {{ catalog_opened: false, found: false, reason: 'catalog_not_found' }};
                    }}
                    var targetName = {target_name};
                    var candidate = findScreenerCatalogScreenItem(catalogState.catalog, targetName);
                    if (!candidate) {{
                        closeScreenerScreenCatalog();
                        delete catalogState.catalog;
                        return Object.assign({{ found: false, reason: 'target_not_found', target_name: targetName }}, catalogState);
                    }}
                    var rect = candidate.getBoundingClientRect();
                    var x = rect.left + rect.width / 2;
                    var y = rect.top + rect.height / 2;
                    delete catalogState.catalog;
                    return Object.assign({{
                        found: true,
                        target_name: targetName,
                        click_point: {{ x: x, y: y }}
                    }}, catalogState);
                }})()
                "#
            )),
            true,
        )
        .await?;

    if !value_bool(&result, "found") {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener screen catalog target was not found",
        )
        .with_details(result));
    }
    let point = screen_menu_click_point(&result)?;
    dispatch_screen_menu_click(runtime, point).await?;
    Ok(())
}

fn screen_menu_click_point(value: &Value) -> Result<ScreenMenuClickPoint, AppError> {
    let point = value.get("click_point").ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener screen menu click point missing",
        )
        .with_details(value.clone())
    })?;
    let x = point.get("x").and_then(Value::as_f64).ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener screen menu click x coordinate missing",
        )
        .with_details(value.clone())
    })?;
    let y = point.get("y").and_then(Value::as_f64).ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener screen menu click y coordinate missing",
        )
        .with_details(value.clone())
    })?;
    Ok(ScreenMenuClickPoint { x, y })
}

async fn dispatch_screen_menu_click(
    runtime: &mut impl RuntimeEvaluator,
    point: ScreenMenuClickPoint,
) -> Result<(), AppError> {
    runtime
        .dispatch_mouse_event(MouseEvent {
            event_type: MouseEventType::Moved,
            x: point.x,
            y: point.y,
            button: None,
            buttons: None,
            click_count: None,
            delta_x: None,
            delta_y: None,
        })
        .await?;
    runtime
        .dispatch_mouse_event(MouseEvent {
            event_type: MouseEventType::Pressed,
            x: point.x,
            y: point.y,
            button: Some("left"),
            buttons: Some(1),
            click_count: Some(1),
            delta_x: None,
            delta_y: None,
        })
        .await?;
    runtime
        .dispatch_mouse_event(MouseEvent {
            event_type: MouseEventType::Released,
            x: point.x,
            y: point.y,
            button: Some("left"),
            buttons: Some(0),
            click_count: Some(1),
            delta_x: None,
            delta_y: None,
        })
        .await
}

async fn wait_for_screen_title(
    runtime: &mut impl RuntimeEvaluator,
    expected_name: &str,
) -> Result<Value, AppError> {
    let mut last_state = Value::Null;
    for _ in 0..6 {
        sleep(Duration::from_millis(250)).await;
        let state = read_screener_state(runtime, None).await?;
        let title = state
            .get("screen_title")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim();
        if title == expected_name {
            return Ok(state);
        }
        last_state = state;
    }
    Err(AppError::new(
        ErrorKind::InternalApiUnavailable,
        "Stock Screener screen title did not change to the requested screen",
    )
    .with_details(json!({
        "expected_screen_title": expected_name,
        "last_state": last_state,
    })))
}

async fn wait_for_screen_save_post_check(
    runtime: &mut impl RuntimeEvaluator,
    expected_name: &str,
) -> Result<Value, AppError> {
    let expected_name = js_string(expected_name)?;
    let result = runtime
        .evaluate(
            &expanded_expression(&format!(
                r#"
                (async function() {{
                    function sleep(ms) {{
                        return new Promise(function(resolve) {{ setTimeout(resolve, ms); }});
                    }}
                    REPLACE_HELPERS
                    for (var i = 0; i < 12; i++) {{
                        await sleep(150);
                        var state = readScreenerState(0);
                        var dialog = findBlockingScreenerMutationDialog();
                        if (dialog) {{
                            state.blocking_dialog_found = true;
                            state.blocking_dialog_text = textOf(dialog).substring(0, 200);
                            return state;
                        }}
                        var title = (state.screen_title || '').trim();
                        if (title === {expected_name}) {{
                            state.blocking_dialog_found = false;
                            return state;
                        }}
                    }}
                    var finalState = readScreenerState(0);
                    var finalDialog = findBlockingScreenerMutationDialog();
                    finalState.blocking_dialog_found = !!finalDialog;
                    if (finalDialog) finalState.blocking_dialog_text = textOf(finalDialog).substring(0, 200);
                    return finalState;
                }})()
                "#
            )),
            true,
        )
        .await?;

    if value_bool(&result, "blocking_dialog_found") {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener save opened an unexpected blocking dialog",
        )
        .with_details(result));
    }

    Ok(result)
}

async fn read_column_actions(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &expanded_expression(SCREENER_COLUMNS_ACTIONS_EXPRESSION),
            true,
        )
        .await
}

async fn read_filter_actions(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &expanded_expression(
                r#"
                (async function() {
                    function sleep(ms) {
                        return new Promise(function(resolve) { setTimeout(resolve, ms); });
                    }
                    REPLACE_HELPERS
                    var state = readScreenerState(0);
                    var addButton = findScreenerAddFilterButton();
                    var candidate = findScreenerNumericFilterPill();
                    var candidatePayload = candidate ? {
                        text: textOf(candidate),
                        data_name: candidate.getAttribute('data-name') || null
                    } : null;
                    var manualSettingsFound = false;
                    var rangeOptions = [];
                    if (candidate) {
                        mouseClick(candidate);
                        for (var i = 0; i < 10; i++) {
                            if (findScreenerManualFilterButton()) {
                                manualSettingsFound = true;
                                break;
                            }
                            await sleep(100);
                        }
                        var manualButton = findScreenerManualFilterButton();
                        if (manualButton) {
                            mouseClick(manualButton);
                            for (var j = 0; j < 10; j++) {
                                var combo = findScreenerRangeCombobox();
                                if (combo) {
                                    mouseClick(combo);
                                    for (var k = 0; k < 10; k++) {
                                        rangeOptions = collectScreenerRangeOptions();
                                        if (rangeOptions.length > 0) break;
                                        await sleep(100);
                                    }
                                    break;
                                }
                                await sleep(100);
                            }
                        }
                        closeScreenerTransientPopups();
                    }
                    return {
                        source: 'ui_screener_dialog',
                        action: 'filters_actions',
                        open: !!state.open,
                        screen_title: state.screen_title || null,
                        add_button_found: !!addButton,
                        add_supported: false,
                        add_unavailable_reason: addButton ? 'filter_add_catalog_not_verified' : 'add_button_not_found',
                        numeric_modify_supported: rangeOptions.length > 0,
                        candidate_filter: candidatePayload,
                        manual_settings_found: manualSettingsFound,
                        range_options: rangeOptions.map(function(option) {
                            return {
                                index: option.index,
                                text: option.text,
                                normalized_text: option.normalized_text
                            };
                        }),
                        unavailable_reason: rangeOptions.length > 0 ? null : 'numeric_range_filter_preset_options_not_found'
                    };
                })()
                "#,
            ),
            true,
        )
        .await
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

async fn click_filter_range_preset(
    runtime: &mut impl RuntimeEvaluator,
    target: &ScreenerFilterTarget,
    preset_label: &str,
) -> Result<(), AppError> {
    let data_name = js_string(&target.data_name)?;
    let preset_label = js_string(preset_label)?;
    let result = runtime
        .evaluate(
            &expanded_expression(&format!(
                r#"
                (async function() {{
                    function sleep(ms) {{
                        return new Promise(function(resolve) {{ setTimeout(resolve, ms); }});
                    }}
                    REPLACE_HELPERS
                    var pill = document.querySelector('[data-name=' + {data_name} + ']');
                    if (!pill || !visible(pill)) {{
                        return {{ found: false, data_name: {data_name}, reason: 'filter_pill_not_found' }};
                    }}
                    mouseClick(pill);
                    var manualButton = null;
                    for (var i = 0; i < 8; i++) {{
                        manualButton = findScreenerManualFilterButton();
                        if (manualButton) break;
                        await sleep(75);
                    }}
                    if (!manualButton) {{
                        closeScreenerTransientPopups();
                        return {{
                            found: true,
                            manual_settings_found: false,
                            range_combobox_found: false,
                            range_option_found: false,
                            requested_range: {preset_label},
                            data_name: {data_name}
                        }};
                    }}
                    mouseClick(manualButton);
                    var combo = null;
                    for (var j = 0; j < 8; j++) {{
                        combo = findScreenerRangeCombobox();
                        if (combo) break;
                        await sleep(75);
                    }}
                    if (!combo) {{
                        closeScreenerTransientPopups();
                        return {{
                            found: true,
                            manual_settings_found: true,
                            range_combobox_found: false,
                            range_option_found: false,
                            requested_range: {preset_label},
                            data_name: {data_name}
                        }};
                    }}
                    mouseClick(combo);
                    var option = null;
                    var options = [];
                    for (var k = 0; k < 8; k++) {{
                        options = collectScreenerRangeOptions();
                        option = options.find(function(candidate) {{
                            return candidate.normalized_text === {preset_label};
                        }});
                        if (option && option.element) break;
                        await sleep(75);
                    }}
                    if (!option || !option.element) {{
                        closeScreenerTransientPopups();
                        return {{
                            found: true,
                            manual_settings_found: true,
                            range_combobox_found: true,
                            range_option_found: false,
                            requested_range: {preset_label},
                            available_options: options.map(function(candidate) {{
                                return candidate.normalized_text;
                            }}),
                            data_name: {data_name}
                        }};
                    }}
                    mouseClick(option.element);
                    return {{
                        found: true,
                        manual_settings_found: true,
                        range_combobox_found: true,
                        range_option_found: true,
                        clicked: true,
                        requested_range: {preset_label},
                        data_name: {data_name}
                    }};
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
    if !value_bool(&result, "manual_settings_found") {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener filter manual settings button not found",
        )
        .with_details(result));
    }
    if !value_bool(&result, "range_combobox_found") {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener filter range combobox not found",
        )
        .with_details(result));
    }
    if !value_bool(&result, "range_option_found") {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener filter range preset option not found",
        )
        .with_details(result));
    }
    sleep(Duration::from_millis(250)).await;
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

async fn wait_for_filter_modified(
    runtime: &mut impl RuntimeEvaluator,
    data_name: &str,
    preset_label: &str,
) -> Result<Value, AppError> {
    let raw_data_name = data_name.to_string();
    let raw_preset_label = preset_label.to_string();
    let mut last_state = Value::Null;
    for _ in 0..12 {
        let state = read_screener_state(runtime, None).await?;
        let modified = filter_targets_from_state(&state).iter().any(|filter| {
            filter.data_name == raw_data_name
                && normalize_screener_text(&filter.text).contains(&raw_preset_label)
        });
        if modified {
            return Ok(state);
        }
        last_state = state;
        sleep(Duration::from_millis(250)).await;
    }

    Err(AppError::new(
        ErrorKind::InternalApiUnavailable,
        "Screener filter text did not reflect requested range preset",
    )
    .with_details(last_state))
}

fn normalize_screener_text(value: &str) -> String {
    value
        .replace(['\u{2066}', '\u{2067}', '\u{2068}', '\u{2069}'], "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
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

const SCREENER_SCREEN_MENU_LIST_EXPRESSION: &str = r#"
(async function() {
    function sleep(ms) {
        return new Promise(function(resolve) { setTimeout(resolve, ms); });
    }
    REPLACE_HELPERS
    if (findScreenerScreenCatalog()) {
        return { found: true, already_open: true, catalog_opened: true };
    }
    var title = document.querySelector('[data-name="screener-topbar-screen-title"]');
    if (!title || !visible(title)) {
        return { menu_opened: false, reason: 'title_not_found', screens: [] };
    }
    var activeTitle = textOf(title);
    mouseClick(title);
    for (var i = 0; i < 10; i++) {
        var menu = findScreenerScreenMenu();
        if (menu) {
            var result = {
                menu_opened: true,
                screen_title: activeTitle,
                catalog_entry_found: /スクリーンを開く|Open screen/i.test(textOf(menu)),
                screens: collectScreenerScreenEntries(menu, activeTitle)
            };
            closeScreenerScreenMenu();
            return result;
        }
        await sleep(150);
    }
    closeScreenerScreenMenu();
    return { menu_opened: false, reason: 'menu_not_found', screen_title: activeTitle, screens: [] };
})()
"#;

const SCREENER_SCREEN_CATALOG_OPEN_POINT_EXPRESSION: &str = r#"
(async function() {
    function sleep(ms) {
        return new Promise(function(resolve) { setTimeout(resolve, ms); });
    }
    REPLACE_HELPERS
    var title = document.querySelector('[data-name="screener-topbar-screen-title"]');
    if (!title || !visible(title)) {
        return { found: false, menu_opened: false, reason: 'title_not_found' };
    }
    var activeTitle = textOf(title);
    mouseClick(title);
    var menu = null;
    for (var i = 0; i < 10; i++) {
        menu = findScreenerScreenMenu();
        if (menu) break;
        await sleep(150);
    }
    if (!menu) {
        return { found: false, menu_opened: false, reason: 'menu_not_found', screen_title: activeTitle };
    }
    var openItem = findScreenerCatalogOpenItem(menu);
    if (!openItem) {
        closeScreenerScreenMenu();
        return {
            found: false,
            menu_opened: true,
            reason: 'catalog_open_item_not_found',
            screen_title: activeTitle,
            screens: collectScreenerScreenEntries(menu, activeTitle)
        };
    }
    var rect = openItem.getBoundingClientRect();
    return {
        found: true,
        menu_opened: true,
        screen_title: activeTitle,
        click_point: {
            x: rect.left + rect.width / 2,
            y: rect.top + rect.height / 2
        }
    };
})()
"#;

const SCREENER_SCREEN_CATALOG_LIST_EXPRESSION: &str = r#"
(async function() {
    function sleep(ms) {
        return new Promise(function(resolve) { setTimeout(resolve, ms); });
    }
    REPLACE_HELPERS
    for (var i = 0; i < 20; i++) {
        var catalog = findScreenerScreenCatalog();
        if (catalog) {
            var title = document.querySelector('[data-name="screener-topbar-screen-title"]');
            var activeTitle = title && visible(title) ? textOf(title) : null;
            var result = {
                catalog_opened: true,
                screen_title: activeTitle,
                screens: collectScreenerCatalogScreenEntries(catalog, activeTitle)
            };
            closeScreenerScreenCatalog();
            return result;
        }
        await sleep(150);
    }
    return { catalog_opened: false, reason: 'catalog_not_found', screens: [] };
})()
"#;

const SCREENER_SCREEN_ACTIONS_EXPRESSION: &str = r#"
(async function() {
    function sleep(ms) {
        return new Promise(function(resolve) { setTimeout(resolve, ms); });
    }
    REPLACE_HELPERS
    var title = document.querySelector('[data-name="screener-topbar-screen-title"]');
    if (!title || !visible(title)) {
        return { menu_opened: false, reason: 'title_not_found', actions: [] };
    }
    var activeTitle = textOf(title);
    mouseClick(title);
    for (var i = 0; i < 10; i++) {
        var menu = findScreenerScreenMenu();
        if (menu) {
            var result = {
                menu_opened: true,
                screen_title: activeTitle,
                actions: collectScreenerScreenActions(menu)
            };
            closeScreenerScreenMenu();
            return result;
        }
        await sleep(150);
    }
    closeScreenerScreenMenu();
    return { menu_opened: false, reason: 'menu_not_found', screen_title: activeTitle, actions: [] };
})()
"#;

const SCREENER_SCREEN_SAVE_POINT_EXPRESSION: &str = r#"
(async function() {
    function sleep(ms) {
        return new Promise(function(resolve) { setTimeout(resolve, ms); });
    }
    REPLACE_HELPERS
    var title = document.querySelector('[data-name="screener-topbar-screen-title"]');
    if (!title || !visible(title)) {
        return { menu_opened: false, found: false, reason: 'title_not_found', actions: [] };
    }
    var activeTitle = textOf(title);
    mouseClick(title);
    for (var i = 0; i < 10; i++) {
        var menu = findScreenerScreenMenu();
        if (menu) {
            var actions = collectScreenerScreenActions(menu);
            var saveItem = findScreenerScreenSaveItem(menu);
            if (!saveItem) {
                closeScreenerScreenMenu();
                return {
                    menu_opened: true,
                    found: false,
                    reason: 'save_action_not_found',
                    screen_title: activeTitle,
                    actions: actions
                };
            }
            var rect = saveItem.getBoundingClientRect();
            var disabled = saveItem.disabled ||
                saveItem.getAttribute('aria-disabled') === 'true' ||
                /disabled/i.test(String(saveItem.className || ''));
            return {
                menu_opened: true,
                found: true,
                enabled: !disabled,
                screen_title: activeTitle,
                actions: actions,
                click_point: {
                    x: rect.left + rect.width / 2,
                    y: rect.top + rect.height / 2
                }
            };
        }
        await sleep(150);
    }
    closeScreenerScreenMenu();
    return { menu_opened: false, found: false, reason: 'menu_not_found', screen_title: activeTitle, actions: [] };
})()
"#;

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
function mouseClick(el) {
    var rect = el.getBoundingClientRect();
    var x = rect.left + rect.width / 2;
    var y = rect.top + rect.height / 2;
    ['mouseover', 'mousedown', 'mouseup', 'click'].forEach(function(type) {
        el.dispatchEvent(new MouseEvent(type, {
            bubbles: true,
            cancelable: true,
            clientX: x,
            clientY: y,
            view: window
        }));
    });
}
function closeScreenerScreenMenu() {
    var title = document.querySelector('[data-name="screener-topbar-screen-title"]');
    if (title && visible(title)) {
        mouseClick(title);
    }
}
function screenerScreenActionText(text) {
    return /スクリーンを保存|スクリーンを共有|コピーを作成|名前を変更|CSV|新規スクリーン|最近使用した項目|スクリーンを開く|Save screen|Share screen|Make a copy|Rename|Download.*CSV|Create new screen|Recent|Open screen/i.test(text);
}
function screenerScreenExactActionText(text) {
    return /^(スクリーンを保存|スクリーンを共有|コピーを作成…?|名前を変更…?|結果をCSVでダウンロード|新規スクリーンを作成…?|最近使用した項目|スクリーンを開く…?|Save screen|Share screen|Make a copy…?|Rename…?|Download.*CSV|Create new screen…?|Recent|Open screen…?)$/i.test(text);
}
function screenerScreenActionKind(text) {
    if (/^(スクリーンを保存|Save screen)$/i.test(text)) return 'save';
    if (/^(スクリーンを共有|Share screen)$/i.test(text)) return 'share';
    if (/^(コピーを作成…?|Make a copy…?)$/i.test(text)) return 'make_copy';
    if (/^(名前を変更…?|Rename…?)$/i.test(text)) return 'rename';
    if (/^(新規スクリーンを作成…?|Create new screen…?)$/i.test(text)) return 'create';
    if (/^(スクリーンを開く…?|Open screen…?)$/i.test(text)) return 'open';
    if (/CSV/i.test(text)) return 'download_csv';
    if (/^(最近使用した項目|Recent)$/i.test(text)) return 'recent';
    return 'unknown';
}
function collectScreenerScreenActions(menu) {
    var seen = {};
    var actions = [];
    var nodes = Array.from(menu.querySelectorAll('button, [role="menuitem"], [role="option"], [role="button"], div, span')).filter(visible);
    nodes.forEach(function(el) {
        var text = textOf(el);
        if (!text || text.length > 120 || !screenerScreenExactActionText(text)) return;
        if (seen[text]) return;
        var disabled = el.disabled ||
            el.getAttribute('aria-disabled') === 'true' ||
            /disabled/i.test(String(el.className || ''));
        seen[text] = true;
        actions.push({
            index: actions.length,
            text: text,
            kind: screenerScreenActionKind(text),
            enabled: !disabled
        });
    });
    return actions;
}
function findScreenerScreenSaveItem(menu) {
    var candidates = Array.from(menu.querySelectorAll('button, [role="menuitem"], [role="option"], [role="button"], div, span')).filter(function(el) {
        return visible(el) && /^(スクリーンを保存|Save screen)$/i.test(textOf(el));
    });
    candidates = candidates.map(function(el) {
        var current = el;
        while (current && current !== menu) {
            var cls = String(current.className || '');
            if (/^(スクリーンを保存|Save screen)$/i.test(textOf(current)) &&
                (current.getAttribute('role') === 'menuitem' ||
                 current.getAttribute('role') === 'button' ||
                 current.tagName === 'BUTTON' ||
                 /button|background|item|row/i.test(cls))) {
                return current;
            }
            current = current.parentElement;
        }
        return el;
    });
    candidates.sort(function(a, b) {
        var ar = a.getBoundingClientRect();
        var br = b.getBoundingClientRect();
        return (br.width * br.height) - (ar.width * ar.height);
    });
    return candidates[0] || null;
}
function collectScreenerScreenEntries(menu, activeTitle) {
    var seen = {};
    var entries = [];
    var nodes = Array.from(menu.querySelectorAll('button, [role="menuitem"], [role="option"], div, span')).filter(visible);
    nodes.forEach(function(el) {
        var name = textOf(el);
        if (!name || name.length > 120 || screenerScreenActionText(name)) return;
        if (name.indexOf('\n') >= 0) return;
        if (name.indexOf(activeTitle + ' ') === 0) return;
        if (seen[name]) return;
        seen[name] = true;
        entries.push({
            index: entries.length,
            name: name,
            active: name === activeTitle
        });
    });
    return entries;
}
function findScreenerScreenMenuItem(menu, targetName) {
    var candidates = Array.from(menu.querySelectorAll('button, [role="menuitem"], [role="option"], div, span')).filter(function(el) {
        return visible(el) && textOf(el) === targetName;
    });
    candidates = candidates.map(function(el) {
        var current = el;
        while (current && current !== menu) {
            var cls = String(current.className || '');
            if (textOf(current) === targetName &&
                (current.getAttribute('role') === 'menuitem' ||
                 current.tagName === 'BUTTON' ||
                 /button|background|item|row/i.test(cls))) {
                return current;
            }
            current = current.parentElement;
        }
        return el;
    });
    candidates.sort(function(a, b) {
        var ar = a.getBoundingClientRect();
        var br = b.getBoundingClientRect();
        return (br.width * br.height) - (ar.width * ar.height);
    });
    return candidates[0] || null;
}
function findScreenerScreenMenu() {
    var menus = visibleElements('.portal-lATuqHRX, [role="menu"], [class*="menu"], [class*="portal"]');
    var matches = menus.filter(function(menu) {
        var text = textOf(menu);
        return /最近使用した項目|Recent|スクリーンを開く|Open screen|スクリーンを保存|Save screen/i.test(text) &&
            /米国株|screen|Screen|スクリーン|CLI|株/i.test(text);
    });
    matches.sort(function(a, b) {
        var ar = a.getBoundingClientRect();
        var br = b.getBoundingClientRect();
        return (ar.width * ar.height) - (br.width * br.height);
    });
    return matches[0] || null;
}
function findScreenerCatalogOpenItem(menu) {
    var candidates = Array.from(menu.querySelectorAll('button, [role="menuitem"], [role="option"], div, span')).filter(function(el) {
        return visible(el) && /^(スクリーンを開く|Open screen)/i.test(textOf(el));
    });
    candidates = candidates.map(function(el) {
        var current = el;
        while (current && current !== menu) {
            var cls = String(current.className || '');
            if (/^(スクリーンを開く|Open screen)/i.test(textOf(current)) &&
                (current.getAttribute('role') === 'menuitem' ||
                 current.tagName === 'BUTTON' ||
                 /button|background|item|row/i.test(cls))) {
                return current;
            }
            current = current.parentElement;
        }
        return el;
    });
    candidates.sort(function(a, b) {
        var ar = a.getBoundingClientRect();
        var br = b.getBoundingClientRect();
        return (br.width * br.height) - (ar.width * ar.height);
    });
    return candidates[0] || null;
}
function openScreenerScreenMenu() {
    var title = document.querySelector('[data-name="screener-topbar-screen-title"]');
    if (!title || !visible(title)) {
        return { menu_opened: false, reason: 'title_not_found', screens: [] };
    }
    var activeTitle = textOf(title);
    mouseClick(title);
    var menu = findScreenerScreenMenu();
    if (!menu) {
        return {
            menu_opened: false,
            reason: 'menu_not_found',
            screen_title: activeTitle,
            screens: []
        };
    }
    var entries = collectScreenerScreenEntries(menu, activeTitle);
    return {
        menu_opened: true,
        screen_title: activeTitle,
        catalog_entry_found: /スクリーンを開く|Open screen/i.test(textOf(menu)),
        screens: entries,
        menu: menu
    };
}
function findScreenerScreenCatalog() {
    var containers = visibleElements('[role="dialog"], [class*="dialog"], [class*="Dialog"], [class*="modal"], [class*="Modal"], .portal-lATuqHRX');
    return containers.find(function(container) {
        if (container.querySelector && container.querySelector('table')) return false;
        var text = textOf(container);
        if (text.length > 3000) return false;
        return /マイスクリーン|My screens|Screeners|スクリーン/i.test(text) &&
            !/スクリーンを保存|Save screen/i.test(text);
    }) || null;
}
function closeScreenerScreenCatalog() {
    var catalog = findScreenerScreenCatalog();
    if (!catalog) return;
    var closeButton = Array.from(catalog.querySelectorAll('button, [role="button"], [aria-label], [data-name]')).filter(visible).find(function(el) {
        var label = [el.getAttribute('aria-label'), el.getAttribute('title'), el.getAttribute('data-name'), textOf(el)].filter(Boolean).join(' ');
        return /close|dismiss|閉じる|キャンセル|cancel/i.test(label);
    });
    if (closeButton) {
        mouseClick(closeButton);
        return;
    }
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', code: 'Escape', keyCode: 27, which: 27, bubbles: true, cancelable: true }));
    document.dispatchEvent(new KeyboardEvent('keyup', { key: 'Escape', code: 'Escape', keyCode: 27, which: 27, bubbles: true, cancelable: true }));
}
function findBlockingScreenerMutationDialog() {
    var containers = visibleElements('[role="dialog"], [class*="dialog"], [class*="Dialog"], [class*="modal"], [class*="Modal"], .portal-lATuqHRX');
    return containers.find(function(container) {
        if (container.querySelector && container.querySelector('table')) return false;
        var text = textOf(container);
        if (text.length > 3000) return false;
        return /名前を変更|Rename|コピーを作成|Make a copy|新規スクリーン|Create new screen|削除|Delete|保存先|Save as/i.test(text);
    }) || null;
}
function screenerCatalogActionText(text) {
    return /マイスクリーン|My screens|最近|Recent|スクリーンを開く|Open screen|スクリーンを保存|Save screen|スクリーンを共有|Share screen|コピーを作成|Make a copy|名前を変更|Rename|新規スクリーン|Create new screen|検索|Search|キャンセル|Cancel|閉じる|Close/i.test(text);
}
function collectScreenerCatalogScreenEntries(catalog, activeTitle) {
    var seen = {};
    var entries = [];
    var inMyScreens = false;
    var leftMyScreens = false;
    var nodes = Array.from(catalog.querySelectorAll('button, [role="option"], [role="menuitem"], [role="row"], [data-name], div, span')).filter(visible);
    nodes.forEach(function(el) {
        if (leftMyScreens) return;
        var name = textOf(el);
        if (/^(マイスクリーン|My screens)$/i.test(name)) {
            inMyScreens = true;
            return;
        }
        if (/^(人気のスクリーン|Popular screens)$/i.test(name)) {
            leftMyScreens = true;
            return;
        }
        if (!inMyScreens) return;
        if (!name || name.length > 120 || screenerCatalogActionText(name)) return;
        if (name.indexOf('\n') >= 0) return;
        if (seen[name]) return;
        var rect = el.getBoundingClientRect();
        if (rect.width < 20 || rect.height < 8) return;
        seen[name] = true;
        entries.push({
            index: entries.length,
            name: name,
            active: name === activeTitle
        });
    });
    return entries;
}
function findScreenerCatalogScreenItem(catalog, targetName) {
    var candidates = Array.from(catalog.querySelectorAll('button, [role="option"], [role="menuitem"], [role="row"], [data-name], div, span')).filter(function(el) {
        return visible(el) && textOf(el) === targetName;
    });
    candidates = candidates.map(function(el) {
        var current = el;
        while (current && current !== catalog) {
            var cls = String(current.className || '');
            var role = current.getAttribute('role') || '';
            if (textOf(current) === targetName &&
                (role === 'option' || role === 'menuitem' || role === 'row' ||
                 current.tagName === 'BUTTON' || /button|item|row|list|cell/i.test(cls))) {
                return current;
            }
            current = current.parentElement;
        }
        return el;
    });
    candidates.sort(function(a, b) {
        var ar = a.getBoundingClientRect();
        var br = b.getBoundingClientRect();
        return (br.width * br.height) - (ar.width * ar.height);
    });
    return candidates[0] || null;
}
async function openScreenerScreenCatalog() {
    function sleep(ms) {
        return new Promise(function(resolve) { setTimeout(resolve, ms); });
    }
    var title = document.querySelector('[data-name="screener-topbar-screen-title"]');
    if (!title || !visible(title)) {
        return { catalog_opened: false, menu_opened: false, reason: 'title_not_found', screens: [] };
    }
    var activeTitle = textOf(title);
    mouseClick(title);
    var menu = null;
    for (var i = 0; i < 10; i++) {
        menu = findScreenerScreenMenu();
        if (menu) break;
        await sleep(150);
    }
    if (!menu) {
        return { catalog_opened: false, menu_opened: false, reason: 'menu_not_found', screen_title: activeTitle, screens: [] };
    }
    var openItem = findScreenerCatalogOpenItem(menu);
    if (!openItem) {
        closeScreenerScreenMenu();
        return { catalog_opened: false, menu_opened: true, reason: 'catalog_open_item_not_found', screen_title: activeTitle, screens: collectScreenerScreenEntries(menu, activeTitle) };
    }
    mouseClick(openItem);
    for (var j = 0; j < 4; j++) {
        var catalog = findScreenerScreenCatalog();
        if (catalog) {
            return {
                catalog_opened: true,
                menu_opened: true,
                screen_title: activeTitle,
                screens: collectScreenerCatalogScreenEntries(catalog, activeTitle),
                catalog: catalog
            };
        }
        await sleep(100);
    }
    return { catalog_opened: false, menu_opened: true, reason: 'catalog_not_found', screen_title: activeTitle, screens: [] };
}
function normalizeScreenerFilterText(value) {
    return String(value || '').replace(/[\u2066\u2067\u2068\u2069]/g, '').replace(/\s+/g, ' ').trim();
}
function findScreenerAddFilterButton() {
    return visibleElements('button, [role="button"], [title], [aria-label], [data-name]').find(function(el) {
        var label = [el.getAttribute('title'), el.getAttribute('aria-label'), el.getAttribute('data-name'), textOf(el)].filter(Boolean).join(' ');
        return /新しいフィルターを追加|フィルターを追加|Add new filter|Add filter/i.test(label);
    }) || null;
}
function findScreenerNumericFilterPill() {
    var filters = visibleElements('[data-name^="screener-filter-pill-"]');
    return filters.find(function(el) {
        var text = normalizeScreenerFilterText(textOf(el));
        return /:/.test(text) && /〜/.test(text);
    }) || filters.find(function(el) {
        var text = normalizeScreenerFilterText(textOf(el));
        return /EMA|SMA|価格|Price/i.test(text) && /〜/.test(text);
    }) || filters.find(function(el) {
        return /〜/.test(normalizeScreenerFilterText(textOf(el)));
    }) || filters.find(function(el) {
        return /以上|以下|未満/.test(normalizeScreenerFilterText(textOf(el)));
    }) || filters.find(function(el) {
        return /greater|less|between|range/i.test(normalizeScreenerFilterText(textOf(el)));
    }) || null;
}
function findScreenerFallbackNumericFilterPill() {
    var filters = visibleElements('[data-name^="screener-filter-pill-"]');
    return filters.find(function(el) {
        return /%|以上|以下|未満|greater|less|between|range/i.test(normalizeScreenerFilterText(textOf(el)));
    }) || filters[0] || null;
}
function findScreenerManualFilterButton() {
    return visibleElements('button, [role="button"], [role="menuitem"], div, span').find(function(el) {
        var text = textOf(el);
        return /手動で設定|Set manually|Manual/i.test(text) && text.length < 120;
    }) || null;
}
function findScreenerRangeCombobox() {
    var candidates = visibleElements('button, [role="button"], [role="combobox"], [aria-haspopup], div, span');
    candidates = candidates.filter(function(el) {
        var text = normalizeScreenerFilterText(textOf(el));
        if (!text || text.length > 80) return false;
        return /%/.test(text) && (/〜|以上|以下|未満|to|or more|less/i.test(text));
    });
    candidates.sort(function(a, b) {
        var ar = a.getBoundingClientRect();
        var br = b.getBoundingClientRect();
        return (ar.width * ar.height) - (br.width * br.height);
    });
    return candidates[0] || null;
}
function collectScreenerRangeOptions() {
    var seen = {};
    var options = [];
    function optionClickTarget(el, label) {
        var current = el;
        while (current && current !== document.body) {
            var role = current.getAttribute && (current.getAttribute('role') || '');
            var cls = String(current.className || '');
            if (textOf(current).indexOf(label) >= 0 &&
                (role === 'option' || role === 'menuitem' || role === 'button' ||
                 current.tagName === 'BUTTON' || /item|option|row|button/i.test(cls))) {
                return current;
            }
            current = current.parentElement;
        }
        return el;
    }
    var nodes = visibleElements('[role="option"], [role="menuitem"], button, div, span');
    nodes.forEach(function(el) {
        var text = normalizeScreenerFilterText(textOf(el));
        if (!text || text.length > 80) return;
        if (!/%/.test(text) || !(/〜|以上|以下|未満|to|or more|less/i.test(text))) return;
        if (!/^-?\d+(?:\.\d+)?%\s*(?:〜|to)\s*-?\d+(?:\.\d+)?%$|^-?\d+(?:\.\d+)?%\s*(?:以上|以下|未満)$|^-?\d+(?:\.\d+)?%\s*(?:or more|less)$/i.test(text)) return;
        if (seen[text]) return;
        seen[text] = true;
        options.push({
            index: options.length,
            text: text,
            normalized_text: text,
            element: optionClickTarget(el, text)
        });
    });
    return options;
}
function closeScreenerTransientPopups() {
    for (var i = 0; i < 3; i++) {
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', code: 'Escape', keyCode: 27, which: 27, bubbles: true, cancelable: true }));
        document.dispatchEvent(new KeyboardEvent('keyup', { key: 'Escape', code: 'Escape', keyCode: 27, which: 27, bubbles: true, cancelable: true }));
    }
}
function readScreenerState(limit) {
    var button = document.querySelector('[data-name="screener-dialog-button"]');
    var screenerDataElements = visibleElements('[data-name*="screener"]');
    var classElements = visibleElements('[class*="screener"]');
    var container = visibleElements('[class*="screenerContainer"], [class*="screener-container"]').find(function(el) {
        return el !== button;
    }) || null;
    var heading = Array.from(document.querySelectorAll('h1, h2, h3'))
        .find(function(el) {
            var text = textOf(el);
            return visible(el) && text.length <= 120 && /screener|スクリーナー/i.test(text);
        }) || Array.from(document.querySelectorAll('button, div, span'))
        .find(function(el) {
            var text = textOf(el);
            return visible(el) && text.length <= 120 && /screener|スクリーナー/i.test(text);
        }) || null;
    var table = container ? (Array.from(container.querySelectorAll('table')).filter(visible)[0] || null) : null;
    var title = container ? container.querySelector('[data-name="screener-topbar-screen-title"]') : document.querySelector('[data-name="screener-topbar-screen-title"]');
    var open = !!(container || heading || screenerDataElements.some(function(el) {
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
        screen_title: title ? textOf(title) : null,
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
    fn validate_screener_filter_modify_accepts_visible_presets() {
        let request =
            validate_screener_filter_modify_request(None, Some("EMA"), Some(0.0), Some(5.0), true)
                .unwrap();

        assert_eq!(
            request.selector,
            ScreenerFilterSelector::Text("EMA".to_string())
        );
        assert_eq!(request.preset_label, "0% 〜 5%");
        assert_eq!(
            filter_modify_range_payload(&request)["preset_label"],
            "0% 〜 5%"
        );

        let request =
            validate_screener_filter_modify_request(Some(1), None, Some(15.0), None, false)
                .unwrap();

        assert_eq!(request.selector, ScreenerFilterSelector::Index(1));
        assert_eq!(request.preset_label, "15%以上");
    }

    #[test]
    fn validate_screener_filter_modify_rejects_unsafe_inputs() {
        assert_eq!(
            validate_screener_filter_modify_request(None, None, Some(0.0), Some(5.0), true)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_filter_modify_request(
                Some(0),
                Some("EMA"),
                Some(0.0),
                Some(5.0),
                true
            )
            .unwrap_err()
            .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_filter_modify_request(Some(0), None, None, None, true)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_filter_modify_request(Some(0), None, None, Some(5.0), true)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_filter_modify_request(Some(0), None, Some(f64::NAN), Some(5.0), true)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_filter_modify_request(Some(0), None, Some(0.0), Some(7.0), true)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
    }

    #[test]
    fn validate_screener_column_selector_requires_one_target() {
        assert_eq!(
            validate_screener_column_selector(Some(2), None).unwrap(),
            ScreenerColumnSelector::Index(2)
        );
        assert_eq!(
            validate_screener_column_selector(None, Some(" Price ")).unwrap(),
            ScreenerColumnSelector::Name("Price".to_string())
        );
        assert_eq!(
            validate_screener_column_selector(None, None)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_column_selector(Some(0), Some("Price"))
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

    #[test]
    fn validate_screener_screen_name_trims_and_rejects_empty() {
        assert_eq!(
            validate_screener_screen_name(" 米国株（テスト用） ").unwrap(),
            "米国株（テスト用）"
        );
        assert_eq!(
            validate_screener_screen_name("   ").unwrap_err().kind,
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
    async fn screener_screens_list_returns_menu_visible_entries() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "dialog_title": "Stock Screener",
                "screen_title": "米国株（テスト用）"
            }),
            json!({
                "menu_opened": true,
                "catalog_entry_found": true,
                "screens": [
                    { "index": 0, "name": "米国株（テスト用）", "active": true },
                    { "index": 1, "name": "米国株", "active": false }
                ]
            }),
        ]);

        let result = screener_screens_list(&mut runtime, false).await.unwrap();

        assert_eq!(result["source"], SCREENER_SOURCE);
        assert_eq!(result["scope"], "screen_title_menu");
        assert_eq!(result["screen_title"], "米国株（テスト用）");
        assert_eq!(result["catalog_entry_found"], true);
        assert_eq!(result["screen_count"], 2);
        assert_eq!(result["screens"][0]["name"], "米国株（テスト用）");
        assert_eq!(result["screens"][0]["active"], true);
    }

    #[tokio::test]
    async fn screener_screens_actions_reports_visible_save_action() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "dialog_title": "Stock Screener",
                "screen_title": "CLI-Test1"
            }),
            json!({
                "menu_opened": true,
                "screen_title": "CLI-Test1",
                "actions": [
                    { "index": 0, "text": "Save screen", "kind": "save", "enabled": true },
                    { "index": 1, "text": "Rename", "kind": "rename", "enabled": true }
                ]
            }),
        ]);

        let result = screener_screens_actions(&mut runtime).await.unwrap();

        assert_eq!(result["source"], SCREENER_SOURCE);
        assert_eq!(result["scope"], "screen_title_menu");
        assert_eq!(result["screen_title"], "CLI-Test1");
        assert_eq!(result["action_count"], 2);
        assert_eq!(result["save_available"], true);
        assert_eq!(result["save_enabled"], true);
        assert_eq!(result["actions"][0]["kind"], "save");
    }

    #[tokio::test]
    async fn screener_screens_save_dry_run_reports_target_without_clicking() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "CLI-Test1"
            }),
            json!({
                "menu_opened": true,
                "found": true,
                "enabled": true,
                "screen_title": "CLI-Test1",
                "actions": [
                    { "index": 0, "text": "Save screen", "kind": "save", "enabled": true },
                    { "index": 1, "text": "Rename", "kind": "rename", "enabled": true }
                ],
                "click_point": { "x": 100.0, "y": 200.0 }
            }),
        ]);

        let result = screener_screens_save(&mut runtime, true).await.unwrap();

        assert_eq!(result["action"], "screen_save");
        assert_eq!(result["dry_run"], true);
        assert_eq!(result["clicked"], false);
        assert_eq!(result["save_requested"], false);
        assert_eq!(result["confirmation"], "dry_run");
        assert_eq!(result["target_action"]["kind"], "save");
        assert_eq!(runtime.mouse_events.len(), 0);
    }

    #[tokio::test]
    async fn screener_screens_save_clicks_exact_save_and_post_checks() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "CLI-Test1"
            }),
            json!({
                "menu_opened": true,
                "found": true,
                "enabled": true,
                "screen_title": "CLI-Test1",
                "actions": [
                    { "index": 0, "text": "Save screen", "kind": "save", "enabled": true },
                    { "index": 1, "text": "Make a copy", "kind": "make_copy", "enabled": true }
                ],
                "click_point": { "x": 100.0, "y": 200.0 }
            }),
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "CLI-Test1",
                "blocking_dialog_found": false
            }),
        ]);

        let result = screener_screens_save(&mut runtime, false).await.unwrap();

        assert_eq!(result["action"], "screen_save");
        assert_eq!(result["dry_run"], false);
        assert_eq!(result["clicked"], true);
        assert_eq!(result["save_requested"], true);
        assert_eq!(result["confirmation"], "not_observable");
        assert_eq!(result["after_screen_title"], "CLI-Test1");
        assert_eq!(result["target_action"]["text"], "Save screen");
        assert_eq!(runtime.mouse_events.len(), 3);
    }

    #[tokio::test]
    async fn screener_screens_save_rejects_missing_save_action() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "CLI-Test1"
            }),
            json!({
                "menu_opened": true,
                "found": false,
                "enabled": false,
                "screen_title": "CLI-Test1",
                "actions": [
                    { "index": 0, "text": "Rename", "kind": "rename", "enabled": true }
                ]
            }),
        ]);

        let error = screener_screens_save(&mut runtime, true).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert!(error.details.is_some());
    }

    #[tokio::test]
    async fn screener_screens_save_rejects_blocking_dialog_after_click() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "CLI-Test1"
            }),
            json!({
                "menu_opened": true,
                "found": true,
                "enabled": true,
                "screen_title": "CLI-Test1",
                "actions": [
                    { "index": 0, "text": "Save screen", "kind": "save", "enabled": true }
                ],
                "click_point": { "x": 100.0, "y": 200.0 }
            }),
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "CLI-Test1",
                "blocking_dialog_found": true,
                "blocking_dialog_text": "Rename"
            }),
        ]);

        let error = screener_screens_save(&mut runtime, false)
            .await
            .unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert!(error.details.is_some());
        assert_eq!(runtime.mouse_events.len(), 3);
    }

    #[tokio::test]
    async fn screener_screens_list_catalog_returns_catalog_entries() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "dialog_title": "Stock Screener",
                "screen_title": "CLI-Test1"
            }),
            json!({
                "found": true,
                "menu_opened": true,
                "screen_title": "CLI-Test1",
                "click_point": { "x": 100.0, "y": 200.0 }
            }),
            json!({
                "catalog_opened": true,
                "screen_title": "CLI-Test1",
                "screens": [
                    { "index": 0, "name": "CLI-Test1", "active": true },
                    { "index": 1, "name": "CLI-Test2", "active": false }
                ]
            }),
        ]);

        let result = screener_screens_list(&mut runtime, true).await.unwrap();

        assert_eq!(result["source"], SCREENER_SOURCE);
        assert_eq!(result["scope"], "screen_catalog");
        assert_eq!(result["screen_title"], "CLI-Test1");
        assert_eq!(result["screen_count"], 2);
        assert_eq!(result["screens"][1]["name"], "CLI-Test2");
        assert_eq!(result["screens"][1]["active"], false);
    }

    #[tokio::test]
    async fn screener_screens_switch_dry_run_returns_target_without_clicking() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "米国株（テスト用）"
            }),
            json!({
                "menu_opened": true,
                "catalog_entry_found": true,
                "screens": [
                    { "index": 0, "name": "米国株（テスト用）", "active": true },
                    { "index": 1, "name": "米国株", "active": false }
                ]
            }),
        ]);

        let result = screener_screens_switch(&mut runtime, "米国株", true, false)
            .await
            .unwrap();

        assert_eq!(result["action"], "screen_switch");
        assert_eq!(result["dry_run"], true);
        assert_eq!(result["switched"], false);
        assert_eq!(result["already_active"], false);
        assert_eq!(result["target_screen"]["name"], "米国株");
        assert_eq!(result["before_screen_title"], "米国株（テスト用）");
        assert_eq!(result["after_screen_title"], "米国株（テスト用）");
    }

    #[tokio::test]
    async fn screener_screens_switch_catalog_dry_run_returns_target_without_clicking() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "CLI-Test1"
            }),
            json!({
                "found": true,
                "menu_opened": true,
                "screen_title": "CLI-Test1",
                "click_point": { "x": 100.0, "y": 200.0 }
            }),
            json!({
                "catalog_opened": true,
                "screen_title": "CLI-Test1",
                "screens": [
                    { "index": 0, "name": "CLI-Test1", "active": true },
                    { "index": 1, "name": "CLI-Test2", "active": false }
                ]
            }),
        ]);

        let result = screener_screens_switch(&mut runtime, "CLI-Test2", true, true)
            .await
            .unwrap();

        assert_eq!(result["action"], "screen_switch");
        assert_eq!(result["scope"], "screen_catalog");
        assert_eq!(result["dry_run"], true);
        assert_eq!(result["switched"], false);
        assert_eq!(result["target_screen"]["name"], "CLI-Test2");
        assert_eq!(runtime.mouse_events.len(), 3);
    }

    #[tokio::test]
    async fn screener_screens_switch_clicks_target_and_verifies_after_title() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "米国株（テスト用）"
            }),
            json!({
                "menu_opened": true,
                "catalog_entry_found": true,
                "screens": [
                    { "index": 0, "name": "米国株（テスト用）", "active": true },
                    { "index": 1, "name": "米国株", "active": false }
                ]
            }),
            json!({
                "found": true,
                "menu_opened": true,
                "target_name": "米国株",
                "click_point": { "x": 120.0, "y": 240.0 }
            }),
            json!({
                "matched": true,
                "button_found": true,
                "open": true,
                "screen_title": "米国株"
            }),
        ]);

        let result = screener_screens_switch(&mut runtime, "米国株", false, false)
            .await
            .unwrap();

        assert_eq!(result["dry_run"], false);
        assert_eq!(result["switched"], true);
        assert_eq!(result["already_active"], false);
        assert_eq!(result["before_screen_title"], "米国株（テスト用）");
        assert_eq!(result["after_screen_title"], "米国株");
        assert_eq!(runtime.mouse_events.len(), 3);
    }

    #[tokio::test]
    async fn screener_screens_switch_catalog_clicks_target_and_verifies_after_title() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "CLI-Test1"
            }),
            json!({
                "found": true,
                "menu_opened": true,
                "screen_title": "CLI-Test1",
                "click_point": { "x": 100.0, "y": 200.0 }
            }),
            json!({
                "catalog_opened": true,
                "screen_title": "CLI-Test1",
                "screens": [
                    { "index": 0, "name": "CLI-Test1", "active": true },
                    { "index": 1, "name": "CLI-Test2", "active": false }
                ]
            }),
            json!({
                "found": true,
                "menu_opened": true,
                "screen_title": "CLI-Test1",
                "click_point": { "x": 100.0, "y": 200.0 }
            }),
            json!({
                "found": true,
                "catalog_opened": true,
                "target_name": "CLI-Test2",
                "click_point": { "x": 320.0, "y": 420.0 }
            }),
            json!({
                "matched": true,
                "button_found": true,
                "open": true,
                "screen_title": "CLI-Test2"
            }),
        ]);

        let result = screener_screens_switch(&mut runtime, "CLI-Test2", false, true)
            .await
            .unwrap();

        assert_eq!(result["scope"], "screen_catalog");
        assert_eq!(result["dry_run"], false);
        assert_eq!(result["switched"], true);
        assert_eq!(result["before_screen_title"], "CLI-Test1");
        assert_eq!(result["after_screen_title"], "CLI-Test2");
        assert_eq!(runtime.mouse_events.len(), 9);
    }

    #[tokio::test]
    async fn screener_screens_switch_rejects_missing_visible_target() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "米国株（テスト用）"
            }),
            json!({
                "menu_opened": true,
                "screens": [
                    { "index": 0, "name": "米国株（テスト用）", "active": true }
                ]
            }),
        ]);

        let error = screener_screens_switch(&mut runtime, "米国株", true, false)
            .await
            .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(error.details.is_some());
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
    async fn screener_filters_actions_reports_detected_capabilities() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "米国株（テスト用）",
                "filters": [
                    { "index": 0, "text": "EMA (21)未満価格 : 0% 〜 10%", "data_name": "screener-filter-pill-ema", "visible": true }
                ],
                "filter_count": 1
            }),
            json!({
                "add_button_found": true,
                "add_supported": false,
                "numeric_modify_supported": true,
                "candidate_filter": { "text": "EMA (21)未満価格 : 0% 〜 10%", "data_name": "screener-filter-pill-ema" },
                "range_options": [
                    { "index": 0, "text": "0% 〜 5%", "normalized_text": "0% 〜 5%" },
                    { "index": 1, "text": "10%以上", "normalized_text": "10%以上" }
                ],
                "unavailable_reason": null
            }),
        ]);

        let result = screener_filters_actions(&mut runtime).await.unwrap();

        assert_eq!(result["action"], "filters_actions");
        assert_eq!(result["screen_title"], "米国株（テスト用）");
        assert_eq!(result["filter_count"], 1);
        assert_eq!(result["add_button_found"], true);
        assert_eq!(result["add_supported"], false);
        assert_eq!(result["numeric_modify_supported"], true);
        assert_eq!(
            result["candidate_filter"]["data_name"],
            "screener-filter-pill-ema"
        );
        assert_eq!(result["range_options"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn screener_filters_modify_dry_run_returns_target_without_mutation() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "filters": [
                    { "index": 0, "text": "EMA (21)未満価格 : 0% 〜 10%", "data_name": "screener-filter-pill-ema", "visible": true },
                    { "index": 1, "text": "PER", "data_name": "screener-filter-pill-pe", "visible": true }
                ],
                "filter_count": 2
            }),
        ]);
        let request =
            validate_screener_filter_modify_request(None, Some("EMA"), Some(0.0), Some(5.0), true)
                .unwrap();

        let result = screener_filters_modify(&mut runtime, request)
            .await
            .unwrap();

        assert_eq!(result["action"], "filter_modify");
        assert_eq!(result["dry_run"], true);
        assert_eq!(result["modified"], false);
        assert_eq!(
            result["target_filter"]["data_name"],
            "screener-filter-pill-ema"
        );
        assert_eq!(result["requested_range"]["preset_label"], "0% 〜 5%");
        assert_eq!(result["before_filter_count"], 2);
        assert_eq!(result["after_filter_count"], 2);
    }

    #[tokio::test]
    async fn screener_filters_modify_clicks_preset_and_verifies_text() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "filters": [
                    { "index": 0, "text": "EMA (21)未満価格 : 0% 〜 10%", "data_name": "screener-filter-pill-ema", "visible": true }
                ],
                "filter_count": 1
            }),
            json!({
                "found": true,
                "manual_settings_found": true,
                "range_combobox_found": true,
                "range_option_found": true,
                "clicked": true,
                "requested_range": "0% 〜 5%"
            }),
            json!({
                "button_found": true,
                "open": true,
                "filters": [
                    { "index": 0, "text": "EMA (21)未満価格 : 0% 〜 5%", "data_name": "screener-filter-pill-ema", "visible": true }
                ],
                "filter_count": 1
            }),
        ]);
        let request =
            validate_screener_filter_modify_request(Some(0), None, Some(0.0), Some(5.0), false)
                .unwrap();

        let result = screener_filters_modify(&mut runtime, request)
            .await
            .unwrap();

        assert_eq!(result["action"], "filter_modify");
        assert_eq!(result["dry_run"], false);
        assert_eq!(result["modified"], true);
        assert_eq!(
            result["target_filter"]["data_name"],
            "screener-filter-pill-ema"
        );
        assert_eq!(
            result["after_filter"]["text"],
            "EMA (21)未満価格 : 0% 〜 5%"
        );
        assert_eq!(result["before_filter_count"], 1);
        assert_eq!(result["after_filter_count"], 1);
        assert_eq!(runtime.mouse_events.len(), 0);
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
    async fn screener_columns_remove_dry_run_returns_target_without_mutation() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "columns": ["Symbol", "Price", "Change %"],
                "column_count": 3
            }),
            json!({
                "settings_button_found": true,
                "settings_opened": true,
                "categories": [],
                "header_menu_actions": [],
                "remove_supported": false,
                "reset_supported": false,
                "unavailable_reason": "visible_column_remove_action_not_found"
            }),
        ]);

        let result = screener_columns_remove(
            &mut runtime,
            ScreenerColumnSelector::Name("Change".to_string()),
            true,
        )
        .await
        .unwrap();

        assert_eq!(result["action"], "columns_remove");
        assert_eq!(result["dry_run"], true);
        assert_eq!(result["removed"], false);
        assert_eq!(result["target_column"]["index"], 2);
        assert_eq!(result["target_column"]["name"], "Change %");
        assert_eq!(result["remove_supported"], false);
        assert_eq!(runtime.mouse_events.len(), 0);
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
