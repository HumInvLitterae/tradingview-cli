//! SDK connection lifecycle shared by public commands and the proof harness.

use crate::{Failure, Result, admission::Admission, http::Http, sse, tools::Tool};
use rmcp::{
    ServiceExt,
    model::{CallToolRequestParams, CallToolResponse, CallToolResult, PaginatedRequestParams},
    transport::{
        StreamableHttpClientTransport, common::client_side_sse::NeverRetry,
        streamable_http_client::StreamableHttpClientTransportConfig,
    },
};
use serde_json::Value;
use std::{sync::Arc, time::Duration};
use tokio::time::timeout_at;
use tradingview_model::mcp_bars::Request;

fn transport_config(http: &Http, token: String) -> StreamableHttpClientTransportConfig {
    let mut config = StreamableHttpClientTransportConfig::with_uri(http.endpoints.resource.clone())
        .auth_header(token)
        .max_concurrent_requests(1)
        .max_sse_event_size(sse::MAX_RESPONSE_BYTES)
        .reinit_on_expired_session(false);
    config.retry_config = Arc::new(NeverRetry::default());
    config
}

pub(crate) async fn read(
    http: &Http,
    token: String,
    requests: &[Request],
    admission: Option<&mut Admission>,
) -> Result<Vec<Result<CallToolResult>>> {
    let arguments: Vec<_> = requests.iter().map(Request::arguments).collect();
    call(http, token, Tool::Bars, &arguments, admission).await
}

pub(crate) async fn call(
    http: &Http,
    token: String,
    tool: Tool,
    arguments: &[Value],
    mut admission: Option<&mut Admission>,
) -> Result<Vec<Result<CallToolResult>>> {
    for args in arguments {
        tool.validate_arguments(args)?;
    }
    let config = transport_config(http, token);
    let transport = StreamableHttpClientTransport::with_client(http.clone(), config);
    let service = timeout_at(http.deadline, ().serve(transport))
        .await
        .map_err(|_| Failure::Timeout)?
        .map_err(|_| http.failure())?;
    let result = timeout_at(http.deadline, async {
        let mut cursor = None;
        let mut tool_name = None;
        for _ in 0..10 {
            let page = service
                .list_tools(cursor.map(|cursor| {
                    let mut params = PaginatedRequestParams::default();
                    params.cursor = Some(cursor);
                    params
                }))
                .await
                .map_err(|error| sdk_failure(error, http))?;
            http.describe_catalog(&page.tools);
            for candidate in page.tools {
                if tool.names().contains(&candidate.name.as_ref()) {
                    if tool_name.is_some() {
                        return Err(Failure::UnsupportedCapability);
                    }
                    if tool == Tool::Bars {
                        http.describe_ohlcv_schema(Some(&candidate.input_schema));
                    }
                    validate_schema(&candidate.input_schema, tool, arguments)?;
                    tool_name = Some(candidate.name.to_string());
                }
            }
            if tool_name.is_some() {
                break;
            }
            cursor = page.next_cursor;
            if cursor.is_none() {
                break;
            }
        }
        if tool_name.is_none() {
            if tool == Tool::Bars {
                http.describe_ohlcv_schema(None);
            }
            return Err(Failure::UnsupportedCapability);
        }
        let mut results = Vec::new();
        for (index, args) in arguments.iter().enumerate() {
            if let Some(admission) = admission.as_deref_mut() {
                admission.before_tool(http.deadline).await?;
            } else if index > 0 {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            let outcome = service
                .call_tool_once(
                    CallToolRequestParams::new(
                        tool_name.clone().ok_or(Failure::UnsupportedCapability)?,
                    )
                    .with_arguments(args.as_object().unwrap().clone()),
                )
                .await;
            let outcome = match outcome {
                Ok(CallToolResponse::Complete(result)) if result.is_error != Some(true) => {
                    Ok(result)
                }
                Ok(CallToolResponse::Complete(_)) => Err(Failure::ProviderError),
                Ok(_) => Err(Failure::UnsupportedCapability),
                Err(error) => Err(sdk_failure(error, http)),
            };
            let failed = outcome.is_err();
            results.push(outcome);
            if failed {
                break;
            }
        }
        Ok(results)
    })
    .await
    .unwrap_or(Err(Failure::Timeout));
    let cleanup = timeout_at(http.deadline, service.cancel()).await;
    if result.is_ok() && !matches!(cleanup, Ok(Ok(_))) {
        return Err(Failure::Timeout);
    }
    result
}

/// Inspect closed schemas only; never dispatch a tool.
pub(crate) async fn inspect_watchlist_tools(http: &Http, token: String) -> Result<Value> {
    use serde_json::json;
    let expected = [
        (
            Tool::CreateWatchlist,
            json!({"name": "Example", "symbols": []}),
        ),
        (
            Tool::UpdateWatchlist,
            json!({"watchlist_id": "12", "name": "Example", "description": ""}),
        ),
        (
            Tool::AddWatchlist,
            json!({"watchlist_id": "12", "symbols": ["NASDAQ:EXAMPLE"]}),
        ),
        (
            Tool::RemoveWatchlist,
            json!({"watchlist_id": "12", "symbols": ["NASDAQ:EXAMPLE"]}),
        ),
        (Tool::DeleteWatchlist, json!({"watchlist_id": "12"})),
    ];
    inspect_tools(http, token, &expected).await
}

pub(crate) async fn inspect_alert_tools(http: &Http, token: String) -> Result<Value> {
    use tradingview_model::mcp_account::{AlertAction, AlertMutation, AlertSettings};
    let requests = [
        AlertMutation::create(
            "NASDAQ:EXAMPLE",
            100.0,
            "greater",
            "1D",
            AlertSettings {
                name: Some("Example".into()),
                ..Default::default()
            },
        ),
        AlertMutation::update(
            12,
            AlertSettings {
                name: Some("Example".into()),
                auto_deactivate: Some(false),
                email: Some(false),
                mobile_push: Some(false),
                popup: Some(false),
            },
        ),
        AlertMutation::state(AlertAction::Stop, &[12]),
        AlertMutation::state(AlertAction::Restart, &[12]),
        AlertMutation::state(AlertAction::Delete, &[12]),
    ];
    let expected: Vec<_> = requests
        .into_iter()
        .map(|request| {
            request
                .map(|request| (Tool::from(request.action()), request.arguments()))
                .map_err(|_| Failure::UnsupportedCapability)
        })
        .collect::<Result<_>>()?;
    inspect_tools(http, token, &expected).await
}

pub(crate) async fn inspect_tools(
    http: &Http,
    token: String,
    expected: &[(Tool, Value)],
) -> Result<Value> {
    for (tool, arguments) in expected {
        tool.validate_arguments(arguments)?;
    }
    let config = transport_config(http, token);
    let service = timeout_at(
        http.deadline,
        ().serve(StreamableHttpClientTransport::with_client(
            http.clone(),
            config,
        )),
    )
    .await
    .map_err(|_| Failure::Timeout)?
    .map_err(|_| http.failure())?;
    let result = timeout_at(http.deadline, async {
        let mut cursor = None;
        let mut found = std::collections::HashSet::new();
        let mut reports = Vec::new();

        for _ in 0..10 {
            let page = service
                .list_tools(cursor.map(|cursor| {
                    let mut params = PaginatedRequestParams::default();
                    params.cursor = Some(cursor);
                    params
                }))
                .await
                .map_err(|error| sdk_failure(error, http))?;

            for candidate in page.tools {
                let Some((tool, arguments)) = expected
                    .iter()
                    .find(|(tool, _)| tool.names().contains(&candidate.name.as_ref()))
                else {
                    continue;
                };
                if !found.insert(tool.names()[0]) {
                    return Err(Failure::SchemaChanged);
                }
                validate_schema(
                    &candidate.input_schema,
                    *tool,
                    std::slice::from_ref(arguments),
                )?;
                let value =
                    serde_json::to_value(candidate).map_err(|_| Failure::InvalidResponse)?;
                reports.push(serde_json::json!({
                    "tool": tool.names()[0],
                    "schema_accepted": true,
                    "output_schema": value.get("outputSchema").map(|schema| schema_outline(schema, 0)),
                    "read_only_hint": value
                        .pointer("/annotations/readOnlyHint")
                        .and_then(Value::as_bool),
                    "destructive_hint": value
                        .pointer("/annotations/destructiveHint")
                        .and_then(Value::as_bool),
                    "security_schemes_present": value.get("securitySchemes").is_some()
                        || value.pointer("/_meta/securitySchemes").is_some()
                }));
            }
            cursor = page.next_cursor;
            if cursor.is_none() {
                break;
            }
        }
        if found.len() != expected.len() {
            return Err(Failure::UnsupportedCapability);
        }
        Ok(serde_json::json!({"tools": reports, "mutation_dispatches": 0}))
    })
    .await
    .unwrap_or(Err(Failure::Timeout));
    let cleanup = timeout_at(http.deadline, service.cancel()).await;
    if result.is_ok() && !matches!(cleanup, Ok(Ok(_))) {
        return Err(Failure::Timeout);
    }
    result
}

// Public tool schema structure only; no descriptions, examples or default values.
fn schema_outline(schema: &Value, depth: usize) -> Value {
    if depth > 8 {
        return serde_json::json!({"truncated": true});
    }
    let mut result = serde_json::Map::new();
    if let Some(kind) = schema.get("type").and_then(Value::as_str)
        && matches!(
            kind,
            "object" | "array" | "string" | "number" | "integer" | "boolean" | "null"
        )
    {
        result.insert("type".into(), Value::String(kind.into()));
    }
    if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
        let properties = properties
            .iter()
            .filter(|(name, _)| {
                name.len() <= 128
                    && name.starts_with(|c: char| c.is_ascii_alphabetic())
                    && name
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
            })
            .take(100)
            .map(|(name, value)| (name.clone(), schema_outline(value, depth + 1)))
            .collect();
        result.insert("properties".into(), Value::Object(properties));
    }
    if let Some(items) = schema.get("items") {
        result.insert("items".into(), schema_outline(items, depth + 1));
    }
    for union in ["anyOf", "oneOf", "allOf"] {
        if let Some(variants) = schema.get(union).and_then(Value::as_array) {
            result.insert(
                union.into(),
                Value::Array(
                    variants
                        .iter()
                        .take(20)
                        .map(|value| schema_outline(value, depth + 1))
                        .collect(),
                ),
            );
        }
    }
    result.insert(
        "ref_present".into(),
        Value::Bool(schema.get("$ref").is_some()),
    );
    Value::Object(result)
}

fn sdk_failure(error: rmcp::ServiceError, http: &Http) -> Failure {
    match error {
        rmcp::ServiceError::McpError(_) => Failure::ProviderError,
        rmcp::ServiceError::Timeout { .. } => Failure::Timeout,
        _ => http.failure(),
    }
}

fn validate_schema(
    schema: &serde_json::Map<String, Value>,
    tool: Tool,
    arguments: &[Value],
) -> Result<()> {
    if schema.get("type").and_then(Value::as_str) != Some("object") {
        return Err(Failure::SchemaChanged);
    }
    let empty = serde_json::Map::new();
    let properties = match schema.get("properties") {
        Some(Value::Object(properties)) => properties,
        None if tool.fields().is_empty() => &empty,
        _ => return Err(Failure::SchemaChanged),
    };
    for &(name, kind) in tool.fields() {
        let field = properties.get(name).ok_or(Failure::SchemaChanged)?;
        if !accepts_type(field, kind) {
            return Err(Failure::SchemaChanged);
        }
        if kind == "array" {
            let array_schema = if accepts_direct_type(field, "array") {
                Some(field)
            } else {
                field
                    .get("anyOf")
                    .and_then(Value::as_array)
                    .and_then(|variants| variants.iter().find(|v| accepts_direct_type(v, "array")))
            }
            .ok_or(Failure::SchemaChanged)?;
            if !array_schema.get("items").is_some_and(|items| {
                accepts_type(
                    items,
                    if matches!(
                        tool,
                        Tool::AlertDetails
                            | Tool::StopAlerts
                            | Tool::RestartAlerts
                            | Tool::DeleteAlerts
                    ) {
                        "integer"
                    } else {
                        "string"
                    },
                )
            }) {
                return Err(Failure::SchemaChanged);
            }
        }
    }
    if let Some(required) = schema.get("required") {
        let values = required.as_array().ok_or(Failure::SchemaChanged)?;
        if values.iter().any(|v| {
            v.as_str()
                .is_none_or(|name| arguments.iter().any(|args| args.get(name).is_none()))
        }) {
            return Err(Failure::SchemaChanged);
        }
    }
    Ok(())
}

fn accepts_direct_type(schema: &Value, kind: &str) -> bool {
    let matches = |value: &Value| {
        value
            .as_str()
            .is_some_and(|value| value == kind || (kind == "integer" && value == "number"))
    };
    schema.get("type").is_some_and(|value| {
        matches(value)
            || value
                .as_array()
                .is_some_and(|types| types.iter().any(matches))
    })
}

fn accepts_type(schema: &Value, kind: &str) -> bool {
    accepts_direct_type(schema, kind)
        || schema
            .get("anyOf")
            .and_then(Value::as_array)
            .is_some_and(|variants| variants.iter().any(|v| accepts_direct_type(v, kind)))
}

pub(crate) fn result_value(result: CallToolResult) -> Result<Value> {
    if result.is_error == Some(true) {
        return Err(Failure::ProviderError);
    }
    if let Some(value) = result.structured_content {
        return Ok(value);
    }
    if result.content.len() != 1 {
        return Err(Failure::InvalidResponse);
    }
    let content = serde_json::to_value(&result.content[0]).map_err(|_| Failure::InvalidResponse)?;
    let text = content
        .get("text")
        .and_then(Value::as_str)
        .ok_or(Failure::InvalidResponse)?;
    serde_json::from_str(text).map_err(|_| Failure::InvalidResponse)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn schema_outline_excludes_values_and_free_text() {
        let summary = schema_outline(
            &json!({
                "type": "object", "description": "private description",
                "properties": {"events": {"type": "array", "items": {
                    "type": "object", "properties": {"alert_id": {"type": "integer", "default": 123}}
                }}}, "examples": [{"secret": true}]
            }),
            0,
        );
        assert_eq!(
            summary["properties"]["events"]["items"]["properties"]["alert_id"]["type"],
            "integer"
        );
        for omitted in [
            "private",
            "description",
            "123",
            "default",
            "examples",
            "secret",
        ] {
            assert!(!summary.to_string().contains(omitted));
        }
    }

    #[test]
    fn empty_and_integer_array_schemas_remain_closed() {
        let empty = json!({"type": "object"});
        validate_schema(empty.as_object().unwrap(), Tool::Watchlists, &[json!({})]).unwrap();
        assert!(validate_schema(empty.as_object().unwrap(), Tool::Alerts, &[json!({})]).is_err());
        for malformed in [
            json!({"type": "object", "properties": null}),
            json!({"type": "object", "required": ["new_field"]}),
            json!({"type": "object", "required": "invalid"}),
        ] {
            assert!(
                validate_schema(
                    malformed.as_object().unwrap(),
                    Tool::Watchlists,
                    &[json!({})]
                )
                .is_err()
            );
        }
        let mut schema = json!({
            "type": "object",
            "properties": {"alert_ids": {"type": "array", "items": {"type": "integer"}}},
            "required": ["alert_ids"]
        });
        let args = [json!({"alert_ids": [10, 12]})];
        validate_schema(schema.as_object().unwrap(), Tool::AlertDetails, &args).unwrap();
        schema["properties"]["alert_ids"]["items"]["type"] = json!("string");
        assert!(validate_schema(schema.as_object().unwrap(), Tool::AlertDetails, &args).is_err());
    }

    #[test]
    fn incompatible_schema_prevents_dispatch() {
        let mut schema = json!({
            "type": "object",
            "properties": {
                "query": {"type": "string"},
                "type_filter": {"anyOf": [{"type": "string"}, {"type": "null"}]}
            },
            "required": ["query"]
        });
        let args = [json!({"query": "Example"})];
        validate_schema(schema.as_object().unwrap(), Tool::Search, &args).unwrap();
        schema["required"] = json!(["query", "type_filter"]);
        assert_eq!(
            validate_schema(schema.as_object().unwrap(), Tool::Search, &args),
            Err(Failure::SchemaChanged)
        );
        schema["required"] = json!("query");
        assert_eq!(
            validate_schema(schema.as_object().unwrap(), Tool::Search, &args),
            Err(Failure::SchemaChanged)
        );
        schema["required"] = json!(["query"]);
        schema["properties"]["query"] = json!({"type": "integer"});
        assert_eq!(
            validate_schema(schema.as_object().unwrap(), Tool::Search, &args),
            Err(Failure::SchemaChanged)
        );
    }

    #[test]
    fn nullable_array_schema_still_requires_string_items() {
        let args = [serde_json::json!({"symbol": "NASDAQ:EXAMPLE", "columns": ["close"]})];
        let mut schema = serde_json::json!({
            "type": "object", "required": ["symbol"],
            "properties": {
                "symbol": {"type": "string"},
                "columns": {"type": ["null", "array"], "items": {"type": "string"}}
            }
        });
        validate_schema(schema.as_object().unwrap(), Tool::Symbol, &args).unwrap();
        schema["properties"]["columns"]["items"] = serde_json::json!({"type": "number"});
        assert_eq!(
            validate_schema(schema.as_object().unwrap(), Tool::Symbol, &args),
            Err(Failure::SchemaChanged)
        );
    }
}
