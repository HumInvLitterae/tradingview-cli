use std::time::{Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};
use tradingview_cdp::TransportConfig;
use tradingview_core::{AppError, ErrorKind, SuccessEnvelope};
use tradingview_model::replay::{MAX_REPLAY_LOG_STEPS, validate_replay_log_steps};

use crate::{
    app::{output::print_jsonl_stdout, runtime::connect_runtime},
    ops,
};

const REPLAY_STEP_LOG_CONTRACT_VERSION: &str = "replay_step_log.v1";
const REPLAY_STEP_LOG_SOURCE: &str = "internal_api";
const REPLAY_STEP_LOG_SOURCE_CATEGORY: &str = "desktop_backed_operation";
const REPLAY_STEP_LOG_LABEL: &str = "log";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReplayLogEndReason {
    StepLimitReached,
    ReplayNotStarted,
    ReplayUnavailable,
    StepFailed,
    Completed,
}

impl ReplayLogEndReason {
    fn label(self) -> &'static str {
        match self {
            Self::StepLimitReached => "step_limit_reached",
            Self::ReplayNotStarted => "replay_not_started",
            Self::ReplayUnavailable => "replay_unavailable",
            Self::StepFailed => "step_failed",
            Self::Completed => "completed",
        }
    }
}

pub async fn run_replay_log_command(steps: u64, config: &TransportConfig) -> Result<(), AppError> {
    validate_replay_log_steps(steps)?;
    let started_at = Instant::now();
    let mut runtime = connect_runtime(config).await?;

    let initial_status = ops::replay_status(&mut runtime).await?;
    let readiness = replay_log_readiness(steps, &initial_status)?;
    let envelope = SuccessEnvelope::new("replay", readiness);
    print_jsonl_stdout(&envelope);

    let started_at_replay_date = replay_current_date(&initial_status);
    let mut ended_at_replay_date = started_at_replay_date.clone();
    let mut last_context = initial_status
        .get("replay_context")
        .cloned()
        .unwrap_or(Value::Null);
    let mut step_count = 0_u64;
    let mut failure_count = 0_u64;
    let mut failure_details = None;

    let end_reason = if !replay_available(&initial_status) {
        ReplayLogEndReason::ReplayUnavailable
    } else if !replay_started(&initial_status) {
        ReplayLogEndReason::ReplayNotStarted
    } else {
        let mut reason = ReplayLogEndReason::Completed;
        for step_index in 1..=steps {
            match ops::replay_step(&mut runtime).await {
                Ok(step) => {
                    step_count += 1;
                    ended_at_replay_date = replay_current_date(&step);
                    last_context = step.get("replay_context").cloned().unwrap_or(Value::Null);
                    let payload = replay_log_step(step_index, step)?;
                    let envelope = SuccessEnvelope::new("replay", payload);
                    print_jsonl_stdout(&envelope);
                }
                Err(err) => {
                    failure_count += 1;
                    failure_details = Some(replay_failure_details(&err));
                    reason = ReplayLogEndReason::StepFailed;
                    break;
                }
            }
        }
        if reason == ReplayLogEndReason::Completed && step_count >= steps {
            ReplayLogEndReason::StepLimitReached
        } else {
            reason
        }
    };

    let summary = replay_log_summary(
        steps,
        step_count,
        failure_count,
        started_at_replay_date,
        ended_at_replay_date,
        last_context,
        failure_details,
        started_at.elapsed().as_millis() as u64,
        end_reason,
    )?;
    let envelope = SuccessEnvelope::new("replay", summary);
    print_jsonl_stdout(&envelope);
    Ok(())
}

fn replay_log_readiness(steps: u64, status: &Value) -> Result<Value, AppError> {
    let mut payload = json!({
        "requested_steps": steps,
        "max_steps": MAX_REPLAY_LOG_STEPS,
        "is_replay_available": status.get("is_replay_available").cloned().unwrap_or(Value::Null),
        "is_replay_started": status.get("is_replay_started").cloned().unwrap_or(Value::Null),
        "replay_context": status.get("replay_context").cloned().unwrap_or(Value::Null),
        "chart_context": status.get("chart_context").cloned().unwrap_or(Value::Null),
    });
    let Some(object) = payload.as_object_mut() else {
        return Err(AppError::new(
            ErrorKind::Internal,
            "Replay log readiness payload was not an object",
        ));
    };
    add_replay_log_metadata(object, "readiness")?;
    Ok(payload)
}

fn replay_log_step(step_index: u64, step: Value) -> Result<Value, AppError> {
    let mut payload = json!({
        "step_index": step_index,
        "operation": step.get("operation").cloned().unwrap_or_else(|| json!("replay_step")),
        "previous_date": step.get("previous_date").cloned().unwrap_or(Value::Null),
        "current_date": step.get("current_date").cloned().unwrap_or(Value::Null),
        "replay_context": step.get("replay_context").cloned().unwrap_or(Value::Null),
        "chart_context": step.get("chart_context").cloned().unwrap_or(Value::Null),
    });
    let Some(object) = payload.as_object_mut() else {
        return Err(AppError::new(
            ErrorKind::Internal,
            "Replay log step payload was not an object",
        ));
    };
    add_replay_log_metadata(object, "step")?;
    Ok(payload)
}

#[allow(clippy::too_many_arguments)]
fn replay_log_summary(
    requested_steps: u64,
    step_count: u64,
    failure_count: u64,
    started_at_replay_date: Value,
    ended_at_replay_date: Value,
    replay_context: Value,
    failure_details: Option<Value>,
    elapsed_ms: u64,
    end_reason: ReplayLogEndReason,
) -> Result<Value, AppError> {
    let mut payload = json!({
        "requested_steps": requested_steps,
        "step_count": step_count,
        "failure_count": failure_count,
        "started_at_replay_date": started_at_replay_date,
        "ended_at_replay_date": ended_at_replay_date,
        "elapsed_ms": elapsed_ms,
        "replay_left_running": replay_context_started(&replay_context),
        "replay_context": replay_context,
        "end_reason": end_reason.label(),
    });
    if let Some(failure_details) = failure_details {
        payload["failure_details"] = failure_details;
    }
    let Some(object) = payload.as_object_mut() else {
        return Err(AppError::new(
            ErrorKind::Internal,
            "Replay log summary payload was not an object",
        ));
    };
    add_replay_log_metadata(object, "summary")?;
    Ok(payload)
}

fn add_replay_log_metadata(
    object: &mut serde_json::Map<String, Value>,
    event: &str,
) -> Result<(), AppError> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))?
        .as_millis() as u64;
    object.insert("_replay".to_string(), json!(REPLAY_STEP_LOG_LABEL));
    object.insert("_event".to_string(), json!(event));
    object.insert("_ts".to_string(), json!(ts));
    object.insert(
        "contract_version".to_string(),
        json!(REPLAY_STEP_LOG_CONTRACT_VERSION),
    );
    object.insert("source".to_string(), json!(REPLAY_STEP_LOG_SOURCE));
    object.insert(
        "source_category".to_string(),
        json!(REPLAY_STEP_LOG_SOURCE_CATEGORY),
    );
    object.insert("requires_desktop".to_string(), json!(true));
    object.insert("non_mutating".to_string(), json!(false));
    Ok(())
}

fn replay_current_date(payload: &Value) -> Value {
    payload
        .get("replay_context")
        .and_then(|context| context.get("current_date"))
        .or_else(|| payload.get("current_date"))
        .cloned()
        .unwrap_or(Value::Null)
}

fn replay_available(payload: &Value) -> bool {
    payload
        .get("is_replay_available")
        .or_else(|| {
            payload
                .get("replay_context")
                .and_then(|context| context.get("is_replay_available"))
        })
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn replay_started(payload: &Value) -> bool {
    payload
        .get("is_replay_started")
        .or_else(|| {
            payload
                .get("replay_context")
                .and_then(|context| context.get("is_replay_started"))
        })
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn replay_context_started(context: &Value) -> bool {
    context
        .get("is_replay_started")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn replay_failure_details(error: &AppError) -> Value {
    json!({
        "kind": error.kind,
        "message": error.message,
        "details": error.details,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn started_status() -> Value {
        json!({
            "is_replay_available": true,
            "is_replay_started": true,
            "replay_context": {
                "is_replay_available": true,
                "is_replay_started": true,
                "current_date": 1775001600000_i64
            },
            "chart_context": {
                "symbol": "NASDAQ:AAPL",
                "timeframe": "D",
                "resolution": "D"
            }
        })
    }

    #[test]
    fn readiness_uses_replay_step_log_contract_metadata() {
        let readiness = replay_log_readiness(3, &started_status()).unwrap();
        assert_eq!(readiness["contract_version"], "replay_step_log.v1");
        assert_eq!(readiness["_replay"], "log");
        assert_eq!(readiness["_event"], "readiness");
        assert_eq!(readiness["requested_steps"], 3);
        assert_eq!(readiness["max_steps"], 100);
        assert_eq!(readiness["source"], "internal_api");
        assert_eq!(readiness["source_category"], "desktop_backed_operation");
        assert_eq!(readiness["requires_desktop"], true);
        assert_eq!(readiness["non_mutating"], false);
        assert_eq!(
            readiness["replay_context"]["current_date"],
            1775001600000_i64
        );
        assert_eq!(readiness["chart_context"]["symbol"], "NASDAQ:AAPL");
    }

    #[test]
    fn step_event_preserves_step_dates_and_context() {
        let step = replay_log_step(
            2,
            json!({
                "operation": "replay_step",
                "previous_date": 1775001600000_i64,
                "current_date": 1775088000000_i64,
                "replay_context": {
                    "is_replay_started": true,
                    "current_date": 1775088000000_i64
                },
                "chart_context": {
                    "symbol": "NASDAQ:AAPL"
                }
            }),
        )
        .unwrap();

        assert_eq!(step["contract_version"], "replay_step_log.v1");
        assert_eq!(step["_event"], "step");
        assert_eq!(step["step_index"], 2);
        assert_eq!(step["operation"], "replay_step");
        assert_eq!(step["previous_date"], 1775001600000_i64);
        assert_eq!(step["current_date"], 1775088000000_i64);
        assert_eq!(step["replay_context"]["current_date"], 1775088000000_i64);
    }

    #[test]
    fn summary_reports_counts_end_reason_and_left_running_state() {
        let summary = replay_log_summary(
            3,
            3,
            0,
            json!(1775001600000_i64),
            json!(1775260800000_i64),
            json!({"is_replay_started": true, "current_date": 1775260800000_i64}),
            None,
            1250,
            ReplayLogEndReason::StepLimitReached,
        )
        .unwrap();

        assert_eq!(summary["contract_version"], "replay_step_log.v1");
        assert_eq!(summary["_event"], "summary");
        assert_eq!(summary["requested_steps"], 3);
        assert_eq!(summary["step_count"], 3);
        assert_eq!(summary["failure_count"], 0);
        assert_eq!(summary["replay_left_running"], true);
        assert_eq!(summary["end_reason"], "step_limit_reached");
        assert!(summary.get("failure_details").is_none());
    }

    #[test]
    fn summary_can_include_sanitized_failure_details() {
        let error = AppError::new(ErrorKind::Validation, "Replay is not started")
            .with_details(json!({"operation": "replay_step"}));
        let summary = replay_log_summary(
            3,
            1,
            1,
            json!(1775001600000_i64),
            json!(1775088000000_i64),
            json!({"is_replay_started": true}),
            Some(replay_failure_details(&error)),
            500,
            ReplayLogEndReason::StepFailed,
        )
        .unwrap();

        assert_eq!(summary["end_reason"], "step_failed");
        assert_eq!(summary["failure_details"]["kind"], "validation");
        assert_eq!(
            summary["failure_details"]["message"],
            "Replay is not started"
        );
        assert_eq!(
            summary["failure_details"]["details"]["operation"],
            "replay_step"
        );
    }

    #[test]
    fn end_reason_labels_are_stable() {
        assert_eq!(
            ReplayLogEndReason::StepLimitReached.label(),
            "step_limit_reached"
        );
        assert_eq!(
            ReplayLogEndReason::ReplayNotStarted.label(),
            "replay_not_started"
        );
        assert_eq!(
            ReplayLogEndReason::ReplayUnavailable.label(),
            "replay_unavailable"
        );
        assert_eq!(ReplayLogEndReason::StepFailed.label(), "step_failed");
        assert_eq!(ReplayLogEndReason::Completed.label(), "completed");
    }
}
