use serde_json::{Value, json};

use crate::error::{AppError, ErrorKind};

use super::common::field_values_object;

const HOTLIST_BASE_URL: &str = "https://scanner.tradingview.com/presets";
const HOTLIST_REGION: &str = "US";
const HOTLIST_SOURCE: &str = "scanner_preset_rest";
const DEFAULT_HOTLIST_LIMIT: usize = 20;
const MAX_HOTLIST_LIMIT: usize = 20;
const HOTLIST_SLUGS: &[&str] = &[
    "volume_gainers",
    "percent_change_gainers",
    "percent_change_losers",
    "percent_range_gainers",
    "percent_range_losers",
    "gap_gainers",
    "gap_losers",
    "percent_gap_gainers",
    "percent_gap_losers",
];

pub async fn scanner_hotlist(slug: &str, limit: Option<usize>) -> Result<Value, AppError> {
    let slug = validate_hotlist_slug(slug)?;
    let limit = normalize_hotlist_limit(limit)?;
    let url = hotlist_url(slug)?;
    let response = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .map_err(|err| AppError::new(ErrorKind::Connection, err.to_string()))?;

    let status = response.status();
    if !status.is_success() {
        return Err(AppError::new(
            ErrorKind::Connection,
            format!("TradingView scanner preset API returned {status}"),
        ));
    }

    let value = response
        .json::<Value>()
        .await
        .map_err(|err| AppError::new(ErrorKind::InternalApiUnavailable, err.to_string()))?;

    normalize_hotlist_response(slug, limit, &value)
}

fn validate_hotlist_slug(slug: &str) -> Result<&'static str, AppError> {
    let slug = slug.trim();
    if slug.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Hotlist slug must not be empty",
        ));
    }

    HOTLIST_SLUGS
        .iter()
        .copied()
        .find(|candidate| *candidate == slug)
        .ok_or_else(|| {
            AppError::new(
                ErrorKind::Validation,
                format!("Unsupported hotlist slug: {slug}"),
            )
            .with_details(json!({ "supported_slugs": HOTLIST_SLUGS }))
        })
}

fn normalize_hotlist_limit(limit: Option<usize>) -> Result<usize, AppError> {
    match limit {
        Some(0) => Err(AppError::new(
            ErrorKind::Validation,
            "--limit must be greater than 0",
        )),
        Some(limit) => Ok(limit.min(MAX_HOTLIST_LIMIT)),
        None => Ok(DEFAULT_HOTLIST_LIMIT),
    }
}

fn hotlist_url(slug: &str) -> Result<reqwest::Url, AppError> {
    reqwest::Url::parse_with_params(
        &format!("{HOTLIST_BASE_URL}/{HOTLIST_REGION}_{slug}"),
        &[("label-product", "right-hotlists")],
    )
    .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))
}

fn normalize_hotlist_response(slug: &str, limit: usize, value: &Value) -> Result<Value, AppError> {
    let object = value
        .as_object()
        .ok_or_else(|| malformed_hotlist("response"))?;
    let fields = string_array(object.get("fields"), "fields")?;
    let symbols = object
        .get("symbols")
        .and_then(Value::as_array)
        .ok_or_else(|| malformed_hotlist("symbols"))?;
    let total_count = object
        .get("totalCount")
        .and_then(Value::as_u64)
        .map(Value::from)
        .unwrap_or(Value::Null);

    let normalized_symbols = symbols
        .iter()
        .take(limit)
        .map(|row| normalize_hotlist_symbol(row, &fields))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json!({
        "source": HOTLIST_SOURCE,
        "region": HOTLIST_REGION,
        "slug": slug,
        "limit": limit,
        "count": normalized_symbols.len(),
        "total_count": total_count,
        "fields": fields,
        "symbols": normalized_symbols,
    }))
}

fn normalize_hotlist_symbol(row: &Value, fields: &[String]) -> Result<Value, AppError> {
    let object = row
        .as_object()
        .ok_or_else(|| malformed_hotlist("symbol row"))?;
    let symbol = object
        .get("s")
        .and_then(Value::as_str)
        .filter(|symbol| !symbol.trim().is_empty())
        .ok_or_else(|| malformed_hotlist("symbol row s"))?;
    let values = object
        .get("f")
        .and_then(Value::as_array)
        .ok_or_else(|| malformed_hotlist("symbol row f"))?;
    let field_values = field_values_object(fields, values);

    Ok(json!({
        "symbol": symbol,
        "values": values,
        "field_values": field_values,
    }))
}

fn string_array(value: Option<&Value>, label: &str) -> Result<Vec<String>, AppError> {
    let values = value
        .and_then(Value::as_array)
        .ok_or_else(|| malformed_hotlist(label))?;
    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| malformed_hotlist(label))
        })
        .collect()
}

fn malformed_hotlist(label: &str) -> AppError {
    AppError::new(
        ErrorKind::InternalApiUnavailable,
        format!("Unexpected TradingView scanner preset response shape at {label}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_hotlist_slug_accepts_known_slugs() {
        for slug in HOTLIST_SLUGS {
            assert_eq!(validate_hotlist_slug(slug).unwrap(), *slug);
        }
    }

    #[test]
    fn validate_hotlist_slug_rejects_unknown_and_empty_slugs() {
        let empty = validate_hotlist_slug("   ").unwrap_err();
        assert_eq!(empty.kind, ErrorKind::Validation);

        let unknown = validate_hotlist_slug("custom_scan").unwrap_err();
        assert_eq!(unknown.kind, ErrorKind::Validation);
        assert!(unknown.details.is_some());
    }

    #[test]
    fn normalize_hotlist_limit_defaults_clamps_and_rejects_zero() {
        assert_eq!(normalize_hotlist_limit(None).unwrap(), 20);
        assert_eq!(normalize_hotlist_limit(Some(3)).unwrap(), 3);
        assert_eq!(normalize_hotlist_limit(Some(50)).unwrap(), 20);

        let error = normalize_hotlist_limit(Some(0)).unwrap_err();
        assert_eq!(error.kind, ErrorKind::Validation);
    }

    #[test]
    fn normalize_hotlist_response_maps_compact_rows() {
        let payload = json!({
            "fields": ["volume", "change"],
            "symbols": [
                { "s": "NASDAQ:AAPL", "f": [123456, 1.5] },
                { "s": "NASDAQ:MSFT", "f": [654321, -0.5] }
            ],
            "time": 1760000000,
            "totalCount": 20
        });

        let result = normalize_hotlist_response("volume_gainers", 1, &payload).unwrap();

        assert_eq!(result["source"], "scanner_preset_rest");
        assert_eq!(result["region"], "US");
        assert_eq!(result["slug"], "volume_gainers");
        assert_eq!(result["limit"], 1);
        assert_eq!(result["count"], 1);
        assert_eq!(result["total_count"], 20);
        assert_eq!(result["fields"], json!(["volume", "change"]));
        assert_eq!(result["symbols"][0]["symbol"], "NASDAQ:AAPL");
        assert_eq!(result["symbols"][0]["values"], json!([123456, 1.5]));
        assert_eq!(result["symbols"][0]["field_values"]["volume"], 123456);
        assert_eq!(result["symbols"][0]["field_values"]["change"], 1.5);
    }

    #[test]
    fn normalize_hotlist_response_keeps_values_without_matching_fields() {
        let payload = json!({
            "fields": ["volume"],
            "symbols": [
                { "s": "NASDAQ:AAPL", "f": [123456, 1.5] }
            ]
        });

        let result = normalize_hotlist_response("volume_gainers", 20, &payload).unwrap();

        assert_eq!(result["symbols"][0]["values"], json!([123456, 1.5]));
        assert_eq!(
            result["symbols"][0]["field_values"],
            json!({ "volume": 123456 })
        );
    }

    #[test]
    fn normalize_hotlist_response_rejects_malformed_shapes() {
        let missing_fields = json!({ "symbols": [] });
        let error = normalize_hotlist_response("volume_gainers", 20, &missing_fields).unwrap_err();
        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);

        let missing_symbol = json!({ "fields": ["volume"], "symbols": [{ "f": [1] }] });
        let error = normalize_hotlist_response("volume_gainers", 20, &missing_symbol).unwrap_err();
        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);

        let missing_values = json!({ "fields": ["volume"], "symbols": [{ "s": "NASDAQ:AAPL" }] });
        let error = normalize_hotlist_response("volume_gainers", 20, &missing_values).unwrap_err();
        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
    }
}
