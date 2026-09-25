//! Pine source, Editor and remote-check boundaries. Specification lookup is offline.

use serde_json::{Value, json};

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    let ["pine", action] = path else {
        return None;
    };
    let source_input = matches!(*action, "set" | "analyze" | "alertconditions" | "check");
    let desktop = !matches!(*action, "analyze" | "alertconditions" | "check");
    let mut result = json!({
        "source": if desktop { "internal_api" } else { "local_source" },
        "requires": {
            "desktop": desktop,
            "authentication": if desktop { Value::Null } else { json!(false) }
        },
        "effects": {
            "may_open_editor": desktop && *action != "list",
            "editor_source_mutation": matches!(*action, "set" | "new" | "open"),
            "chart_mutation": matches!(*action, "compile" | "raw-compile"),
            "may_save_script": matches!(*action, "save" | "raw-compile"),
            "local_file_write": false,
            "reads_source_input": source_input,
            "transmits_source": if desktop { Value::Null } else { json!(*action == "check") }
        },
        "output_contract": null,
        "constraints": {},
        "discovery": [],
        "limits": []
    });
    if source_input {
        result["constraints"]["source_input"] = json!({
            "encoding": "UTF-8",
            "nonblank": true,
            "selection": "--file when supplied, otherwise non-terminal stdin",
            "reads_current_editor": false
        });
    }
    if desktop {
        result["discovery"] = json!([{
            "argument": "target-id",
            "argv": ["tv", "tab", "list"],
            "result_path": "data.tabs[].id"
        }]);
        result["limits"] = json!([
            "Use the intended Desktop target. Desktop session access is independent of MCP OAuth.",
            "Source transmission by Desktop is not tracked here; compile/save can send source through the application.",
            "Editor-backed operations can open the Editor even when reading. Inspect opened_editor and editor_open_before where returned.",
            "After a failed mutation, inspect Editor/chart state before retrying; source restoration is not guaranteed."
        ]);
    }
    let limit = match *action {
        "get" => {
            "Returns current Editor source and counts; this does not establish the active saved-script identity or saved state."
        }
        "set" => {
            result["readback"] = json!({"argv": ["tv", "pine", "get"]});
            "Replaces current Editor text and verifies it with normalized line endings. Does not save, compile or rebind the active saved script."
        }
        "new" => {
            result["constraints"]["script_type"] = json!({
                "choices": ["indicator", "strategy", "library"],
                "default": "indicator",
                "normalization": "trim and ASCII lowercase"
            });
            result["readback"] = json!({"argv": ["tv", "pine", "get"]});
            "Writes a version-6 template into the current Editor and verifies text. Does not create a new saved-script identity; saving can affect the existing binding."
        }
        "open" => {
            result["constraints"]["name"] = json!({
                "nonblank": true,
                "selection": "join words with spaces; case-insensitive exact name/title first, otherwise unique partial match",
                "unique_match_required": true
            });
            result["discovery"].as_array_mut().unwrap().push(json!({
                "argument": "name",
                "argv": ["tv", "pine", "list"],
                "result_path": "data.scripts[].name"
            }));
            "Rebinds and verifies the active saved script, including identity/version. No source-only fallback, save or compile; source readback alone is not binding proof."
        }
        "compile" => {
            result["readback"] = json!({"argv": ["tv", "pine", "errors"]});
            "Can add/update a chart study. Rejects save-related compile buttons; otherwise clicks a compile action or sends Ctrl+Enter. Inspect has_errors and study counts: no markers or a larger count does not prove the intended study's identity or runtime correctness."
        }
        "raw-compile" => {
            result["readback"] = json!({"argv": ["tv", "pine", "errors"]});
            "Legacy button selection can click Save and add to chart or a save button; Ctrl+Enter is used only when no button was clicked. Reports the action without diagnostics or study verification; do not treat success as compilation or persistence proof."
        }
        "save" => {
            "Sends the platform save shortcut and requires explicit saved=true and dirty_after=false. Naming an unsaved script is unsupported; a naming dialog can remain open on failure. This verifies UI dirty-state evidence, not an independent server revision fetch."
        }
        "errors" => {
            "Reads current Monaco markers without compiling. An empty result is not a fresh compile check; unavailable model markers can also yield an empty list."
        }
        "console" => {
            "Reads visible console/log DOM entries without compiling; entries may be historical or incomplete and are not a structured execution trace."
        }
        "list" => {
            "Lists saved scripts using the Desktop session without opening the Editor. Inspect data.error even on envelope success; an empty normalized list alone does not prove there are no saved scripts."
        }
        "analyze" => {
            "Offline static checks are heuristic, not the TradingView compiler. Inspect issue_count/diagnostics; zero issues does not establish valid compilation."
        }
        "alertconditions" => {
            "Offline candidates have best_effort confidence. Plot indices and condition IDs require TradingView compilation before use; discovery does not create alerts."
        }
        "check" => {
            result["source"] = json!("pine_facade");
            "Sends supplied source to the credential-free TradingView Pine facade over HTTP. It is not official MCP and does not touch the chart or Editor. Inspect compiled/errors/warnings separately from envelope success; compiled is based on returned diagnostics, not chart execution."
        }
        _ => return None,
    };
    result["limits"].as_array_mut().unwrap().push(json!(limit));
    result["examples"] = if source_input {
        json!([["tv", "pine", action, "--file", "example.pine"]])
    } else if *action == "open" {
        json!([["tv", "pine", "open", "Example Script"]])
    } else {
        json!([["tv", "pine", action]])
    };
    if desktop {
        result["limits"].as_array_mut().unwrap().push(json!(
            "Use the same explicit --target-id for discovery, operation and readback examples."
        ));
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cli::Cli, ops::validate_pine_script_type};
    use clap::Parser;

    #[test]
    fn examples_and_template_choices_match_execution() {
        for action in [
            "get",
            "set",
            "compile",
            "raw-compile",
            "save",
            "new",
            "open",
            "analyze",
            "alertconditions",
            "check",
            "errors",
            "console",
            "list",
        ] {
            let detail = describe(&["pine", action]).unwrap();
            let argv = detail["examples"][0].as_array().unwrap();
            assert!(Cli::try_parse_from(argv.iter().map(|v| v.as_str().unwrap())).is_ok());
        }

        let detail = describe(&["pine", "new"]).unwrap();
        for choice in detail["constraints"]["script_type"]["choices"]
            .as_array()
            .unwrap()
        {
            assert!(validate_pine_script_type(choice.as_str().unwrap()).is_ok());
        }
        assert!(validate_pine_script_type("unsupported").is_err());
    }
}
