use serde_json::Value;

use tradingview_core::{AppError, ErrorKind};

pub fn observe_readiness_event(mut readiness: Value) -> Result<Value, AppError> {
    let Some(object) = readiness.as_object_mut() else {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Readiness payload was not an object",
        )
        .with_details(readiness));
    };
    object.insert("_event".to_string(), Value::String("readiness".to_string()));
    object.insert("_observe".to_string(), Value::String("chart".to_string()));
    Ok(readiness)
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tradingview_core::ErrorKind;

    use super::*;

    #[test]
    fn observe_readiness_event_marks_payload() {
        let event = observe_readiness_event(json!({
            "source": "desktop_readiness",
            "ready": true
        }))
        .unwrap();

        assert_eq!(event["_event"], "readiness");
        assert_eq!(event["_observe"], "chart");
        assert_eq!(event["source"], "desktop_readiness");
        assert_eq!(event["ready"], true);
    }

    #[test]
    fn observe_readiness_event_rejects_non_object_payload() {
        let error = observe_readiness_event(json!(["not", "object"])).unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.message, "Readiness payload was not an object");
    }
}
