use super::*;
use crate::ops::test_support::FakeRuntime;

const NAME: &str = r#"Synthetic 'saved' "script""#;
const SOURCE: &str = "//@version=6\nindicator(\"Synthetic\")\nplot(close)\n";

fn saved_result() -> Value {
    json!({
        "status": "saved", "script": {"id": "synthetic-new", "name": NAME, "version": "1.0"},
        "saved_source": SOURCE,
        "compilation": {"compiled": true, "errors": [], "warnings": []}
    })
}

#[tokio::test]
async fn pine_create_validates_before_evaluation() {
    for (name, source) in [(" ", SOURCE), (NAME, "\n ")] {
        let mut runtime = FakeRuntime::new([]);
        assert_eq!(
            pine_create(&mut runtime, name, source)
                .await
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert!(runtime.evaluated.is_empty());
    }
}

#[tokio::test]
async fn pine_create_checks_saved_source_without_exposing_it() {
    let mut raw = saved_result();
    raw["saved_source"] = json!(SOURCE.replace('\n', "\r\n"));
    let mut runtime = FakeRuntime::new([raw]);
    let result = pine_create(&mut runtime, &format!(" {NAME} "), SOURCE)
        .await
        .unwrap();
    assert_eq!(result["saved"], true);
    assert_eq!(result["script"]["name"], NAME);
    assert_eq!(result["source_verified"], true);
    assert_eq!(result["compilation"]["compiled"], true);
    assert!(!result.to_string().contains("plot(close)"));
    assert_eq!(runtime.evaluated.len(), 1);
    assert!(runtime.evaluated[0].1);
    assert!(runtime.key_events.is_empty());
    assert!(runtime.inserted_text.is_empty());
}

#[tokio::test]
async fn pine_create_preserves_unknown_runtime_outcomes_and_failure_stage() {
    for kind in [
        ErrorKind::Connection,
        ErrorKind::Timeout,
        ErrorKind::Internal,
    ] {
        let error = AppError::new(kind, "synthetic-private-source")
            .with_details(json!({"failure_stage": "method_call", "raw": "private"}));
        let mut runtime = FakeRuntime::new([]).with_evaluate_app_error(error);
        let error = pine_create(&mut runtime, NAME, SOURCE).await.unwrap_err();
        assert_eq!(error.kind, kind);
        let details = error.details.unwrap();
        assert!(details["saved"].is_null());
        assert_eq!(details["save_outcome"], "unknown");
        assert_eq!(details["failure_stage"], "method_call");
        assert!(details.get("raw").is_none());
        assert!(!error.message.contains("private"));
        assert_eq!(runtime.evaluated.len(), 1);
    }
}

#[test]
fn pine_create_rejects_incomplete_or_mismatched_success() {
    let mut variants = vec![Value::Null, json!({}), json!({"status": "unknown"})];
    for (pointer, replacement) in [
        ("/script/id", json!("")),
        ("/script/name", json!("other")),
        ("/script/version", json!(null)),
        ("/saved_source", json!(SOURCE.trim_end())),
    ] {
        let mut raw = saved_result();
        *raw.pointer_mut(pointer).unwrap() = replacement;
        variants.push(raw);
    }
    for raw in variants {
        let error = normalize_result(raw, NAME, SOURCE).unwrap_err();
        assert!(error.details.as_ref().unwrap()["saved"].is_null());
        assert_eq!(error.details.unwrap()["save_outcome"], "unknown");
    }
}

#[test]
fn pine_create_distinguishes_saved_from_compiled() {
    for (compilation, expected) in [
        (
            json!({"compiled": false, "errors": [{"message": "Syntax error"}], "warnings": []}),
            json!(false),
        ),
        (
            json!({"compiled": true, "errors": [], "warnings": [{"message": "Warning", "raw": "private"}]}),
            json!(true),
        ),
        (
            json!({"compiled": true, "errors": [{"message": "Error"}], "warnings": []}),
            Value::Null,
        ),
        (json!({"compiled": true, "errors": []}), Value::Null),
        (
            json!({"compiled": true, "errors": [], "warnings": [null]}),
            Value::Null,
        ),
        (Value::Null, Value::Null),
    ] {
        let mut raw = saved_result();
        raw["compilation"] = compilation;
        let result = normalize_result(raw, NAME, SOURCE).unwrap();
        assert_eq!(result["saved"], true);
        assert_eq!(result["compilation"]["compiled"], expected);
        assert!(!result.to_string().contains("private"));
    }
}

#[tokio::test]
#[ignore = "run through scripts/check-pine-open-js-contract.py with pinned Node.js"]
async fn javascript_pine_create_contract() {
    let mut runtime = FakeRuntime::new([]);
    let _ = pine_create(&mut runtime, NAME, SOURCE).await;
    let fixture = include_str!("fixture.js")
        .replace(
            "__EXPRESSION__",
            &serde_json::to_string(&runtime.evaluated[0].0).unwrap(),
        )
        .replace("__NAME__", &serde_json::to_string(NAME).unwrap())
        .replace("__SOURCE__", &serde_json::to_string(SOURCE).unwrap());
    let output = std::process::Command::new("node")
        .args(["-e", &fixture])
        .output()
        .expect("Node.js is required for the Pine JavaScript contract");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let cases: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
    for case in &cases {
        let mut runtime = FakeRuntime::new([case["raw"].clone()]);
        let result = pine_create(&mut runtime, NAME, SOURCE).await;
        match case["expected"].as_str().unwrap() {
            "saved" => assert_eq!(result.unwrap()["saved"], true, "{case}"),
            "not_saved" => assert_eq!(
                result.unwrap_err().details.unwrap()["saved"],
                false,
                "{case}"
            ),
            "unknown" => assert!(
                result.unwrap_err().details.unwrap()["saved"].is_null(),
                "{case}"
            ),
            _ => unreachable!(),
        }
        assert_eq!(runtime.evaluated.len(), 1);
    }
    eprintln!("{} Pine creation JavaScript scenarios passed", cases.len());
}
