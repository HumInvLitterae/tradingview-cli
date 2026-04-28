use serde_json::{Value, json};
use tokio::time::{Duration, sleep};

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};

use super::{
    super::common::js_string,
    engine::{
        SCREENER_SOURCE, ScreenerMutationSession, dispatch_screen_menu_click, ensure_dialog_open,
        expanded_expression, read_screener_state, read_screener_with_restore,
        screen_menu_click_point, value_bool,
    },
    validation::validate_screener_screen_name,
};
use crate::domain::screener::screens::{
    ScreenerScreenTarget, resolve_save_screen_action, resolve_screen_action, resolve_screen_target,
    screen_action_payload, screen_actions_from_menu, screen_actions_payload,
    screen_name_dialog_payload, screen_target_payload, screen_targets_from_menu,
    screen_targets_payload,
};

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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::super::test_support::FakeRuntime;
    use super::*;

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
}
