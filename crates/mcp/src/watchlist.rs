//! One explicit watchlist mutation followed by one non-mutating readback.

use crate::{Failure, admission::Admission, http::Http, transport};
use serde_json::{Value, json};
use tradingview_core::AppError;
use tradingview_model::mcp_account::{self, WatchlistMutation};

pub(crate) async fn change(
    request: &WatchlistMutation,
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
    let target = request.target_after_reply(&value)?;
    let Some(id) = target else {
        return Ok(request.report(
            None,
            json!({
                "status": "not_performed",
                "reason": "target_id_unreported",
                "tool_attempts": 0
            }),
        ));
    };

    let read = request.readback(&id)?;
    let before = http.tool_attempts()?;
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
        mcp_account::normalize(&read, value, crate::admission::now_ms()?)
    }
    .await;
    let attempts = http.tool_attempts()?.saturating_sub(before);
    let readback = match result {
        Ok(data) => request.verified_readback(&id, data, attempts),
        Err(error) => {
            let error = if let Some(failure) = error
                .details
                .as_ref()
                .and_then(|details| details.get("failure"))
                .and_then(|value| serde_json::from_value::<Failure>(value.clone()).ok())
            {
                crate::client::failure(failure, "watchlist_readback", attempts, Some(http))
            } else {
                error
            };
            json!({
                "status": "failed",
                "tool_attempts": attempts,
                "error": error.details,
                "next_action": "read the target explicitly; do not repeat the mutation automatically"
            })
        }
    };
    Ok(request.report(Some(&id), readback))
}
