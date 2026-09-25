//! Chart-study and strategy-report interpretation; lookup does not touch Desktop.

use serde_json::{Value, json};

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    let mut result = json!({
        "source": "internal_api",
        "requires": {"desktop": true, "authentication": null},
        "effects": {"chart_mutation": false, "opens_strategy_tester": false, "changes_study_visibility": false},
        "output_contract": null,
        "constraints": {},
        "discovery": [{
            "argument": "target-id",
            "argv": ["tv", "tab", "list"],
            "result_path": "data.tabs[].id"
        }],
        "limits": [
            "Read the intended explicit Desktop target; no MCP, scanner or another chart is used as fallback.",
            "These are observations of the current chart, not trade instructions or a reproducible backtest artifact."
        ]
    });
    if path == ["values"] {
        result["limits"].as_array_mut().unwrap().extend([
            json!("Returns formatted data-window values from readable studies, not numerical time series. No timestamp or closed-bar guarantee is attached to each value."),
            json!("Rows without readable named values can be omitted. Missing/empty output is not zero or proof that no studies exist. Repeated value titles can overwrite within a row."),
            json!("Same-name studies remain separate. Use entity_id, inputs, visible and study_kind when available; identity and compact inputs can be unknown or truncated. Hidden studies may still return readable values."),
            json!("Use state to inspect chart studies and data indicator with the same chart-local entity_id for input details. No per-study filter is accepted by values.")
        ]);
        result["examples"] = json!([["tv", "--target-id", "<target_id>", "values"]]);
        return Some(result);
    }
    let ["data", action @ ("strategy" | "trades" | "equity")] = path else {
        return None;
    };
    result["constraints"]["selection"] = json!({
        "explicit_entity_selector": false,
        "rule": "only candidate, otherwise only candidate with any report capability; unresolved ties are ambiguous"
    });
    result["limits"].as_array_mut().unwrap().extend([
        json!("Inspect strategy_context and data.error even on envelope success. not_found, strategy_hidden, report_not_ready, ambiguous and unknown are unavailable observations, not zero performance."),
        json!("A hidden selected strategy is unavailable. The reader does not select an arbitrary visible strategy, unhide one or open a panel. Suggested state changes require separate user intent."),
        json!("report_available means at least one report capability was detected, not that this specific reader has data. Selection and extraction are separate reads; state can change between them."),
        json!("strategy_context records chart selection, not independent identity verification of DOM fallback content. Compare the actual report before combining results; titles and panel_status can remain unknown.")
    ]);
    match *action {
        "strategy" => {
            result["limits"].as_array_mut().unwrap().push(json!(
                "Reads scalar performance fields, then may parse already rendered Strategy Report text. Inspect source (internal_api or dom_fallback); keys, units and formatted values depend on the exposed report. DOM values are not a canonical numeric schema."
            ));
        }
        "trades" => {
            result["constraints"]["max"] = json!({
                "default": 20,
                "clamped_minimum": 1,
                "clamped_maximum": 20
            });
            result["limits"].as_array_mut().unwrap().extend([
                json!("Returns the first available entries up to max, not guaranteed newest-first. Compare trade_count with total_trade_count when present; there is no pagination argument."),
                json!("The report-trades path normalizes entry/exit times as milliseconds. Alternative order/trade arrays copy scalar fields; DOM fallback contains only rendered rows and formatted text, possibly generic page rows. Do not assume identical row semantics or full history across paths.")
            ]);
        }
        "equity" => {
            result["limits"].as_array_mut().unwrap().extend([
                json!("Current extraction prefers _reportData.buyHold before equityData or strategy bars. source=internal_api alone does not identify which series was returned; data_points is not proof of a strategy equity curve."),
                json!("Rows can be index/value, provider-shaped objects or time/equity/drawdown. The bars path converts zero drawdown to null. Do not infer currency, timestamps or strategy returns from unlabeled values."),
                json!("If no series is available, equity_summary can contain performance metrics with data_points=0. Keep the curve unavailable; do not synthesize it from the summary. Independent series provenance is required before calculating strategy performance from this output.")
            ]);
        }
        _ => unreachable!(),
    }
    result["examples"] = json!([["tv", "--target-id", "<target_id>", "data", action]]);
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;

    #[test]
    fn analysis_examples_parse() {
        for path in [
            vec!["values"],
            vec!["data", "strategy"],
            vec!["data", "trades"],
            vec!["data", "equity"],
        ] {
            let detail = describe(&path).unwrap();
            let argv = detail["examples"][0].as_array().unwrap();
            assert!(Cli::try_parse_from(argv.iter().map(|v| v.as_str().unwrap())).is_ok());
        }
    }
}
