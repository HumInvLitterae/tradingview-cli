use serde::Serialize;
use serde_json::Value;
use tradingview_core::{AppError, ErrorKind};

use crate::{
    fundamentals_symbol_typed, quote_symbol_typed, symbol_info_typed,
    types::{
        Compare, CompareItem, CompareItemError, CompareMissingSummary, SnapshotSection,
        SnapshotSectionError, SnapshotSections,
    },
};

const COMPARE_SOURCE: &str = "compare_desktop_free";
const DESKTOP_FREE_READ_CATEGORY: &str = "desktop_free_read";

pub async fn compare_symbols(symbols: Vec<String>) -> Result<Value, AppError> {
    serde_json::to_value(compare_symbols_typed(symbols).await?)
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))
}

/// Reads a Desktop-free comparison packet for several symbols.
///
/// This is the typed Rust API. Use [`compare_symbols`] only when preserving the
/// CLI-compatible JSON payload shape is required.
pub async fn compare_symbols_typed(symbols: Vec<String>) -> Result<Compare, AppError> {
    let requested_symbols = normalize_compare_symbols(symbols)?;
    let requested_count = requested_symbols.len();
    let mut items = Vec::with_capacity(requested_count);

    for requested_symbol in requested_symbols {
        items.push(compare_one_symbol(requested_symbol).await);
    }

    finalize_compare_items(requested_count, items)
}

async fn compare_one_symbol(requested_symbol: String) -> CompareItem {
    let quote = section_from_result("quote", quote_symbol_typed(&requested_symbol).await);
    let info = section_from_result("info", symbol_info_typed(&requested_symbol).await);
    let fundamentals = section_from_result(
        "fundamentals",
        fundamentals_symbol_typed(&requested_symbol, Vec::new()).await,
    );

    let sections = SnapshotSections {
        quote,
        info,
        fundamentals,
    };
    let errors = section_errors(&sections);
    let ok = [&sections.quote, &sections.info, &sections.fundamentals]
        .iter()
        .any(|section| section.ok);
    let missing_summary = missing_summary(&sections);

    CompareItem {
        requested_symbol,
        symbol: best_symbol(&sections),
        observed_symbol: best_observed_symbol(&sections),
        ok,
        sections,
        errors,
        missing_summary,
    }
}

fn finalize_compare_items(
    requested_count: usize,
    items: Vec<CompareItem>,
) -> Result<Compare, AppError> {
    let resolved_count = items.iter().filter(|item| item.ok).count();
    let error_count = requested_count.saturating_sub(resolved_count);
    let errors = compare_errors(&items);
    let compare = Compare {
        source: COMPARE_SOURCE.to_string(),
        source_category: DESKTOP_FREE_READ_CATEGORY.to_string(),
        requires_desktop: false,
        non_mutating: true,
        requested_count,
        resolved_count,
        error_count,
        items,
        errors,
        next_action_hints: vec![
            "Use tv snapshot <SYMBOL> for one-symbol detail after narrowing candidates."
                .to_string(),
            "Use tv observe chart for selected-chart time-window evidence.".to_string(),
            "Use tv quote <SYMBOL> --source chart only for explicit single-symbol chart-feed follow-up."
                .to_string(),
        ],
    };

    if resolved_count > 0 {
        Ok(compare)
    } else {
        Err(AppError::new(
            first_error_kind(&compare),
            "TradingView compare did not resolve any evidence sections",
        )
        .with_details(
            serde_json::to_value(compare)
                .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))?,
        ))
    }
}

fn normalize_compare_symbols(symbols: Vec<String>) -> Result<Vec<String>, AppError> {
    if symbols.len() < 2 {
        return Err(AppError::new(
            ErrorKind::Validation,
            "compare requires at least two symbols",
        ));
    }

    let mut normalized = Vec::with_capacity(symbols.len());
    for symbol in symbols {
        let symbol = symbol.trim();
        if symbol.is_empty() {
            return Err(AppError::new(
                ErrorKind::Validation,
                "compare symbol must not be empty",
            ));
        }
        normalized.push(symbol.to_string());
    }

    Ok(normalized)
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

fn compare_errors(items: &[CompareItem]) -> Vec<CompareItemError> {
    items
        .iter()
        .flat_map(|item| {
            item.errors.iter().map(|error| CompareItemError {
                requested_symbol: item.requested_symbol.clone(),
                section: error.section.clone(),
                kind: error.kind,
                message: error.message.clone(),
                details: error.details.clone(),
            })
        })
        .collect()
}

fn first_error_kind(compare: &Compare) -> ErrorKind {
    compare
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

fn missing_summary(sections: &SnapshotSections) -> CompareMissingSummary {
    let fundamentals = section_string_array(&sections.fundamentals, "missing_fields");
    CompareMissingSummary {
        quote: Vec::new(),
        info: Vec::new(),
        total_count: fundamentals.len(),
        fundamentals,
    }
}

fn section_string_array(section: &SnapshotSection, key: &str) -> Vec<String> {
    section
        .data
        .as_ref()
        .and_then(|data| data.get(key))
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn compare_requires_at_least_two_non_empty_symbols() {
        assert_eq!(
            normalize_compare_symbols(vec!["AAPL".to_string()])
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            normalize_compare_symbols(vec!["AAPL".to_string(), " ".to_string()])
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            normalize_compare_symbols(vec![" AAPL ".to_string(), "NYSE:IONQ".to_string()]).unwrap(),
            vec!["AAPL".to_string(), "NYSE:IONQ".to_string()]
        );
    }

    #[test]
    fn compare_item_preserves_best_symbols_errors_and_missing_summary() {
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
                ok: true,
                data: Some(json!({
                    "symbol": "NASDAQ:AAPL",
                    "observed_symbol": "AAPL",
                    "missing_fields": ["dividends_yield_current"]
                })),
                error: None,
            },
        };

        assert_eq!(best_symbol(&sections), json!("NASDAQ:AAPL"));
        assert_eq!(best_observed_symbol(&sections), json!("AAPL"));
        assert_eq!(
            missing_summary(&sections).fundamentals,
            vec!["dividends_yield_current".to_string()]
        );
        assert_eq!(missing_summary(&sections).total_count, 1);
    }

    #[test]
    fn compare_errors_include_requested_symbol_and_section() {
        let item = CompareItem {
            requested_symbol: "AAPL".to_string(),
            symbol: Value::Null,
            observed_symbol: Value::Null,
            ok: false,
            sections: SnapshotSections {
                quote: SnapshotSection {
                    ok: false,
                    data: None,
                    error: Some(SnapshotSectionError {
                        section: "quote".to_string(),
                        kind: ErrorKind::InternalApiUnavailable,
                        message: "temporary failure".to_string(),
                        details: Some(json!({"phase": "scanner"})),
                    }),
                },
                info: SnapshotSection {
                    ok: false,
                    data: None,
                    error: None,
                },
                fundamentals: SnapshotSection {
                    ok: false,
                    data: None,
                    error: None,
                },
            },
            errors: vec![SnapshotSectionError {
                section: "quote".to_string(),
                kind: ErrorKind::InternalApiUnavailable,
                message: "temporary failure".to_string(),
                details: Some(json!({"phase": "scanner"})),
            }],
            missing_summary: CompareMissingSummary {
                quote: Vec::new(),
                info: Vec::new(),
                fundamentals: Vec::new(),
                total_count: 0,
            },
        };

        let errors = compare_errors(&[item]);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].requested_symbol, "AAPL");
        assert_eq!(errors[0].section, "quote");
    }
}
