//! Primary chart streams: periodic samples rather than source-interchangeable ticks.

use serde_json::{Value, json};

use crate::ops::StreamKind;

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    let ["stream", action @ ("values" | "quote" | "bars")] = path else {
        return None;
    };
    let kind = match *action {
        "values" => StreamKind::Values,
        "quote" => StreamKind::Quote,
        _ => StreamKind::Bars,
    };
    let mut result = json!({
        "source": "desktop_chart_stream",
        "output_contract": "stream.v1",
        "requires": {"desktop": true, "authentication": null},
        "effects": {"chart_mutation": false, "local_file_write": false, "activates_tab": false},
        "constraints": super::observe::controls(kind),
        "discovery": [{"argument": "target-id", "argv": ["tv", "tab", "list"], "result_path": "data.tabs[].id"}],
        "output": {
            "format": "JSONL envelopes",
            "stdout_events": ["sample", "heartbeat", "summary"],
            "stderr": "sample error envelopes; collection continues",
            "readiness_event": false
        },
        "limits": [
            "Uses the current Desktop chart, with no symbol argument, chart switching, automatic reconnect or provider fallback. External chart changes are not prevented; inspect each sample's context.",
            "No initial readiness event is emitted, unlike observe chart. The observation clock starts after connection; setup and in-flight reads can extend wall time beyond duration_ms.",
            "Consecutive equal samples suppress output after ignoring _ts/_event. max_events counts emitted samples only, not polls, errors or heartbeats; include duration_ms to bound unchanged/erroring observations.",
            "Polling is scheduled after each read and is not a fixed-rate or tick-complete feed. Heartbeats report output silence/liveness, not successful market-data reads.",
            "Monitor stderr as well as stdout. Summary has sample/heartbeat counts but no error count; normal completion does not prove error-free coverage. Connection failure, broken pipe or interruption can omit summary.",
            "_ts is client epoch milliseconds, not provider freshness or bar-close evidence. This operation does not validate realtime access or historical completeness."
        ],
        "examples": [["tv", "--target-id", "<target_id>", "stream", action, "--duration-ms", "5000", "--heartbeat-ms", "1000"]]
    });
    if *action == "values" {
        result["limits"].as_array_mut().unwrap().extend([
            json!("Reads numeric properties from visible studies' internal last-bar data, not values' formatted data-window strings. Explicitly hidden studies are skipped when visibility is readable; unknown visibility is not proof of visible state."),
            json!("Unavailable studies, nonnumeric entries and per-study failures can be omitted. Empty studies is not a zero-valued indicator result. Neither per-study timestamps nor closed-bar identity are guaranteed."),
            json!("Same-name rows remain separate with optional entity_id, short_name, study_kind, compact inputs and visible. Identity/input/visibility changes participate in deduplication; no entity-ID or name filter option is accepted."),
            json!("Samples carry symbol but not an explicit timeframe field. Verify chart state before interpreting values across external timeframe changes; do not assume stream values and values are interchangeable.")
        ]);
    } else {
        result["limits"].as_array_mut().unwrap().extend([
            json!("Reads current main-series last-bar OHLCV, not scanner quote fields, Desktop quote-data or official MCP. It can update within an unfinished bar and does not provide every market tick."),
            json!("Falsy/missing volume defaults to zero in the existing reader; zero is not independent evidence of no trading.")
        ]);
        let timestamp_limit = if *action == "quote" {
            "Bar timestamp is time. No resolution/bar_index or scanner-style extended_hours contract is included; the command name does not make this an exchange tick stream."
        } else {
            "Bar timestamp is bar_time, with resolution and bar_index. It is a current-bar observation, not the historical bars.v1 endpoint or a replay of all bars between polls."
        };
        result["limits"]
            .as_array_mut()
            .unwrap()
            .push(json!(timestamp_limit));
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cli::Cli, ops::StreamRequest};
    use clap::Parser;

    #[test]
    fn primary_stream_examples_and_defaults_match_requests() {
        for (action, kind) in [
            ("quote", StreamKind::Quote),
            ("bars", StreamKind::Bars),
            ("values", StreamKind::Values),
        ] {
            let spec = describe(&["stream", action]).unwrap();
            let argv = spec["examples"][0].as_array().unwrap();
            assert!(Cli::try_parse_from(argv.iter().map(|v| v.as_str().unwrap())).is_ok());
            let request = StreamRequest::new(kind, None, None).unwrap();
            assert_eq!(
                spec["constraints"]["interval"]["default"],
                request.interval_ms
            );
        }
    }
}
