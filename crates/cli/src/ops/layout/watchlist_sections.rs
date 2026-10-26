use serde_json::{Value, json};
use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};
use tradingview_model::watchlist::{
    WatchlistSectionChange, validate_watchlist_id, watchlist_entries,
};

const SCRIPT: &str = include_str!("watchlist_sections.js");

fn expression(request: &Value) -> String {
    format!("(async () => {{ const request = {request}; {SCRIPT} }})()")
}

fn normalize(mut result: Value) -> Result<Value, AppError> {
    if result.get("ok").and_then(Value::as_bool) == Some(false) {
        let kind = if result["error_kind"] == "validation" {
            ErrorKind::Validation
        } else {
            ErrorKind::InternalApiUnavailable
        };
        result.as_object_mut().unwrap().remove("ok");
        result.as_object_mut().unwrap().remove("error_kind");
        return Err(
            AppError::new(kind, "Watchlist operation was not confirmed").with_details(result)
        );
    }
    if result.get("ok").and_then(Value::as_bool) != Some(true) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Invalid watchlist operation response",
        ));
    }
    result.as_object_mut().unwrap().remove("ok");
    if let Some(watchlist) = result.get_mut("watchlist").filter(|v| v.is_object()) {
        watchlist["entries"] = watchlist_entries(&watchlist["symbols"])?;
    }
    Ok(result)
}

pub async fn watchlist_inspect(
    runtime: &mut impl RuntimeEvaluator,
    id: &str,
) -> Result<Value, AppError> {
    validate_watchlist_id(id)?;
    normalize(
        runtime
            .evaluate(&expression(&json!({"action": "inspect", "id": id})), true)
            .await?,
    )
}

pub async fn watchlist_section_change(
    runtime: &mut impl RuntimeEvaluator,
    request: &WatchlistSectionChange,
) -> Result<Value, AppError> {
    request.validate()?;
    let request = request.arguments();
    let unobserved = |kind| {
        AppError::new(kind, "Watchlist section execution could not be observed").with_details(
            json!({
                "contract_version": "watchlist_section_mutation.v1",
                "source": "desktop_watchlist_handler", "action": request["action"],
                "reason": "execution_unobserved", "write_attempted": null, "changed": null,
                "saved_confirmation": "unavailable", "display_confirmation": "unavailable"
            }),
        )
    };
    let raw = runtime
        .evaluate(&expression(&request), true)
        .await
        .map_err(|error| unobserved(error.kind))?;
    normalize(raw).map_err(|error| {
        if error
            .details
            .as_ref()
            .is_some_and(|details| details["contract_version"] == "watchlist_section_mutation.v1")
        {
            error
        } else {
            unobserved(error.kind)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::test_support::FakeRuntime;

    #[tokio::test]
    async fn section_validation_prevents_runtime_calls_and_preserves_uncertainty() {
        let mut runtime = FakeRuntime::new([]);
        let invalid = WatchlistSectionChange::Collapse {
            section: " ".into(),
        };
        assert_eq!(
            watchlist_section_change(&mut runtime, &invalid)
                .await
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert!(runtime.evaluated.is_empty());
        let mut runtime = FakeRuntime::new([]).with_evaluate_error(ErrorKind::Timeout);
        let valid = WatchlistSectionChange::Expand {
            section: "Example".into(),
        };
        let error = watchlist_section_change(&mut runtime, &valid)
            .await
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Timeout);
        assert!(error.details.unwrap()["write_attempted"].is_null());
        assert_eq!(runtime.evaluated.len(), 1);
        let mut runtime = FakeRuntime::new([Value::Null]);
        let error = watchlist_section_change(&mut runtime, &valid)
            .await
            .unwrap_err();
        assert!(error.details.unwrap()["changed"].is_null());
    }

    #[test]
    #[ignore = "run through scripts/check-account-js-contract.py with pinned Node.js"]
    fn javascript_account_watchlist_sections_verify_saved_and_displayed_state() {
        let script = format!(
            "const operation = async request => {{ {SCRIPT} }};\n{}",
            include_str!("watchlist_sections_fixture.js")
        );
        let output = std::process::Command::new("node")
            .args(["-e", &script])
            .output()
            .expect("Node.js is required for the account JavaScript contract");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
