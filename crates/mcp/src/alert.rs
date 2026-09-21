//! One explicit alert mutation followed by one non-mutating readback.

use crate::{Failure, admission::Admission, http::Http, transport};
use serde_json::{Value, json};
use tradingview_core::AppError;
use tradingview_model::mcp_account::AlertMutation;

pub(crate) async fn change(
    request: &AlertMutation,
    http: &Http,
    token: String,
    admission: &mut Admission,
) -> Result<Value, AppError> {
    let mut responses = transport::call(
        http,
        token.clone(),
        request.action().into(),
        &[request.arguments()],
        Some(admission),
    )
    .await?;
    let value = transport::result_value(responses.pop().ok_or(Failure::InvalidResponse)??)?;
    let ids = request.targets_after_reply(&value)?;
    if ids.is_empty() {
        return Ok(request.report(
            &ids,
            json!({
                "status": "not_performed",
                "reason": "target_id_unreported",
                "tool_attempts": 0
            }),
        ));
    }

    let read = request.readback(&ids)?;
    let readback =
        match crate::account::readback(&read, http, token, admission, "alert_readback").await {
            Ok((data, attempts)) => request.verified_readback(&ids, data, attempts),
            Err(readback) => readback,
        };
    Ok(request.report(&ids, readback))
}
