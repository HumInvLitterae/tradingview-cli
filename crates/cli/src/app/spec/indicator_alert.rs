//! Pine alertcondition previews are not compilation or source-identity proofs.

use serde_json::{Value, json};

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    if path != ["alert", "create-indicator"] {
        return None;
    }
    Some(json!({
        "source": null, "output_contract": null,
        "requires": {"desktop": true, "authentication": null},
        "effects": {"provider_request": true, "account_mutation": null,
            "chart_mutation": false, "local_file_write": false, "local_source_read": true},
        "constraints": {
            "script": {"trimmed": true, "nonblank": true, "matching": "exact case-sensitive saved name or title; exactly one match"},
            "source": {"file": "UTF-8 file takes precedence", "otherwise": "nonterminal stdin", "nonblank": true},
            "selector": {"exactly_one": ["condition-title", "alert-cond-id"],
                "condition_title": "trimmed exact case-sensitive literal title; unique candidate",
                "alert_cond_id": "plot_<N>; must exactly match a locally inferred candidate"},
            "symbol_resolution": "optional trimmed strings; blanks use chart defaults; no MCP symbol/timeframe validation",
            "message": "nonblank trimmed override, else source candidate message, else (none)",
            "dry_run": {"default": false}
        },
        "variants": [
            {"when": {"present": ["dry-run"]}, "source": "indicator_alert_dry_run", "effects": {"account_mutation": false}},
            {"when": {"absent": ["dry-run"]}, "source": "indicator_alert_api", "effects": {"account_mutation": true}}
        ],
        "discovery": [
            {"argument": "target-id", "argv": ["tv", "tab", "list"], "result_path": "data.tabs[].id"},
            {"argument": "condition selector", "argv": ["tv", "pine", "alertconditions", "--file", "<source.pine>"], "meaning": "best-effort local candidates, not compiled identities"}
        ],
        "limits": [
            "Requires supplied Pine source and a uniquely matching saved account script. The local source is not compared with the saved version, saved to the account or compiled by this command. Plot IDs depend on preceding outputs and remain best-effort candidates.",
            "The full local source is not uploaded. Execution sends derived condition/feature metadata, saved script ID/version and study input values through the Desktop session. It does not add an indicator, change chart inputs or switch the symbol/timeframe.",
            "Dry-run still connects and fetches the saved-script catalog. would_create/mutation_supported true means a preview was assembled, not provider acceptance. A missing script ID can pass preview; execution rejects it. Preview does not read study inputs, validate chart defaults or exercise creation/readback.",
            "Execution uses the first chart study matching the requested/saved name or title, without unique entity-ID selection or source-version verification. Inputs are assigned in returned order to in_0, in_1, etc. A matching study can expose zero values; completeness is not verified.",
            "Input requirement detection is a textual search for input. or input(. If inputs are detected and no matching study is available, execution fails; otherwise base metadata can be used without a study. Comments and formatting can affect this heuristic.",
            "Symbol/resolution overrides do not change the chart; input values still come from its matching study. Resolution falls back to 1, saved script version to 1.0 and currency to USD when absent. No cross-symbol/currency compatibility proof is supplied.",
            "The request uses dividends adjustment, alert_cond/on_bar_close, about 30-day expiration, auto_deactivate false and notifications off (popup/email/SMS/mobile false, webhook null). These differ from legacy price-alert defaults.",
            "Readback excludes previously seen IDs and checks alert_cond type, condition ID and message; symbol is checked only if reported. It does not independently verify saved script version, complete inputs, resolution, expiry, notification delivery or every field.",
            "No DOM or MCP fallback follows failed creation/readback. A failed post-check can follow a completed write; inspect the account before retrying. Keep script/account metadata and messages private. Official MCP simple-price alerts do not replace Pine alertcondition logic."
        ],
        "examples": [["tv", "--target-id", "<target_id>", "alert", "create-indicator", "--script", "Example indicator", "--file", "<source.pine>", "--condition-title", "Example condition", "--dry-run"]]
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;

    #[test]
    fn indicator_alert_preview_example_parses_without_reading_source() {
        let spec = describe(&["alert", "create-indicator"]).unwrap();
        let argv = spec["examples"][0].as_array().unwrap();
        assert!(Cli::try_parse_from(argv.iter().map(|v| v.as_str().unwrap())).is_ok());
    }
}
