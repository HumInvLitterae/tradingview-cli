use std::{
    collections::HashSet,
    future::Future,
    pin::Pin,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use reqwest::Client;
use serde_json::{Value, json};

use tradingview_core::{AppError, ErrorKind};

use super::common::field_values_object;
use super::http::{configured_client, map_http_error, remote_status_error};
use super::types::{
    ScannerAggregateScanResult, ScannerPageScanResult, ScannerRow, ScannerScanResult, ScannerSort,
};

const SCAN_BASE_URL: &str = "https://scanner.tradingview.com";
const SCAN_SOURCE: &str = "scanner_scan_rest";
const DESKTOP_FREE_READ_CATEGORY: &str = "desktop_free_read";
const DEFAULT_SCAN_LIMIT: usize = 20;
const MAX_SCAN_LIMIT: usize = 100;
const MAX_AGGREGATE_RESULTS: usize = 10_000;
const MAX_AGGREGATE_PAGES: usize = 100;
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
    "earnings_release_next_date",
    "earnings_release_date",
    "earnings_release_next_time",
    "earnings_release_next_calendar_date",
    "earnings_release_next_trading_date_fq",
    "earnings_release_trading_date_fq",
    "earnings_release_time",
    "earnings_publication_type_next_fq",
    "earnings_publication_type_fq",
    "dividend_amount_recent",
    "dividend_amount_upcoming",
    "dividend_frequency_recent",
    "dividend_frequency_upcoming",
    "next_dividend_date",
    "expected_annual_dividends",
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

#[derive(Debug, Clone)]
/// Request for a Desktop-free scanner table read.
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

#[derive(Debug, Clone)]
/// Request for one scanner page starting at an explicit provider offset.
pub struct ScannerPageScanRequest {
    pub scan: ScannerScanRequest,
    pub offset: usize,
}

#[derive(Debug, Clone)]
/// Request for a bounded sequence of Desktop-free scanner pages.
pub struct ScannerAggregateScanRequest {
    pub scan: ScannerScanRequest,
    pub page_size: Option<usize>,
    pub max_results: usize,
}

pub async fn scanner_scan(request: ScannerScanRequest) -> Result<Value, AppError> {
    serde_json::to_value(scanner_scan_typed(request).await?)
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))
}

pub async fn scanner_scan_aggregate(
    request: ScannerAggregateScanRequest,
) -> Result<Value, AppError> {
    serde_json::to_value(scanner_scan_aggregate_typed(request).await?)
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))
}

/// Reads a scanner table without connecting to TradingView Desktop.
///
/// This is the typed Rust API. Use [`scanner_scan`] only when preserving the
/// CLI-compatible JSON payload shape is required.
pub async fn scanner_scan_typed(
    request: ScannerScanRequest,
) -> Result<ScannerScanResult, AppError> {
    let normalized = normalize_scan_request(request)?;
    let client = configured_client()?;
    Ok(fetch_scanner_page_typed(&client, &normalized).await?.result)
}

pub async fn scanner_scan_page(request: ScannerPageScanRequest) -> Result<Value, AppError> {
    serde_json::to_value(scanner_scan_page_typed(request).await?)
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))
}

pub async fn scanner_scan_page_typed(
    request: ScannerPageScanRequest,
) -> Result<ScannerPageScanResult, AppError> {
    let normalized = normalize_scan_request_at_offset(request.scan, request.offset)?;
    let client = configured_client()?;
    let page = fetch_scanner_page_typed(&client, &normalized).await?.result;
    Ok(ScannerPageScanResult {
        offset: normalized.offset,
        page,
    })
}

async fn fetch_scanner_page_typed(
    client: &Client,
    normalized: &NormalizedScannerScanRequest,
) -> Result<NormalizedScannerPage, AppError> {
    let url = scan_url(&normalized.market)?;
    let response = client
        .post(url)
        .json(&normalized.body)
        .send()
        .await
        .map_err(|err| map_http_error(err, "Scanner scan request"))?;

    let status = response.status();
    if !status.is_success() {
        return Err(remote_status_error("TradingView scanner scan API", status));
    }

    let value = response
        .json::<Value>()
        .await
        .map_err(|err| map_http_error(err, "Scanner scan response"))?;

    normalize_scan_response_page(normalized, &value)
}

/// Reads a complete bounded scanner population through sequential pages.
pub async fn scanner_scan_aggregate_typed(
    request: ScannerAggregateScanRequest,
) -> Result<ScannerAggregateScanResult, AppError> {
    let page_size = validate_aggregate_bounds(request.max_results, request.page_size)?;
    if request.scan.limit.is_some() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "aggregate scanner reads cannot use --limit",
        ));
    }

    let mut base = request.scan;
    base.limit = Some(page_size);
    let normalized = normalize_scan_request(base)?;
    let client = Arc::new(configured_client()?);
    let started_at_epoch_seconds = epoch_seconds()?;
    scanner_scan_aggregate_with(
        Arc::clone(&client),
        normalized,
        request.max_results,
        started_at_epoch_seconds,
        |client, page| Box::pin(async move { fetch_scanner_page_typed(&client, page).await }),
    )
    .await
}

fn validate_aggregate_bounds(
    max_results: usize,
    requested_page_size: Option<usize>,
) -> Result<usize, AppError> {
    if max_results == 0 || max_results > MAX_AGGREGATE_RESULTS {
        return Err(AppError::new(
            ErrorKind::Validation,
            format!("--max-results must be between 1 and {MAX_AGGREGATE_RESULTS}"),
        ));
    }
    let page_size = requested_page_size.unwrap_or(MAX_SCAN_LIMIT);
    if page_size == 0 || page_size > MAX_SCAN_LIMIT {
        return Err(AppError::new(
            ErrorKind::Validation,
            format!("--page-size must be between 1 and {MAX_SCAN_LIMIT}"),
        ));
    }
    if max_results.div_ceil(page_size) > MAX_AGGREGATE_PAGES {
        return Err(AppError::new(
            ErrorKind::Validation,
            format!(
                "--max-results and --page-size may require at most {MAX_AGGREGATE_PAGES} pages"
            ),
        ));
    }
    Ok(page_size)
}

async fn scanner_scan_aggregate_with<C, F>(
    context: C,
    mut normalized: NormalizedScannerScanRequest,
    max_results: usize,
    started_at_epoch_seconds: u64,
    mut fetch: F,
) -> Result<ScannerAggregateScanResult, AppError>
where
    C: Clone,
    F: for<'a> FnMut(
        C,
        &'a NormalizedScannerScanRequest,
    )
        -> Pin<Box<dyn Future<Output = Result<NormalizedScannerPage, AppError>> + 'a>>,
{
    let page_size = normalized.limit;
    let query_fingerprint = scan_query_fingerprint(&normalized);
    let mut rows = Vec::new();
    let mut seen = HashSet::new();
    let mut raw_count = 0usize;
    let mut duplicate_count = 0usize;
    let mut pages_fetched = 0usize;
    let mut totals = Vec::new();

    loop {
        let page = fetch(context.clone(), &normalized).await?;
        pages_fetched += 1;
        let total = page
            .result
            .total_count
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| malformed_scan("aggregate totalCount"))?;
        if total > max_results {
            return Err(AppError::new(
                ErrorKind::Validation,
                "Scanner population exceeds --max-results",
            )
            .with_details(json!({
                "max_results": max_results,
                "observed_total_count": total,
            })));
        }
        totals.push(total);

        let expected_page_count = page_size.min(total.saturating_sub(normalized.offset));
        if page.provider_row_count != expected_page_count {
            return Err(AppError::new(
                ErrorKind::InternalApiUnavailable,
                "TradingView scanner returned an incomplete page before bounded completion",
            )
            .with_details(json!({
                "offset": normalized.offset,
                "page_size": page_size,
                "observed_total_count": total,
                "expected_page_count": expected_page_count,
                "observed_page_count": page.provider_row_count,
            })));
        }

        raw_count = raw_count
            .checked_add(page.provider_row_count)
            .ok_or_else(|| {
                AppError::new(ErrorKind::Internal, "Scanner aggregate row count overflow")
            })?;
        for row in page.result.symbols {
            if seen.insert(row.symbol.clone()) {
                rows.push(row);
            } else {
                duplicate_count += 1;
            }
        }

        let next_offset = normalized.offset.checked_add(page_size).ok_or_else(|| {
            AppError::new(ErrorKind::Internal, "Scanner aggregate offset overflow")
        })?;
        if next_offset >= total {
            break;
        }
        normalized.offset = next_offset;
        normalized.body["range"] = json!([next_offset, next_offset + page_size]);
    }

    let first_total_count = totals.first().copied().unwrap_or(0);
    let last_total_count = totals.last().copied().unwrap_or(0);
    let maximum_total_count = totals.iter().copied().max().unwrap_or(0);
    let completed_at_epoch_seconds = epoch_seconds()?.max(started_at_epoch_seconds);
    Ok(ScannerAggregateScanResult {
        source: SCAN_SOURCE.to_string(),
        source_category: DESKTOP_FREE_READ_CATEGORY.to_string(),
        requires_desktop: false,
        non_mutating: true,
        market: normalized.market,
        page_size,
        max_results,
        count: rows.len(),
        raw_count,
        duplicate_count,
        pages_fetched,
        first_total_count,
        last_total_count,
        maximum_total_count,
        query_fingerprint,
        started_at_epoch_seconds,
        completed_at_epoch_seconds,
        total_count_changed: totals.iter().any(|total| *total != first_total_count),
        duplicates_observed: duplicate_count > 0,
        sequential_observation: true,
        columns: normalized.columns,
        sort: ScannerSort {
            field: normalized.sort_field,
            order: normalized.sort_order,
        },
        filters: normalized.filters,
        symbols: rows,
    })
}

fn scan_query_fingerprint(request: &NormalizedScannerScanRequest) -> String {
    let query = json!({
        "market": request.market,
        "columns": request.columns,
        "sort": { "field": request.sort_field, "order": request.sort_order },
        "filters": request.filters,
    })
    .to_string();
    let hash = query
        .as_bytes()
        .iter()
        .fold(0xcbf29ce484222325u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
        });
    format!("fnv1a64:{hash:016x}")
}

fn epoch_seconds() -> Result<u64, AppError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| AppError::new(ErrorKind::Internal, "System clock is before Unix epoch"))
}

fn scan_url(market: &str) -> Result<reqwest::Url, AppError> {
    reqwest::Url::parse(&format!("{SCAN_BASE_URL}/{market}/scan"))
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))
}

#[derive(Debug)]
struct NormalizedScannerScanRequest {
    market: String,
    limit: usize,
    offset: usize,
    columns: Vec<String>,
    sort_field: String,
    sort_order: String,
    filters: Vec<Value>,
    body: Value,
}

fn normalize_scan_request(
    request: ScannerScanRequest,
) -> Result<NormalizedScannerScanRequest, AppError> {
    normalize_scan_request_at_offset(request, 0)
}

fn normalize_scan_request_at_offset(
    request: ScannerScanRequest,
    offset: usize,
) -> Result<NormalizedScannerScanRequest, AppError> {
    let market = validate_scan_market(&request.market)?;
    let limit = normalize_scan_limit(request.limit)?;
    let range_end = offset.checked_add(limit).ok_or_else(|| {
        AppError::new(ErrorKind::Validation, "--offset plus --limit is too large")
    })?;
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
        "range": [offset, range_end],
    });

    Ok(NormalizedScannerScanRequest {
        market,
        limit,
        offset,
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

#[cfg(test)]
fn normalize_scan_response_typed(
    request: &NormalizedScannerScanRequest,
    value: &Value,
) -> Result<ScannerScanResult, AppError> {
    Ok(normalize_scan_response_page(request, value)?.result)
}

struct NormalizedScannerPage {
    result: ScannerScanResult,
    provider_row_count: usize,
}

fn normalize_scan_response_page(
    request: &NormalizedScannerScanRequest,
    value: &Value,
) -> Result<NormalizedScannerPage, AppError> {
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

    Ok(NormalizedScannerPage {
        provider_row_count: symbols.len(),
        result: ScannerScanResult {
            source: SCAN_SOURCE.to_string(),
            source_category: DESKTOP_FREE_READ_CATEGORY.to_string(),
            requires_desktop: false,
            non_mutating: true,
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
        },
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
    use std::{cell::RefCell, collections::VecDeque, rc::Rc};

    use super::*;

    fn aggregate_base_request(page_size: usize) -> NormalizedScannerScanRequest {
        normalize_scan_request(ScannerScanRequest {
            market: "america".to_string(),
            exchanges: Vec::new(),
            columns: Some("name,close".to_string()),
            sort: Some("name".to_string()),
            asc: true,
            desc: false,
            limit: Some(page_size),
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
        .unwrap()
    }

    fn scanner_page(total: Value, symbols: &[&str]) -> ScannerScanResult {
        ScannerScanResult {
            source: SCAN_SOURCE.to_string(),
            source_category: DESKTOP_FREE_READ_CATEGORY.to_string(),
            requires_desktop: false,
            non_mutating: true,
            market: "america".to_string(),
            limit: 2,
            count: symbols.len(),
            total_count: total,
            columns: vec!["name".to_string(), "close".to_string()],
            sort: ScannerSort {
                field: "name".to_string(),
                order: "asc".to_string(),
            },
            filters: Vec::new(),
            symbols: symbols
                .iter()
                .map(|symbol| ScannerRow {
                    symbol: (*symbol).to_string(),
                    values: vec![json!(symbol), json!(1.0)],
                    field_values: json!({ "name": symbol, "close": 1.0 }),
                })
                .collect(),
        }
    }

    fn aggregate_page(total: Value, symbols: &[&str]) -> NormalizedScannerPage {
        NormalizedScannerPage {
            result: scanner_page(total, symbols),
            provider_row_count: symbols.len(),
        }
    }

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
    fn normalize_scan_request_accepts_earnings_columns_without_changing_defaults() {
        let request = ScannerScanRequest {
            market: "america".to_string(),
            exchanges: Vec::new(),
            columns: Some(
                "name,earnings_release_next_date,earnings_release_date,earnings_release_next_time,earnings_release_next_trading_date_fq,dividend_amount_recent"
                    .to_string(),
            ),
            sort: Some("earnings_release_next_date".to_string()),
            asc: true,
            desc: false,
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
                "earnings_release_next_date",
                "earnings_release_date",
                "earnings_release_next_time",
                "earnings_release_next_trading_date_fq",
                "dividend_amount_recent"
            ]
        );
        assert_eq!(normalized.sort_field, "earnings_release_next_date");
        assert_eq!(normalized.sort_order, "asc");

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
        assert_eq!(result["source_category"], "desktop_free_read");
        assert_eq!(result["requires_desktop"], false);
        assert_eq!(result["non_mutating"], true);
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

        assert_eq!(result.source, "scanner_scan_rest");
        assert_eq!(result.source_category, "desktop_free_read");
        assert!(!result.requires_desktop);
        assert!(result.non_mutating);
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

    #[test]
    fn normalize_scan_request_builds_offset_range_and_rejects_overflow() {
        let mut request = aggregate_base_request(25);
        request.offset = 100;
        request.body["range"] = json!([100, 125]);
        assert_eq!(request.body["range"], json!([100, 125]));

        let raw = ScannerScanRequest {
            market: "america".to_string(),
            exchanges: Vec::new(),
            columns: None,
            sort: None,
            asc: false,
            desc: false,
            limit: Some(2),
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
        assert_eq!(
            normalize_scan_request_at_offset(raw.clone(), usize::MAX)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        let normalized = normalize_scan_request_at_offset(raw, 100).unwrap();
        assert_eq!(normalized.body["range"], json!([100, 102]));
    }

    #[tokio::test]
    async fn aggregate_scan_advances_offsets_deduplicates_and_reports_drift() {
        let pages = Rc::new(RefCell::new(VecDeque::from([
            aggregate_page(json!(4), &["NASDAQ:A", "NASDAQ:B"]),
            aggregate_page(json!(5), &["NASDAQ:B", "NASDAQ:C"]),
            aggregate_page(json!(5), &["NASDAQ:D"]),
        ])));
        let offsets = Rc::new(RefCell::new(Vec::new()));
        let context = Arc::new(Client::new());
        let observed_contexts = Rc::new(RefCell::new(Vec::new()));
        let result =
            scanner_scan_aggregate_with(Arc::clone(&context), aggregate_base_request(2), 10, 1, {
                let pages = Rc::clone(&pages);
                let offsets = Rc::clone(&offsets);
                let observed_contexts = Rc::clone(&observed_contexts);
                move |context, request| {
                    observed_contexts.borrow_mut().push(context);
                    offsets.borrow_mut().push(request.offset);
                    let page = pages.borrow_mut().pop_front().unwrap();
                    Box::pin(std::future::ready(Ok(page)))
                }
            })
            .await
            .unwrap();

        assert_eq!(*offsets.borrow(), [0, 2, 4]);
        assert!(
            observed_contexts
                .borrow()
                .iter()
                .all(|observed| Arc::ptr_eq(observed, &context))
        );
        assert_eq!(result.pages_fetched, 3);
        assert_eq!(result.raw_count, 5);
        assert_eq!(result.count, 4);
        assert_eq!(result.duplicate_count, 1);
        assert_eq!(result.first_total_count, 4);
        assert_eq!(result.last_total_count, 5);
        assert_eq!(result.maximum_total_count, 5);
        assert!(result.total_count_changed);
        assert!(result.duplicates_observed);
        assert!(result.sequential_observation);
        assert!(result.query_fingerprint.starts_with("fnv1a64:"));
        assert_eq!(
            result
                .symbols
                .iter()
                .map(|row| row.symbol.as_str())
                .collect::<Vec<_>>(),
            ["NASDAQ:A", "NASDAQ:B", "NASDAQ:C", "NASDAQ:D"]
        );
        assert!(result.completed_at_epoch_seconds >= result.started_at_epoch_seconds);
    }

    #[tokio::test]
    async fn aggregate_scan_fails_closed_for_bound_missing_total_and_premature_empty() {
        for (page, kind) in [
            (
                aggregate_page(json!(11), &["NASDAQ:A"]),
                ErrorKind::Validation,
            ),
            (
                aggregate_page(Value::Null, &["NASDAQ:A"]),
                ErrorKind::InternalApiUnavailable,
            ),
            (
                aggregate_page(json!(2), &[]),
                ErrorKind::InternalApiUnavailable,
            ),
        ] {
            let page = Rc::new(RefCell::new(Some(page)));
            let error = scanner_scan_aggregate_with((), aggregate_base_request(2), 10, 1, {
                let page = Rc::clone(&page);
                move |(), _| {
                    let page = page.borrow_mut().take().unwrap();
                    Box::pin(std::future::ready(Ok(page)))
                }
            })
            .await
            .unwrap_err();
            assert_eq!(error.kind, kind);
        }
    }

    #[tokio::test]
    async fn aggregate_scan_propagates_page_failure_without_fetching_again() {
        let calls = Rc::new(RefCell::new(0usize));
        let error = scanner_scan_aggregate_with((), aggregate_base_request(2), 10, 1, {
            let calls = Rc::clone(&calls);
            move |(), _| {
                *calls.borrow_mut() += 1;
                let result = if *calls.borrow() == 1 {
                    Ok(aggregate_page(json!(4), &["NASDAQ:A", "NASDAQ:B"]))
                } else {
                    Err(AppError::new(ErrorKind::Connection, "page failed"))
                };
                Box::pin(std::future::ready(result))
            }
        })
        .await
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Connection);
        assert_eq!(*calls.borrow(), 2);
    }

    #[tokio::test]
    async fn aggregate_scan_rejects_non_empty_short_page() {
        let page = Rc::new(RefCell::new(Some(aggregate_page(json!(4), &["NASDAQ:A"]))));
        let error = scanner_scan_aggregate_with((), aggregate_base_request(2), 4, 1, {
            let page = Rc::clone(&page);
            move |(), _| {
                let page = page.borrow_mut().take().unwrap();
                Box::pin(std::future::ready(Ok(page)))
            }
        })
        .await
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        let details = error.details.unwrap();
        assert_eq!(details["expected_page_count"], json!(2));
        assert_eq!(details["observed_page_count"], json!(1));
    }

    #[tokio::test]
    async fn aggregate_scan_rejects_overfull_provider_page_before_truncation() {
        let request = aggregate_base_request(2);
        let payload = json!({
            "totalCount": 2,
            "data": [
                { "s": "NASDAQ:A", "d": ["A", 1.0] },
                { "s": "NASDAQ:B", "d": ["B", 2.0] },
                { "s": "NASDAQ:C", "d": ["C", 3.0] }
            ]
        });
        let page = normalize_scan_response_page(&request, &payload).unwrap();
        assert_eq!(page.provider_row_count, 3);
        assert_eq!(page.result.symbols.len(), 2);

        let page = Rc::new(RefCell::new(Some(page)));
        let error = scanner_scan_aggregate_with((), request, 2, 1, {
            let page = Rc::clone(&page);
            move |(), _| {
                let page = page.borrow_mut().take().unwrap();
                Box::pin(std::future::ready(Ok(page)))
            }
        })
        .await
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        let details = error.details.unwrap();
        assert_eq!(details["expected_page_count"], json!(2));
        assert_eq!(details["observed_page_count"], json!(3));
    }

    #[tokio::test]
    async fn aggregate_scan_handles_zero_exact_boundary_and_downward_drift() {
        let empty = scanner_scan_aggregate_with((), aggregate_base_request(2), 2, 1, {
            move |(), _| Box::pin(std::future::ready(Ok(aggregate_page(json!(0), &[]))))
        })
        .await
        .unwrap();
        assert_eq!(empty.pages_fetched, 1);
        assert_eq!(empty.count, 0);
        assert_eq!(empty.maximum_total_count, 0);

        let exact_pages = Rc::new(RefCell::new(VecDeque::from([
            aggregate_page(json!(4), &["NASDAQ:A", "NASDAQ:B"]),
            aggregate_page(json!(4), &["NASDAQ:C", "NASDAQ:D"]),
        ])));
        let exact = scanner_scan_aggregate_with((), aggregate_base_request(2), 4, 1, {
            let pages = Rc::clone(&exact_pages);
            move |(), _| {
                Box::pin(std::future::ready(Ok(pages
                    .borrow_mut()
                    .pop_front()
                    .unwrap())))
            }
        })
        .await
        .unwrap();
        assert_eq!(exact.pages_fetched, 2);
        assert_eq!(exact.raw_count, 4);
        assert_eq!(exact.maximum_total_count, 4);

        let drift_pages = Rc::new(RefCell::new(VecDeque::from([
            aggregate_page(json!(5), &["NASDAQ:A", "NASDAQ:B"]),
            aggregate_page(json!(3), &["NASDAQ:C"]),
        ])));
        let offsets = Rc::new(RefCell::new(Vec::new()));
        let drift = scanner_scan_aggregate_with((), aggregate_base_request(2), 5, 1, {
            let pages = Rc::clone(&drift_pages);
            let offsets = Rc::clone(&offsets);
            move |(), request| {
                offsets.borrow_mut().push(request.offset);
                Box::pin(std::future::ready(Ok(pages
                    .borrow_mut()
                    .pop_front()
                    .unwrap())))
            }
        })
        .await
        .unwrap();
        assert_eq!(*offsets.borrow(), [0, 2]);
        assert_eq!(drift.first_total_count, 5);
        assert_eq!(drift.last_total_count, 3);
        assert_eq!(drift.maximum_total_count, 5);
        assert!(drift.total_count_changed);
    }

    #[test]
    fn aggregate_bounds_enforce_row_and_request_limits() {
        assert_eq!(validate_aggregate_bounds(1, Some(1)).unwrap(), 1);
        assert_eq!(
            validate_aggregate_bounds(MAX_AGGREGATE_RESULTS, Some(MAX_SCAN_LIMIT)).unwrap(),
            MAX_SCAN_LIMIT
        );
        assert_eq!(validate_aggregate_bounds(100, Some(1)).unwrap(), 1);
        for (max_results, page_size) in [
            (0, Some(1)),
            (MAX_AGGREGATE_RESULTS + 1, Some(100)),
            (1, Some(0)),
            (1, Some(MAX_SCAN_LIMIT + 1)),
            (101, Some(1)),
        ] {
            assert_eq!(
                validate_aggregate_bounds(max_results, page_size)
                    .unwrap_err()
                    .kind,
                ErrorKind::Validation
            );
        }
    }

    #[test]
    fn aggregate_fingerprint_excludes_range_and_tracks_fixed_query_fields() {
        let mut request = aggregate_base_request(25);
        let original = scan_query_fingerprint(&request);
        request.offset = 100;
        request.body["range"] = json!([100, 125]);
        assert_eq!(scan_query_fingerprint(&request), original);

        request.columns.push("volume".to_string());
        assert_ne!(scan_query_fingerprint(&request), original);
    }

    #[test]
    fn explicit_page_wrapper_preserves_default_result_json() {
        let page = scanner_page(json!(1), &["NASDAQ:A"]);
        let default_json = serde_json::to_value(&page).unwrap();
        assert!(default_json.get("offset").is_none());

        let explicit_json =
            serde_json::to_value(ScannerPageScanResult { offset: 100, page }).unwrap();
        assert_eq!(explicit_json["offset"], json!(100));
        assert_eq!(explicit_json["source"], json!(SCAN_SOURCE));
        assert!(explicit_json.get("page").is_none());
    }
}
