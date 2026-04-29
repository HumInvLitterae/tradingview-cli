use serde_json::{Value, json};

use tradingview_core::{AppError, ErrorKind};

use super::validation::{ScreenerFilterSelector, is_test_screener_screen_name};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScreenerFilterTarget {
    pub index: usize,
    pub text: String,
    pub data_name: String,
    pub visible: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScreenerStorageFilterTarget {
    pub index: usize,
    pub text: Option<String>,
    pub filter_type: Option<String>,
    pub subtype: Option<String>,
    pub operation: Option<String>,
    pub raw: Value,
}

pub fn normalize_filters(filters: Option<&Value>) -> Vec<Value> {
    filters
        .and_then(Value::as_array)
        .map(|filters| {
            filters
                .iter()
                .enumerate()
                .map(|(index, filter)| {
                    json!({
                        "index": index,
                        "text": filter.get("text").cloned().unwrap_or(Value::Null),
                        "data_name": filter.get("data_name").cloned().unwrap_or(Value::Null),
                        "visible": filter.get("visible").and_then(Value::as_bool).unwrap_or(false),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn filter_targets_from_state(state: &Value) -> Vec<ScreenerFilterTarget> {
    state
        .get("filters")
        .and_then(Value::as_array)
        .map(|filters| {
            filters
                .iter()
                .enumerate()
                .filter_map(|(fallback_index, filter)| {
                    let data_name = filter.get("data_name").and_then(Value::as_str)?;
                    let text = filter
                        .get("text")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .trim();
                    Some(ScreenerFilterTarget {
                        index: filter
                            .get("index")
                            .and_then(Value::as_u64)
                            .and_then(|value| usize::try_from(value).ok())
                            .unwrap_or(fallback_index),
                        text: text.to_string(),
                        data_name: data_name.to_string(),
                        visible: filter
                            .get("visible")
                            .and_then(Value::as_bool)
                            .unwrap_or(false),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn resolve_filter_target(
    filters: &[ScreenerFilterTarget],
    selector: &ScreenerFilterSelector,
) -> Result<ScreenerFilterTarget, AppError> {
    match selector {
        ScreenerFilterSelector::Index(index) => filters
            .iter()
            .find(|filter| filter.index == *index)
            .cloned()
            .ok_or_else(|| {
                AppError::new(
                    ErrorKind::Validation,
                    format!("No visible Screener filter found at index {index}"),
                )
                .with_details(json!({ "filters": filter_targets_payload(filters) }))
            }),
        ScreenerFilterSelector::Text(text) => {
            let needle = text.to_lowercase();
            let matches = filters
                .iter()
                .filter(|filter| filter.text.to_lowercase().contains(&needle))
                .cloned()
                .collect::<Vec<_>>();
            match matches.len() {
                0 => Err(AppError::new(
                    ErrorKind::Validation,
                    format!("No visible Screener filter matched text {text:?}"),
                )
                .with_details(json!({ "filters": filter_targets_payload(filters) }))),
                1 => Ok(matches[0].clone()),
                _ => Err(AppError::new(
                    ErrorKind::Validation,
                    format!("Screener filter text {text:?} matched multiple filters"),
                )
                .with_details(json!({ "matches": filter_targets_payload(&matches) }))),
            }
        }
    }
}

pub fn filter_target_payload(filter: &ScreenerFilterTarget) -> Value {
    json!({
        "index": filter.index,
        "text": filter.text,
        "data_name": filter.data_name,
        "visible": filter.visible,
    })
}

pub fn filter_targets_payload(filters: &[ScreenerFilterTarget]) -> Vec<Value> {
    filters.iter().map(filter_target_payload).collect()
}

pub fn ensure_test_screener_screen_for_filter_mutation(
    screen_title: &str,
    operation: &str,
) -> Result<(), AppError> {
    if is_test_screener_screen_name(screen_title) {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorKind::Validation,
            format!(
                "Screener filters {operation} mutation is limited to test screen names containing CLI-Test or テスト"
            ),
        )
        .with_details(json!({ "screen_title": screen_title })))
    }
}

pub fn storage_filters_from_config(
    config: &Value,
    visible_filters: &[ScreenerFilterTarget],
) -> Vec<ScreenerStorageFilterTarget> {
    config
        .get("storage_screen")
        .and_then(|screen| screen.get("filters"))
        .and_then(Value::as_array)
        .map(|filters| {
            filters
                .iter()
                .enumerate()
                .map(|(index, filter)| ScreenerStorageFilterTarget {
                    index,
                    text: visible_filters.get(index).map(|filter| filter.text.clone()),
                    filter_type: filter
                        .get("type")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                    subtype: filter
                        .get("subtype")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                    operation: filter
                        .get("operation")
                        .and_then(|operation| {
                            operation
                                .as_str()
                                .or_else(|| operation.get("type").and_then(Value::as_str))
                        })
                        .map(str::to_string),
                    raw: filter.clone(),
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn storage_filter_target_payload(filter: &ScreenerStorageFilterTarget) -> Value {
    json!({
        "index": filter.index,
        "text": filter.text,
        "text_source": filter.text.as_ref().map(|_| "visible_filter_index"),
        "type": filter.filter_type,
        "subtype": filter.subtype,
        "operation": filter.operation,
    })
}

pub fn storage_filter_targets_payload(filters: &[ScreenerStorageFilterTarget]) -> Vec<Value> {
    filters.iter().map(storage_filter_target_payload).collect()
}

pub fn storage_filter_update_payload(filters: &[ScreenerStorageFilterTarget]) -> Vec<Value> {
    filters.iter().map(|filter| filter.raw.clone()).collect()
}

pub fn replace_storage_filter_range(
    filters: &[ScreenerStorageFilterTarget],
    index: usize,
    min: Option<f64>,
    max: Option<f64>,
) -> Result<Vec<ScreenerStorageFilterTarget>, AppError> {
    ensure_storage_filter_index(filters, index)?;
    let mut updated = filters.to_vec();
    let mut filter = updated[index].clone();
    let target_payload = storage_filter_target_payload(&filter);
    let raw = filter.raw.as_object_mut().ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener storage filter payload was not an object",
        )
        .with_details(target_payload.clone())
    })?;
    let filter_type = raw.get("type").and_then(Value::as_str);
    if filter_type != Some("Condition") {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener storage filter range update only supports Condition filters",
        )
        .with_details(target_payload.clone()));
    }
    let operation_type = raw
        .get("operation")
        .and_then(|operation| operation.get("type"))
        .and_then(Value::as_str);
    if !matches!(operation_type, Some("above" | "between")) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener storage filter range update only supports simple above/between operations",
        )
        .with_details(target_payload.clone()));
    }

    match (min, max) {
        (Some(min), Some(max)) => {
            if min != 0.0 || max <= min {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "Screener storage range update supports only --min 0 --max <N>",
                ));
            }
            raw.insert("operation".to_string(), json!({ "type": "between" }));
            raw.insert("right".to_string(), json!({ "left": min, "right": max }));
            filter.operation = Some("between".to_string());
        }
        (Some(min), None) => {
            raw.insert("operation".to_string(), json!({ "type": "above" }));
            raw.insert("right".to_string(), json!({ "value": min }));
            filter.operation = Some("above".to_string());
        }
        (None, Some(_)) => {
            return Err(AppError::new(
                ErrorKind::Validation,
                "Screener storage range update does not support --max without --min",
            ));
        }
        (None, None) => {
            return Err(AppError::new(
                ErrorKind::Validation,
                "Either --min or --max is required",
            ));
        }
    }
    updated[index] = filter;
    Ok(updated)
}

pub fn ensure_storage_filter_alignment(
    visible_filters: &[ScreenerFilterTarget],
    storage_filters: &[ScreenerStorageFilterTarget],
) -> Result<(), AppError> {
    if visible_filters.len() == storage_filters.len() {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Visible Screener filters did not align with saved storage filters",
        )
        .with_details(json!({
            "visible_filter_count": visible_filters.len(),
            "storage_filter_count": storage_filters.len(),
            "visible_filters": filter_targets_payload(visible_filters),
            "storage_filters": storage_filter_targets_payload(storage_filters),
        })))
    }
}

pub fn ensure_storage_filter_index(
    filters: &[ScreenerStorageFilterTarget],
    index: usize,
) -> Result<(), AppError> {
    if index < filters.len() {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorKind::Validation,
            format!("No saved Screener filter found at index {index}"),
        )
        .with_details(json!({
            "filter_count": filters.len(),
            "filters": storage_filter_targets_payload(filters),
        })))
    }
}

pub fn remove_storage_filter(
    filters: &[ScreenerStorageFilterTarget],
    index: usize,
) -> Vec<ScreenerStorageFilterTarget> {
    filters
        .iter()
        .enumerate()
        .filter(|(filter_index, _)| *filter_index != index)
        .map(|(_, filter)| filter.clone())
        .enumerate()
        .map(|(index, mut filter)| {
            filter.index = index;
            filter
        })
        .collect()
}

pub fn storage_filter_order_matches(
    actual: &[ScreenerStorageFilterTarget],
    expected: &[ScreenerStorageFilterTarget],
) -> bool {
    actual.len() == expected.len()
        && actual
            .iter()
            .zip(expected)
            .all(|(actual, expected)| json_values_equivalent(&actual.raw, &expected.raw))
}

fn json_values_equivalent(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::Number(actual), Value::Number(expected)) => {
            match (actual.as_f64(), expected.as_f64()) {
                (Some(actual), Some(expected)) => (actual - expected).abs() < f64::EPSILON,
                _ => actual == expected,
            }
        }
        (Value::Array(actual), Value::Array(expected)) => {
            actual.len() == expected.len()
                && actual
                    .iter()
                    .zip(expected)
                    .all(|(actual, expected)| json_values_equivalent(actual, expected))
        }
        (Value::Object(actual), Value::Object(expected)) => {
            actual.len() == expected.len()
                && actual.iter().all(|(key, actual)| {
                    expected
                        .get(key)
                        .is_some_and(|expected| json_values_equivalent(actual, expected))
                })
        }
        _ => actual == expected,
    }
}

pub fn added_filter_target(
    before_filters: &[ScreenerFilterTarget],
    after_filters: &[ScreenerFilterTarget],
    name: &str,
    matchers: &[String],
) -> Option<ScreenerFilterTarget> {
    let before_names = before_filters
        .iter()
        .map(|filter| filter.data_name.as_str())
        .collect::<std::collections::HashSet<_>>();
    let name_tokens = screener_filter_name_tokens(name);
    let numeric_tokens = screener_filter_numeric_tokens(matchers);
    after_filters
        .iter()
        .find(|filter| {
            if before_names.contains(filter.data_name.as_str()) {
                return false;
            }
            let normalized = normalize_screener_text(&filter.text).to_lowercase();
            let has_name = name_tokens
                .iter()
                .any(|token| normalized.contains(&token.to_lowercase()));
            let has_number = numeric_tokens
                .iter()
                .any(|token| normalized.contains(token));
            has_name && has_number
        })
        .cloned()
}

fn screener_filter_name_tokens(name: &str) -> Vec<String> {
    normalize_screener_text(name)
        .split(|ch: char| {
            !(ch.is_ascii_alphanumeric()
                || ('\u{3040}'..='\u{30ff}').contains(&ch)
                || ('\u{3400}'..='\u{9fff}').contains(&ch))
        })
        .map(str::trim)
        .filter(|token| token.len() >= 2)
        .map(ToString::to_string)
        .collect()
}

fn screener_filter_numeric_tokens(matchers: &[String]) -> Vec<String> {
    matchers
        .iter()
        .flat_map(|matcher| {
            matcher
                .split(|ch: char| !(ch.is_ascii_digit() || ch == '.'))
                .filter(|token| !token.is_empty())
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .collect()
}

pub fn normalize_screener_text(value: &str) -> String {
    value
        .replace(['\u{2066}', '\u{2067}', '\u{2068}', '\u{2069}'], "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn screener_filter_text_matches_option(filter_text: &str, option: &str) -> bool {
    let option = normalize_screener_text(option);
    if option.is_empty() {
        return false;
    }
    let text = normalize_screener_text(filter_text);
    if text == option || text.split_whitespace().any(|token| token == option) {
        return true;
    }
    if option == "買い" && text.ends_with("強い買い") {
        return false;
    }
    if option == "売り" && text.ends_with("強い売り") {
        return false;
    }
    text.ends_with(&option)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn storage_target(raw: Value) -> ScreenerStorageFilterTarget {
        ScreenerStorageFilterTarget {
            index: 0,
            text: None,
            filter_type: raw.get("type").and_then(Value::as_str).map(str::to_string),
            subtype: raw
                .get("subtype")
                .and_then(Value::as_str)
                .map(str::to_string),
            operation: raw
                .get("operation")
                .and_then(|operation| operation.get("type"))
                .and_then(Value::as_str)
                .map(str::to_string),
            raw,
        }
    }

    #[test]
    fn replace_storage_filter_range_sets_between_payload() {
        let filters = vec![storage_target(json!({
            "type": "Condition",
            "left": { "column": { "id": "change", "params": {} } },
            "operation": { "type": "above" },
            "right": { "value": 10 },
            "target": "change"
        }))];

        let updated = replace_storage_filter_range(&filters, 0, Some(0.0), Some(5.0)).unwrap();

        assert_eq!(updated[0].operation.as_deref(), Some("between"));
        assert_eq!(updated[0].raw["operation"]["type"], "between");
        assert_eq!(
            updated[0].raw["right"],
            json!({ "left": 0.0, "right": 5.0 })
        );
    }

    #[test]
    fn replace_storage_filter_range_rejects_complex_operation() {
        let filters = vec![storage_target(json!({
            "type": "Condition",
            "left": { "column": { "id": "close", "params": {} } },
            "operation": {
                "type": "belowPercent",
                "params": { "offsetRangeId": "offset_range_0_10" }
            },
            "right": { "column": { "id": "ema", "params": { "length": 21 } } },
            "target": "close"
        }))];

        let error = replace_storage_filter_range(&filters, 0, Some(0.0), Some(5.0)).unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
    }

    #[test]
    fn storage_filter_order_matches_treats_integer_and_float_numbers_as_equivalent() {
        let actual = vec![storage_target(json!({
            "type": "Condition",
            "operation": { "type": "between" },
            "right": { "left": 0, "right": 5 }
        }))];
        let expected = vec![storage_target(json!({
            "type": "Condition",
            "operation": { "type": "between" },
            "right": { "left": 0.0, "right": 5.0 }
        }))];

        assert!(storage_filter_order_matches(&actual, &expected));
    }
}
