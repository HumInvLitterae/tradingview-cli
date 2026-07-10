use serde_json::{Map, Value, json};

use tradingview_core::{AppError, ErrorKind};

use super::http::{configured_client, map_http_error};
use super::types::{ScannerFieldInfo, ScannerMetainfoResult};

const METAINFO_BASE_URL: &str = "https://scanner.tradingview.com";
const METAINFO_SOURCE: &str = "scanner_metainfo_rest";
const DESKTOP_FREE_READ_CATEGORY: &str = "desktop_free_read";
const SUPPORTED_METAINFO_MARKETS: &[&str] = &["america"];

#[derive(Debug)]
/// Request for Desktop-free scanner field metadata.
pub struct ScannerMetainfoRequest {
    pub market: String,
    pub fields: Vec<String>,
}

#[derive(Debug)]
struct NormalizedMetainfoRequest {
    market: String,
    fields: Vec<String>,
}

pub async fn scanner_metainfo(request: ScannerMetainfoRequest) -> Result<Value, AppError> {
    serde_json::to_value(scanner_metainfo_typed(request).await?)
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))
}

/// Reads scanner field metadata without connecting to TradingView Desktop.
///
/// This is the typed Rust API. Use [`scanner_metainfo`] only when preserving
/// the CLI-compatible JSON payload shape is required.
pub async fn scanner_metainfo_typed(
    request: ScannerMetainfoRequest,
) -> Result<ScannerMetainfoResult, AppError> {
    let normalized = normalize_metainfo_request(request)?;
    let url = metainfo_url(&normalized.market)?;
    let client = configured_client()?;
    let mut builder = client.post(url);
    if !normalized.fields.is_empty() {
        builder = builder.json(&json!({ "fields": normalized.fields }));
    }

    let response = builder
        .send()
        .await
        .map_err(|err| map_http_error(err, ErrorKind::Connection, "Scanner metainfo request"))?;

    let status = response.status();
    if !status.is_success() {
        return Err(AppError::new(
            ErrorKind::Connection,
            format!("TradingView scanner metainfo API returned {status}"),
        ));
    }

    let value = response.json::<Value>().await.map_err(|err| {
        map_http_error(
            err,
            ErrorKind::InternalApiUnavailable,
            "Scanner metainfo response",
        )
    })?;

    normalize_metainfo_response_typed(&normalized, &value)
}

fn normalize_metainfo_request(
    request: ScannerMetainfoRequest,
) -> Result<NormalizedMetainfoRequest, AppError> {
    let market = validate_metainfo_market(&request.market)?;
    let fields = normalize_requested_fields(request.fields)?;
    Ok(NormalizedMetainfoRequest { market, fields })
}

fn validate_metainfo_market(market: &str) -> Result<String, AppError> {
    let market = market.trim();
    SUPPORTED_METAINFO_MARKETS
        .iter()
        .copied()
        .find(|candidate| *candidate == market)
        .map(str::to_string)
        .ok_or_else(|| {
            AppError::new(
                ErrorKind::Validation,
                format!("Unsupported scanner metainfo market: {market}"),
            )
            .with_details(json!({ "supported_markets": SUPPORTED_METAINFO_MARKETS }))
        })
}

fn normalize_requested_fields(fields: Vec<String>) -> Result<Vec<String>, AppError> {
    let mut normalized = Vec::new();
    for field in fields {
        let field = field.trim();
        if field.is_empty() {
            return Err(AppError::new(
                ErrorKind::Validation,
                "--field values must not be empty",
            ));
        }
        if !normalized.iter().any(|value| value == field) {
            normalized.push(field.to_string());
        }
    }
    Ok(normalized)
}

fn metainfo_url(market: &str) -> Result<reqwest::Url, AppError> {
    reqwest::Url::parse(&format!("{METAINFO_BASE_URL}/{market}/metainfo"))
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))
}

#[cfg(test)]
fn normalize_metainfo_response(
    request: &NormalizedMetainfoRequest,
    value: &Value,
) -> Result<Value, AppError> {
    serde_json::to_value(normalize_metainfo_response_typed(request, value)?)
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))
}

fn normalize_metainfo_response_typed(
    request: &NormalizedMetainfoRequest,
    value: &Value,
) -> Result<ScannerMetainfoResult, AppError> {
    let object = value
        .as_object()
        .ok_or_else(|| malformed_metainfo("response"))?;
    let fields_value = object
        .get("fields")
        .or_else(|| object.get("columns"))
        .unwrap_or(value);
    let all_fields = normalize_fields(fields_value)?;

    let (fields, missing_fields) = if request.fields.is_empty() {
        (all_fields, Vec::new())
    } else {
        let requested = request
            .fields
            .iter()
            .map(|field| field.as_str())
            .collect::<Vec<_>>();
        let mut matched = Vec::new();
        for field in all_fields {
            if requested.contains(&field.name.as_str()) {
                matched.push(field);
            }
        }
        let missing = request
            .fields
            .iter()
            .filter(|field| !matched.iter().any(|item| item.name == **field))
            .cloned()
            .collect();
        (matched, missing)
    };

    Ok(ScannerMetainfoResult {
        source: METAINFO_SOURCE.to_string(),
        source_category: DESKTOP_FREE_READ_CATEGORY.to_string(),
        requires_desktop: false,
        non_mutating: true,
        market: request.market.clone(),
        requested_fields: request.fields.clone(),
        field_count: fields.len(),
        fields,
        missing_fields,
        financial_currency: object.get("financial_currency").cloned(),
    })
}

fn normalize_fields(value: &Value) -> Result<Vec<ScannerFieldInfo>, AppError> {
    match value {
        Value::Array(fields) => fields.iter().map(normalize_field).collect(),
        Value::Object(fields) => fields
            .iter()
            .map(|(key, value)| normalize_field_object(key, value))
            .collect(),
        _ => Err(malformed_metainfo("fields")),
    }
}

fn normalize_field(value: &Value) -> Result<ScannerFieldInfo, AppError> {
    match value {
        Value::String(name) if !name.trim().is_empty() => Ok(ScannerFieldInfo {
            name: name.trim().to_string(),
            field_type: Value::Null,
            label: None,
            range: None,
        }),
        Value::Object(object) => {
            let name = first_string(object, &["n", "name", "id", "propName"])
                .ok_or_else(|| malformed_metainfo("field name"))?;
            Ok(field_payload(
                name,
                first_string(object, &["t", "kind", "type", "dataType"]),
                first_string(object, &["title", "shortName", "label"]),
                object.get("r").or_else(|| object.get("range")),
            ))
        }
        _ => Err(malformed_metainfo("field")),
    }
}

fn normalize_field_object(key: &str, value: &Value) -> Result<ScannerFieldInfo, AppError> {
    match value {
        Value::Object(object) => Ok(field_payload(
            first_string(object, &["propName", "name"]).unwrap_or(key),
            first_string(object, &["kind", "type", "dataType", "t"]),
            first_string(object, &["title", "shortName", "label"]),
            object.get("range").or_else(|| object.get("r")),
        )),
        _ => Ok(ScannerFieldInfo {
            name: key.to_string(),
            field_type: Value::Null,
            label: None,
            range: None,
        }),
    }
}

fn field_payload(
    name: &str,
    field_type: Option<&str>,
    label: Option<&str>,
    range: Option<&Value>,
) -> ScannerFieldInfo {
    ScannerFieldInfo {
        name: name.to_string(),
        field_type: field_type.map(Value::from).unwrap_or(Value::Null),
        label: label
            .filter(|value| *value != name)
            .map(ToString::to_string),
        range: range.cloned(),
    }
}

fn first_string<'a>(object: &'a Map<String, Value>, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| object.get(*key).and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn malformed_metainfo(label: &str) -> AppError {
    AppError::new(
        ErrorKind::InternalApiUnavailable,
        format!("Unexpected TradingView scanner metainfo response shape at {label}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_metainfo_request_rejects_unsupported_market_and_blank_fields() {
        let invalid_market = ScannerMetainfoRequest {
            market: "global".to_string(),
            fields: Vec::new(),
        };
        assert_eq!(
            normalize_metainfo_request(invalid_market).unwrap_err().kind,
            ErrorKind::Validation
        );

        let blank_field = ScannerMetainfoRequest {
            market: "america".to_string(),
            fields: vec![" ".to_string()],
        };
        assert_eq!(
            normalize_metainfo_request(blank_field).unwrap_err().kind,
            ErrorKind::Validation
        );
    }

    #[test]
    fn normalize_metainfo_response_handles_compact_array_and_missing_fields() {
        let request = normalize_metainfo_request(ScannerMetainfoRequest {
            market: "america".to_string(),
            fields: vec!["close".to_string(), "banana".to_string()],
        })
        .unwrap();
        let payload = json!({
            "financial_currency": "USD",
            "fields": [
                { "n": "close", "t": "price", "r": [0, 1000] },
                { "n": "market_cap_basic", "t": "number", "title": "Market Cap" }
            ]
        });

        let result = normalize_metainfo_response(&request, &payload).unwrap();

        assert_eq!(result["source"], "scanner_metainfo_rest");
        assert_eq!(result["source_category"], "desktop_free_read");
        assert_eq!(result["requires_desktop"], false);
        assert_eq!(result["non_mutating"], true);
        assert_eq!(result["market"], "america");
        assert_eq!(result["requested_fields"], json!(["close", "banana"]));
        assert_eq!(result["field_count"], 1);
        assert_eq!(result["financial_currency"], "USD");
        assert_eq!(result["fields"][0]["name"], "close");
        assert_eq!(result["fields"][0]["type"], "price");
        assert_eq!(result["fields"][0]["range"], json!([0, 1000]));
        assert_eq!(result["missing_fields"], json!(["banana"]));
    }

    #[test]
    fn normalize_metainfo_response_typed_preserves_requested_and_missing_fields() {
        let request = normalize_metainfo_request(ScannerMetainfoRequest {
            market: "america".to_string(),
            fields: vec!["close".to_string(), "banana".to_string()],
        })
        .unwrap();
        let payload = json!({
            "financial_currency": "USD",
            "fields": [
                { "n": "close", "t": "price", "r": [0, 1000] }
            ]
        });

        let result = normalize_metainfo_response_typed(&request, &payload).unwrap();

        assert_eq!(result.source, "scanner_metainfo_rest");
        assert_eq!(result.source_category, "desktop_free_read");
        assert!(!result.requires_desktop);
        assert!(result.non_mutating);
        assert_eq!(result.market, "america");
        assert_eq!(result.requested_fields, ["close", "banana"]);
        assert_eq!(result.field_count, 1);
        assert_eq!(result.fields[0].name, "close");
        assert_eq!(result.fields[0].field_type, json!("price"));
        assert_eq!(result.missing_fields, ["banana"]);
        assert_eq!(result.financial_currency, Some(json!("USD")));
    }

    #[test]
    fn normalize_metainfo_response_handles_object_fields() {
        let request = normalize_metainfo_request(ScannerMetainfoRequest {
            market: "america".to_string(),
            fields: Vec::new(),
        })
        .unwrap();
        let payload = json!({
            "fields": {
                "close": { "kind": "price", "title": "Close" },
                "name": null
            }
        });

        let result = normalize_metainfo_response(&request, &payload).unwrap();

        assert_eq!(result["field_count"], 2);
        assert_eq!(result["fields"][0]["name"], "close");
        assert_eq!(result["fields"][0]["type"], "price");
        assert_eq!(result["fields"][1]["name"], "name");
        assert!(result["missing_fields"].as_array().unwrap().is_empty());
    }

    #[test]
    fn normalize_metainfo_response_rejects_malformed_shapes() {
        let request = normalize_metainfo_request(ScannerMetainfoRequest {
            market: "america".to_string(),
            fields: Vec::new(),
        })
        .unwrap();

        assert_eq!(
            normalize_metainfo_response(&request, &json!(null))
                .unwrap_err()
                .kind,
            ErrorKind::InternalApiUnavailable
        );
        assert_eq!(
            normalize_metainfo_response(&request, &json!({ "fields": [123] }))
                .unwrap_err()
                .kind,
            ErrorKind::InternalApiUnavailable
        );
    }
}
