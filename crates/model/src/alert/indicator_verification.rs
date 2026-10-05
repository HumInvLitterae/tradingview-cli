use serde_json::{Value, json};
use tradingview_core::{AppError, ErrorKind};

use super::normalize_indicator_alert_create_payload;

/// Select the chart instance used by verified indicator-alert preparation.
#[derive(Debug, Clone, Copy)]
pub enum IndicatorStudySelection<'a> {
    /// Use no study for zero inputs; otherwise require one exact saved-revision match.
    Automatic,
    /// Require this chart entity even when the saved script has no user inputs.
    Entity(&'a str),
}

pub fn normalize_indicator_alert_verified_payload(
    data: Value,
    dry_run: bool,
    selection: IndicatorStudySelection<'_>,
) -> Result<Value, AppError> {
    if data.get("error").and_then(Value::as_str).is_some() {
        return normalize_indicator_alert_create_payload(data);
    }
    let invalid = || {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Indicator alert evaluation did not return a valid verification result",
        )
    };
    let verification = data
        .get("verification")
        .and_then(Value::as_object)
        .filter(|value| value.len() == 4)
        .ok_or_else(invalid)?;
    if verification.get("condition_source").and_then(Value::as_str) != Some("saved_compilation") {
        return Err(invalid());
    }
    let input_count = verification
        .get("input_count")
        .and_then(Value::as_u64)
        .ok_or_else(invalid)?;
    let study = verification.get("study").ok_or_else(invalid)?;
    match verification.get("input_source").and_then(Value::as_str) {
        Some("none") if input_count == 0 => {}
        Some("active_chart_study") if input_count > 0 && study.is_object() => {}
        _ => return Err(invalid()),
    }
    if study.is_null() {
        if !matches!(selection, IndicatorStudySelection::Automatic) || input_count != 0 {
            return Err(invalid());
        }
    } else {
        let study = study
            .as_object()
            .filter(|value| value.len() == 2)
            .ok_or_else(invalid)?;
        let entity_id = study
            .get("entity_id")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(invalid)?;
        let mode = study.get("selection").and_then(Value::as_str);
        match selection {
            IndicatorStudySelection::Automatic if mode == Some("unique") && input_count > 0 => {}
            IndicatorStudySelection::Entity(expected)
                if mode == Some("explicit") && entity_id == expected => {}
            _ => return Err(invalid()),
        }
    }
    let study_matched = !study.is_null();
    let verification = Value::Object(verification.clone());
    let symbol = data
        .get("symbol")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(invalid)?;
    let resolution = data
        .get("resolution")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(invalid)?;
    let mut result = if dry_run {
        if data.get("action").and_then(Value::as_str) != Some("dry_run")
            || data.get("created") != Some(&Value::Bool(false))
        {
            return Err(invalid());
        }
        json!({"symbol": symbol, "resolution": resolution})
    } else {
        if data.get("created") != Some(&Value::Bool(true)) {
            return Err(invalid());
        }
        let metadata = data.get("input_metadata").ok_or_else(invalid)?;
        if metadata.get("input_count").and_then(Value::as_u64) != Some(input_count)
            || metadata.get("study_matched").and_then(Value::as_bool) != Some(study_matched)
        {
            return Err(invalid());
        }
        normalize_indicator_alert_create_payload(data)?
    };
    result["verification"] = verification;
    Ok(result)
}
