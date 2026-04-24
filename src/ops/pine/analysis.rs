use serde::Serialize;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ArrayInfo {
    size: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct PineDiagnostic {
    line: usize,
    column: usize,
    message: String,
    severity: &'static str,
}

pub fn pine_analyze(source: &str, input_source: &str) -> Value {
    let diagnostics = analyze_source(source);

    json!({
        "input_source": input_source,
        "issue_count": diagnostics.len(),
        "diagnostics": diagnostics,
        "note": if diagnostics.is_empty() {
            Value::String("No static analysis issues found. Use pine check or pine compile for full TradingView validation.".to_string())
        } else {
            Value::Null
        },
    })
}

fn analyze_source(source: &str) -> Vec<PineDiagnostic> {
    let lines = source.lines().collect::<Vec<_>>();
    let arrays = collect_arrays(&lines);
    let mut diagnostics = Vec::new();

    diagnostics.extend(analyze_array_bounds(&lines, &arrays));
    diagnostics.extend(analyze_empty_first_last(&lines, &arrays));
    diagnostics.extend(analyze_strategy_usage(&lines));
    diagnostics.extend(analyze_version(source, &lines));

    diagnostics
}

fn collect_arrays(lines: &[&str]) -> HashMap<String, ArrayInfo> {
    let mut arrays = HashMap::new();
    for line in lines {
        if let Some((name, args)) = assignment_after(line, "array.from(") {
            arrays.insert(
                name.to_string(),
                ArrayInfo {
                    size: Some(count_comma_args(args)),
                },
            );
            continue;
        }

        if let Some((name, args)) = assignment_after(line, "array.new")
            && let Some(open_index) = args.find('(')
        {
            let args = &args[open_index + 1..];
            arrays.insert(
                name.to_string(),
                ArrayInfo {
                    size: first_integer_arg(args),
                },
            );
        }
    }
    arrays
}

fn assignment_after<'a>(line: &'a str, marker: &str) -> Option<(&'a str, &'a str)> {
    let equal_index = line.find('=')?;
    let name = line[..equal_index].trim();
    if name.is_empty() || !is_identifier(name) {
        return None;
    }
    let rhs = line[equal_index + 1..].trim_start();
    let marker_index = rhs.find(marker)?;
    Some((name, &rhs[marker_index + marker.len()..]))
}

fn analyze_array_bounds(
    lines: &[&str],
    arrays: &HashMap<String, ArrayInfo>,
) -> Vec<PineDiagnostic> {
    let mut diagnostics = Vec::new();
    for (line_index, line) in lines.iter().enumerate() {
        for method in ["get", "set"] {
            let needle = format!("array.{method}(");
            let mut offset = 0;
            while let Some(index) = line[offset..].find(&needle) {
                let start = offset + index;
                let args = &line[start + needle.len()..];
                if let Some((arr_name, arg_tail)) = split_first_arg(args)
                    && let Some(raw_index) = first_raw_arg(arg_tail)
                    && let Ok(index_value) = raw_index.trim().parse::<isize>()
                    && let Some(info) = arrays.get(arr_name.trim())
                    && let Some(size) = info.size
                    && (index_value < 0 || index_value as usize >= size)
                {
                    diagnostics.push(PineDiagnostic {
                        line: line_index + 1,
                        column: start + 1,
                        message: format!(
                            "array.{method}({}, {index_value}) — index {index_value} out of bounds (array size is {size})",
                            arr_name.trim()
                        ),
                        severity: "error",
                    });
                }
                offset = start + needle.len();
            }
        }
    }
    diagnostics
}

fn analyze_empty_first_last(
    lines: &[&str],
    arrays: &HashMap<String, ArrayInfo>,
) -> Vec<PineDiagnostic> {
    let mut diagnostics = Vec::new();
    for (line_index, line) in lines.iter().enumerate() {
        for method in ["first", "last"] {
            let needle = format!(".{method}()");
            let mut offset = 0;
            while let Some(index) = line[offset..].find(&needle) {
                let dot_index = offset + index;
                let name = identifier_before(line, dot_index);
                if let Some(name) = name
                    && name != "array"
                    && arrays.get(name).and_then(|info| info.size) == Some(0)
                {
                    diagnostics.push(PineDiagnostic {
                        line: line_index + 1,
                        column: dot_index + 1 - name.len(),
                        message: format!(
                            "{name}.{method}() called on possibly empty array (declared with size 0)"
                        ),
                        severity: "warning",
                    });
                }
                offset = dot_index + needle.len();
            }
        }
    }
    diagnostics
}

fn analyze_strategy_usage(lines: &[&str]) -> Vec<PineDiagnostic> {
    let has_strategy_decl = lines
        .iter()
        .any(|line| line.trim_start().starts_with("strategy("));
    if has_strategy_decl {
        return Vec::new();
    }

    for (line_index, line) in lines.iter().enumerate() {
        if line.contains("strategy.entry") || line.contains("strategy.close") {
            return vec![PineDiagnostic {
                line: line_index + 1,
                column: 1,
                message: "strategy.entry/close used but no strategy() declaration found — did you mean to use indicator()?".to_string(),
                severity: "error",
            }];
        }
    }
    Vec::new()
}

fn analyze_version(source: &str, lines: &[&str]) -> Vec<PineDiagnostic> {
    if !source.contains("//@version=") {
        return Vec::new();
    }
    for line in lines {
        let trimmed = line.trim();
        if trimmed == "//@version=6" {
            return Vec::new();
        }
        if let Some(version) = trimmed.strip_prefix("//@version=") {
            if let Ok(version) = version.parse::<u32>()
                && version < 5
            {
                return vec![PineDiagnostic {
                    line: 1,
                    column: 1,
                    message: format!(
                        "Script uses Pine v{version} — consider upgrading to v6 for latest features"
                    ),
                    severity: "info",
                }];
            }
            return Vec::new();
        }
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            return Vec::new();
        }
    }
    Vec::new()
}

fn split_first_arg(args: &str) -> Option<(&str, &str)> {
    let comma = args.find(',')?;
    Some((&args[..comma], &args[comma + 1..]))
}

fn first_raw_arg(args: &str) -> Option<&str> {
    let trimmed = args.trim_start();
    let end = trimmed.find([',', ')']).unwrap_or(trimmed.len());
    let value = &trimmed[..end];
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

fn count_comma_args(args: &str) -> usize {
    let closed = args.split(')').next().unwrap_or(args).trim();
    if closed.is_empty() {
        0
    } else {
        closed.split(',').count()
    }
}

fn first_integer_arg(args: &str) -> Option<usize> {
    let first = args.split([',', ')']).next()?.trim();
    if first.is_empty() {
        None
    } else {
        first.parse::<usize>().ok()
    }
}

fn identifier_before(line: &str, end: usize) -> Option<&str> {
    let prefix = &line[..end];
    let start = prefix
        .char_indices()
        .rev()
        .find_map(|(index, ch)| {
            if is_identifier_char(ch) {
                None
            } else {
                Some(index + ch.len_utf8())
            }
        })
        .unwrap_or(0);
    let name = &prefix[start..];
    if is_identifier(name) {
        Some(name)
    } else {
        None
    }
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(ch) if ch == '_' || ch.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(is_identifier_char)
}

fn is_identifier_char(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diagnostics(source: &str) -> Vec<PineDiagnostic> {
        analyze_source(source)
    }

    #[test]
    fn clean_v6_script_has_no_issues() {
        let result = diagnostics(
            r#"//@version=6
indicator("Test", overlay=true)
a = array.from(1, 2, 3)
val = array.get(a, 1)
plot(close)"#,
        );

        assert!(result.is_empty());
    }

    #[test]
    fn array_get_out_of_bounds_is_error() {
        let result = diagnostics(
            r#"//@version=6
indicator("Test")
a = array.from(1, 2, 3)
val = array.get(a, 5)"#,
        );

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].severity, "error");
        assert!(result[0].message.contains("out of bounds"));
        assert!(result[0].message.contains("index 5"));
        assert!(result[0].message.contains("array size is 3"));
    }

    #[test]
    fn array_get_negative_index_is_error() {
        let result = diagnostics(
            r#"//@version=6
indicator("Test")
a = array.from(1, 2)
val = array.get(a, -1)"#,
        );

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].severity, "error");
    }

    #[test]
    fn array_set_out_of_bounds_is_error() {
        let result = diagnostics(
            r#"//@version=6
indicator("Test")
a = array.new_float(3)
array.set(a, 10, 99.0)"#,
        );

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].severity, "error");
        assert!(result[0].message.contains("array.set"));
    }

    #[test]
    fn valid_array_index_has_no_issue() {
        let result = diagnostics(
            r#"//@version=6
indicator("Test")
a = array.from(10, 20, 30, 40, 50)
val = array.get(a, 4)"#,
        );

        assert!(result.is_empty());
    }

    #[test]
    fn first_on_empty_array_is_warning() {
        let result = diagnostics(
            r#"//@version=6
indicator("Test")
a = array.new_float(0)
x = a.first()"#,
        );

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].severity, "warning");
        assert!(result[0].message.contains("empty array"));
    }

    #[test]
    fn last_on_empty_array_is_warning() {
        let result = diagnostics(
            r#"//@version=6
indicator("Test")
a = array.new_float(0)
x = a.last()"#,
        );

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].severity, "warning");
    }

    #[test]
    fn first_on_non_empty_array_has_no_issue() {
        let result = diagnostics(
            r#"//@version=6
indicator("Test")
a = array.from(1, 2, 3)
x = a.first()"#,
        );

        assert!(result.is_empty());
    }

    #[test]
    fn strategy_entry_without_strategy_declaration_is_error() {
        let result = diagnostics(
            r#"//@version=6
indicator("Test")
strategy.entry("Long", strategy.long)"#,
        );

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].severity, "error");
        assert!(result[0].message.contains("no strategy() declaration"));
    }

    #[test]
    fn strategy_entry_with_strategy_declaration_has_no_issue() {
        let result = diagnostics(
            r#"//@version=6
strategy("Test", overlay=true)
if close > open
    strategy.entry("Long", strategy.long)"#,
        );

        assert!(result.is_empty());
    }

    #[test]
    fn old_version_reports_info() {
        let result = diagnostics(
            r#"//@version=3
study("Test")
plot(close)"#,
        );

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].severity, "info");
        assert!(result[0].message.contains("v3"));
        assert!(result[0].message.contains("upgrading"));
    }

    #[test]
    fn v5_has_no_version_warning() {
        let result = diagnostics(
            r#"//@version=5
indicator("Test")
plot(close)"#,
        );

        assert!(result.is_empty());
    }

    #[test]
    fn reports_multiple_issues() {
        let result = diagnostics(
            r#"//@version=6
indicator("Test")
a = array.from(1, 2)
b = array.new_float(0)
x = array.get(a, 5)
y = b.first()
strategy.entry("Long", strategy.long)"#,
        );

        assert!(result.len() >= 3);
        assert!(
            result
                .iter()
                .filter(|item| item.severity == "error")
                .count()
                >= 2
        );
        assert!(
            result
                .iter()
                .filter(|item| item.severity == "warning")
                .count()
                >= 1
        );
    }

    #[test]
    fn pine_analyze_returns_input_source_and_note() {
        let result = pine_analyze("//@version=6\nindicator(\"X\")\nplot(close)", "stdin");

        assert_eq!(result["input_source"], "stdin");
        assert_eq!(result["issue_count"], 0);
        assert!(result["diagnostics"].as_array().unwrap().is_empty());
        assert!(result["note"].as_str().unwrap().contains("No static"));
    }
}
