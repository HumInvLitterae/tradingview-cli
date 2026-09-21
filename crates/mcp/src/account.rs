//! Shared account readback transport; mutation results remain domain-specific.

use crate::{Failure, admission::Admission, http::Http, transport};
use serde_json::{Value, json};
use tradingview_core::AppError;
use tradingview_model::mcp_account;

pub(crate) async fn readback(
    read: &mcp_account::Request,
    http: &Http,
    token: String,
    admission: &mut Admission,
    stage: &str,
) -> Result<(Value, u32), Value> {
    let before = http
        .tool_attempts()
        .map_err(|_| json!({"status": "failed", "reason": "attempt_count_unavailable"}))?;
    let result: Result<Value, AppError> = async {
        let mut responses = transport::call(
            http,
            token,
            read.kind().into(),
            &[read.arguments()],
            Some(admission),
        )
        .await?;
        let value = transport::result_value(responses.pop().ok_or(Failure::InvalidResponse)??)?;
        mcp_account::normalize(read, value, crate::admission::now_ms()?)
    }
    .await;
    let attempts = http
        .tool_attempts()
        .map_err(|_| json!({"status": "failed", "reason": "attempt_count_unavailable"}))?
        .saturating_sub(before);
    match result {
        Ok(data) => Ok((data, attempts)),
        Err(error) => {
            let error = if let Some(failure) = error
                .details
                .as_ref()
                .and_then(|details| details.get("failure"))
                .and_then(|value| serde_json::from_value::<Failure>(value.clone()).ok())
            {
                crate::client::failure(failure, stage, attempts, Some(http))
            } else {
                error
            };
            Err(json!({
                "status": "failed",
                "tool_attempts": attempts,
                "error": error.details,
                "next_action":
                    "read the target explicitly; do not repeat the mutation automatically"
            }))
        }
    }
}
