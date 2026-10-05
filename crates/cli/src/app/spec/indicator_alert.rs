//! Indicator alerts preserve legacy calls and support explicit compiled preflight.

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
            "dry_run": {"default": false},
            "verify": {"default": false, "meaning": "verify saved compiled condition and required native chart inputs"},
            "study_id": {"trimmed": true, "nonblank": true, "case_sensitive": true, "implies": "verify"}
        },
        "variants": [
            {"when": {"present": ["dry-run"]}, "source": "indicator_alert_dry_run", "effects": {"account_mutation": false}},
            {"when": {"absent": ["dry-run"]}, "source": "indicator_alert_api", "effects": {"account_mutation": true}}
        ],
        "discovery": [
            {"argument": "target-id", "argv": ["tv", "tab", "list"], "result_path": "data.tabs[].id"},
            {"argument": "study-id", "argv": ["tv", "state"], "result_path": "data.studies[].id"},
            {"argument": "condition selector", "argv": ["tv", "pine", "alertconditions", "--file", "<source.pine>"], "meaning": "best-effort local candidates, not compiled identities"}
        ],
        "limits": [
            "Requires supplied Pine source and a uniquely matching saved account script with an explicit ID and version. Both modes read that exact saved revision and compare locally, normalizing CRLF/LF/lone CR only. Source is not saved or submitted for compilation. Without verify/study-id, plot IDs remain best-effort candidates.",
            "Source mismatch returns validation with phase saved_source_verification and reason source_mismatch. Unavailable identity or source returns internal_api_unavailable with reason saved_identity_unavailable or saved_source_unavailable. Spaces, comments, BOM and terminal-newline differences are mismatches; no source repair, retry or alternate source is used.",
            "The full local source is not uploaded. Execution sends derived condition/feature metadata, saved script ID/version and study input values through the Desktop session. It does not add an indicator, change chart inputs or switch the symbol/timeframe.",
            "Legacy dry-run without verify/study-id connects and reads the saved-script catalog and source. would_create/mutation_supported true means a preview was assembled after a source match, not provider acceptance. Missing ID/version, unavailable source or a mismatch stops both modes before alert listing or creation. Preview does not read study inputs, validate chart defaults or exercise creation/readback.",
            "Legacy execution without verify/study-id uses the first chart study matching the requested/saved name or title, without unique entity-ID selection or verification of that chart study's source version. Native declarations and returned values must expose unique recognized IDs. User input IDs such as in_0 are preserved; missing/undeclared inputs, duplicate IDs, unavailable metadata or absent values fail before alert listing/creation with phase study_input_metadata_unavailable and created:false. Defaults do not fill gaps.",
            "System text/pineId/pineVersion/pineFeatures/__fast_calc/__profile entries are excluded from user-input assignment; generated base metadata is unchanged. input_metadata.input_count counts verified user inputs. Legacy completeness is relative to native declarations, not saved-version or compiled-condition identity. Legacy dry-run does not check these inputs.",
            "Legacy input requirement detection is a textual search for input. or input(. If inputs are detected and no matching study is available, execution fails; otherwise base metadata can be used without a study. Comments and formatting can affect this heuristic.",
            "Verify and study-id add a read of the exact saved revision's compiled metadata through the existing Desktop session. ID/version, selected alertcondition plot/type and known local title must agree. This internal service has no stability guarantee; missing or malformed evidence fails without legacy fallback. Local source and raw compiled payloads are not uploaded or returned.",
            "Verified calls with zero compiled user inputs need no chart study unless study-id was supplied. With inputs, automatic selection requires exactly one native saved-ID/version match; study-id selects that exact entity and implies verify. Native identity representations and compiled/native input declarations must agree. Values are copied by native ID before account requests; saved defaults never substitute for current values.",
            "Verified dry-run performs the same preflight as verified creation, resolves symbol/resolution, and stops before all alert-list/create calls. would_create is still not provider acceptance. Only verified successes add verification with condition_source saved_compilation, input_source none or active_chart_study, input_count, and study null or entity_id/selection explicit or unique. No-flag JSON remains unchanged.",
            "Compiled preflight reports phase compiled_condition_verification with reason saved_revision_mismatch, condition_mismatch or compiled_metadata_unavailable. Study selection reports study_identity_verification with study_not_found, saved_revision_mismatch, no_matching_study, ambiguous_study or study_metadata_unavailable. Mismatches are validation errors; unavailable evidence is internal_api_unavailable. Input failures retain study_input_metadata_unavailable. These returned preflight errors have created:false.",
            "Symbol/resolution overrides do not change the chart; input values still come from its matching study. Resolution falls back to 1 and currency to USD when absent. Saved script version has no fallback. No cross-symbol/currency compatibility proof is supplied.",
            "The request uses dividends adjustment, alert_cond/on_bar_close, about 30-day expiration, auto_deactivate false and notifications off (popup/email/SMS/mobile false, webhook null). These differ from legacy price-alert defaults.",
            "Readback excludes previously seen IDs and checks alert_cond type, condition ID and message; symbol is checked only if reported. It does not independently verify saved script version, complete inputs, resolution, expiry, notification delivery or every field.",
            "Creation POST request/response failures, failed readback, and unavailable/malformed creation-evaluation results carry created:null and creation_outcome:unknown in error details. HTTP/provider failure alone does not prove non-creation. Returned chart/input/list preflight failures retain created:false; saved-source errors and confirmed successful output keep their shapes.",
            "Verified preview evaluation failures retain created:false with no unknown creation outcome because preview cannot dispatch. No DOM or MCP fallback follows failed creation/readback. A failed post-check can follow a completed write; inspect the account before retrying. Keep script/account metadata and messages private. Official MCP simple-price alerts do not replace Pine alertcondition logic."
        ],
        "examples": [
            ["tv", "--target-id", "<target_id>", "alert", "create-indicator", "--script", "Example indicator", "--file", "<source.pine>", "--condition-title", "Example condition", "--dry-run"],
            ["tv", "--target-id", "<target_id>", "alert", "create-indicator", "--script", "Example indicator", "--file", "<source.pine>", "--condition-title", "Example condition", "--verify", "--dry-run"],
            ["tv", "--target-id", "<target_id>", "alert", "create-indicator", "--script", "Example indicator", "--file", "<source.pine>", "--condition-title", "Example condition", "--study-id", "<entity_id>", "--dry-run"]
        ]
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
        for example in spec["examples"].as_array().unwrap() {
            let argv = example.as_array().unwrap();
            assert!(Cli::try_parse_from(argv.iter().map(|v| v.as_str().unwrap())).is_ok());
        }
    }
}
