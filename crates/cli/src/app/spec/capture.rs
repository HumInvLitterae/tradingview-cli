//! Selected-chart data and capture effects. No Desktop or filesystem access.

use serde_json::{Value, json};

use crate::ops::{DEFAULT_OHLCV_COUNT, MAX_OHLCV_COUNT};

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    let mut result = json!({
        "source": "chart_api",
        "requires": {"desktop": true, "authentication": null},
        "effects": {"chart_mutation": false, "local_file_write": false},
        "output_contract": null,
        "constraints": {},
        "discovery": [{
            "argument": "target-id",
            "argv": ["tv", "tab", "list"],
            "result_path": "data.tabs[].id"
        }],
        "limits": [
            "Use the intended explicit Desktop target; no official MCP or Desktop-free bars fallback is implied.",
            "Returned chart data is not proof of realtime access, entitlement or complete requested history."
        ]
    });
    let example = match path {
        ["ohlcv"] => {
            result["constraints"]["count"] = json!({
                "default": DEFAULT_OHLCV_COUNT,
                "clamped_minimum": 1,
                "clamped_maximum": MAX_OHLCV_COUNT
            });
            result["variants"] = summary_variants();
            result["limits"].as_array_mut().unwrap().extend([
                json!("Reads recent loaded chart bars, not only bars inside the viewport. count is clamped, including zero to one; fewer bars can be returned."),
                json!("Summary is a derived aggregate, not a lossless bar export. Missing numeric fields can be skipped or defaulted by the existing summarizer; inspect raw bars when values matter."),
                json!("Unavailable bars produce structured readiness errors. Read state on the same target before retrying.")
            ]);
            json!(["tv", "ohlcv", "--count", "10"])
        }
        ["export", "chart-bars"] => {
            result["source"] = json!("selected_chart_cdp");
            result["output_contract"] = json!("export_chart_bars.v1");
            result["effects"] = json!({
                "chart_mutation": true,
                "may_request_older_history": true,
                "restores_viewport": false,
                "local_file_write": false
            });
            result["constraints"] = json!({
                "from": {"finite": true, "unit": "Unix seconds"},
                "to": {"finite": true, "unit": "Unix seconds"},
                "ordering": "from < to",
                "count": {"minimum": 1, "maximum": MAX_OHLCV_COUNT, "default": MAX_OHLCV_COUNT}
            });
            result["variants"] = summary_variants();
            result["readback"] = json!({"argv": ["tv", "--target-id", "<target_id>", "range"]});
            result["limits"].as_array_mut().unwrap().extend([
                json!("Moves the selected chart viewport, then reads recent loaded bars. This is not a symbol-targeted historical query or a range-filtered bars export."),
                json!("Inspect range_operation, returned_bars_range and selected_chart_range_match separately from envelope success. Requested viewport and returned bars can differ."),
                json!("Prints a JSON envelope to stdout without writing a file. Summary omits last_5_bars. A later read failure does not undo the earlier viewport change.")
            ]);
            json!([
                "tv",
                "export",
                "chart-bars",
                "--from",
                "1704067200",
                "--to",
                "1704153600"
            ])
        }
        ["scroll"] => {
            result["effects"]["chart_mutation"] = json!(true);
            result["constraints"]["date"] = json!({
                "nonblank": true,
                "parsing": "digits-only Unix seconds, otherwise Desktop JavaScript Date parsing"
            });
            result["readback"] = json!({"argv": ["tv", "--target-id", "<target_id>", "range"]});
            result["limits"].as_array_mut().unwrap().extend([
                json!("Zooms within already loaded bars using an approximate 50-bar time window. It does not page older history; month length and session gaps are approximations."),
                json!("centered_on and window describe the request, not verified final viewport bounds. Prefer ISO dates or Unix seconds and read range afterward; the prior viewport is not restored.")
            ]);
            json!(["tv", "scroll", "2024-01-15"])
        }
        ["screenshot"] => {
            result["source"] = json!("desktop_screenshot");
            result["effects"] = json!({
                "chart_mutation": false,
                "local_file_write": true,
                "creates_parent_directories": true,
                "overwrites_existing_file": true
            });
            result["constraints"] = json!({
                "region": {"choices": ["full", "chart", "strategy"], "default": "full", "case_sensitive": true},
                "output": {"nonblank": true},
                "wait_timeout_ms": {
                    "requires": "wait_for_render",
                    "minimum": 500,
                    "maximum": 30000,
                    "default_when_waiting": 5000
                }
            });
            result["variants"] = json!([
                {
                    "when": {"flag_true": ["wait_for_render"]},
                    "operation": "wait_for_stable_render_then_capture"
                },
                {
                    "when": {"flag_false": ["wait_for_render"]},
                    "operation": "capture_without_render_wait"
                }
            ]);
            result["limits"].as_array_mut().unwrap().extend([
                json!("Captures the selected page or visible chart/Strategy Tester region; it does not open Strategy Tester. full means the page capture, not the whole operating-system screen."),
                json!("Chart/strategy capture can fall back from CDP clipping to cropping a full-page capture, retaining the requested region; it does not return an uncropped image as that region."),
                json!("Render waiting checks stable chart/region observations, not every pixel or data freshness. Wait failure occurs before capture/write."),
                json!("Ordinary screenshot overwrites its output and is different from Replay's no-overwrite attachment path. A filesystem write failure is not an atomic rollback guarantee.")
            ]);
            json!([
                "tv",
                "screenshot",
                "--region",
                "chart",
                "--output",
                "chart.png"
            ])
        }
        _ => return None,
    };
    result["examples"] = json!([example]);
    Some(result)
}

fn summary_variants() -> Value {
    json!([
        {"when": {"flag_true": ["summary"]}, "output_mode": "summary"},
        {"when": {"flag_false": ["summary"]}, "output_mode": "bars"}
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cli::Cli, ops};
    use clap::Parser;

    #[test]
    fn examples_and_export_wait_bounds_match_execution() {
        for path in [
            vec!["ohlcv"],
            vec!["export", "chart-bars"],
            vec!["scroll"],
            vec!["screenshot"],
        ] {
            let detail = describe(&path).unwrap();
            let argv = detail["examples"][0].as_array().unwrap();
            assert!(Cli::try_parse_from(argv.iter().map(|v| v.as_str().unwrap())).is_ok());
        }

        assert!(ops::validate_export_chart_bars_request(1.0, 2.0, Some(MAX_OHLCV_COUNT)).is_ok());
        assert!(
            ops::validate_export_chart_bars_request(1.0, 2.0, Some(MAX_OHLCV_COUNT + 1)).is_err()
        );
        assert!(ops::validate_export_chart_bars_request(1.0, 2.0, Some(0)).is_err());
        assert!(ops::validate_export_chart_bars_request(2.0, 1.0, None).is_err());
        assert!(ops::validate_export_chart_bars_request(f64::NAN, 2.0, None).is_err());

        let detail = describe(&["screenshot"]).unwrap();
        let wait = &detail["constraints"]["wait_timeout_ms"];
        let min = wait["minimum"].as_u64().unwrap();
        let max = wait["maximum"].as_u64().unwrap();
        assert!(ops::validate_screenshot_render_wait(true, Some(min)).is_ok());
        assert!(ops::validate_screenshot_render_wait(true, Some(max)).is_ok());
        assert!(ops::validate_screenshot_render_wait(true, Some(min - 1)).is_err());
        assert!(ops::validate_screenshot_render_wait(true, Some(max + 1)).is_err());
        assert!(ops::validate_screenshot_render_wait(false, Some(min)).is_err());
    }
}
