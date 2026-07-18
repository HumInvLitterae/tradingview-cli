use std::{
    collections::BTreeMap,
    fs::File,
    io::Read,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use serde::Serialize;
use serde_json::Value;
use tempfile::NamedTempFile;

const LIVE_GATE: &str = "TV_LIVE_CONSECUTIVE_INVOCATION_RESILIENCE";
const TARGET_ENV: &str = "TV_LIVE_CONSECUTIVE_INVOCATION_TARGET_ID";
const INVOCATIONS_PER_COHORT: usize = 20;
const INVOCATION_TIMEOUT: Duration = Duration::from_secs(12);
const COHORT_TIMEOUT: Duration = Duration::from_secs(270);
const RUN_TIMEOUT: Duration = Duration::from_secs(1_800);

#[derive(Clone, Copy, Debug)]
enum ReadKind {
    Readiness,
    OhlcvSummary,
    Values,
}

impl ReadKind {
    fn args(self) -> &'static [&'static str] {
        match self {
            Self::Readiness => &["readiness"],
            Self::OhlcvSummary => &["ohlcv", "--summary", "--count", "20"],
            Self::Values => &["values"],
        }
    }
}

#[derive(Clone, Copy)]
struct Cohort {
    label: &'static str,
    explicit: bool,
    delay: Duration,
    reads: &'static [ReadKind],
}

const LIGHT: &[ReadKind] = &[ReadKind::Readiness];
const LARGE: &[ReadKind] = &[ReadKind::Values];
const MIXED: &[ReadKind] = &[
    ReadKind::Readiness,
    ReadKind::OhlcvSummary,
    ReadKind::Values,
];

const COHORTS: &[Cohort] = &[
    Cohort {
        label: "same_light_explicit_no_delay",
        explicit: true,
        delay: Duration::ZERO,
        reads: LIGHT,
    },
    Cohort {
        label: "same_light_heuristic_no_delay",
        explicit: false,
        delay: Duration::ZERO,
        reads: LIGHT,
    },
    Cohort {
        label: "same_large_explicit_fixed_delay",
        explicit: true,
        delay: Duration::from_millis(250),
        reads: LARGE,
    },
    Cohort {
        label: "same_large_heuristic_fixed_delay",
        explicit: false,
        delay: Duration::from_millis(250),
        reads: LARGE,
    },
    Cohort {
        label: "mixed_explicit_fixed_delay",
        explicit: true,
        delay: Duration::from_millis(250),
        reads: MIXED,
    },
    Cohort {
        label: "mixed_heuristic_fixed_delay",
        explicit: false,
        delay: Duration::from_millis(250),
        reads: MIXED,
    },
];

#[derive(Clone, Debug)]
struct InvocationResult {
    success: bool,
    failure_stage: Option<String>,
    ambiguous: bool,
    target_count: Option<u64>,
    latency_ms: u64,
}

#[derive(Default, Serialize)]
struct Summary {
    cohorts_requested: usize,
    cohorts_completed: usize,
    invocations_requested: usize,
    invocations_completed: usize,
    success_count: usize,
    failure_count: usize,
    failure_stage_counts: BTreeMap<String, usize>,
    ambiguity_count: usize,
    deadline_stop_count: usize,
    target_drift_count: usize,
    latency_p50_ms: u64,
    latency_p95_ms: u64,
    cohort_summaries: Vec<CohortSummary>,
    #[serde(skip)]
    latencies: Vec<u64>,
    #[serde(skip)]
    prior_target_count: Option<u64>,
}

#[derive(Serialize)]
struct CohortSummary {
    cohort: &'static str,
    invocations_requested: usize,
    invocations_completed: usize,
    success_count: usize,
    failure_count: usize,
    failure_stage_counts: BTreeMap<String, usize>,
    ambiguity_count: usize,
    deadline_stop_count: usize,
    target_drift_count: usize,
    latency_p50_ms: u64,
    latency_p95_ms: u64,
}

impl Summary {
    fn requested() -> Self {
        Self {
            cohorts_requested: COHORTS.len(),
            invocations_requested: COHORTS.len() * INVOCATIONS_PER_COHORT,
            ..Self::default()
        }
    }

    fn record(&mut self, result: InvocationResult) {
        self.invocations_completed += 1;
        self.latencies.push(result.latency_ms);
        if result.success {
            self.success_count += 1;
        } else {
            self.failure_count += 1;
        }
        if let Some(stage) = result.failure_stage {
            *self.failure_stage_counts.entry(stage).or_default() += 1;
        }
        self.ambiguity_count += usize::from(result.ambiguous);
        if let Some(count) = result.target_count {
            if self.prior_target_count.is_some_and(|prior| prior != count) {
                self.target_drift_count += 1;
            }
            self.prior_target_count = Some(count);
        }
    }

    fn finish(&mut self) {
        self.latencies.sort_unstable();
        self.latency_p50_ms = percentile(&self.latencies, 50);
        self.latency_p95_ms = percentile(&self.latencies, 95);
        assert_eq!(
            self.success_count + self.failure_count,
            self.invocations_completed
        );
        assert!(self.invocations_completed <= self.invocations_requested);
        assert!(self.cohorts_completed <= self.cohorts_requested);
    }
}

#[test]
#[ignore = "requires TradingView Desktop, an explicit disposable target, and owner approval"]
fn consecutive_invocation_resilience_live_matrix() {
    require_live_gate();
    let target_id = required_target_id();
    let tv = env!("CARGO_BIN_EXE_tv");
    let run_deadline = Instant::now() + RUN_TIMEOUT;
    let mut summary = Summary::requested();

    for cohort in COHORTS {
        let cohort_deadline = Instant::now() + COHORT_TIMEOUT;
        let mut completed = true;
        let mut cohort_summary = Summary {
            cohorts_requested: 1,
            invocations_requested: INVOCATIONS_PER_COHORT,
            ..Summary::default()
        };
        for index in 0..INVOCATIONS_PER_COHORT {
            if Instant::now() >= cohort_deadline || Instant::now() >= run_deadline {
                summary.deadline_stop_count += 1;
                cohort_summary.deadline_stop_count += 1;
                completed = false;
                break;
            }
            let read = cohort.reads[index % cohort.reads.len()];
            match run_tv(tv, cohort.explicit.then_some(target_id.as_str()), read) {
                Ok((result, malformed)) => {
                    summary.record(result.clone());
                    cohort_summary.record(result);
                    if malformed {
                        completed = false;
                        break;
                    }
                }
                Err(()) => {
                    summary.deadline_stop_count += 1;
                    cohort_summary.deadline_stop_count += 1;
                    completed = false;
                    break;
                }
            }
            if cohort.delay != Duration::ZERO && index + 1 < INVOCATIONS_PER_COHORT {
                thread::sleep(cohort.delay);
            }
        }
        if completed {
            summary.cohorts_completed += 1;
            cohort_summary.cohorts_completed = 1;
        }
        cohort_summary.finish();
        summary.cohort_summaries.push(CohortSummary {
            cohort: cohort.label,
            invocations_requested: cohort_summary.invocations_requested,
            invocations_completed: cohort_summary.invocations_completed,
            success_count: cohort_summary.success_count,
            failure_count: cohort_summary.failure_count,
            failure_stage_counts: cohort_summary.failure_stage_counts,
            ambiguity_count: cohort_summary.ambiguity_count,
            deadline_stop_count: cohort_summary.deadline_stop_count,
            target_drift_count: cohort_summary.target_drift_count,
            latency_p50_ms: cohort_summary.latency_p50_ms,
            latency_p95_ms: cohort_summary.latency_p95_ms,
        });
    }

    summary.finish();
    println!(
        "{}",
        serde_json::to_string(&summary).expect("aggregate summary should serialize")
    );
}

fn run_tv(
    tv: &str,
    target_id: Option<&str>,
    read: ReadKind,
) -> Result<(InvocationResult, bool), ()> {
    let stdout = NamedTempFile::new().expect("temporary stdout file should be created");
    let stderr = NamedTempFile::new().expect("temporary stderr file should be created");
    let mut command = Command::new(tv);
    if let Some(target_id) = target_id {
        command.args(["--target-id", target_id]);
    }
    command.args(read.args());
    command.stdout(Stdio::from(
        stdout.reopen().expect("stdout file should reopen"),
    ));
    command.stderr(Stdio::from(
        stderr.reopen().expect("stderr file should reopen"),
    ));
    let started = Instant::now();
    let mut child = command.spawn().expect("test-built tv binary should start");
    let status = loop {
        if let Some(status) = child.try_wait().expect("child status should be readable") {
            break status;
        }
        if started.elapsed() >= INVOCATION_TIMEOUT {
            let _ = child.kill();
            let _ = child.wait();
            return Err(());
        }
        thread::sleep(Duration::from_millis(10));
    };
    let bytes = if status.success() {
        read_file(stdout.as_file())
    } else {
        read_file(stderr.as_file())
    };
    let latency_ms = elapsed_ms(started.elapsed());
    let Some(value) = parse_envelope(&bytes) else {
        return Ok((
            InvocationResult {
                success: false,
                failure_stage: Some("transport_unknown".to_string()),
                ambiguous: false,
                target_count: None,
                latency_ms,
            },
            true,
        ));
    };
    Ok((
        classify_envelope(status.success(), &value, latency_ms),
        false,
    ))
}

fn read_file(file: &File) -> Vec<u8> {
    let mut file = file.try_clone().expect("temporary output should clone");
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .expect("temporary output should be readable");
    bytes
}

fn classify_envelope(success: bool, value: &Value, latency_ms: u64) -> InvocationResult {
    let envelope_success = value.get("success").and_then(Value::as_bool);
    assert_eq!(envelope_success, Some(success));
    let stage = value
        .pointer("/error/details/failure_stage")
        .and_then(Value::as_str)
        .map(normalize_failure_stage);
    let ambiguous = value
        .pointer("/error/kind")
        .and_then(Value::as_str)
        .is_some_and(|kind| kind == "target_ambiguous")
        || value
            .pointer("/data/target_selection")
            .and_then(Value::as_str)
            .is_some_and(|selection| selection == "ambiguous")
        || value
            .pointer("/data/desktop_readiness/target_selection")
            .and_then(Value::as_str)
            .is_some_and(|selection| selection == "ambiguous");
    let target_count = [
        "/data/cdp/target_count",
        "/data/desktop_readiness/target_count",
        "/data/target_count",
        "/error/details/target_count",
    ]
    .into_iter()
    .find_map(|pointer| value.pointer(pointer).and_then(Value::as_u64));
    InvocationResult {
        success,
        failure_stage: stage,
        ambiguous,
        target_count,
        latency_ms,
    }
}

fn parse_envelope(bytes: &[u8]) -> Option<Value> {
    serde_json::from_slice(bytes).ok()
}

fn normalize_failure_stage(stage: &str) -> String {
    match stage {
        "http_client" | "target_list" | "target_select" | "websocket_connect" | "method_call" => {
            stage.to_string()
        }
        _ => "transport_unknown".to_string(),
    }
}

fn percentile(values: &[u64], percentile: usize) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let index = (values.len() * percentile).div_ceil(100).saturating_sub(1);
    values[index]
}

fn elapsed_ms(duration: Duration) -> u64 {
    duration.as_millis().min(u128::from(u64::MAX)) as u64
}

fn require_live_gate() {
    assert_eq!(
        std::env::var(LIVE_GATE).ok().as_deref(),
        Some("1"),
        "consecutive invocation live matrix is gated"
    );
}

fn required_target_id() -> String {
    std::env::var(TARGET_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .expect("consecutive invocation live matrix requires an explicit target id")
}

#[test]
fn aggregate_counts_and_percentiles_are_deterministic() {
    let mut summary = Summary::requested();
    summary.record(InvocationResult {
        success: true,
        failure_stage: None,
        ambiguous: false,
        target_count: Some(2),
        latency_ms: 10,
    });
    summary.record(InvocationResult {
        success: false,
        failure_stage: Some("target_list".to_string()),
        ambiguous: true,
        target_count: Some(3),
        latency_ms: 30,
    });
    summary.cohorts_completed = 1;
    summary.finish();
    assert_eq!(summary.success_count, 1);
    assert_eq!(summary.failure_count, 1);
    assert_eq!(summary.failure_stage_counts["target_list"], 1);
    assert_eq!(summary.ambiguity_count, 1);
    assert_eq!(summary.target_drift_count, 1);
    assert_eq!(summary.latency_p50_ms, 10);
    assert_eq!(summary.latency_p95_ms, 30);
}

#[test]
fn envelope_classification_is_allowlisted() {
    let private = "private-target-value";
    let value = serde_json::json!({
        "success": false,
        "error": {
            "kind": "connection",
            "message": private,
            "details": {
                "failure_stage": "future_private_stage",
                "target_id": private,
                "url": "ws://private"
            }
        }
    });
    let result = classify_envelope(false, &value, 12);
    assert_eq!(result.failure_stage.as_deref(), Some("transport_unknown"));
    let serialized = serde_json::to_string(&Summary::requested()).unwrap();
    assert!(!serialized.contains(private));
    assert!(!serialized.contains("ws://"));
}

#[test]
fn malformed_output_is_rejected_without_retaining_it() {
    let private = b"not-json-private-target-value";
    assert!(parse_envelope(private).is_none());
    let result = InvocationResult {
        success: false,
        failure_stage: Some("transport_unknown".to_string()),
        ambiguous: false,
        target_count: None,
        latency_ms: 1,
    };
    let mut summary = Summary::requested();
    summary.record(result);
    summary.finish();
    let serialized = serde_json::to_string(&summary).unwrap();
    assert!(!serialized.contains("private-target-value"));
    assert_eq!(summary.failure_stage_counts["transport_unknown"], 1);
}

#[test]
fn cohort_matrix_is_exactly_bounded() {
    assert_eq!(COHORTS.len(), 6);
    assert_eq!(COHORTS.len() * INVOCATIONS_PER_COHORT, 120);
    assert_eq!(MIXED.len(), 3);
    assert!(COHORTS.iter().any(|cohort| cohort.explicit));
    assert!(COHORTS.iter().any(|cohort| !cohort.explicit));
    assert!(COHORTS.iter().any(|cohort| cohort.delay.is_zero()));
    assert!(
        COHORTS
            .iter()
            .any(|cohort| cohort.delay == Duration::from_millis(250))
    );
    let labels = COHORTS
        .iter()
        .map(|cohort| cohort.label)
        .collect::<Vec<_>>();
    assert_eq!(labels.len(), 6);
    assert_eq!(
        labels
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        6
    );
}
