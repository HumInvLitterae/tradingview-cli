use serde_json::{Map, Value, json};

use tradingview_core::{AppError, ErrorKind};

const DEFAULT_SCREENER_LIMIT: usize = 20;
const MAX_SCREENER_LIMIT: usize = 100;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScreenerFilterSelector {
    Index(usize),
    Text(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScreenerFilterModifyRequest {
    pub selector: ScreenerFilterSelector,
    pub dry_run: bool,
    pub mode: ScreenerFilterModifyMode,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ScreenerFilterModifyMode {
    Range {
        min: Option<f64>,
        max: Option<f64>,
        preset_label: String,
    },
    Option {
        option: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScreenerFilterAddRequest {
    pub name: String,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub dry_run: bool,
    pub range_matchers: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScreenerColumnAddRequest {
    pub id: String,
    pub params: Value,
    pub after_index: Option<usize>,
    pub dry_run: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScreenerColumnSelector {
    Index(usize),
    Name(String),
}

pub fn validate_screener_limit(limit: Option<usize>) -> Result<usize, AppError> {
    match limit {
        Some(0) => Err(AppError::new(
            ErrorKind::Validation,
            "--limit must be greater than 0",
        )),
        Some(limit) => Ok(limit.min(MAX_SCREENER_LIMIT)),
        None => Ok(DEFAULT_SCREENER_LIMIT),
    }
}

pub fn validate_screener_filter_selector(
    index: Option<usize>,
    text: Option<&str>,
) -> Result<ScreenerFilterSelector, AppError> {
    let text = text.map(str::trim).filter(|value| !value.is_empty());
    match (index, text) {
        (Some(_), Some(_)) => Err(AppError::new(
            ErrorKind::Validation,
            "--index and --text are mutually exclusive",
        )),
        (Some(index), None) => Ok(ScreenerFilterSelector::Index(index)),
        (None, Some(text)) => Ok(ScreenerFilterSelector::Text(text.to_string())),
        (None, None) => Err(AppError::new(
            ErrorKind::Validation,
            "Either --index or --text is required",
        )),
    }
}

pub fn validate_screener_filter_modify_request(
    index: Option<usize>,
    text: Option<&str>,
    min: Option<f64>,
    max: Option<f64>,
    option: Option<&str>,
    dry_run: bool,
) -> Result<ScreenerFilterModifyRequest, AppError> {
    let selector = validate_screener_filter_selector(index, text)?;
    let option = match option {
        Some(value) => {
            let value = value.trim();
            if value.is_empty() {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "--option must not be empty",
                ));
            }
            Some(value.to_string())
        }
        None => None,
    };
    let mode = if let Some(option) = option {
        if min.is_some() || max.is_some() {
            return Err(AppError::new(
                ErrorKind::Validation,
                "--option cannot be used with --min or --max",
            ));
        }
        ScreenerFilterModifyMode::Option { option }
    } else {
        let preset_label = screener_filter_range_preset_label(min, max)?;
        ScreenerFilterModifyMode::Range {
            min,
            max,
            preset_label,
        }
    };

    Ok(ScreenerFilterModifyRequest {
        selector,
        dry_run,
        mode,
    })
}

pub fn validate_screener_filter_add_request(
    name: &str,
    min: Option<f64>,
    max: Option<f64>,
    dry_run: bool,
) -> Result<ScreenerFilterAddRequest, AppError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "--name must not be empty",
        ));
    }
    let range_matchers = screener_filter_add_range_matchers(min, max)?;
    Ok(ScreenerFilterAddRequest {
        name: name.to_string(),
        min,
        max,
        dry_run,
        range_matchers,
    })
}

pub fn validate_screener_column_selector(
    index: Option<usize>,
    name: Option<&str>,
) -> Result<ScreenerColumnSelector, AppError> {
    let name = name.map(str::trim).filter(|value| !value.is_empty());
    match (index, name) {
        (Some(_), Some(_)) => Err(AppError::new(
            ErrorKind::Validation,
            "--index and --name are mutually exclusive",
        )),
        (Some(index), None) => Ok(ScreenerColumnSelector::Index(index)),
        (None, Some(name)) => Ok(ScreenerColumnSelector::Name(name.to_string())),
        (None, None) => Err(AppError::new(
            ErrorKind::Validation,
            "Either --index or --name is required",
        )),
    }
}

pub fn validate_screener_column_add_request(
    id: &str,
    params_json: Option<&str>,
    after_index: Option<usize>,
    dry_run: bool,
) -> Result<ScreenerColumnAddRequest, AppError> {
    let id = id.trim();
    if id.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "screener columns add requires a non-empty --id",
        ));
    }
    let params = match params_json {
        Some(raw) => {
            let value: Value = serde_json::from_str(raw).map_err(|error| {
                AppError::new(
                    ErrorKind::Validation,
                    format!("--params-json must be valid JSON: {error}"),
                )
            })?;
            if !value.is_object() {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "--params-json must be a JSON object",
                )
                .with_details(json!({ "params_json": value })));
            }
            value
        }
        None => Value::Object(Map::new()),
    };
    Ok(ScreenerColumnAddRequest {
        id: id.to_string(),
        params,
        after_index,
        dry_run,
    })
}

pub fn validate_screener_column_reorder_request(
    from_index: usize,
    to_index: usize,
) -> Result<(usize, usize), AppError> {
    if from_index == to_index {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Screener columns reorder requires different --from-index and --to-index values",
        ));
    }
    Ok((from_index, to_index))
}

pub fn validate_screener_filter_clear(dry_run: bool, confirm_clear: bool) -> Result<(), AppError> {
    if !dry_run && !confirm_clear {
        return Err(AppError::new(
            ErrorKind::Validation,
            "screener filters clear requires --confirm-clear unless --dry-run is used",
        ));
    }
    Ok(())
}

pub fn validate_screener_screen_name(name: &str) -> Result<String, AppError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "--name must not be empty",
        ));
    }
    Ok(name.to_string())
}

pub fn validate_screener_screen_rename_request(
    name: &str,
    new_name: &str,
    dry_run: bool,
) -> Result<(String, String), AppError> {
    let name = validate_screener_screen_name(name)?;
    let new_name = validate_screener_screen_name(new_name)?;
    if name == new_name {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Screener rename requires a different --to name",
        ));
    }
    if !dry_run
        && (!is_test_screener_screen_name(&name) || !is_test_screener_screen_name(&new_name))
    {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Screener rename mutation is limited to test screen names containing CLI-Test or テスト",
        ));
    }
    Ok((name, new_name))
}

pub fn validate_screener_screen_test_mutation_name(
    name: &str,
    dry_run: bool,
    operation: &str,
) -> Result<String, AppError> {
    let name = validate_screener_screen_name(name)?;
    if !dry_run && !is_test_screener_screen_name(&name) {
        return Err(AppError::new(
            ErrorKind::Validation,
            format!(
                "Screener {operation} mutation is limited to test screen names containing CLI-Test or テスト"
            ),
        ));
    }
    Ok(name)
}

pub fn validate_screener_screen_delete_request(
    name: &str,
    dry_run: bool,
    confirm_delete: bool,
) -> Result<String, AppError> {
    let name = validate_screener_screen_name(name)?;
    if !dry_run && !confirm_delete {
        return Err(AppError::new(
            ErrorKind::Validation,
            "screener screens delete requires --confirm-delete unless --dry-run is used",
        ));
    }
    if !dry_run && !is_test_screener_screen_name(&name) {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Screener delete mutation is limited to test screen names containing CLI-Test or テスト",
        ));
    }
    Ok(name)
}

pub fn is_test_screener_screen_name(name: &str) -> bool {
    name.contains("CLI-Test") || name.contains("テスト")
}

pub fn filter_modify_range_payload(request: &ScreenerFilterModifyRequest) -> Value {
    match &request.mode {
        ScreenerFilterModifyMode::Range {
            min,
            max,
            preset_label,
        } => json!({
            "min": min,
            "max": max,
            "preset_label": preset_label,
        }),
        ScreenerFilterModifyMode::Option { .. } => Value::Null,
    }
}

pub fn filter_modify_option_payload(option: &str, matched_option: Option<&Value>) -> Value {
    json!({
        "option": option,
        "matched_option": matched_option.cloned().unwrap_or(Value::Null),
    })
}

pub fn filter_add_range_payload(request: &ScreenerFilterAddRequest) -> Value {
    json!({
        "min": request.min,
        "max": request.max,
        "matchers": request.range_matchers,
    })
}

fn screener_filter_range_preset_label(
    min: Option<f64>,
    max: Option<f64>,
) -> Result<String, AppError> {
    if min.is_none() && max.is_none() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Either --min or --max is required",
        ));
    }
    if let Some(value) = min {
        require_finite(value, "--min")?;
        if value < 0.0 {
            return Err(AppError::new(
                ErrorKind::Validation,
                "--min must be greater than or equal to 0",
            ));
        }
    }
    if let Some(value) = max {
        require_finite(value, "--max")?;
        if value <= 0.0 {
            return Err(AppError::new(
                ErrorKind::Validation,
                "--max must be greater than 0",
            ));
        }
    }

    match (min, max) {
        (Some(min), Some(max)) => {
            if !approximately(min, 0.0) || max <= min {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "Screener filter preset ranges currently support --min 0 --max <N>",
                ));
            }
            ensure_supported_filter_preset(max, &[3.0, 5.0, 10.0, 20.0, 30.0], "--max")?;
            Ok(format!("0% 〜 {}%", format_filter_percent(max)))
        }
        (Some(min), None) => {
            ensure_supported_filter_preset(
                min,
                &[
                    3.0, 5.0, 10.0, 15.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0,
                ],
                "--min",
            )?;
            Ok(format!("{}%以上", format_filter_percent(min)))
        }
        (None, Some(_)) => Err(AppError::new(
            ErrorKind::Validation,
            "Screener filter preset ranges do not currently support --max without --min",
        )),
        (None, None) => unreachable!(),
    }
}

fn require_finite(value: f64, label: &str) -> Result<(), AppError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorKind::Validation,
            format!("{label} must be a finite number"),
        ))
    }
}

fn screener_filter_add_range_matchers(
    min: Option<f64>,
    max: Option<f64>,
) -> Result<Vec<String>, AppError> {
    if min.is_none() && max.is_none() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Either --min or --max is required",
        ));
    }
    if let Some(value) = min {
        require_finite(value, "--min")?;
    }
    if let Some(value) = max {
        require_finite(value, "--max")?;
    }
    match (min, max) {
        (Some(min), Some(max)) => {
            if max <= min {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "--max must be greater than --min",
                ));
            }
            let min = format_filter_percent(min);
            let max = format_filter_percent(max);
            Ok(vec![
                format!("{min}% 〜 {max}%"),
                format!("{min}% to {max}%"),
                format!("{min} 〜 {max}"),
                format!("{min} to {max}"),
            ])
        }
        (Some(min), None) => {
            let min = format_filter_percent(min);
            Ok(vec![
                format!("> {min}"),
                format!(">{min}"),
                format!("{min}%以上"),
                format!("{min}以上"),
            ])
        }
        (None, Some(max)) => {
            let max = format_filter_percent(max);
            Ok(vec![
                format!("< {max}"),
                format!("<{max}"),
                format!("{max}%以下"),
                format!("{max}以下"),
                format!("{max}%未満"),
                format!("{max}未満"),
            ])
        }
        (None, None) => unreachable!(),
    }
}

fn ensure_supported_filter_preset(
    value: f64,
    supported: &[f64],
    label: &str,
) -> Result<(), AppError> {
    if supported
        .iter()
        .any(|supported| approximately(value, *supported))
    {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorKind::Validation,
            format!("{label} does not match a supported visible Screener preset"),
        )
        .with_details(json!({ "supported": supported })))
    }
}

fn approximately(left: f64, right: f64) -> bool {
    (left - right).abs() < 0.000_001
}

fn format_filter_percent(value: f64) -> String {
    if approximately(value.fract(), 0.0) {
        format!("{}", value.trunc() as i64)
    } else {
        format!("{value}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_screener_limit_defaults_clamps_and_rejects_zero() {
        assert_eq!(validate_screener_limit(None).unwrap(), 20);
        assert_eq!(validate_screener_limit(Some(3)).unwrap(), 3);
        assert_eq!(validate_screener_limit(Some(500)).unwrap(), 100);

        let error = validate_screener_limit(Some(0)).unwrap_err();
        assert_eq!(error.kind, ErrorKind::Validation);
    }

    #[test]
    fn validate_screener_filter_selector_requires_one_target() {
        assert_eq!(
            validate_screener_filter_selector(Some(2), None).unwrap(),
            ScreenerFilterSelector::Index(2)
        );
        assert_eq!(
            validate_screener_filter_selector(None, Some(" PER ")).unwrap(),
            ScreenerFilterSelector::Text("PER".to_string())
        );
        assert_eq!(
            validate_screener_filter_selector(None, None)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_filter_selector(Some(0), Some("PER"))
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
    }

    #[test]
    fn validate_screener_filter_modify_accepts_visible_presets() {
        let request = validate_screener_filter_modify_request(
            None,
            Some("EMA"),
            Some(0.0),
            Some(5.0),
            None,
            true,
        )
        .unwrap();

        assert_eq!(
            request.selector,
            ScreenerFilterSelector::Text("EMA".to_string())
        );
        assert_eq!(
            filter_modify_range_payload(&request)["preset_label"],
            "0% 〜 5%"
        );

        let request =
            validate_screener_filter_modify_request(Some(1), None, Some(15.0), None, None, false)
                .unwrap();

        assert_eq!(request.selector, ScreenerFilterSelector::Index(1));
        assert_eq!(
            filter_modify_range_payload(&request)["preset_label"],
            "15%以上"
        );
    }

    #[test]
    fn validate_screener_filter_modify_accepts_option_mode() {
        let request = validate_screener_filter_modify_request(
            Some(7),
            None,
            None,
            None,
            Some(" 買い "),
            true,
        )
        .unwrap();

        assert_eq!(request.selector, ScreenerFilterSelector::Index(7));
        assert_eq!(
            request.mode,
            ScreenerFilterModifyMode::Option {
                option: "買い".to_string()
            }
        );
    }

    #[test]
    fn validate_screener_filter_modify_rejects_unsafe_inputs() {
        assert_eq!(
            validate_screener_filter_modify_request(None, None, Some(0.0), Some(5.0), None, true)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_filter_modify_request(
                Some(0),
                Some("EMA"),
                Some(0.0),
                Some(5.0),
                None,
                true,
            )
            .unwrap_err()
            .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_filter_modify_request(Some(0), None, None, None, None, true)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_filter_modify_request(Some(0), None, None, Some(5.0), None, true)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_filter_modify_request(
                Some(0),
                None,
                Some(f64::NAN),
                Some(5.0),
                None,
                true,
            )
            .unwrap_err()
            .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_filter_modify_request(
                Some(0),
                None,
                Some(0.0),
                Some(7.0),
                None,
                true,
            )
            .unwrap_err()
            .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_filter_modify_request(Some(0), None, None, None, Some(" "), true)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_filter_modify_request(
                Some(0),
                None,
                Some(0.0),
                None,
                Some("買い"),
                true,
            )
            .unwrap_err()
            .kind,
            ErrorKind::Validation
        );
    }

    #[test]
    fn validate_screener_filter_add_accepts_generic_numeric_presets() {
        let request =
            validate_screener_filter_add_request(" RSI (相対力指数) ", Some(70.0), None, true)
                .unwrap();

        assert_eq!(request.name, "RSI (相対力指数)");
        assert_eq!(
            request.range_matchers,
            vec!["> 70", ">70", "70%以上", "70以上"]
        );
        assert_eq!(filter_add_range_payload(&request)["min"], 70.0);

        let request = validate_screener_filter_add_request("RSI", None, Some(30.0), false).unwrap();
        assert!(request.range_matchers.contains(&"< 30".to_string()));

        let request =
            validate_screener_filter_add_request("Change", Some(0.0), Some(5.0), false).unwrap();
        assert!(request.range_matchers.contains(&"0% 〜 5%".to_string()));
    }

    #[test]
    fn validate_screener_filter_add_rejects_unsafe_inputs() {
        assert_eq!(
            validate_screener_filter_add_request("   ", Some(70.0), None, true)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_filter_add_request("RSI", None, None, true)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_filter_add_request("RSI", Some(f64::NAN), None, true)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_filter_add_request("RSI", Some(70.0), Some(60.0), true)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
    }

    #[test]
    fn validate_screener_column_selector_requires_one_target() {
        assert_eq!(
            validate_screener_column_selector(Some(2), None).unwrap(),
            ScreenerColumnSelector::Index(2)
        );
        assert_eq!(
            validate_screener_column_selector(None, Some(" Price ")).unwrap(),
            ScreenerColumnSelector::Name("Price".to_string())
        );
        assert_eq!(
            validate_screener_column_selector(None, None)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_column_selector(Some(0), Some("Price"))
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
    }

    #[test]
    fn validate_screener_column_reorder_rejects_same_index() {
        assert_eq!(
            validate_screener_column_reorder_request(1, 2).unwrap(),
            (1, 2)
        );
        assert_eq!(
            validate_screener_column_reorder_request(1, 1)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
    }

    #[test]
    fn validate_screener_column_add_rejects_unsafe_inputs() {
        let request = validate_screener_column_add_request(
            " TechnicalRating ",
            Some(r#"{"resolution":"TimeResolution1D"}"#),
            Some(11),
            true,
        )
        .unwrap();

        assert_eq!(request.id, "TechnicalRating");
        assert_eq!(request.params["resolution"], "TimeResolution1D");
        assert_eq!(request.after_index, Some(11));
        assert!(request.dry_run);

        assert_eq!(
            validate_screener_column_add_request("   ", None, None, true)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_column_add_request("Price", Some("{bad"), None, true)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_column_add_request("Price", Some("[]"), None, true)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
    }

    #[test]
    fn validate_screener_filter_clear_requires_confirmation_for_mutation() {
        assert!(validate_screener_filter_clear(true, false).is_ok());
        assert!(validate_screener_filter_clear(false, true).is_ok());
        assert_eq!(
            validate_screener_filter_clear(false, false)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
    }

    #[test]
    fn validate_screener_screen_name_trims_and_rejects_empty() {
        assert_eq!(
            validate_screener_screen_name(" 米国株（テスト用） ").unwrap(),
            "米国株（テスト用）"
        );
        assert_eq!(
            validate_screener_screen_name("   ").unwrap_err().kind,
            ErrorKind::Validation
        );
    }

    #[test]
    fn validate_screener_screen_lifecycle_requests_are_guarded() {
        assert_eq!(
            validate_screener_screen_test_mutation_name(" CLI-Test-New ", false, "create").unwrap(),
            "CLI-Test-New"
        );
        assert_eq!(
            validate_screener_screen_test_mutation_name("Production", false, "create")
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_screen_rename_request("CLI-Test1", "CLI-Test2", false).unwrap(),
            ("CLI-Test1".to_string(), "CLI-Test2".to_string())
        );
        assert_eq!(
            validate_screener_screen_rename_request("CLI-Test1", "CLI-Test1", true)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_screen_rename_request("Production", "CLI-Test2", false)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert!(validate_screener_screen_delete_request("CLI-Test1", true, false).is_ok());
        assert_eq!(
            validate_screener_screen_delete_request("CLI-Test1", false, false)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_screener_screen_delete_request("Production", false, true)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
    }
}
