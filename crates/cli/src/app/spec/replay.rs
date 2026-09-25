//! Replay semantics are selected-chart practice operations, not market orders.

use serde_json::{Value, json};
use tradingview_model::replay::{
    MAX_REPLAY_LOG_STEPS, REPLAY_TRADE_ACTIONS, VALID_AUTOPLAY_DELAYS,
};

use super::super::replay_log::{
    DEFAULT_REPLAY_LOG_OHLCV_COUNT, MAX_REPLAY_LOG_OHLCV_COUNT, REPLAY_STEP_LOG_CONTRACT_VERSION,
};

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    let ["replay", action] = path else {
        return None;
    };
    let mut result = json!({
        "source": "internal_api",
        "requires": {"desktop": true, "authentication": null},
        "effects": {"replay_state_mutation": *action != "status", "broker_order": false},
        "discovery": [{"argument": "target-id", "argv": ["tv", "tab", "list"],
            "result_path": "data.tabs[].id"}],
        "readback": {"argv": ["tv", "--target-id", "<target_id>", "replay", "status"]},
        "output_contract": null,
        "output_format": "json",
        "constraints": {},
        "limits": [
            "Select the requested Desktop chart; no MCP or historical-bar fallback is used.",
            "Replay trades are practice actions, not broker orders or trading recommendations.",
            "On errors inspect current Replay state before another mutation; do not automatically repeat steps, trades or autoplay toggles.",
            "Preserve the requested end state; do not stop Replay or close a pre-existing practice position as automatic cleanup."
        ]
    });
    let example = match *action {
        "start" => {
            result["constraints"]["date"] = json!({"format": "YYYY-MM-DD", "trimmed": true,
                "calendar_date_required": true, "omitted": "select_first_available_date"});
            result["effects"]["shows_replay_toolbar"] = json!(true);
            result["limits"].as_array_mut().unwrap().push(json!(
                "Date availability depends on the selected chart/timeframe. Startup failure can attempt stopReplay cleanup; failure is not proof of unchanged state."
            ));
            json!(["tv", "replay", "start", "--date", "2024-01-15"])
        }
        "step" => {
            result["requires"]["replay_started"] = json!(true);
            result["effects"]["requested_steps"] = json!(1);
            json!(["tv", "replay", "step"])
        }
        "stop" => {
            result["effects"]["stops_replay"] = json!(true);
            result["limits"].as_array_mut().unwrap().push(json!(
                "Already-stopped Replay is reported without starting a session."
            ));
            json!(["tv", "replay", "stop"])
        }
        "status" => json!(["tv", "replay", "status"]),
        "autoplay" => {
            result["requires"]["replay_started"] = json!(true);
            result["effects"]["toggles_autoplay"] = json!(true);
            let mut choices = vec![0];
            choices.extend(VALID_AUTOPLAY_DELAYS);
            result["constraints"]["speed"] = json!({"choices": choices, "unit": "milliseconds",
                "zero_or_omitted": "keep_delay_and_toggle", "positive": "set_delay_then_toggle"});
            result["limits"].as_array_mut().unwrap().push(json!(
                "This toggles, not sets, playback state. Zero is not a stop instruction."
            ));
            json!(["tv", "replay", "autoplay", "--speed", "1000"])
        }
        "trade" => {
            result["requires"]["replay_started"] = json!(true);
            result["constraints"]["action"] = json!({"choices": REPLAY_TRADE_ACTIONS});
            result["effects"]["practice_position_mutation"] = json!(true);
            json!(["tv", "replay", "trade", "buy"])
        }
        "log" => {
            result["requires"]["replay_started"] = json!(true);
            result["output_contract"] = json!(REPLAY_STEP_LOG_CONTRACT_VERSION);
            result["output_format"] = json!("jsonl");
            result["constraints"] = json!({
                "steps": {"minimum": 1, "maximum": MAX_REPLAY_LOG_STEPS},
                "ohlcv-count": {"minimum": 1, "maximum": MAX_REPLAY_LOG_OHLCV_COUNT,
                    "default": DEFAULT_REPLAY_LOG_OHLCV_COUNT, "requires_flag": "attach-ohlcv-summary"},
                "screenshot-output-dir": {"requires_flag": "attach-chart-screenshot",
                    "existing_destination_files": "rejected"},
                "attach-chart-screenshot": {"requires_argument": "screenshot-output-dir"}
            });
            result["effects"]["advances_replay"] = json!(true);
            result["effects"]["writes_files"] = Value::Null;
            result["variants"] = json!([
                {"when": {"flag_false": ["attach_chart_screenshot"]}, "effects": {"writes_files": false}},
                {"when": {"flag_true": ["attach_chart_screenshot"]}, "effects": {"writes_files": true}}
            ]);
            result["limits"].as_array_mut().unwrap().extend([
                json!("Requires a started session; does not start or stop Replay. Read readiness, step and final summary events for actual progress and end state."),
                json!("Screenshots use replay-step-0001.png and increasing indices without overwrite. Setup can create the output directory before session readiness is established."),
                json!("Attachments follow successful steps. Attachment failure is reported separately and never repeats the step; logging is not historical dataset export.")
            ]);
            json!(["tv", "replay", "log", "--steps", "3"])
        }
        _ => return None,
    };
    result["examples"] = json!([example]);
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;
    use tradingview_model::replay::{
        validate_replay_autoplay_speed, validate_replay_log_steps, validate_replay_trade_action,
    };

    #[test]
    fn examples_and_choices_match_execution() {
        for action in [
            "start", "step", "stop", "status", "autoplay", "trade", "log",
        ] {
            for example in describe(&["replay", action]).unwrap()["examples"]
                .as_array()
                .unwrap()
            {
                assert!(
                    Cli::try_parse_from(
                        example
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|v| v.as_str().unwrap())
                    )
                    .is_ok()
                );
            }
        }
        let autoplay = describe(&["replay", "autoplay"]).unwrap();
        for speed in autoplay["constraints"]["speed"]["choices"]
            .as_array()
            .unwrap()
        {
            assert!(validate_replay_autoplay_speed(speed.as_u64().unwrap()).is_ok());
        }
        assert!(validate_replay_autoplay_speed(1).is_err());
        let trade = describe(&["replay", "trade"]).unwrap();
        for action in trade["constraints"]["action"]["choices"]
            .as_array()
            .unwrap()
        {
            assert!(validate_replay_trade_action(action.as_str().unwrap()).is_ok());
        }
        let max = describe(&["replay", "log"]).unwrap()["constraints"]["steps"]["maximum"]
            .as_u64()
            .unwrap();
        assert!(validate_replay_log_steps(max).is_ok());
        assert!(validate_replay_log_steps(max + 1).is_err());
        assert!(validate_replay_log_steps(0).is_err());
    }

    #[test]
    fn reads_toggles_and_conditional_files_are_distinct() {
        assert_eq!(
            describe(&["replay", "status"]).unwrap()["effects"]["replay_state_mutation"],
            false
        );
        assert_eq!(
            describe(&["replay", "autoplay"]).unwrap()["effects"]["toggles_autoplay"],
            true
        );
        let log = describe(&["replay", "log"]).unwrap();
        assert_eq!(log["output_format"], "jsonl");
        assert!(log["effects"]["writes_files"].is_null());
        assert_eq!(log["variants"][1]["effects"]["writes_files"], true);
    }
}
