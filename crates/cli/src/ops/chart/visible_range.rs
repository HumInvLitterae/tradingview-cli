use std::time::Duration;

use serde_json::{Value, json};
use tokio::time::{Instant, sleep, timeout_at};

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};
use tradingview_model::visible_range::{
    LoadedBarPoint, LoadedRangeObservation, PagingDecision, PagingStopReason, ProgressState,
    VisibleRangeBounds, VisibleRangeValidationFailure, coverage_status, initial_paging_decision,
    progress_paging_decision, validate_visible_range_bounds, viewport_decision,
};

use super::super::common::CHART_API;

const HISTORY_REQUEST_SIZE: usize = 1000;
const HISTORY_REQUEST_LIMIT: usize = 25;
const HISTORY_DEADLINE: Duration = Duration::from_secs(30);
const HISTORY_PROGRESS_WINDOW: Duration = Duration::from_secs(3);
const HISTORY_OBSERVATION_INTERVAL: Duration = Duration::from_millis(200);

#[derive(Debug, Clone, Copy)]
struct PagingControls {
    request_limit: usize,
    deadline: Duration,
    progress_window: Duration,
    observation_interval: Duration,
}

impl Default for PagingControls {
    fn default() -> Self {
        Self {
            request_limit: HISTORY_REQUEST_LIMIT,
            deadline: HISTORY_DEADLINE,
            progress_window: HISTORY_PROGRESS_WINDOW,
            observation_interval: HISTORY_OBSERVATION_INTERVAL,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct HistoryEvidence {
    earliest_before: Option<f64>,
    earliest_after: Option<f64>,
}

pub async fn visible_range(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var chart = {CHART_API};
                    return {{
                        operation: "visible_range",
                        source: "chart_api",
                        source_category: "desktop_backed_read",
                        requires_desktop: true,
                        non_mutating: true,
                        visible_range: chart.getVisibleRange(),
                        bars_range: chart.getVisibleBarsRange()
                    }};
                }})()
                "#
            ),
            false,
        )
        .await
}

pub async fn set_visible_range(
    runtime: &mut impl RuntimeEvaluator,
    from: f64,
    to: f64,
) -> Result<Value, AppError> {
    set_visible_range_with_controls(runtime, from, to, PagingControls::default()).await
}

async fn set_visible_range_with_controls(
    runtime: &mut impl RuntimeEvaluator,
    from: f64,
    to: f64,
    controls: PagingControls,
) -> Result<Value, AppError> {
    let bounds = validate_visible_range_request(from, to)?;
    let started = Instant::now();
    let deadline = started + controls.deadline;
    let mut request_count = 0usize;

    let initial = inspect_history_before(
        runtime,
        deadline,
        bounds,
        "inspect_initial",
        request_count,
        HistoryEvidence::default(),
    )
    .await?
    .ok_or_else(|| {
        history_timeout_error(
            bounds,
            "inspect_initial",
            request_count,
            HistoryEvidence::default(),
        )
    })?;
    let earliest_before = initial.observation.earliest;
    let mut evidence = HistoryEvidence {
        earliest_before: Some(earliest_before),
        earliest_after: Some(earliest_before),
    };
    let mut current = initial;
    let mut stop_reason = match initial_paging_decision(
        bounds,
        current.observation_for(bounds, "inspect_initial", request_count, evidence)?,
        Instant::now() >= deadline,
    ) {
        PagingDecision::Stop(reason) => Some(reason),
        PagingDecision::RequestMore => None,
    };

    while stop_reason.is_none() {
        if Instant::now() >= deadline {
            stop_reason = Some(PagingStopReason::DeadlineElapsed);
            break;
        }
        current.require_request_api(bounds, request_count, evidence)?;
        let previous_earliest = current.observation.earliest;
        request_count += 1;
        if request_history_before(runtime, deadline, bounds, request_count, evidence)
            .await?
            .is_none()
        {
            stop_reason = Some(PagingStopReason::DeadlineElapsed);
            break;
        }

        let progress_deadline = Instant::now() + controls.progress_window;
        let observation_deadline = deadline.min(progress_deadline);
        loop {
            let now = Instant::now();
            if now >= deadline {
                stop_reason = Some(PagingStopReason::DeadlineElapsed);
                break;
            }
            if now >= progress_deadline {
                stop_reason = Some(PagingStopReason::NoProgress);
                break;
            }
            sleep(
                controls
                    .observation_interval
                    .min(observation_deadline.saturating_duration_since(now)),
            )
            .await;
            if Instant::now() >= observation_deadline {
                stop_reason = Some(if deadline <= progress_deadline {
                    PagingStopReason::DeadlineElapsed
                } else {
                    PagingStopReason::NoProgress
                });
                break;
            }

            let Some(observed) = inspect_history_before(
                runtime,
                observation_deadline,
                bounds,
                "observe_progress",
                request_count,
                evidence,
            )
            .await?
            else {
                stop_reason = Some(if deadline <= progress_deadline {
                    PagingStopReason::DeadlineElapsed
                } else {
                    PagingStopReason::NoProgress
                });
                break;
            };
            if Instant::now() >= observation_deadline {
                stop_reason = Some(if deadline <= progress_deadline {
                    PagingStopReason::DeadlineElapsed
                } else {
                    PagingStopReason::NoProgress
                });
                break;
            }
            evidence.earliest_after = Some(observed.observation.earliest);
            let observed_range =
                observed.observation_for(bounds, "observe_progress", request_count, evidence)?;
            let made_progress = observed_range.earliest < previous_earliest;
            match progress_paging_decision(
                bounds,
                ProgressState {
                    previous_earliest,
                    current: observed_range,
                    request_count,
                    request_limit: controls.request_limit,
                    deadline_elapsed: Instant::now() >= deadline,
                    progress_window_elapsed: Instant::now() >= progress_deadline,
                },
            ) {
                PagingDecision::Stop(reason) => {
                    current = observed;
                    stop_reason = Some(reason);
                    break;
                }
                PagingDecision::RequestMore if made_progress => {
                    current = observed;
                    break;
                }
                PagingDecision::RequestMore => {
                    current = observed;
                }
            }
        }
    }

    let stop_reason = stop_reason.expect("paging loop must choose a terminal reason");
    let elapsed_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
    let paging_observation = current.observation;
    let full = inspect_loaded_bars(runtime, bounds, request_count, evidence).await?;
    let viewport = viewport_decision(bounds, full.observation, &full.bars);
    let actual =
        if let (Some(from_index), Some(to_index)) = (viewport.from_index, viewport.to_index) {
            apply_visible_indices(
                runtime,
                from_index,
                to_index,
                bounds,
                request_count,
                evidence,
            )
            .await?
        } else {
            full.visible_range.clone()
        };

    Ok(json!({
        "operation": "visible_range",
        "source": "chart_api",
        "source_category": "desktop_backed_operation",
        "requires_desktop": true,
        "non_mutating": false,
        "requested": { "from": bounds.from, "to": bounds.to },
        "actual": actual,
        "viewport_application": {
            "status": viewport.status.as_str(),
            "applied": viewport.applied(),
            "clamped": viewport.clamped(),
            "matching_bar_count": viewport.matching_bar_count,
            "applied_range": viewport.applied_range.map(|range| json!({
                "from": range.from,
                "to": range.to,
            })),
        },
        "history_paging": {
            "request_size": HISTORY_REQUEST_SIZE,
            "request_limit": controls.request_limit,
            "request_count": request_count,
            "deadline_ms": controls.deadline.as_millis() as u64,
            "elapsed_ms": elapsed_ms,
            "earliest_loaded_before": earliest_before,
            "earliest_loaded_after": paging_observation.earliest,
            "latest_loaded_after": paging_observation.latest,
            "coverage_status": coverage_status(bounds, paging_observation).as_str(),
            "stop_reason": stop_reason.as_str(),
            "history_exhausted": stop_reason == PagingStopReason::HistoryExhausted,
            "limit_reached": stop_reason.limit_reached(),
            "timed_out": stop_reason.timed_out(),
        }
    }))
}

pub fn validate_visible_range_request(from: f64, to: f64) -> Result<VisibleRangeBounds, AppError> {
    validate_visible_range_bounds(from, to).map_err(range_validation_error)
}

fn range_validation_error(failure: VisibleRangeValidationFailure) -> AppError {
    match failure {
        VisibleRangeValidationFailure::NonFinite { field } => AppError::new(
            ErrorKind::Validation,
            format!("{field} must be a finite number"),
        ),
        VisibleRangeValidationFailure::InvalidOrder { from, to } => AppError::new(
            ErrorKind::Validation,
            "range requires --from to be less than --to",
        )
        .with_details(json!({ "from": from, "to": to })),
    }
}

#[derive(Debug, Clone)]
struct HistoryState {
    observation: LoadedRangeObservation,
    more_available_observed: Option<bool>,
    request_method_available: bool,
    availability_method_available: bool,
}

impl HistoryState {
    fn observation_for(
        &self,
        bounds: VisibleRangeBounds,
        phase: &'static str,
        request_count: usize,
        evidence: HistoryEvidence,
    ) -> Result<LoadedRangeObservation, AppError> {
        if self.observation.earliest > bounds.from {
            if !self.availability_method_available || self.more_available_observed.is_none() {
                return Err(history_api_error(
                    bounds,
                    phase,
                    request_count,
                    evidence,
                    "Selected chart history availability is not readable",
                ));
            }
            return Ok(LoadedRangeObservation {
                more_available: self.more_available_observed.unwrap_or(false),
                ..self.observation
            });
        }
        Ok(self.observation)
    }

    fn require_request_api(
        &self,
        bounds: VisibleRangeBounds,
        request_count: usize,
        evidence: HistoryEvidence,
    ) -> Result<(), AppError> {
        if self.request_method_available {
            Ok(())
        } else {
            Err(history_api_error(
                bounds,
                "request_history",
                request_count,
                evidence,
                "Selected chart history request method is unavailable",
            ))
        }
    }
}

#[derive(Debug)]
struct LoadedBarsState {
    observation: LoadedRangeObservation,
    bars: Vec<LoadedBarPoint>,
    visible_range: Value,
}

async fn inspect_history_before(
    runtime: &mut impl RuntimeEvaluator,
    deadline: Instant,
    bounds: VisibleRangeBounds,
    phase: &'static str,
    request_count: usize,
    evidence: HistoryEvidence,
) -> Result<Option<HistoryState>, AppError> {
    let expression = format!(
        r#"
        (function() {{
            try {{
                var chart = {CHART_API};
                var series = chart._chartWidget.model().mainSeries();
                var bars = series && typeof series.bars === "function" ? series.bars() : null;
                if (!bars || typeof bars.firstIndex !== "function" || typeof bars.lastIndex !== "function" || typeof bars.valueAt !== "function") {{
                    return {{ status: "unavailable" }};
                }}
                var firstIndex = bars.firstIndex();
                var lastIndex = bars.lastIndex();
                var first = bars.valueAt(firstIndex);
                var last = bars.valueAt(lastIndex);
                if (!first || !last || !Number.isFinite(Number(first[0])) || !Number.isFinite(Number(last[0]))) {{
                    return {{ status: "unavailable" }};
                }}
                var needsPaging = Number(first[0]) > {requested_from};
                var availabilityMethod = typeof series.requestMoreDataAvailable === "function";
                var more = null;
                if (needsPaging && availabilityMethod) {{
                    try {{ more = series.requestMoreDataAvailable(); }} catch (_) {{
                        return {{ status: "availability_error" }};
                    }}
                }}
                return {{
                    status: "ok",
                    earliest: Number(first[0]),
                    latest: Number(last[0]),
                    more_available: more,
                    request_method_available: typeof series.requestMoreData === "function",
                    availability_method_available: availabilityMethod
                }};
            }} catch (_) {{
                return {{ status: "unavailable" }};
            }}
        }})()
        "#,
        requested_from = bounds.from,
    );
    match timeout_at(deadline, runtime.evaluate(&expression, false)).await {
        Err(_) => Ok(None),
        Ok(Err(error)) => Err(add_history_error_details(
            error,
            bounds,
            phase,
            request_count,
            evidence,
        )),
        Ok(Ok(value)) => parse_history_state(value)
            .map_err(|error| {
                add_history_error_details(error, bounds, phase, request_count, evidence)
            })
            .map(Some),
    }
}

fn parse_history_state(value: Value) -> Result<HistoryState, AppError> {
    let status = value.get("status").and_then(Value::as_str);
    if status == Some("availability_error") {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Selected chart history availability could not be inspected",
        ));
    }
    if status != Some("ok") {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Selected chart loaded history is unavailable",
        ));
    }
    let earliest = value.get("earliest").and_then(Value::as_f64);
    let latest = value.get("latest").and_then(Value::as_f64);
    let (Some(earliest), Some(latest)) = (earliest, latest) else {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Selected chart loaded history timestamps are unavailable",
        ));
    };
    let observation = LoadedRangeObservation {
        earliest,
        latest,
        more_available: value
            .get("more_available")
            .and_then(Value::as_bool)
            .unwrap_or(true),
    };
    if !observation.is_valid() {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Selected chart loaded history range is invalid",
        ));
    }
    Ok(HistoryState {
        observation,
        more_available_observed: value.get("more_available").and_then(Value::as_bool),
        request_method_available: value
            .get("request_method_available")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        availability_method_available: value
            .get("availability_method_available")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

async fn request_history_before(
    runtime: &mut impl RuntimeEvaluator,
    deadline: Instant,
    bounds: VisibleRangeBounds,
    request_count: usize,
    evidence: HistoryEvidence,
) -> Result<Option<()>, AppError> {
    let expression = format!(
        r#"
        (function() {{
            var series = {CHART_API}._chartWidget.model().mainSeries();
            if (!series || typeof series.requestMoreData !== "function") return false;
            series.requestMoreData({HISTORY_REQUEST_SIZE});
            return true;
        }})()
        "#
    );
    match timeout_at(deadline, runtime.evaluate(&expression, false)).await {
        Err(_) => Ok(None),
        Ok(Err(error)) => Err(add_history_error_details(
            error,
            bounds,
            "request_history",
            request_count,
            evidence,
        )),
        Ok(Ok(Value::Bool(true))) => Ok(Some(())),
        Ok(Ok(_)) => Err(history_api_error(
            bounds,
            "request_history",
            request_count,
            evidence,
            "Selected chart history request was not accepted",
        )),
    }
}

async fn inspect_loaded_bars(
    runtime: &mut impl RuntimeEvaluator,
    bounds: VisibleRangeBounds,
    request_count: usize,
    evidence: HistoryEvidence,
) -> Result<LoadedBarsState, AppError> {
    let expression = format!(
        r#"
        (function() {{
            try {{
                var chart = {CHART_API};
                var bars = chart._chartWidget.model().mainSeries().bars();
                var startIndex = bars.firstIndex();
                var endIndex = bars.lastIndex();
                var points = [];
                for (var i = startIndex; i <= endIndex; i++) {{
                    var value = bars.valueAt(i);
                    if (value && Number.isFinite(Number(value[0]))) {{
                        points.push({{ index: i, timestamp: Number(value[0]) }});
                    }}
                }}
                var visible = chart.getVisibleRange();
                return {{ status: "ok", bars: points, visible_range: visible }};
            }} catch (_) {{
                return {{ status: "unavailable" }};
            }}
        }})()
        "#
    );
    let value = runtime
        .evaluate(&expression, false)
        .await
        .map_err(|error| {
            add_history_error_details(error, bounds, "inspect_final", request_count, evidence)
        })?;
    if value.get("status").and_then(Value::as_str) != Some("ok") {
        return Err(history_api_error(
            bounds,
            "inspect_final",
            request_count,
            evidence,
            "Selected chart bars are unavailable for viewport application",
        ));
    }
    let mut bars = Vec::new();
    for point in value
        .get("bars")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let index = point.get("index").and_then(Value::as_i64);
        let timestamp = point.get("timestamp").and_then(Value::as_f64);
        if let (Some(index), Some(timestamp)) = (index, timestamp)
            && timestamp.is_finite()
        {
            bars.push(LoadedBarPoint { index, timestamp });
        }
    }
    let (Some(first), Some(last)) = (bars.first(), bars.last()) else {
        return Err(history_api_error(
            bounds,
            "inspect_final",
            request_count,
            evidence,
            "Selected chart loaded bars are empty",
        ));
    };
    let visible_range = value.get("visible_range").cloned().unwrap_or(Value::Null);
    if visible_range.is_null() {
        return Err(history_api_error(
            bounds,
            "inspect_final",
            request_count,
            HistoryEvidence {
                earliest_after: Some(first.timestamp),
                ..evidence
            },
            "Selected chart visible range is unavailable",
        ));
    }
    Ok(LoadedBarsState {
        observation: LoadedRangeObservation {
            earliest: first.timestamp,
            latest: last.timestamp,
            more_available: true,
        },
        bars,
        visible_range,
    })
}

async fn apply_visible_indices(
    runtime: &mut impl RuntimeEvaluator,
    from_index: i64,
    to_index: i64,
    bounds: VisibleRangeBounds,
    request_count: usize,
    evidence: HistoryEvidence,
) -> Result<Value, AppError> {
    let expression = format!(
        r#"
        (function() {{
            var chart = {CHART_API};
            chart._chartWidget.model().timeScale().zoomToBarsRange({from_index}, {to_index});
            return new Promise(function(resolve) {{
                setTimeout(function() {{
                    try {{ resolve(chart.getVisibleRange()); }}
                    catch (_) {{ resolve(null); }}
                }}, 500);
            }});
        }})()
        "#
    );
    let actual = runtime.evaluate(&expression, true).await.map_err(|error| {
        add_history_error_details(
            error,
            bounds,
            "apply_visible_range",
            request_count,
            evidence,
        )
    })?;
    if actual.is_null() {
        return Err(history_api_error(
            bounds,
            "apply_visible_range",
            request_count,
            evidence,
            "Selected chart visible range could not be read after application",
        ));
    }
    Ok(actual)
}

fn history_timeout_error(
    bounds: VisibleRangeBounds,
    phase: &'static str,
    request_count: usize,
    evidence: HistoryEvidence,
) -> AppError {
    history_error(
        ErrorKind::Timeout,
        "Selected chart history paging timed out before a safe range was observed",
        bounds,
        phase,
        request_count,
        evidence,
    )
}

fn history_api_error(
    bounds: VisibleRangeBounds,
    phase: &'static str,
    request_count: usize,
    evidence: HistoryEvidence,
    message: &'static str,
) -> AppError {
    history_error(
        ErrorKind::InternalApiUnavailable,
        message,
        bounds,
        phase,
        request_count,
        evidence,
    )
}

fn history_error(
    kind: ErrorKind,
    message: &'static str,
    bounds: VisibleRangeBounds,
    phase: &'static str,
    request_count: usize,
    evidence: HistoryEvidence,
) -> AppError {
    AppError::new(kind, message).with_details(json!({
        "operation": "visible_range",
        "source": "chart_api",
        "source_category": "desktop_backed_operation",
        "requires_desktop": true,
        "non_mutating": false,
        "requested": { "from": bounds.from, "to": bounds.to },
        "paging_phase": phase,
        "request_count": request_count,
        "earliest_loaded_before": evidence.earliest_before,
        "earliest_loaded_after": evidence.earliest_after,
        "next_action_hint": "Run `tv state` and `tv range` without bounds to confirm selected-chart readiness, then retry the bounded range operation."
    }))
}

fn add_history_error_details(
    error: AppError,
    bounds: VisibleRangeBounds,
    phase: &'static str,
    request_count: usize,
    evidence: HistoryEvidence,
) -> AppError {
    history_error(
        error.kind,
        "Selected chart history operation failed",
        bounds,
        phase,
        request_count,
        evidence,
    )
}

#[cfg(test)]
mod timing_tests {
    use std::collections::VecDeque;

    use serde_json::json;
    use tokio::time::sleep;

    use tradingview_cdp::{KeyEvent, MouseEvent, ScreenshotClip};

    use super::*;

    enum Evaluation {
        Immediate(Result<Value, ErrorKind>),
        Delayed(Duration, Result<Value, ErrorKind>),
    }

    struct ControlledRuntime {
        evaluations: VecDeque<Evaluation>,
    }

    impl ControlledRuntime {
        fn new(evaluations: impl Into<VecDeque<Evaluation>>) -> Self {
            Self {
                evaluations: evaluations.into(),
            }
        }
    }

    impl RuntimeEvaluator for ControlledRuntime {
        async fn evaluate(
            &mut self,
            _expression: &str,
            _await_promise: bool,
        ) -> Result<Value, AppError> {
            let evaluation = self.evaluations.pop_front().ok_or_else(|| {
                AppError::new(ErrorKind::Internal, "missing controlled evaluation")
            })?;
            let result = match evaluation {
                Evaluation::Immediate(result) => result,
                Evaluation::Delayed(delay, result) => {
                    sleep(delay).await;
                    result
                }
            };
            result.map_err(|kind| AppError::new(kind, "controlled evaluation failure"))
        }

        async fn capture_screenshot(&mut self) -> Result<Vec<u8>, AppError> {
            Err(unsupported_test_operation())
        }

        async fn capture_screenshot_clip(
            &mut self,
            _clip: ScreenshotClip,
        ) -> Result<Vec<u8>, AppError> {
            Err(unsupported_test_operation())
        }

        async fn insert_text(&mut self, _text: &str) -> Result<(), AppError> {
            Err(unsupported_test_operation())
        }

        async fn dispatch_key_event(&mut self, _event: KeyEvent) -> Result<(), AppError> {
            Err(unsupported_test_operation())
        }

        async fn dispatch_mouse_event(&mut self, _event: MouseEvent) -> Result<(), AppError> {
            Err(unsupported_test_operation())
        }
    }

    fn unsupported_test_operation() -> AppError {
        AppError::new(
            ErrorKind::Internal,
            "unsupported controlled runtime operation",
        )
    }

    fn controls(
        deadline: Duration,
        progress_window: Duration,
        request_limit: usize,
    ) -> PagingControls {
        PagingControls {
            request_limit,
            deadline,
            progress_window,
            observation_interval: Duration::from_millis(100),
        }
    }

    fn history(earliest: f64, more_available: bool) -> Value {
        json!({
            "status": "ok",
            "earliest": earliest,
            "latest": 500.0,
            "more_available": more_available,
            "request_method_available": true,
            "availability_method_available": true
        })
    }

    fn loaded_bars() -> Value {
        json!({
            "status": "ok",
            "visible_range": {"from": 300.0, "to": 500.0},
            "bars": [
                {"index": 1, "timestamp": 100.0},
                {"index": 2, "timestamp": 300.0},
                {"index": 3, "timestamp": 500.0}
            ]
        })
    }

    #[tokio::test(start_paused = true)]
    async fn progress_observation_is_bounded_by_progress_window() {
        let mut runtime = ControlledRuntime::new([
            Evaluation::Immediate(Ok(history(300.0, true))),
            Evaluation::Immediate(Ok(Value::Bool(true))),
            Evaluation::Delayed(Duration::from_secs(4), Ok(history(100.0, true))),
            Evaluation::Immediate(Ok(loaded_bars())),
            Evaluation::Immediate(Ok(json!({"from": 100.0, "to": 500.0}))),
        ]);

        let result = set_visible_range_with_controls(
            &mut runtime,
            100.0,
            500.0,
            controls(Duration::from_secs(30), Duration::from_secs(3), 25),
        )
        .await
        .unwrap();

        assert_eq!(result["history_paging"]["stop_reason"], "no_progress");
        assert_eq!(result["history_paging"]["earliest_loaded_after"], 300.0);
        assert_eq!(result["history_paging"]["elapsed_ms"], 3000);
    }

    #[tokio::test(start_paused = true)]
    async fn absolute_deadline_wins_when_it_precedes_progress_window() {
        let mut runtime = ControlledRuntime::new([
            Evaluation::Immediate(Ok(history(300.0, true))),
            Evaluation::Immediate(Ok(Value::Bool(true))),
            Evaluation::Delayed(Duration::from_secs(4), Ok(history(100.0, true))),
            Evaluation::Immediate(Ok(loaded_bars())),
            Evaluation::Immediate(Ok(json!({"from": 100.0, "to": 500.0}))),
        ]);

        let result = set_visible_range_with_controls(
            &mut runtime,
            100.0,
            500.0,
            controls(Duration::from_secs(2), Duration::from_secs(3), 25),
        )
        .await
        .unwrap();

        assert_eq!(result["history_paging"]["stop_reason"], "deadline_elapsed");
        assert_eq!(result["history_paging"]["elapsed_ms"], 2000);
    }

    #[tokio::test(start_paused = true)]
    async fn coverage_wins_on_the_request_limit_collision() {
        let mut runtime = ControlledRuntime::new([
            Evaluation::Immediate(Ok(history(300.0, true))),
            Evaluation::Immediate(Ok(Value::Bool(true))),
            Evaluation::Immediate(Ok(history(100.0, true))),
            Evaluation::Immediate(Ok(loaded_bars())),
            Evaluation::Immediate(Ok(json!({"from": 100.0, "to": 500.0}))),
        ]);

        let result = set_visible_range_with_controls(
            &mut runtime,
            100.0,
            500.0,
            controls(Duration::from_secs(30), Duration::from_secs(3), 1),
        )
        .await
        .unwrap();

        assert_eq!(result["history_paging"]["stop_reason"], "coverage_reached");
        assert_eq!(result["history_paging"]["request_count"], 1);
    }

    #[tokio::test(start_paused = true)]
    async fn request_limit_stops_after_progress_without_coverage() {
        let mut runtime = ControlledRuntime::new([
            Evaluation::Immediate(Ok(history(300.0, true))),
            Evaluation::Immediate(Ok(Value::Bool(true))),
            Evaluation::Immediate(Ok(history(200.0, true))),
            Evaluation::Immediate(Ok(loaded_bars())),
            Evaluation::Immediate(Ok(json!({"from": 100.0, "to": 500.0}))),
        ]);

        let result = set_visible_range_with_controls(
            &mut runtime,
            100.0,
            500.0,
            controls(Duration::from_secs(30), Duration::from_secs(3), 1),
        )
        .await
        .unwrap();

        assert_eq!(
            result["history_paging"]["stop_reason"],
            "request_limit_reached"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn exhausted_history_stops_without_requesting_more() {
        let mut runtime = ControlledRuntime::new([
            Evaluation::Immediate(Ok(history(300.0, false))),
            Evaluation::Immediate(Ok(loaded_bars())),
            Evaluation::Immediate(Ok(json!({"from": 100.0, "to": 500.0}))),
        ]);

        let result = set_visible_range_with_controls(
            &mut runtime,
            100.0,
            500.0,
            controls(Duration::from_secs(30), Duration::from_secs(3), 25),
        )
        .await
        .unwrap();

        assert_eq!(result["history_paging"]["stop_reason"], "history_exhausted");
        assert_eq!(result["history_paging"]["request_count"], 0);
    }

    #[tokio::test(start_paused = true)]
    async fn request_guard_timeout_is_a_mid_loop_deadline_stop() {
        let mut runtime = ControlledRuntime::new([
            Evaluation::Immediate(Ok(history(300.0, true))),
            Evaluation::Delayed(Duration::from_secs(2), Ok(Value::Bool(true))),
            Evaluation::Immediate(Ok(loaded_bars())),
            Evaluation::Immediate(Ok(json!({"from": 100.0, "to": 500.0}))),
        ]);

        let result = set_visible_range_with_controls(
            &mut runtime,
            100.0,
            500.0,
            controls(Duration::from_secs(1), Duration::from_secs(3), 25),
        )
        .await
        .unwrap();

        assert_eq!(result["history_paging"]["stop_reason"], "deadline_elapsed");
        assert_eq!(result["history_paging"]["request_count"], 1);
        assert_eq!(result["history_paging"]["elapsed_ms"], 1000);
    }

    #[tokio::test(start_paused = true)]
    async fn initial_deadline_is_a_zero_request_timeout() {
        let mut runtime = ControlledRuntime::new([Evaluation::Delayed(
            Duration::from_secs(2),
            Ok(history(300.0, true)),
        )]);

        let error = set_visible_range_with_controls(
            &mut runtime,
            100.0,
            500.0,
            controls(Duration::from_secs(1), Duration::from_secs(3), 25),
        )
        .await
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Timeout);
        assert_eq!(error.details.as_ref().unwrap()["request_count"], 0);
    }

    #[tokio::test(start_paused = true)]
    async fn runtime_failure_preserves_safe_earliest_evidence() {
        let mut runtime = ControlledRuntime::new([
            Evaluation::Immediate(Ok(history(300.0, true))),
            Evaluation::Immediate(Ok(Value::Bool(true))),
            Evaluation::Immediate(Err(ErrorKind::Connection)),
        ]);

        let error = set_visible_range_with_controls(
            &mut runtime,
            100.0,
            500.0,
            controls(Duration::from_secs(30), Duration::from_secs(3), 25),
        )
        .await
        .unwrap_err();

        let details = error.details.as_ref().unwrap();
        assert_eq!(error.kind, ErrorKind::Connection);
        assert_eq!(details["paging_phase"], "observe_progress");
        assert_eq!(details["earliest_loaded_before"], 300.0);
        assert_eq!(details["earliest_loaded_after"], 300.0);
    }

    #[tokio::test(start_paused = true)]
    async fn request_failure_preserves_safe_earliest_evidence() {
        let mut runtime = ControlledRuntime::new([
            Evaluation::Immediate(Ok(history(300.0, true))),
            Evaluation::Immediate(Err(ErrorKind::Connection)),
        ]);

        let error = set_visible_range_with_controls(
            &mut runtime,
            100.0,
            500.0,
            controls(Duration::from_secs(30), Duration::from_secs(3), 25),
        )
        .await
        .unwrap_err();

        let details = error.details.as_ref().unwrap();
        assert_eq!(error.kind, ErrorKind::Connection);
        assert_eq!(details["paging_phase"], "request_history");
        assert_eq!(details["earliest_loaded_before"], 300.0);
        assert_eq!(details["earliest_loaded_after"], 300.0);
    }

    #[tokio::test(start_paused = true)]
    async fn missing_request_api_preserves_safe_earliest_evidence() {
        let mut unavailable = history(300.0, true);
        unavailable["request_method_available"] = Value::Bool(false);
        let mut runtime = ControlledRuntime::new([Evaluation::Immediate(Ok(unavailable))]);

        let error = set_visible_range_with_controls(
            &mut runtime,
            100.0,
            500.0,
            controls(Duration::from_secs(30), Duration::from_secs(3), 25),
        )
        .await
        .unwrap_err();

        let details = error.details.as_ref().unwrap();
        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(details["earliest_loaded_before"], 300.0);
        assert_eq!(details["earliest_loaded_after"], 300.0);
    }
}
