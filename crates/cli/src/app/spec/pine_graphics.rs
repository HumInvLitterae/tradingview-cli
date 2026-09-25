//! Pine graphics readback and lossy summary boundaries.

use serde_json::{Value, json};

use crate::ops::{DEFAULT_OHLCV_COUNT, MAX_OHLCV_COUNT};

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    if path == ["data", "shapes"] {
        return Some(shapes());
    }
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

fn shapes() -> Value {
    json!({
        "source": "internal_api", "output_contract": null,
        "requires": {"desktop": true, "authentication": null},
        "effects": {"chart_mutation": false, "local_file_write": false},
        "constraints": {
            "filter": {"matching": "case-sensitive substring of first available description, shortDescription or meta ID", "trimmed": false, "empty": "all readable studies", "entity_id_selector": false},
            "count": {"minimum": 0, "default": DEFAULT_OHLCV_COUNT, "clamped_maximum": MAX_OHLCV_COUNT, "unit": "bar-index window per study, not signal count"},
            "verbose": {"default": false}
        },
        "discovery": [{"argument": "target-id", "argv": ["tv", "tab", "list"], "result_path": "data.tabs[].id"}],
        "limits": [
            "Reads only study plots whose internal type is shapes, intended for plotshape/plotchar observations. It does not read hand-drawn objects, line.new/label.new primitives or every Pine output type. Visibility is not used as a filter.",
            "Scans each study from its last available index backward, then plot declaration order within each bar. count caps the index span, not returned signals; multiple plots can yield multiple signals per bar. scan_count is the effective requested cap, while bars_scanned is the index span even if individual rows are missing. count=0 can retain plot metadata with no signals.",
            "Null, undefined, false, numeric zero and nonfinite numbers are inactive. Nonempty strings and other values are treated as active; returned value is the raw plot value, not necessarily a price or boolean. Zero-valued absolute-position plots can be omitted.",
            "Shape/location/color/size come from metainfo styles/defaults, not guaranteed current rendered overrides. The result does not fully reconstruct glyph/text, pixel position, offsets or plotting geometry.",
            "OHLC is read from the main chart series at the same bar index, without independent timestamp alignment or plot-offset correction. Numeric timestamps are interpreted as Unix seconds. OHLC values are rounded to two decimals even with verbose; missing data remains null.",
            "verbose adds plot IDs, plot/data indexes and size; it does not add study entity IDs or full chart symbol/timeframe identity. Same-name instances remain ambiguous. Confirm chart context before interpretation.",
            "Unavailable studies and per-study extraction exceptions are omitted. A malformed non-array raw response normalizes to empty success. Empty signals do not prove no chart markers, complete historical coverage or closed-bar confirmation; recent unclosed bars can change. No source fallback or trading-signal validation is performed."
        ],
        "examples": [["tv", "--target-id", "<target_id>", "data", "shapes", "--filter", "Example", "--count", "100", "--verbose"]]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;

    #[test]
    fn graphics_examples_match_cli_options() {
        for action in ["lines", "labels", "tables", "boxes", "shapes"] {
            let detail = describe(&["data", action]).unwrap();
            let argv = detail["examples"][0].as_array().unwrap();
            assert!(Cli::try_parse_from(argv.iter().map(|v| v.as_str().unwrap())).is_ok());
        }

        assert!(Cli::try_parse_from(["tv", "data", "labels", "--max", "0"]).is_ok());
        assert!(Cli::try_parse_from(["tv", "data", "shapes", "--count", "0"]).is_ok());
        assert!(Cli::try_parse_from(["tv", "data", "tables", "--verbose"]).is_err());
    }
}
