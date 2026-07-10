use serde_json::{Value, json};

use tradingview_core::{AppError, ErrorKind};

use crate::http::{configured_client, map_http_error};

const PINE_CHECK_URL: &str = "https://pine-facade.tradingview.com/pine-facade/translate_light?user_name=Guest&pine_id=00000000-0000-0000-0000-000000000000";

pub async fn pine_check(source: &str, input_source: &str) -> Result<Value, AppError> {
    let body =
        reqwest::Url::parse_with_params("https://www.tradingview.com/", &[("source", source)])
            .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))?
            .query()
            .unwrap_or("")
            .to_string();
    let client = configured_client()?;
    let response = client
        .post(PINE_CHECK_URL)
        .header("Accept", "application/json")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Referer", "https://www.tradingview.com/")
        .body(body)
        .send()
        .await
        .map_err(|err| map_http_error(err, ErrorKind::Connection, "Pine check request"))?;

    let status = response.status();
    if !status.is_success() {
        return Err(AppError::new(
            ErrorKind::Connection,
            format!("Pine check API returned {status}"),
        ));
    }

    let value = response.json::<Value>().await.map_err(|err| {
        map_http_error(
            err,
            ErrorKind::InternalApiUnavailable,
            "Pine check response",
        )
    })?;
    normalize_check_response(value, input_source)
}

fn normalize_check_response(value: Value, input_source: &str) -> Result<Value, AppError> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if let Some(message) = value.get("error").and_then(Value::as_str) {
        errors.push(json!({ "message": message }));
    }

    if let Some(result) = value.get("result") {
        if let Some(items) = result.get("errors2").and_then(Value::as_array) {
            errors.extend(items.iter().map(normalize_diagnostic));
        }
        if let Some(items) = result.get("warnings2").and_then(Value::as_array) {
            warnings.extend(items.iter().map(normalize_diagnostic));
        }
    } else if !value.get("error").is_some_and(Value::is_string) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Pine check payload did not include result or error",
        )
        .with_details(value));
    }

    let compiled = errors.is_empty();
    Ok(json!({
        "input_source": input_source,
        "compiled": compiled,
        "error_count": errors.len(),
        "warning_count": warnings.len(),
        "errors": errors,
        "warnings": warnings,
        "source": "pine_facade",
        "note": if compiled {
            Value::String("Pine Script compiled successfully.".to_string())
        } else {
            Value::Null
        },
    }))
}

fn normalize_diagnostic(value: &Value) -> Value {
    let start = value.get("start");
    let end = value.get("end");
    json!({
        "line": start.and_then(|item| item.get("line")).cloned().unwrap_or(Value::Null),
        "column": start.and_then(|item| item.get("column")).cloned().unwrap_or(Value::Null),
        "end_line": end.and_then(|item| item.get("line")).cloned().unwrap_or(Value::Null),
        "end_column": end.and_then(|item| item.get("column")).cloned().unwrap_or(Value::Null),
        "message": value.get("message").cloned().unwrap_or(Value::Null),
        "ctx": value.get("ctx").cloned().unwrap_or(Value::Null),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_successful_compile_payload() {
        let result = normalize_check_response(json!({"result": {}}), "file").unwrap();

        assert_eq!(result["input_source"], "file");
        assert_eq!(result["compiled"], true);
        assert_eq!(result["error_count"], 0);
        assert_eq!(result["warning_count"], 0);
        assert_eq!(result["source"], "pine_facade");
        assert!(result["note"].as_str().unwrap().contains("compiled"));
    }

    #[test]
    fn normalizes_errors_and_warnings() {
        let result = normalize_check_response(
            json!({
                "result": {
                    "errors2": [{
                        "start": {"line": 3, "column": 1},
                        "end": {"line": 3, "column": 20},
                        "message": "Could not find function",
                        "ctx": {"fullName": "missing_fn"}
                    }],
                    "warnings2": [{
                        "start": {"line": 2, "column": 1},
                        "message": "Deprecated"
                    }]
                }
            }),
            "stdin",
        )
        .unwrap();

        assert_eq!(result["compiled"], false);
        assert_eq!(result["error_count"], 1);
        assert_eq!(result["warning_count"], 1);
        assert_eq!(result["errors"][0]["line"], 3);
        assert_eq!(result["errors"][0]["ctx"]["fullName"], "missing_fn");
        assert_eq!(result["warnings"][0]["message"], "Deprecated");
    }

    #[test]
    fn normalizes_outer_error_as_compile_error() {
        let result =
            normalize_check_response(json!({"error": "source is empty"}), "stdin").unwrap();

        assert_eq!(result["compiled"], false);
        assert_eq!(result["error_count"], 1);
        assert_eq!(result["errors"][0]["message"], "source is empty");
    }

    #[test]
    fn rejects_malformed_payload() {
        let error = normalize_check_response(json!({"unexpected": true}), "stdin").unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(
            error.message,
            "Pine check payload did not include result or error"
        );
    }
}
