//! Offline command discovery from the same clap tree used to parse invocations.

use std::{any::TypeId, path::PathBuf};

use clap::{Arg, Command, CommandFactory};
use serde_json::{Value, json};
use tradingview_core::{AppError, ErrorKind};

use crate::{build_info, cli::Cli};

mod desktop;
mod mcp_mutations;
mod mcp_reads;
mod replay;

pub(super) fn describe(path: &[String]) -> Result<Value, AppError> {
    let mut root = Cli::command();
    root.build();
    let mut command = &root;
    let mut canonical = Vec::new();
    for segment in path {
        command = command
            .get_subcommands()
            .find(|child| {
                !child.is_hide_set()
                    && child.get_name() != "help"
                    && (child.get_name() == segment
                        || child.get_all_aliases().any(|alias| alias == segment))
            })
            .ok_or_else(|| AppError::new(ErrorKind::Validation, "Unknown spec command path"))?;
        canonical.push(command.get_name());
    }

    let mut data = json!({
        "contract_version": "cli_spec.v1",
        "binary_version": build_info::VERSION,
        "command": canonical,
        "coverage": {
            "syntax": "clap_metadata",
            "validation": "partial",
            "semantics": "unavailable"
        }
    });
    if path.is_empty() {
        let mut entries = Vec::new();
        index(&root, &mut Vec::new(), &mut entries);
        data["commands"] = json!(entries);
        return Ok(data);
    }

    data["description"] = json!(
        command
            .get_long_about()
            .or(command.get_about())
            .map(ToString::to_string)
    );
    data["aliases"] = json!(command.get_all_aliases().collect::<Vec<_>>());
    data["arguments"] = command
        .get_arguments()
        .filter(|arg| !arg.is_hide_set())
        .map(|arg| argument(command, arg))
        .collect();
    data["subcommands"] = json!(
        command
            .get_subcommands()
            .filter(|child| !child.is_hide_set() && child.get_name() != "help")
            .map(Command::get_name)
            .collect::<Vec<_>>()
    );
    data["limitations"] = json!([
        "Conditional requirements and adapter validation are not fully represented.",
        "Defaults and examples are not observations of provider or account state."
    ]);
    data["semantics"] = Value::Null;
    if let Some(semantics) = mcp_reads::describe(&canonical)
        .or_else(|| mcp_mutations::describe(&canonical))
        .or_else(|| desktop::describe(&canonical))
        .or_else(|| replay::describe(&canonical))
    {
        data["coverage"]["semantics"] = json!("documented");
        data["semantics"] = semantics;
    }
    Ok(data)
}

fn index(command: &Command, path: &mut Vec<String>, entries: &mut Vec<Value>) {
    for child in command
        .get_subcommands()
        .filter(|child| !child.is_hide_set() && child.get_name() != "help")
    {
        path.push(child.get_name().to_owned());
        entries.push(json!({
            "command": path,
            "description": child.get_about().map(ToString::to_string)
        }));
        index(child, path, entries);
        path.pop();
    }
}

fn argument(command: &Command, arg: &Arg) -> Value {
    let arity = arg.get_num_args();
    let names = arg
        .get_value_names()
        .map(|names| names.iter().map(|name| name.as_str()).collect::<Vec<_>>());
    let defaults = arg
        .get_default_values()
        .iter()
        .map(|value| value.to_string_lossy())
        .collect::<Vec<_>>();
    let values = arg.get_possible_values();
    let choices = values
        .iter()
        .filter(|value| !value.is_hide_set())
        .map(|value| value.get_name())
        .collect::<Vec<_>>();
    let conflicts = command
        .get_arg_conflicts_with(arg)
        .iter()
        .map(|other| other.get_id().to_string())
        .collect::<Vec<_>>();
    json!({
        "id": arg.get_id().as_str(),
        "long": arg.get_long(),
        "short": arg.get_short(),
        "aliases": arg.get_all_aliases().unwrap_or_default(),
        "short_aliases": arg.get_all_short_aliases().unwrap_or_default(),
        "position": arg.get_index(),
        "required": arg.is_required_set(),
        "global": arg.is_global_set(),
        "action": format!("{:?}", arg.get_action()),
        "type": value_type(arg),
        "value_names": names,
        "arity": arity.map(|range| json!({
            "min": range.min_values(),
            "max": (range.max_values() != usize::MAX).then_some(range.max_values())
        })),
        "delimiter": arg.get_value_delimiter(),
        "defaults": defaults,
        "choices": choices,
        "conflicts": conflicts,
        "description": arg.get_long_help().or(arg.get_help()).map(ToString::to_string)
    })
}

fn value_type(arg: &Arg) -> Option<&'static str> {
    let id = arg.get_value_parser().type_id();
    for (candidate, name) in [
        (TypeId::of::<String>(), "string"),
        (TypeId::of::<PathBuf>(), "path"),
        (TypeId::of::<bool>(), "boolean"),
        (TypeId::of::<u16>(), "u16"),
        (TypeId::of::<u32>(), "u32"),
        (TypeId::of::<u64>(), "u64"),
        (TypeId::of::<usize>(), "usize"),
        (TypeId::of::<i32>(), "i32"),
        (TypeId::of::<i64>(), "i64"),
        (TypeId::of::<f64>(), "f64"),
    ] {
        if id == candidate {
            return Some(name);
        }
    }
    (!arg.get_possible_values().is_empty()).then_some("enum")
}

#[cfg(test)]
mod tests {
    use super::mcp_reads::history;
    use super::*;
    use tradingview_model::mcp_account::Request;

    #[test]
    fn nested_details_include_inherited_options_and_model_bounds() {
        let result = describe(&["mcp".into(), "alert".into(), "history".into()]).unwrap();
        let args = result["arguments"].as_array().unwrap();
        assert!(
            args.iter()
                .any(|arg| arg["long"] == "timeout" && arg["global"] == true)
        );
        assert!(
            args.iter()
                .any(|arg| arg["long"] == "symbol" && arg["required"] == true)
        );
        assert!(
            args.iter()
                .any(|arg| arg["long"] == "days" && arg["defaults"] == json!(["7"]))
        );
        let max = result["semantics"]["constraints"]["limit"]["maximum"]
            .as_u64()
            .unwrap() as u32;
        assert!(Request::alert_history("NASDAQ:EXAMPLE", 1, max).is_ok());
        assert!(Request::alert_history("NASDAQ:EXAMPLE", 1, max + 1).is_err());
        assert!(Request::alert_history("NASDAQ:EXAMPLE", 0, 1).is_err());
    }

    #[test]
    fn enumerations_and_examples_are_accepted_by_the_parser() {
        use clap::Parser;

        let detail = describe(&["quote".into()]).unwrap();
        let source = detail["arguments"]
            .as_array()
            .unwrap()
            .iter()
            .find(|arg| arg["id"] == "source")
            .unwrap();
        assert!(!source["choices"].as_array().unwrap().is_empty());
        for choice in source["choices"].as_array().unwrap() {
            assert!(
                Cli::try_parse_from(["tv", "quote", "--source", choice.as_str().unwrap()]).is_ok()
            );
        }
        for example in history()["examples"].as_array().unwrap() {
            let argv = example
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap());
            assert!(Cli::try_parse_from(argv).is_ok());
        }
    }

    #[test]
    fn every_index_entry_resolves_without_claiming_complete_validation() {
        let result = describe(&[]).unwrap();
        let entries = result["commands"].as_array().unwrap();
        assert!(entries.len() > 50);
        assert!(entries.iter().all(|entry| {
            !entry["command"]
                .as_array()
                .unwrap()
                .contains(&json!("help"))
        }));
        for entry in entries {
            let path = entry["command"]
                .as_array()
                .unwrap()
                .iter()
                .map(|part| part.as_str().unwrap().to_owned())
                .collect::<Vec<_>>();
            let detail = describe(&path).unwrap();
            assert_eq!(detail["command"], entry["command"]);
            assert_eq!(detail["coverage"]["validation"], "partial");
        }
        assert!(describe(&["mcp".into(), "nonexistent".into()]).is_err());
    }
}
