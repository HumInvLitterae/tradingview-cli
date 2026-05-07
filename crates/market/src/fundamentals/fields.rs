use serde_json::json;
use tradingview_core::{AppError, ErrorKind};

const DEFAULT_FUNDAMENTAL_FIELDS: &[&str] = &[
    "name",
    "description",
    "exchange",
    "sector",
    "industry",
    "market_cap_basic",
    "price_earnings_ttm",
    "earnings_per_share_basic_ttm",
    "dividend_yield_recent",
    "earnings_release_next_date",
    "earnings_release_next_time",
    "earnings_release_date",
];

const EARNINGS_FUNDAMENTAL_FIELDS: &[&str] = &[
    "earnings_release_next_date",
    "earnings_release_date",
    "earnings_release_next_time",
    "earnings_release_next_calendar_date",
    "earnings_release_calendar_date",
    "earnings_release_next_trading_date_fy",
    "earnings_release_trading_date_fy",
    "earnings_release_next_trading_date_fq",
    "earnings_release_trading_date_fq",
    "earnings_publication_type_next_fq",
    "earnings_release_time",
    "earnings_publication_type_fq",
];

const VALUATION_FUNDAMENTAL_FIELDS: &[&str] = &[
    "market_cap_basic",
    "price_earnings_ttm",
    "price_earnings_forward_fy",
    "earnings_per_share_basic_ttm",
    "earnings_per_share_basic_fq",
    "earnings_per_share_fq",
    "earnings_per_share_forecast_next_fq",
    "earnings_per_share_forecast_next_fy",
];

const DIVIDENDS_FUNDAMENTAL_FIELDS: &[&str] = &[
    "dividend_yield_recent",
    "dividends_yield_current",
    "dividend_ex_date_recent",
    "dividend_ex_date_upcoming",
    "dividend_payment_date_recent",
    "dividend_payment_date_upcoming",
    "dividend_amount_recent",
    "dividend_amount_upcoming",
    "dividend_frequency_recent",
    "dividend_frequency_upcoming",
    "next_dividend_date",
    "expected_annual_dividends",
];

const FINANCIALS_FUNDAMENTAL_FIELDS: &[&str] = &[
    "total_revenue_ttm",
    "total_revenue_fq",
    "net_income_ttm",
    "net_income_fq",
    "revenue_forecast_next_fq",
    "revenue_forecast_next_fy",
];

const SUPPORTED_FUNDAMENTAL_GROUPS: &[&str] = &["earnings", "valuation", "dividends", "financials"];

const SUPPORTED_FUNDAMENTAL_FIELDS: &[&str] = &[
    "name",
    "description",
    "exchange",
    "type",
    "subtype",
    "sector",
    "industry",
    "market_cap_basic",
    "price_earnings_ttm",
    "price_earnings_forward_fy",
    "earnings_per_share_basic_ttm",
    "earnings_per_share_basic_fq",
    "earnings_per_share_fq",
    "earnings_per_share_forecast_next_fq",
    "earnings_per_share_forecast_next_fy",
    "revenue_forecast_next_fq",
    "revenue_forecast_next_fy",
    "total_revenue_ttm",
    "total_revenue_fq",
    "net_income_ttm",
    "net_income_fq",
    "dividend_yield_recent",
    "dividends_yield_current",
    "dividend_ex_date_recent",
    "dividend_ex_date_upcoming",
    "dividend_payment_date_recent",
    "dividend_payment_date_upcoming",
    "dividend_amount_recent",
    "dividend_amount_upcoming",
    "dividend_frequency_recent",
    "dividend_frequency_upcoming",
    "next_dividend_date",
    "expected_annual_dividends",
    "earnings_release_next_date",
    "earnings_release_date",
    "earnings_release_next_time",
    "earnings_release_next_calendar_date",
    "earnings_release_calendar_date",
    "earnings_release_next_trading_date_fy",
    "earnings_release_trading_date_fy",
    "earnings_release_next_trading_date_fq",
    "earnings_release_trading_date_fq",
    "earnings_publication_type_next_fq",
    "earnings_release_time",
    "earnings_publication_type_fq",
];

#[derive(Debug, Clone, PartialEq)]
pub(super) struct FundamentalSelection {
    pub groups: Vec<String>,
    pub fields: Vec<String>,
}

#[cfg(test)]
pub(super) fn normalize_fundamental_fields(fields: Vec<String>) -> Result<Vec<String>, AppError> {
    normalize_fundamental_selection(Vec::new(), fields).map(|selection| selection.fields)
}

pub(super) fn normalize_fundamental_selection(
    groups: Vec<String>,
    fields: Vec<String>,
) -> Result<FundamentalSelection, AppError> {
    if groups.is_empty() && fields.is_empty() {
        return Ok(FundamentalSelection {
            groups: Vec::new(),
            fields: DEFAULT_FUNDAMENTAL_FIELDS
                .iter()
                .map(|field| (*field).to_string())
                .collect(),
        });
    }

    let mut normalized_groups = Vec::with_capacity(groups.len());
    let mut normalized = Vec::new();
    for group in groups {
        let group = normalize_fundamental_group(&group)?;
        if !normalized_groups.iter().any(|value| value == group) {
            normalized_groups.push(group.to_string());
            for field in fundamental_group_fields(group) {
                push_supported_fundamental_field(&mut normalized, field)?;
            }
        }
    }
    for field in fields {
        let field = field.trim();
        if field.is_empty() {
            return Err(AppError::new(
                ErrorKind::Validation,
                "--field must not be empty",
            ));
        }
        push_supported_fundamental_field(&mut normalized, field)?;
    }

    Ok(FundamentalSelection {
        groups: normalized_groups,
        fields: normalized,
    })
}

fn normalize_fundamental_group(group: &str) -> Result<&'static str, AppError> {
    let group = group.trim();
    if group.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "--group must not be empty",
        ));
    }
    SUPPORTED_FUNDAMENTAL_GROUPS
        .iter()
        .copied()
        .find(|candidate| *candidate == group)
        .ok_or_else(|| {
            AppError::new(
                ErrorKind::Validation,
                format!("Unsupported fundamentals group: {group}"),
            )
            .with_details(json!({
                "supported_groups": SUPPORTED_FUNDAMENTAL_GROUPS,
            }))
        })
}

fn fundamental_group_fields(group: &str) -> &'static [&'static str] {
    match group {
        "earnings" => EARNINGS_FUNDAMENTAL_FIELDS,
        "valuation" => VALUATION_FUNDAMENTAL_FIELDS,
        "dividends" => DIVIDENDS_FUNDAMENTAL_FIELDS,
        "financials" => FINANCIALS_FUNDAMENTAL_FIELDS,
        _ => &[],
    }
}

pub(super) fn fundamental_field_in_group(field: &str, group: &str) -> bool {
    fundamental_group_fields(group).contains(&field)
}

fn push_supported_fundamental_field(
    normalized: &mut Vec<String>,
    field: &str,
) -> Result<(), AppError> {
    let supported = SUPPORTED_FUNDAMENTAL_FIELDS
        .iter()
        .copied()
        .find(|candidate| *candidate == field)
        .ok_or_else(|| {
            AppError::new(
                ErrorKind::Validation,
                format!("Unsupported fundamentals field: {field}"),
            )
            .with_details(json!({ "supported_fields": SUPPORTED_FUNDAMENTAL_FIELDS }))
        })?;
    if !normalized.iter().any(|value| value == supported) {
        normalized.push(supported.to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn normalize_fundamental_fields_uses_curated_defaults() {
        let fields = normalize_fundamental_fields(Vec::new()).unwrap();

        assert!(fields.contains(&"name".to_string()));
        assert!(fields.contains(&"price_earnings_ttm".to_string()));
        assert!(fields.contains(&"earnings_release_next_date".to_string()));
        assert!(fields.contains(&"earnings_release_next_time".to_string()));
    }

    #[test]
    fn normalize_fundamental_fields_rejects_unknown_field() {
        let error = normalize_fundamental_fields(vec!["banana".to_string()]).unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(
            error.details.as_ref().unwrap()["supported_fields"]
                .as_array()
                .unwrap()
                .contains(&json!("earnings_release_next_date"))
        );
    }

    #[test]
    fn normalize_fundamental_selection_expands_groups_before_fields() {
        let selection = normalize_fundamental_selection(
            vec!["earnings".to_string(), "dividends".to_string()],
            vec![
                "price_earnings_ttm".to_string(),
                "earnings_release_next_date".to_string(),
            ],
        )
        .unwrap();

        assert_eq!(selection.groups, vec!["earnings", "dividends"]);
        assert_eq!(selection.fields[0], "earnings_release_next_date");
        assert!(
            selection
                .fields
                .contains(&"earnings_release_next_trading_date_fq".to_string())
        );
        assert!(
            selection
                .fields
                .contains(&"dividend_ex_date_upcoming".to_string())
        );
        assert!(
            selection
                .fields
                .contains(&"dividend_amount_recent".to_string())
        );
        assert!(selection.fields.contains(&"price_earnings_ttm".to_string()));
        assert_eq!(
            selection
                .fields
                .iter()
                .filter(|field| *field == "earnings_release_next_date")
                .count(),
            1
        );
    }

    #[test]
    fn normalize_fundamental_selection_accepts_enriched_event_fields() {
        let selection = normalize_fundamental_selection(
            vec!["earnings".to_string()],
            vec![
                "earnings_release_next_trading_date_fq".to_string(),
                "dividend_amount_recent".to_string(),
            ],
        )
        .unwrap();

        assert_eq!(
            selection
                .fields
                .iter()
                .filter(|field| *field == "earnings_release_next_trading_date_fq")
                .count(),
            1
        );
        assert!(
            selection
                .fields
                .contains(&"earnings_release_trading_date_fq".to_string())
        );
        assert!(
            selection
                .fields
                .contains(&"earnings_release_time".to_string())
        );
        assert!(
            selection
                .fields
                .contains(&"earnings_publication_type_fq".to_string())
        );
        assert!(
            selection
                .fields
                .contains(&"dividend_amount_recent".to_string())
        );
    }

    #[test]
    fn normalize_fundamental_selection_rejects_unknown_group() {
        let error =
            normalize_fundamental_selection(vec!["banana".to_string()], Vec::new()).unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(
            error.details.as_ref().unwrap()["supported_groups"]
                .as_array()
                .unwrap()
                .contains(&json!("earnings"))
        );
    }
}
