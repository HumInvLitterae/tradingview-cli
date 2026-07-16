use serde_json::{Value, json};

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};

use super::runtime::{ensure_pine_editor_open, with_monaco};

pub async fn pine_get(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    let open_state = ensure_pine_editor_open(runtime).await?;
    let value = evaluate_pine_source(
        runtime,
        &with_monaco(PINE_GET_SOURCE_EXPRESSION),
        "get",
        "source_readback",
    )
    .await?;
    let source = pine_source_string(
        &value,
        "get",
        "source_readback",
        "Monaco editor found but source was not a string",
    )?;

    Ok(json!({
        "source": source,
        "line_count": source.split('\n').count(),
        "char_count": source.chars().count(),
        "editor_open_before": open_state.editor_open_before,
        "opened_editor": open_state.opened_editor,
    }))
}

pub async fn pine_set(
    runtime: &mut impl RuntimeEvaluator,
    source: &str,
    input_source: &str,
) -> Result<Value, AppError> {
    let open_state = ensure_pine_editor_open(runtime).await?;
    let value = evaluate_pine_source(
        runtime,
        &pine_set_source_expression(source),
        "set",
        "source_verification",
    )
    .await?;
    let observed_source = pine_source_string(
        &value,
        "set",
        "source_verification",
        "Monaco editor found but set source verification was not a string",
    )?;

    if !pine_sources_match(source, observed_source) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Pine source set verification failed",
        )
        .with_details(json!({
            "expected_char_count": source.chars().count(),
            "observed_char_count": observed_source.chars().count(),
        })));
    }

    Ok(json!({
        "lines_set": source.split('\n').count(),
        "char_count": source.chars().count(),
        "input_source": input_source,
        "editor_open_before": open_state.editor_open_before,
        "opened_editor": open_state.opened_editor,
    }))
}

pub fn validate_pine_script_type(script_type: &str) -> Result<&'static str, AppError> {
    match script_type.trim().to_ascii_lowercase().as_str() {
        "indicator" => Ok("indicator"),
        "strategy" => Ok("strategy"),
        "library" => Ok("library"),
        _ => Err(AppError::new(
            ErrorKind::Validation,
            "Pine script type must be one of: indicator, strategy, library",
        )
        .with_details(json!({ "script_type": script_type }))),
    }
}

pub async fn pine_new(
    runtime: &mut impl RuntimeEvaluator,
    script_type: &str,
) -> Result<Value, AppError> {
    let script_type = validate_pine_script_type(script_type)?;
    let template = pine_template(script_type);
    let open_state = ensure_pine_editor_open(runtime).await?;
    let value = evaluate_pine_source(
        runtime,
        &pine_set_source_expression(template),
        "new",
        "source_verification",
    )
    .await?;
    let observed_source = pine_source_string(
        &value,
        "new",
        "source_verification",
        "Monaco editor found but new script verification was not a string",
    )?;

    if !pine_sources_match(template, observed_source) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Pine new script verification failed",
        )
        .with_details(json!({
            "expected_char_count": template.chars().count(),
            "observed_char_count": observed_source.chars().count(),
        })));
    }

    Ok(json!({
        "type": script_type,
        "action": "new_script_created",
        "template": template,
        "lines_set": template.split('\n').count(),
        "char_count": template.chars().count(),
        "editor_open_before": open_state.editor_open_before,
        "opened_editor": open_state.opened_editor,
    }))
}

fn pine_set_source_expression(source: &str) -> String {
    let source = serde_json::to_string(source).expect("string serialization should not fail");
    with_monaco(&format!(
        r#"
var m = __FIND_MONACO__;
if (!m) return null;
m.editor.setValue({source});
return m.editor.getValue();
"#
    ))
}

async fn evaluate_pine_source(
    runtime: &mut impl RuntimeEvaluator,
    expression: &str,
    operation: &'static str,
    stage: &'static str,
) -> Result<Value, AppError> {
    runtime.evaluate(expression, false).await.map_err(|error| {
        AppError::new(error.kind, "Pine source evaluation failed").with_details(json!({
            "operation": operation,
            "stage": stage,
        }))
    })
}

fn pine_source_string<'a>(
    value: &'a Value,
    operation: &'static str,
    stage: &'static str,
    message: &'static str,
) -> Result<&'a str, AppError> {
    value.as_str().ok_or_else(|| {
        AppError::new(ErrorKind::InternalApiUnavailable, message).with_details(json!({
            "operation": operation,
            "stage": stage,
            "response_type": pine_source_response_type(value),
        }))
    })
}

fn pine_source_response_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn pine_sources_match(expected: &str, observed: &str) -> bool {
    normalize_pine_line_endings(expected) == normalize_pine_line_endings(observed)
}

fn normalize_pine_line_endings(source: &str) -> String {
    source.replace("\r\n", "\n").replace('\r', "\n")
}

fn pine_template(script_type: &str) -> &'static str {
    match script_type {
        "strategy" => "//@version=6\nstrategy(\"My strategy\", overlay=true)\n",
        "library" => {
            "//@version=6\n// @description TODO: add library description here\nlibrary(\"MyLibrary\")\n"
        }
        _ => "//@version=6\nindicator(\"My script\")\nplot(close)",
    }
}

const PINE_GET_SOURCE_EXPRESSION: &str = r#"
var m = __FIND_MONACO__;
if (!m) return null;
return m.editor.getValue();
"#;

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::ops::test_support::FakeRuntime;

    use super::*;

    #[tokio::test]
    async fn pine_get_returns_source_counts_and_open_state() {
        let mut runtime = FakeRuntime::new([json!(true), json!("//@version=6\nindicator(\"X\")")]);

        let result = pine_get(&mut runtime).await.unwrap();

        assert_eq!(result["source"], "//@version=6\nindicator(\"X\")");
        assert_eq!(result["line_count"], 2);
        assert_eq!(result["char_count"], 27);
        assert_eq!(result["editor_open_before"], true);
        assert_eq!(result["opened_editor"], false);
        assert!(runtime.evaluated[1].0.contains("getValue"));
    }

    #[tokio::test]
    async fn pine_get_opens_editor_when_needed() {
        let mut runtime = FakeRuntime::new([
            json!(false),
            json!(true),
            json!(false),
            json!(true),
            json!("plot(close)"),
        ]);

        let result = pine_get(&mut runtime).await.unwrap();

        assert_eq!(result["source"], "plot(close)");
        assert_eq!(result["editor_open_before"], false);
        assert_eq!(result["opened_editor"], true);
        assert!(runtime.evaluated[1].0.contains("activateScriptEditorTab"));
    }

    #[tokio::test]
    async fn pine_set_updates_source_and_returns_counts() {
        let source = "//@version=6\nindicator(\"Quoted \\\"X\\\"\")\nplot(close)";
        let mut runtime = FakeRuntime::new([json!(true), json!(source)]);

        let result = pine_set(&mut runtime, source, "stdin").await.unwrap();

        assert_eq!(result["lines_set"], 3);
        assert_eq!(result["char_count"], source.chars().count());
        assert_eq!(result["input_source"], "stdin");
        assert_eq!(result["editor_open_before"], true);
        assert_eq!(result["opened_editor"], false);
        assert!(runtime.evaluated[1].0.contains("setValue"));
        let serialized_source = serde_json::to_string(source).unwrap();
        assert!(runtime.evaluated[1].0.contains(&serialized_source));
    }

    #[tokio::test]
    async fn pine_set_accepts_monaco_line_ending_normalization() {
        let source = "//@version=6\r\nindicator(\"X\")\nplot(close)\r\n// matrix\n";
        let observed = "//@version=6\r\nindicator(\"X\")\r\nplot(close)\r\n// matrix\r\n";
        let mut runtime = FakeRuntime::new([json!(true), json!(observed)]);

        let result = pine_set(&mut runtime, source, "file").await.unwrap();

        assert_eq!(result["char_count"], source.chars().count());
        assert_eq!(result["lines_set"], 5);
    }

    #[tokio::test]
    async fn pine_set_opens_editor_when_needed() {
        let mut runtime = FakeRuntime::new([
            json!(false),
            json!(true),
            json!(false),
            json!(true),
            json!("plot(close)"),
        ]);

        let result = pine_set(&mut runtime, "plot(close)", "file").await.unwrap();

        assert_eq!(result["lines_set"], 1);
        assert_eq!(result["input_source"], "file");
        assert_eq!(result["editor_open_before"], false);
        assert_eq!(result["opened_editor"], true);
    }

    #[tokio::test]
    async fn pine_set_errors_when_verification_differs() {
        let mut runtime = FakeRuntime::new([json!(true), json!("plot(open)")]);

        let error = pine_set(&mut runtime, "plot(close)", "stdin")
            .await
            .unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.message, "Pine source set verification failed");
    }

    #[tokio::test]
    async fn pine_source_operations_sanitize_runtime_evaluation_failures() {
        const PRIVATE_MARKER: &str = "private-runtime-marker";

        for (operation, mut runtime) in [
            (
                "get",
                FakeRuntime::new([json!(true)]).with_evaluate_app_error_after_responses(
                    AppError::new(ErrorKind::Timeout, PRIVATE_MARKER).with_details(json!({
                        "source": PRIVATE_MARKER,
                        "scriptId": PRIVATE_MARKER,
                    })),
                ),
            ),
            (
                "set",
                FakeRuntime::new([json!(true)]).with_evaluate_app_error_after_responses(
                    AppError::new(ErrorKind::Timeout, PRIVATE_MARKER).with_details(json!({
                        "source": PRIVATE_MARKER,
                        "scriptId": PRIVATE_MARKER,
                    })),
                ),
            ),
            (
                "new",
                FakeRuntime::new([json!(true)]).with_evaluate_app_error_after_responses(
                    AppError::new(ErrorKind::Timeout, PRIVATE_MARKER).with_details(json!({
                        "source": PRIVATE_MARKER,
                        "scriptId": PRIVATE_MARKER,
                    })),
                ),
            ),
        ] {
            let error = match operation {
                "get" => pine_get(&mut runtime).await.unwrap_err(),
                "set" => pine_set(&mut runtime, "plot(close)", "stdin")
                    .await
                    .unwrap_err(),
                "new" => pine_new(&mut runtime, "indicator").await.unwrap_err(),
                _ => unreachable!(),
            };

            assert_eq!(error.kind, ErrorKind::Timeout);
            assert_eq!(error.message, "Pine source evaluation failed");
            assert_eq!(error.details.as_ref().unwrap()["operation"], operation);
            let serialized = serde_json::to_string(&error.details).unwrap();
            assert!(!serialized.contains(PRIVATE_MARKER));
            assert!(!serialized.contains("scriptId"));
        }
    }

    #[tokio::test]
    async fn pine_source_operations_sanitize_malformed_runtime_values() {
        const PRIVATE_MARKER: &str = "private-payload-marker";
        let malformed = json!({
            "source": PRIVATE_MARKER,
            "scriptId": PRIVATE_MARKER,
            "raw": PRIVATE_MARKER,
        });

        for (operation, mut runtime) in [
            ("get", FakeRuntime::new([json!(true), malformed.clone()])),
            ("set", FakeRuntime::new([json!(true), malformed.clone()])),
            ("new", FakeRuntime::new([json!(true), malformed.clone()])),
        ] {
            let error = match operation {
                "get" => pine_get(&mut runtime).await.unwrap_err(),
                "set" => pine_set(&mut runtime, "plot(close)", "stdin")
                    .await
                    .unwrap_err(),
                "new" => pine_new(&mut runtime, "indicator").await.unwrap_err(),
                _ => unreachable!(),
            };

            assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
            assert_eq!(error.details.as_ref().unwrap()["operation"], operation);
            assert_eq!(error.details.as_ref().unwrap()["response_type"], "object");
            let serialized = serde_json::to_string(&error.details).unwrap();
            assert!(!serialized.contains(PRIVATE_MARKER));
            assert!(!serialized.contains("scriptId"));
            assert!(!serialized.contains("raw"));
        }
    }

    #[test]
    fn pine_source_verification_normalizes_crlf_and_lone_cr_only() {
        assert!(pine_sources_match("a\nb\n", "a\r\nb\r\n"));
        assert!(pine_sources_match("a\rb", "a\nb"));
        assert!(!pine_sources_match("plot(close)\n", "plot(open)\r\n"));
    }

    #[test]
    fn validate_pine_script_type_accepts_known_types() {
        assert_eq!(validate_pine_script_type("indicator").unwrap(), "indicator");
        assert_eq!(validate_pine_script_type("STRATEGY").unwrap(), "strategy");
        assert_eq!(validate_pine_script_type(" library ").unwrap(), "library");
    }

    #[test]
    fn validate_pine_script_type_rejects_unknown_type() {
        let error = validate_pine_script_type("study").unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(error.message.contains("indicator, strategy, library"));
    }

    #[tokio::test]
    async fn pine_new_sets_indicator_template_by_default_shape() {
        let template = pine_template("indicator");
        let mut runtime = FakeRuntime::new([json!(true), json!(template)]);

        let result = pine_new(&mut runtime, "indicator").await.unwrap();

        assert_eq!(result["type"], "indicator");
        assert_eq!(result["action"], "new_script_created");
        assert_eq!(result["template"], template);
        assert_eq!(result["lines_set"], 3);
        assert_eq!(result["char_count"], template.chars().count());
        assert_eq!(result["editor_open_before"], true);
        assert_eq!(result["opened_editor"], false);
        assert!(runtime.evaluated[1].0.contains("setValue"));
    }

    #[tokio::test]
    async fn pine_new_accepts_monaco_line_ending_normalization() {
        let template = pine_template("strategy");
        let observed = template.replace('\n', "\r\n");
        let mut runtime = FakeRuntime::new([json!(true), json!(observed)]);

        let result = pine_new(&mut runtime, "strategy").await.unwrap();

        assert_eq!(result["type"], "strategy");
        assert_eq!(result["char_count"], template.chars().count());
    }

    #[tokio::test]
    async fn pine_new_supports_strategy_and_library_templates() {
        let strategy = pine_template("strategy");
        let mut runtime = FakeRuntime::new([json!(true), json!(strategy)]);

        let result = pine_new(&mut runtime, "strategy").await.unwrap();

        assert_eq!(result["type"], "strategy");
        assert!(result["template"].as_str().unwrap().contains("strategy("));

        let library = pine_template("library");
        let mut runtime = FakeRuntime::new([json!(true), json!(library)]);

        let result = pine_new(&mut runtime, "library").await.unwrap();

        assert_eq!(result["type"], "library");
        assert!(result["template"].as_str().unwrap().contains("library("));
    }

    #[tokio::test]
    async fn pine_new_errors_when_verification_differs() {
        let mut runtime = FakeRuntime::new([json!(true), json!("plot(open)")]);

        let error = pine_new(&mut runtime, "indicator").await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.message, "Pine new script verification failed");
    }
}
