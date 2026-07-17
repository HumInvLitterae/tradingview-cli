use std::{collections::BTreeMap, time::Duration};

use serde::Serialize;
use serde_json::Value;
use tokio::time::{Instant, timeout_at};
use tradingview_core::{AppError, ErrorKind};

use crate::{
    CdpClient, CdpHttpSession, RuntimeEvaluator, Target, TransportConfig,
    diagnostics::{
        PublicFailureStage, StageOutcome, StageSample, StaleTargetDiagnosis, TransportObserver,
        TransportStage, diagnose_stale_target_from_targets,
    },
};

const DEFAULT_ITERATIONS: u32 = 10;
const MAX_ITERATIONS: u32 = 100;
const DEFAULT_DEADLINE_MS: u64 = 120_000;
const MIN_DEADLINE_MS: u64 = 1_000;
const MAX_DEADLINE_MS: u64 = 300_000;

#[derive(Debug, Clone)]
struct ProbeConfig {
    target_id: String,
    iterations: u32,
    deadline: Duration,
}

impl ProbeConfig {
    fn from_env() -> Result<Self, &'static str> {
        Self::from_values(
            std::env::var("TV_LIVE_TRANSPORT_MEASUREMENT")
                .ok()
                .as_deref(),
            std::env::var("TV_LIVE_TRANSPORT_MEASUREMENT_TARGET_ID")
                .ok()
                .as_deref(),
            std::env::var("TV_LIVE_TRANSPORT_MEASUREMENT_ITERATIONS")
                .ok()
                .as_deref(),
            std::env::var("TV_LIVE_TRANSPORT_MEASUREMENT_DEADLINE_MS")
                .ok()
                .as_deref(),
        )
    }

    fn from_values(
        gate: Option<&str>,
        target_id: Option<&str>,
        iterations: Option<&str>,
        deadline_ms: Option<&str>,
    ) -> Result<Self, &'static str> {
        if gate != Some("1") {
            return Err("live transport measurement gate is not enabled");
        }
        let target_id = target_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or("live transport measurement requires an explicit target")?
            .to_string();
        let iterations = parse_bounded_u32(
            iterations,
            DEFAULT_ITERATIONS,
            1,
            MAX_ITERATIONS,
            "live transport measurement iterations are invalid",
        )?;
        let deadline_ms = parse_bounded_u64(
            deadline_ms,
            DEFAULT_DEADLINE_MS,
            MIN_DEADLINE_MS,
            MAX_DEADLINE_MS,
            "live transport measurement deadline is invalid",
        )?;
        Ok(Self {
            target_id,
            iterations,
            deadline: Duration::from_millis(deadline_ms),
        })
    }
}

fn parse_bounded_u32(
    raw: Option<&str>,
    default: u32,
    minimum: u32,
    maximum: u32,
    message: &'static str,
) -> Result<u32, &'static str> {
    let value = match raw {
        Some(raw) => raw.parse::<u32>().map_err(|_| message)?,
        None => default,
    };
    if !(minimum..=maximum).contains(&value) {
        return Err(message);
    }
    Ok(value)
}

fn parse_bounded_u64(
    raw: Option<&str>,
    default: u64,
    minimum: u64,
    maximum: u64,
    message: &'static str,
) -> Result<u64, &'static str> {
    let value = match raw {
        Some(raw) => raw.parse::<u64>().map_err(|_| message)?,
        None => default,
    };
    if !(minimum..=maximum).contains(&value) {
        return Err(message);
    }
    Ok(value)
}

#[derive(Debug, Clone, Default)]
struct ProbeAccumulator {
    iterations_requested: u32,
    iterations_completed: u32,
    success_count: u32,
    failure_count: u32,
    deadline_reached: bool,
    stage_samples: BTreeMap<TransportStage, Vec<u64>>,
    failure_stage_counts: BTreeMap<&'static str, u32>,
    stale_target_diagnosis_counts: BTreeMap<&'static str, u32>,
}

impl ProbeAccumulator {
    fn record_samples(&mut self, samples: Vec<StageSample>) -> Option<PublicFailureStage> {
        let failure_stage = samples.iter().rev().find_map(|sample| {
            matches!(sample.outcome, StageOutcome::Failure(_))
                .then(|| PublicFailureStage::from(Some(sample.stage)))
        });
        for sample in samples {
            self.stage_samples
                .entry(sample.stage)
                .or_default()
                .push(sample.elapsed_ms);
        }
        failure_stage
    }

    fn record_failure(&mut self, stage: PublicFailureStage) {
        self.failure_count = self.failure_count.saturating_add(1);
        *self.failure_stage_counts.entry(stage.as_str()).or_default() += 1;
    }

    fn record_diagnosis(&mut self, diagnosis: StaleTargetDiagnosis) {
        *self
            .stale_target_diagnosis_counts
            .entry(diagnosis.as_str())
            .or_default() += 1;
    }

    fn summary(&self) -> ProbeSummary {
        let stage_latency_ms = self
            .stage_samples
            .iter()
            .map(|(stage, samples)| {
                (
                    PublicFailureStage::from(Some(*stage)).as_str().to_string(),
                    StageLatency::from_samples(samples),
                )
            })
            .collect();
        ProbeSummary {
            iterations_requested: self.iterations_requested,
            iterations_completed: self.iterations_completed,
            success_count: self.success_count,
            failure_count: self.failure_count,
            deadline_reached: self.deadline_reached,
            stage_latency_ms,
            failure_stage_counts: owned_counts(&self.failure_stage_counts),
            stale_target_diagnosis_counts: owned_counts(&self.stale_target_diagnosis_counts),
        }
    }
}

fn owned_counts(source: &BTreeMap<&'static str, u32>) -> BTreeMap<String, u32> {
    source
        .iter()
        .map(|(key, value)| ((*key).to_string(), *value))
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProbeSummary {
    iterations_requested: u32,
    iterations_completed: u32,
    success_count: u32,
    failure_count: u32,
    deadline_reached: bool,
    stage_latency_ms: BTreeMap<String, StageLatency>,
    failure_stage_counts: BTreeMap<String, u32>,
    stale_target_diagnosis_counts: BTreeMap<String, u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct StageLatency {
    sample_count: usize,
    p50: Option<u64>,
    p95: Option<u64>,
}

impl StageLatency {
    fn from_samples(samples: &[u64]) -> Self {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        Self {
            sample_count: sorted.len(),
            p50: percentile(&sorted, 50),
            p95: percentile(&sorted, 95),
        }
    }
}

fn percentile(sorted: &[u64], percentile: usize) -> Option<u64> {
    if sorted.is_empty() {
        return None;
    }
    let rank = percentile
        .saturating_mul(sorted.len())
        .div_ceil(100)
        .saturating_sub(1);
    sorted.get(rank).copied()
}

async fn run_probe(config: ProbeConfig) -> ProbeSummary {
    let transport = TransportConfig::from_env_with_target_id(Some(&config.target_id));
    let Ok(transport) = transport else {
        return setup_failure_summary(config.iterations);
    };
    run_probe_with_transport(config, transport).await
}

async fn run_probe_with_transport(config: ProbeConfig, transport: TransportConfig) -> ProbeSummary {
    let mut accumulator = ProbeAccumulator {
        iterations_requested: config.iterations,
        ..ProbeAccumulator::default()
    };
    let observer = TransportObserver::default();
    let session = match CdpHttpSession::new(&transport) {
        Ok(session) => session.with_observer(observer.clone()),
        Err(_) => return setup_failure_summary(config.iterations),
    };
    let deadline = Instant::now() + config.deadline;

    for _ in 0..config.iterations {
        if Instant::now() >= deadline {
            accumulator.deadline_reached = true;
            break;
        }

        let list_started = Instant::now();
        let targets = match timeout_at(deadline, session.fetch_targets()).await {
            Ok(Ok(targets)) => {
                let _ = accumulator.record_samples(observer.take_samples());
                targets
            }
            Ok(Err(_)) => {
                let stage = accumulator
                    .record_samples(observer.take_samples())
                    .unwrap_or(PublicFailureStage::TransportUnknown);
                accumulator.record_failure(stage);
                accumulator.iterations_completed += 1;
                continue;
            }
            Err(_) => {
                observer.record(
                    TransportStage::TargetList,
                    list_started.elapsed(),
                    &Err::<(), _>(AppError::new(ErrorKind::Timeout, "probe deadline reached")),
                );
                let _ = accumulator.record_samples(observer.take_samples());
                accumulator.record_failure(PublicFailureStage::TargetList);
                accumulator.iterations_completed += 1;
                accumulator.deadline_reached = true;
                break;
            }
        };

        let target = match session.select_target_from(targets) {
            Ok(target) => {
                let _ = accumulator.record_samples(observer.take_samples());
                target
            }
            Err(_) => {
                let stage = accumulator
                    .record_samples(observer.take_samples())
                    .unwrap_or(PublicFailureStage::TransportUnknown);
                accumulator.record_failure(stage);
                accumulator.iterations_completed += 1;
                continue;
            }
        };

        let connect_started = Instant::now();
        let mut client = match timeout_at(
            deadline,
            CdpClient::connect_with_timeout_and_observer(
                &target,
                Duration::from_secs(5),
                Some(observer.clone()),
            ),
        )
        .await
        {
            Ok(Ok(client)) => {
                let _ = accumulator.record_samples(observer.take_samples());
                client
            }
            Ok(Err(_)) => {
                let stage = accumulator
                    .record_samples(observer.take_samples())
                    .unwrap_or(PublicFailureStage::TransportUnknown);
                accumulator.record_failure(stage);
                let diagnosis = diagnose_after_connect_failure(
                    &session,
                    &observer,
                    &target,
                    &config.target_id,
                    deadline,
                )
                .await;
                let _ = accumulator.record_samples(observer.take_samples());
                accumulator.record_diagnosis(diagnosis);
                accumulator.iterations_completed += 1;
                if Instant::now() >= deadline {
                    accumulator.deadline_reached = true;
                    break;
                }
                continue;
            }
            Err(_) => {
                observer.record(
                    TransportStage::WebSocketConnect,
                    connect_started.elapsed(),
                    &Err::<(), _>(AppError::new(ErrorKind::Timeout, "probe deadline reached")),
                );
                let _ = accumulator.record_samples(observer.take_samples());
                accumulator.record_failure(PublicFailureStage::WebsocketConnect);
                accumulator.iterations_completed += 1;
                accumulator.deadline_reached = true;
                break;
            }
        };

        let method_started = Instant::now();
        match timeout_at(deadline, client.evaluate("1", false)).await {
            Ok(Ok(Value::Number(_))) => {
                let _ = accumulator.record_samples(observer.take_samples());
                accumulator.success_count += 1;
            }
            Ok(Ok(_)) => {
                let _ = accumulator.record_samples(observer.take_samples());
                accumulator.record_failure(PublicFailureStage::MethodCall);
            }
            Ok(Err(_)) => {
                let stage = accumulator
                    .record_samples(observer.take_samples())
                    .unwrap_or(PublicFailureStage::TransportUnknown);
                accumulator.record_failure(stage);
            }
            Err(_) => {
                observer.record(
                    TransportStage::MethodCall,
                    method_started.elapsed(),
                    &Err::<(), _>(AppError::new(ErrorKind::Timeout, "probe deadline reached")),
                );
                let _ = accumulator.record_samples(observer.take_samples());
                accumulator.record_failure(PublicFailureStage::MethodCall);
                accumulator.iterations_completed += 1;
                accumulator.deadline_reached = true;
                break;
            }
        }
        accumulator.iterations_completed += 1;
    }

    accumulator.summary()
}

fn setup_failure_summary(iterations_requested: u32) -> ProbeSummary {
    let mut accumulator = ProbeAccumulator {
        iterations_requested,
        iterations_completed: 1,
        ..ProbeAccumulator::default()
    };
    accumulator.record_failure(PublicFailureStage::TransportUnknown);
    accumulator.summary()
}

async fn diagnose_after_connect_failure(
    session: &CdpHttpSession,
    observer: &TransportObserver,
    failed_target: &Target,
    target_id: &str,
    deadline: Instant,
) -> StaleTargetDiagnosis {
    let started = Instant::now();
    let fresh_result = match timeout_at(deadline, session.fetch_targets()).await {
        Ok(result) => result,
        Err(_) => {
            observer.record(
                TransportStage::TargetList,
                started.elapsed(),
                &Err::<(), _>(AppError::new(ErrorKind::Timeout, "probe deadline reached")),
            );
            Err(AppError::new(ErrorKind::Timeout, "probe deadline reached"))
        }
    };
    diagnose_stale_target_from_targets(failed_target, target_id, &fresh_result)
}

fn target(id: &str, endpoint: &str) -> Target {
    Target {
        id: id.to_string(),
        title: "test".to_string(),
        kind: "page".to_string(),
        url: "https://example.invalid/chart".to_string(),
        web_socket_debugger_url: Some(endpoint.to_string()),
    }
}

#[test]
fn probe_config_is_bounded_before_network_access() {
    assert!(ProbeConfig::from_values(None, Some("target"), None, None).is_err());
    assert!(ProbeConfig::from_values(Some("1"), None, None, None).is_err());
    assert!(ProbeConfig::from_values(Some("1"), Some(" "), None, None).is_err());
    assert!(ProbeConfig::from_values(Some("1"), Some("target"), Some("0"), None).is_err());
    assert!(ProbeConfig::from_values(Some("1"), Some("target"), Some("101"), None).is_err());
    assert!(ProbeConfig::from_values(Some("1"), Some("target"), None, Some("999")).is_err());
    assert!(ProbeConfig::from_values(Some("1"), Some("target"), None, Some("300001")).is_err());

    let config = ProbeConfig::from_values(Some("1"), Some(" target "), None, None).unwrap();
    assert_eq!(config.target_id, "target");
    assert_eq!(config.iterations, DEFAULT_ITERATIONS);
    assert_eq!(config.deadline, Duration::from_millis(DEFAULT_DEADLINE_MS));
}

#[test]
fn aggregation_uses_nearest_rank_percentiles_without_private_values() {
    let mut accumulator = ProbeAccumulator {
        iterations_requested: 4,
        iterations_completed: 4,
        success_count: 3,
        failure_count: 1,
        ..ProbeAccumulator::default()
    };
    accumulator
        .stage_samples
        .insert(TransportStage::TargetList, vec![40, 10, 30, 20]);
    accumulator
        .failure_stage_counts
        .insert("websocket_connect", 1);
    accumulator
        .stale_target_diagnosis_counts
        .insert("endpoint_changed", 1);

    let summary = accumulator.summary();
    assert_eq!(
        summary.stage_latency_ms["target_list"],
        StageLatency {
            sample_count: 4,
            p50: Some(20),
            p95: Some(40),
        }
    );
    assert_eq!(StageLatency::from_samples(&[]).p50, None);
    let encoded = serde_json::to_string(&summary).unwrap();
    assert!(!encoded.contains("target-id"));
    assert!(!encoded.contains("ws://"));
    assert!(!encoded.contains("Runtime.evaluate"));
}

#[test]
fn stale_target_diagnosis_covers_all_aggregate_labels() {
    let failed = target("target-a", "ws://private/old");
    let unchanged = target("target-a", "ws://private/old");
    let changed_endpoint = target("target-a", "ws://private/new");
    let duplicate = target("target-a", "ws://private/other");

    assert_eq!(
        diagnose_stale_target_from_targets(&failed, "target-a", &Ok(vec![unchanged.clone()]),),
        StaleTargetDiagnosis::Unchanged
    );
    assert_eq!(
        diagnose_stale_target_from_targets(&failed, "target-a", &Ok(vec![changed_endpoint])),
        StaleTargetDiagnosis::EndpointChanged
    );
    assert_eq!(
        diagnose_stale_target_from_targets(&failed, "target-a", &Ok(vec![unchanged, duplicate]),),
        StaleTargetDiagnosis::SelectionChangedOrAmbiguous
    );
    assert_eq!(
        diagnose_stale_target_from_targets(&failed, "target-a", &Ok(vec![])),
        StaleTargetDiagnosis::SelectionMissing
    );
    assert_eq!(
        diagnose_stale_target_from_targets(
            &failed,
            "target-a",
            &Err(AppError::new(ErrorKind::Connection, "unavailable")),
        ),
        StaleTargetDiagnosis::Unavailable
    );

    let labels = [
        StaleTargetDiagnosis::Unchanged,
        StaleTargetDiagnosis::EndpointChanged,
        StaleTargetDiagnosis::SelectionMissing,
        StaleTargetDiagnosis::SelectionChangedOrAmbiguous,
        StaleTargetDiagnosis::Unavailable,
    ]
    .map(StaleTargetDiagnosis::as_str);
    let encoded = serde_json::to_string(&labels).unwrap();
    assert!(!encoded.contains("target-a"));
    assert!(!encoded.contains("ws://"));
}

#[test]
fn setup_failure_preserves_iteration_count_invariant() {
    let summary = setup_failure_summary(10);
    assert_eq!(summary.iterations_requested, 10);
    assert_eq!(summary.iterations_completed, 1);
    assert_eq!(summary.success_count + summary.failure_count, 1);
    assert_eq!(summary.failure_stage_counts["transport_unknown"], 1);
}

#[tokio::test]
async fn fresh_read_path_classifies_duplicate_exact_targets_as_ambiguous() {
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let body = serde_json::to_string(&vec![
        target("target-a", "ws://private/first"),
        target("target-a", "ws://private/second"),
    ])
    .unwrap();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut request = [0u8; 512];
        let _ = stream.read(&mut request).await;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).await.unwrap();
    });
    let observer = TransportObserver::default();
    let session = CdpHttpSession::new(&TransportConfig {
        host: address.ip().to_string(),
        port: address.port(),
        target_id: Some("target-a".to_string()),
    })
    .unwrap()
    .with_observer(observer.clone());
    let failed = target("target-a", "ws://private/old");

    let diagnosis = diagnose_after_connect_failure(
        &session,
        &observer,
        &failed,
        "target-a",
        Instant::now() + Duration::from_secs(2),
    )
    .await;

    assert_eq!(diagnosis, StaleTargetDiagnosis::SelectionChangedOrAmbiguous);
    let samples = observer.take_samples();
    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].stage, TransportStage::TargetList);
    server.await.unwrap();
}

#[tokio::test]
async fn one_absolute_deadline_stops_a_stalled_probe() {
    use tokio::{
        io::AsyncReadExt,
        net::TcpListener,
        time::{Duration, sleep},
    };

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut request = [0u8; 256];
        let _ = stream.read(&mut request).await;
        sleep(Duration::from_secs(1)).await;
    });
    let config = ProbeConfig {
        target_id: "target".to_string(),
        iterations: 10,
        deadline: Duration::from_millis(250),
    };
    let transport = TransportConfig {
        host: address.ip().to_string(),
        port: address.port(),
        target_id: Some("target".to_string()),
    };

    let summary = run_probe_with_transport(config, transport).await;
    assert!(summary.deadline_reached);
    assert_eq!(summary.iterations_completed, 1);
    assert_eq!(summary.failure_count, 1);
    assert_eq!(summary.failure_stage_counts["target_list"], 1);
    assert!(summary.stage_latency_ms["target_list"].p95.unwrap() < 750);
    server.abort();
}

#[tokio::test]
#[ignore = "requires explicit owner-approved TradingView Desktop transport measurement"]
async fn live_transport_measurement() {
    let config = ProbeConfig::from_env().unwrap_or_else(|message| panic!("{message}"));
    let summary = run_probe(config).await;
    println!(
        "{}",
        serde_json::to_string_pretty(&summary)
            .expect("transport measurement summary should serialize")
    );
}
