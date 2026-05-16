use serde_json::Value;

use tradingview_core::{AppError, ErrorKind};

const OBSERVE_CHART_CONTRACT_VERSION: &str = "observe_chart.v1";

pub fn observe_readiness_event(mut readiness: Value) -> Result<Value, AppError> {
    let Some(object) = readiness.as_object_mut() else {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Readiness payload was not an object",
        )
        .with_details(readiness));
    };
    object.insert(
        "contract_version".to_string(),
        Value::String(OBSERVE_CHART_CONTRACT_VERSION.to_string()),
    );
    object.insert("_event".to_string(), Value::String("readiness".to_string()));
    object.insert("_observe".to_string(), Value::String("chart".to_string()));
    Ok(readiness)
}

pub fn observe_chart_event(mut event: Value) -> Result<Value, AppError> {
    let Some(object) = event.as_object_mut() else {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Observe chart payload was not an object",
        )
        .with_details(event));
    };
    object.insert(
        "contract_version".to_string(),
        Value::String(OBSERVE_CHART_CONTRACT_VERSION.to_string()),
    );
    object.insert("_observe".to_string(), Value::String("chart".to_string()));
    Ok(event)
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
        assert_eq!(event["contract_version"], "observe_chart.v1");
        assert_eq!(event["source"], "desktop_readiness");
        assert_eq!(event["ready"], true);
    }

    #[test]
    fn observe_readiness_event_rejects_non_object_payload() {
        let error = observe_readiness_event(json!(["not", "object"])).unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.message, "Readiness payload was not an object");
    }

    #[test]
    fn observe_chart_event_marks_sample_payload_without_losing_stream_metadata() {
        let event = observe_chart_event(json!({
            "contract_version": "stream.v1",
            "_event": "sample",
            "_stream": "bars",
            "source": "desktop_chart_stream",
            "source_category": "desktop_backed_read",
            "requires_desktop": true,
            "non_mutating": true
        }))
        .unwrap();

        assert_eq!(event["contract_version"], "observe_chart.v1");
        assert_eq!(event["_event"], "sample");
        assert_eq!(event["_observe"], "chart");
        assert_eq!(event["_stream"], "bars");
        assert_eq!(event["source"], "desktop_chart_stream");
        assert_eq!(event["source_category"], "desktop_backed_read");
        assert_eq!(event["requires_desktop"], true);
        assert_eq!(event["non_mutating"], true);
    }

    #[test]
    fn observe_chart_event_rejects_non_object_payload() {
        let error = observe_chart_event(json!("not an object")).unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.message, "Observe chart payload was not an object");
    }
}
