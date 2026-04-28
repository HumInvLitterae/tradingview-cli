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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PineAlertconditionCandidate {
    pub line: usize,
    pub column: usize,
    pub title: Option<String>,
    pub message: Option<String>,
    pub alert_cond_id: String,
    pub plot_index: usize,
    pub preceding_output_count: usize,
    pub confidence: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PineCall {
    name: String,
    start: usize,
    open: usize,
    close: usize,
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

pub fn pine_alertconditions(source: &str, input_source: &str) -> Value {
    let candidates = pine_alertcondition_candidates(source);
    let counted_output_count = collect_pine_calls(source).len();

    json!({
        "input_source": input_source,
        "candidate_count": candidates.len(),
        "counted_output_count": counted_output_count,
        "candidates": candidates,
        "note": "Static Pine alertcondition discovery only. Compile the script in TradingView before relying on plot indexes for alert creation.",
    })
}

pub fn pine_alertcondition_candidates(source: &str) -> Vec<PineAlertconditionCandidate> {
    let calls = collect_pine_calls(source);
    let mut candidates = Vec::new();

    for (counted_output_count, call) in calls.into_iter().enumerate() {
        if call.name == "alertcondition" {
            let (title, message) = alertcondition_literal_fields(source, &call);
            let (line, column) = line_column(source, call.start);
            candidates.push(PineAlertconditionCandidate {
                line,
                column,
                title,
                message,
                alert_cond_id: format!("plot_{counted_output_count}"),
                plot_index: counted_output_count,
                preceding_output_count: counted_output_count,
                confidence: "best_effort",
            });
        }
    }

    candidates
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

fn collect_pine_calls(source: &str) -> Vec<PineCall> {
    let sanitized = sanitize_for_call_scan(source);
    let bytes = sanitized.as_bytes();
    let mut calls = Vec::new();
    let mut index = 0usize;

    while index < bytes.len() {
        let ch = bytes[index] as char;
        if ch == '_' || ch.is_ascii_alphabetic() {
            let start = index;
            index += 1;
            while index < bytes.len() {
                let next = bytes[index] as char;
                if is_identifier_char(next) {
                    index += 1;
                } else {
                    break;
                }
            }

            let name = &sanitized[start..index];
            let mut open = index;
            while open < bytes.len() && bytes[open].is_ascii_whitespace() {
                open += 1;
            }

            let mut matched_call = false;
            if open < bytes.len()
                && bytes[open] == b'('
                && is_counted_pine_call(name)
                && !is_member_call(&sanitized, start)
                && let Some(close) = find_matching_paren(&sanitized, open)
            {
                calls.push(PineCall {
                    name: name.to_string(),
                    start,
                    open,
                    close,
                });
                index = close + 1;
                matched_call = true;
            }

            if !matched_call {
                index = index.max(start + 1);
            }
        } else {
            index += 1;
        }
    }

    calls
}

fn is_counted_pine_call(name: &str) -> bool {
    matches!(
        name,
        "alertcondition" | "bgcolor" | "plot" | "plotbar" | "plotcandle" | "plotchar" | "plotshape"
    )
}

fn is_member_call(source: &str, start: usize) -> bool {
    source[..start].chars().rev().find(|ch| !ch.is_whitespace()) == Some('.')
}

fn find_matching_paren(source: &str, open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (index, ch) in source[open..].char_indices() {
        let absolute = open + index;
        match ch {
            '(' => depth += 1,
            ')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(absolute);
                }
            }
            _ => {}
        }
    }
    None
}

fn alertcondition_literal_fields(
    source: &str,
    call: &PineCall,
) -> (Option<String>, Option<String>) {
    let args = &source[call.open + 1..call.close];
    let args = split_top_level_args(args);
    let title = named_string_arg(&args, "title")
        .or_else(|| args.get(1).and_then(|arg| string_literal_value(arg)));
    let message = named_string_arg(&args, "message")
        .or_else(|| args.get(2).and_then(|arg| string_literal_value(arg)));
    (title, message)
}

fn named_string_arg(args: &[String], name: &str) -> Option<String> {
    args.iter().find_map(|arg| {
        let (candidate, value) = arg.split_once('=')?;
        if candidate.trim() == name {
            string_literal_value(value)
        } else {
            None
        }
    })
}

fn split_top_level_args(args: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    let mut quote: Option<char> = None;
    let mut escaped = false;

    for ch in args.chars() {
        if let Some(quote_char) = quote {
            current.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == quote_char {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                current.push(ch);
            }
            '(' | '[' | '{' => {
                depth += 1;
                current.push(ch);
            }
            ')' | ']' | '}' => {
                depth = depth.saturating_sub(1);
                current.push(ch);
            }
            ',' if depth == 0 => {
                result.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    if !current.trim().is_empty() || args.ends_with(',') {
        result.push(current.trim().to_string());
    }

    result
}

fn string_literal_value(value: &str) -> Option<String> {
    let trimmed = value.trim();
    let mut chars = trimmed.chars();
    let quote = chars.next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }

    let mut result = String::new();
    let mut escaped = false;
    for ch in chars {
        if escaped {
            result.push(match ch {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                other => other,
            });
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == quote {
            return Some(result);
        } else {
            result.push(ch);
        }
    }
    None
}

fn line_column(source: &str, byte_index: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut line_start = 0usize;
    for (index, ch) in source.char_indices() {
        if index >= byte_index {
            break;
        }
        if ch == '\n' {
            line += 1;
            line_start = index + ch.len_utf8();
        }
    }
    (line, byte_index - line_start + 1)
}

fn sanitize_for_call_scan(source: &str) -> String {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum State {
        Normal,
        LineComment,
        BlockComment,
        String(char),
    }

    let mut output = String::with_capacity(source.len());
    let mut chars = source.char_indices().peekable();
    let mut state = State::Normal;
    let mut escaped = false;

    while let Some((_, ch)) = chars.next() {
        match state {
            State::Normal => {
                if ch == '/'
                    && let Some((_, next)) = chars.peek().copied()
                {
                    if next == '/' {
                        push_spaces_like(&mut output, ch);
                        chars.next();
                        push_spaces_like(&mut output, next);
                        state = State::LineComment;
                        continue;
                    }
                    if next == '*' {
                        push_spaces_like(&mut output, ch);
                        chars.next();
                        push_spaces_like(&mut output, next);
                        state = State::BlockComment;
                        continue;
                    }
                }

                if ch == '"' || ch == '\'' {
                    push_spaces_like(&mut output, ch);
                    state = State::String(ch);
                    escaped = false;
                } else {
                    output.push(ch);
                }
            }
            State::LineComment => {
                if ch == '\n' {
                    output.push('\n');
                    state = State::Normal;
                } else {
                    push_spaces_like(&mut output, ch);
                }
            }
            State::BlockComment => {
                if ch == '*'
                    && let Some((_, next)) = chars.peek().copied()
                    && next == '/'
                {
                    push_spaces_like(&mut output, ch);
                    chars.next();
                    push_spaces_like(&mut output, next);
                    state = State::Normal;
                } else if ch == '\n' {
                    output.push('\n');
                } else {
                    push_spaces_like(&mut output, ch);
                }
            }
            State::String(quote) => {
                if ch == '\n' {
                    output.push('\n');
                    escaped = false;
                } else {
                    push_spaces_like(&mut output, ch);
                }

                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == quote {
                    state = State::Normal;
                }
            }
        }
    }

    output
}

fn push_spaces_like(output: &mut String, ch: char) {
    for _ in 0..ch.len_utf8() {
        output.push(' ');
    }
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

    #[test]
    fn pine_alertconditions_reports_best_effort_plot_ids() {
        let result = pine_alertconditions(
            r#"//@version=6
indicator("Signals")
plot(close)
bgcolor(close > open ? color.green : na)
plotshape(close > open, title="Shape")
alertcondition(close > open, "Long", "Long message")
alertcondition(close < open, title="Short", message="Short message")"#,
            "stdin",
        );

        assert_eq!(result["input_source"], "stdin");
        assert_eq!(result["candidate_count"], 2);
        assert_eq!(result["counted_output_count"], 5);
        assert_eq!(result["candidates"][0]["alert_cond_id"], "plot_3");
        assert_eq!(result["candidates"][0]["plot_index"], 3);
        assert_eq!(result["candidates"][0]["title"], "Long");
        assert_eq!(result["candidates"][0]["message"], "Long message");
        assert_eq!(result["candidates"][1]["alert_cond_id"], "plot_4");
        assert_eq!(result["candidates"][1]["title"], "Short");
        assert_eq!(result["candidates"][1]["message"], "Short message");
        assert_eq!(result["candidates"][1]["confidence"], "best_effort");
    }

    #[test]
    fn pine_alertconditions_ignores_comments_and_strings() {
        let result = pine_alertconditions(
            r#"//@version=6
indicator("alertcondition(false, \"No\")")
// alertcondition(close > open, "Commented")
/*
plot(close)
alertcondition(close > open, "Blocked")
*/
plot(close)
label = "plotshape(close > open)"
alertcondition(close > open, "Real")"#,
            "stdin",
        );

        assert_eq!(result["candidate_count"], 1);
        assert_eq!(result["counted_output_count"], 2);
        assert_eq!(result["candidates"][0]["alert_cond_id"], "plot_1");
        assert_eq!(result["candidates"][0]["title"], "Real");
    }

    #[test]
    fn pine_alertconditions_handles_multiline_calls() {
        let result = pine_alertconditions(
            r#"//@version=6
indicator("Signals")
plot(
    close
)
alertcondition(
    close > open,
    title="Long",
    message="Multi line"
)"#,
            "file",
        );

        assert_eq!(result["input_source"], "file");
        assert_eq!(result["candidate_count"], 1);
        assert_eq!(result["candidates"][0]["alert_cond_id"], "plot_1");
        assert_eq!(result["candidates"][0]["title"], "Long");
        assert_eq!(result["candidates"][0]["message"], "Multi line");
    }
}
