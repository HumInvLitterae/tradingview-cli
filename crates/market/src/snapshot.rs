use serde::Serialize;
use serde_json::Value;
use tradingview_core::{AppError, ErrorKind};

use crate::{
    fundamentals_symbol_with_groups_typed, quote_symbol_typed, symbol_info_typed,
    types::{
        Snapshot, SnapshotFieldCoverage, SnapshotFollowUpHint, SnapshotMissingEvidence,
        SnapshotSection, SnapshotSectionError, SnapshotSections, SnapshotSummary,
    },
    validate_fundamentals_selection,
};

const SNAPSHOT_CONTRACT_VERSION: &str = "snapshot.v1";
const SNAPSHOT_SOURCE: &str = "snapshot_desktop_free";
const DESKTOP_FREE_READ_CATEGORY: &str = "desktop_free_read";
const COVERAGE_STATUS_BLOCKED: &str = "blocked";
const COVERAGE_STATUS_COMPLETE: &str = "complete";
const COVERAGE_STATUS_PARTIAL: &str = "partial";
const FOLLOW_UP_CHART_QUOTE: &str = "chart_quote";
const FOLLOW_UP_OBSERVE_CHART: &str = "observe_chart";
const FOLLOW_UP_SCREENSHOT: &str = "screenshot";
const FOLLOW_UP_SNAPSHOT: &str = "snapshot";
const MISSING_REASON_FIELDS: &str = "missing_fields";
const MISSING_REASON_SECTION_ERROR: &str = "section_error";

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
    let summary = snapshot_summary(&sections, errors.len());
    let missing_evidence = missing_evidence(&sections);
    let symbol = best_symbol(&sections);
    let observed_symbol = best_observed_symbol(&sections);
    let follow_up_hints = follow_up_hints(&symbol);
    let snapshot = Snapshot {
        contract_version: SNAPSHOT_CONTRACT_VERSION.to_string(),
        source: SNAPSHOT_SOURCE.to_string(),
        source_category: DESKTOP_FREE_READ_CATEGORY.to_string(),
        requires_desktop: false,
        non_mutating: true,
        requested_symbol: requested_symbol.to_string(),
        symbol,
        observed_symbol,
        summary,
        sections,
        errors,
        missing_evidence,
        follow_up_hints,
        next_action_hints: vec![
            "Use tv quote <SYMBOL> --source chart only for explicit single-symbol chart-feed follow-up."
                .to_string(),
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

fn snapshot_summary(sections: &SnapshotSections, error_count: usize) -> SnapshotSummary {
    let quote_ok = sections.quote.ok;
    let info_ok = sections.info.ok;
    let fundamentals_ok = sections.fundamentals.ok;
    let fundamentals_missing = section_string_array(&sections.fundamentals, "missing_fields");
    let missing_total_count = fundamentals_missing.len();
    let field_coverage = SnapshotFieldCoverage {
        quote_ok,
        quote_missing_count: 0,
        info_ok,
        info_missing_count: 0,
        fundamentals_ok,
        fundamentals_missing_count: fundamentals_missing.len(),
        total_missing_count: missing_total_count,
    };

    SnapshotSummary {
        coverage_status: coverage_status(
            quote_ok,
            info_ok,
            fundamentals_ok,
            error_count,
            missing_total_count,
        )
        .to_string(),
        quote_ok,
        info_ok,
        fundamentals_ok,
        error_count,
        missing_total_count,
        field_coverage,
    }
}

fn coverage_status(
    quote_ok: bool,
    info_ok: bool,
    fundamentals_ok: bool,
    error_count: usize,
    missing_total_count: usize,
) -> &'static str {
    if !quote_ok && !info_ok && !fundamentals_ok {
        return COVERAGE_STATUS_BLOCKED;
    }

    if quote_ok && info_ok && fundamentals_ok && error_count == 0 && missing_total_count == 0 {
        return COVERAGE_STATUS_COMPLETE;
    }

    COVERAGE_STATUS_PARTIAL
}

fn missing_evidence(sections: &SnapshotSections) -> Vec<SnapshotMissingEvidence> {
    let mut evidence = Vec::new();
    push_section_error_missing_evidence(&mut evidence, "quote", &sections.quote);
    push_section_error_missing_evidence(&mut evidence, "info", &sections.info);
    push_section_error_missing_evidence(&mut evidence, "fundamentals", &sections.fundamentals);

    let fundamentals_missing = section_string_array(&sections.fundamentals, "missing_fields");
    if !fundamentals_missing.is_empty() {
        evidence.push(SnapshotMissingEvidence {
            section: "fundamentals".to_string(),
            missing_fields: fundamentals_missing,
            missing_reason: MISSING_REASON_FIELDS.to_string(),
            suggested_follow_up: FOLLOW_UP_SNAPSHOT.to_string(),
            requires_desktop: false,
        });
    }

    evidence
}

fn push_section_error_missing_evidence(
    evidence: &mut Vec<SnapshotMissingEvidence>,
    section_name: &str,
    section: &SnapshotSection,
) {
    if section.error.is_none() {
        return;
    }

    evidence.push(SnapshotMissingEvidence {
        section: section_name.to_string(),
        missing_fields: Vec::new(),
        missing_reason: MISSING_REASON_SECTION_ERROR.to_string(),
        suggested_follow_up: missing_evidence_follow_up(section_name).to_string(),
        requires_desktop: section_name == "quote",
    });
}

fn missing_evidence_follow_up(section_name: &str) -> &'static str {
    if section_name == "quote" {
        FOLLOW_UP_CHART_QUOTE
    } else {
        FOLLOW_UP_SNAPSHOT
    }
}

fn follow_up_hints(symbol: &Value) -> Vec<SnapshotFollowUpHint> {
    let command_symbol = symbol.as_str().unwrap_or("<SYMBOL>");
    vec![
        SnapshotFollowUpHint {
            kind: FOLLOW_UP_CHART_QUOTE.to_string(),
            command: format!("tv quote {command_symbol} --source chart"),
            reason: "single_symbol_chart_quote".to_string(),
            requires_desktop: true,
        },
        SnapshotFollowUpHint {
            kind: FOLLOW_UP_OBSERVE_CHART.to_string(),
            command: "tv observe chart --duration-ms <MS>".to_string(),
            reason: "selected_chart_observation".to_string(),
            requires_desktop: true,
        },
        SnapshotFollowUpHint {
            kind: FOLLOW_UP_SCREENSHOT.to_string(),
            command: "tv screenshot --region chart --output <PATH>".to_string(),
            reason: "visual_evidence".to_string(),
            requires_desktop: true,
        },
    ]
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
        let summary = snapshot_summary(&sections, 1);
        assert_eq!(summary.coverage_status, "partial");
        assert!(summary.quote_ok);
        assert!(summary.info_ok);
        assert!(!summary.fundamentals_ok);
        assert_eq!(summary.error_count, 1);
        assert_eq!(summary.missing_total_count, 0);
        let evidence = missing_evidence(&sections);
        assert_eq!(evidence.len(), 1);
        assert_eq!(evidence[0].section, "fundamentals");
        assert_eq!(evidence[0].missing_reason, "section_error");
        assert_eq!(evidence[0].suggested_follow_up, "snapshot");
        assert!(!evidence[0].requires_desktop);
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

    #[test]
    fn snapshot_summary_reports_complete_partial_and_blocked() {
        assert_eq!(coverage_status(true, true, true, 0, 0), "complete");
        assert_eq!(coverage_status(true, true, true, 0, 1), "partial");
        assert_eq!(coverage_status(true, true, false, 1, 0), "partial");
        assert_eq!(coverage_status(false, false, false, 3, 0), "blocked");
    }

    #[test]
    fn snapshot_missing_evidence_reports_missing_fields_and_empty_state() {
        let missing_sections = SnapshotSections {
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
                data: Some(json!({
                    "symbol": "NASDAQ:AAPL",
                    "missing_fields": ["next_dividend_date"]
                })),
                error: None,
            },
        };
        let evidence = missing_evidence(&missing_sections);
        assert_eq!(evidence.len(), 1);
        assert_eq!(evidence[0].section, "fundamentals");
        assert_eq!(evidence[0].missing_reason, "missing_fields");
        assert_eq!(evidence[0].missing_fields, vec!["next_dividend_date"]);
        assert_eq!(evidence[0].suggested_follow_up, "snapshot");
        assert!(!evidence[0].requires_desktop);

        let complete_sections = SnapshotSections {
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
        };
        assert_eq!(
            missing_evidence(&complete_sections),
            Vec::<SnapshotMissingEvidence>::new()
        );
    }

    #[test]
    fn snapshot_quote_error_routes_to_chart_quote_follow_up() {
        let sections = SnapshotSections {
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
                error: Some(SnapshotSectionError {
                    section: "info".to_string(),
                    kind: ErrorKind::InternalApiUnavailable,
                    message: "temporary failure".to_string(),
                    details: None,
                }),
            },
            fundamentals: SnapshotSection {
                ok: false,
                data: None,
                error: Some(SnapshotSectionError {
                    section: "fundamentals".to_string(),
                    kind: ErrorKind::InternalApiUnavailable,
                    message: "temporary failure".to_string(),
                    details: None,
                }),
            },
        };
        let evidence = missing_evidence(&sections);
        assert_eq!(evidence.len(), 3);
        assert_eq!(evidence[0].section, "quote");
        assert_eq!(evidence[0].missing_reason, "section_error");
        assert_eq!(evidence[0].suggested_follow_up, "chart_quote");
        assert!(evidence[0].requires_desktop);
        assert_eq!(evidence[1].section, "info");
        assert_eq!(evidence[1].suggested_follow_up, "snapshot");
        assert!(!evidence[1].requires_desktop);
        assert_eq!(evidence[2].section, "fundamentals");
        assert_eq!(evidence[2].suggested_follow_up, "snapshot");
        assert!(!evidence[2].requires_desktop);
    }

    #[test]
    fn snapshot_follow_up_hints_are_machine_readable_without_recommendation() {
        let hints = follow_up_hints(&json!("NASDAQ:AAPL"));
        assert_eq!(hints.len(), 3);
        assert_eq!(hints[0].kind, "chart_quote");
        assert_eq!(hints[0].command, "tv quote NASDAQ:AAPL --source chart");
        assert_eq!(hints[0].reason, "single_symbol_chart_quote");
        assert!(hints[0].requires_desktop);
        assert!(hints.iter().any(|hint| hint.kind == "observe_chart"));
        assert!(hints.iter().any(|hint| hint.kind == "screenshot"));
    }

    #[test]
    fn total_failure_details_include_snapshot_contract_and_blocked_status() {
        let sections = SnapshotSections {
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
                error: Some(SnapshotSectionError {
                    section: "info".to_string(),
                    kind: ErrorKind::InternalApiUnavailable,
                    message: "temporary failure".to_string(),
                    details: None,
                }),
            },
            fundamentals: SnapshotSection {
                ok: false,
                data: None,
                error: Some(SnapshotSectionError {
                    section: "fundamentals".to_string(),
                    kind: ErrorKind::InternalApiUnavailable,
                    message: "temporary failure".to_string(),
                    details: None,
                }),
            },
        };
        let errors = section_errors(&sections);
        let snapshot = Snapshot {
            contract_version: SNAPSHOT_CONTRACT_VERSION.to_string(),
            source: SNAPSHOT_SOURCE.to_string(),
            source_category: DESKTOP_FREE_READ_CATEGORY.to_string(),
            requires_desktop: false,
            non_mutating: true,
            requested_symbol: "AAPL".to_string(),
            symbol: best_symbol(&sections),
            observed_symbol: best_observed_symbol(&sections),
            summary: snapshot_summary(&sections, errors.len()),
            missing_evidence: missing_evidence(&sections),
            sections,
            errors,
            follow_up_hints: follow_up_hints(&Value::Null),
            next_action_hints: Vec::new(),
        };

        assert_eq!(snapshot.contract_version, "snapshot.v1");
        assert_eq!(snapshot.summary.coverage_status, "blocked");
        assert!(!snapshot.summary.quote_ok);
        assert!(!snapshot.summary.info_ok);
        assert!(!snapshot.summary.fundamentals_ok);
        assert_eq!(snapshot.missing_evidence.len(), 3);
    }
}
