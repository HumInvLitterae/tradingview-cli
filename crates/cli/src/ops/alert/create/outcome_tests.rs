use super::*;
use crate::ops::test_support::FakeRuntime;
use serde_json::json;
use tradingview_core::{ErrorEnvelope, ErrorKind};

#[tokio::test]
#[ignore = "requires the pinned Node.js account contract gate"]
async fn javascript_account_price_creation_outcomes() {
    let mut runtime = FakeRuntime::new([]);
    let _ = alert_create(&mut runtime, 100.0, "crossing", Some("Example")).await;
    let expression = serde_json::to_string(&runtime.evaluated[0].0).unwrap();
    let script = format!(
        "const expression = {expression};\n{}",
        include_str!("outcomes.js")
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
    let rows: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
    for row in rows {
        let mode = row["mode"].as_str().unwrap();
        let mut runtime = FakeRuntime::new([row["result"].clone()]);
        let result = alert_create(&mut runtime, 100.0, "crossing", Some("Example")).await;
        if mode == "success" {
            let data = result.unwrap();
            assert_eq!(data["created"], true);
            assert_eq!(data["alert_id"], "new");
            assert_eq!(data["price_set"], true);
            assert!(data.get("creation_outcome").is_none());
            assert!(data["matched_alert"]["condition"].get("series").is_none());
        } else {
            let error = result.unwrap_err();
            assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
            assert_eq!(error.exit_code(), 3);
            let envelope = serde_json::to_value(ErrorEnvelope::new("alert", error.into())).unwrap();
            assert_eq!(envelope["success"], false);
            let details = &envelope["error"]["details"];
            assert_eq!(details["phase"], row["result"]["phase"]);
            assert_eq!(details["api_fallback_allowed"], false);
            let preflight = mode.starts_with("pre_") || mode == "chart";
            assert_eq!(details["price_set"], !preflight);
            if preflight {
                assert_eq!(details["created"], false);
                assert!(details.get("creation_outcome").is_none());
            } else {
                assert_eq!(details.get("created"), Some(&Value::Null), "{mode}");
                assert_eq!(details["creation_outcome"], "unknown", "{mode}");
            }
        }
        assert_eq!(runtime.evaluated.len(), 1);
        assert!(runtime.mouse_events.is_empty());
        assert!(runtime.key_events.is_empty());
    }
}

#[tokio::test]
async fn price_creation_lost_runtime_result_preserves_error_and_unknown_outcome() {
    for kind in [
        ErrorKind::Timeout,
        ErrorKind::Connection,
        ErrorKind::InternalApiUnavailable,
    ] {
        let original = AppError::new(kind, "evaluation response unavailable")
            .with_details(json!({"failure_stage": "method_call"}));
        let exit_code = original.exit_code();
        let mut runtime = FakeRuntime::new([]).with_evaluate_app_error(original);
        let error = alert_create(&mut runtime, 100.0, "crossing", None)
            .await
            .unwrap_err();
        assert_eq!(error.kind, kind);
        assert_eq!(error.exit_code(), exit_code);
        assert_eq!(error.message, "evaluation response unavailable");
        let details = error.details.unwrap();
        assert_eq!(details["failure_stage"], "method_call");
        assert_eq!(details.get("created"), Some(&Value::Null));
        assert_eq!(details["creation_outcome"], "unknown");
        assert_eq!(runtime.evaluated.len(), 1);
    }
}

#[tokio::test]
async fn price_creation_malformed_result_does_not_prove_absence_of_write() {
    for response in [
        Value::Null,
        json!({}),
        json!("invalid"),
        json!([]),
        json!({"created": false}),
        json!({"created": false, "phase": "post_check_failed", "error": "unconfirmed"}),
        json!({"created": false, "phase": "pre_list_unavailable"}),
        json!({"created": true, "price_set": false}),
    ] {
        let mut runtime = FakeRuntime::new([response.clone()]);
        let error = alert_create(&mut runtime, 100.0, "crossing", None)
            .await
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        let details = error.details.unwrap();
        assert_eq!(details.get("created"), Some(&Value::Null), "{response}");
        assert_eq!(details["creation_outcome"], "unknown");
    }
}
