use std::{collections::BTreeMap, future::Future, time::Duration};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::time::{Instant, sleep_until, timeout_at};
use tradingview_cdp::{CdpClient, CdpHttpSession, RuntimeEvaluator, TransportConfig};
use tradingview_core::{AppError, ErrorKind};

const LIVE_GATE: &str = "TV_LIVE_RENDERER_FOREGROUND_FEASIBILITY";
const RESTORE_TARGET_ENV: &str = "TV_LIVE_RENDERER_RESTORE_TARGET_ID";
const PROBE_TARGET_ENV: &str = "TV_LIVE_RENDERER_PROBE_TARGET_ID";
const TIMER_TIMEOUT: Duration = Duration::from_secs(3);
const CANDIDATE_TIMEOUT: Duration = Duration::from_secs(12);
const RUN_TIMEOUT: Duration = Duration::from_secs(60);
const POLL_INTERVAL: Duration = Duration::from_millis(100);

const SNAPSHOT_EXPRESSION: &str = r#"(() => {
  const marker = window.__tvCliRendererForegroundProbeV1;
  const visibility = ['visible', 'hidden', 'prerender'].includes(document.visibilityState)
    ? document.visibilityState : 'unknown';
  return {
    visibility,
    hidden: document.hidden === true,
    has_focus: document.hasFocus() === true,
    viewport_positive: window.innerWidth > 0 && window.innerHeight > 0,
    marker_present: marker !== undefined,
    timeout_completed: marker?.timeout_completed === true,
    animation_frame_completed: marker?.animation_frame_completed === true,
  };
})()"#;

const INSTALL_EXPRESSION: &str = r#"(() => {
  const key = '__tvCliRendererForegroundProbeV1';
  if (Object.prototype.hasOwnProperty.call(window, key)) return false;
  const token = { active: true, timeout_completed: false, animation_frame_completed: false };
  window[key] = token;
  setTimeout(() => {
    if (token.active && window[key] === token) token.timeout_completed = true;
  }, 0);
  requestAnimationFrame(() => {
    if (token.active && window[key] === token) token.animation_frame_completed = true;
  });
  return true;
})()"#;

const CLEANUP_EXPRESSION: &str = r#"(() => {
  const key = '__tvCliRendererForegroundProbeV1';
  const token = window[key];
  if (token && typeof token === 'object') token.active = false;
  delete window[key];
  return !Object.prototype.hasOwnProperty.call(window, key);
})()"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TargetRole {
    Restore,
    Probe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum TransitionCandidate {
    HttpActivate,
    PageBringToFront,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ObservationSnapshot {
    visibility: String,
    hidden: bool,
    has_focus: bool,
    viewport_positive: bool,
    marker_present: bool,
    timeout_completed: bool,
    animation_frame_completed: bool,
}

impl ObservationSnapshot {
    fn callbacks_complete(&self) -> bool {
        self.timeout_completed && self.animation_frame_completed
    }

    fn valid(&self) -> bool {
        matches!(
            self.visibility.as_str(),
            "visible" | "hidden" | "prerender" | "unknown"
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TimerTrial {
    initial: ObservationSnapshot,
    observed: ObservationSnapshot,
    marker_absent: bool,
}

impl TimerTrial {
    fn callbacks_complete(&self) -> bool {
        self.observed.callbacks_complete()
    }

    fn restore_matches(&self, baseline: &Self) -> bool {
        self.marker_absent
            && self.observed.visibility == baseline.observed.visibility
            && self.observed.hidden == baseline.observed.hidden
            && self.observed.has_focus == baseline.observed.has_focus
            && self.observed.viewport_positive == baseline.observed.viewport_positive
            && self.callbacks_complete() == baseline.callbacks_complete()
    }
}

trait ProbeBackend {
    fn timer_timeout(&self) -> Duration {
        TIMER_TIMEOUT
    }

    fn poll_interval(&self) -> Duration {
        POLL_INTERVAL
    }

    fn candidate_timeout(&self) -> Duration {
        CANDIDATE_TIMEOUT
    }

    fn run_timeout(&self) -> Duration {
        RUN_TIMEOUT
    }

    fn snapshot(
        &mut self,
        role: TargetRole,
    ) -> impl Future<Output = Result<ObservationSnapshot, AppError>> + Send;
    fn install_marker(
        &mut self,
        role: TargetRole,
    ) -> impl Future<Output = Result<bool, AppError>> + Send;
    fn cleanup_marker(
        &mut self,
        role: TargetRole,
    ) -> impl Future<Output = Result<bool, AppError>> + Send;
    fn transition(
        &mut self,
        candidate: TransitionCandidate,
        role: TargetRole,
    ) -> impl Future<Output = Result<(), AppError>> + Send;
}

#[derive(Debug, Clone, Serialize)]
struct ObservationCounts {
    visibility: BTreeMap<String, usize>,
    hidden: usize,
    has_focus: usize,
    viewport_positive: usize,
    timeout_completed: usize,
    animation_frame_completed: usize,
}

impl From<&ObservationSnapshot> for ObservationCounts {
    fn from(snapshot: &ObservationSnapshot) -> Self {
        Self {
            visibility: BTreeMap::from([(snapshot.visibility.clone(), 1)]),
            hidden: usize::from(snapshot.hidden),
            has_focus: usize::from(snapshot.has_focus),
            viewport_positive: usize::from(snapshot.viewport_positive),
            timeout_completed: usize::from(snapshot.timeout_completed),
            animation_frame_completed: usize::from(snapshot.animation_frame_completed),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct CandidateResult {
    candidate: TransitionCandidate,
    requested: usize,
    completed: usize,
    transition_calls: usize,
    restoration_calls: usize,
    responsive_failures: usize,
    unknown_stops: usize,
    restore_observation_matched: bool,
    probe_callbacks_completed: bool,
    before_probe: ObservationCounts,
    after_probe: Option<ObservationCounts>,
    failure_stages: BTreeMap<String, usize>,
}

impl CandidateResult {
    fn new(candidate: TransitionCandidate, baseline: &ObservationSnapshot) -> Self {
        Self {
            candidate,
            requested: 1,
            completed: 0,
            transition_calls: 0,
            restoration_calls: 0,
            responsive_failures: 0,
            unknown_stops: 0,
            restore_observation_matched: false,
            probe_callbacks_completed: false,
            before_probe: ObservationCounts::from(baseline),
            after_probe: None,
            failure_stages: BTreeMap::new(),
        }
    }

    fn record_error(&mut self, error: &AppError) {
        self.responsive_failures += 1;
        *self
            .failure_stages
            .entry(failure_stage(error).to_string())
            .or_default() += 1;
    }
}

#[derive(Debug, Serialize)]
struct MatrixSummary {
    status: &'static str,
    baseline_responsive_failures: usize,
    unknown_stops: usize,
    candidates: Vec<CandidateResult>,
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

fn invalid(message: &'static str) -> AppError {
    AppError::new(ErrorKind::Internal, message)
}

async fn timer_trial<B: ProbeBackend + Send>(
    backend: &mut B,
    role: TargetRole,
    deadline: Instant,
) -> Result<TimerTrial, AppError> {
    let initial = backend.snapshot(role).await?;
    if !initial.valid() || initial.marker_present {
        return Err(invalid("renderer probe baseline was malformed or occupied"));
    }
    match backend.install_marker(role).await {
        Ok(true) => {}
        Ok(false) => return Err(invalid("renderer probe marker installation was refused")),
        Err(error) => {
            cleanup_and_verify(backend, role).await?;
            return Err(error);
        }
    }

    let observed_result = async {
        loop {
            let snapshot = backend.snapshot(role).await?;
            if !snapshot.valid() || !snapshot.marker_present {
                return Err(invalid("renderer probe marker observation was malformed"));
            }
            if snapshot.callbacks_complete() || Instant::now() >= deadline {
                break Ok(snapshot);
            }
            sleep_until((Instant::now() + backend.poll_interval()).min(deadline)).await;
            if Instant::now() >= deadline {
                break Ok(snapshot);
            }
        }
    }
    .await;

    let cleanup_result = cleanup_and_verify(backend, role).await;
    let observed = match observed_result {
        Ok(snapshot) => {
            cleanup_result?;
            snapshot
        }
        Err(error) => {
            cleanup_result?;
            return Err(error);
        }
    };

    Ok(TimerTrial {
        initial,
        observed,
        marker_absent: true,
    })
}

async fn cleanup_and_verify<B: ProbeBackend + Send>(
    backend: &mut B,
    role: TargetRole,
) -> Result<(), AppError> {
    if !backend.cleanup_marker(role).await? {
        return Err(invalid("renderer probe marker cleanup failed"));
    }
    let final_snapshot = backend.snapshot(role).await?;
    if !final_snapshot.valid() || final_snapshot.marker_present {
        return Err(invalid("renderer probe marker cleanup was not verified"));
    }
    Ok(())
}

/*
The marker lifecycle above deliberately keeps cleanup outside the observation
future's error propagation. Responsive failures therefore perform one cleanup
and one verification, while cancellation by the outer timeout performs neither.
*/

async fn bounded_timer_trial<B: ProbeBackend + Send>(
    backend: &mut B,
    role: TargetRole,
    parent_deadline: Instant,
) -> Result<Result<TimerTrial, AppError>, ()> {
    let deadline = (Instant::now() + backend.timer_timeout()).min(parent_deadline);
    timeout_at(deadline, timer_trial(backend, role, deadline))
        .await
        .map_err(|_| ())
}

async fn candidate_inner<B: ProbeBackend + Send>(
    backend: &mut B,
    candidate: TransitionCandidate,
    restore_baseline: &TimerTrial,
    deadline: Instant,
    result: &mut CandidateResult,
) {
    result.transition_calls += 1;
    let probe_result = match backend.transition(candidate, TargetRole::Probe).await {
        Ok(()) => bounded_timer_trial(backend, TargetRole::Probe, deadline).await,
        Err(error) => {
            result.record_error(&error);
            Ok(Err(error))
        }
    };

    match probe_result {
        Err(()) => {
            result.unknown_stops += 1;
            return;
        }
        Ok(Err(error)) if result.responsive_failures == 0 => result.record_error(&error),
        Ok(Err(_)) => {}
        Ok(Ok(trial)) => {
            result.probe_callbacks_completed = trial.callbacks_complete();
            result.after_probe = Some(ObservationCounts::from(&trial.observed));
        }
    }

    result.restoration_calls += 1;
    if let Err(error) = backend.transition(candidate, TargetRole::Restore).await {
        result.record_error(&error);
        return;
    }
    match bounded_timer_trial(backend, TargetRole::Restore, deadline).await {
        Err(()) => result.unknown_stops += 1,
        Ok(Err(error)) => result.record_error(&error),
        Ok(Ok(trial)) => {
            result.restore_observation_matched = trial.restore_matches(restore_baseline)
        }
    }
    result.completed = 1;
}

async fn run_matrix<B: ProbeBackend + Send>(backend: &mut B) -> MatrixSummary {
    let run_deadline = Instant::now() + backend.run_timeout();
    let probe_baseline = match bounded_timer_trial(backend, TargetRole::Probe, run_deadline).await {
        Ok(Ok(trial)) => trial,
        Ok(Err(_)) => {
            return MatrixSummary {
                status: "baseline_failed",
                baseline_responsive_failures: 1,
                unknown_stops: 0,
                candidates: Vec::new(),
            };
        }
        Err(()) => {
            return MatrixSummary {
                status: "unknown_stop",
                baseline_responsive_failures: 0,
                unknown_stops: 1,
                candidates: Vec::new(),
            };
        }
    };
    let restore_baseline =
        match bounded_timer_trial(backend, TargetRole::Restore, run_deadline).await {
            Ok(Ok(trial)) => trial,
            Ok(Err(_)) => {
                return MatrixSummary {
                    status: "baseline_failed",
                    baseline_responsive_failures: 1,
                    unknown_stops: 0,
                    candidates: Vec::new(),
                };
            }
            Err(()) => {
                return MatrixSummary {
                    status: "unknown_stop",
                    baseline_responsive_failures: 0,
                    unknown_stops: 1,
                    candidates: Vec::new(),
                };
            }
        };

    if probe_baseline.callbacks_complete() {
        return MatrixSummary {
            status: "no_observable_need",
            baseline_responsive_failures: 0,
            unknown_stops: 0,
            candidates: Vec::new(),
        };
    }

    let mut candidates = Vec::new();
    for candidate in [
        TransitionCandidate::HttpActivate,
        TransitionCandidate::PageBringToFront,
    ] {
        let mut result = CandidateResult::new(candidate, &probe_baseline.observed);
        let deadline = (Instant::now() + backend.candidate_timeout()).min(run_deadline);
        if timeout_at(
            deadline,
            candidate_inner(backend, candidate, &restore_baseline, deadline, &mut result),
        )
        .await
        .is_err()
        {
            result.unknown_stops += 1;
        }
        let unknown = result.unknown_stops > 0;
        candidates.push(result);
        if unknown || Instant::now() >= run_deadline {
            break;
        }
    }
    MatrixSummary {
        status: "completed",
        baseline_responsive_failures: 0,
        unknown_stops: candidates.iter().map(|result| result.unknown_stops).sum(),
        candidates,
    }
}

struct LiveBackend {
    session: CdpHttpSession,
    restore: CdpClient,
    probe: CdpClient,
    restore_target_id: String,
    probe_target_id: String,
}

impl LiveBackend {
    fn client(&mut self, role: TargetRole) -> &mut CdpClient {
        match role {
            TargetRole::Restore => &mut self.restore,
            TargetRole::Probe => &mut self.probe,
        }
    }

    fn target_id(&self, role: TargetRole) -> &str {
        match role {
            TargetRole::Restore => &self.restore_target_id,
            TargetRole::Probe => &self.probe_target_id,
        }
    }
}

impl ProbeBackend for LiveBackend {
    async fn snapshot(&mut self, role: TargetRole) -> Result<ObservationSnapshot, AppError> {
        let value = self
            .client(role)
            .evaluate(SNAPSHOT_EXPRESSION, false)
            .await?;
        serde_json::from_value(value)
            .map_err(|_| invalid("renderer probe snapshot response was malformed"))
    }

    async fn install_marker(&mut self, role: TargetRole) -> Result<bool, AppError> {
        Ok(self
            .client(role)
            .evaluate(INSTALL_EXPRESSION, false)
            .await?
            .as_bool()
            == Some(true))
    }

    async fn cleanup_marker(&mut self, role: TargetRole) -> Result<bool, AppError> {
        Ok(self
            .client(role)
            .evaluate(CLEANUP_EXPRESSION, false)
            .await?
            .as_bool()
            == Some(true))
    }

    async fn transition(
        &mut self,
        candidate: TransitionCandidate,
        role: TargetRole,
    ) -> Result<(), AppError> {
        match candidate {
            TransitionCandidate::HttpActivate => {
                let target_id = self.target_id(role).to_string();
                self.session.activate_target(&target_id).await
            }
            TransitionCandidate::PageBringToFront => {
                self.client(role)
                    .call_method("Page.bringToFront", json!({}))
                    .await?;
                Ok(())
            }
        }
    }
}

async fn live_backend(restore_target_id: String, probe_target_id: String) -> LiveBackend {
    let restore_config = TransportConfig::from_env_with_target_id(Some(&restore_target_id))
        .expect("renderer probe restore transport configuration must be valid");
    let probe_config = TransportConfig::from_env_with_target_id(Some(&probe_target_id))
        .expect("renderer probe target transport configuration must be valid");
    let session = CdpHttpSession::new(&restore_config)
        .expect("renderer probe HTTP session must be constructible");
    let restore_target = session
        .discover_target()
        .await
        .expect("renderer probe restore target must resolve exactly");
    let probe_target = CdpHttpSession::new(&probe_config)
        .expect("renderer probe target HTTP session must be constructible")
        .discover_target()
        .await
        .expect("renderer probe target must resolve exactly");
    let restore = CdpClient::connect(&restore_target)
        .await
        .expect("renderer probe restore WebSocket must connect");
    let probe = CdpClient::connect(&probe_target)
        .await
        .expect("renderer probe WebSocket must connect");
    LiveBackend {
        session,
        restore,
        probe,
        restore_target_id,
        probe_target_id,
    }
}

#[tokio::test]
#[ignore = "requires two explicit TradingView Desktop targets and owner approval"]
async fn live_renderer_foreground_feasibility() {
    assert_eq!(std::env::var(LIVE_GATE).as_deref(), Ok("1"));
    let restore_target = required_target(RESTORE_TARGET_ENV);
    let probe_target = required_target(PROBE_TARGET_ENV);
    assert_ne!(
        restore_target, probe_target,
        "renderer probe targets must differ"
    );

    let mut backend = live_backend(restore_target, probe_target).await;
    let summary = run_matrix(&mut backend).await;
    let encoded = serde_json::to_string(&summary).expect("renderer summary should serialize");
    assert_public_safe_summary(&encoded);
    println!("{encoded}");
}

fn required_target(name: &str) -> String {
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| panic!("{name} must be a non-empty explicit target id"))
}

fn assert_public_safe_summary(encoded: &str) {
    for forbidden in [
        "target_id",
        "title",
        "url",
        "symbol",
        "dom",
        "marker",
        "token",
        "payload",
        "exception",
        "stack",
        "endpoint",
        "account",
        "environment",
        "ws://",
        "wss://",
    ] {
        assert!(!encoded.to_ascii_lowercase().contains(forbidden));
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Call {
        Snapshot(TargetRole),
        Install(TargetRole),
        Cleanup(TargetRole),
        Transition(TransitionCandidate, TargetRole),
    }

    struct FakeBackend {
        snapshots: VecDeque<ObservationSnapshot>,
        calls: Vec<Call>,
        snapshot_calls: usize,
        fail_snapshot_call: Option<usize>,
        transition_delay: Duration,
        candidate_timeout: Duration,
    }

    impl FakeBackend {
        fn new(snapshots: impl Into<VecDeque<ObservationSnapshot>>) -> Self {
            Self {
                snapshots: snapshots.into(),
                calls: Vec::new(),
                snapshot_calls: 0,
                fail_snapshot_call: None,
                transition_delay: Duration::ZERO,
                candidate_timeout: Duration::from_millis(100),
            }
        }

        fn failing_snapshot(mut self, call: usize) -> Self {
            self.fail_snapshot_call = Some(call);
            self
        }

        fn delayed_transition(mut self, delay: Duration) -> Self {
            self.transition_delay = delay;
            self.candidate_timeout = Duration::from_millis(5);
            self
        }
    }

    impl ProbeBackend for FakeBackend {
        fn timer_timeout(&self) -> Duration {
            Duration::from_millis(5)
        }

        fn poll_interval(&self) -> Duration {
            Duration::from_millis(5)
        }

        fn candidate_timeout(&self) -> Duration {
            self.candidate_timeout
        }

        fn run_timeout(&self) -> Duration {
            Duration::from_secs(1)
        }

        async fn snapshot(&mut self, role: TargetRole) -> Result<ObservationSnapshot, AppError> {
            self.calls.push(Call::Snapshot(role));
            self.snapshot_calls += 1;
            if self.fail_snapshot_call == Some(self.snapshot_calls) {
                return Err(AppError::new(
                    ErrorKind::Connection,
                    "fixture responsive snapshot failure",
                ));
            }
            self.snapshots
                .pop_front()
                .ok_or_else(|| invalid("fixture snapshot exhausted"))
        }

        async fn install_marker(&mut self, role: TargetRole) -> Result<bool, AppError> {
            self.calls.push(Call::Install(role));
            Ok(true)
        }

        async fn cleanup_marker(&mut self, role: TargetRole) -> Result<bool, AppError> {
            self.calls.push(Call::Cleanup(role));
            Ok(true)
        }

        async fn transition(
            &mut self,
            candidate: TransitionCandidate,
            role: TargetRole,
        ) -> Result<(), AppError> {
            self.calls.push(Call::Transition(candidate, role));
            tokio::time::sleep(self.transition_delay).await;
            Ok(())
        }
    }

    fn snapshot(marker: bool, complete: bool) -> ObservationSnapshot {
        ObservationSnapshot {
            visibility: "hidden".into(),
            hidden: true,
            has_focus: false,
            viewport_positive: true,
            marker_present: marker,
            timeout_completed: complete,
            animation_frame_completed: complete,
        }
    }

    fn timer_snapshots(complete: bool) -> [ObservationSnapshot; 3] {
        [
            snapshot(false, false),
            snapshot(true, complete),
            snapshot(false, false),
        ]
    }

    #[tokio::test]
    async fn ready_probe_baseline_stops_before_transitions_even_if_restore_is_incomplete() {
        let snapshots = timer_snapshots(true)
            .into_iter()
            .chain(timer_snapshots(false))
            .collect::<VecDeque<_>>();
        let mut backend = FakeBackend::new(snapshots);
        let summary = run_matrix(&mut backend).await;

        assert_eq!(summary.status, "no_observable_need");
        assert!(summary.candidates.is_empty());
        assert!(
            !backend
                .calls
                .iter()
                .any(|call| matches!(call, Call::Transition(..)))
        );
    }

    #[tokio::test]
    async fn candidates_use_exact_transition_restore_order() {
        let snapshots = timer_snapshots(false)
            .into_iter()
            .chain(timer_snapshots(false))
            .chain(timer_snapshots(true))
            .chain(timer_snapshots(false))
            .chain(timer_snapshots(true))
            .chain(timer_snapshots(false))
            .collect::<VecDeque<_>>();
        let mut backend = FakeBackend::new(snapshots);
        let summary = run_matrix(&mut backend).await;

        assert_eq!(summary.status, "completed");
        assert_eq!(summary.candidates.len(), 2);
        let transitions = backend
            .calls
            .iter()
            .filter_map(|call| match call {
                Call::Transition(candidate, role) => Some((*candidate, *role)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            transitions,
            vec![
                (TransitionCandidate::HttpActivate, TargetRole::Probe),
                (TransitionCandidate::HttpActivate, TargetRole::Restore),
                (TransitionCandidate::PageBringToFront, TargetRole::Probe),
                (TransitionCandidate::PageBringToFront, TargetRole::Restore),
            ]
        );
        assert!(summary.candidates.iter().all(|result| {
            result.transition_calls == 1
                && result.restoration_calls == 1
                && result.restore_observation_matched
        }));
    }

    #[tokio::test]
    async fn responsive_probe_failure_cleans_marker_then_restores_once() {
        let snapshots = timer_snapshots(false)
            .into_iter()
            .chain(timer_snapshots(false))
            .chain([snapshot(false, false), snapshot(false, false)])
            .chain(timer_snapshots(false))
            .collect::<VecDeque<_>>();
        let mut backend = FakeBackend::new(snapshots).failing_snapshot(8);
        let summary = run_matrix(&mut backend).await;

        let first = &summary.candidates[0];
        assert_eq!(first.responsive_failures, 1);
        assert_eq!(first.restoration_calls, 1);
        assert_eq!(
            backend
                .calls
                .iter()
                .filter(|call| matches!(call, Call::Cleanup(TargetRole::Probe)))
                .count(),
            2
        );
        assert_eq!(
            backend
                .calls
                .iter()
                .filter(|call| matches!(
                    call,
                    Call::Transition(TransitionCandidate::HttpActivate, TargetRole::Restore)
                ))
                .count(),
            1
        );
    }

    #[tokio::test]
    async fn unknown_transition_timeout_does_not_restore_or_retry() {
        let snapshots = timer_snapshots(false)
            .into_iter()
            .chain(timer_snapshots(false))
            .collect::<VecDeque<_>>();
        let mut backend = FakeBackend::new(snapshots).delayed_transition(Duration::from_millis(20));
        let summary = run_matrix(&mut backend).await;

        assert_eq!(summary.candidates.len(), 1);
        assert_eq!(summary.candidates[0].unknown_stops, 1);
        assert_eq!(summary.candidates[0].restoration_calls, 0);
        assert_eq!(
            backend
                .calls
                .iter()
                .filter(|call| matches!(call, Call::Transition(..)))
                .count(),
            1
        );
    }

    #[test]
    fn expressions_use_fixed_semantic_marker_without_class_or_text_evidence() {
        for expression in [SNAPSHOT_EXPRESSION, INSTALL_EXPRESSION, CLEANUP_EXPRESSION] {
            assert!(expression.contains("__tvCliRendererForegroundProbeV1"));
            assert!(!expression.contains("className"));
            assert!(!expression.contains("textContent"));
        }
    }

    #[test]
    fn aggregate_is_public_safe_and_uses_limited_restore_wording() {
        let summary = MatrixSummary {
            status: "completed",
            baseline_responsive_failures: 0,
            unknown_stops: 0,
            candidates: vec![CandidateResult::new(
                TransitionCandidate::HttpActivate,
                &snapshot(false, false),
            )],
        };
        let encoded = serde_json::to_string(&summary).unwrap();
        assert_public_safe_summary(&encoded);
        assert!(encoded.contains("restore_observation_matched"));
        assert!(!encoded.contains("desktop_tab_restored"));
        assert!(!encoded.contains("os_focus_restored"));
    }

    #[test]
    fn snapshot_rejects_unknown_fields_and_private_values() {
        let malformed = json!({
            "visibility": "hidden",
            "hidden": true,
            "has_focus": false,
            "viewport_positive": true,
            "marker_present": false,
            "timeout_completed": false,
            "animation_frame_completed": false,
            "target_id": "private-target"
        });
        assert!(serde_json::from_value::<ObservationSnapshot>(malformed).is_err());
    }
}
