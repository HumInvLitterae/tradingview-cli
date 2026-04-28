use serde_json::{Value, json};

use tradingview_core::{AppError, ErrorKind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScreenerScreenTarget {
    pub index: usize,
    pub id: Option<String>,
    pub name: String,
    pub active: bool,
    pub owner: Option<bool>,
    pub shared: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScreenerScreenAction {
    pub index: usize,
    pub text: String,
    pub kind: String,
    pub enabled: bool,
}

pub fn screen_targets_from_menu(value: &Value) -> Vec<ScreenerScreenTarget> {
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

pub fn resolve_screen_target(
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

pub fn screen_target_payload(screen: &ScreenerScreenTarget) -> Value {
    json!({
        "index": screen.index,
        "id": screen.id,
        "name": screen.name,
        "active": screen.active,
        "owner": screen.owner,
        "shared": screen.shared,
    })
}

pub fn screen_targets_payload(screens: &[ScreenerScreenTarget]) -> Vec<Value> {
    screens.iter().map(screen_target_payload).collect()
}

pub fn screen_actions_from_menu(value: &Value) -> Vec<ScreenerScreenAction> {
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

pub fn resolve_save_screen_action(
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

pub fn resolve_screen_action(
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

pub fn screen_name_dialog_payload(value: &Value) -> Value {
    json!({
        "dialog_opened": value_bool(value, "dialog_opened"),
        "input_found": value_bool(value, "input_found"),
        "submit_found": value_bool(value, "submit_found"),
        "initial_value": value.get("input_value").cloned().unwrap_or(Value::Null),
        "dialog_title": value.get("dialog_title").cloned().unwrap_or(Value::Null),
    })
}

pub fn screen_action_payload(action: &ScreenerScreenAction) -> Value {
    json!({
        "index": action.index,
        "text": action.text,
        "kind": action.kind,
        "enabled": action.enabled,
    })
}

pub fn screen_actions_payload(actions: &[ScreenerScreenAction]) -> Vec<Value> {
    actions.iter().map(screen_action_payload).collect()
}

fn value_bool(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}
