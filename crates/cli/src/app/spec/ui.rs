//! UI effects depend on the selected target, focus and element, not syntax alone.

use super::super::safety::UNSAFE_UI_EVAL_ENV;
use crate::ops::{
    DEFAULT_SCROLL_AMOUNT, ELEMENT_STRATEGIES, FIND_STRATEGIES, PANEL_ACTIONS, SCROLL_DIRECTIONS,
};
use serde_json::{Value, json};

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    let ["ui", action] = path else { return None };
    let mut result = json!({
        "source": "desktop_ui",
        "requires": {"desktop": true, "authentication": null},
        "effects": {"ui_mutation": *action != "find", "account_mutation": null},
        "output_contract": null,
        "constraints": {},
        "discovery": [{"argument": "target-id", "argv": ["tv", "tab", "list"], "result_path": "data.tabs[].id"}],
        "limits": [
            "Select the intended target and inspect its current UI before acting. UI state and focus determine the effect.",
            "Use a dedicated command when it expresses the requested operation. A successful input dispatch is not proof of the intended UI outcome.",
            "After uncertain outcomes inspect the UI before retrying clicks, text input or toggles. Account effects are not inferred from command syntax."
        ]
    });
    let example = match *action {
        "find" => {
            result["effects"]["account_mutation"] = json!(false);
            result["constraints"] = json!({"query": {"nonblank": true, "tokens_joined_with": " "},
                "strategy": {"choices": FIND_STRATEGIES, "default": "text", "trimmed": true}});
            result["limits"].as_array_mut().unwrap().push(json!("Returns at most 20 element observations; absence does not prove that no matching UI exists. CSS support here does not imply CSS support for click/hover."));
            json!(["tv", "ui", "find", "Indicators"])
        }
        "click" | "hover" => {
            result["constraints"] =
                json!({"by": {"choices": ELEMENT_STRATEGIES}, "value": {"nonblank": true}});
            result["discovery"].as_array_mut().unwrap().push(json!({"argument": "value",
                "argv": ["tv", "ui", "find", "<query>"], "result_path": "data.elements[]",
                "selection": "Inspect text, aria_label and data_name and choose a supported selector; these are not stable IDs."}));
            result["limits"].as_array_mut().unwrap().push(json!("The first matching element may be selected. Hover can change menus/tooltips without clicking; inspect the current match rather than guessing."));
            json!([
                "tv",
                "ui",
                action,
                "--by",
                "text",
                "--value",
                "<observed text>"
            ])
        }
        "keyboard" => {
            result["constraints"]["key"] = json!({"nonblank": true, "trimmed": true,
                "supported_description": "ASCII letters a-z/A-Z; Enter, Escape, Tab, Backspace, Delete, ArrowUp/Down/Left/Right, Space, Home, End, PageUp, PageDown, F1, F2, F5."});
            result["limits"].as_array_mut().unwrap().push(json!("Modifiers are explicit flags. The focused UI receives the key; shortcuts can trigger account or chart actions."));
            json!(["tv", "ui", "keyboard", "Escape"])
        }
        "type" => {
            result["constraints"]["text"] = json!({"nonempty": true, "tokens_joined_with": " "});
            result["effects"]["inserts_text_at_focus"] = json!(true);
            result["limits"].as_array_mut().unwrap().push(json!("Does not select or focus an input for you. Output includes a prefix of typed text; do not use this command to enter secrets."));
            json!(["tv", "ui", "type", "Example text"])
        }
        "scroll" => {
            result["constraints"] = json!({"direction": {"choices": SCROLL_DIRECTIONS, "default": "down", "trimmed": true, "case": "ASCII lowercase"},
                "amount": {"finite": true, "default": DEFAULT_SCROLL_AMOUNT}});
            result["effects"]["viewport_change"] = json!(true);
            result["limits"].as_array_mut().unwrap().push(json!("Dispatches a wheel event near the chart center; does not establish a particular visible date range. Amount is finite but not restricted to positive values."));
            json!(["tv", "ui", "scroll", "down", "--amount", "300"])
        }
        "panel" => {
            result["constraints"] = json!({"panel": {"nonblank": true, "trimmed": true,
                "known_names": ["pine-editor", "strategy-tester", "watchlist", "alerts", "trading"]},
                "action": {"choices": PANEL_ACTIONS, "default": "toggle", "trimmed": true}});
            result["limits"].as_array_mut().unwrap().push(json!("Panel support is checked in the current page. Toggle reverses current state; open/close depend on observed visibility."));
            json!(["tv", "ui", "panel", "watchlist", "open"])
        }
        "fullscreen" => {
            result["effects"]["toggles_fullscreen"] = json!(true);
            json!(["tv", "ui", "fullscreen"])
        }
        "mouse" => {
            result["constraints"] = json!({"x": {"finite": true}, "y": {"finite": true}});
            result["effects"]["coordinate_click"] = json!(true);
            result["limits"].as_array_mut().unwrap().push(json!("Uses current page coordinates. Right selects the right button; double adds the second click. Coordinates are not stable element identifiers."));
            json!(["tv", "ui", "mouse", "100", "100"])
        }
        "eval" => {
            result["effects"]["ui_mutation"] = Value::Null;
            result["effects"]["arbitrary_page_javascript"] = json!(true);
            result["requires"]["environment"] = json!({UNSAFE_UI_EVAL_ENV: "1"});
            result["constraints"]["expression"] =
                json!({"nonblank": true, "tokens_joined_with": " "});
            result["limits"].as_array_mut().unwrap().push(json!("Disabled by default. Runs in the authenticated page context with unconstrained effects; opt-in is not permission for an arbitrary account operation. Spec lookup neither checks nor enables the gate."));
            json!(["tv", "ui", "eval", "1+1"])
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

    #[test]
    fn examples_parse_without_erasing_unknown_effects() {
        for action in [
            "find",
            "click",
            "hover",
            "keyboard",
            "type",
            "scroll",
            "panel",
            "fullscreen",
            "mouse",
            "eval",
        ] {
            let spec = describe(&["ui", action]).unwrap();
            for example in spec["examples"].as_array().unwrap() {
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
            if action == "eval" {
                assert!(spec["effects"]["ui_mutation"].is_null());
            } else {
                assert_eq!(spec["effects"]["ui_mutation"], action != "find");
            }
        }
        let find = describe(&["ui", "find"]).unwrap();
        let click = describe(&["ui", "click"]).unwrap();
        assert!(
            find["constraints"]["strategy"]["choices"]
                .as_array()
                .unwrap()
                .contains(&json!("css"))
        );
        assert!(
            !click["constraints"]["by"]["choices"]
                .as_array()
                .unwrap()
                .contains(&json!("css"))
        );
        assert_eq!(
            describe(&["ui", "eval"]).unwrap()["requires"]["environment"][UNSAFE_UI_EVAL_ENV],
            "1"
        );
    }
}
