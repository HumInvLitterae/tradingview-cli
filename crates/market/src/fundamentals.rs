use serde_json::{Value, json};
use tradingview_core::{AppError, ErrorKind};

use crate::{info::preferred_symbol_candidates, search::symbol_search, types::Fundamentals};

mod client;
mod fields;
mod normalize;

use client::fundamentals_symbol_via_scanner;
use fields::normalize_fundamental_selection;
use normalize::normalize_fundamentals_response_typed;

pub async fn fundamentals_symbol(symbol: &str, fields: Vec<String>) -> Result<Value, AppError> {
    fundamentals_symbol_with_groups(symbol, Vec::new(), fields).await
}

pub async fn fundamentals_symbol_with_groups(
    symbol: &str,
    groups: Vec<String>,
    fields: Vec<String>,
) -> Result<Value, AppError> {
    serde_json::to_value(fundamentals_symbol_with_groups_typed(symbol, groups, fields).await?)
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))
}

/// Reads scanner-backed fundamental fields without connecting to TradingView Desktop.
///
/// This is the typed Rust API. Use [`fundamentals_symbol`] only when preserving
/// the CLI-compatible JSON payload shape is required.
pub async fn fundamentals_symbol_typed(
    symbol: &str,
    fields: Vec<String>,
) -> Result<Fundamentals, AppError> {
    fundamentals_symbol_with_groups_typed(symbol, Vec::new(), fields).await
}

/// Reads scanner-backed fundamental fields with optional field groups.
///
/// Groups are convenience bundles around supported scanner fields. They do not
/// change the data source and do not infer meanings beyond TradingView's raw
/// scanner values.
pub async fn fundamentals_symbol_with_groups_typed(
    symbol: &str,
    groups: Vec<String>,
    fields: Vec<String>,
) -> Result<Fundamentals, AppError> {
    let requested_symbol = symbol.trim();
    if requested_symbol.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "fundamentals symbol must not be empty",
        ));
    }
    let selection = normalize_fundamental_selection(groups, fields)?;
    let value = fundamentals_symbol_via_scanner(requested_symbol, &selection.fields).await?;
    match normalize_fundamentals_response_typed(
        requested_symbol,
        &selection.fields,
        &selection.groups,
        &value,
    ) {
        Ok(payload) => Ok(payload),
        Err(err) if err.kind == ErrorKind::Validation => {
            Err(add_symbol_search_candidates(err, requested_symbol).await)
        }
        Err(err) => Err(err),
    }
}

/// Validates fundamentals group and field selection without performing network I/O.
pub fn validate_fundamentals_selection(
    groups: Vec<String>,
    fields: Vec<String>,
) -> Result<(), AppError> {
    normalize_fundamental_selection(groups, fields).map(|_| ())
}

async fn add_symbol_search_candidates(mut error: AppError, requested_symbol: &str) -> AppError {
    let Ok(search) = symbol_search(requested_symbol).await else {
        return error;
    };
    let candidates = preferred_symbol_candidates(requested_symbol, &search);
    if let Some(details) = error.details.as_mut().and_then(Value::as_object_mut) {
        details.insert("candidate_count".to_string(), json!(candidates.len()));
        details.insert("candidates".to_string(), Value::Array(candidates));
        details.insert("candidate_source".to_string(), json!("symbol_search_rest"));
    }
    error
}
