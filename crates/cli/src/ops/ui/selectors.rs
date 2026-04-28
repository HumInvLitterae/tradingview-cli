use serde_json::{Value, json};

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};

use super::super::common::{js_string, require_finite};

#[derive(Debug)]
pub(super) struct ElementCoordinates {
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) tag: String,
}

pub(super) async fn ui_element_coordinates(
    runtime: &mut impl RuntimeEvaluator,
    by: &str,
    value: &str,
) -> Result<ElementCoordinates, AppError> {
    validate_selector_strategy(by, &["text", "aria-label", "data-name", "class-contains"])?;
    if value.trim().is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Value must not be empty",
        ));
    }
    let by_literal = js_string(by)?;
    let value_literal = js_string(value)?;
    let result = runtime
        .evaluate(
            &format!(
                r#"
            (function() {{
                var by = {by_literal};
                var value = {value_literal};
                var element = null;
                function textOf(el) {{
                    return (el.textContent || el.innerText || '').trim();
                }}
                if (by === 'aria-label') element = document.querySelector('[aria-label="' + CSS.escape(value) + '"]') || document.querySelector('[aria-label*="' + value.replace(/"/g, '\\\\"') + '"]');
                else if (by === 'data-name') element = document.querySelector('[data-name="' + CSS.escape(value) + '"]');
                else if (by === 'class-contains') element = document.querySelector('[class*="' + value.replace(/"/g, '\\\\"') + '"]');
                else {{
                    var candidates = Array.from(document.querySelectorAll('button, a, [role="button"], [role="menuitem"], [role="tab"], span, div'));
                    for (var i = 0; i < candidates.length; i++) {{
                        var text = textOf(candidates[i]);
                        if (text === value || text.toLowerCase() === value.toLowerCase()) {{
                            element = candidates[i];
                            break;
                        }}
                    }}
                }}
                if (!element) return null;
                var rect = element.getBoundingClientRect();
                return {{
                    x: rect.x + rect.width / 2,
                    y: rect.y + rect.height / 2,
                    tag: element.tagName.toLowerCase()
                }};
            }})()
            "#
            ),
            false,
        )
        .await?;
    if result.is_null() {
        return Err(AppError::new(
            ErrorKind::Validation,
            format!("Element not found for {by}=\"{value}\""),
        ));
    }
    Ok(ElementCoordinates {
        x: number_field(&result, "x")?,
        y: number_field(&result, "y")?,
        tag: result
            .get("tag")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
    })
}

pub(super) fn validate_selector_strategy(value: &str, allowed: &[&str]) -> Result<(), AppError> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorKind::Validation,
            format!("Unsupported selector strategy: {value}"),
        )
        .with_details(json!({ "supported": allowed })))
    }
}

pub(super) fn number_field(value: &Value, field: &str) -> Result<f64, AppError> {
    let number = value.get(field).and_then(Value::as_f64).ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!("UI payload did not include numeric {field}"),
        )
        .with_details(value.clone())
    })?;
    require_finite(number, field)?;
    Ok(number)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::super::test_support::FakeRuntime;
    use super::*;

    #[test]
    fn selector_strategy_validation_rejects_unknown_values() {
        let err = validate_selector_strategy("xpath", &["text", "css"]).unwrap_err();

        assert_eq!(err.kind, ErrorKind::Validation);
        assert_eq!(err.details.unwrap()["supported"], json!(["text", "css"]));
    }

    #[tokio::test]
    async fn element_coordinates_rejects_missing_element() {
        let mut runtime = FakeRuntime::new([Value::Null]);

        let err = ui_element_coordinates(&mut runtime, "text", "Missing")
            .await
            .unwrap_err();

        assert_eq!(err.kind, ErrorKind::Validation);
    }

    #[test]
    fn number_field_requires_numeric_finite_value() {
        let err = number_field(&json!({"x": "bad"}), "x").unwrap_err();
        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);

        let err = number_field(&json!({"x": f64::NAN}), "x").unwrap_err();
        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);

        assert_eq!(number_field(&json!({"x": 1.5}), "x").unwrap(), 1.5);
    }
}
