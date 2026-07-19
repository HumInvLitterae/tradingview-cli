use std::{
    fs,
    path::{Path, PathBuf},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use serde_json::{Value, json};
use tradingview_cdp::TransportConfig;
use tradingview_core::{AppError, ErrorKind, SuccessEnvelope};
use tradingview_model::replay::{MAX_REPLAY_LOG_STEPS, validate_replay_log_steps};

use crate::{
    app::{
        output::{JsonlOutput, JsonlRunError, OutputDisposition, emit_jsonl_stdout},
        runtime::connect_runtime,
    },
    ops,
};

const REPLAY_STEP_LOG_CONTRACT_VERSION: &str = "replay_step_log.v1";
const REPLAY_STEP_LOG_SOURCE: &str = "internal_api";
const REPLAY_STEP_LOG_SOURCE_CATEGORY: &str = "desktop_backed_operation";
const REPLAY_STEP_LOG_LABEL: &str = "log";
const REPLAY_LOG_OHLCV_ATTACHMENT_CONTRACT_VERSION: &str = "replay_log_ohlcv_summary_attachment.v1";
const REPLAY_LOG_OHLCV_ATTACHMENT_SOURCE: &str = "selected_chart_cdp";
const REPLAY_LOG_SCREENSHOT_ATTACHMENT_CONTRACT_VERSION: &str =
    "replay_log_chart_screenshot_attachment.v1";
const DEFAULT_REPLAY_LOG_OHLCV_COUNT: usize = 100;
const MAX_REPLAY_LOG_OHLCV_COUNT: usize = 500;

#[derive(Debug, Clone, Copy)]
struct ReplayLogAttachmentControls {
    attach_ohlcv_summary: bool,
    ohlcv_count: usize,
}

impl ReplayLogAttachmentControls {
    fn new(attach_ohlcv_summary: bool, ohlcv_count: Option<usize>) -> Result<Self, AppError> {
        if !attach_ohlcv_summary && ohlcv_count.is_some() {
            return Err(AppError::new(
                ErrorKind::Validation,
                "replay log --ohlcv-count requires --attach-ohlcv-summary",
            )
            .with_details(json!({
                "field": "ohlcv_count",
                "requires": "attach_ohlcv_summary",
                "source_category": REPLAY_STEP_LOG_SOURCE_CATEGORY,
                "requires_desktop": true,
                "non_mutating": false
            })));
        }
        let ohlcv_count = ohlcv_count.unwrap_or(DEFAULT_REPLAY_LOG_OHLCV_COUNT);
        if ohlcv_count == 0 || ohlcv_count > MAX_REPLAY_LOG_OHLCV_COUNT {
            return Err(AppError::new(
                ErrorKind::Validation,
                format!(
                    "replay log --ohlcv-count must be between 1 and {MAX_REPLAY_LOG_OHLCV_COUNT}"
                ),
            )
            .with_details(json!({
                "field": "ohlcv_count",
                "minimum": 1,
                "maximum": MAX_REPLAY_LOG_OHLCV_COUNT,
                "value": ohlcv_count,
                "source_category": REPLAY_STEP_LOG_SOURCE_CATEGORY,
                "requires_desktop": true,
                "non_mutating": false
            })));
        }
        Ok(Self {
            attach_ohlcv_summary,
            ohlcv_count,
        })
    }

    fn attachments_requested(self) -> bool {
        self.attach_ohlcv_summary
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct ReplayLogAttachmentCounters {
    requested: u64,
    ok: u64,
    error: u64,
}

#[derive(Debug, Default, Clone, Copy)]
struct ReplayLogScreenshotCounters {
    requested: u64,
    ok: u64,
    error: u64,
}

#[derive(Debug)]
struct ReplayLogScreenshotPlan {
    paths: Vec<PathBuf>,
}

impl ReplayLogScreenshotPlan {
    fn new(
        steps: u64,
        attach: bool,
        output_dir: Option<PathBuf>,
    ) -> Result<Option<Self>, AppError> {
        match (attach, output_dir) {
            (false, None) => Ok(None),
            (false, Some(_)) => Err(replay_screenshot_validation_error(
                "screenshot_output_dir",
                "replay log --screenshot-output-dir requires --attach-chart-screenshot",
            )),
            (true, None) => Err(replay_screenshot_validation_error(
                "attach_chart_screenshot",
                "replay log --attach-chart-screenshot requires --screenshot-output-dir",
            )),
            (true, Some(output_dir)) => {
                if output_dir.exists() && !output_dir.is_dir() {
                    return Err(replay_screenshot_validation_error(
                        "screenshot_output_dir",
                        "Replay screenshot output path must be a directory",
                    ));
                }
                let paths = (1..=steps)
                    .map(|index| output_dir.join(format!("replay-step-{index:04}.png")))
                    .collect::<Vec<_>>();
                if paths.iter().any(|path| path.exists()) {
                    return Err(replay_screenshot_validation_error(
                        "screenshot_output_dir",
                        "Replay screenshot destination already exists",
                    ));
                }
                fs::create_dir_all(&output_dir).map_err(|_| {
                    replay_screenshot_validation_error(
                        "screenshot_output_dir",
                        "Could not create Replay screenshot output directory",
                    )
                })?;
                Ok(Some(Self { paths }))
            }
        }
    }

    fn path(&self, step_index: u64) -> &Path {
        &self.paths[(step_index - 1) as usize]
    }
}

fn replay_screenshot_validation_error(field: &str, message: &str) -> AppError {
    AppError::new(ErrorKind::Validation, message).with_details(json!({
        "field": field,
        "source_category": REPLAY_STEP_LOG_SOURCE_CATEGORY,
        "requires_desktop": true,
        "non_mutating": false
    }))
}

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

pub async fn run_replay_log_command(
    steps: u64,
    attach_ohlcv_summary: bool,
    ohlcv_count: Option<usize>,
    attach_chart_screenshot: bool,
    screenshot_output_dir: Option<PathBuf>,
    config: &TransportConfig,
) -> Result<(), JsonlRunError> {
    validate_replay_log_steps(steps)?;
    let attachment_controls = ReplayLogAttachmentControls::new(attach_ohlcv_summary, ohlcv_count)?;
    let screenshot_plan =
        ReplayLogScreenshotPlan::new(steps, attach_chart_screenshot, screenshot_output_dir)?;
    let stdout = std::io::stdout();
    let mut stdout = JsonlOutput::new(stdout.lock());
    let started_at = Instant::now();
    let mut runtime = connect_runtime(config).await?;

    let initial_status = ops::replay_status(&mut runtime).await?;
    let readiness = replay_log_readiness(
        steps,
        attachment_controls,
        screenshot_plan.is_some(),
        &initial_status,
    )?;
    let envelope = SuccessEnvelope::new("replay", readiness);
    if emit_jsonl_stdout(&mut stdout, &envelope)? == OutputDisposition::BrokenPipe {
        return Ok(());
    }

    let started_at_replay_date = replay_current_date(&initial_status);
    let mut ended_at_replay_date = started_at_replay_date.clone();
    let mut last_context = initial_status
        .get("replay_context")
        .cloned()
        .unwrap_or(Value::Null);
    let mut step_count = 0_u64;
    let mut failure_count = 0_u64;
    let mut failure_details = None;
    let mut attachment_counters = ReplayLogAttachmentCounters::default();
    let mut screenshot_counters = ReplayLogScreenshotCounters::default();

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
                    let attachment = if attachment_controls.attach_ohlcv_summary {
                        attachment_counters.requested += 1;
                        Some(
                            match replay_log_ohlcv_summary_attachment(
                                &mut runtime,
                                attachment_controls.ohlcv_count,
                            )
                            .await
                            {
                                Ok(attachment) => {
                                    attachment_counters.ok += 1;
                                    attachment
                                }
                                Err(err) => {
                                    attachment_counters.error += 1;
                                    replay_log_ohlcv_summary_attachment_error(&err)?
                                }
                            },
                        )
                    } else {
                        None
                    };
                    let screenshot_attachment = if let Some(plan) = &screenshot_plan {
                        screenshot_counters.requested += 1;
                        Some(
                            match replay_log_chart_screenshot_attachment(
                                &mut runtime,
                                step_index,
                                plan.path(step_index),
                            )
                            .await
                            {
                                Ok(attachment) => {
                                    screenshot_counters.ok += 1;
                                    attachment
                                }
                                Err(error) => {
                                    screenshot_counters.error += 1;
                                    replay_log_chart_screenshot_attachment_error(
                                        step_index, &error,
                                    )?
                                }
                            },
                        )
                    } else {
                        None
                    };
                    let payload =
                        replay_log_step(step_index, step, attachment, screenshot_attachment)?;
                    let envelope = SuccessEnvelope::new("replay", payload);
                    if emit_jsonl_stdout(&mut stdout, &envelope)? == OutputDisposition::BrokenPipe {
                        return Ok(());
                    }
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
        attachment_counters,
        screenshot_counters,
        started_at.elapsed().as_millis() as u64,
        end_reason,
    )?;
    let envelope = SuccessEnvelope::new("replay", summary);
    let _ = emit_jsonl_stdout(&mut stdout, &envelope)?;
    Ok(())
}

fn replay_log_readiness(
    steps: u64,
    attachment_controls: ReplayLogAttachmentControls,
    attach_chart_screenshot: bool,
    status: &Value,
) -> Result<Value, AppError> {
    let mut payload = json!({
        "requested_steps": steps,
        "max_steps": MAX_REPLAY_LOG_STEPS,
        "attachments_requested": attachment_controls.attachments_requested() || attach_chart_screenshot,
        "attach_ohlcv_summary": attachment_controls.attach_ohlcv_summary,
        "attach_chart_screenshot": attach_chart_screenshot,
        "ohlcv_count": attachment_controls.ohlcv_count,
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

fn replay_log_step(
    step_index: u64,
    step: Value,
    ohlcv_attachment: Option<Value>,
    screenshot_attachment: Option<Value>,
) -> Result<Value, AppError> {
    let mut payload = json!({
        "step_index": step_index,
        "operation": step.get("operation").cloned().unwrap_or_else(|| json!("replay_step")),
        "previous_date": step.get("previous_date").cloned().unwrap_or(Value::Null),
        "current_date": step.get("current_date").cloned().unwrap_or(Value::Null),
        "replay_context": step.get("replay_context").cloned().unwrap_or(Value::Null),
        "chart_context": step.get("chart_context").cloned().unwrap_or(Value::Null),
    });
    if let Some(attachment) = ohlcv_attachment {
        payload["attachments"] = json!({
            "ohlcv_summary": attachment
        });
    }
    if let Some(attachment) = screenshot_attachment {
        if payload.get("attachments").is_none() {
            payload["attachments"] = json!({});
        }
        payload["attachments"]["chart_screenshot"] = attachment;
    }
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
    attachment_counters: ReplayLogAttachmentCounters,
    screenshot_counters: ReplayLogScreenshotCounters,
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
        "attachment_requested_count": attachment_counters.requested,
        "attachment_ok_count": attachment_counters.ok,
        "attachment_error_count": attachment_counters.error,
        "screenshot_attachment_requested_count": screenshot_counters.requested,
        "screenshot_attachment_ok_count": screenshot_counters.ok,
        "screenshot_attachment_error_count": screenshot_counters.error,
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

async fn replay_log_ohlcv_summary_attachment(
    runtime: &mut impl tradingview_cdp::RuntimeEvaluator,
    count: usize,
) -> Result<Value, AppError> {
    let summary = ops::ohlcv_summary(runtime, Some(count)).await?;
    replay_log_ohlcv_summary_attachment_ok(summary)
}

async fn replay_log_chart_screenshot_attachment(
    runtime: &mut impl tradingview_cdp::RuntimeEvaluator,
    step_index: u64,
    output_path: &Path,
) -> Result<Value, AppError> {
    let screenshot = ops::screenshot_chart_attachment(runtime, output_path).await?;
    Ok(json!({
        "contract_version": REPLAY_LOG_SCREENSHOT_ATTACHMENT_CONTRACT_VERSION,
        "source": "desktop_screenshot",
        "source_category": "desktop_backed_read",
        "requires_desktop": true,
        "non_mutating": true,
        "writes_file": true,
        "status": "ok",
        "step_index": step_index,
        "output_path": screenshot.get("output_path").cloned().unwrap_or(Value::Null),
        "region": screenshot.get("region").cloned().unwrap_or(Value::Null),
        "capture_mode": screenshot.get("capture_mode").cloned().unwrap_or(Value::Null),
        "size_bytes": screenshot.get("size_bytes").cloned().unwrap_or(Value::Null)
    }))
}

fn replay_log_chart_screenshot_attachment_error(
    step_index: u64,
    error: &AppError,
) -> Result<Value, AppError> {
    let phase = error
        .details
        .as_ref()
        .and_then(|details| details.get("phase"))
        .and_then(Value::as_str)
        .map(|phase| {
            if phase.contains("bounds") {
                "bounds"
            } else if phase.contains("write") {
                "write"
            } else {
                "capture"
            }
        })
        .unwrap_or("capture");
    Ok(json!({
        "contract_version": REPLAY_LOG_SCREENSHOT_ATTACHMENT_CONTRACT_VERSION,
        "source": "desktop_screenshot",
        "source_category": "desktop_backed_read",
        "requires_desktop": true,
        "non_mutating": true,
        "writes_file": false,
        "status": "error",
        "step_index": step_index,
        "failure_details": {
            "kind": error.kind,
            "message": "Replay chart screenshot attachment failed",
            "phase": phase
        }
    }))
}

fn replay_log_ohlcv_summary_attachment_ok(summary: Value) -> Result<Value, AppError> {
    let mut payload = json!({
        "contract_version": REPLAY_LOG_OHLCV_ATTACHMENT_CONTRACT_VERSION,
        "source": REPLAY_LOG_OHLCV_ATTACHMENT_SOURCE,
        "source_category": "desktop_backed_read",
        "requires_desktop": true,
        "non_mutating": true,
        "status": "ok",
        "ohlcv_summary": summary
    });
    let Some(object) = payload.as_object_mut() else {
        return Err(AppError::new(
            ErrorKind::Internal,
            "Replay log OHLCV attachment payload was not an object",
        ));
    };
    sanitize_value_object(object);
    Ok(payload)
}

fn replay_log_ohlcv_summary_attachment_error(error: &AppError) -> Result<Value, AppError> {
    let mut payload = json!({
        "contract_version": REPLAY_LOG_OHLCV_ATTACHMENT_CONTRACT_VERSION,
        "source": REPLAY_LOG_OHLCV_ATTACHMENT_SOURCE,
        "source_category": "desktop_backed_read",
        "requires_desktop": true,
        "non_mutating": true,
        "status": "error",
        "failure_details": replay_failure_details(error)
    });
    let Some(object) = payload.as_object_mut() else {
        return Err(AppError::new(
            ErrorKind::Internal,
            "Replay log OHLCV attachment error payload was not an object",
        ));
    };
    sanitize_value_object(object);
    Ok(payload)
}

fn replay_failure_details(error: &AppError) -> Value {
    let mut payload = json!({
        "kind": error.kind,
        "message": error.message,
        "details": error.details,
    });
    sanitize_value(&mut payload);
    payload
}

fn sanitize_value(value: &mut Value) {
    match value {
        Value::Object(object) => sanitize_value_object(object),
        Value::Array(items) => {
            for item in items {
                sanitize_value(item);
            }
        }
        _ => {}
    }
}

fn sanitize_value_object(object: &mut serde_json::Map<String, Value>) {
    for key in [
        "raw",
        "raw_dom",
        "raw_payload",
        "raw_response",
        "target_id",
        "session_id",
        "cookie",
        "authorization",
        "credential",
        "credentials",
        "account_local_metadata",
        "local_path",
        "absolute_path",
    ] {
        object.remove(key);
    }
    for value in object.values_mut() {
        sanitize_value(value);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tempfile::tempdir;

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
        let controls = ReplayLogAttachmentControls::new(false, None).unwrap();
        let readiness = replay_log_readiness(3, controls, false, &started_status()).unwrap();
        assert_eq!(readiness["contract_version"], "replay_step_log.v1");
        assert_eq!(readiness["_replay"], "log");
        assert_eq!(readiness["_event"], "readiness");
        assert_eq!(readiness["requested_steps"], 3);
        assert_eq!(readiness["max_steps"], 100);
        assert_eq!(readiness["attachments_requested"], false);
        assert_eq!(readiness["attach_ohlcv_summary"], false);
        assert_eq!(readiness["ohlcv_count"], 100);
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
            None,
            None,
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
            ReplayLogAttachmentCounters::default(),
            ReplayLogScreenshotCounters::default(),
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
        assert_eq!(summary["attachment_requested_count"], 0);
        assert_eq!(summary["attachment_ok_count"], 0);
        assert_eq!(summary["attachment_error_count"], 0);
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
            ReplayLogAttachmentCounters::default(),
            ReplayLogScreenshotCounters::default(),
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
    fn attachment_controls_validate_ohlcv_count_usage() {
        let missing_flag = ReplayLogAttachmentControls::new(false, Some(100)).unwrap_err();
        assert_eq!(missing_flag.kind, ErrorKind::Validation);
        let missing_flag_details = missing_flag.details.unwrap();
        assert_eq!(
            missing_flag_details["requires"],
            json!("attach_ohlcv_summary")
        );

        for count in [0, 501] {
            let err = ReplayLogAttachmentControls::new(true, Some(count)).unwrap_err();
            assert_eq!(err.kind, ErrorKind::Validation);
            let details = err.details.unwrap();
            assert_eq!(details["field"], "ohlcv_count");
            assert_eq!(details["maximum"], 500);
        }
    }

    #[test]
    fn readiness_reports_attachment_controls() {
        let controls = ReplayLogAttachmentControls::new(true, Some(42)).unwrap();
        let readiness = replay_log_readiness(3, controls, false, &started_status()).unwrap();
        assert_eq!(readiness["attachments_requested"], true);
        assert_eq!(readiness["attach_ohlcv_summary"], true);
        assert_eq!(readiness["ohlcv_count"], 42);
    }

    #[test]
    fn screenshot_plan_requires_both_options_and_refuses_existing_files() {
        let missing_dir = ReplayLogScreenshotPlan::new(1, true, None).unwrap_err();
        assert_eq!(missing_dir.kind, ErrorKind::Validation);

        let dir_without_flag = tempdir().unwrap();
        let missing_flag =
            ReplayLogScreenshotPlan::new(1, false, Some(dir_without_flag.path().to_path_buf()))
                .unwrap_err();
        assert_eq!(missing_flag.kind, ErrorKind::Validation);

        let output = tempdir().unwrap();
        fs::write(output.path().join("replay-step-0001.png"), b"existing").unwrap();
        let existing =
            ReplayLogScreenshotPlan::new(2, true, Some(output.path().to_path_buf())).unwrap_err();
        assert_eq!(existing.kind, ErrorKind::Validation);
        assert_eq!(
            fs::read(output.path().join("replay-step-0001.png")).unwrap(),
            b"existing"
        );
    }

    #[test]
    fn screenshot_plan_uses_deterministic_widening_names() {
        let root = tempdir().unwrap();
        let output = root.path().join("screenshots");
        let plan = ReplayLogScreenshotPlan::new(10_000, true, Some(output))
            .unwrap()
            .unwrap();
        assert_eq!(plan.path(1).file_name().unwrap(), "replay-step-0001.png");
        assert_eq!(
            plan.path(10_000).file_name().unwrap(),
            "replay-step-10000.png"
        );
    }

    #[test]
    fn step_event_can_include_ohlcv_summary_attachment() {
        let attachment = replay_log_ohlcv_summary_attachment_ok(json!({
            "symbol": "NASDAQ:AAPL",
            "bar_count": 42,
            "source_category": "desktop_backed_read"
        }))
        .unwrap();
        let step = replay_log_step(
            1,
            json!({
                "operation": "replay_step",
                "previous_date": 1775001600000_i64,
                "current_date": 1775088000000_i64,
                "replay_context": {
                    "is_replay_started": true,
                    "current_date": 1775088000000_i64
                }
            }),
            Some(attachment),
            None,
        )
        .unwrap();

        let ohlcv = &step["attachments"]["ohlcv_summary"];
        assert_eq!(
            ohlcv["contract_version"],
            "replay_log_ohlcv_summary_attachment.v1"
        );
        assert_eq!(ohlcv["source"], "selected_chart_cdp");
        assert_eq!(ohlcv["source_category"], "desktop_backed_read");
        assert_eq!(ohlcv["requires_desktop"], true);
        assert_eq!(ohlcv["non_mutating"], true);
        assert_eq!(ohlcv["status"], "ok");
        assert_eq!(ohlcv["ohlcv_summary"]["bar_count"], 42);
    }

    #[test]
    fn attachment_failure_details_are_sanitized() {
        let error = AppError::new(ErrorKind::InternalApiUnavailable, "OHLCV unavailable")
            .with_details(json!({
                "phase": "ohlcv_bars_read",
                "raw_payload": {"secret": true},
                "target_id": "target-123",
                "local_path": "/tmp/private",
                "public": "kept"
            }));
        let attachment = replay_log_ohlcv_summary_attachment_error(&error).unwrap();
        assert_eq!(attachment["status"], "error");
        assert_eq!(attachment["failure_details"]["details"]["public"], "kept");
        assert!(
            attachment["failure_details"]["details"]
                .get("raw_payload")
                .is_none()
        );
        assert!(
            attachment["failure_details"]["details"]
                .get("target_id")
                .is_none()
        );
        assert!(
            attachment["failure_details"]["details"]
                .get("local_path")
                .is_none()
        );
    }

    #[test]
    fn screenshot_error_is_fixed_and_public_safe() {
        let error = AppError::new(ErrorKind::InternalApiUnavailable, "private exception")
            .with_details(json!({
                "phase": "chart_bounds_invalid",
                "target_id": "private-target",
                "raw_payload": "private-source"
            }));
        let attachment = replay_log_chart_screenshot_attachment_error(2, &error).unwrap();
        assert_eq!(attachment["status"], "error");
        assert_eq!(attachment["step_index"], 2);
        assert_eq!(attachment["failure_details"]["phase"], "bounds");
        assert_eq!(
            attachment["failure_details"]["message"],
            "Replay chart screenshot attachment failed"
        );
        let encoded = serde_json::to_string(&attachment).unwrap();
        assert!(!encoded.contains("private"));
        assert!(!encoded.contains("target_id"));
        assert!(!encoded.contains("raw_payload"));
    }

    #[test]
    fn step_event_composes_ohlcv_and_screenshot_attachments() {
        let step = replay_log_step(
            1,
            json!({"operation": "replay_step"}),
            Some(json!({"status": "ok", "kind": "ohlcv"})),
            Some(json!({"status": "ok", "kind": "screenshot"})),
        )
        .unwrap();
        assert_eq!(step["attachments"]["ohlcv_summary"]["kind"], "ohlcv");
        assert_eq!(
            step["attachments"]["chart_screenshot"]["kind"],
            "screenshot"
        );
    }

    #[test]
    fn summary_reports_attachment_counters() {
        let summary = replay_log_summary(
            3,
            3,
            0,
            json!(1775001600000_i64),
            json!(1775260800000_i64),
            json!({"is_replay_started": true}),
            None,
            ReplayLogAttachmentCounters {
                requested: 3,
                ok: 2,
                error: 1,
            },
            ReplayLogScreenshotCounters::default(),
            1250,
            ReplayLogEndReason::StepLimitReached,
        )
        .unwrap();

        assert_eq!(summary["attachment_requested_count"], 3);
        assert_eq!(summary["attachment_ok_count"], 2);
        assert_eq!(summary["attachment_error_count"], 1);
        assert_eq!(summary["screenshot_attachment_requested_count"], 0);
        assert_eq!(summary["end_reason"], "step_limit_reached");
    }

    #[test]
    fn summary_reports_screenshot_attachment_counters() {
        let summary = replay_log_summary(
            3,
            3,
            0,
            Value::Null,
            Value::Null,
            json!({"is_replay_started": true}),
            None,
            ReplayLogAttachmentCounters::default(),
            ReplayLogScreenshotCounters {
                requested: 3,
                ok: 2,
                error: 1,
            },
            50,
            ReplayLogEndReason::StepLimitReached,
        )
        .unwrap();
        assert_eq!(summary["screenshot_attachment_requested_count"], 3);
        assert_eq!(summary["screenshot_attachment_ok_count"], 2);
        assert_eq!(summary["screenshot_attachment_error_count"], 1);
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
