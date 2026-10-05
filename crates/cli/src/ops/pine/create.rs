use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};

use super::pine_sources_match;
use crate::ops::common::js_string;

const CREATE_SCRIPT: &str = include_str!("create/create.js");

#[derive(Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
enum CreateResult {
    NotSaved {
        reason: PreflightFailure,
    },
    Saved {
        script: SavedScript,
        saved_source: String,
        #[serde(default)]
        compilation: Value,
    },
    Unknown {
        reason: UnknownSave,
    },
}

#[derive(Deserialize, Serialize)]
struct SavedScript {
    id: String,
    name: String,
    version: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum PreflightFailure {
    SaveApiUnavailable,
    CatalogUnavailable,
    NameConflict,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum UnknownSave {
    SaveRequestFailed,
    SaveIdentityUnavailable,
    SavedIdentityMismatch,
    CatalogReadbackFailed,
    SourceReadbackFailed,
    SourceMismatch,
    InvalidResult,
    EvaluationFailed,
}

#[derive(Deserialize, Serialize)]
struct Compilation {
    compiled: bool,
    errors: Vec<Diagnostic>,
    warnings: Vec<Diagnostic>,
}

#[derive(Deserialize, Serialize)]
struct Diagnostic {
    message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    start: Option<Position>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    end: Option<Position>,
}

#[derive(Deserialize, Serialize)]
struct Position {
    line: u64,
    column: u64,
}

pub fn validate_pine_create_name(name: &str) -> Result<&str, AppError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Pine script name must not be empty",
        ));
    }
    Ok(name)
}

pub async fn pine_create(
    runtime: &mut impl RuntimeEvaluator,
    name: &str,
    source: &str,
) -> Result<Value, AppError> {
    let name = validate_pine_create_name(name)?;
    if source.trim().is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Pine source must not be empty",
        ));
    }
    let expression = format!(
        "(async function() {{\n{CREATE_SCRIPT}\nreturn createSavedScript({}, {});\n}})()",
        js_string(name)?,
        js_string(source)?,
    );
    let raw = runtime.evaluate(&expression, true).await.map_err(|error| {
        let mut failure = unknown_save(name, UnknownSave::EvaluationFailed);
        failure.kind = error.kind;
        if let Some(stage) = error.details.as_ref().and_then(|d| d.get("failure_stage"))
            && stage.is_string()
        {
            failure.details.as_mut().unwrap()["failure_stage"] = stage.clone();
        }
        failure
    })?;
    normalize_result(raw, name, source)
}

fn normalize_result(raw: Value, name: &str, source: &str) -> Result<Value, AppError> {
    let result =
        serde_json::from_value(raw).map_err(|_| unknown_save(name, UnknownSave::InvalidResult))?;
    match result {
        CreateResult::NotSaved { reason } => {
            let kind = if matches!(reason, PreflightFailure::NameConflict) {
                ErrorKind::Validation
            } else {
                ErrorKind::InternalApiUnavailable
            };
            Err(
                AppError::new(kind, "Pine script creation stopped before saving").with_details(
                    json!({
                        "operation": "pine_create", "name": name,
                        "saved": false, "phase": "preflight", "reason": reason,
                        "source": "internal_api"
                    }),
                ),
            )
        }
        CreateResult::Unknown { reason } => Err(unknown_save(name, reason)),
        CreateResult::Saved {
            script,
            saved_source,
            compilation,
        } => {
            if script.id.trim().is_empty()
                || script.version.trim().is_empty()
                || script.name != name
            {
                return Err(unknown_save(name, UnknownSave::InvalidResult));
            }
            if !pine_sources_match(source, &saved_source) {
                return Err(unknown_save(name, UnknownSave::SourceMismatch));
            }
            Ok(json!({
                "operation": "pine_create", "saved": true,
                "script": script, "source_verified": true,
                "compilation": compilation_evidence(compilation),
                "source": "internal_api", "source_category": "desktop_backed_operation",
                "requires_desktop": true, "non_mutating": false
            }))
        }
    }
}

fn compilation_evidence(raw: Value) -> Value {
    if let Ok(evidence) = serde_json::from_value::<Compilation>(raw)
        && evidence.compiled == evidence.errors.is_empty()
    {
        return json!({
            "compiled": evidence.compiled,
            "error_count": evidence.errors.len(), "warning_count": evidence.warnings.len(),
            "errors": evidence.errors, "warnings": evidence.warnings
        });
    }
    json!({
        "compiled": null, "error_count": null, "warning_count": null,
        "errors": null, "warnings": null
    })
}

fn unknown_save(name: &str, reason: UnknownSave) -> AppError {
    AppError::new(
        ErrorKind::InternalApiUnavailable,
        "Pine script save outcome is unknown; inspect saved scripts before retrying",
    )
    .with_details(json!({
        "operation": "pine_create", "name": name, "saved": null,
        "save_outcome": "unknown", "reason": reason, "source": "internal_api",
        "next_action_hint": "Inspect saved scripts for the requested name; do not automatically retry."
    }))
}

#[cfg(test)]
mod tests;
