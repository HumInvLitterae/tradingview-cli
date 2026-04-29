use serde_json::{Map, Value, json};

use tradingview_core::{AppError, ErrorKind};

const METAINFO_BASE_URL: &str = "https://scanner.tradingview.com";
const METAINFO_SOURCE: &str = "scanner_metainfo_rest";
const SUPPORTED_METAINFO_MARKETS: &[&str] = &["america"];

#[derive(Debug)]
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
    let normalized = normalize_metainfo_request(request)?;
    let url = metainfo_url(&normalized.market)?;
    let client = reqwest::Client::new();
    let mut builder = client.post(url);
    if !normalized.fields.is_empty() {
        builder = builder.json(&json!({ "fields": normalized.fields }));
    }

    let response = builder
        .send()
        .await
        .map_err(|err| AppError::new(ErrorKind::Connection, err.to_string()))?;

    let status = response.status();
    if !status.is_success() {
        return Err(AppError::new(
            ErrorKind::Connection,
            format!("TradingView scanner metainfo API returned {status}"),
        ));
    }

    let value = response
        .json::<Value>()
        .await
        .map_err(|err| AppError::new(ErrorKind::InternalApiUnavailable, err.to_string()))?;

    normalize_metainfo_response(&normalized, &value)
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

fn normalize_metainfo_response(
    request: &NormalizedMetainfoRequest,
    value: &Value,
) -> Result<Value, AppError> {
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
            let name = field
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if requested.contains(&name) {
                matched.push(field);
            }
        }
        let missing = request
            .fields
            .iter()
            .filter(|field| {
                !matched.iter().any(|item| {
                    item.get("name")
                        .and_then(Value::as_str)
                        .is_some_and(|name| name == *field)
                })
            })
            .cloned()
            .collect();
        (matched, missing)
    };

    let mut payload = Map::new();
    payload.insert("source".to_string(), json!(METAINFO_SOURCE));
    payload.insert("market".to_string(), json!(request.market));
    payload.insert("requested_fields".to_string(), json!(request.fields));
    payload.insert("field_count".to_string(), json!(fields.len()));
    payload.insert("fields".to_string(), json!(fields));
    payload.insert("missing_fields".to_string(), json!(missing_fields));
    if let Some(currency) = object.get("financial_currency") {
        payload.insert("financial_currency".to_string(), currency.clone());
    }

    Ok(Value::Object(payload))
}

fn normalize_fields(value: &Value) -> Result<Vec<Value>, AppError> {
    match value {
        Value::Array(fields) => fields.iter().map(normalize_field).collect(),
        Value::Object(fields) => fields
            .iter()
            .map(|(key, value)| normalize_field_object(key, value))
            .collect(),
        _ => Err(malformed_metainfo("fields")),
    }
}

fn normalize_field(value: &Value) -> Result<Value, AppError> {
    match value {
        Value::String(name) if !name.trim().is_empty() => Ok(json!({
            "name": name.trim(),
            "type": Value::Null,
        })),
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

fn normalize_field_object(key: &str, value: &Value) -> Result<Value, AppError> {
    match value {
        Value::Object(object) => Ok(field_payload(
            first_string(object, &["propName", "name"]).unwrap_or(key),
            first_string(object, &["kind", "type", "dataType", "t"]),
            first_string(object, &["title", "shortName", "label"]),
            object.get("range").or_else(|| object.get("r")),
        )),
        _ => Ok(json!({
            "name": key,
            "type": Value::Null,
        })),
    }
}

fn field_payload(
    name: &str,
    field_type: Option<&str>,
    label: Option<&str>,
    range: Option<&Value>,
) -> Value {
    let mut payload = Map::new();
    payload.insert("name".to_string(), json!(name));
    payload.insert(
        "type".to_string(),
        field_type.map(Value::from).unwrap_or(Value::Null),
    );
    if let Some(label) = label.filter(|value| *value != name) {
        payload.insert("label".to_string(), json!(label));
    }
    if let Some(range) = range {
        payload.insert("range".to_string(), range.clone());
    }
    Value::Object(payload)
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
