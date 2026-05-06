use serde::Serialize;
use serde_json::Value;
use tradingview_core::{AppError, ErrorKind};

use crate::{
    fundamentals_symbol_with_groups_typed, quote_symbol_typed, symbol_info_typed,
    types::{Snapshot, SnapshotSection, SnapshotSectionError, SnapshotSections},
    validate_fundamentals_selection,
};

const SNAPSHOT_SOURCE: &str = "snapshot_desktop_free";
const DESKTOP_FREE_READ_CATEGORY: &str = "desktop_free_read";

pub async fn snapshot_symbol(
    symbol: &str,
    groups: Vec<String>,
    fields: Vec<String>,
) -> Result<Value, AppError> {
    serde_json::to_value(snapshot_symbol_typed(symbol, groups, fields).await?)
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))
}

/// Reads a Desktop-free one-symbol evidence packet.
///
/// This is the typed Rust API. Use [`snapshot_symbol`] only when preserving the
/// CLI-compatible JSON payload shape is required.
pub async fn snapshot_symbol_typed(
    symbol: &str,
    groups: Vec<String>,
    fields: Vec<String>,
) -> Result<Snapshot, AppError> {
    let requested_symbol = symbol.trim();
    if requested_symbol.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "snapshot symbol must not be empty",
        ));
    }
    validate_fundamentals_selection(groups.clone(), fields.clone())?;

    let quote = section_from_result("quote", quote_symbol_typed(requested_symbol).await);
    let info = section_from_result("info", symbol_info_typed(requested_symbol).await);
    let fundamentals = section_from_result(
        "fundamentals",
        fundamentals_symbol_with_groups_typed(requested_symbol, groups, fields).await,
    );

    let sections = SnapshotSections {
        quote,
        info,
        fundamentals,
    };
    let errors = section_errors(&sections);
    let success_count = [&sections.quote, &sections.info, &sections.fundamentals]
        .iter()
        .filter(|section| section.ok)
        .count();
    let snapshot = Snapshot {
        source: SNAPSHOT_SOURCE.to_string(),
        source_category: DESKTOP_FREE_READ_CATEGORY.to_string(),
        requires_desktop: false,
        non_mutating: true,
        requested_symbol: requested_symbol.to_string(),
        symbol: best_symbol(&sections),
        observed_symbol: best_observed_symbol(&sections),
        sections,
        errors,
        next_action_hints: vec![
            "Use tv observe chart for selected-chart time-window evidence.".to_string(),
            "Use tv screenshot only when structured reads are insufficient.".to_string(),
        ],
    };

    if success_count > 0 {
        Ok(snapshot)
    } else {
        Err(AppError::new(
            first_error_kind(&snapshot),
            "TradingView snapshot did not resolve any evidence sections",
        )
        .with_details(
            serde_json::to_value(snapshot)
                .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))?,
        ))
    }
}

fn section_from_result<T>(section: &str, result: Result<T, AppError>) -> SnapshotSection
where
    T: Serialize,
{
    match result {
        Ok(data) => match serde_json::to_value(data) {
            Ok(value) => SnapshotSection {
                ok: true,
                data: Some(value),
                error: None,
            },
            Err(error) => SnapshotSection {
                ok: false,
                data: None,
                error: Some(SnapshotSectionError {
                    section: section.to_string(),
                    kind: ErrorKind::Internal,
                    message: error.to_string(),
                    details: None,
                }),
            },
        },
        Err(error) => SnapshotSection {
            ok: false,
            data: None,
            error: Some(SnapshotSectionError {
                section: section.to_string(),
                kind: error.kind,
                message: error.message,
                details: error.details,
            }),
        },
    }
}

fn section_errors(sections: &SnapshotSections) -> Vec<SnapshotSectionError> {
    [&sections.quote, &sections.info, &sections.fundamentals]
        .iter()
        .filter_map(|section| section.error.clone())
        .collect()
}

fn first_error_kind(snapshot: &Snapshot) -> ErrorKind {
    snapshot
        .errors
        .first()
        .map(|error| error.kind)
        .unwrap_or(ErrorKind::Validation)
}

fn best_symbol(sections: &SnapshotSections) -> Value {
    section_value(&sections.quote, "symbol")
        .or_else(|| section_value(&sections.fundamentals, "symbol"))
        .or_else(|| section_value(&sections.info, "full_name"))
        .unwrap_or(Value::Null)
}

fn best_observed_symbol(sections: &SnapshotSections) -> Value {
    section_value(&sections.quote, "observed_symbol")
        .or_else(|| section_value(&sections.fundamentals, "observed_symbol"))
        .or_else(|| section_value(&sections.info, "symbol"))
        .unwrap_or(Value::Null)
}

fn section_value(section: &SnapshotSection, key: &str) -> Option<Value> {
    section
        .data
        .as_ref()
        .and_then(|data| data.get(key))
        .cloned()
        .filter(|value| !value.is_null())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn snapshot_preserves_partial_errors_and_best_symbols() {
        let sections = SnapshotSections {
            quote: SnapshotSection {
                ok: true,
                data: Some(json!({
                    "symbol": "NASDAQ:AAPL",
                    "observed_symbol": "AAPL"
                })),
                error: None,
            },
            info: SnapshotSection {
                ok: true,
                data: Some(json!({
                    "full_name": "NASDAQ:AAPL",
                    "symbol": "AAPL"
                })),
                error: None,
            },
            fundamentals: SnapshotSection {
                ok: false,
                data: None,
                error: Some(SnapshotSectionError {
                    section: "fundamentals".to_string(),
                    kind: ErrorKind::InternalApiUnavailable,
                    message: "temporary failure".to_string(),
                    details: Some(json!({"phase": "scanner"})),
                }),
            },
        };

        assert_eq!(best_symbol(&sections), json!("NASDAQ:AAPL"));
        assert_eq!(best_observed_symbol(&sections), json!("AAPL"));
        assert_eq!(section_errors(&sections).len(), 1);
    }

    #[test]
    fn snapshot_falls_back_to_info_symbol_identity() {
        let sections = SnapshotSections {
            quote: SnapshotSection {
                ok: false,
                data: None,
                error: Some(SnapshotSectionError {
                    section: "quote".to_string(),
                    kind: ErrorKind::Connection,
                    message: "offline".to_string(),
                    details: None,
                }),
            },
            info: SnapshotSection {
                ok: true,
                data: Some(json!({
                    "full_name": "NYSE:IONQ",
                    "symbol": "IONQ"
                })),
                error: None,
            },
            fundamentals: SnapshotSection {
                ok: false,
                data: None,
                error: None,
            },
        };

        assert_eq!(best_symbol(&sections), json!("NYSE:IONQ"));
        assert_eq!(best_observed_symbol(&sections), json!("IONQ"));
    }
}
