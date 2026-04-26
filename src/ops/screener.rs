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

#[derive(Clone, Debug, PartialEq)]
pub struct ScreenerFilterAddRequest {
    pub name: String,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub dry_run: bool,
    range_matchers: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScreenerColumnAddRequest {
    pub id: String,
    pub params: Value,
    pub after_index: Option<usize>,
    pub dry_run: bool,
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
    id: Option<String>,
    name: String,
    active: bool,
    owner: Option<bool>,
    shared: Option<bool>,
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

#[derive(Clone, Debug, PartialEq)]
struct ScreenerStorageColumnTarget {
    index: usize,
    id: String,
    name: Option<String>,
    params: Value,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ScreenMenuClickPoint {
    x: f64,
    y: f64,
}

type ScreenerClickPoint = ScreenMenuClickPoint;

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

pub fn validate_screener_filter_add_request(
    name: &str,
    min: Option<f64>,
    max: Option<f64>,
    dry_run: bool,
) -> Result<ScreenerFilterAddRequest, AppError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "--name must not be empty",
        ));
    }
    let range_matchers = screener_filter_add_range_matchers(min, max)?;
    Ok(ScreenerFilterAddRequest {
        name: name.to_string(),
        min,
        max,
        dry_run,
        range_matchers,
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

pub fn validate_screener_column_add_request(
    id: &str,
    params_json: Option<&str>,
    after_index: Option<usize>,
    dry_run: bool,
) -> Result<ScreenerColumnAddRequest, AppError> {
    let id = id.trim();
    if id.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "screener columns add requires a non-empty --id",
        ));
    }
    let params = match params_json {
        Some(raw) => {
            let value: Value = serde_json::from_str(raw).map_err(|error| {
                AppError::new(
                    ErrorKind::Validation,
                    format!("--params-json must be valid JSON: {error}"),
                )
            })?;
            if !value.is_object() {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "--params-json must be a JSON object",
                )
                .with_details(json!({ "params_json": value })));
            }
            value
        }
        None => Value::Object(Map::new()),
    };
    Ok(ScreenerColumnAddRequest {
        id: id.to_string(),
        params,
        after_index,
        dry_run,
    })
}

pub fn validate_screener_column_reorder_request(
    from_index: usize,
    to_index: usize,
) -> Result<(usize, usize), AppError> {
    if from_index == to_index {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Screener columns reorder requires different --from-index and --to-index values",
        ));
    }
    Ok((from_index, to_index))
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

pub fn validate_screener_screen_rename_request(
    name: &str,
    new_name: &str,
    dry_run: bool,
) -> Result<(String, String), AppError> {
    let name = validate_screener_screen_name(name)?;
    let new_name = validate_screener_screen_name(new_name)?;
    if name == new_name {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Screener rename requires a different --to name",
        ));
    }
    if !dry_run
        && (!is_test_screener_screen_name(&name) || !is_test_screener_screen_name(&new_name))
    {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Screener rename mutation is limited to test screen names containing CLI-Test or テスト",
        ));
    }
    Ok((name, new_name))
}

pub fn validate_screener_screen_test_mutation_name(
    name: &str,
    dry_run: bool,
    operation: &str,
) -> Result<String, AppError> {
    let name = validate_screener_screen_name(name)?;
    if !dry_run && !is_test_screener_screen_name(&name) {
        return Err(AppError::new(
            ErrorKind::Validation,
            format!(
                "Screener {operation} mutation is limited to test screen names containing CLI-Test or テスト"
            ),
        ));
    }
    Ok(name)
}

pub fn validate_screener_screen_delete_request(
    name: &str,
    dry_run: bool,
    confirm_delete: bool,
) -> Result<String, AppError> {
    let name = validate_screener_screen_name(name)?;
    if !dry_run && !confirm_delete {
        return Err(AppError::new(
            ErrorKind::Validation,
            "screener screens delete requires --confirm-delete unless --dry-run is used",
        ));
    }
    if !dry_run && !is_test_screener_screen_name(&name) {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Screener delete mutation is limited to test screen names containing CLI-Test or テスト",
        ));
    }
    Ok(name)
}

fn is_test_screener_screen_name(name: &str) -> bool {
    name.contains("CLI-Test") || name.contains("テスト")
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
        close_screener_screen_lifecycle_popups(session.runtime).await?;
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
        close_screener_screen_lifecycle_popups(session.runtime).await?;
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

pub async fn screener_screens_create(
    runtime: &mut impl RuntimeEvaluator,
    name: &str,
    dry_run: bool,
) -> Result<Value, AppError> {
    screener_screens_name_dialog_mutation(runtime, "create", "screen_create", name, dry_run).await
}

pub async fn screener_screens_rename(
    runtime: &mut impl RuntimeEvaluator,
    name: &str,
    new_name: &str,
    dry_run: bool,
) -> Result<Value, AppError> {
    let mut session = ScreenerMutationSession::open(runtime).await?;
    let before_state = read_screener_state(session.runtime, None).await?;
    ensure_dialog_open(&before_state)?;
    let before_title = before_state
        .get("screen_title")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    if before_title != name {
        let _ = session.restore().await;
        return Err(AppError::new(
            ErrorKind::Validation,
            format!("Active Screener screen is {before_title:?}, not {name:?}"),
        )
        .with_details(json!({
            "active_screen_title": before_title,
            "requested_screen_title": name,
        })));
    }
    let dialog = open_screen_name_dialog(session.runtime, "rename").await?;
    let actions = screen_actions_from_menu(&dialog);
    let target_action = resolve_screen_action(&actions, "rename")?;
    if dry_run {
        close_screener_screen_lifecycle_popups(session.runtime).await?;
        let close_result = session.restore().await;
        close_result?;
        return Ok(json!({
            "source": SCREENER_SOURCE,
            "action": "screen_rename",
            "scope": "screen_title_menu",
            "dry_run": true,
            "renamed": false,
            "opened_for_mutation": session.opened_for_mutation,
            "restored_open_state": session.restored_open_state,
            "before_screen_title": before_title,
            "after_screen_title": before_title,
            "target_screen": { "name": name },
            "new_name": new_name,
            "target_action": screen_action_payload(&target_action),
            "dialog": screen_name_dialog_payload(&dialog),
        }));
    }

    session.runtime.insert_text(new_name).await?;
    let submit = find_screen_name_dialog_submit_point(session.runtime, "rename").await?;
    let point = screen_menu_click_point(&submit)?;
    dispatch_screen_menu_click(session.runtime, point).await?;
    let after_state_result = wait_for_screen_title(session.runtime, new_name).await;
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
        "action": "screen_rename",
        "scope": "screen_title_menu",
        "dry_run": false,
        "renamed": true,
        "opened_for_mutation": session.opened_for_mutation,
        "restored_open_state": session.restored_open_state,
        "before_screen_title": before_title,
        "after_screen_title": after_title,
        "target_screen": { "name": name },
        "new_name": new_name,
        "target_action": screen_action_payload(&target_action),
    }))
}

pub async fn screener_screens_save_as(
    runtime: &mut impl RuntimeEvaluator,
    name: &str,
    dry_run: bool,
) -> Result<Value, AppError> {
    screener_screens_name_dialog_mutation(runtime, "make_copy", "screen_save_as", name, dry_run)
        .await
}

pub async fn screener_screens_delete(
    runtime: &mut impl RuntimeEvaluator,
    name: &str,
    dry_run: bool,
    confirm_delete: bool,
) -> Result<Value, AppError> {
    let before_state = read_screener_state(runtime, None).await?;
    let before_screen_title = before_state
        .get("screen_title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .map(str::to_string);
    let storage = fetch_screener_storage_screens(runtime, before_screen_title.as_deref()).await?;
    let screens = screen_targets_from_menu(&storage);
    let target = resolve_screen_target(&screens, name)?;

    if dry_run {
        return Ok(json!({
            "source": SCREENER_SOURCE,
            "action": "screen_delete",
            "scope": "screen_storage_api",
            "dry_run": true,
            "deleted": false,
            "confirmed": false,
            "before_screen_title": before_screen_title,
            "target_screen": screen_target_payload(&target),
            "screen_count": screens.len(),
            "screens": screen_targets_payload(&screens),
            "delete_supported": true,
        }));
    }

    if target.active {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Refusing to delete the active Screener screen; switch to another screen first",
        )
        .with_details(json!({ "target_screen": screen_target_payload(&target) })));
    }

    let _ = confirm_delete;
    let deleted = delete_screener_storage_screen(runtime, &target).await?;
    let after_storage =
        fetch_screener_storage_screens(runtime, before_screen_title.as_deref()).await?;
    let after_screens = screen_targets_from_menu(&after_storage);
    if after_screens
        .iter()
        .any(|screen| screen.name == target.name)
    {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener screen still appears after delete",
        )
        .with_details(json!({
            "target_screen": screen_target_payload(&target),
            "delete_result": deleted,
            "screens": screen_targets_payload(&after_screens),
        })));
    }

    Ok(json!({
        "source": SCREENER_SOURCE,
        "action": "screen_delete",
        "scope": "screen_storage_api",
        "dry_run": false,
        "deleted": true,
        "confirmed": true,
        "before_screen_title": before_screen_title,
        "target_screen": screen_target_payload(&target),
        "delete_result": deleted,
        "before_screen_count": screens.len(),
        "after_screen_count": after_screens.len(),
        "screens": screen_targets_payload(&after_screens),
    }))
}

async fn screener_screens_name_dialog_mutation(
    runtime: &mut impl RuntimeEvaluator,
    action_kind: &str,
    payload_action: &str,
    name: &str,
    dry_run: bool,
) -> Result<Value, AppError> {
    let mut session = ScreenerMutationSession::open(runtime).await?;
    let before_state = read_screener_state(session.runtime, None).await?;
    ensure_dialog_open(&before_state)?;
    let before_title = before_state
        .get("screen_title")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let dialog = open_screen_name_dialog(session.runtime, action_kind).await?;
    let actions = screen_actions_from_menu(&dialog);
    let target_action = resolve_screen_action(&actions, action_kind)?;
    if dry_run {
        close_screener_screen_lifecycle_popups(session.runtime).await?;
        let close_result = session.restore().await;
        close_result?;
        return Ok(json!({
            "source": SCREENER_SOURCE,
            "action": payload_action,
            "scope": "screen_title_menu",
            "dry_run": true,
            "created": false,
            "opened_for_mutation": session.opened_for_mutation,
            "restored_open_state": session.restored_open_state,
            "before_screen_title": before_title,
            "after_screen_title": before_title,
            "name": name,
            "target_action": screen_action_payload(&target_action),
            "dialog": screen_name_dialog_payload(&dialog),
        }));
    }

    session.runtime.insert_text(name).await?;
    let submit = find_screen_name_dialog_submit_point(session.runtime, action_kind).await?;
    let point = screen_menu_click_point(&submit)?;
    dispatch_screen_menu_click(session.runtime, point).await?;
    let after_state_result = wait_for_screen_title(session.runtime, name).await;
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
        "action": payload_action,
        "scope": "screen_title_menu",
        "dry_run": false,
        "created": true,
        "opened_for_mutation": session.opened_for_mutation,
        "restored_open_state": session.restored_open_state,
        "before_screen_title": before_title,
        "after_screen_title": after_title,
        "name": name,
        "target_action": screen_action_payload(&target_action),
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

pub async fn screener_filters_add(
    runtime: &mut impl RuntimeEvaluator,
    request: ScreenerFilterAddRequest,
) -> Result<Value, AppError> {
    let mut session = ScreenerMutationSession::open(runtime).await?;
    let before_state = read_screener_state(session.runtime, None).await?;
    ensure_dialog_open(&before_state)?;
    let before_filters = filter_targets_from_state(&before_state);
    let requested_range = filter_add_range_payload(&request);

    let search_result = open_filter_add_search(session.runtime).await?;
    ensure_filter_add_search_opened(&search_result)?;
    session.runtime.insert_text(&request.name).await?;
    let candidate = wait_for_filter_add_candidate(session.runtime, &request.name).await?;

    if request.dry_run {
        close_screener_transient_popups(session.runtime).await?;
        let close_result = session.restore().await;
        close_result?;
        return Ok(json!({
            "source": SCREENER_SOURCE,
            "action": "filter_add",
            "dry_run": true,
            "added": false,
            "open": value_bool(&before_state, "open"),
            "opened_for_mutation": session.opened_for_mutation,
            "restored_open_state": session.restored_open_state,
            "before_filter_count": before_filters.len(),
            "after_filter_count": before_filters.len(),
            "target_filter": candidate.get("candidate").cloned().unwrap_or(Value::Null),
            "requested_range": requested_range,
        }));
    }

    let candidate_point = screener_click_point(&candidate, "candidate_click_point")?;
    dispatch_screen_menu_click(session.runtime, candidate_point).await?;
    let option = wait_for_filter_add_range_option(session.runtime, &request.range_matchers).await?;
    let option_point = screener_click_point(&option, "range_click_point")?;
    dispatch_screen_menu_click(session.runtime, option_point).await?;
    let after_state = wait_for_filter_added(
        session.runtime,
        &before_filters,
        &request.name,
        &request.range_matchers,
    )
    .await?;
    let after_filters = filter_targets_from_state(&after_state);
    let after_filter = added_filter_target(
        &before_filters,
        &after_filters,
        &request.name,
        &request.range_matchers,
    );
    let close_result = session.restore().await;
    close_result?;

    Ok(json!({
        "source": SCREENER_SOURCE,
        "action": "filter_add",
        "dry_run": false,
        "added": true,
        "open": value_bool(&after_state, "open"),
        "opened_for_mutation": session.opened_for_mutation,
        "restored_open_state": session.restored_open_state,
        "before_filter_count": before_filters.len(),
        "after_filter_count": after_filters.len(),
        "target_filter": candidate.get("candidate").cloned().unwrap_or(Value::Null),
        "after_filter": after_filter.as_ref().map(filter_target_payload).unwrap_or(Value::Null),
        "requested_range": requested_range,
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

fn filter_add_range_payload(request: &ScreenerFilterAddRequest) -> Value {
    json!({
        "min": request.min,
        "max": request.max,
        "matchers": request.range_matchers,
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

fn screener_filter_add_range_matchers(
    min: Option<f64>,
    max: Option<f64>,
) -> Result<Vec<String>, AppError> {
    if min.is_none() && max.is_none() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Either --min or --max is required",
        ));
    }
    if let Some(value) = min {
        require_finite(value, "--min")?;
    }
    if let Some(value) = max {
        require_finite(value, "--max")?;
    }
    match (min, max) {
        (Some(min), Some(max)) => {
            if max <= min {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "--max must be greater than --min",
                ));
            }
            let min = format_filter_percent(min);
            let max = format_filter_percent(max);
            Ok(vec![
                format!("{min}% 〜 {max}%"),
                format!("{min}% to {max}%"),
                format!("{min} 〜 {max}"),
                format!("{min} to {max}"),
            ])
        }
        (Some(min), None) => {
            let min = format_filter_percent(min);
            Ok(vec![
                format!("> {min}"),
                format!(">{min}"),
                format!("{min}%以上"),
                format!("{min}以上"),
            ])
        }
        (None, Some(max)) => {
            let max = format_filter_percent(max);
            Ok(vec![
                format!("< {max}"),
                format!("<{max}"),
                format!("{max}%以下"),
                format!("{max}以下"),
                format!("{max}%未満"),
                format!("{max}未満"),
            ])
        }
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

fn require_active_screen_title(state: &Value) -> Result<String, AppError> {
    state
        .get("screen_title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            AppError::new(
                ErrorKind::InternalApiUnavailable,
                "Active Screener screen title was not available",
            )
            .with_details(state.clone())
        })
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

fn ensure_filter_add_search_opened(value: &Value) -> Result<(), AppError> {
    if value_bool(value, "opened") && value_bool(value, "input_found") {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener filter add search did not open",
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
                        id: screen.get("id").and_then(Value::as_str).map(str::to_string),
                        name: name.to_string(),
                        active: screen
                            .get("active")
                            .and_then(Value::as_bool)
                            .unwrap_or(false),
                        owner: screen.get("owner").and_then(Value::as_bool),
                        shared: screen.get("shared").and_then(Value::as_bool),
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
        "id": screen.id,
        "name": screen.name,
        "active": screen.active,
        "owner": screen.owner,
        "shared": screen.shared,
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

fn resolve_screen_action(
    actions: &[ScreenerScreenAction],
    kind: &str,
) -> Result<ScreenerScreenAction, AppError> {
    let matches = actions
        .iter()
        .filter(|action| action.kind == kind)
        .cloned()
        .collect::<Vec<_>>();
    match matches.len() {
        0 => Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!("No visible Screener {kind} action found"),
        )
        .with_details(json!({ "actions": screen_actions_payload(actions) }))),
        1 => Ok(matches[0].clone()),
        _ => Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!("Multiple visible Screener {kind} actions found"),
        )
        .with_details(json!({ "matches": screen_actions_payload(&matches) }))),
    }
}

fn screen_name_dialog_payload(value: &Value) -> Value {
    json!({
        "dialog_opened": value_bool(value, "dialog_opened"),
        "input_found": value_bool(value, "input_found"),
        "submit_found": value_bool(value, "submit_found"),
        "initial_value": value.get("input_value").cloned().unwrap_or(Value::Null),
        "dialog_title": value.get("dialog_title").cloned().unwrap_or(Value::Null),
    })
}

async fn open_screen_name_dialog(
    runtime: &mut impl RuntimeEvaluator,
    action_kind: &str,
) -> Result<Value, AppError> {
    let action_kind = js_string(action_kind)?;
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
                        return {{ menu_opened: false, dialog_opened: false, reason: 'title_not_found', actions: [] }};
                    }}
                    var activeTitle = textOf(title);
                    mouseClick(title);
                    var menu = null;
                    for (var i = 0; i < 10; i++) {{
                        menu = findScreenerScreenMenu();
                        if (menu) break;
                        await sleep(150);
                    }}
                    if (!menu) {{
                        return {{ menu_opened: false, dialog_opened: false, reason: 'menu_not_found', screen_title: activeTitle, actions: [] }};
                    }}
                    var actions = collectScreenerScreenActions(menu);
                    var action = findScreenerScreenActionItem(menu, {action_kind});
                    if (!action) {{
                        closeScreenerScreenMenu();
                        return {{ menu_opened: true, dialog_opened: false, reason: 'action_not_found', screen_title: activeTitle, actions: actions }};
                    }}
                    mouseClick(action);
                    var dialog = null;
                    for (var j = 0; j < 12; j++) {{
                        dialog = findScreenerScreenNameDialog({action_kind});
                        if (dialog) break;
                        await sleep(150);
                    }}
                    if (!dialog) {{
                        return {{ menu_opened: true, dialog_opened: false, reason: 'dialog_not_found', screen_title: activeTitle, actions: actions }};
                    }}
                    var input = Array.from(dialog.querySelectorAll('input, textarea')).filter(visible)[0] || null;
                    var submit = findScreenerScreenNameDialogSubmit(dialog, {action_kind});
                    var dialogTitle = screenerScreenNameDialogTitle(dialog, {action_kind});
                    if (input) {{
                        input.focus();
                        if (input.select) input.select();
                    }}
                    var rect = submit ? submit.getBoundingClientRect() : null;
                    return {{
                        menu_opened: true,
                        dialog_opened: true,
                        blocking_dialog_found: false,
                        screen_title: activeTitle,
                        actions: actions,
                        dialog_title: dialogTitle,
                        input_found: !!input,
                        input_value: input ? input.value : null,
                        submit_found: !!submit,
                        click_point: rect ? {{
                            x: rect.left + rect.width / 2,
                            y: rect.top + rect.height / 2
                        }} : null
                    }};
                }})()
                "#
            )),
            true,
        )
        .await?;

    if !(value_bool(&result, "dialog_opened") && value_bool(&result, "input_found")) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener screen lifecycle dialog was not available",
        )
        .with_details(result));
    }
    Ok(result)
}

async fn find_screen_name_dialog_submit_point(
    runtime: &mut impl RuntimeEvaluator,
    action_kind: &str,
) -> Result<Value, AppError> {
    let action_kind = js_string(action_kind)?;
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
                        var dialog = findScreenerScreenNameDialog({action_kind});
                        var submit = dialog ? findScreenerScreenNameDialogSubmit(dialog, {action_kind}) : null;
                        if (submit) {{
                            var rect = submit.getBoundingClientRect();
                            return {{
                                dialog_opened: true,
                                submit_found: true,
                                click_point: {{
                                    x: rect.left + rect.width / 2,
                                    y: rect.top + rect.height / 2
                                }}
                            }};
                        }}
                        await sleep(100);
                    }}
                    return {{ dialog_opened: !!findScreenerScreenNameDialog({action_kind}), submit_found: false }};
                }})()
                "#
            )),
            true,
        )
        .await?;

    if !value_bool(&result, "submit_found") {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener screen lifecycle submit action was not available",
        )
        .with_details(result));
    }
    Ok(result)
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

async fn fetch_screener_storage_screens(
    runtime: &mut impl RuntimeEvaluator,
    active_title: Option<&str>,
) -> Result<Value, AppError> {
    let active_title = active_title.map(js_string).transpose()?;
    let active_title = active_title.unwrap_or_else(|| "null".to_string());
    let result = runtime
        .evaluate(
            &expanded_expression(&format!(
                r#"
                (async function() {{
                    REPLACE_HELPERS
                    var initData = window.initData || {{}};
                    var storageUrl = initData.SCREENER_STORAGE_URL;
                    var version = initData.screener_storage_release_version;
                    var screenerKey = initData.standalone_type ||
                        (initData.screen_data && initData.screen_data.screener_key) ||
                        'stock';
                    if (!storageUrl || !version || !screenerKey) {{
                        return {{
                            storage_available: false,
                            reason: 'missing_screener_storage_init_data',
                            screens: []
                        }};
                    }}
                    var activeTitle = {active_title};
                    var base = String(storageUrl).replace(/\/$/, '') + '/api/v2/screens/';
                    var url = base + '?screener_key=' + encodeURIComponent(screenerKey) +
                        '&version=' + encodeURIComponent(version) +
                        '&sort_by=updated&sort_order=desc';
                    var response = await fetch(url, {{ credentials: 'include' }});
                    var body = await response.json().catch(function() {{ return null; }});
                    if (!response.ok || !Array.isArray(body)) {{
                        return {{
                            storage_available: true,
                            fetch_ok: response.ok,
                            status: response.status,
                            status_text: response.statusText,
                            reason: 'custom_screens_fetch_failed',
                            screens: []
                        }};
                    }}
                    return {{
                        storage_available: true,
                        fetch_ok: true,
                        status: response.status,
                        screener_key: screenerKey,
                        screen_title: activeTitle,
                        screen_count: body.length,
                        screens: body.map(function(screen, index) {{
                            return {{
                                index: index,
                                id: String(screen.id || ''),
                                name: String(screen.title || ''),
                                active: !!activeTitle && String(screen.title || '') === activeTitle,
                                owner: screen.is_owner === undefined ? null : !!screen.is_owner,
                                shared: screen.is_shared === undefined ? null : !!screen.is_shared
                            }};
                        }}).filter(function(screen) {{ return screen.id && screen.name; }})
                    }};
                }})()
                "#
            )),
            true,
        )
        .await?;

    if value_bool(&result, "storage_available") && value_bool(&result, "fetch_ok") {
        Ok(result)
    } else {
        Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener custom screen storage API was not available",
        )
        .with_details(result))
    }
}

async fn fetch_active_screener_storage_config(
    runtime: &mut impl RuntimeEvaluator,
    expected_title: &str,
) -> Result<Value, AppError> {
    let expected_title = js_string(expected_title)?;
    let result = runtime
        .evaluate(
            &expanded_expression(&format!(
                r#"
                (async function() {{
                    REPLACE_HELPERS
                    var initData = window.initData || {{}};
                    var active = initData.screen_data || {{}};
                    var storageUrl = initData.SCREENER_STORAGE_URL;
                    var version = initData.screener_storage_release_version;
                    var screenId = String(active.id || '');
                    var screenerKey = initData.standalone_type ||
                        active.screener_key ||
                        'stock';
                    if (!storageUrl || !version || !screenId || !screenerKey) {{
                        return {{
                            storage_available: false,
                            reason: 'missing_screener_storage_init_data',
                            columns: []
                        }};
                    }}
                    var expectedTitle = {expected_title};
                    var base = String(storageUrl).replace(/\/$/, '') + '/api/v2/screens/';
                    var url = base + encodeURIComponent(screenId) + '/?screener_key=' +
                        encodeURIComponent(screenerKey) + '&version=' + encodeURIComponent(version);
                    var response = await fetch(url, {{ credentials: 'include' }});
                    var body = await response.json().catch(function() {{ return null; }});
                    if (!response.ok || !body) {{
                        return {{
                            storage_available: true,
                            fetch_ok: response.ok,
                            status: response.status,
                            status_text: response.statusText,
                            reason: 'screen_fetch_failed',
                            columns: []
                        }};
                    }}
                    var title = String(body.title || active.title || '');
                    var columns = Array.isArray(body.default_custom_column_set)
                        ? body.default_custom_column_set
                        : (Array.isArray(active.default_custom_column_set)
                            ? active.default_custom_column_set
                            : []);
                    return {{
                        storage_available: true,
                        fetch_ok: true,
                        status: response.status,
                        screener_key: screenerKey,
                        version: body.version || active.version || version,
                        screen_id: String(body.id || screenId),
                        screen_title: title,
                        expected_title: expectedTitle,
                        title_matches: title === expectedTitle,
                        active_column_set: body.active_column_set || active.active_column_set || null,
                        storage_screen: body,
                        column_count: columns.length,
                        columns: columns.map(function(column, index) {{
                            return {{
                                index: index,
                                id: String(column && column.id || ''),
                                params: column && column.params ? column.params : {{}}
                            }};
                        }}).filter(function(column) {{ return column.id; }})
                    }};
                }})()
                "#
            )),
            true,
        )
        .await?;

    if !(value_bool(&result, "storage_available") && value_bool(&result, "fetch_ok")) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener active screen storage API was not available",
        )
        .with_details(result));
    }
    if !value_bool(&result, "title_matches") {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener active screen storage title did not match visible title",
        )
        .with_details(result));
    }
    Ok(result)
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

async fn delete_screener_storage_screen(
    runtime: &mut impl RuntimeEvaluator,
    target: &ScreenerScreenTarget,
) -> Result<Value, AppError> {
    let target_id = target.id.as_deref().ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener screen storage id was not available",
        )
        .with_details(json!({ "target_screen": screen_target_payload(target) }))
    })?;
    let target_id = js_string(target_id)?;
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
                            deleted: false,
                            reason: 'missing_screener_storage_init_data'
                        }};
                    }}
                    var base = String(storageUrl).replace(/\/$/, '') + '/api/v2/screens/';
                    var targetId = {target_id};
                    var response = await fetch(base + encodeURIComponent(targetId) + '/', {{
                        method: 'DELETE',
                        credentials: 'include',
                        headers: {{ 'Content-Type': 'application/json' }}
                    }});
                    return {{
                        deleted: response.ok,
                        status: response.status,
                        status_text: response.statusText,
                        id: targetId
                    }};
                }})()
                "#
            )),
            true,
        )
        .await?;

    if value_bool(&result, "deleted") {
        Ok(result)
    } else {
        Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener custom screen delete request failed",
        )
        .with_details(result))
    }
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
    screener_click_point(value, "click_point")
}

fn screener_click_point(value: &Value, field: &str) -> Result<ScreenerClickPoint, AppError> {
    let point = value.get(field).ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener click point missing",
        )
        .with_details(value.clone())
    })?;
    let x = point.get("x").and_then(Value::as_f64).ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener click x coordinate missing",
        )
        .with_details(value.clone())
    })?;
    let y = point.get("y").and_then(Value::as_f64).ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener click y coordinate missing",
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
                        var editScope = null;
                        for (var i = 0; i < 10; i++) {
                            editScope = findScreenerFilterEditPopoverForPill(candidate);
                            if (findScreenerManualFilterButton(editScope)) {
                                manualSettingsFound = true;
                                break;
                            }
                            await sleep(100);
                        }
                        var manualButton = findScreenerManualFilterButton(editScope);
                        if (manualButton) {
                            mouseClick(manualButton);
                            for (var j = 0; j < 10; j++) {
                                rangeOptions = collectScreenerRangeOptions(editScope);
                                if (rangeOptions.length > 0) break;
                                await sleep(100);
                            }
                            if (rangeOptions.length === 0) {
                                var combo = findScreenerRangeCombobox(editScope);
                                if (combo) {
                                    mouseClick(combo);
                                    for (var k = 0; k < 10; k++) {
                                        rangeOptions = collectScreenerRangeOptions(editScope);
                                        if (rangeOptions.length > 0) break;
                                        await sleep(100);
                                    }
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

async fn open_filter_add_search(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &expanded_expression(
                r#"
                (async function() {
                    function sleep(ms) {
                        return new Promise(function(resolve) { setTimeout(resolve, ms); });
                    }
                    REPLACE_HELPERS
                    closeScreenerTransientPopups();
                    var addButton = findScreenerAddFilterButton();
                    if (!addButton) {
                        return { opened: false, reason: 'add_button_not_found' };
                    }
                    mouseClick(addButton);
                    var input = null;
                    for (var i = 0; i < 10; i++) {
                        input = findScreenerAddFilterSearchInput();
                        if (input) break;
                        await sleep(150);
                    }
                    if (!input) {
                        return { opened: false, reason: 'search_input_not_found' };
                    }
                    input.focus();
                    return {
                        opened: true,
                        input_found: true,
                        placeholder: input.getAttribute('placeholder') || null,
                        aria_label: input.getAttribute('aria-label') || null
                    };
                })()
                "#,
            ),
            true,
        )
        .await
}

async fn wait_for_filter_add_candidate(
    runtime: &mut impl RuntimeEvaluator,
    name: &str,
) -> Result<Value, AppError> {
    let name = js_string(name)?;
    let result = runtime
        .evaluate(
            &expanded_expression(&format!(
                r#"
                (async function() {{
                    function sleep(ms) {{
                        return new Promise(function(resolve) {{ setTimeout(resolve, ms); }});
                    }}
                    REPLACE_HELPERS
                    var name = {name};
                    var last = {{ found: false, reason: 'candidate_not_found', query: name }};
                    for (var i = 0; i < 12; i++) {{
                        var candidate = findScreenerAddFilterCandidate(name);
                        if (candidate) {{
                            var rect = candidate.getBoundingClientRect();
                            return {{
                                found: true,
                                candidate: {{
                                    text: textOf(candidate),
                                    normalized_text: normalizeScreenerFilterText(textOf(candidate)),
                                    role: candidate.getAttribute('role') || null
                                }},
                                candidate_click_point: {{
                                    x: rect.left + rect.width / 2,
                                    y: rect.top + rect.height / 2
                                }}
                            }};
                        }}
                        var dialog = findScreenerAddFilterDialog();
                        last = {{
                            found: false,
                            reason: 'candidate_not_found',
                            query: name,
                            dialog_text: dialog ? textOf(dialog).substring(0, 240) : null
                        }};
                        await sleep(150);
                    }}
                    return last;
                }})()
                "#
            )),
            true,
        )
        .await?;

    if value_bool(&result, "found") {
        Ok(result)
    } else {
        Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener filter add candidate was not found",
        )
        .with_details(result))
    }
}

async fn wait_for_filter_add_range_option(
    runtime: &mut impl RuntimeEvaluator,
    matchers: &[String],
) -> Result<Value, AppError> {
    let matchers = serde_json::to_string(matchers).map_err(|err| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!("Could not serialize Screener range matchers: {err}"),
        )
    })?;
    let result = runtime
        .evaluate(
            &expanded_expression(&format!(
                r#"
                (async function() {{
                    function sleep(ms) {{
                        return new Promise(function(resolve) {{ setTimeout(resolve, ms); }});
                    }}
                    REPLACE_HELPERS
                    var matchers = {matchers};
                    var last = {{ found: false, reason: 'range_option_not_found', matchers: matchers }};
                    for (var i = 0; i < 16; i++) {{
                        var option = findScreenerAddFilterRangeOption(matchers);
                        if (option) {{
                            var rect = option.getBoundingClientRect();
                            return {{
                                found: true,
                                range_option: {{
                                    text: textOf(option),
                                    normalized_text: normalizeScreenerFilterText(textOf(option)),
                                    role: option.getAttribute('role') || null
                                }},
                                range_click_point: {{
                                    x: rect.left + rect.width / 2,
                                    y: rect.top + rect.height / 2
                                }}
                            }};
                        }}
                        var dialog = findScreenerAddFilterDialog();
                        last = {{
                            found: false,
                            reason: 'range_option_not_found',
                            matchers: matchers,
                            dialog_text: dialog ? textOf(dialog).substring(0, 300) : null
                        }};
                        await sleep(150);
                    }}
                    return last;
                }})()
                "#
            )),
            true,
        )
        .await?;

    if value_bool(&result, "found") {
        Ok(result)
    } else {
        Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener filter add range option was not found",
        )
        .with_details(result))
    }
}

async fn close_screener_transient_popups(
    runtime: &mut impl RuntimeEvaluator,
) -> Result<(), AppError> {
    runtime
        .evaluate(
            &expanded_expression(
                r#"
                (function() {
                    REPLACE_HELPERS
                    closeScreenerTransientPopups();
                    return { closed: true };
                })()
                "#,
            ),
            true,
        )
        .await
        .map(|_| ())
}

async fn close_screener_screen_lifecycle_popups(
    runtime: &mut impl RuntimeEvaluator,
) -> Result<(), AppError> {
    runtime
        .evaluate(
            &expanded_expression(
                r#"
                (function() {
                    REPLACE_HELPERS
                    closeScreenerScreenLifecyclePopups();
                    return { closed: true };
                })()
                "#,
            ),
            true,
        )
        .await
        .map(|_| ())
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
                    var editScope = null;
                    for (var i = 0; i < 8; i++) {{
                        editScope = findScreenerFilterEditPopoverForPill(pill);
                        manualButton = findScreenerManualFilterButton(editScope);
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
                    var option = null;
                    var options = [];
                    var rangeOptionsOpenedDirectly = false;
                    for (var j = 0; j < 8; j++) {{
                        options = collectScreenerRangeOptions(editScope);
                        option = options.find(function(candidate) {{
                            return candidate.normalized_text === {preset_label};
                        }});
                        if (option && option.element) {{
                            rangeOptionsOpenedDirectly = true;
                            break;
                        }}
                        await sleep(75);
                    }}
                    var combo = null;
                    for (var j = 0; j < 8; j++) {{
                        if (option && option.element) break;
                        combo = findScreenerRangeCombobox(editScope);
                        if (combo) break;
                        await sleep(75);
                    }}
                    if (!combo && !rangeOptionsOpenedDirectly) {{
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
                    if (!option || !option.element) {{
                        mouseClick(combo);
                        for (var k = 0; k < 8; k++) {{
                            options = collectScreenerRangeOptions(editScope);
                            option = options.find(function(candidate) {{
                                return candidate.normalized_text === {preset_label};
                            }});
                            if (option && option.element) break;
                            await sleep(75);
                        }}
                    }}
                    if (!option || !option.element) {{
                        closeScreenerTransientPopups();
                        return {{
                            found: true,
                            manual_settings_found: true,
                            range_combobox_found: !!combo || rangeOptionsOpenedDirectly,
                            range_option_found: false,
                            requested_range: {preset_label},
                            available_options: options.map(function(candidate) {{
                                return candidate.normalized_text;
                            }}),
                            data_name: {data_name}
                        }};
                    }}
                    setTimeout(function() {{
                        mouseClick(option.element);
                    }}, 0);
                    return {{
                        found: true,
                        manual_settings_found: true,
                        range_combobox_found: !!combo || rangeOptionsOpenedDirectly,
                        range_option_found: true,
                        click_scheduled: true,
                        range_options_opened_directly: rangeOptionsOpenedDirectly,
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

async fn wait_for_filter_added(
    runtime: &mut impl RuntimeEvaluator,
    before_filters: &[ScreenerFilterTarget],
    name: &str,
    matchers: &[String],
) -> Result<Value, AppError> {
    let mut last_state = Value::Null;
    for _ in 0..12 {
        let state = read_screener_state(runtime, None).await?;
        let after_filters = filter_targets_from_state(&state);
        if added_filter_target(before_filters, &after_filters, name, matchers).is_some() {
            return Ok(state);
        }
        last_state = state;
        sleep(Duration::from_millis(250)).await;
    }

    Err(AppError::new(
        ErrorKind::InternalApiUnavailable,
        "Screener filter text did not reflect requested added filter",
    )
    .with_details(last_state))
}

fn added_filter_target(
    before_filters: &[ScreenerFilterTarget],
    after_filters: &[ScreenerFilterTarget],
    name: &str,
    matchers: &[String],
) -> Option<ScreenerFilterTarget> {
    let before_names = before_filters
        .iter()
        .map(|filter| filter.data_name.as_str())
        .collect::<std::collections::HashSet<_>>();
    let name_tokens = screener_filter_name_tokens(name);
    let numeric_tokens = screener_filter_numeric_tokens(matchers);
    after_filters
        .iter()
        .find(|filter| {
            if before_names.contains(filter.data_name.as_str()) {
                return false;
            }
            let normalized = normalize_screener_text(&filter.text).to_lowercase();
            let has_name = name_tokens
                .iter()
                .any(|token| normalized.contains(&token.to_lowercase()));
            let has_number = numeric_tokens
                .iter()
                .any(|token| normalized.contains(token));
            has_name && has_number
        })
        .cloned()
}

fn screener_filter_name_tokens(name: &str) -> Vec<String> {
    normalize_screener_text(name)
        .split(|ch: char| {
            !(ch.is_ascii_alphanumeric()
                || ('\u{3040}'..='\u{30ff}').contains(&ch)
                || ('\u{3400}'..='\u{9fff}').contains(&ch))
        })
        .map(str::trim)
        .filter(|token| token.len() >= 2)
        .map(ToString::to_string)
        .collect()
}

fn screener_filter_numeric_tokens(matchers: &[String]) -> Vec<String> {
    matchers
        .iter()
        .flat_map(|matcher| {
            matcher
                .split(|ch: char| !(ch.is_ascii_digit() || ch == '.'))
                .filter(|token| !token.is_empty())
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .collect()
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
    var style = window.getComputedStyle ? window.getComputedStyle(el) : null;
    var inViewport = rect.bottom > 0 && rect.right > 0 && rect.top < window.innerHeight && rect.left < window.innerWidth;
    return rect.width > 0 && rect.height > 0 && inViewport && (!style || (style.visibility !== 'hidden' && style.display !== 'none'));
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
    var target = document.elementFromPoint(x, y) || el;
    ['mouseover', 'mousedown', 'mouseup', 'click'].forEach(function(type) {
        target.dispatchEvent(new MouseEvent(type, {
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
function screenerScreenShortcutText(text) {
    return /^(⌘\s*S|Ctrl\s*\+\s*S|⇧\s*N|Shift\s*\+\s*N|ドット|Dot)$/i.test(text);
}
function screenerElementLabel(el) {
    return {
        text: textOf(el),
        aria_label: el.getAttribute('aria-label') || null,
        title: el.getAttribute('title') || null,
        data_name: el.getAttribute('data-name') || null
    };
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
function findScreenerScreenActionItem(menu, kind) {
    var candidates = Array.from(menu.querySelectorAll('button, [role="menuitem"], [role="option"], [role="button"], div, span')).filter(function(el) {
        return visible(el) && screenerScreenActionKind(textOf(el)) === kind;
    });
    candidates = candidates.map(function(el) {
        var current = el;
        while (current && current !== menu) {
            var cls = String(current.className || '');
            if (screenerScreenActionKind(textOf(current)) === kind &&
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
function screenerScreenNameDialogTitle(dialog, kind) {
    var text = textOf(dialog);
    if (kind === 'create' && /スクリーンを作成|Create screen/i.test(text)) return 'create';
    if (kind === 'rename' && /スクリーン名の変更|Rename screen|Change screen name/i.test(text)) return 'rename';
    if (kind === 'make_copy' && /スクリーンのコピーを作成|Make a copy|Copy screen/i.test(text)) return 'make_copy';
    return null;
}
function findScreenerScreenNameDialog(kind) {
    var inputs = visibleElements('input, textarea');
    for (var i = 0; i < inputs.length; i++) {
        var current = inputs[i];
        while (current && current !== document.body && current !== document.documentElement) {
            var rect = current.getBoundingClientRect();
            if (rect.width >= 240 && rect.height >= 80) {
                var text = textOf(current);
                if (text.length <= 800) {
                    if (kind === 'create' && /スクリーンを作成|Create screen/i.test(text)) return current;
                    if (kind === 'rename' && /スクリーン名の変更|Rename screen|Change screen name/i.test(text)) return current;
                    if (kind === 'make_copy' && /スクリーンのコピーを作成|Make a copy|Copy screen/i.test(text)) return current;
                }
            }
            current = current.parentElement;
        }
    }
    return null;
}
function findScreenerScreenNameDialogSubmit(dialog, kind) {
    return Array.from(dialog.querySelectorAll('button, [role="button"]')).filter(visible).find(function(el) {
        var text = textOf(el);
        if (kind === 'create') return /^(作成|Create)$/i.test(text);
        if (kind === 'rename') return /^(名前を変更|Rename)$/i.test(text);
        if (kind === 'make_copy') return /^(コピーを作成|Make a copy)$/i.test(text);
        return false;
    }) || null;
}
function collectScreenerScreenEntries(menu, activeTitle) {
    var seen = {};
    var entries = [];
    var nodes = Array.from(menu.querySelectorAll('button, [role="menuitem"], [role="option"], div, span')).filter(visible);
    nodes.forEach(function(el) {
        var name = textOf(el);
        if (!name || name.length > 120 || screenerScreenActionText(name)) return;
        if (screenerScreenShortcutText(name)) return;
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
function closeScreenerScreenLifecyclePopups() {
    var buttons = visibleElements('button, [role="button"], [aria-label], [title]');
    var closeButton = buttons.find(function(el) {
        var label = [textOf(el), el.getAttribute('aria-label'), el.getAttribute('title')].filter(Boolean).join(' ');
        return /^(キャンセル|Cancel|close|閉じる)$/i.test(label);
    });
    if (closeButton) {
        mouseClick(closeButton);
        return;
    }
    closeScreenerScreenCatalog();
    closeScreenerScreenMenu();
}
function findBlockingScreenerMutationDialog() {
    var containers = visibleElements('[role="dialog"], [class*="dialog"], [class*="Dialog"], [class*="modal"], [class*="Modal"], .portal-lATuqHRX');
    return containers.find(function(container) {
        if (container.querySelector && container.querySelector('table')) return false;
        var text = textOf(container);
        if (text.length > 3000) return false;
        if (/スクリーンを保存|Save screen/i.test(text) &&
            /スクリーンを開く|Open screen/i.test(text) &&
            /最近使用した項目|Recent/i.test(text)) return false;
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
function findScreenerCatalogDeleteButton(catalog) {
    var buttons = Array.from(catalog.querySelectorAll('button, [role="button"], [aria-label], [title], [data-name]')).filter(visible);
    return buttons.find(function(el) {
        var label = [textOf(el), el.getAttribute('aria-label'), el.getAttribute('title'), el.getAttribute('data-name')].filter(Boolean).join(' ');
        return /^(削除|Delete)$|削除|Delete/i.test(label);
    }) || null;
}
function findScreenerDeleteConfirmDialog(targetName) {
    var containers = visibleElements('[role="dialog"], [class*="dialog"], [class*="Dialog"], [class*="modal"], [class*="Modal"], .portal-lATuqHRX');
    return containers.find(function(container) {
        if (container.querySelector && container.querySelector('table')) return false;
        var text = textOf(container);
        if (text.length > 1000) return false;
        return /削除|Delete/i.test(text) && (!targetName || text.indexOf(targetName) >= 0 || /本当に|Are you sure|確認/i.test(text));
    }) || null;
}
function findScreenerDeleteConfirmButton(dialog) {
    return Array.from(dialog.querySelectorAll('button, [role="button"]')).filter(visible).find(function(el) {
        return /^(削除|Delete)$/i.test(textOf(el));
    }) || null;
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
function findScreenerAddFilterDialog() {
    return visibleElements('[role="dialog"], [class*="popover"], [class*="contentDefaultAppearance"]').find(function(el) {
        var text = normalizeScreenerFilterText(textOf(el));
        return /フィルター|Filter/i.test(text) && el.getBoundingClientRect().width >= 180;
    }) || null;
}
function findScreenerAddFilterSearchInput() {
    var dialog = findScreenerAddFilterDialog();
    if (!dialog) return null;
    return scopedVisibleElements(dialog, 'input, textarea').find(function(input) {
        var label = [input.getAttribute('placeholder'), input.getAttribute('aria-label'), input.getAttribute('role')].filter(Boolean).join(' ');
        return /検索|Search|combobox/i.test(label);
    }) || null;
}
function findScreenerAddFilterCandidate(name) {
    var dialog = findScreenerAddFilterDialog();
    if (!dialog) return null;
    var normalizedName = normalizeScreenerFilterText(name).toLowerCase();
    var options = scopedVisibleElements(dialog, '[role="option"], button, [role="button"], [data-name]');
    var exact = options.find(function(option) {
        return normalizeScreenerFilterText(textOf(option)).toLowerCase() === normalizedName;
    });
    if (exact) return exact;
    return options.find(function(option) {
        var text = normalizeScreenerFilterText(textOf(option)).toLowerCase();
        return text.indexOf(normalizedName) >= 0 || normalizedName.indexOf(text) >= 0;
    }) || null;
}
function findScreenerAddFilterRangeOption(matchers) {
    var dialog = findScreenerAddFilterDialog();
    if (!dialog) return null;
    var normalizedMatchers = matchers.map(function(matcher) {
        return normalizeScreenerFilterText(matcher).toLowerCase();
    }).filter(Boolean);
    return scopedVisibleElements(dialog, '[role="option"], button, [role="button"], [data-name]').find(function(option) {
        var text = normalizeScreenerFilterText(textOf(option)).toLowerCase();
        return normalizedMatchers.some(function(matcher) {
            return text.indexOf(matcher) >= 0;
        });
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
function scopedVisibleElements(scope, selector) {
    return Array.from((scope || document).querySelectorAll(selector)).filter(visible);
}
function screenerFilterPopoverTokens(text) {
    return normalizeScreenerFilterText(text)
        .replace(/-?\d+(?:\.\d+)?%\s*(?:〜|to)\s*-?\d+(?:\.\d+)?%/ig, ' ')
        .replace(/-?\d+(?:\.\d+)?%\s*(?:以上|以下|未満|or more|less)/ig, ' ')
        .split(/[^A-Za-z\u3040-\u30ff\u3400-\u9fff]+/)
        .filter(function(token) {
            return token.length >= 2 && !/^(未満|以上|以下|価格|Price)$/.test(token);
        });
}
function findScreenerFilterEditPopoverForPill(pill) {
    if (!pill) return null;
    var tokens = screenerFilterPopoverTokens(textOf(pill));
    var popovers = visibleElements('[role="dialog"], [class*="popover"], [class*="contentDefaultAppearance"]').filter(function(el) {
        var rect = el.getBoundingClientRect();
        return rect.width >= 160 && rect.height >= 80;
    });
    var scored = popovers.map(function(popover) {
        var text = normalizeScreenerFilterText(textOf(popover)).toLowerCase();
        var score = tokens.reduce(function(total, token) {
            return total + (text.indexOf(token.toLowerCase()) >= 0 ? 1 : 0);
        }, 0);
        return { popover: popover, score: score };
    }).filter(function(candidate) {
        return candidate.score > 0;
    });
    scored.sort(function(a, b) {
        if (b.score !== a.score) return b.score - a.score;
        var ar = a.popover.getBoundingClientRect();
        var br = b.popover.getBoundingClientRect();
        return ar.top - br.top;
    });
    return scored.length > 0 ? scored[0].popover : null;
}
function findScreenerManualFilterButton(scope) {
    return scopedVisibleElements(scope, 'button, [role="button"], [role="menuitem"], div, span').find(function(el) {
        var text = textOf(el);
        return /手動で設定|Set manually|Manual/i.test(text) && text.length < 120;
    }) || null;
}
function findScreenerRangeCombobox(scope) {
    var candidates = scopedVisibleElements(scope, 'button, [role="button"], [role="combobox"], [aria-haspopup], div, span');
    candidates = candidates.filter(function(el) {
        var text = normalizeScreenerFilterText(textOf(el));
        if (!text || text.length > 80) return false;
        if (el.closest && el.closest('[data-name^="screener-filter-pill-"]')) return false;
        return /%/.test(text) && (/〜|以上|以下|未満|to|or more|less/i.test(text));
    });
    candidates.sort(function(a, b) {
        var ar = a.getBoundingClientRect();
        var br = b.getBoundingClientRect();
        return (ar.width * ar.height) - (br.width * br.height);
    });
    return candidates[0] || null;
}
function collectScreenerRangeOptions(scope) {
    var seen = {};
    var options = [];
    function rangeLabel(text) {
        var normalized = normalizeScreenerFilterText(text);
        var match = normalized.match(/-?\d+(?:\.\d+)?%\s*(?:〜|to)\s*-?\d+(?:\.\d+)?%/i) ||
            normalized.match(/-?\d+(?:\.\d+)?%\s*(?:以上|以下|未満)/) ||
            normalized.match(/-?\d+(?:\.\d+)?%\s*(?:or more|less)/i);
        return match ? match[0] : null;
    }
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
    var scopes = scopedVisibleElements(scope, '[role="listbox"], [role="dialog"], [class*="popover"], [class*="menu"], [class*="contentDefaultAppearance"]').filter(function(candidateScope) {
        var rect = candidateScope.getBoundingClientRect();
        var text = normalizeScreenerFilterText(textOf(candidateScope));
        return rect.width >= 100 && rect.height >= 80 && /%/.test(text) && (/〜|以上|以下|未満|to|or more|less/i.test(text));
    });
    var nodes = [];
    if (scopes.length > 0) {
        scopes.forEach(function(scope) {
            nodes = nodes.concat(Array.from(scope.querySelectorAll('[role="option"], [role="menuitem"], button, div, span')).filter(visible));
        });
    } else {
        nodes = visibleElements('[role="option"], [role="menuitem"], button, div, span');
    }
    nodes.forEach(function(el) {
        var label = rangeLabel(textOf(el));
        if (!label) return;
        if (seen[label]) return;
        seen[label] = true;
        options.push({
            index: options.length,
            text: label,
            normalized_text: label,
            element: optionClickTarget(el, label)
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
function nearestScreenerPanelRoot(el) {
    var current = el;
    while (current && current !== document.body && current !== document.documentElement) {
        var rect = current.getBoundingClientRect();
        if (rect.width >= 260 && rect.height >= 160) {
            var hasTitle = !!current.querySelector('[data-name="screener-topbar-screen-title"]');
            var hasFilters = !!current.querySelector('[data-name^="screener-filter-pill-"]');
            var hasTable = !!current.querySelector('table');
            var dataName = current.getAttribute('data-name') || '';
            var className = String(current.className || '');
            if (hasTitle || hasFilters || hasTable || /screenerContainer|screener-container/i.test(className) || /screener/i.test(dataName)) {
                return current;
            }
        }
        current = current.parentElement;
    }
    return null;
}
function findScreenerPanelRoot(button) {
    var roots = visibleElements('[class*="screenerContainer"], [class*="screener-container"]').filter(function(el) {
        return el !== button && el.getBoundingClientRect().width >= 260 && el.getBoundingClientRect().height >= 160;
    });
    if (roots.length > 0) return roots[0];
    var anchors = visibleElements('[data-name="screener-topbar-screen-title"], [data-name^="screener-filter-pill-"], table').filter(function(el) {
        return el !== button && !buttonContains(button, el);
    });
    for (var i = 0; i < anchors.length; i++) {
        var root = nearestScreenerPanelRoot(anchors[i]);
        if (root) return root;
    }
    return null;
}
function buttonContains(button, el) {
    return !!(button && el && button !== el && button.contains(el));
}
function readScreenerState(limit) {
    var button = document.querySelector('[data-name="screener-dialog-button"]');
    var panelRoot = findScreenerPanelRoot(button);
    var screenerDataElements = panelRoot ? scopedVisibleElements(panelRoot, '[data-name*="screener"]') : [];
    var classElements = panelRoot ? scopedVisibleElements(panelRoot, '[class*="screener"]') : [];
    var heading = panelRoot ? Array.from(panelRoot.querySelectorAll('h1, h2, h3'))
        .find(function(el) {
            var text = textOf(el);
            return visible(el) && text.length <= 120 && /screener|スクリーナー/i.test(text);
        }) || Array.from(panelRoot.querySelectorAll('button, div, span'))
        .find(function(el) {
            var text = textOf(el);
            return visible(el) && text.length <= 120 && /screener|スクリーナー/i.test(text);
        }) || null : null;
    var table = panelRoot ? (Array.from(panelRoot.querySelectorAll('table')).filter(visible)[0] || null) : null;
    var title = panelRoot ? panelRoot.querySelector('[data-name="screener-topbar-screen-title"]') : null;
    var open = !!panelRoot;
    var filters = panelRoot ? scopedVisibleElements(panelRoot, '[data-name^="screener-filter-pill-"]').map(function(el) {
        return {
            text: textOf(el),
            data_name: el.getAttribute('data-name') || null,
            visible: visible(el)
        };
    }) : [];
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
        panel_root_found: !!panelRoot,
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
                "default_custom_column_set": columns
            },
            "columns": columns
        })
    }

    fn storage_column(id: &str) -> Value {
        json!({ "id": id, "params": {} })
    }

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
    fn validate_screener_filter_add_accepts_generic_numeric_presets() {
        let request =
            validate_screener_filter_add_request(" RSI (相対力指数) ", Some(70.0), None, true)
                .unwrap();

        assert_eq!(request.name, "RSI (相対力指数)");
        assert_eq!(
            request.range_matchers,
            vec!["> 70", ">70", "70%以上", "70以上"]
        );
        assert_eq!(filter_add_range_payload(&request)["min"], 70.0);

        let request = validate_screener_filter_add_request("RSI", None, Some(30.0), false).unwrap();
        assert!(request.range_matchers.contains(&"< 30".to_string()));

        let request =
            validate_screener_filter_add_request("Change", Some(0.0), Some(5.0), false).unwrap();
        assert!(request.range_matchers.contains(&"0% 〜 5%".to_string()));
    }

    #[test]
    fn validate_screener_filter_add_rejects_unsafe_inputs() {
        assert_eq!(
            validate_screener_filter_add_request("   ", Some(70.0), None, true)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_filter_add_request("RSI", None, None, true)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_filter_add_request("RSI", Some(f64::NAN), None, true)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_filter_add_request("RSI", Some(70.0), Some(60.0), true)
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
    fn validate_screener_column_reorder_rejects_same_index() {
        assert_eq!(
            validate_screener_column_reorder_request(1, 2).unwrap(),
            (1, 2)
        );
        assert_eq!(
            validate_screener_column_reorder_request(1, 1)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
    }

    #[test]
    fn validate_screener_column_add_rejects_unsafe_inputs() {
        let request = validate_screener_column_add_request(
            " TechnicalRating ",
            Some(r#"{"resolution":"TimeResolution1D"}"#),
            Some(11),
            true,
        )
        .unwrap();

        assert_eq!(request.id, "TechnicalRating");
        assert_eq!(request.params["resolution"], "TimeResolution1D");
        assert_eq!(request.after_index, Some(11));
        assert!(request.dry_run);

        assert_eq!(
            validate_screener_column_add_request("   ", None, None, true)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_column_add_request("Price", Some("{bad"), None, true)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_column_add_request("Price", Some("[]"), None, true)
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

    #[test]
    fn validate_screener_screen_lifecycle_requests_are_guarded() {
        assert_eq!(
            validate_screener_screen_test_mutation_name(" CLI-Test-New ", false, "create").unwrap(),
            "CLI-Test-New"
        );
        assert_eq!(
            validate_screener_screen_test_mutation_name("Production", false, "create")
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_screen_rename_request("CLI-Test1", "CLI-Test2", false).unwrap(),
            ("CLI-Test1".to_string(), "CLI-Test2".to_string())
        );
        assert_eq!(
            validate_screener_screen_rename_request("CLI-Test1", "CLI-Test1", true)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_screen_rename_request("Production", "CLI-Test2", false)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert!(validate_screener_screen_delete_request("CLI-Test1", true, false).is_ok());
        assert_eq!(
            validate_screener_screen_delete_request("CLI-Test1", false, false)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_screen_delete_request("Production", false, true)
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
    async fn screener_screens_create_dry_run_reports_dialog_without_clicking_submit() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "CLI-Test1"
            }),
            json!({
                "menu_opened": true,
                "dialog_opened": true,
                "input_found": true,
                "input_value": "Untitled screen",
                "submit_found": true,
                "dialog_title": "create",
                "screen_title": "CLI-Test1",
                "actions": [
                    { "index": 0, "text": "Create new screen", "kind": "create", "enabled": true }
                ],
                "click_point": { "x": 100.0, "y": 200.0 }
            }),
            json!({ "closed": true }),
        ]);

        let result = screener_screens_create(&mut runtime, "CLI-Test-New", true)
            .await
            .unwrap();

        assert_eq!(result["action"], "screen_create");
        assert_eq!(result["dry_run"], true);
        assert_eq!(result["created"], false);
        assert_eq!(result["name"], "CLI-Test-New");
        assert_eq!(result["target_action"]["kind"], "create");
        assert!(runtime.inserted_text.is_empty());
        assert!(runtime.mouse_events.is_empty());
    }

    #[tokio::test]
    async fn screener_screens_rename_clicks_submit_and_verifies_title() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "CLI-Test1"
            }),
            json!({
                "menu_opened": true,
                "dialog_opened": true,
                "input_found": true,
                "input_value": "CLI-Test1",
                "submit_found": true,
                "dialog_title": "rename",
                "screen_title": "CLI-Test1",
                "actions": [
                    { "index": 0, "text": "Rename", "kind": "rename", "enabled": true }
                ],
                "click_point": { "x": 100.0, "y": 200.0 }
            }),
            json!({
                "dialog_opened": true,
                "submit_found": true,
                "click_point": { "x": 120.0, "y": 220.0 }
            }),
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "CLI-Test2"
            }),
        ]);

        let result = screener_screens_rename(&mut runtime, "CLI-Test1", "CLI-Test2", false)
            .await
            .unwrap();

        assert_eq!(result["action"], "screen_rename");
        assert_eq!(result["dry_run"], false);
        assert_eq!(result["renamed"], true);
        assert_eq!(result["before_screen_title"], "CLI-Test1");
        assert_eq!(result["after_screen_title"], "CLI-Test2");
        assert_eq!(runtime.inserted_text, vec!["CLI-Test2".to_string()]);
        assert_eq!(runtime.mouse_events.len(), 3);
    }

    #[tokio::test]
    async fn screener_screens_delete_dry_run_resolves_storage_target() {
        let mut runtime = FakeRuntime::new([
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "CLI-Test1"
            }),
            json!({
                "storage_available": true,
                "fetch_ok": true,
                "screens": [
                    { "index": 0, "id": "screen-1", "name": "CLI-Test1", "active": true, "owner": true, "shared": false },
                    { "index": 1, "id": "screen-2", "name": "CLI-Test-Delete", "active": false, "owner": true, "shared": false }
                ],
            }),
        ]);

        let result = screener_screens_delete(&mut runtime, "CLI-Test-Delete", true, false)
            .await
            .unwrap();

        assert_eq!(result["action"], "screen_delete");
        assert_eq!(result["scope"], "screen_storage_api");
        assert_eq!(result["dry_run"], true);
        assert_eq!(result["deleted"], false);
        assert_eq!(result["target_screen"]["name"], "CLI-Test-Delete");
        assert_eq!(result["target_screen"]["id"], "screen-2");
        assert!(runtime.mouse_events.is_empty());
    }

    #[tokio::test]
    async fn screener_screens_delete_uses_storage_api_and_post_checks_absence() {
        let mut runtime = FakeRuntime::new([
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "CLI-Test1"
            }),
            json!({
                "storage_available": true,
                "fetch_ok": true,
                "screens": [
                    { "index": 0, "id": "screen-1", "name": "CLI-Test1", "active": true, "owner": true, "shared": false },
                    { "index": 1, "id": "screen-2", "name": "CLI-Test-Delete", "active": false, "owner": true, "shared": false }
                ],
            }),
            json!({
                "deleted": true,
                "status": 204,
                "id": "screen-2"
            }),
            json!({
                "storage_available": true,
                "fetch_ok": true,
                "screens": [
                    { "index": 0, "id": "screen-1", "name": "CLI-Test1", "active": true, "owner": true, "shared": false }
                ],
            }),
        ]);

        let result = screener_screens_delete(&mut runtime, "CLI-Test-Delete", false, true)
            .await
            .unwrap();

        assert_eq!(result["action"], "screen_delete");
        assert_eq!(result["scope"], "screen_storage_api");
        assert_eq!(result["dry_run"], false);
        assert_eq!(result["deleted"], true);
        assert_eq!(result["target_screen"]["name"], "CLI-Test-Delete");
        assert_eq!(result["before_screen_count"], 2);
        assert_eq!(result["after_screen_count"], 1);
        assert!(runtime.mouse_events.is_empty());
    }

    #[tokio::test]
    async fn screener_screens_delete_refuses_active_storage_target() {
        let mut runtime = FakeRuntime::new([
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "CLI-Test-Delete"
            }),
            json!({
                "storage_available": true,
                "fetch_ok": true,
                "screens": [
                    { "index": 0, "id": "screen-2", "name": "CLI-Test-Delete", "active": true, "owner": true, "shared": false }
                ],
            }),
        ]);

        let error = screener_screens_delete(&mut runtime, "CLI-Test-Delete", false, true)
            .await
            .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(runtime.mouse_events.is_empty());
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
    async fn screener_filters_add_dry_run_resolves_candidate_without_mutation() {
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
            json!({ "opened": true, "input_found": true }),
            json!({
                "found": true,
                "candidate": { "text": "RSI (相対力指数)", "normalized_text": "RSI (相対力指数)", "role": "option" },
                "candidate_click_point": { "x": 900.0, "y": 450.0 }
            }),
            json!({ "closed": true }),
        ]);
        let request = validate_screener_filter_add_request("RSI", Some(70.0), None, true).unwrap();

        let result = screener_filters_add(&mut runtime, request).await.unwrap();

        assert_eq!(result["action"], "filter_add");
        assert_eq!(result["dry_run"], true);
        assert_eq!(result["added"], false);
        assert_eq!(result["target_filter"]["text"], "RSI (相対力指数)");
        assert_eq!(result["requested_range"]["min"], 70.0);
        assert_eq!(runtime.inserted_text, vec!["RSI"]);
        assert_eq!(runtime.mouse_events.len(), 0);
    }

    #[tokio::test]
    async fn screener_filters_add_clicks_candidate_range_and_post_checks() {
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
            json!({ "opened": true, "input_found": true }),
            json!({
                "found": true,
                "candidate": { "text": "RSI (相対力指数)", "normalized_text": "RSI (相対力指数)", "role": "option" },
                "candidate_click_point": { "x": 900.0, "y": 450.0 }
            }),
            json!({
                "found": true,
                "range_option": { "text": "> 70 買われすぎ", "normalized_text": "> 70 買われすぎ", "role": "option" },
                "range_click_point": { "x": 900.0, "y": 540.0 }
            }),
            json!({
                "button_found": true,
                "open": true,
                "filters": [
                    { "index": 0, "text": "EMA (21)未満価格 : 0% 〜 10%", "data_name": "screener-filter-pill-ema", "visible": true },
                    { "index": 1, "text": "RSI (14)70", "data_name": "screener-filter-pill-rsi", "visible": true }
                ],
                "filter_count": 2
            }),
        ]);
        let request = validate_screener_filter_add_request("RSI", Some(70.0), None, false).unwrap();

        let result = screener_filters_add(&mut runtime, request).await.unwrap();

        assert_eq!(result["action"], "filter_add");
        assert_eq!(result["dry_run"], false);
        assert_eq!(result["added"], true);
        assert_eq!(
            result["after_filter"]["data_name"],
            "screener-filter-pill-rsi"
        );
        assert_eq!(result["before_filter_count"], 1);
        assert_eq!(result["after_filter_count"], 2);
        assert_eq!(runtime.inserted_text, vec!["RSI"]);
        assert_eq!(runtime.mouse_events.len(), 6);
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
                "click_scheduled": true,
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
    fn ensure_dialog_open_rejects_toolbar_only_state() {
        let value = json!({
            "button_found": true,
            "open": false,
            "panel_root_found": false,
            "dialog_title": null,
            "screen_title": null,
            "filter_count": 0,
            "column_count": 0
        });

        let error = ensure_dialog_open(&value).unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
    }
}
