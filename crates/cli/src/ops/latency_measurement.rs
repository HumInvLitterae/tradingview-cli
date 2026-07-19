use std::{
    collections::BTreeMap,
    future::Future,
    pin::Pin,
    time::{Duration, Instant},
};

use serde::Serialize;
use serde_json::Value;
use tokio::time::{Instant as TokioInstant, timeout_at};
use tradingview_cdp::{KeyEvent, MouseEvent, RuntimeEvaluator, ScreenshotClip, TransportConfig};
use tradingview_core::{AppError, ErrorKind};

use crate::app::connect_runtime;

use super::{ohlcv_summary, study_values};

const LIVE_GATE: &str = "TV_LIVE_CHART_READ_LATENCY_ATTRIBUTION";
const LIVE_TARGET: &str = "TV_LIVE_CHART_READ_LATENCY_ATTRIBUTION_TARGET_ID";
const TRIALS_PER_COHORT: usize = 20;
const TRIAL_TIMEOUT: Duration = Duration::from_secs(12);
const COHORT_TIMEOUT: Duration = Duration::from_secs(300);
const RUN_TIMEOUT: Duration = Duration::from_secs(600);

type OperationFuture<'a> = Pin<Box<dyn Future<Output = Result<Value, AppError>> + Send + 'a>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Operation {
    OhlcvSummary,
    StudyValues,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DirectTimings {
    connect_ms: u64,
    evaluate_ms: u64,
    operation_ms: u64,
    payload_serialize_ms: u64,
    trial_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TrialTimings {
    direct: DirectTimings,
    normalization_residual_ms: u64,
    unattributed_residual_ms: u64,
}

impl TrialTimings {
    fn new(direct: DirectTimings) -> Self {
        let normalization_residual_ms = direct.operation_ms.saturating_sub(direct.evaluate_ms);
        let accounted_ms = direct
            .connect_ms
            .saturating_add(direct.operation_ms)
            .saturating_add(direct.payload_serialize_ms);
        let unattributed_residual_ms = direct.trial_ms.saturating_sub(accounted_ms);
        Self {
            direct,
            normalization_residual_ms,
            unattributed_residual_ms,
        }
    }
}

struct TimedRuntime<R> {
    inner: R,
    evaluate_samples_ms: Vec<u64>,
}

impl<R> TimedRuntime<R> {
    fn new(inner: R) -> Self {
        Self {
            inner,
            evaluate_samples_ms: Vec::new(),
        }
    }

    fn one_evaluate_ms(&self) -> Result<u64, AppError> {
        match self.evaluate_samples_ms.as_slice() {
            [sample] => Ok(*sample),
            samples => Err(AppError::new(
                ErrorKind::Internal,
                format!(
                    "latency attribution trial expected one evaluation but observed {}",
                    samples.len()
                ),
            )),
        }
    }
}

impl<R: RuntimeEvaluator + Send> RuntimeEvaluator for TimedRuntime<R> {
    async fn evaluate(&mut self, expression: &str, await_promise: bool) -> Result<Value, AppError> {
        let started = Instant::now();
        let result = self.inner.evaluate(expression, await_promise).await;
        self.evaluate_samples_ms.push(elapsed_ms(started));
        result
    }

    async fn capture_screenshot(&mut self) -> Result<Vec<u8>, AppError> {
        self.inner.capture_screenshot().await
    }

    async fn capture_screenshot_clip(&mut self, clip: ScreenshotClip) -> Result<Vec<u8>, AppError> {
        self.inner.capture_screenshot_clip(clip).await
    }

    async fn insert_text(&mut self, text: &str) -> Result<(), AppError> {
        self.inner.insert_text(text).await
    }

    async fn dispatch_key_event(&mut self, event: KeyEvent) -> Result<(), AppError> {
        self.inner.dispatch_key_event(event).await
    }

    async fn dispatch_mouse_event(&mut self, event: MouseEvent) -> Result<(), AppError> {
        self.inner.dispatch_mouse_event(event).await
    }
}

async fn run_trial<R, Connect, ConnectFuture, Operate, SerializePayload>(
    connect: Connect,
    operate: Operate,
    serialize_payload: SerializePayload,
) -> Result<TrialTimings, AppError>
where
    R: RuntimeEvaluator + Send,
    Connect: FnOnce() -> ConnectFuture,
    ConnectFuture: Future<Output = Result<R, AppError>>,
    Operate: for<'a> FnOnce(&'a mut TimedRuntime<R>) -> OperationFuture<'a>,
    SerializePayload: FnOnce(&Value) -> Result<Vec<u8>, AppError>,
{
    let trial_started = Instant::now();

    let connect_started = Instant::now();
    let runtime = connect().await?;
    let connect_ms = elapsed_ms(connect_started);
    let mut runtime = TimedRuntime::new(runtime);

    let operation_started = Instant::now();
    let value = operate(&mut runtime).await?;
    let operation_ms = elapsed_ms(operation_started);
    let evaluate_ms = runtime.one_evaluate_ms()?;

    let serialization_started = Instant::now();
    let _payload = serialize_payload(&value)?;
    let payload_serialize_ms = elapsed_ms(serialization_started);

    Ok(TrialTimings::new(DirectTimings {
        connect_ms,
        evaluate_ms,
        operation_ms,
        payload_serialize_ms,
        trial_ms: elapsed_ms(trial_started),
    }))
}

fn elapsed_ms(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn nearest_rank(samples: &[u64], percentile: usize) -> Option<u64> {
    if samples.is_empty() {
        return None;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).saturating_add(99) / 100;
    sorted.get(rank.saturating_sub(1)).copied()
}

#[derive(Debug, Default, Serialize)]
struct Percentiles {
    p50_ms: Option<u64>,
    p95_ms: Option<u64>,
}

impl Percentiles {
    fn from(samples: &[u64]) -> Self {
        Self {
            p50_ms: nearest_rank(samples, 50),
            p95_ms: nearest_rank(samples, 95),
        }
    }
}

#[derive(Debug, Serialize)]
struct CohortSummary {
    operation: Operation,
    requested: usize,
    completed: usize,
    success: usize,
    failure: usize,
    deadline_stops: usize,
    invalid_trial_stops: usize,
    failure_stages: BTreeMap<String, usize>,
    connect: Percentiles,
    evaluate: Percentiles,
    operation_time: Percentiles,
    payload_serialize: Percentiles,
    trial: Percentiles,
    normalization_residual: Percentiles,
    unattributed_residual: Percentiles,
}

impl CohortSummary {
    fn new(operation: Operation) -> Self {
        Self {
            operation,
            requested: TRIALS_PER_COHORT,
            completed: 0,
            success: 0,
            failure: 0,
            deadline_stops: 0,
            invalid_trial_stops: 0,
            failure_stages: BTreeMap::new(),
            connect: Percentiles::default(),
            evaluate: Percentiles::default(),
            operation_time: Percentiles::default(),
            payload_serialize: Percentiles::default(),
            trial: Percentiles::default(),
            normalization_residual: Percentiles::default(),
            unattributed_residual: Percentiles::default(),
        }
    }

    fn finish(&mut self, samples: &[TrialTimings]) {
        self.connect = Percentiles::from(
            &samples
                .iter()
                .map(|s| s.direct.connect_ms)
                .collect::<Vec<_>>(),
        );
        self.evaluate = Percentiles::from(
            &samples
                .iter()
                .map(|s| s.direct.evaluate_ms)
                .collect::<Vec<_>>(),
        );
        self.operation_time = Percentiles::from(
            &samples
                .iter()
                .map(|s| s.direct.operation_ms)
                .collect::<Vec<_>>(),
        );
        self.payload_serialize = Percentiles::from(
            &samples
                .iter()
                .map(|s| s.direct.payload_serialize_ms)
                .collect::<Vec<_>>(),
        );
        self.trial = Percentiles::from(
            &samples
                .iter()
                .map(|s| s.direct.trial_ms)
                .collect::<Vec<_>>(),
        );
        self.normalization_residual = Percentiles::from(
            &samples
                .iter()
                .map(|s| s.normalization_residual_ms)
                .collect::<Vec<_>>(),
        );
        self.unattributed_residual = Percentiles::from(
            &samples
                .iter()
                .map(|s| s.unattributed_residual_ms)
                .collect::<Vec<_>>(),
        );
        assert_eq!(self.success + self.failure, self.completed);
        assert!(self.completed <= self.requested);
    }
}

fn failure_stage(error: &AppError) -> &'static str {
    match error
        .details
        .as_ref()
        .and_then(|details| details.get("failure_stage"))
        .and_then(Value::as_str)
    {
        Some("target_list") => "target_list",
        Some("target_select") => "target_select",
        Some("websocket_connect") => "websocket_connect",
        Some("method_call") => "method_call",
        Some("event_wait") => "event_wait",
        _ => "transport_unknown",
    }
}

fn invalid_trial_error(error: &AppError) -> bool {
    error.message.contains("expected one evaluation")
        || error.message == "latency attribution payload serialization failed"
}

async fn run_live_cohort(
    operation: Operation,
    config: &TransportConfig,
    cohort_deadline: TokioInstant,
    run_deadline: TokioInstant,
) -> CohortSummary {
    let mut summary = CohortSummary::new(operation);
    let mut samples = Vec::new();

    for _ in 0..TRIALS_PER_COHORT {
        let now = TokioInstant::now();
        if now >= cohort_deadline || now >= run_deadline {
            summary.deadline_stops += 1;
            break;
        }
        let trial_deadline = (now + TRIAL_TIMEOUT).min(cohort_deadline).min(run_deadline);
        let result = timeout_at(
            trial_deadline,
            run_trial(
                || connect_runtime(config),
                |runtime| {
                    Box::pin(async move {
                        match operation {
                            Operation::OhlcvSummary => ohlcv_summary(runtime, Some(20)).await,
                            Operation::StudyValues => study_values(runtime).await,
                        }
                    })
                },
                |value| {
                    serde_json::to_vec(value).map_err(|_| {
                        AppError::new(
                            ErrorKind::Internal,
                            "latency attribution payload serialization failed",
                        )
                    })
                },
            ),
        )
        .await;

        summary.completed += 1;
        match result {
            Err(_) => {
                summary.failure += 1;
                summary.deadline_stops += 1;
                break;
            }
            Ok(Err(error)) => {
                summary.failure += 1;
                if invalid_trial_error(&error) {
                    summary.invalid_trial_stops += 1;
                    break;
                }
                *summary
                    .failure_stages
                    .entry(failure_stage(&error).to_string())
                    .or_default() += 1;
            }
            Ok(Ok(timings)) => {
                summary.success += 1;
                samples.push(timings);
            }
        }
    }
    summary.finish(&samples);
    summary
}

#[tokio::test]
#[ignore = "requires owner-approved TradingView Desktop latency measurement"]
async fn live_chart_read_latency_attribution() {
    assert_eq!(std::env::var(LIVE_GATE).as_deref(), Ok("1"));
    let target_id = std::env::var(LIVE_TARGET)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .expect("latency attribution requires an explicit target id");
    let config = TransportConfig::from_env_with_target_id(Some(&target_id))
        .expect("latency attribution transport configuration must be valid");
    let run_deadline = TokioInstant::now() + RUN_TIMEOUT;

    let mut summaries = Vec::new();
    for operation in [Operation::OhlcvSummary, Operation::StudyValues] {
        let cohort_deadline = (TokioInstant::now() + COHORT_TIMEOUT).min(run_deadline);
        summaries.push(run_live_cohort(operation, &config, cohort_deadline, run_deadline).await);
    }

    let encoded = serde_json::to_string(&summaries).expect("summary should serialize");
    assert_public_safe_summary(&encoded);
    println!("{encoded}");
}

fn assert_public_safe_summary(encoded: &str) {
    for forbidden in [
        "target_id",
        "webSocketDebuggerUrl",
        "ws://",
        "wss://",
        "symbol",
        "bars",
        "studies",
        "environment",
        "exception",
    ] {
        assert!(!encoded.contains(forbidden), "summary exposed {forbidden}");
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use serde_json::json;

    use super::*;

    struct DelayedRuntime {
        responses: VecDeque<Value>,
        delay: Duration,
        evaluate_count: usize,
    }

    impl RuntimeEvaluator for DelayedRuntime {
        async fn evaluate(
            &mut self,
            _expression: &str,
            _await_promise: bool,
        ) -> Result<Value, AppError> {
            self.evaluate_count += 1;
            tokio::time::sleep(self.delay).await;
            Ok(self.responses.pop_front().unwrap_or(Value::Null))
        }

        async fn capture_screenshot(&mut self) -> Result<Vec<u8>, AppError> {
            Ok(Vec::new())
        }

        async fn capture_screenshot_clip(
            &mut self,
            _clip: ScreenshotClip,
        ) -> Result<Vec<u8>, AppError> {
            Ok(Vec::new())
        }

        async fn insert_text(&mut self, _text: &str) -> Result<(), AppError> {
            Ok(())
        }

        async fn dispatch_key_event(&mut self, _event: KeyEvent) -> Result<(), AppError> {
            Ok(())
        }

        async fn dispatch_mouse_event(&mut self, _event: MouseEvent) -> Result<(), AppError> {
            Ok(())
        }
    }

    fn runtime(delay: Duration, responses: impl Into<VecDeque<Value>>) -> DelayedRuntime {
        DelayedRuntime {
            responses: responses.into(),
            delay,
            evaluate_count: 0,
        }
    }

    #[tokio::test]
    async fn trial_attributes_direct_and_derived_phases() {
        let result = run_trial(
            || async {
                tokio::time::sleep(Duration::from_millis(20)).await;
                Ok(runtime(Duration::from_millis(30), [json!({"ok": true})]))
            },
            |runtime| {
                Box::pin(async move {
                    let value = runtime.evaluate("fixture", false).await?;
                    tokio::time::sleep(Duration::from_millis(40)).await;
                    Ok(value)
                })
            },
            |value| {
                std::thread::sleep(Duration::from_millis(10));
                serde_json::to_vec(value)
                    .map_err(|_| AppError::new(ErrorKind::Internal, "fixture serialization failed"))
            },
        )
        .await
        .unwrap();

        assert!(result.direct.connect_ms >= 15);
        assert!(result.direct.evaluate_ms >= 25);
        assert!(result.direct.operation_ms >= 65);
        assert!(result.direct.payload_serialize_ms >= 8);
        assert!(result.direct.trial_ms >= 90);
        assert!(result.normalization_residual_ms >= 35);
        assert!(result.unattributed_residual_ms <= 15);
    }

    #[tokio::test]
    async fn trial_rejects_zero_or_multiple_evaluations() {
        let zero = run_trial(
            || async { Ok(runtime(Duration::ZERO, [])) },
            |_runtime| Box::pin(async { Ok(json!({"ok": true})) }),
            |value| serde_json::to_vec(value).map_err(|_| AppError::new(ErrorKind::Internal, "x")),
        )
        .await
        .unwrap_err();
        assert!(zero.message.contains("observed 0"));

        let multiple = run_trial(
            || async { Ok(runtime(Duration::ZERO, [Value::Null, Value::Null])) },
            |runtime| {
                Box::pin(async move {
                    runtime.evaluate("one", false).await?;
                    runtime.evaluate("two", false).await
                })
            },
            |value| serde_json::to_vec(value).map_err(|_| AppError::new(ErrorKind::Internal, "x")),
        )
        .await
        .unwrap_err();
        assert!(multiple.message.contains("observed 2"));
    }

    #[tokio::test]
    async fn real_operation_paths_each_observe_exactly_one_evaluation() {
        let ohlcv = run_trial(
            || async {
                Ok(runtime(
                    Duration::ZERO,
                    [json!({
                        "bars": [
                            {"time": 1, "open": 10.0, "high": 12.0, "low": 9.0, "close": 11.0, "volume": 100.0}
                        ]
                    })],
                ))
            },
            |runtime| Box::pin(async move { ohlcv_summary(runtime, Some(20)).await }),
            |value| {
                serde_json::to_vec(value).map_err(|_| {
                    AppError::new(
                        ErrorKind::Internal,
                        "latency attribution payload serialization failed",
                    )
                })
            },
        )
        .await
        .unwrap();
        assert!(ohlcv.direct.operation_ms >= ohlcv.direct.evaluate_ms);

        let values = run_trial(
            || async {
                Ok(runtime(
                    Duration::ZERO,
                    [json!({"study_count": 0, "studies": []})],
                ))
            },
            |runtime| Box::pin(async move { study_values(runtime).await }),
            |value| {
                serde_json::to_vec(value).map_err(|_| {
                    AppError::new(
                        ErrorKind::Internal,
                        "latency attribution payload serialization failed",
                    )
                })
            },
        )
        .await
        .unwrap();
        assert!(values.direct.operation_ms >= values.direct.evaluate_ms);
    }

    #[tokio::test]
    async fn malformed_serialization_is_an_invalid_trial() {
        let error = run_trial(
            || async { Ok(runtime(Duration::ZERO, [json!({"ok": true})])) },
            |runtime| Box::pin(async move { runtime.evaluate("fixture", false).await }),
            |_value| {
                Err(AppError::new(
                    ErrorKind::Internal,
                    "latency attribution payload serialization failed",
                ))
            },
        )
        .await
        .unwrap_err();

        assert!(invalid_trial_error(&error));
    }

    #[tokio::test]
    async fn absolute_deadline_stops_delayed_trial() {
        let result = timeout_at(
            TokioInstant::now() + Duration::from_millis(20),
            run_trial(
                || async { Ok(runtime(Duration::from_secs(1), [Value::Null])) },
                |runtime| Box::pin(async move { runtime.evaluate("slow", false).await }),
                |value| {
                    serde_json::to_vec(value).map_err(|_| AppError::new(ErrorKind::Internal, "x"))
                },
            ),
        )
        .await;
        assert!(result.is_err());
    }

    #[test]
    fn percentiles_and_saturation_are_deterministic() {
        assert_eq!(nearest_rank(&[5, 1, 4, 2, 3], 50), Some(3));
        assert_eq!(nearest_rank(&[5, 1, 4, 2, 3], 95), Some(5));
        let timings = TrialTimings::new(DirectTimings {
            connect_ms: u64::MAX,
            evaluate_ms: 20,
            operation_ms: 10,
            payload_serialize_ms: u64::MAX,
            trial_ms: 5,
        });
        assert_eq!(timings.normalization_residual_ms, 0);
        assert_eq!(timings.unattributed_residual_ms, 0);
    }

    #[test]
    fn summary_serialization_is_aggregate_only() {
        let mut summary = CohortSummary::new(Operation::StudyValues);
        summary.completed = 1;
        summary.failure = 1;
        summary
            .failure_stages
            .insert("transport_unknown".to_string(), 1);
        summary.finish(&[]);
        let encoded = serde_json::to_string(&[summary]).unwrap();
        assert_public_safe_summary(&encoded);
        assert!(!encoded.contains("private-marker"));
    }

    #[test]
    fn failure_stage_uses_the_public_allowlist() {
        for stage in [
            "target_list",
            "target_select",
            "websocket_connect",
            "method_call",
            "event_wait",
        ] {
            let error = AppError::new(ErrorKind::Connection, "fixed")
                .with_details(json!({"failure_stage": stage}));
            assert_eq!(failure_stage(&error), stage);
        }
        let unknown = AppError::new(ErrorKind::Connection, "fixed")
            .with_details(json!({"failure_stage": "private-stage"}));
        assert_eq!(failure_stage(&unknown), "transport_unknown");
    }
}
