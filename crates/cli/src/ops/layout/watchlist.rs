use std::time::Duration;

use serde_json::{Value, json};

use tradingview_cdp::{KeyEvent, KeyEventType, RuntimeEvaluator};
use tradingview_core::{AppError, ErrorKind};

pub use tradingview_model::watchlist::validate_watchlist_add_bulk_request;
use tradingview_model::watchlist::{
    WatchlistBulkAccumulator, normalize_watchlist_api_payload, normalize_watchlist_remove_payload,
    normalize_watchlist_symbol, unique_watchlist_symbol_count, watchlist_api_error_allows_fallback,
};

use super::super::common::js_string;

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

    match watchlist_add_via_api(runtime, symbol).await {
        Ok(data) => return Ok(data),
        Err(error) if watchlist_api_error_allows_fallback(&error) => {}
        Err(error) => return Err(error),
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

pub async fn watchlist_add_bulk(
    runtime: &mut impl RuntimeEvaluator,
    symbols: &[String],
    delay_ms: u64,
    allow_partial: bool,
) -> Result<Value, AppError> {
    validate_watchlist_add_bulk_request(symbols, delay_ms)?;

    let unique_total = unique_watchlist_symbol_count(symbols)?;
    let mut accumulator = WatchlistBulkAccumulator::new(symbols.len(), delay_ms, allow_partial);

    for (input_index, symbol) in symbols.iter().enumerate() {
        let normalized = normalize_watchlist_symbol(symbol)?;
        if !accumulator.mark_seen_or_duplicate(input_index, &normalized) {
            continue;
        }

        match watchlist_add(runtime, &normalized).await {
            Ok(data) => accumulator.record_success(input_index, &normalized, data),
            Err(error) => accumulator.record_failure(input_index, &normalized, error),
        }

        if delay_ms > 0 && accumulator.processed_count() < unique_total {
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }
    }

    let failed_count = accumulator.failed_count();
    let payload = accumulator.payload();
    if failed_count > 0 && !allow_partial {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!("Watchlist bulk add failed for {failed_count} symbol(s)"),
        )
        .with_details(payload));
    }

    Ok(payload)
}

pub async fn watchlist_add_via_api(
    runtime: &mut impl RuntimeEvaluator,
    symbol: &str,
) -> Result<Value, AppError> {
    watchlist_mutate_via_api(runtime, symbol, "add").await
}

pub async fn watchlist_remove_via_api(
    runtime: &mut impl RuntimeEvaluator,
    symbol: &str,
) -> Result<Value, AppError> {
    watchlist_mutate_via_api(runtime, symbol, "remove").await
}

async fn watchlist_mutate_via_api(
    runtime: &mut impl RuntimeEvaluator,
    symbol: &str,
    action: &str,
) -> Result<Value, AppError> {
    let symbol = normalize_watchlist_symbol(symbol)?;
    let action_literal = js_string(action)?;
    let symbol_literal = js_string(&symbol)?;
    let expression = format!(
        r#"
        (async function() {{
            const requestedSymbol = {symbol_literal};
            const requestedAction = {action_literal};
            const source = 'watchlist_api';

            function sleep(ms) {{
                return new Promise(function(resolve) {{ setTimeout(resolve, ms); }});
            }}

            function safeListName(list) {{
                return list && (list.name || list.color || null);
            }}

            function publicList(list) {{
                if (!list) return null;
                return {{
                    name: safeListName(list),
                    type: list.type || null,
                    color: list.color || null,
                    symbol_count: Array.isArray(list.symbols) ? list.symbols.length : null,
                    active: !!list.active
                }};
            }}

            function hasSymbol(list, symbol) {{
                return Array.isArray(list && list.symbols) && list.symbols.indexOf(symbol) >= 0;
            }}

            async function fetchLists() {{
                let response;
                let text;
                try {{
                    response = await fetch('/api/v1/symbols_list/all/?source=web-tvd', {{
                        credentials: 'include'
                    }});
                    text = await response.text();
                }} catch (error) {{
                    return {{
                        error: error && error.message ? error.message : String(error),
                        error_kind: 'internal_api_unavailable',
                        phase: 'list_unavailable',
                        api_fallback_allowed: true,
                        source
                    }};
                }}

                let parsed = null;
                try {{
                    parsed = text ? JSON.parse(text) : null;
                }} catch (error) {{
                    return {{
                        error: 'Watchlist API list response was not JSON',
                        error_kind: 'internal_api_unavailable',
                        phase: 'list_unavailable',
                        status: response.status,
                        api_fallback_allowed: true,
                        source
                    }};
                }}

                if (!response.ok || !Array.isArray(parsed)) {{
                    return {{
                        error: 'Watchlist API list request failed',
                        error_kind: 'internal_api_unavailable',
                        phase: 'list_unavailable',
                        status: response.status,
                        api_fallback_allowed: true,
                        source
                    }};
                }}

                return {{ lists: parsed }};
            }}

            function activeList(lists) {{
                return (lists || []).find(function(list) {{ return list && list.active; }}) || null;
            }}

            function sameList(candidate, previous) {{
                return candidate && previous && String(candidate.id) === String(previous.id);
            }}

            const beforeResult = await fetchLists();
            if (beforeResult.error) return beforeResult;

            const beforeActive = activeList(beforeResult.lists);
            if (!beforeActive) {{
                return {{
                    error: 'No active watchlist found in API response',
                    error_kind: 'internal_api_unavailable',
                    phase: 'active_list_missing',
                    api_fallback_allowed: true,
                    source
                }};
            }}

            if (beforeActive.type !== 'custom' || beforeActive.id == null) {{
                return {{
                    error: 'Active watchlist is not a custom API list',
                    error_kind: 'internal_api_unavailable',
                    phase: 'active_list_unsupported',
                    active_list: publicList(beforeActive),
                    api_fallback_allowed: true,
                    source
                }};
            }}

            const beforeCount = Array.isArray(beforeActive.symbols) ? beforeActive.symbols.length : null;
            const matchedBefore = hasSymbol(beforeActive, requestedSymbol);
            if (requestedAction === 'add' && matchedBefore) {{
                return {{
                    symbol: requestedSymbol,
                    requested_symbol: requestedSymbol,
                    action: 'already_present',
                    source,
                    target_list: publicList(beforeActive),
                    before_count: beforeCount,
                    after_count: beforeCount,
                    matched_before: true,
                    matched_after: true
                }};
            }}
            if (requestedAction === 'remove' && !matchedBefore) {{
                return {{
                    error: 'Watchlist symbol not found: ' + requestedSymbol,
                    error_kind: 'validation',
                    phase: 'precheck_absent',
                    symbol: requestedSymbol,
                    requested_symbol: requestedSymbol,
                    source,
                    target_list: publicList(beforeActive),
                    before_count: beforeCount,
                    matched_before: false,
                    api_fallback_allowed: false
                }};
            }}

            const endpointAction = requestedAction === 'add' ? 'append' : 'remove';
            const endpoint = '/api/v1/symbols_list/custom/' +
                encodeURIComponent(String(beforeActive.id)) + '/' + endpointAction + '/?source=web-tvd';
            let mutationResponse;
            let mutationText;
            try {{
                mutationResponse = await fetch(endpoint, {{
                    method: 'POST',
                    credentials: 'include',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify([requestedSymbol])
                }});
                mutationText = await mutationResponse.text();
            }} catch (error) {{
                return {{
                    error: error && error.message ? error.message : String(error),
                    error_kind: 'internal_api_unavailable',
                    phase: 'mutation_unavailable',
                    symbol: requestedSymbol,
                    requested_symbol: requestedSymbol,
                    source,
                    target_list: publicList(beforeActive),
                    before_count: beforeCount,
                    matched_before: matchedBefore,
                    api_fallback_allowed: true
                }};
            }}

            if (!mutationResponse.ok) {{
                return {{
                    error: 'Watchlist API mutation failed',
                    error_kind: 'internal_api_unavailable',
                    phase: 'mutation_unavailable',
                    status: mutationResponse.status,
                    body_excerpt: String(mutationText || '').slice(0, 120),
                    symbol: requestedSymbol,
                    requested_symbol: requestedSymbol,
                    source,
                    target_list: publicList(beforeActive),
                    before_count: beforeCount,
                    matched_before: matchedBefore,
                    api_fallback_allowed: true
                }};
            }}

            await sleep(250);
            const afterResult = await fetchLists();
            if (afterResult.error) {{
                afterResult.phase = 'post_check_unavailable';
                afterResult.api_fallback_allowed = false;
                afterResult.symbol = requestedSymbol;
                afterResult.requested_symbol = requestedSymbol;
                afterResult.target_list = publicList(beforeActive);
                afterResult.before_count = beforeCount;
                afterResult.matched_before = matchedBefore;
                return afterResult;
            }}

            const afterActive = (afterResult.lists || []).find(function(list) {{
                return sameList(list, beforeActive);
            }}) || activeList(afterResult.lists);
            const afterCount = Array.isArray(afterActive && afterActive.symbols) ? afterActive.symbols.length : null;
            const matchedAfter = hasSymbol(afterActive, requestedSymbol);

            if (requestedAction === 'add' && !matchedAfter) {{
                return {{
                    error: 'Watchlist API add did not confirm symbol after mutation: ' + requestedSymbol,
                    error_kind: 'internal_api_unavailable',
                    phase: 'post_check_failed',
                    symbol: requestedSymbol,
                    requested_symbol: requestedSymbol,
                    source,
                    target_list: publicList(beforeActive),
                    before_count: beforeCount,
                    after_count: afterCount,
                    matched_before: matchedBefore,
                    matched_after: matchedAfter,
                    api_fallback_allowed: false
                }};
            }}
            if (requestedAction === 'remove' && matchedAfter) {{
                return {{
                    error: 'Watchlist API remove did not remove the requested symbol: ' + requestedSymbol,
                    error_kind: 'internal_api_unavailable',
                    phase: 'post_check_failed',
                    symbol: requestedSymbol,
                    requested_symbol: requestedSymbol,
                    source,
                    target_list: publicList(beforeActive),
                    before_count: beforeCount,
                    after_count: afterCount,
                    matched_before: matchedBefore,
                    matched_after: matchedAfter,
                    api_fallback_allowed: false
                }};
            }}

            if (requestedAction === 'add') {{
                return {{
                    symbol: requestedSymbol,
                    requested_symbol: requestedSymbol,
                    action: 'added',
                    source,
                    target_list: publicList(beforeActive),
                    before_count: beforeCount,
                    after_count: afterCount,
                    matched_before: matchedBefore,
                    matched_after: true,
                    click_method: null
                }};
            }}

            return {{
                symbol: requestedSymbol,
                requested_symbol: requestedSymbol,
                action: 'removed',
                removed: true,
                source,
                target_list: publicList(beforeActive),
                before_count: beforeCount,
                after_count: afterCount,
                matched_before: true,
                matched_after: false,
                remove_method: 'api',
                click_method: null,
                confirmation_clicked: false
            }};
        }})()
        "#
    );

    let result = runtime.evaluate(&expression, true).await?;
    normalize_watchlist_api_payload(result)
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

    match watchlist_remove_via_api(runtime, symbol).await {
        Ok(data) => return Ok(data),
        Err(error) if watchlist_api_error_allows_fallback(&error) => {}
        Err(error) => return Err(error),
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

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tradingview_cdp::KeyEventType;

    use super::super::super::test_support::FakeRuntime;
    use super::*;

    fn watchlist_api_fallback() -> serde_json::Value {
        json!({
            "error": "Watchlist API unavailable in test",
            "error_kind": "internal_api_unavailable",
            "phase": "list_unavailable",
            "api_fallback_allowed": true,
            "source": "watchlist_api"
        })
    }

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
            watchlist_api_fallback(),
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
            runtime.evaluated[1]
                .0
                .contains("base-watchlist-widget-button")
        );
        assert!(runtime.evaluated[3].0.contains("add-symbol-button"));
        assert!(runtime.evaluated[3].0.contains("new MouseEvent"));
        assert!(!runtime.evaluated[3].0.contains(".click()"));
    }

    #[tokio::test]
    async fn watchlist_add_uses_api_when_post_check_confirms_symbol() {
        let mut runtime = FakeRuntime::new([json!({
            "symbol": "NASDAQ:AAPL",
            "requested_symbol": "NASDAQ:AAPL",
            "action": "added",
            "source": "watchlist_api",
            "target_list": {"name": "Test", "type": "custom", "symbol_count": 1, "active": true},
            "before_count": 1,
            "after_count": 2,
            "matched_before": false,
            "matched_after": true,
            "click_method": null
        })]);

        let result = watchlist_add(&mut runtime, "NASDAQ:AAPL").await.unwrap();

        assert_eq!(result["action"], "added");
        assert_eq!(result["source"], "watchlist_api");
        assert_eq!(result["before_count"], 1);
        assert_eq!(result["after_count"], 2);
        assert!(runtime.inserted_text.is_empty());
        assert!(runtime.key_events.is_empty());
        assert!(runtime.evaluated[0].0.contains("symbols_list/all"));
        assert!(runtime.evaluated[0].0.contains("'append'"));
    }

    #[tokio::test]
    async fn watchlist_add_api_returns_already_present_without_dom_input() {
        let mut runtime = FakeRuntime::new([json!({
            "symbol": "NASDAQ:AAPL",
            "requested_symbol": "NASDAQ:AAPL",
            "action": "already_present",
            "source": "watchlist_api",
            "before_count": 2,
            "after_count": 2,
            "matched_before": true,
            "matched_after": true
        })]);

        let result = watchlist_add(&mut runtime, "NASDAQ:AAPL").await.unwrap();

        assert_eq!(result["action"], "already_present");
        assert_eq!(result["source"], "watchlist_api");
        assert!(runtime.inserted_text.is_empty());
        assert!(runtime.key_events.is_empty());
    }

    #[tokio::test]
    async fn watchlist_add_continues_when_watchlist_rows_are_already_visible() {
        let mut runtime = FakeRuntime::new([
            watchlist_api_fallback(),
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
            watchlist_api_fallback(),
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
            watchlist_api_fallback(),
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
        let mut runtime = FakeRuntime::new([
            watchlist_api_fallback(),
            json!({"error": "Watchlist button not found"}),
        ]);

        let err = watchlist_add(&mut runtime, "NASDAQ:AAPL")
            .await
            .unwrap_err();

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
        assert!(runtime.inserted_text.is_empty());
    }

    #[tokio::test]
    async fn watchlist_add_maps_missing_add_button_to_internal_api_error() {
        let mut runtime = FakeRuntime::new([
            watchlist_api_fallback(),
            json!({"opened": false}),
            json!({"found": false}),
        ]);

        let err = watchlist_add(&mut runtime, "NASDAQ:AAPL")
            .await
            .unwrap_err();

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
        assert!(runtime.inserted_text.is_empty());
    }

    #[tokio::test]
    async fn watchlist_add_bulk_aggregates_added_present_and_duplicates() {
        let mut runtime = FakeRuntime::new([
            watchlist_api_fallback(),
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
            watchlist_api_fallback(),
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
        let mut runtime = FakeRuntime::new([
            watchlist_api_fallback(),
            json!({"opened": false}),
            json!({"found": false}),
        ]);
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
        let mut runtime = FakeRuntime::new([
            watchlist_api_fallback(),
            json!({"opened": false}),
            json!({"found": false}),
        ]);
        let symbols = vec!["NASDAQ:AAPL".to_string()];

        let err = watchlist_add_bulk(&mut runtime, &symbols, 0, false)
            .await
            .unwrap_err();

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
        let details = err.details.unwrap();
        assert_eq!(details["failed_count"], 1);
        assert_eq!(details["results"][0]["status"], "failed");
    }

    #[tokio::test]
    async fn watchlist_remove_removes_exact_symbol() {
        let mut runtime = FakeRuntime::new([
            watchlist_api_fallback(),
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
            runtime.evaluated[2]
                .0
                .contains("entry.symbol === requestedSymbol")
        );
        assert!(runtime.evaluated[2].0.contains("removeButton"));
        assert!(runtime.evaluated[2].0.contains("new MouseEvent"));
        assert!(!runtime.evaluated[2].0.contains("button.click"));
        assert!(!runtime.evaluated[2].0.contains("contextmenu"));
    }

    #[tokio::test]
    async fn watchlist_remove_uses_api_when_post_check_confirms_absence() {
        let mut runtime = FakeRuntime::new([json!({
            "symbol": "NASDAQ:AAPL",
            "requested_symbol": "NASDAQ:AAPL",
            "action": "removed",
            "removed": true,
            "source": "watchlist_api",
            "target_list": {"name": "Test", "type": "custom", "symbol_count": 2, "active": true},
            "before_count": 2,
            "after_count": 1,
            "matched_before": true,
            "matched_after": false,
            "remove_method": "api",
            "click_method": null,
            "confirmation_clicked": false
        })]);

        let result = watchlist_remove(&mut runtime, "NASDAQ:AAPL").await.unwrap();

        assert_eq!(result["action"], "removed");
        assert_eq!(result["removed"], true);
        assert_eq!(result["source"], "watchlist_api");
        assert_eq!(result["remove_method"], "api");
        assert!(runtime.evaluated[0].0.contains("symbols_list/all"));
        assert!(runtime.evaluated[0].0.contains("'remove'"));
    }

    #[tokio::test]
    async fn watchlist_remove_api_absent_symbol_is_validation_without_dom_fallback() {
        let mut runtime = FakeRuntime::new([json!({
            "error": "Watchlist symbol not found: NASDAQ:AAPL",
            "error_kind": "validation",
            "phase": "precheck_absent",
            "symbol": "NASDAQ:AAPL",
            "requested_symbol": "NASDAQ:AAPL",
            "source": "watchlist_api",
            "before_count": 1,
            "matched_before": false,
            "api_fallback_allowed": false
        })]);

        let error = watchlist_remove(&mut runtime, "NASDAQ:AAPL")
            .await
            .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert_eq!(error.details.as_ref().unwrap()["source"], "watchlist_api");
        assert_eq!(runtime.evaluated.len(), 1);
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
            watchlist_api_fallback(),
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
            watchlist_api_fallback(),
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
            watchlist_api_fallback(),
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
}
