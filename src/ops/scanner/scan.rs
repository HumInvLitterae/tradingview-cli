use serde_json::{Value, json};

use crate::error::{AppError, ErrorKind};

use super::common::field_values_object;

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
    "relative_volume_10d_calc",
    "market_cap_basic",
    "exchange",
    "sector",
    "industry",
    "open",
    "high",
    "low",
    "price_earnings_ttm",
    "earnings_per_share_basic_ttm",
    "dividend_yield_recent",
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
}

pub async fn scanner_scan(request: ScannerScanRequest) -> Result<Value, AppError> {
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

    normalize_scan_response(&normalized, &value)
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

    push_min_filter(&mut filters, "close", request.min_price, "--min-price")?;
    push_max_filter(&mut filters, "close", request.max_price, "--max-price")?;
    push_min_filter(&mut filters, "volume", request.min_volume, "--min-volume")?;
    push_min_filter(
        &mut filters,
        "market_cap_basic",
        request.min_market_cap,
        "--min-market-cap",
    )?;
    Ok(filters)
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

fn finite_non_negative(value: Option<f64>, label: &str) -> Result<Option<f64>, AppError> {
    if let Some(value) = value {
        if !value.is_finite() {
            return Err(AppError::new(
                ErrorKind::Validation,
                format!("{label} must be finite"),
            ));
        }
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

fn normalize_scan_response(
    request: &NormalizedScannerScanRequest,
    value: &Value,
) -> Result<Value, AppError> {
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

    Ok(json!({
        "source": SCAN_SOURCE,
        "market": request.market,
        "limit": request.limit,
        "count": normalized_symbols.len(),
        "total_count": total_count,
        "columns": request.columns,
        "sort": {
            "field": request.sort_field,
            "order": request.sort_order,
        },
        "filters": request.filters,
        "symbols": normalized_symbols,
    }))
}

fn normalize_scan_symbol(row: &Value, columns: &[String]) -> Result<Value, AppError> {
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

    Ok(json!({
        "symbol": symbol,
        "values": values,
        "field_values": field_values,
    }))
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
