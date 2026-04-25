use std::{collections::HashSet, time::Duration};

use serde_json::{Value, json};

use crate::{
    cdp::{KeyEvent, KeyEventType, RuntimeEvaluator},
    error::{AppError, ErrorKind},
};

use super::common::{CHART_API, CHART_WIDGET_COLLECTION, js_string};

const MAX_WATCHLIST_BULK_SYMBOLS: usize = 50;
const MAX_WATCHLIST_BULK_DELAY_MS: u64 = 10_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PaneLayout {
    code: &'static str,
    name: &'static str,
}

const PANE_LAYOUTS: [PaneLayout; 18] = [
    PaneLayout {
        code: "s",
        name: "1 chart",
    },
    PaneLayout {
        code: "2h",
        name: "2 horizontal",
    },
    PaneLayout {
        code: "2v",
        name: "2 vertical",
    },
    PaneLayout {
        code: "2-1",
        name: "2 top, 1 bottom",
    },
    PaneLayout {
        code: "1-2",
        name: "1 top, 2 bottom",
    },
    PaneLayout {
        code: "3h",
        name: "3 horizontal",
    },
    PaneLayout {
        code: "3v",
        name: "3 vertical",
    },
    PaneLayout {
        code: "3s",
        name: "3 custom",
    },
    PaneLayout {
        code: "4",
        name: "2x2 grid",
    },
    PaneLayout {
        code: "4h",
        name: "4 horizontal",
    },
    PaneLayout {
        code: "4v",
        name: "4 vertical",
    },
    PaneLayout {
        code: "4s",
        name: "4 custom",
    },
    PaneLayout {
        code: "6",
        name: "6 charts",
    },
    PaneLayout {
        code: "8",
        name: "8 charts",
    },
    PaneLayout {
        code: "10",
        name: "10 charts",
    },
    PaneLayout {
        code: "12",
        name: "12 charts",
    },
    PaneLayout {
        code: "14",
        name: "14 charts",
    },
    PaneLayout {
        code: "16",
        name: "16 charts",
    },
];

pub fn validate_pane_layout(layout: &str) -> Result<(), AppError> {
    parse_pane_layout(layout).map(|_| ())
}

fn parse_pane_layout(layout: &str) -> Result<PaneLayout, AppError> {
    let normalized: String = layout
        .chars()
        .filter(|character| !character.is_ascii_whitespace())
        .flat_map(char::to_lowercase)
        .collect();

    let canonical = match normalized.as_str() {
        "single" | "1" | "1x1" => "s",
        "2x1" => "2h",
        "1x2" => "2v",
        "2x2" | "grid" | "quad" => "4",
        "3x1" => "3h",
        "1x3" => "3v",
        other => other,
    };

    PANE_LAYOUTS
        .iter()
        .copied()
        .find(|layout| layout.code == canonical)
        .ok_or_else(|| {
            AppError::new(
                ErrorKind::Validation,
                format!("Unknown pane layout: {layout}"),
            )
            .with_details(json!({
                "supported": supported_pane_layouts(),
                "aliases": {
                    "single": "s",
                    "1": "s",
                    "1x1": "s",
                    "2x1": "2h",
                    "1x2": "2v",
                    "2x2": "4",
                    "grid": "4",
                    "quad": "4",
                    "3x1": "3h",
                    "1x3": "3v"
                }
            }))
        })
}

fn supported_pane_layouts() -> Vec<Value> {
    PANE_LAYOUTS
        .iter()
        .map(|layout| json!({ "layout": layout.code, "layout_name": layout.name }))
        .collect()
}

pub async fn watchlist_get(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            r#"
            (function() {
                try {
                    var rightArea = document.querySelector('[class*="layout__area--right"]');
                    if (!rightArea || rightArea.offsetWidth < 50) return { count: 0, source: "panel_closed", symbols: [] };
                } catch(e) {}

                var results = [];
                var seen = {};
                var container = document.querySelector('[class*="layout__area--right"]');
                if (!container) return { count: 0, source: "no_container", symbols: [] };

                var symbolEls = container.querySelectorAll('[data-symbol-full]');
                for (var i = 0; i < symbolEls.length; i++) {
                    var sym = symbolEls[i].getAttribute('data-symbol-full');
                    if (!sym || seen[sym]) continue;
                    seen[sym] = true;

                    var row = symbolEls[i].closest('[class*="row"]') || symbolEls[i].parentElement;
                    var cells = row ? row.querySelectorAll('[class*="cell"], [class*="column"]') : [];
                    var nums = [];
                    for (var j = 0; j < cells.length; j++) {
                        var t = cells[j].textContent.trim();
                        if (t && /^[\-+]?[\d,]+\.?\d*%?$/.test(t.replace(/[\s,]/g, ''))) nums.push(t);
                    }
                    results.push({ symbol: sym, last: nums[0] || null, change: nums[1] || null, change_percent: nums[2] || null });
                }

                if (results.length > 0) return { count: results.length, source: "data_attributes", symbols: results };

                var items = container.querySelectorAll('[class*="symbolName"], [class*="tickerName"], [class*="symbol-"]');
                for (var k = 0; k < items.length; k++) {
                    var text = items[k].textContent.trim();
                    if (text && /^[A-Z][A-Z0-9.:!]{0,20}$/.test(text) && !seen[text]) {
                        seen[text] = true;
                        results.push({ symbol: text, last: null, change: null, change_percent: null });
                    }
                }

                return { count: results.length, source: results.length > 0 ? "text_scan" : "empty", symbols: results };
            })()
            "#,
            false,
        )
        .await
}

pub async fn watchlist_add(
    runtime: &mut impl RuntimeEvaluator,
    symbol: &str,
) -> Result<Value, AppError> {
    let symbol = symbol.trim();
    if symbol.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Symbol must not be empty",
        ));
    }

    let panel_state = ensure_watchlist_panel_open(runtime).await?;
    wait_after_panel_open(runtime, &panel_state).await?;

    let symbol_literal = js_string(symbol)?;
    let mut add_expression = format!(
        r#"
            (function() {{
                var requestedSymbol = {};
        "#,
        symbol_literal
    );
    add_expression.push_str(
        r#"
                function isVisible(element) {
                    if (!element) return false;
                    const rect = element.getBoundingClientRect();
                    return rect.width > 0 && rect.height > 0 && window.getComputedStyle(element).visibility !== 'hidden';
                }
                function rowForSymbolElement(element) {
                    return element.closest('[role="row"], [class*="row"], [data-role="list-item"], [class*="item"]')
                        || element.closest('tr')
                        || element.parentElement;
                }
                function readRows() {
                    const container = document.querySelector('[class*="layout__area--right"]');
                    if (!container) return { container: null, rows: [] };
                    const seen = {};
                    const rows = Array.from(container.querySelectorAll('[data-symbol-full]'))
                        .map(function(element) {
                            const symbol = element.getAttribute('data-symbol-full');
                            if (!symbol || seen[symbol]) return null;
                            seen[symbol] = true;
                            return { symbol: symbol, element: element, row: rowForSymbolElement(element) };
                        })
                        .filter(function(entry) { return entry && entry.row; });
                    return { container: container, rows: rows };
                }
                function dispatchMouseClick(element) {
                    const rect = element.getBoundingClientRect();
                    const eventOptions = {
                        bubbles: true,
                        cancelable: true,
                        view: window,
                        clientX: rect.left + (rect.width / 2),
                        clientY: rect.top + (rect.height / 2)
                    };
                    ['mousedown', 'mouseup', 'click'].forEach(function(type) {
                        element.dispatchEvent(new MouseEvent(type, eventOptions));
                    });
                    return 'mouse_event';
                }

                var before = readRows();
                var matchedBefore = before.rows.some(function(entry) { return entry.symbol === requestedSymbol; });
                if (matchedBefore) {
                    return {
                        found: true,
                        skipped: true,
                        action: 'already_present',
                        before_count: before.rows.length,
                        after_count: before.rows.length,
                        matched_before: true,
                        matched_after: true,
                        click_method: null
                    };
                }

                var selectors = [
                    '[data-name="add-symbol-button"]',
                    '[aria-label="Add symbol"]',
                    '[aria-label*="Add symbol"]',
                    'button[class*="addSymbol"]'
                ];
                for (var s = 0; s < selectors.length; s++) {
                    var btn = document.querySelector(selectors[s]);
                    if (btn && isVisible(btn)) {
                        var clickMethod = dispatchMouseClick(btn);
                        return {
                            found: true,
                            skipped: false,
                            selector: selectors[s],
                            before_count: before.rows.length,
                            matched_before: false,
                            click_method: clickMethod
                        };
                    }
                }
                var container = document.querySelector('[class*="layout__area--right"]');
                if (container) {
                    var buttons = container.querySelectorAll('button');
                    for (var i = 0; i < buttons.length; i++) {
                        var ariaLabel = buttons[i].getAttribute('aria-label') || '';
                        if (/add.*symbol/i.test(ariaLabel) || buttons[i].textContent.trim() === '+') {
                            var fallbackClickMethod = dispatchMouseClick(buttons[i]);
                            return {
                                found: true,
                                skipped: false,
                                method: 'fallback',
                                before_count: before.rows.length,
                                matched_before: false,
                                click_method: fallbackClickMethod
                            };
                        }
                    }
                }
                return {
                    found: false,
                    before_count: before.rows.length,
                    matched_before: false,
                    click_method: null
                };
            })()
        "#,
    );
    let add_clicked = runtime.evaluate(&add_expression, false).await?;

    if add_clicked
        .get("skipped")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Ok(json!({
            "symbol": symbol,
            "requested_symbol": symbol,
            "action": "already_present",
            "source": "dom_input",
            "opened_panel": panel_state.get("opened").cloned().unwrap_or(Value::Bool(false)),
            "add_button": add_clicked,
            "before_count": add_clicked.get("before_count").cloned().unwrap_or(Value::Null),
            "after_count": add_clicked.get("after_count").cloned().unwrap_or(Value::Null),
            "matched_before": true,
            "matched_after": true,
            "click_method": Value::Null,
        }));
    }

    if !add_clicked
        .get("found")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Add symbol button not found in watchlist panel",
        ));
    }

    runtime
        .evaluate(
            "new Promise(function(resolve) { setTimeout(function() { resolve(true); }, 300); })",
            true,
        )
        .await?;
    runtime.insert_text(symbol).await?;
    runtime
        .evaluate(
            "new Promise(function(resolve) { setTimeout(function() { resolve(true); }, 500); })",
            true,
        )
        .await?;
    dispatch_key(runtime, KeyEventType::KeyDown, "Enter", "Enter", 13).await?;
    dispatch_key(runtime, KeyEventType::KeyUp, "Enter", "Enter", 13).await?;
    runtime
        .evaluate(
            "new Promise(function(resolve) { setTimeout(function() { resolve(true); }, 300); })",
            true,
        )
        .await?;
    dispatch_key(runtime, KeyEventType::KeyDown, "Escape", "Escape", 27).await?;
    dispatch_key(runtime, KeyEventType::KeyUp, "Escape", "Escape", 27).await?;

    let verify = runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var requestedSymbol = {symbol_literal};
                    function rowForSymbolElement(element) {{
                        return element.closest('[role="row"], [class*="row"], [data-role="list-item"], [class*="item"]')
                            || element.closest('tr')
                            || element.parentElement;
                    }}
                    function readRows() {{
                        const container = document.querySelector('[class*="layout__area--right"]');
                        if (!container) return {{ container: null, rows: [] }};
                        const seen = {{}};
                        const rows = Array.from(container.querySelectorAll('[data-symbol-full]'))
                            .map(function(element) {{
                                const symbol = element.getAttribute('data-symbol-full');
                                if (!symbol || seen[symbol]) return null;
                                seen[symbol] = true;
                                return {{ symbol: symbol, element: element, row: rowForSymbolElement(element) }};
                            }})
                            .filter(function(entry) {{ return entry && entry.row; }});
                        return {{ container: container, rows: rows }};
                    }}
                    var after = readRows();
                    var matchedAfter = after.rows.some(function(entry) {{ return entry.symbol === requestedSymbol; }});
                    return {{
                        after_count: after.rows.length,
                        matched_after: matchedAfter
                    }};
                }})()
                "#
            ),
            false,
        )
        .await?;

    if verify.get("matched_after").and_then(Value::as_bool) != Some(true) {
        let details = json!({
            "symbol": symbol,
            "requested_symbol": symbol,
            "action": "add_unverified",
            "source": "dom_input",
            "opened_panel": panel_state.get("opened").cloned().unwrap_or(Value::Bool(false)),
            "add_button": add_clicked,
            "verify": verify,
        });
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!("Watchlist add did not confirm symbol after input: {symbol}"),
        )
        .with_details(details));
    }

    Ok(json!({
        "symbol": symbol,
        "requested_symbol": symbol,
        "action": "added",
        "source": "dom_input",
        "opened_panel": panel_state.get("opened").cloned().unwrap_or(Value::Bool(false)),
        "add_button": add_clicked,
        "before_count": add_clicked.get("before_count").cloned().unwrap_or(Value::Null),
        "after_count": verify.get("after_count").cloned().unwrap_or(Value::Null),
        "matched_before": add_clicked.get("matched_before").and_then(Value::as_bool).unwrap_or(false),
        "matched_after": true,
        "click_method": add_clicked.get("click_method").cloned().unwrap_or(Value::Null),
    }))
}

pub fn validate_watchlist_add_bulk_request(
    symbols: &[String],
    delay_ms: u64,
) -> Result<(), AppError> {
    if symbols.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "At least one symbol is required",
        ));
    }
    if delay_ms > MAX_WATCHLIST_BULK_DELAY_MS {
        return Err(AppError::new(
            ErrorKind::Validation,
            format!("--delay-ms must be at most {MAX_WATCHLIST_BULK_DELAY_MS}"),
        ));
    }

    let mut unique = HashSet::new();
    for symbol in symbols {
        let normalized = normalize_watchlist_symbol(symbol)?;
        unique.insert(normalized);
    }
    if unique.len() > MAX_WATCHLIST_BULK_SYMBOLS {
        return Err(AppError::new(
            ErrorKind::Validation,
            format!("At most {MAX_WATCHLIST_BULK_SYMBOLS} unique symbols can be added at once"),
        ));
    }

    Ok(())
}

pub async fn watchlist_add_bulk(
    runtime: &mut impl RuntimeEvaluator,
    symbols: &[String],
    delay_ms: u64,
    allow_partial: bool,
) -> Result<Value, AppError> {
    validate_watchlist_add_bulk_request(symbols, delay_ms)?;

    let unique_total = symbols
        .iter()
        .map(|symbol| normalize_watchlist_symbol(symbol))
        .collect::<Result<HashSet<_>, _>>()?
        .len();
    let mut seen = HashSet::new();
    let mut results = Vec::new();
    let mut processed_count = 0usize;
    let mut added_count = 0usize;
    let mut already_present_count = 0usize;
    let mut failed_count = 0usize;
    let mut skipped_duplicate_count = 0usize;

    for (input_index, symbol) in symbols.iter().enumerate() {
        let normalized = normalize_watchlist_symbol(symbol)?;
        if !seen.insert(normalized.clone()) {
            skipped_duplicate_count += 1;
            results.push(json!({
                "input_index": input_index,
                "symbol": normalized,
                "status": "skipped_duplicate",
            }));
            continue;
        }

        processed_count += 1;
        match watchlist_add(runtime, &normalized).await {
            Ok(data) => {
                let action = data
                    .get("action")
                    .and_then(Value::as_str)
                    .unwrap_or("added");
                let status = if action == "already_present" {
                    already_present_count += 1;
                    "already_present"
                } else {
                    added_count += 1;
                    "added"
                };
                results.push(json!({
                    "input_index": input_index,
                    "symbol": normalized,
                    "status": status,
                    "data": data,
                }));
            }
            Err(error) => {
                failed_count += 1;
                results.push(json!({
                    "input_index": input_index,
                    "symbol": normalized,
                    "status": "failed",
                    "error": {
                        "kind": error.kind,
                        "message": error.message,
                        "details": error.details,
                    },
                }));
            }
        }

        if delay_ms > 0 && processed_count < unique_total {
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }
    }

    let payload = json!({
        "action": "bulk_add",
        "requested_count": symbols.len(),
        "processed_count": processed_count,
        "added_count": added_count,
        "already_present_count": already_present_count,
        "failed_count": failed_count,
        "skipped_duplicate_count": skipped_duplicate_count,
        "delay_ms": delay_ms,
        "allow_partial": allow_partial,
        "results": results,
    });

    if failed_count > 0 && !allow_partial {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!("Watchlist bulk add failed for {failed_count} symbol(s)"),
        )
        .with_details(payload));
    }

    Ok(payload)
}

fn normalize_watchlist_symbol(symbol: &str) -> Result<String, AppError> {
    let symbol = symbol.trim();
    if symbol.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Symbol must not be empty",
        ));
    }
    Ok(symbol.to_string())
}

pub async fn watchlist_remove(
    runtime: &mut impl RuntimeEvaluator,
    symbol: &str,
) -> Result<Value, AppError> {
    let symbol = symbol.trim();
    if symbol.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Symbol must not be empty",
        ));
    }

    let panel_state = ensure_watchlist_panel_open(runtime).await?;
    wait_after_panel_open(runtime, &panel_state).await?;

    let symbol_literal = js_string(symbol)?;
    let mut expression = format!(
        r#"
            (async function() {{
                const requestedSymbol = {};
        "#,
        symbol_literal
    );
    expression.push_str(
        r#"
                function sleep(ms) {
                    return new Promise(function(resolve) { setTimeout(resolve, ms); });
                }

                function isVisible(element) {
                    if (!element) return false;
                    const rect = element.getBoundingClientRect();
                    return rect.width > 0 && rect.height > 0 && window.getComputedStyle(element).visibility !== 'hidden';
                }

                function textOf(element) {
                    return [
                        element.getAttribute('aria-label') || '',
                        element.getAttribute('title') || '',
                        element.getAttribute('data-name') || '',
                        String(element.className || ''),
                        element.textContent || ''
                    ].join(' ').trim();
                }

                function isRemoveText(text) {
                    return /(remove|delete|削除|リストから削除|ウォッチリストから削除)/i.test(text)
                        && !/(add|追加)/i.test(text);
                }

                function rowForSymbolElement(element) {
                    return element.closest('[role="row"], [class*="row"], [data-role="list-item"], [class*="item"]')
                        || element.closest('tr')
                        || element.parentElement;
                }

                function readRows() {
                    const container = document.querySelector('[class*="layout__area--right"]');
                    if (!container) return { container: null, rows: [] };
                    const seen = {};
                    const rows = Array.from(container.querySelectorAll('[data-symbol-full]'))
                        .map(function(element) {
                            const symbol = element.getAttribute('data-symbol-full');
                            if (!symbol || seen[symbol]) return null;
                            seen[symbol] = true;
                            return { symbol: symbol, element: element, row: rowForSymbolElement(element) };
                        })
                        .filter(function(entry) { return entry && entry.row; });
                    return { container: container, rows: rows };
                }

                function publicRows(rows) {
                    return rows.map(function(entry) { return { symbol: entry.symbol }; });
                }

                async function revealRowControls(row) {
                    const rect = row.getBoundingClientRect();
                    const eventOptions = {
                        bubbles: true,
                        cancelable: true,
                        view: window,
                        clientX: rect.right - 20,
                        clientY: rect.top + (rect.height / 2)
                    };
                    ['mouseenter', 'mouseover', 'mousemove'].forEach(function(type) {
                        row.dispatchEvent(new MouseEvent(type, eventOptions));
                    });
                    await sleep(300);
                }

                function dispatchMouseClick(element) {
                    const rect = element.getBoundingClientRect();
                    const eventOptions = {
                        bubbles: true,
                        cancelable: true,
                        view: window,
                        clientX: rect.left + (rect.width / 2),
                        clientY: rect.top + (rect.height / 2)
                    };
                    ['mousedown', 'mouseup', 'click'].forEach(function(type) {
                        element.dispatchEvent(new MouseEvent(type, eventOptions));
                    });
                    return 'mouse_event';
                }

                function findRemoveButton(row) {
                    const candidates = Array.from(row.querySelectorAll('button, [role="button"], [aria-label], [title], [class*="removeButton"]'));
                    for (let i = 0; i < candidates.length; i++) {
                        const candidate = candidates[i];
                        const className = String(candidate.className || '');
                        const isRowRemoveIcon = /removeButton/.test(className);
                        if (!isVisible(candidate) && !isRowRemoveIcon) continue;
                        if (candidate.disabled || candidate.getAttribute('aria-disabled') === 'true') continue;
                        if (isRemoveText(textOf(candidate))) return candidate;
                    }
                    return null;
                }

                const before = readRows();
                if (!before.container) {
                    return {
                        error: 'Watchlist panel not found',
                        error_kind: 'internal_api_unavailable',
                        symbol: requestedSymbol,
                        requested_symbol: requestedSymbol,
                        source: 'dom_row'
                    };
                }

                const matched = before.rows.find(function(entry) {
                    return entry.symbol === requestedSymbol;
                }) || null;
                if (!matched) {
                    return {
                        error: 'Watchlist symbol not found: ' + requestedSymbol,
                        error_kind: 'validation',
                        symbol: requestedSymbol,
                        requested_symbol: requestedSymbol,
                        source: 'dom_row',
                        before_count: before.rows.length,
                        matched_before: false,
                        symbols_before: publicRows(before.rows)
                    };
                }

                let method = null;
                let clickMethod = null;
                await revealRowControls(matched.row);
                const button = findRemoveButton(matched.row);
                if (button) {
                    clickMethod = dispatchMouseClick(button);
                    method = 'row_remove_button';
                }

                if (!method) {
                    return {
                        error: 'Remove control not found for watchlist symbol: ' + requestedSymbol,
                        error_kind: 'internal_api_unavailable',
                        symbol: requestedSymbol,
                        requested_symbol: requestedSymbol,
                        source: 'dom_row',
                        before_count: before.rows.length,
                        matched_before: true,
                        remove_method: null
                    };
                }

                await sleep(700);
                const after = readRows();
                const matchedAfter = after.rows.some(function(entry) {
                    return entry.symbol === requestedSymbol;
                });

                return {
                    symbol: requestedSymbol,
                    requested_symbol: requestedSymbol,
                    action: 'removed',
                    removed: !matchedAfter,
                    source: 'dom_row',
                    before_count: before.rows.length,
                    after_count: after.rows.length,
                    matched_before: true,
                    matched_after: matchedAfter,
                    remove_method: method,
                    click_method: clickMethod,
                    confirmation_clicked: false
                };
            })()
        "#,
    );

    let result = runtime.evaluate(&expression, true).await?;
    normalize_watchlist_remove_payload(result)
}

fn normalize_watchlist_remove_payload(data: Value) -> Result<Value, AppError> {
    if let Some(message) = data.get("error").and_then(Value::as_str) {
        let kind = match data.get("error_kind").and_then(Value::as_str) {
            Some("validation") => ErrorKind::Validation,
            _ => ErrorKind::InternalApiUnavailable,
        };
        return Err(AppError::new(kind, message.to_string()).with_details(data));
    }

    if !data
        .get("removed")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Watchlist remove did not remove the requested symbol",
        )
        .with_details(data));
    }

    Ok(json!({
        "symbol": data.get("symbol").cloned().unwrap_or(Value::Null),
        "requested_symbol": data.get("requested_symbol").cloned().unwrap_or(Value::Null),
        "action": data.get("action").cloned().unwrap_or_else(|| json!("removed")),
        "removed": true,
        "source": data
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or("dom_row"),
        "before_count": data.get("before_count").cloned().unwrap_or(Value::Null),
        "after_count": data.get("after_count").cloned().unwrap_or(Value::Null),
        "matched_before": data
            .get("matched_before")
            .and_then(Value::as_bool)
            .unwrap_or(true),
        "matched_after": data
            .get("matched_after")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        "remove_method": data.get("remove_method").cloned().unwrap_or(Value::Null),
        "click_method": data.get("click_method").cloned().unwrap_or(Value::Null),
        "confirmation_clicked": data
            .get("confirmation_clicked")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    }))
}

async fn ensure_watchlist_panel_open(
    runtime: &mut impl RuntimeEvaluator,
) -> Result<Value, AppError> {
    let panel_state = runtime
        .evaluate(
            r#"
            (function() {
                function dispatchMouseClick(element) {
                    const rect = element.getBoundingClientRect();
                    const eventOptions = {
                        bubbles: true,
                        cancelable: true,
                        view: window,
                        clientX: rect.left + (rect.width / 2),
                        clientY: rect.top + (rect.height / 2)
                    };
                    ['mousedown', 'mouseup', 'click'].forEach(function(type) {
                        element.dispatchEvent(new MouseEvent(type, eventOptions));
                    });
                    return 'mouse_event';
                }
                function isWatchlistLabel(label) {
                    return /Watchlist/i.test(label || '') || /ウォッチリスト/.test(label || '');
                }

                var rightArea = document.querySelector('[class*="layout__area--right"]');
                if (rightArea && rightArea.offsetWidth >= 50 && rightArea.querySelector('[data-symbol-full]')) {
                    return { opened: false, already_open: true, source: 'visible_watchlist_rows' };
                }

                var buttons = Array.from(document.querySelectorAll('[data-name="base-watchlist-widget-button"], button[aria-label]'));
                var btn = null;
                for (var i = 0; i < buttons.length; i++) {
                    var label = buttons[i].getAttribute('aria-label') || '';
                    if (buttons[i].getAttribute('data-name') === 'base-watchlist-widget-button' || isWatchlistLabel(label)) {
                        btn = buttons[i];
                        break;
                    }
                }
                if (!btn) return { error: 'Watchlist button not found' };
                var isActive = btn.getAttribute('aria-pressed') === 'true'
                    || btn.classList.toString().indexOf('Active') !== -1
                    || btn.classList.toString().indexOf('active') !== -1;
                if (!isActive) {
                    return { opened: true, source: 'watchlist_button', label: btn.getAttribute('aria-label') || null, click_method: dispatchMouseClick(btn) };
                }
                return { opened: false, source: 'watchlist_button', label: btn.getAttribute('aria-label') || null };
            })()
            "#,
            false,
        )
        .await?;

    if let Some(message) = panel_state.get("error").and_then(Value::as_str) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            message.to_string(),
        ));
    }

    Ok(panel_state)
}

async fn wait_after_panel_open(
    runtime: &mut impl RuntimeEvaluator,
    panel_state: &Value,
) -> Result<(), AppError> {
    if panel_state
        .get("opened")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        runtime
            .evaluate(
                "new Promise(function(resolve) { setTimeout(function() { resolve(true); }, 500); })",
                true,
            )
            .await?;
    }
    Ok(())
}

async fn dispatch_key(
    runtime: &mut impl RuntimeEvaluator,
    event_type: KeyEventType,
    key: &'static str,
    code: &'static str,
    windows_virtual_key_code: i64,
) -> Result<(), AppError> {
    runtime
        .dispatch_key_event(KeyEvent {
            event_type,
            key,
            code,
            windows_virtual_key_code,
            modifiers: 0,
        })
        .await
}

pub async fn pane_list(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    var layoutNames = {{
                        "s": "1 chart",
                        "single": "1 chart",
                        "2h": "2 horizontal",
                        "2v": "2 vertical",
                        "2-1": "2 top, 1 bottom",
                        "1-2": "1 top, 2 bottom",
                        "3h": "3 horizontal",
                        "3v": "3 vertical",
                        "3s": "3 custom",
                        "2x2": "2x2 grid",
                        "4": "2x2 grid",
                        "4h": "4 horizontal",
                        "4v": "4 vertical",
                        "4s": "4 custom",
                        "6": "6 charts",
                        "8": "8 charts",
                        "10": "10 charts",
                        "12": "12 charts",
                        "14": "14 charts",
                        "16": "16 charts"
                    }};
                    var cwc = {CHART_WIDGET_COLLECTION};
                    var layoutType = cwc._layoutType;
                    if (typeof layoutType === "object" && layoutType && typeof layoutType.value === "function") layoutType = layoutType.value();
                    var count = cwc.inlineChartsCount;
                    if (typeof count === "object" && count && typeof count.value === "function") count = count.value();

                    var all = cwc.getAll();
                    var panes = [];
                    for (var i = 0; i < all.length; i++) {{
                        try {{
                            var c = all[i];
                            var model = c.model ? c.model() : null;
                            var mainSeries = model ? model.mainSeries() : null;
                            var sym = mainSeries ? mainSeries.symbol() : "unknown";
                            var res = mainSeries ? mainSeries.interval() : null;
                            panes.push({{ index: i, symbol: sym, resolution: res || null }});
                        }} catch(e) {{
                            panes.push({{ index: i, symbol: null, resolution: null, error: e.message }});
                        }}
                    }}

                    var activeChart = {CHART_API};
                    var activeIndex = null;
                    for (var j = 0; j < all.length; j++) {{
                        try {{
                            if (all[j].model && activeChart._chartWidget && all[j] === activeChart._chartWidget) {{
                                activeIndex = j;
                                break;
                            }}
                        }} catch(e) {{}}
                    }}

                    return {{
                        layout: layoutType,
                        layout_name: layoutNames[layoutType] || layoutType,
                        chart_count: count,
                        active_index: activeIndex,
                        panes: panes
                    }};
                }})()
                "#
            ),
            false,
        )
        .await
}

pub async fn pane_layout(
    runtime: &mut impl RuntimeEvaluator,
    layout: &str,
) -> Result<Value, AppError> {
    let layout = parse_pane_layout(layout)?;
    let layout_literal = js_string(layout.code)?;
    let layout_name_literal = js_string(layout.name)?;
    let result = runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    try {{
                        var cwc = {CHART_WIDGET_COLLECTION};
                        if (!cwc || typeof cwc.setLayout !== "function") {{
                            return {{ error: "Chart widget collection setLayout unavailable" }};
                        }}
                        cwc.setLayout({layout_literal});
                        return new Promise(function(resolve) {{
                            setTimeout(function() {{
                                try {{
                                    var layoutType = cwc._layoutType;
                                    if (typeof layoutType === "object" && layoutType && typeof layoutType.value === "function") layoutType = layoutType.value();
                                    var count = cwc.inlineChartsCount;
                                    if (typeof count === "object" && count && typeof count.value === "function") count = count.value();

                                    var all = cwc.getAll();
                                    var panes = [];
                                    for (var i = 0; i < all.length; i++) {{
                                        try {{
                                            var c = all[i];
                                            var model = c.model ? c.model() : null;
                                            var mainSeries = model ? model.mainSeries() : null;
                                            var sym = mainSeries ? mainSeries.symbol() : "unknown";
                                            var res = mainSeries ? mainSeries.interval() : null;
                                            panes.push({{ index: i, symbol: sym, resolution: res || null }});
                                        }} catch(e) {{
                                            panes.push({{ index: i, symbol: null, resolution: null, error: e.message }});
                                        }}
                                    }}

                                    var activeChart = {CHART_API};
                                    var activeIndex = null;
                                    for (var j = 0; j < all.length; j++) {{
                                        try {{
                                            if (all[j].model && activeChart._chartWidget && all[j] === activeChart._chartWidget) {{
                                                activeIndex = j;
                                                break;
                                            }}
                                        }} catch(e) {{}}
                                    }}

                                    resolve({{
                                        layout: {layout_literal},
                                        layout_name: {layout_name_literal},
                                        observed_layout: layoutType,
                                        chart_count: count,
                                        active_index: activeIndex,
                                        panes: panes
                                    }});
                                }} catch(e) {{
                                    resolve({{ error: e && e.message ? e.message : String(e) }});
                                }}
                            }}, 500);
                        }});
                    }} catch(e) {{
                        return {{ error: e && e.message ? e.message : String(e) }};
                    }}
                }})()
                "#
            ),
            true,
        )
        .await?;

    if let Some(message) = result.get("error").and_then(Value::as_str) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            message.to_string(),
        ));
    }

    Ok(result)
}

pub async fn pane_focus(
    runtime: &mut impl RuntimeEvaluator,
    index: usize,
) -> Result<Value, AppError> {
    let result = runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    try {{
                        var cwc = {CHART_WIDGET_COLLECTION};
                        var all = cwc && typeof cwc.getAll === "function" ? cwc.getAll() : [];
                        if ({index} >= all.length) {{
                            return {{ error: "Pane index {index} out of range", total: all.length }};
                        }}
                        var chart = all[{index}];
                        if (chart && chart._mainDiv && typeof chart._mainDiv.click === "function") {{
                            chart._mainDiv.click();
                        }}
                        return {{ focused: {index}, total: all.length }};
                    }} catch(e) {{
                        return {{ error: e && e.message ? e.message : String(e) }};
                    }}
                }})()
                "#
            ),
            false,
        )
        .await?;

    if let Some(message) = result.get("error").and_then(Value::as_str) {
        return Err(
            AppError::new(ErrorKind::InternalApiUnavailable, message.to_string()).with_details(
                json!({
                    "index": index,
                    "total_panes": result.get("total").cloned().unwrap_or(Value::Null),
                }),
            ),
        );
    }

    Ok(json!({
        "focused_index": result.get("focused").cloned().unwrap_or_else(|| json!(index)),
        "total_panes": result.get("total").cloned().unwrap_or(Value::Null),
    }))
}

pub async fn pane_symbol(
    runtime: &mut impl RuntimeEvaluator,
    index: usize,
    symbol: &str,
) -> Result<Value, AppError> {
    let focus = pane_focus(runtime, index).await?;
    runtime
        .evaluate(
            "new Promise(function(resolve) { setTimeout(function() { resolve(true); }, 300); })",
            true,
        )
        .await?;

    let symbol_literal = js_string(symbol)?;
    let result = runtime
        .evaluate(
            &format!(
                r#"
                (function() {{
                    try {{
                        var chart = {CHART_API};
                        if (!chart || typeof chart.setSymbol !== "function") {{
                            return {{ error: "Active chart setSymbol unavailable" }};
                        }}
                        chart.setSymbol({symbol_literal}, {{}});
                        return new Promise(function(resolve) {{
                            setTimeout(function() {{
                                resolve({{
                                    index: {index},
                                    symbol: {symbol_literal},
                                    requested_symbol: {symbol_literal},
                                    source: "active_chart_after_focus"
                                }});
                            }}, 500);
                        }});
                    }} catch(e) {{
                        return {{ error: e && e.message ? e.message : String(e) }};
                    }}
                }})()
                "#
            ),
            true,
        )
        .await?;

    if let Some(message) = result.get("error").and_then(Value::as_str) {
        return Err(
            AppError::new(ErrorKind::InternalApiUnavailable, message.to_string()).with_details(
                json!({
                    "index": index,
                    "symbol": symbol,
                }),
            ),
        );
    }

    Ok(json!({
        "index": result.get("index").cloned().unwrap_or_else(|| json!(index)),
        "symbol": result.get("symbol").cloned().unwrap_or_else(|| json!(symbol)),
        "requested_symbol": result.get("requested_symbol").cloned().unwrap_or_else(|| json!(symbol)),
        "source": result.get("source").cloned().unwrap_or_else(|| json!("active_chart_after_focus")),
        "focused_index": focus.get("focused_index").cloned().unwrap_or_else(|| json!(index)),
        "total_panes": focus.get("total_panes").cloned().unwrap_or(Value::Null),
    }))
}

#[cfg(test)]
mod tests {
    use crate::cdp::KeyEventType;
    use serde_json::json;

    use super::super::test_support::FakeRuntime;
    use super::*;

    #[tokio::test]
    async fn watchlist_get_returns_runtime_payload() {
        let payload = json!({
            "count": 1,
            "source": "data_attributes",
            "symbols": [{"symbol": "NASDAQ:AAPL", "last": "100", "change": "1", "change_percent": "1%"}]
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);

        let result = watchlist_get(&mut runtime).await.unwrap();

        assert_eq!(result, payload);
        assert!(runtime.evaluated[0].0.contains("data-symbol-full"));
    }

    #[tokio::test]
    async fn watchlist_add_opens_panel_clicks_add_and_sends_input() {
        let mut runtime = FakeRuntime::new([
            json!({"opened": true, "click_method": "mouse_event"}),
            json!(true),
            json!({
                "found": true,
                "skipped": false,
                "selector": "[data-name=\"add-symbol-button\"]",
                "before_count": 0,
                "matched_before": false,
                "click_method": "mouse_event"
            }),
            json!(true),
            json!(true),
            json!(true),
            json!({"after_count": 1, "matched_after": true}),
        ]);

        let result = watchlist_add(&mut runtime, "NASDAQ:AAPL").await.unwrap();

        assert_eq!(result["symbol"], "NASDAQ:AAPL");
        assert_eq!(result["action"], "added");
        assert_eq!(result["before_count"], 0);
        assert_eq!(result["after_count"], 1);
        assert_eq!(result["matched_before"], false);
        assert_eq!(result["matched_after"], true);
        assert_eq!(result["click_method"], "mouse_event");
        assert_eq!(runtime.inserted_text, vec!["NASDAQ:AAPL"]);
        assert_eq!(runtime.key_events.len(), 4);
        assert_eq!(runtime.key_events[0].event_type, KeyEventType::KeyDown);
        assert_eq!(runtime.key_events[0].key, "Enter");
        assert_eq!(runtime.key_events[1].event_type, KeyEventType::KeyUp);
        assert_eq!(runtime.key_events[2].key, "Escape");
        assert!(
            runtime.evaluated[0]
                .0
                .contains("base-watchlist-widget-button")
        );
        assert!(runtime.evaluated[2].0.contains("add-symbol-button"));
        assert!(runtime.evaluated[2].0.contains("new MouseEvent"));
        assert!(!runtime.evaluated[2].0.contains(".click()"));
    }

    #[tokio::test]
    async fn watchlist_add_continues_when_watchlist_rows_are_already_visible() {
        let mut runtime = FakeRuntime::new([
            json!({"opened": false, "already_open": true, "source": "visible_watchlist_rows"}),
            json!({
                "found": true,
                "skipped": false,
                "method": "fallback",
                "before_count": 2,
                "matched_before": false,
                "click_method": "mouse_event"
            }),
            json!(true),
            json!(true),
            json!(true),
            json!({"after_count": 3, "matched_after": true}),
        ]);

        let result = watchlist_add(&mut runtime, "NASDAQ:AAPL").await.unwrap();

        assert_eq!(result["symbol"], "NASDAQ:AAPL");
        assert_eq!(result["opened_panel"], false);
        assert_eq!(result["before_count"], 2);
        assert_eq!(result["after_count"], 3);
        assert_eq!(runtime.inserted_text, vec!["NASDAQ:AAPL"]);
        assert_eq!(runtime.key_events.len(), 4);
    }

    #[tokio::test]
    async fn watchlist_add_returns_already_present_without_input() {
        let mut runtime = FakeRuntime::new([
            json!({"opened": false, "already_open": true, "source": "visible_watchlist_rows"}),
            json!({
                "found": true,
                "skipped": true,
                "action": "already_present",
                "before_count": 1,
                "after_count": 1,
                "matched_before": true,
                "matched_after": true,
                "click_method": null
            }),
        ]);

        let result = watchlist_add(&mut runtime, "NASDAQ:AAPL").await.unwrap();

        assert_eq!(result["action"], "already_present");
        assert_eq!(result["matched_before"], true);
        assert_eq!(result["matched_after"], true);
        assert!(runtime.inserted_text.is_empty());
        assert!(runtime.key_events.is_empty());
    }

    #[tokio::test]
    async fn watchlist_add_requires_post_add_confirmation() {
        let mut runtime = FakeRuntime::new([
            json!({"opened": false, "already_open": true, "source": "visible_watchlist_rows"}),
            json!({
                "found": true,
                "skipped": false,
                "selector": "[data-name=\"add-symbol-button\"]",
                "before_count": 0,
                "matched_before": false,
                "click_method": "mouse_event"
            }),
            json!(true),
            json!(true),
            json!(true),
            json!({"after_count": 0, "matched_after": false}),
        ]);

        let err = watchlist_add(&mut runtime, "NASDAQ:AAPL")
            .await
            .unwrap_err();

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(
            err.details.as_ref().unwrap()["verify"]["matched_after"],
            false
        );
    }

    #[tokio::test]
    async fn watchlist_add_maps_missing_watchlist_ui_to_internal_api_error() {
        let mut runtime = FakeRuntime::new([json!({"error": "Watchlist button not found"})]);

        let err = watchlist_add(&mut runtime, "NASDAQ:AAPL")
            .await
            .unwrap_err();

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
        assert!(runtime.inserted_text.is_empty());
    }

    #[tokio::test]
    async fn watchlist_add_maps_missing_add_button_to_internal_api_error() {
        let mut runtime = FakeRuntime::new([json!({"opened": false}), json!({"found": false})]);

        let err = watchlist_add(&mut runtime, "NASDAQ:AAPL")
            .await
            .unwrap_err();

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
        assert!(runtime.inserted_text.is_empty());
    }

    #[tokio::test]
    async fn watchlist_add_bulk_aggregates_added_present_and_duplicates() {
        let mut runtime = FakeRuntime::new([
            json!({"opened": false, "already_open": true, "source": "visible_watchlist_rows"}),
            json!({
                "found": true,
                "skipped": false,
                "selector": "[data-name=\"add-symbol-button\"]",
                "before_count": 0,
                "matched_before": false,
                "click_method": "mouse_event"
            }),
            json!(true),
            json!(true),
            json!(true),
            json!({"after_count": 1, "matched_after": true}),
            json!({"opened": false, "already_open": true, "source": "visible_watchlist_rows"}),
            json!({
                "found": true,
                "skipped": true,
                "action": "already_present",
                "before_count": 1,
                "after_count": 1,
                "matched_before": true,
                "matched_after": true,
                "click_method": null
            }),
        ]);
        let symbols = vec![
            "NASDAQ:AAPL".to_string(),
            "NASDAQ:AAPL".to_string(),
            "NASDAQ:MSFT".to_string(),
        ];

        let result = watchlist_add_bulk(&mut runtime, &symbols, 0, false)
            .await
            .unwrap();

        assert_eq!(result["action"], "bulk_add");
        assert_eq!(result["requested_count"], 3);
        assert_eq!(result["processed_count"], 2);
        assert_eq!(result["added_count"], 1);
        assert_eq!(result["already_present_count"], 1);
        assert_eq!(result["failed_count"], 0);
        assert_eq!(result["skipped_duplicate_count"], 1);
        assert_eq!(result["results"][0]["status"], "added");
        assert_eq!(result["results"][1]["status"], "skipped_duplicate");
        assert_eq!(result["results"][2]["status"], "already_present");
        assert_eq!(runtime.inserted_text, vec!["NASDAQ:AAPL"]);
    }

    #[tokio::test]
    async fn watchlist_add_bulk_returns_partial_payload_when_allowed() {
        let mut runtime = FakeRuntime::new([json!({"opened": false}), json!({"found": false})]);
        let symbols = vec!["NASDAQ:AAPL".to_string()];

        let result = watchlist_add_bulk(&mut runtime, &symbols, 0, true)
            .await
            .unwrap();

        assert_eq!(result["failed_count"], 1);
        assert_eq!(result["results"][0]["status"], "failed");
        assert_eq!(
            result["results"][0]["error"]["kind"],
            json!("internal_api_unavailable")
        );
    }

    #[tokio::test]
    async fn watchlist_add_bulk_fails_strictly_after_attempting_symbols() {
        let mut runtime = FakeRuntime::new([json!({"opened": false}), json!({"found": false})]);
        let symbols = vec!["NASDAQ:AAPL".to_string()];

        let err = watchlist_add_bulk(&mut runtime, &symbols, 0, false)
            .await
            .unwrap_err();

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
        let details = err.details.unwrap();
        assert_eq!(details["failed_count"], 1);
        assert_eq!(details["results"][0]["status"], "failed");
    }

    #[test]
    fn watchlist_add_bulk_validates_inputs_before_connecting() {
        assert_eq!(
            validate_watchlist_add_bulk_request(&[], 0)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_watchlist_add_bulk_request(&[" ".to_string()], 0)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
        assert_eq!(
            validate_watchlist_add_bulk_request(
                &["NASDAQ:AAPL".to_string()],
                MAX_WATCHLIST_BULK_DELAY_MS + 1,
            )
            .unwrap_err()
            .kind,
            ErrorKind::Validation
        );
        let too_many = (0..=MAX_WATCHLIST_BULK_SYMBOLS)
            .map(|index| format!("NASDAQ:TEST{index}"))
            .collect::<Vec<_>>();
        assert_eq!(
            validate_watchlist_add_bulk_request(&too_many, 0)
                .unwrap_err()
                .kind,
            ErrorKind::Validation
        );
    }

    #[tokio::test]
    async fn watchlist_remove_removes_exact_symbol() {
        let mut runtime = FakeRuntime::new([
            json!({"opened": false, "already_open": true, "source": "visible_watchlist_rows"}),
            json!({
                "symbol": "NASDAQ:AAPL",
                "requested_symbol": "NASDAQ:AAPL",
                "action": "removed",
                "removed": true,
                "source": "dom_row",
                "before_count": 2,
                "after_count": 1,
                "matched_before": true,
                "matched_after": false,
                "remove_method": "row_remove_button",
                "click_method": "mouse_event",
                "confirmation_clicked": false
            }),
        ]);

        let result = watchlist_remove(&mut runtime, "NASDAQ:AAPL").await.unwrap();

        assert_eq!(result["symbol"], "NASDAQ:AAPL");
        assert_eq!(result["requested_symbol"], "NASDAQ:AAPL");
        assert_eq!(result["action"], "removed");
        assert_eq!(result["removed"], true);
        assert_eq!(result["before_count"], 2);
        assert_eq!(result["after_count"], 1);
        assert_eq!(result["remove_method"], "row_remove_button");
        assert_eq!(result["click_method"], "mouse_event");
        assert!(
            runtime.evaluated[1]
                .0
                .contains("entry.symbol === requestedSymbol")
        );
        assert!(runtime.evaluated[1].0.contains("removeButton"));
        assert!(runtime.evaluated[1].0.contains("new MouseEvent"));
        assert!(!runtime.evaluated[1].0.contains("button.click"));
        assert!(!runtime.evaluated[1].0.contains("contextmenu"));
    }

    #[tokio::test]
    async fn watchlist_remove_rejects_empty_symbol_before_evaluating() {
        let mut runtime = FakeRuntime::new([]);

        let err = watchlist_remove(&mut runtime, " ").await.unwrap_err();

        assert_eq!(err.kind, ErrorKind::Validation);
        assert!(runtime.evaluated.is_empty());
    }

    #[tokio::test]
    async fn watchlist_remove_maps_absent_symbol_to_validation() {
        let mut runtime = FakeRuntime::new([
            json!({"opened": false, "already_open": true, "source": "visible_watchlist_rows"}),
            json!({
                "error": "Watchlist symbol not found: NASDAQ:AAPL",
                "error_kind": "validation",
                "symbol": "NASDAQ:AAPL",
                "requested_symbol": "NASDAQ:AAPL",
                "source": "dom_row",
                "before_count": 1,
                "matched_before": false,
                "symbols_before": [{"symbol": "NASDAQ:MSFT"}]
            }),
        ]);

        let err = watchlist_remove(&mut runtime, "NASDAQ:AAPL")
            .await
            .unwrap_err();

        assert_eq!(err.kind, ErrorKind::Validation);
        assert_eq!(
            err.details.as_ref().unwrap()["symbols_before"][0]["symbol"],
            "NASDAQ:MSFT"
        );
    }

    #[tokio::test]
    async fn watchlist_remove_maps_missing_remove_control_to_internal_api_error() {
        let mut runtime = FakeRuntime::new([
            json!({"opened": false, "already_open": true, "source": "visible_watchlist_rows"}),
            json!({
                "error": "Remove control not found for watchlist symbol: NASDAQ:AAPL",
                "error_kind": "internal_api_unavailable",
                "symbol": "NASDAQ:AAPL",
                "requested_symbol": "NASDAQ:AAPL",
                "source": "dom_row",
                "before_count": 1,
                "matched_before": true,
                "remove_method": null
            }),
        ]);

        let err = watchlist_remove(&mut runtime, "NASDAQ:AAPL")
            .await
            .unwrap_err();

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(err.details.as_ref().unwrap()["matched_before"], true);
    }

    #[tokio::test]
    async fn watchlist_remove_fails_when_symbol_remains_after_delete() {
        let mut runtime = FakeRuntime::new([
            json!({"opened": false, "already_open": true, "source": "visible_watchlist_rows"}),
            json!({
                "symbol": "NASDAQ:AAPL",
                "requested_symbol": "NASDAQ:AAPL",
                "action": "removed",
                "removed": false,
                "source": "dom_row",
                "before_count": 1,
                "after_count": 1,
                "matched_before": true,
                "matched_after": true,
                "remove_method": "context_menu",
                "confirmation_clicked": false
            }),
        ]);

        let err = watchlist_remove(&mut runtime, "NASDAQ:AAPL")
            .await
            .unwrap_err();

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(err.details.as_ref().unwrap()["matched_after"], true);
    }

    #[tokio::test]
    async fn pane_list_returns_runtime_payload() {
        let payload = json!({
            "layout": "single",
            "layout_name": "1 chart",
            "chart_count": 1,
            "active_index": 0,
            "panes": [{"index": 0, "symbol": "NASDAQ:AAPL", "resolution": "D"}]
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);

        let result = pane_list(&mut runtime).await.unwrap();

        assert_eq!(result, payload);
        assert!(runtime.evaluated[0].0.contains("_chartWidgetCollection"));
    }

    #[test]
    fn validate_pane_layout_accepts_aliases() {
        assert!(validate_pane_layout("2x2").is_ok());
        assert_eq!(parse_pane_layout("2x2").unwrap().code, "4");
        assert_eq!(parse_pane_layout(" single ").unwrap().code, "s");
    }

    #[test]
    fn validate_pane_layout_rejects_unknown_layout() {
        let err = validate_pane_layout("banana").unwrap_err();

        assert_eq!(err.kind, ErrorKind::Validation);
        assert!(
            err.details
                .as_ref()
                .and_then(|details| details.get("supported"))
                .and_then(Value::as_array)
                .is_some_and(|supported| supported.contains(&json!({
                    "layout": "4",
                    "layout_name": "2x2 grid"
                })))
        );
    }

    #[tokio::test]
    async fn pane_layout_sets_canonical_layout_and_returns_runtime_payload() {
        let mut runtime = FakeRuntime::new([json!({
            "layout": "4",
            "layout_name": "2x2 grid",
            "observed_layout": "4",
            "chart_count": 4,
            "active_index": 0,
            "panes": []
        })]);

        let result = pane_layout(&mut runtime, "2x2").await.unwrap();

        assert_eq!(result["layout"], "4");
        assert_eq!(result["layout_name"], "2x2 grid");
        assert!(runtime.evaluated[0].0.contains("setLayout(\"4\")"));
        assert!(runtime.evaluated[0].1);
    }

    #[tokio::test]
    async fn pane_layout_maps_runtime_error_to_internal_api_error() {
        let mut runtime = FakeRuntime::new([json!({
            "error": "Chart widget collection setLayout unavailable"
        })]);

        let err = pane_layout(&mut runtime, "s").await.unwrap_err();

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
    }

    #[tokio::test]
    async fn pane_focus_returns_practical_old_cli_fields() {
        let mut runtime = FakeRuntime::new([json!({"focused": 1, "total": 2})]);

        let result = pane_focus(&mut runtime, 1).await.unwrap();

        assert_eq!(result["focused_index"], 1);
        assert_eq!(result["total_panes"], 2);
        assert!(runtime.evaluated[0].0.contains("_mainDiv.click"));
    }

    #[tokio::test]
    async fn pane_focus_maps_range_error_to_internal_api_error() {
        let mut runtime = FakeRuntime::new([json!({
            "error": "Pane index 3 out of range",
            "total": 1
        })]);

        let err = pane_focus(&mut runtime, 3).await.unwrap_err();

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(err.details.unwrap()["total_panes"], 1);
    }

    #[tokio::test]
    async fn pane_symbol_focuses_then_sets_symbol() {
        let mut runtime = FakeRuntime::new([
            json!({"focused": 1, "total": 2}),
            json!(true),
            json!({
                "index": 1,
                "symbol": "NASDAQ:AAPL",
                "requested_symbol": "NASDAQ:AAPL",
                "source": "active_chart_after_focus"
            }),
        ]);

        let result = pane_symbol(&mut runtime, 1, "NASDAQ:AAPL").await.unwrap();

        assert_eq!(result["index"], 1);
        assert_eq!(result["symbol"], "NASDAQ:AAPL");
        assert_eq!(result["focused_index"], 1);
        assert_eq!(result["total_panes"], 2);
        assert!(runtime.evaluated[0].0.contains("_mainDiv.click"));
        assert!(runtime.evaluated[2].0.contains("setSymbol(\"NASDAQ:AAPL\""));
    }

    #[tokio::test]
    async fn pane_symbol_serializes_symbol_as_js_string() {
        let mut runtime = FakeRuntime::new([
            json!({"focused": 0, "total": 1}),
            json!(true),
            json!({"index": 0, "symbol": "NYSE:BRK\"B"}),
        ]);

        pane_symbol(&mut runtime, 0, "NYSE:BRK\"B").await.unwrap();

        assert!(
            runtime.evaluated[2]
                .0
                .contains("setSymbol(\"NYSE:BRK\\\"B\"")
        );
    }
}
