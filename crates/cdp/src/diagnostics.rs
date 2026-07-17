use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use serde::Serialize;
use serde_json::{Map, Value};
use tradingview_core::{AppError, ErrorKind};

#[cfg(test)]
use crate::Target;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum TransportStage {
    TargetList,
    TargetSelect,
    WebSocketConnect,
    MethodCall,
    EventWait,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StageOutcome {
    Success,
    Failure(ErrorKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StageSample {
    pub stage: TransportStage,
    pub elapsed_ms: u64,
    pub outcome: StageOutcome,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct TransportObserver {
    samples: Arc<Mutex<Vec<StageSample>>>,
}

impl TransportObserver {
    pub(crate) fn record<T>(
        &self,
        stage: TransportStage,
        elapsed: Duration,
        result: &Result<T, AppError>,
    ) {
        let outcome = match result {
            Ok(_) => StageOutcome::Success,
            Err(error) => StageOutcome::Failure(error.kind),
        };
        let sample = StageSample {
            stage,
            elapsed_ms: elapsed.as_millis().min(u128::from(u64::MAX)) as u64,
            outcome,
        };
        self.samples
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(sample);
    }

    #[cfg(test)]
    pub(crate) fn samples(&self) -> Vec<StageSample> {
        self.samples
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    #[cfg(test)]
    pub(crate) fn take_samples(&self) -> Vec<StageSample> {
        std::mem::take(
            &mut *self
                .samples
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PublicFailureStage {
    TargetList,
    TargetSelect,
    WebsocketConnect,
    MethodCall,
    EventWait,
    TransportUnknown,
}

impl From<Option<TransportStage>> for PublicFailureStage {
    fn from(stage: Option<TransportStage>) -> Self {
        match stage {
            Some(TransportStage::TargetList) => Self::TargetList,
            Some(TransportStage::TargetSelect) => Self::TargetSelect,
            Some(TransportStage::WebSocketConnect) => Self::WebsocketConnect,
            Some(TransportStage::MethodCall) => Self::MethodCall,
            Some(TransportStage::EventWait) => Self::EventWait,
            None => Self::TransportUnknown,
        }
    }
}

#[cfg(test)]
impl PublicFailureStage {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::TargetList => "target_list",
            Self::TargetSelect => "target_select",
            Self::WebsocketConnect => "websocket_connect",
            Self::MethodCall => "method_call",
            Self::EventWait => "event_wait",
            Self::TransportUnknown => "transport_unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[cfg(test)]
pub(crate) enum StaleTargetDiagnosis {
    Unchanged,
    EndpointChanged,
    SelectionMissing,
    SelectionChangedOrAmbiguous,
    Unavailable,
}

#[cfg(test)]
impl StaleTargetDiagnosis {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Unchanged => "unchanged",
            Self::EndpointChanged => "endpoint_changed",
            Self::SelectionMissing => "selection_missing",
            Self::SelectionChangedOrAmbiguous => "selection_changed_or_ambiguous",
            Self::Unavailable => "unavailable",
        }
    }
}

#[cfg(test)]
pub(crate) fn diagnose_stale_target_from_targets(
    failed_target: &Target,
    target_id: &str,
    fresh_result: &Result<Vec<Target>, AppError>,
) -> StaleTargetDiagnosis {
    match fresh_result {
        Ok(targets) => {
            let matches = targets
                .iter()
                .filter(|target| target.id == target_id)
                .collect::<Vec<_>>();
            match matches.as_slice() {
                [] => StaleTargetDiagnosis::SelectionMissing,
                [fresh_target]
                    if fresh_target.web_socket_debugger_url
                        != failed_target.web_socket_debugger_url =>
                {
                    StaleTargetDiagnosis::EndpointChanged
                }
                [_] => StaleTargetDiagnosis::Unchanged,
                _ => StaleTargetDiagnosis::SelectionChangedOrAmbiguous,
            }
        }
        Err(error) if error.kind == ErrorKind::TargetAmbiguous => {
            StaleTargetDiagnosis::SelectionChangedOrAmbiguous
        }
        Err(_) => StaleTargetDiagnosis::Unavailable,
    }
}

pub(crate) fn with_failure_stage(mut error: AppError, stage: PublicFailureStage) -> AppError {
    let mut details = match error.details.take() {
        Some(Value::Object(details)) => details,
        Some(_) => {
            let mut details = Map::new();
            details.insert("previous_details_omitted".to_string(), Value::Bool(true));
            details
        }
        None => Map::new(),
    };
    details.insert(
        "failure_stage".to_string(),
        serde_json::to_value(stage).expect("failure stage should serialize"),
    );
    error.details = Some(Value::Object(details));
    error
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn target(id: &str, endpoint: Option<&str>) -> Target {
        Target {
            id: id.to_string(),
            title: "test".to_string(),
            kind: "page".to_string(),
            url: "https://example.invalid/chart".to_string(),
            web_socket_debugger_url: endpoint.map(str::to_string),
        }
    }

    #[test]
    fn public_failure_stage_preserves_safe_object_details() {
        let error = with_failure_stage(
            AppError::new(ErrorKind::Timeout, "timed out").with_details(json!({
                "operation": "target list",
                "count": 2,
            })),
            PublicFailureStage::TargetList,
        );

        assert_eq!(error.kind, ErrorKind::Timeout);
        assert_eq!(error.message, "timed out");
        let details = error.details.unwrap();
        assert_eq!(details["operation"], "target list");
        assert_eq!(details["count"], 2);
        assert_eq!(details["failure_stage"], "target_list");
    }

    #[test]
    fn public_failure_stage_omits_non_object_details() {
        let error = with_failure_stage(
            AppError::new(ErrorKind::Connection, "failed")
                .with_details(json!("private-runtime-value")),
            PublicFailureStage::TransportUnknown,
        );

        let encoded = serde_json::to_string(&error.details).unwrap();
        assert!(!encoded.contains("private-runtime-value"));
        let details = error.details.unwrap();
        assert_eq!(details["previous_details_omitted"], true);
        assert_eq!(details["failure_stage"], "transport_unknown");
    }

    #[test]
    fn public_failure_stage_maps_every_internal_stage_and_unknown() {
        let cases = [
            (Some(TransportStage::TargetList), "target_list"),
            (Some(TransportStage::TargetSelect), "target_select"),
            (Some(TransportStage::WebSocketConnect), "websocket_connect"),
            (Some(TransportStage::MethodCall), "method_call"),
            (Some(TransportStage::EventWait), "event_wait"),
            (None, "transport_unknown"),
        ];

        for (stage, expected) in cases {
            assert_eq!(
                serde_json::to_value(PublicFailureStage::from(stage)).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn observer_records_saturating_milliseconds_and_outcome() {
        let observer = TransportObserver::default();
        observer.record(
            TransportStage::TargetSelect,
            Duration::from_millis(12),
            &Err::<(), _>(AppError::new(ErrorKind::TargetAmbiguous, "ambiguous")),
        );

        assert_eq!(
            observer.samples(),
            vec![StageSample {
                stage: TransportStage::TargetSelect,
                elapsed_ms: 12,
                outcome: StageOutcome::Failure(ErrorKind::TargetAmbiguous),
            }]
        );
    }

    #[test]
    fn stale_target_diagnosis_covers_all_public_safe_outcomes() {
        let failed = target("target-a", Some("ws://private/old"));

        assert_eq!(
            diagnose_stale_target_from_targets(
                &failed,
                "target-a",
                &Ok(vec![target("target-a", Some("ws://private/old"))]),
            ),
            StaleTargetDiagnosis::Unchanged
        );
        assert_eq!(
            diagnose_stale_target_from_targets(
                &failed,
                "target-a",
                &Ok(vec![target("target-a", Some("ws://private/new"))]),
            ),
            StaleTargetDiagnosis::EndpointChanged
        );
        assert_eq!(
            diagnose_stale_target_from_targets(
                &failed,
                "target-a",
                &Ok(vec![
                    target("target-a", Some("ws://private/first")),
                    target("target-a", Some("ws://private/second")),
                ]),
            ),
            StaleTargetDiagnosis::SelectionChangedOrAmbiguous
        );

        assert_eq!(
            diagnose_stale_target_from_targets(&failed, "target-a", &Ok(vec![])),
            StaleTargetDiagnosis::SelectionMissing
        );

        let ambiguous = with_failure_stage(
            AppError::new(ErrorKind::TargetAmbiguous, "ambiguous"),
            PublicFailureStage::TargetSelect,
        );
        assert_eq!(
            diagnose_stale_target_from_targets(&failed, "target-a", &Err(ambiguous)),
            StaleTargetDiagnosis::SelectionChangedOrAmbiguous
        );

        let unavailable = with_failure_stage(
            AppError::new(ErrorKind::Connection, "unavailable"),
            PublicFailureStage::TargetList,
        );
        assert_eq!(
            diagnose_stale_target_from_targets(&failed, "target-a", &Err(unavailable)),
            StaleTargetDiagnosis::Unavailable
        );

        let encoded = serde_json::to_string(&[
            StaleTargetDiagnosis::Unchanged,
            StaleTargetDiagnosis::EndpointChanged,
            StaleTargetDiagnosis::SelectionMissing,
            StaleTargetDiagnosis::SelectionChangedOrAmbiguous,
            StaleTargetDiagnosis::Unavailable,
        ])
        .unwrap();
        assert!(!encoded.contains("target-a"));
        assert!(!encoded.contains("ws://"));
    }
}
