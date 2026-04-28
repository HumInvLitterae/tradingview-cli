use serde_json::{Value, json};

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::AppError;

pub async fn ui_eval(
    runtime: &mut impl RuntimeEvaluator,
    expression: &str,
) -> Result<Value, AppError> {
    let result = runtime.evaluate(expression, true).await?;
    Ok(json!({
        "result": result,
        "unsafe_eval_enabled": true,
    }))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::super::test_support::FakeRuntime;
    use super::*;

    #[tokio::test]
    async fn ui_eval_returns_runtime_result() {
        let mut runtime = FakeRuntime::new([json!(2)]);

        let result = ui_eval(&mut runtime, "1+1").await.unwrap();

        assert_eq!(result["result"], 2);
        assert_eq!(result["unsafe_eval_enabled"], true);
        assert_eq!(runtime.evaluated[0].0, "1+1");
        assert!(runtime.evaluated[0].1);
    }
}
