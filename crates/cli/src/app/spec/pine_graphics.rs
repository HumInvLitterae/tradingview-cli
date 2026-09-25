//! Pine graphics readback and lossy summary boundaries.

use serde_json::{Value, json};

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    let ["data", action @ ("lines" | "labels" | "tables" | "boxes")] = path else {
        return None;
    };
    let mut result = json!({
        "source": "internal_api",
        "requires": {"desktop": true, "authentication": null},
        "effects": {"chart_mutation": false, "local_file_write": false},
        "output_contract": null,
        "constraints": {
            "filter": {
                "matching": "case-sensitive substring of study description or short description",
                "trimmed": false,
                "empty": "all readable studies",
                "entity_id_selector": false
            }
        },
        "discovery": [{
            "argument": "target-id",
            "argv": ["tv", "tab", "list"],
            "result_path": "data.tabs[].id"
        }],
        "limits": [
            "Reads Pine-generated graphics from the selected chart, not hand-drawn objects from draw list. No MCP or other target fallback is used.",
            "Study rows contain names but no study entity IDs. Same-name instances can remain ambiguous; matching names alone do not identify the intended instance.",
            "Studies without readable primitives can be omitted and per-study extraction errors are suppressed. Empty output is not proof that no graphics exist; visibility is not used as a filter.",
            "Coordinates are exposed internal values. Do not assume x/x1/x2 are Unix timestamps or that y values are prices without the Pine script's coordinate and scale context.",
            "Summaries are current chart observations, not complete historical exports or trading signals."
        ]
    });
    match *action {
        "lines" => {
            result["limits"].as_array_mut().unwrap().extend([
                json!("y1/y2 are rounded to two decimals before equality comparison. horizontal_levels deduplicates equal rounded levels and sorts descending, so small slopes can appear horizontal."),
                json!("verbose adds all_lines with primitive IDs, coordinates and style fields; y values are still rounded. total_lines is the extracted primitive count, not the number of unique horizontal levels.")
            ]);
        }
        "labels" => {
            result["constraints"]["max"] = json!({
                "default": 500,
                "minimum": 0,
                "upper_limit": "usize parser bound only",
                "scope": "per study",
                "selection": "last N readable labels in primitive iteration order"
            });
            result["limits"].as_array_mut().unwrap().extend([
                json!("Labels without text and without numeric y are omitted. price is rounded to two decimals and can be null; verbose adds id/x/yloc/style fields without validating price meaning."),
                json!("max=0 returns no labels while retaining counts. Read total_labels, available_labels, showing and truncated separately. This limit is applied after extraction, not a bound on Desktop-side collection."),
                json!("Iteration order is not a guaranteed chronological order. The retained last N entries are not guaranteed newest labels; there is no paging argument.")
            ]);
        }
        "tables" => {
            result["limits"].as_array_mut().unwrap().extend([
                json!("Cells are grouped by internal table/row/column integers and sorted by those keys. Missing indices default to zero; repeated coordinates overwrite earlier text."),
                json!("Output retains only formatted row strings joined with ' | '. Empty cells and rows are dropped; table IDs and row/column coordinates are not returned. This is not a lossless rectangular table or typed dataset."),
                json!("There is no verbose or max option for tables. Delimiter characters in cell text are not escaped, so splitting row strings cannot reliably reconstruct cells.")
            ]);
        }
        "boxes" => {
            result["limits"].as_array_mut().unwrap().extend([
                json!("Boxes with two numeric y endpoints become high/low zones rounded to two decimals. Equal rounded pairs are deduplicated and zones sort by descending high."),
                json!("verbose adds all_boxes for readable numeric-endpoint boxes, retaining primitive IDs, x coordinates and colors. total_boxes includes extracted boxes that could not produce a zone; zones are not independently validated support/resistance levels.")
            ]);
        }
        _ => unreachable!(),
    }
    result["examples"] = if *action == "tables" {
        json!([[
            "tv",
            "--target-id",
            "<target_id>",
            "data",
            action,
            "--filter",
            "Example"
        ]])
    } else {
        json!([[
            "tv",
            "--target-id",
            "<target_id>",
            "data",
            action,
            "--filter",
            "Example",
            "--verbose"
        ]])
    };
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;

    #[test]
    fn graphics_examples_match_cli_options() {
        for action in ["lines", "labels", "tables", "boxes"] {
            let detail = describe(&["data", action]).unwrap();
            let argv = detail["examples"][0].as_array().unwrap();
            assert!(Cli::try_parse_from(argv.iter().map(|v| v.as_str().unwrap())).is_ok());
        }

        assert!(Cli::try_parse_from(["tv", "data", "labels", "--max", "0"]).is_ok());
        assert!(Cli::try_parse_from(["tv", "data", "tables", "--verbose"]).is_err());
    }
}
