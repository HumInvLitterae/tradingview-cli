//! Bounded chart observation lifecycle; specification lookup never starts it.

use serde_json::{Value, json};

use crate::ops::StreamKind;

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    if path != ["observe", "chart"] {
        return None;
    }
    Some(json!({
        "source": "desktop_chart_stream",
        "output_contract": "observe_chart.v1",
        "requires": {"desktop": true, "authentication": null},
        "effects": {"chart_mutation": false, "activates_tab": false, "captures_screenshot": false, "local_file_write": false},
        "constraints": {
            "interval": {"minimum": 100, "default": StreamKind::Bars.default_interval_ms(), "unit": "milliseconds"},
            "duration_ms": {"minimum": 1, "optional": true},
            "max_events": {"minimum": 1, "optional": true, "counts": "emitted distinct samples only"},
            "heartbeat_ms": {"minimum": 100, "optional": true, "default": null},
            "termination": "first duration or sample-count bound reached; no bound means open-ended"
        },
        "discovery": [{"argument": "target-id", "argv": ["tv", "tab", "list"], "result_path": "data.tabs[].id"}],
        "output": {
            "format": "JSONL envelopes, not one JSON document",
            "stdout_events": ["readiness", "sample", "heartbeat", "summary"],
            "stderr": "sample errors as JSONL error envelopes; collection continues",
            "sample_time": "_ts is client epoch milliseconds; bar_time is the chart's bar timestamp"
        },
        "limits": [
            "Readiness is emitted before the sampling connection and does not guarantee later samples. Initial readiness/connection failure can end the run without summary.",
            "Reads the current last bar, including possible changes to an unclosed bar. It is polling, not a tick-complete or historical bar export; no closed-bar or realtime guarantee is supplied.",
            "Consecutive identical samples suppress output after excluding _ts/_event. max_events excludes readiness, errors, heartbeats and summary; max_events alone can wait indefinitely when data is unchanged or reads fail.",
            "Set duration_ms for a time-bounded observation. Its clock starts after readiness and connection, and is checked between operations; setup and a running read can extend total wall time. interval is scheduled after the previous read, not a fixed-rate guarantee.",
            "Heartbeats occur during output silence, including repeated sample failures. They demonstrate process progress, not successful market-data updates; monitor stderr as well as stdout.",
            "Summary reports emitted samples and heartbeats, not an error count or lossless coverage. A clean exit/summary can follow sample errors. Broken pipes or interruption can omit the summary.",
            "The command does not switch symbols, lock the chart against user changes or restore chart state. Inspect each sample's symbol/resolution before combining observations; changed chart context can produce a new sample.",
            "The existing last-bar reader defaults falsy/missing volume to zero; do not treat that as independently verified zero trading volume. No scanner/MCP/other-target fallback is used."
        ],
        "examples": [["tv", "--target-id", "<target_id>", "observe", "chart", "--duration-ms", "5000", "--max-events", "10", "--heartbeat-ms", "1000"]]
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cli::Cli, ops::StreamRequest};
    use clap::Parser;

    #[test]
    fn observation_example_and_controls_match_execution() {
        let spec = describe(&["observe", "chart"]).unwrap();
        let argv = spec["examples"][0].as_array().unwrap();
        assert!(Cli::try_parse_from(argv.iter().map(|v| v.as_str().unwrap())).is_ok());
        let min = spec["constraints"]["interval"]["minimum"].as_u64().unwrap();
        assert!(
            StreamRequest::with_controls(
                StreamKind::Bars,
                Some(min),
                None,
                Some(1),
                Some(1),
                Some(min)
            )
            .is_ok()
        );
        assert!(
            StreamRequest::with_controls(StreamKind::Bars, Some(min - 1), None, None, None, None)
                .is_err()
        );
        assert!(
            StreamRequest::with_controls(StreamKind::Bars, None, None, Some(0), None, None)
                .is_err()
        );
        assert!(
            StreamRequest::with_controls(StreamKind::Bars, None, None, None, Some(0), None)
                .is_err()
        );
        assert!(
            StreamRequest::with_controls(StreamKind::Bars, None, None, None, None, Some(min - 1))
                .is_err()
        );
    }
}
