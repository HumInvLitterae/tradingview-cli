//! Offline output descriptions, not runtime data admission or market evidence.

use serde_json::{Value, json};
use tradingview_core::{AppError, ErrorKind};

use crate::build_info;

const CONTRACT: &str = "cli_schema.v1";

pub(super) fn describe(path: &[String]) -> Result<Value, AppError> {
    let (_, canonical) =
        super::command_path::resolve(path).ok_or_else(|| failure("unknown_command"))?;
    let mut data = json!({
        "contract_version": CONTRACT,
        "binary_version": build_info::VERSION,
        "command": canonical
    });
    if path.is_empty() {
        data["commands"] = json!([["values"], ["mcp", "bars"], ["ohlcv"]]);
        return Ok(data);
    }
    let path = canonical.iter().map(String::as_str).collect::<Vec<_>>();
    let (document, command, unchecked) = match path.as_slice() {
        ["values"] => (
            include_str!("schema/values.json"),
            "values",
            vec!["dynamic_study_values", "runtime_semantics", "error_details"],
        ),
        ["ohlcv"] => (
            include_str!("schema/ohlcv.json"),
            "ohlcv",
            vec![
                "cross_field_invariants",
                "runtime_semantics",
                "error_details",
            ],
        ),
        ["mcp", "bars"] => (
            include_str!("schema/mcp_bars.json"),
            "mcp",
            vec![
                "cross_field_invariants",
                "runtime_semantics",
                "error_details",
            ],
        ),
        _ => return Err(failure("unsupported_command")),
    };
    let payload: Value =
        serde_json::from_str(document).expect("embedded payload schema is valid JSON");
    let error: Value = serde_json::from_str(include_str!("schema/error.json"))
        .expect("embedded error schema is valid JSON");
    data["coverage"] = json!("documented_fields");
    data["unchecked"] = json!(unchecked);
    data["schema"] = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$defs": {"payload": payload, "error": error},
        "oneOf": [
            {
                "type": "object", "required": ["success", "command", "data"],
                "properties": {
                    "success": {"const": true}, "command": {"const": command},
                    "data": {"$ref": "#/$defs/payload"}
                }
            },
            {
                "type": "object", "required": ["success", "command", "error"],
                "properties": {
                    "success": {"const": false}, "command": {"enum": [command, "tv"]},
                    "error": {"$ref": "#/$defs/error"}
                }
            }
        ]
    });
    Ok(data)
}

pub(super) fn failure(code: &str) -> AppError {
    AppError::new(ErrorKind::Validation, "Output schema unavailable")
        .with_details(json!({"contract_version": CONTRACT, "code": code}))
}

#[cfg(test)]
mod tests;
