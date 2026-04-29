use serde_json::{Value, json};

use tradingview_core::{AppError, ErrorKind};

use super::common::field_values_object;
use super::types::{ScannerRow, ScannerScanResult, ScannerSort};

const SCAN_BASE_URL: &str = "https://scanner.tradingview.com";
const SCAN_SOURCE: &str = "scanner_scan_rest";
const DEFAULT_SCAN_LIMIT: usize = 20;
const MAX_SCAN_LIMIT: usize = 100;
const DEFAULT_SCAN_COLUMNS: &[&str] = &[
    "name",
    "description",
    "close",
    "change",
    "volume",
    "market_cap_basic",
];
const SUPPORTED_SCAN_COLUMNS: &[&str] = &[
    "name",
    "description",
    "close",
    "change",
    "change_abs",
    "volume",
    "average_volume_10d_calc",
    "relative_volume_10d_calc",
    "market_cap_basic",
    "exchange",
    "type",
    "subtype",
    "sector",
    "industry",
    "open",
    "high",
    "low",
    "price_earnings_ttm",
    "earnings_per_share_basic_ttm",
    "dividend_yield_recent",
    "Perf.W",
    "Perf.1M",
    "Perf.3M",
    "RSI",
    "Recommend.All",
    "premarket_open",
    "premarket_high",
    "premarket_low",
    "premarket_close",
    "premarket_change",
    "premarket_change_abs",
    "premarket_gap",
    "premarket_volume",
    "postmarket_open",
    "postmarket_high",
    "postmarket_low",
    "postmarket_close",
    "postmarket_change",
    "postmarket_change_abs",
    "postmarket_volume",
];

#[derive(Debug)]
pub struct ScannerScanRequest {
    pub market: String,
    pub exchanges: Vec<String>,
    pub columns: Option<String>,
    pub sort: Option<String>,
    pub asc: bool,
    pub desc: bool,
    pub limit: Option<usize>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub min_volume: Option<f64>,
    pub min_market_cap: Option<f64>,
    pub sectors: Vec<String>,
    pub industries: Vec<String>,
    pub symbol_types: Vec<String>,
    pub subtypes: Vec<String>,
    pub min_change: Option<f64>,
    pub max_change: Option<f64>,
    pub min_relative_volume: Option<f64>,
    pub max_pe: Option<f64>,
    pub min_average_volume: Option<f64>,
    pub min_performance_week: Option<f64>,
    pub max_performance_week: Option<f64>,
    pub min_performance_month: Option<f64>,
    pub max_performance_month: Option<f64>,
    pub min_performance_quarter: Option<f64>,
    pub max_performance_quarter: Option<f64>,
    pub min_rsi: Option<f64>,
    pub max_rsi: Option<f64>,
    pub min_recommendation: Option<f64>,
    pub max_recommendation: Option<f64>,
}

pub async fn scanner_scan(request: ScannerScanRequest) -> Result<Value, AppError> {
    serde_json::to_value(scanner_scan_typed(request).await?)
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))
}

pub async fn scanner_scan_typed(
    request: ScannerScanRequest,
) -> Result<ScannerScanResult, AppError> {
    let normalized = normalize_scan_request(request)?;
    let url = scan_url(&normalized.market)?;
    let response = reqwest::Client::new()
        .post(url)
        .json(&normalized.body)
        .send()
        .await
        .map_err(|err| AppError::new(ErrorKind::Connection, err.to_string()))?;

    let status = response.status();
    if !status.is_success() {
        return Err(AppError::new(
            ErrorKind::Connection,
            format!("TradingView scanner scan API returned {status}"),
        ));
    }

    let value = response
        .json::<Value>()
        .await
        .map_err(|err| AppError::new(ErrorKind::InternalApiUnavailable, err.to_string()))?;

    normalize_scan_response_typed(&normalized, &value)
}

fn scan_url(market: &str) -> Result<reqwest::Url, AppError> {
    reqwest::Url::parse(&format!("{SCAN_BASE_URL}/{market}/scan"))
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))
}

#[derive(Debug)]
struct NormalizedScannerScanRequest {
    market: String,
    limit: usize,
    columns: Vec<String>,
    sort_field: String,
    sort_order: String,
    filters: Vec<Value>,
    body: Value,
}

fn normalize_scan_request(
    request: ScannerScanRequest,
) -> Result<NormalizedScannerScanRequest, AppError> {
    let market = validate_scan_market(&request.market)?;
    let limit = normalize_scan_limit(request.limit)?;
    let columns = normalize_scan_columns(request.columns.as_deref())?;
    let sort_field = validate_scan_field(
        request.sort.as_deref().unwrap_or("market_cap_basic"),
        "sort",
    )?
    .to_string();
    if request.asc && request.desc {
        return Err(AppError::new(
            ErrorKind::Validation,
            "--asc and --desc cannot be used together",
        ));
    }
    let sort_order = if request.asc { "asc" } else { "desc" }.to_string();
    let filters = scan_filters(&request)?;
    let body = json!({
        "columns": columns,
        "filter": filters,
        "sort": {
            "sortBy": sort_field,
            "sortOrder": sort_order,
        },
        "range": [0, limit],
    });

    Ok(NormalizedScannerScanRequest {
        market,
        limit,
        columns,
        sort_field,
        sort_order,
        filters,
        body,
    })
}

fn validate_scan_market(market: &str) -> Result<String, AppError> {
    let market = market.trim();
    if market == "america" {
        Ok(market.to_string())
    } else {
        Err(AppError::new(
            ErrorKind::Validation,
            format!("Unsupported scanner market: {market}"),
        )
        .with_details(json!({ "supported_markets": ["america"] })))
    }
}

fn normalize_scan_limit(limit: Option<usize>) -> Result<usize, AppError> {
    match limit {
        Some(0) => Err(AppError::new(
            ErrorKind::Validation,
            "--limit must be greater than 0",
        )),
        Some(limit) => Ok(limit.min(MAX_SCAN_LIMIT)),
        None => Ok(DEFAULT_SCAN_LIMIT),
    }
}

fn normalize_scan_columns(columns: Option<&str>) -> Result<Vec<String>, AppError> {
    match columns {
        Some(columns) => {
            let values = columns
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| validate_scan_field(value, "column").map(str::to_string))
                .collect::<Result<Vec<_>, _>>()?;
            if values.is_empty() {
                Err(AppError::new(
                    ErrorKind::Validation,
                    "--columns must include at least one column",
                ))
            } else {
                Ok(values)
            }
        }
        None => Ok(DEFAULT_SCAN_COLUMNS
            .iter()
            .map(|value| value.to_string())
            .collect()),
    }
}

fn validate_scan_field<'a>(field: &'a str, label: &str) -> Result<&'a str, AppError> {
    let field = field.trim();
    SUPPORTED_SCAN_COLUMNS
        .iter()
        .copied()
        .find(|candidate| *candidate == field)
        .ok_or_else(|| {
            AppError::new(
                ErrorKind::Validation,
                format!("Unsupported scanner {label}: {field}"),
            )
            .with_details(json!({ "supported_fields": SUPPORTED_SCAN_COLUMNS }))
        })
}

fn scan_filters(request: &ScannerScanRequest) -> Result<Vec<Value>, AppError> {
    let mut filters = Vec::new();
    let exchanges = request
        .exchanges
        .iter()
        .map(|exchange| exchange.trim())
        .filter(|exchange| !exchange.is_empty())
        .map(|exchange| exchange.to_ascii_uppercase())
        .collect::<Vec<_>>();
    if !exchanges.is_empty() {
        filters.push(json!({
            "left": "exchange",
            "operation": "in_range",
            "right": exchanges,
        }));
    }
    push_string_filter(&mut filters, "sector", &request.sectors, "--sector")?;
    push_string_filter(&mut filters, "industry", &request.industries, "--industry")?;
    push_string_filter(&mut filters, "type", &request.symbol_types, "--type")?;
    push_string_filter(&mut filters, "subtype", &request.subtypes, "--subtype")?;

    push_min_filter(&mut filters, "close", request.min_price, "--min-price")?;
    push_max_filter(&mut filters, "close", request.max_price, "--max-price")?;
    push_min_filter(&mut filters, "volume", request.min_volume, "--min-volume")?;
    push_min_filter(
        &mut filters,
        "average_volume_10d_calc",
        request.min_average_volume,
        "--min-average-volume",
    )?;
    push_min_filter(
        &mut filters,
        "market_cap_basic",
        request.min_market_cap,
        "--min-market-cap",
    )?;
    push_min_signed_filter(&mut filters, "change", request.min_change, "--min-change")?;
    push_max_signed_filter(&mut filters, "change", request.max_change, "--max-change")?;
    push_min_filter(
        &mut filters,
        "relative_volume_10d_calc",
        request.min_relative_volume,
        "--min-relative-volume",
    )?;
    push_max_filter(
        &mut filters,
        "price_earnings_ttm",
        request.max_pe,
        "--max-pe",
    )?;
    push_min_signed_filter(
        &mut filters,
        "Perf.W",
        request.min_performance_week,
        "--min-performance-week",
    )?;
    push_max_signed_filter(
        &mut filters,
        "Perf.W",
        request.max_performance_week,
        "--max-performance-week",
    )?;
    push_min_signed_filter(
        &mut filters,
        "Perf.1M",
        request.min_performance_month,
        "--min-performance-month",
    )?;
    push_max_signed_filter(
        &mut filters,
        "Perf.1M",
        request.max_performance_month,
        "--max-performance-month",
    )?;
    push_min_signed_filter(
        &mut filters,
        "Perf.3M",
        request.min_performance_quarter,
        "--min-performance-quarter",
    )?;
    push_max_signed_filter(
        &mut filters,
        "Perf.3M",
        request.max_performance_quarter,
        "--max-performance-quarter",
    )?;
    push_min_bounded_filter(
        &mut filters,
        "RSI",
        request.min_rsi,
        "--min-rsi",
        0.0,
        100.0,
    )?;
    push_max_bounded_filter(
        &mut filters,
        "RSI",
        request.max_rsi,
        "--max-rsi",
        0.0,
        100.0,
    )?;
    push_min_bounded_filter(
        &mut filters,
        "Recommend.All",
        request.min_recommendation,
        "--min-recommendation",
        -1.0,
        1.0,
    )?;
    push_max_bounded_filter(
        &mut filters,
        "Recommend.All",
        request.max_recommendation,
        "--max-recommendation",
        -1.0,
        1.0,
    )?;
    Ok(filters)
}

fn push_string_filter(
    filters: &mut Vec<Value>,
    field: &str,
    values: &[String],
    label: &str,
) -> Result<(), AppError> {
    let values = values
        .iter()
        .map(|value| value.trim())
        .map(|value| {
            if value.is_empty() {
                Err(AppError::new(
                    ErrorKind::Validation,
                    format!("{label} values must not be empty"),
                ))
            } else {
                Ok(value.to_string())
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    if !values.is_empty() {
        filters.push(json!({
            "left": field,
            "operation": "in_range",
            "right": values,
        }));
    }
    Ok(())
}

fn push_min_filter(
    filters: &mut Vec<Value>,
    field: &str,
    value: Option<f64>,
    label: &str,
) -> Result<(), AppError> {
    if let Some(value) = finite_non_negative(value, label)? {
        filters.push(json!({
            "left": field,
            "operation": "greater",
            "right": value,
        }));
    }
    Ok(())
}

fn push_max_filter(
    filters: &mut Vec<Value>,
    field: &str,
    value: Option<f64>,
    label: &str,
) -> Result<(), AppError> {
    if let Some(value) = finite_non_negative(value, label)? {
        filters.push(json!({
            "left": field,
            "operation": "less",
            "right": value,
        }));
    }
    Ok(())
}

fn push_min_signed_filter(
    filters: &mut Vec<Value>,
    field: &str,
    value: Option<f64>,
    label: &str,
) -> Result<(), AppError> {
    if let Some(value) = finite(value, label)? {
        filters.push(json!({
            "left": field,
            "operation": "greater",
            "right": value,
        }));
    }
    Ok(())
}

fn push_max_signed_filter(
    filters: &mut Vec<Value>,
    field: &str,
    value: Option<f64>,
    label: &str,
) -> Result<(), AppError> {
    if let Some(value) = finite(value, label)? {
        filters.push(json!({
            "left": field,
            "operation": "less",
            "right": value,
        }));
    }
    Ok(())
}

fn push_min_bounded_filter(
    filters: &mut Vec<Value>,
    field: &str,
    value: Option<f64>,
    label: &str,
    min: f64,
    max: f64,
) -> Result<(), AppError> {
    if let Some(value) = finite_in_range(value, label, min, max)? {
        filters.push(json!({
            "left": field,
            "operation": "greater",
            "right": value,
        }));
    }
    Ok(())
}

fn push_max_bounded_filter(
    filters: &mut Vec<Value>,
    field: &str,
    value: Option<f64>,
    label: &str,
    min: f64,
    max: f64,
) -> Result<(), AppError> {
    if let Some(value) = finite_in_range(value, label, min, max)? {
        filters.push(json!({
            "left": field,
            "operation": "less",
            "right": value,
        }));
    }
    Ok(())
}

fn finite_non_negative(value: Option<f64>, label: &str) -> Result<Option<f64>, AppError> {
    if let Some(value) = finite(value, label)? {
        if value < 0.0 {
            return Err(AppError::new(
                ErrorKind::Validation,
                format!("{label} must be non-negative"),
            ));
        }
        Ok(Some(value))
    } else {
        Ok(None)
    }
}

fn finite_in_range(
    value: Option<f64>,
    label: &str,
    min: f64,
    max: f64,
) -> Result<Option<f64>, AppError> {
    if let Some(value) = finite(value, label)? {
        if !(min..=max).contains(&value) {
            return Err(AppError::new(
                ErrorKind::Validation,
                format!("{label} must be between {min} and {max}"),
            ));
        }
        Ok(Some(value))
    } else {
        Ok(None)
    }
}

fn finite(value: Option<f64>, label: &str) -> Result<Option<f64>, AppError> {
    if let Some(value) = value {
        if !value.is_finite() {
            return Err(AppError::new(
                ErrorKind::Validation,
                format!("{label} must be finite"),
            ));
        }
        Ok(Some(value))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
fn normalize_scan_response(
    request: &NormalizedScannerScanRequest,
    value: &Value,
) -> Result<Value, AppError> {
    serde_json::to_value(normalize_scan_response_typed(request, value)?)
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))
}

fn normalize_scan_response_typed(
    request: &NormalizedScannerScanRequest,
    value: &Value,
) -> Result<ScannerScanResult, AppError> {
    let object = value
        .as_object()
        .ok_or_else(|| malformed_scan("response"))?;
    let symbols = object
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| malformed_scan("data"))?;
    let total_count = object
        .get("totalCount")
        .and_then(Value::as_u64)
        .map(Value::from)
        .unwrap_or(Value::Null);
    let normalized_symbols = symbols
        .iter()
        .take(request.limit)
        .map(|row| normalize_scan_symbol(row, &request.columns))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ScannerScanResult {
        source: SCAN_SOURCE.to_string(),
        market: request.market.clone(),
        limit: request.limit,
        count: normalized_symbols.len(),
        total_count,
        columns: request.columns.clone(),
        sort: ScannerSort {
            field: request.sort_field.clone(),
            order: request.sort_order.clone(),
        },
        filters: request.filters.clone(),
        symbols: normalized_symbols,
    })
}

fn normalize_scan_symbol(row: &Value, columns: &[String]) -> Result<ScannerRow, AppError> {
    let object = row
        .as_object()
        .ok_or_else(|| malformed_scan("symbol row"))?;
    let symbol = object
        .get("s")
        .and_then(Value::as_str)
        .filter(|symbol| !symbol.trim().is_empty())
        .ok_or_else(|| malformed_scan("symbol row s"))?;
    let values = object
        .get("d")
        .and_then(Value::as_array)
        .ok_or_else(|| malformed_scan("symbol row d"))?;
    let field_values = field_values_object(columns, values);

    Ok(ScannerRow {
        symbol: symbol.to_string(),
        values: values.clone(),
        field_values: Value::Object(field_values),
    })
}

fn malformed_scan(label: &str) -> AppError {
    AppError::new(
        ErrorKind::InternalApiUnavailable,
        format!("Unexpected TradingView scanner scan response shape at {label}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_scan_request_uses_defaults_and_builds_body() {
        let request = ScannerScanRequest {
            market: "america".to_string(),
            exchanges: Vec::new(),
            columns: None,
            sort: None,
            asc: false,
            desc: false,
            limit: None,
            min_price: None,
            max_price: None,
            min_volume: None,
            min_market_cap: None,
            sectors: Vec::new(),
            industries: Vec::new(),
            symbol_types: Vec::new(),
            subtypes: Vec::new(),
            min_change: None,
            max_change: None,
            min_relative_volume: None,
            max_pe: None,
            min_average_volume: None,
            min_performance_week: None,
            max_performance_week: None,
            min_performance_month: None,
            max_performance_month: None,
            min_performance_quarter: None,
            max_performance_quarter: None,
            min_rsi: None,
            max_rsi: None,
            min_recommendation: None,
            max_recommendation: None,
        };

        let normalized = normalize_scan_request(request).unwrap();

        assert_eq!(normalized.market, "america");
        assert_eq!(normalized.limit, 20);
        assert_eq!(normalized.sort_field, "market_cap_basic");
        assert_eq!(normalized.sort_order, "desc");
        assert_eq!(normalized.columns, DEFAULT_SCAN_COLUMNS);
        assert_eq!(normalized.body["range"], json!([0, 20]));
        assert_eq!(
            normalized.body["columns"],
            json!([
                "name",
                "description",
                "close",
                "change",
                "volume",
                "market_cap_basic"
            ])
        );
    }

    #[test]
    fn normalize_scan_request_builds_exchange_and_numeric_filters() {
        let request = ScannerScanRequest {
            market: "america".to_string(),
            exchanges: vec!["nasdaq".to_string(), "NYSE".to_string()],
            columns: Some("name,close,volume".to_string()),
            sort: Some("volume".to_string()),
            asc: true,
            desc: false,
            limit: Some(150),
            min_price: Some(10.0),
            max_price: Some(500.0),
            min_volume: Some(1_000_000.0),
            min_market_cap: Some(10_000_000_000.0),
            sectors: Vec::new(),
            industries: Vec::new(),
            symbol_types: Vec::new(),
            subtypes: Vec::new(),
            min_change: None,
            max_change: None,
            min_relative_volume: None,
            max_pe: None,
            min_average_volume: None,
            min_performance_week: None,
            max_performance_week: None,
            min_performance_month: None,
            max_performance_month: None,
            min_performance_quarter: None,
            max_performance_quarter: None,
            min_rsi: None,
            max_rsi: None,
            min_recommendation: None,
            max_recommendation: None,
        };

        let normalized = normalize_scan_request(request).unwrap();

        assert_eq!(normalized.limit, 100);
        assert_eq!(normalized.columns, ["name", "close", "volume"]);
        assert_eq!(normalized.sort_field, "volume");
        assert_eq!(normalized.sort_order, "asc");
        assert_eq!(normalized.filters.len(), 5);
        assert_eq!(
            normalized.filters[0],
            json!({
                "left": "exchange",
                "operation": "in_range",
                "right": ["NASDAQ", "NYSE"]
            })
        );
        assert_eq!(normalized.body["filter"], json!(normalized.filters));
    }

    #[test]
    fn normalize_scan_request_builds_string_and_extra_numeric_filters() {
        let request = ScannerScanRequest {
            market: "america".to_string(),
            exchanges: Vec::new(),
            columns: Some("name,type,subtype,sector,relative_volume_10d_calc".to_string()),
            sort: Some("relative_volume_10d_calc".to_string()),
            asc: false,
            desc: true,
            limit: Some(10),
            min_price: None,
            max_price: None,
            min_volume: None,
            min_market_cap: None,
            sectors: vec![
                "Technology Services".to_string(),
                "Electronic Technology".to_string(),
            ],
            industries: vec!["Packaged Software".to_string()],
            symbol_types: vec!["stock".to_string()],
            subtypes: vec!["common".to_string()],
            min_change: Some(2.0),
            max_change: Some(20.0),
            min_relative_volume: Some(1.5),
            max_pe: Some(50.0),
            min_average_volume: None,
            min_performance_week: None,
            max_performance_week: None,
            min_performance_month: None,
            max_performance_month: None,
            min_performance_quarter: None,
            max_performance_quarter: None,
            min_rsi: None,
            max_rsi: None,
            min_recommendation: None,
            max_recommendation: None,
        };

        let normalized = normalize_scan_request(request).unwrap();

        assert_eq!(
            normalized.columns,
            [
                "name",
                "type",
                "subtype",
                "sector",
                "relative_volume_10d_calc"
            ]
        );
        assert_eq!(normalized.sort_field, "relative_volume_10d_calc");
        assert_eq!(normalized.sort_order, "desc");
        assert_eq!(normalized.filters.len(), 8);
        assert_eq!(
            normalized.filters[0],
            json!({
                "left": "sector",
                "operation": "in_range",
                "right": ["Technology Services", "Electronic Technology"]
            })
        );
        assert_eq!(
            normalized.filters[2],
            json!({
                "left": "type",
                "operation": "in_range",
                "right": ["stock"]
            })
        );
        assert_eq!(
            normalized.filters[6],
            json!({
                "left": "relative_volume_10d_calc",
                "operation": "greater",
                "right": 1.5
            })
        );
        assert_eq!(
            normalized.filters[7],
            json!({
                "left": "price_earnings_ttm",
                "operation": "less",
                "right": 50.0
            })
        );
    }

    #[test]
    fn normalize_scan_request_builds_technical_filters_with_signed_values() {
        let request = ScannerScanRequest {
            market: "america".to_string(),
            exchanges: Vec::new(),
            columns: Some("name,average_volume_10d_calc,Perf.W,RSI,Recommend.All".to_string()),
            sort: Some("Perf.W".to_string()),
            asc: false,
            desc: true,
            limit: Some(10),
            min_price: None,
            max_price: None,
            min_volume: None,
            min_market_cap: None,
            sectors: Vec::new(),
            industries: Vec::new(),
            symbol_types: vec!["stock".to_string()],
            subtypes: Vec::new(),
            min_change: None,
            max_change: Some(-5.0),
            min_relative_volume: None,
            max_pe: None,
            min_average_volume: Some(1_000_000.0),
            min_performance_week: Some(5.0),
            max_performance_week: None,
            min_performance_month: None,
            max_performance_month: Some(-10.0),
            min_performance_quarter: None,
            max_performance_quarter: None,
            min_rsi: None,
            max_rsi: Some(70.0),
            min_recommendation: Some(0.1),
            max_recommendation: None,
        };

        let normalized = normalize_scan_request(request).unwrap();

        assert_eq!(
            normalized.columns,
            [
                "name",
                "average_volume_10d_calc",
                "Perf.W",
                "RSI",
                "Recommend.All"
            ]
        );
        assert_eq!(normalized.sort_field, "Perf.W");
        assert_eq!(normalized.filters.len(), 7);
        assert_eq!(
            normalized.filters[1],
            json!({
                "left": "average_volume_10d_calc",
                "operation": "greater",
                "right": 1_000_000.0
            })
        );
        assert_eq!(
            normalized.filters[2],
            json!({
                "left": "change",
                "operation": "less",
                "right": -5.0
            })
        );
        assert_eq!(
            normalized.filters[3],
            json!({
                "left": "Perf.W",
                "operation": "greater",
                "right": 5.0
            })
        );
        assert_eq!(
            normalized.filters[4],
            json!({
                "left": "Perf.1M",
                "operation": "less",
                "right": -10.0
            })
        );
        assert_eq!(
            normalized.filters[5],
            json!({
                "left": "RSI",
                "operation": "less",
                "right": 70.0
            })
        );
        assert_eq!(
            normalized.filters[6],
            json!({
                "left": "Recommend.All",
                "operation": "greater",
                "right": 0.1
            })
        );
    }

    #[test]
    fn normalize_scan_request_accepts_extended_hours_columns_without_changing_defaults() {
        let request = ScannerScanRequest {
            market: "america".to_string(),
            exchanges: Vec::new(),
            columns: Some(
                "name,close,premarket_close,premarket_volume,postmarket_close,postmarket_volume"
                    .to_string(),
            ),
            sort: Some("premarket_volume".to_string()),
            asc: false,
            desc: true,
            limit: Some(5),
            min_price: None,
            max_price: None,
            min_volume: None,
            min_market_cap: None,
            sectors: Vec::new(),
            industries: Vec::new(),
            symbol_types: Vec::new(),
            subtypes: Vec::new(),
            min_change: None,
            max_change: None,
            min_relative_volume: None,
            max_pe: None,
            min_average_volume: None,
            min_performance_week: None,
            max_performance_week: None,
            min_performance_month: None,
            max_performance_month: None,
            min_performance_quarter: None,
            max_performance_quarter: None,
            min_rsi: None,
            max_rsi: None,
            min_recommendation: None,
            max_recommendation: None,
        };

        let normalized = normalize_scan_request(request).unwrap();

        assert_eq!(
            normalized.columns,
            [
                "name",
                "close",
                "premarket_close",
                "premarket_volume",
                "postmarket_close",
                "postmarket_volume"
            ]
        );
        assert_eq!(normalized.sort_field, "premarket_volume");
        assert_eq!(
            normalized.body["columns"],
            json!([
                "name",
                "close",
                "premarket_close",
                "premarket_volume",
                "postmarket_close",
                "postmarket_volume"
            ])
        );

        let defaults = normalize_scan_request(ScannerScanRequest {
            market: "america".to_string(),
            exchanges: Vec::new(),
            columns: None,
            sort: None,
            asc: false,
            desc: false,
            limit: None,
            min_price: None,
            max_price: None,
            min_volume: None,
            min_market_cap: None,
            sectors: Vec::new(),
            industries: Vec::new(),
            symbol_types: Vec::new(),
            subtypes: Vec::new(),
            min_change: None,
            max_change: None,
            min_relative_volume: None,
            max_pe: None,
            min_average_volume: None,
            min_performance_week: None,
            max_performance_week: None,
            min_performance_month: None,
            max_performance_month: None,
            min_performance_quarter: None,
            max_performance_quarter: None,
            min_rsi: None,
            max_rsi: None,
            min_recommendation: None,
            max_recommendation: None,
        })
        .unwrap();
        assert_eq!(defaults.columns, DEFAULT_SCAN_COLUMNS);
    }

    #[test]
    fn normalize_scan_request_rejects_invalid_inputs() {
        let base = || ScannerScanRequest {
            market: "america".to_string(),
            exchanges: Vec::new(),
            columns: None,
            sort: None,
            asc: false,
            desc: false,
            limit: None,
            min_price: None,
            max_price: None,
            min_volume: None,
            min_market_cap: None,
            sectors: Vec::new(),
            industries: Vec::new(),
            symbol_types: Vec::new(),
            subtypes: Vec::new(),
            min_change: None,
            max_change: None,
            min_relative_volume: None,
            max_pe: None,
            min_average_volume: None,
            min_performance_week: None,
            max_performance_week: None,
            min_performance_month: None,
            max_performance_month: None,
            min_performance_quarter: None,
            max_performance_quarter: None,
            min_rsi: None,
            max_rsi: None,
            min_recommendation: None,
            max_recommendation: None,
        };

        let mut invalid_market = base();
        invalid_market.market = "global".to_string();
        assert_eq!(
            normalize_scan_request(invalid_market).unwrap_err().kind,
            ErrorKind::Validation
        );

        let mut invalid_column = base();
        invalid_column.columns = Some("name,unknown".to_string());
        assert_eq!(
            normalize_scan_request(invalid_column).unwrap_err().kind,
            ErrorKind::Validation
        );

        let mut invalid_sort = base();
        invalid_sort.sort = Some("unknown".to_string());
        assert_eq!(
            normalize_scan_request(invalid_sort).unwrap_err().kind,
            ErrorKind::Validation
        );

        let mut invalid_order = base();
        invalid_order.asc = true;
        invalid_order.desc = true;
        assert_eq!(
            normalize_scan_request(invalid_order).unwrap_err().kind,
            ErrorKind::Validation
        );

        let mut invalid_limit = base();
        invalid_limit.limit = Some(0);
        assert_eq!(
            normalize_scan_request(invalid_limit).unwrap_err().kind,
            ErrorKind::Validation
        );

        let mut invalid_number = base();
        invalid_number.min_price = Some(f64::NAN);
        assert_eq!(
            normalize_scan_request(invalid_number).unwrap_err().kind,
            ErrorKind::Validation
        );

        let mut invalid_non_negative = base();
        invalid_non_negative.min_average_volume = Some(-1.0);
        assert_eq!(
            normalize_scan_request(invalid_non_negative)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );

        let mut invalid_rsi = base();
        invalid_rsi.max_rsi = Some(101.0);
        assert_eq!(
            normalize_scan_request(invalid_rsi).unwrap_err().kind,
            ErrorKind::Validation
        );

        let mut invalid_recommendation = base();
        invalid_recommendation.min_recommendation = Some(-1.1);
        assert_eq!(
            normalize_scan_request(invalid_recommendation)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );

        let mut invalid_string = base();
        invalid_string.sectors = vec![" ".to_string()];
        assert_eq!(
            normalize_scan_request(invalid_string).unwrap_err().kind,
            ErrorKind::Validation
        );
    }

    #[test]
    fn normalize_scan_response_maps_compact_rows() {
        let request = normalize_scan_request(ScannerScanRequest {
            market: "america".to_string(),
            exchanges: Vec::new(),
            columns: Some("name,close,volume".to_string()),
            sort: Some("volume".to_string()),
            asc: false,
            desc: true,
            limit: Some(1),
            min_price: None,
            max_price: None,
            min_volume: None,
            min_market_cap: None,
            sectors: Vec::new(),
            industries: Vec::new(),
            symbol_types: Vec::new(),
            subtypes: Vec::new(),
            min_change: None,
            max_change: None,
            min_relative_volume: None,
            max_pe: None,
            min_average_volume: None,
            min_performance_week: None,
            max_performance_week: None,
            min_performance_month: None,
            max_performance_month: None,
            min_performance_quarter: None,
            max_performance_quarter: None,
            min_rsi: None,
            max_rsi: None,
            min_recommendation: None,
            max_recommendation: None,
        })
        .unwrap();
        let payload = json!({
            "totalCount": 2,
            "data": [
                { "s": "NASDAQ:AAPL", "d": ["AAPL", 200.0, 123456] },
                { "s": "NASDAQ:MSFT", "d": ["MSFT", 300.0, 654321] }
            ]
        });

        let result = normalize_scan_response(&request, &payload).unwrap();

        assert_eq!(result["source"], "scanner_scan_rest");
        assert_eq!(result["market"], "america");
        assert_eq!(result["count"], 1);
        assert_eq!(result["total_count"], 2);
        assert_eq!(result["columns"], json!(["name", "close", "volume"]));
        assert_eq!(result["sort"], json!({"field": "volume", "order": "desc"}));
        assert_eq!(result["symbols"][0]["symbol"], "NASDAQ:AAPL");
        assert_eq!(
            result["symbols"][0]["values"],
            json!(["AAPL", 200.0, 123456])
        );
        assert_eq!(result["symbols"][0]["field_values"]["close"], 200.0);
    }

    #[test]
    fn normalize_scan_response_typed_preserves_columns_and_field_values() {
        let request = normalize_scan_request(ScannerScanRequest {
            market: "america".to_string(),
            exchanges: Vec::new(),
            columns: Some("name,close,premarket_close".to_string()),
            sort: Some("close".to_string()),
            asc: true,
            desc: false,
            limit: Some(3),
            min_price: None,
            max_price: None,
            min_volume: None,
            min_market_cap: None,
            sectors: Vec::new(),
            industries: Vec::new(),
            symbol_types: Vec::new(),
            subtypes: Vec::new(),
            min_change: None,
            max_change: None,
            min_relative_volume: None,
            max_pe: None,
            min_average_volume: None,
            min_performance_week: None,
            max_performance_week: None,
            min_performance_month: None,
            max_performance_month: None,
            min_performance_quarter: None,
            max_performance_quarter: None,
            min_rsi: None,
            max_rsi: None,
            min_recommendation: None,
            max_recommendation: None,
        })
        .unwrap();
        let payload = json!({
            "totalCount": 1,
            "data": [
                { "s": "NASDAQ:AAPL", "d": ["AAPL", 266.39, 268.2] }
            ]
        });

        let result = normalize_scan_response_typed(&request, &payload).unwrap();

        assert_eq!(result.market, "america");
        assert_eq!(result.columns, ["name", "close", "premarket_close"]);
        assert_eq!(result.sort.field, "close");
        assert_eq!(result.sort.order, "asc");
        assert_eq!(result.symbols[0].symbol, "NASDAQ:AAPL");
        assert_eq!(result.symbols[0].field_values["premarket_close"], 268.2);
    }

    #[test]
    fn normalize_scan_response_rejects_malformed_shapes() {
        let request = normalize_scan_request(ScannerScanRequest {
            market: "america".to_string(),
            exchanges: Vec::new(),
            columns: None,
            sort: None,
            asc: false,
            desc: false,
            limit: None,
            min_price: None,
            max_price: None,
            min_volume: None,
            min_market_cap: None,
            sectors: Vec::new(),
            industries: Vec::new(),
            symbol_types: Vec::new(),
            subtypes: Vec::new(),
            min_change: None,
            max_change: None,
            min_relative_volume: None,
            max_pe: None,
            min_average_volume: None,
            min_performance_week: None,
            max_performance_week: None,
            min_performance_month: None,
            max_performance_month: None,
            min_performance_quarter: None,
            max_performance_quarter: None,
            min_rsi: None,
            max_rsi: None,
            min_recommendation: None,
            max_recommendation: None,
        })
        .unwrap();

        let missing_data = json!({ "totalCount": 1 });
        assert_eq!(
            normalize_scan_response(&request, &missing_data)
                .unwrap_err()
                .kind,
            ErrorKind::InternalApiUnavailable
        );

        let missing_values = json!({ "data": [{ "s": "NASDAQ:AAPL" }] });
        assert_eq!(
            normalize_scan_response(&request, &missing_values)
                .unwrap_err()
                .kind,
            ErrorKind::InternalApiUnavailable
        );
    }
}
