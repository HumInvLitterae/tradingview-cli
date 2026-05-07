use serde::Serialize;
use serde_json::Value;
use tradingview_core::{AppError, ErrorKind};

use crate::{
    fundamentals::is_fundamental_field_in_group,
    fundamentals_symbol_typed, quote_symbol_typed, symbol_info_typed,
    types::{
        Compare, CompareFieldCoverage, CompareFollowUpHint, CompareItem, CompareItemError,
        CompareMissingSummary, SnapshotSection, SnapshotSectionError, SnapshotSections,
    },
};

const COMPARE_CONTRACT_VERSION: &str = "compare.v1";
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

    for (requested_index, requested_symbol) in requested_symbols.into_iter().enumerate() {
        items.push(compare_one_symbol(requested_index, requested_symbol).await);
    }

    finalize_compare_items(requested_count, items)
}

async fn compare_one_symbol(requested_index: usize, requested_symbol: String) -> CompareItem {
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
    let symbol = best_symbol(&sections);
    let observed_symbol = best_observed_symbol(&sections);
    let follow_up_hints = follow_up_hints(&symbol);

    CompareItem {
        requested_index,
        requested_symbol,
        symbol,
        observed_symbol,
        ok,
        sections,
        errors,
        missing_summary,
        follow_up_hints,
    }
}

fn finalize_compare_items(
    requested_count: usize,
    items: Vec<CompareItem>,
) -> Result<Compare, AppError> {
    let resolved_count = items.iter().filter(|item| item.ok).count();
    let error_count = requested_count.saturating_sub(resolved_count);
    let errors = compare_errors(&items);
    let summary = compare_summary(requested_count, resolved_count, error_count, &items);
    let compare = Compare {
        contract_version: COMPARE_CONTRACT_VERSION.to_string(),
        source: COMPARE_SOURCE.to_string(),
        source_category: DESKTOP_FREE_READ_CATEGORY.to_string(),
        requires_desktop: false,
        non_mutating: true,
        requested_count,
        resolved_count,
        error_count,
        summary,
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

fn compare_summary(
    requested_count: usize,
    resolved_count: usize,
    error_count: usize,
    items: &[CompareItem],
) -> crate::types::CompareSummary {
    let quote_ok_count = items.iter().filter(|item| item.sections.quote.ok).count();
    let info_ok_count = items.iter().filter(|item| item.sections.info.ok).count();
    let fundamentals_ok_count = items
        .iter()
        .filter(|item| item.sections.fundamentals.ok)
        .count();
    let missing_total_count = items
        .iter()
        .map(|item| item.missing_summary.total_count)
        .sum();
    let field_coverage =
        field_coverage(quote_ok_count, info_ok_count, fundamentals_ok_count, items);
    let resolved_symbols = items
        .iter()
        .map(|item| crate::types::CompareResolvedSymbol {
            requested_index: item.requested_index,
            requested_symbol: item.requested_symbol.clone(),
            ok: item.ok,
            symbol: item.symbol.clone(),
            observed_symbol: item.observed_symbol.clone(),
            quote_ok: item.sections.quote.ok,
            info_ok: item.sections.info.ok,
            fundamentals_ok: item.sections.fundamentals.ok,
            missing_total_count: item.missing_summary.total_count,
        })
        .collect();

    crate::types::CompareSummary {
        requested_count,
        resolved_count,
        error_count,
        quote_ok_count,
        info_ok_count,
        fundamentals_ok_count,
        missing_total_count,
        field_coverage,
        resolved_symbols,
    }
}

fn field_coverage(
    quote_ok_count: usize,
    info_ok_count: usize,
    fundamentals_ok_count: usize,
    items: &[CompareItem],
) -> CompareFieldCoverage {
    let quote_missing_count = items
        .iter()
        .map(|item| item.missing_summary.quote.len())
        .sum();
    let info_missing_count = items
        .iter()
        .map(|item| item.missing_summary.info.len())
        .sum();
    let fundamentals_missing_count = items
        .iter()
        .map(|item| item.missing_summary.fundamentals.len())
        .sum();
    let earnings_missing_count = items
        .iter()
        .flat_map(|item| item.missing_summary.fundamentals.iter())
        .filter(|field| is_fundamental_field_in_group(field, "earnings"))
        .count();
    let dividends_missing_count = items
        .iter()
        .flat_map(|item| item.missing_summary.fundamentals.iter())
        .filter(|field| is_fundamental_field_in_group(field, "dividends"))
        .count();
    let total_missing_count = items
        .iter()
        .map(|item| item.missing_summary.total_count)
        .sum();

    CompareFieldCoverage {
        quote_ok_count,
        quote_missing_count,
        info_ok_count,
        info_missing_count,
        fundamentals_ok_count,
        fundamentals_missing_count,
        earnings_missing_count,
        dividends_missing_count,
        total_missing_count,
    }
}

fn follow_up_hints(symbol: &Value) -> Vec<CompareFollowUpHint> {
    let command_symbol = symbol.as_str().unwrap_or("<SYMBOL>");
    vec![
        CompareFollowUpHint {
            kind: "snapshot".to_string(),
            command: format!("tv snapshot {command_symbol}"),
            reason: "one_symbol_detail".to_string(),
        },
        CompareFollowUpHint {
            kind: "observe_chart".to_string(),
            command: "tv observe chart --duration-ms <MS>".to_string(),
            reason: "selected_chart_observation".to_string(),
        },
        CompareFollowUpHint {
            kind: "chart_quote".to_string(),
            command: format!("tv quote {command_symbol} --source chart"),
            reason: "single_symbol_chart_quote".to_string(),
        },
        CompareFollowUpHint {
            kind: "screenshot".to_string(),
            command: "tv screenshot --region chart --output <PATH>".to_string(),
            reason: "visual_evidence".to_string(),
        },
    ]
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
    fn compare_summary_preserves_counts_and_symbol_order() {
        let first = CompareItem {
            requested_index: 0,
            requested_symbol: "AAPL".to_string(),
            symbol: json!("NASDAQ:AAPL"),
            observed_symbol: json!("AAPL"),
            ok: true,
            sections: SnapshotSections {
                quote: SnapshotSection {
                    ok: true,
                    data: Some(json!({"symbol": "NASDAQ:AAPL"})),
                    error: None,
                },
                info: SnapshotSection {
                    ok: true,
                    data: Some(json!({"symbol": "AAPL"})),
                    error: None,
                },
                fundamentals: SnapshotSection {
                    ok: true,
                    data: Some(json!({"symbol": "NASDAQ:AAPL"})),
                    error: None,
                },
            },
            errors: Vec::new(),
            missing_summary: CompareMissingSummary {
                quote: Vec::new(),
                info: Vec::new(),
                fundamentals: vec![
                    "next_dividend_date".to_string(),
                    "earnings_release_next_date".to_string(),
                ],
                total_count: 2,
            },
            follow_up_hints: follow_up_hints(&json!("NASDAQ:AAPL")),
        };
        let second = CompareItem {
            requested_index: 1,
            requested_symbol: "NYSE:IONQ".to_string(),
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
                        details: None,
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
            errors: Vec::new(),
            missing_summary: CompareMissingSummary {
                quote: Vec::new(),
                info: Vec::new(),
                fundamentals: Vec::new(),
                total_count: 0,
            },
            follow_up_hints: follow_up_hints(&Value::Null),
        };

        let summary = compare_summary(2, 1, 1, &[first, second]);
        assert_eq!(summary.requested_count, 2);
        assert_eq!(summary.resolved_count, 1);
        assert_eq!(summary.error_count, 1);
        assert_eq!(summary.quote_ok_count, 1);
        assert_eq!(summary.info_ok_count, 1);
        assert_eq!(summary.fundamentals_ok_count, 1);
        assert_eq!(summary.missing_total_count, 2);
        assert_eq!(summary.field_coverage.quote_ok_count, 1);
        assert_eq!(summary.field_coverage.quote_missing_count, 0);
        assert_eq!(summary.field_coverage.info_ok_count, 1);
        assert_eq!(summary.field_coverage.info_missing_count, 0);
        assert_eq!(summary.field_coverage.fundamentals_ok_count, 1);
        assert_eq!(summary.field_coverage.fundamentals_missing_count, 2);
        assert_eq!(summary.field_coverage.earnings_missing_count, 1);
        assert_eq!(summary.field_coverage.dividends_missing_count, 1);
        assert_eq!(summary.field_coverage.total_missing_count, 2);
        assert_eq!(summary.resolved_symbols.len(), 2);
        assert_eq!(summary.resolved_symbols[0].requested_index, 0);
        assert_eq!(summary.resolved_symbols[0].requested_symbol, "AAPL");
        assert_eq!(summary.resolved_symbols[0].symbol, json!("NASDAQ:AAPL"));
        assert_eq!(summary.resolved_symbols[0].observed_symbol, json!("AAPL"));
        assert!(summary.resolved_symbols[0].ok);
        assert_eq!(summary.resolved_symbols[0].missing_total_count, 2);
        assert_eq!(summary.resolved_symbols[1].requested_index, 1);
        assert_eq!(summary.resolved_symbols[1].requested_symbol, "NYSE:IONQ");
        assert!(!summary.resolved_symbols[1].ok);
    }

    #[test]
    fn follow_up_hints_are_machine_readable_without_recommendation() {
        let hints = follow_up_hints(&json!("NASDAQ:AAPL"));
        assert_eq!(hints.len(), 4);
        assert_eq!(hints[0].kind, "snapshot");
        assert_eq!(hints[0].command, "tv snapshot NASDAQ:AAPL");
        assert_eq!(hints[0].reason, "one_symbol_detail");
        assert!(hints.iter().any(|hint| hint.kind == "observe_chart"));
        assert!(hints.iter().any(|hint| hint.kind == "chart_quote"));
        assert!(hints.iter().any(|hint| hint.kind == "screenshot"));
    }

    #[test]
    fn compare_errors_include_requested_symbol_and_section() {
        let item = CompareItem {
            requested_index: 0,
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
            follow_up_hints: follow_up_hints(&Value::Null),
        };

        let errors = compare_errors(&[item]);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].requested_symbol, "AAPL");
        assert_eq!(errors[0].section, "quote");
    }
}
