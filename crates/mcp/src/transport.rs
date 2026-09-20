//! SDK connection lifecycle shared by public commands and the proof harness.

use crate::{
    Failure, Result,
    admission::Admission,
    http::{Http, OHLCV_TOOL_NAMES},
    sse,
};
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

pub(crate) async fn read(
    http: &Http,
    token: String,
    requests: &[Request],
    mut admission: Option<&mut Admission>,
) -> Result<Vec<Result<CallToolResult>>> {
    let mut config = StreamableHttpClientTransportConfig::with_uri(http.endpoints.resource.clone())
        .auth_header(token)
        .max_concurrent_requests(1)
        .max_sse_event_size(sse::MAX_RESPONSE_BYTES)
        .reinit_on_expired_session(false);
    config.retry_config = Arc::new(NeverRetry::default());
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
            for tool in page.tools {
                if OHLCV_TOOL_NAMES.contains(&tool.name.as_ref()) {
                    if tool_name.is_some() {
                        return Err(Failure::UnsupportedCapability);
                    }
                    http.describe_ohlcv_schema(Some(&tool.input_schema));
                    validate_schema(&tool.input_schema)?;
                    tool_name = Some(tool.name.to_string());
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
            http.describe_ohlcv_schema(None);
            return Err(Failure::UnsupportedCapability);
        }
        let mut results = Vec::new();
        for (index, request) in requests.iter().enumerate() {
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
                    .with_arguments(request.arguments().as_object().unwrap().clone()),
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

fn sdk_failure(error: rmcp::ServiceError, http: &Http) -> Failure {
    match error {
        rmcp::ServiceError::McpError(_) => Failure::ProviderError,
        rmcp::ServiceError::Timeout { .. } => Failure::Timeout,
        _ => http.failure(),
    }
}

fn validate_schema(schema: &serde_json::Map<String, Value>) -> Result<()> {
    if schema.get("type").and_then(Value::as_str) != Some("object") {
        return Err(Failure::SchemaChanged);
    }
    let properties = schema
        .get("properties")
        .and_then(Value::as_object)
        .ok_or(Failure::SchemaChanged)?;
    for (name, kind) in [
        ("symbol", "string"),
        ("interval", "string"),
        ("count", "integer"),
        ("summary", "boolean"),
    ] {
        let field = properties.get(name).ok_or(Failure::SchemaChanged)?;
        let accepts = |v: &Value| {
            v.get("type")
                .and_then(Value::as_str)
                .is_some_and(|actual| actual == kind || (name == "count" && actual == "number"))
        };
        if !accepts(field)
            && !field
                .get("anyOf")
                .and_then(Value::as_array)
                .is_some_and(|variants| variants.iter().any(accepts))
        {
            return Err(Failure::SchemaChanged);
        }
    }
    if schema
        .get("required")
        .and_then(Value::as_array)
        .is_some_and(|values| {
            values.iter().any(|v| {
                !matches!(
                    v.as_str(),
                    Some("symbol" | "interval" | "count" | "summary")
                )
            })
        })
    {
        return Err(Failure::SchemaChanged);
    }
    Ok(())
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
