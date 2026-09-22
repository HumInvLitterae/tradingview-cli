//! One explicit watchlist mutation followed by one non-mutating readback.

use crate::{Failure, admission::Admission, http::Http, transport};
use serde_json::{Value, json};
use tradingview_core::AppError;
use tradingview_model::mcp_account::WatchlistMutation;

pub(crate) async fn change(
    request: &WatchlistMutation,
    http: &Http,
    token: String,
    admission: &mut Admission,
) -> Result<Value, AppError> {
    let mut session = transport::Session::connect(http, token).await?;
    let target = async {
        let mut responses = session
            .call(
                request.action().into(),
                &[request.arguments()],
                Some(admission),
            )
            .await?;
        let value = transport::result_value(responses.pop().ok_or(Failure::InvalidResponse)??)?;
        request.target_after_reply(&value)
    }
    .await;
    let target = match target {
        Ok(target) => target,
        Err(error) => return session.finish(Err(error)).await,
    };
    let Some(id) = target else {
        return session
            .finish(Ok(request.report(
                None,
                json!({
                    "status": "not_performed",
                    "reason": "target_id_unreported",
                    "tool_attempts": 0
                }),
            )))
            .await;
    };

    let read = request.readback(&id)?;
    let readback =
        match crate::account::readback(&read, http, session, admission, "watchlist_readback").await
        {
            Ok((data, attempts)) => request.verified_readback(&id, data, attempts),
            Err(readback) => readback,
        };
    Ok(request.report(Some(&id), readback))
}
