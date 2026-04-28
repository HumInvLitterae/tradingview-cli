use serde_json::{Map, Value, json};

use tradingview_core::{AppError, ErrorKind};

use super::validation::{
    ScreenerColumnAddRequest, ScreenerColumnSelector, is_test_screener_screen_name,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScreenerColumnTarget {
    pub index: usize,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScreenerStorageColumnTarget {
    pub index: usize,
    pub id: String,
    pub name: Option<String>,
    pub params: Value,
}

pub fn column_targets_from_state(state: &Value) -> Vec<ScreenerColumnTarget> {
    state
        .get("columns")
        .and_then(Value::as_array)
        .map(|columns| {
            columns
                .iter()
                .enumerate()
                .filter_map(|(index, column)| {
                    column.as_str().map(str::trim).and_then(|name| {
                        if name.is_empty() {
                            None
                        } else {
                            Some(ScreenerColumnTarget {
                                index,
                                name: name.to_string(),
                            })
                        }
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn resolve_column_target(
    columns: &[ScreenerColumnTarget],
    selector: &ScreenerColumnSelector,
) -> Result<ScreenerColumnTarget, AppError> {
    match selector {
        ScreenerColumnSelector::Index(index) => columns
            .iter()
            .find(|column| column.index == *index)
            .cloned()
            .ok_or_else(|| {
                AppError::new(
                    ErrorKind::Validation,
                    format!("No visible Screener column found at index {index}"),
                )
                .with_details(json!({ "columns": column_targets_payload(columns) }))
            }),
        ScreenerColumnSelector::Name(name) => {
            let needle = name.to_lowercase();
            let matches = columns
                .iter()
                .filter(|column| column.name.to_lowercase().contains(&needle))
                .cloned()
                .collect::<Vec<_>>();
            match matches.len() {
                0 => Err(AppError::new(
                    ErrorKind::Validation,
                    format!("No visible Screener column matched name {name:?}"),
                )
                .with_details(json!({ "columns": column_targets_payload(columns) }))),
                1 => Ok(matches[0].clone()),
                _ => Err(AppError::new(
                    ErrorKind::Validation,
                    format!("Screener column name {name:?} matched multiple columns"),
                )
                .with_details(json!({ "matches": column_targets_payload(&matches) }))),
            }
        }
    }
}

pub fn column_target_payload(column: &ScreenerColumnTarget) -> Value {
    json!({
        "index": column.index,
        "name": column.name,
    })
}

pub fn column_targets_payload(columns: &[ScreenerColumnTarget]) -> Vec<Value> {
    columns.iter().map(column_target_payload).collect()
}

pub fn ensure_test_screener_screen_for_column_mutation(
    screen_title: &str,
    operation: &str,
) -> Result<(), AppError> {
    if is_test_screener_screen_name(screen_title) {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorKind::Validation,
            format!(
                "Screener columns {operation} mutation is limited to test screen names containing CLI-Test or テスト"
            ),
        )
        .with_details(json!({ "screen_title": screen_title })))
    }
}

pub fn storage_columns_from_config(
    config: &Value,
    visible_columns: &[ScreenerColumnTarget],
) -> Vec<ScreenerStorageColumnTarget> {
    config
        .get("columns")
        .and_then(Value::as_array)
        .map(|columns| {
            columns
                .iter()
                .enumerate()
                .filter_map(|(index, column)| {
                    let id = column.get("id").and_then(Value::as_str)?.trim();
                    if id.is_empty() {
                        return None;
                    }
                    let params = column
                        .get("params")
                        .cloned()
                        .unwrap_or_else(|| Value::Object(Map::new()));
                    Some(ScreenerStorageColumnTarget {
                        index,
                        id: id.to_string(),
                        name: visible_columns.get(index).map(|column| column.name.clone()),
                        params,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn storage_column_target_payload(column: &ScreenerStorageColumnTarget) -> Value {
    json!({
        "index": column.index,
        "id": column.id,
        "name": column.name,
        "name_source": column.name.as_ref().map(|_| "visible_column_index"),
        "params": column.params,
    })
}

pub fn storage_column_targets_payload(columns: &[ScreenerStorageColumnTarget]) -> Vec<Value> {
    columns.iter().map(storage_column_target_payload).collect()
}

pub fn storage_column_update_payload(columns: &[ScreenerStorageColumnTarget]) -> Vec<Value> {
    columns
        .iter()
        .map(|column| {
            json!({
                "id": column.id,
                "params": column.params,
            })
        })
        .collect()
}

pub fn ensure_storage_column_index(
    columns: &[ScreenerStorageColumnTarget],
    index: usize,
) -> Result<(), AppError> {
    if index < columns.len() {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorKind::Validation,
            format!("No saved Screener column found at index {index}"),
        )
        .with_details(json!({
            "column_count": columns.len(),
            "columns": storage_column_targets_payload(columns),
        })))
    }
}

pub fn remove_storage_column(
    columns: &[ScreenerStorageColumnTarget],
    index: usize,
) -> Vec<ScreenerStorageColumnTarget> {
    columns
        .iter()
        .enumerate()
        .filter(|(column_index, _)| *column_index != index)
        .map(|(_, column)| column.clone())
        .enumerate()
        .map(|(index, mut column)| {
            column.index = index;
            column
        })
        .collect()
}

pub fn add_storage_column(
    columns: &[ScreenerStorageColumnTarget],
    request: &ScreenerColumnAddRequest,
) -> Result<Vec<ScreenerStorageColumnTarget>, AppError> {
    if let Some(after_index) = request.after_index {
        ensure_storage_column_index(columns, after_index)?;
    }
    let insert_index = request
        .after_index
        .map(|index| index + 1)
        .unwrap_or(columns.len());
    let mut added = columns.to_vec();
    added.insert(
        insert_index,
        ScreenerStorageColumnTarget {
            index: insert_index,
            id: request.id.clone(),
            name: None,
            params: request.params.clone(),
        },
    );
    Ok(added
        .into_iter()
        .enumerate()
        .map(|(index, mut column)| {
            column.index = index;
            column
        })
        .collect())
}

pub fn reorder_storage_columns(
    columns: &[ScreenerStorageColumnTarget],
    from_index: usize,
    to_index: usize,
) -> Vec<ScreenerStorageColumnTarget> {
    let mut reordered = columns.to_vec();
    let column = reordered.remove(from_index);
    reordered.insert(to_index, column);
    reordered
        .into_iter()
        .enumerate()
        .map(|(index, mut column)| {
            column.index = index;
            column
        })
        .collect()
}

pub fn storage_column_order_matches(
    actual: &[ScreenerStorageColumnTarget],
    expected: &[ScreenerStorageColumnTarget],
) -> bool {
    actual.len() == expected.len()
        && actual
            .iter()
            .zip(expected)
            .all(|(actual, expected)| actual.id == expected.id && actual.params == expected.params)
}
