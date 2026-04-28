use serde_json::{Value, json};

use tradingview_cdp::{MouseEvent, MouseEventType, RuntimeEvaluator};
use tradingview_core::{AppError, ErrorKind};

use super::{
    super::common::js_string,
    state::{screener_close, screener_open},
};

pub(super) const SCREENER_SOURCE: &str = "ui_screener_dialog";

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ScreenMenuClickPoint {
    x: f64,
    y: f64,
}

pub(super) type ScreenerClickPoint = ScreenMenuClickPoint;

pub(super) async fn read_screener_state(
    runtime: &mut impl RuntimeEvaluator,
    limit: Option<usize>,
) -> Result<Value, AppError> {
    let expression = match limit {
        Some(limit) => expanded_expression(&screener_read_expression(limit)),
        None => screener_state_expression(0),
    };
    let state = runtime.evaluate(&expression, true).await?;
    Ok(state)
}

pub(super) struct ScreenerReadState {
    pub(super) state: Value,
    pub(super) opened_for_read: bool,
    pub(super) restored_open_state: bool,
}

pub(super) struct ScreenerMutationSession<'a, R: RuntimeEvaluator> {
    pub(super) runtime: &'a mut R,
    pub(super) opened_for_mutation: bool,
    pub(super) restored_open_state: bool,
}

impl<'a, R: RuntimeEvaluator> ScreenerMutationSession<'a, R> {
    pub(super) async fn open(runtime: &'a mut R) -> Result<Self, AppError> {
        let initial = read_screener_state(runtime, None).await?;
        let restored_open_state = value_bool(&initial, "open");
        let mut opened_for_mutation = false;

        if !restored_open_state {
            screener_open(runtime).await?;
            opened_for_mutation = true;
        }

        Ok(Self {
            runtime,
            opened_for_mutation,
            restored_open_state,
        })
    }

    pub(super) async fn restore(&mut self) -> Result<(), AppError> {
        if self.opened_for_mutation {
            screener_close(self.runtime).await?;
        }
        Ok(())
    }
}

pub(super) async fn read_screener_with_restore(
    runtime: &mut impl RuntimeEvaluator,
    limit: Option<usize>,
) -> Result<ScreenerReadState, AppError> {
    let initial = read_screener_state(runtime, None).await?;
    let restored_open_state = value_bool(&initial, "open");
    let mut opened_for_read = false;

    if !restored_open_state {
        screener_open(runtime).await?;
        opened_for_read = true;
    }

    let read_result = read_screener_state(runtime, limit).await;
    let close_result = if opened_for_read {
        Some(screener_close(runtime).await)
    } else {
        None
    };

    let state = read_result?;
    if let Some(Err(err)) = close_result {
        return Err(err);
    }
    ensure_dialog_open(&state)?;

    Ok(ScreenerReadState {
        state,
        opened_for_read,
        restored_open_state,
    })
}

pub(super) fn ensure_dialog_open(value: &Value) -> Result<(), AppError> {
    if value_bool(value, "open") {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Stock Screener dialog is not open",
        )
        .with_details(value.clone()))
    }
}

pub(super) fn value_bool(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}

pub(super) async fn fetch_active_screener_storage_config(
    runtime: &mut impl RuntimeEvaluator,
    expected_title: &str,
) -> Result<Value, AppError> {
    let expected_title = js_string(expected_title)?;
    let result = runtime
        .evaluate(
            &expanded_expression(&format!(
                r#"
                (async function() {{
                    REPLACE_HELPERS
                    var initData = window.initData || {{}};
                    var active = initData.screen_data || {{}};
                    var storageUrl = initData.SCREENER_STORAGE_URL;
                    var version = initData.screener_storage_release_version;
                    var screenId = String(active.id || '');
                    var screenerKey = initData.standalone_type ||
                        active.screener_key ||
                        'stock';
                    if (!storageUrl || !version || !screenId || !screenerKey) {{
                        return {{
                            storage_available: false,
                            reason: 'missing_screener_storage_init_data',
                            columns: []
                        }};
                    }}
                    var expectedTitle = {expected_title};
                    var base = String(storageUrl).replace(/\/$/, '') + '/api/v2/screens/';
                    var url = base + encodeURIComponent(screenId) + '/?screener_key=' +
                        encodeURIComponent(screenerKey) + '&version=' + encodeURIComponent(version);
                    var response = await fetch(url, {{ credentials: 'include' }});
                    var body = await response.json().catch(function() {{ return null; }});
                    if (!response.ok || !body) {{
                        return {{
                            storage_available: true,
                            fetch_ok: response.ok,
                            status: response.status,
                            status_text: response.statusText,
                            reason: 'screen_fetch_failed',
                            columns: []
                        }};
                    }}
                    var title = String(body.title || active.title || '');
                    var columns = Array.isArray(body.default_custom_column_set)
                        ? body.default_custom_column_set
                        : (Array.isArray(active.default_custom_column_set)
                            ? active.default_custom_column_set
                            : []);
                    return {{
                        storage_available: true,
                        fetch_ok: true,
                        status: response.status,
                        screener_key: screenerKey,
                        version: body.version || active.version || version,
                        screen_id: String(body.id || screenId),
                        screen_title: title,
                        expected_title: expectedTitle,
                        title_matches: title === expectedTitle,
                        active_column_set: body.active_column_set || active.active_column_set || null,
                        storage_screen: body,
                        column_count: columns.length,
                        columns: columns.map(function(column, index) {{
                            return {{
                                index: index,
                                id: String(column && column.id || ''),
                                params: column && column.params ? column.params : {{}}
                            }};
                        }}).filter(function(column) {{ return column.id; }})
                    }};
                }})()
                "#
            )),
            true,
        )
        .await?;

    if !(value_bool(&result, "storage_available") && value_bool(&result, "fetch_ok")) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener active screen storage API was not available",
        )
        .with_details(result));
    }
    if !value_bool(&result, "title_matches") {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener active screen storage title did not match visible title",
        )
        .with_details(result));
    }
    Ok(result)
}

pub(super) fn screen_menu_click_point(value: &Value) -> Result<ScreenMenuClickPoint, AppError> {
    screener_click_point(value, "click_point")
}

pub(super) fn screener_click_point(
    value: &Value,
    field: &str,
) -> Result<ScreenerClickPoint, AppError> {
    let point = value.get(field).ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener click point missing",
        )
        .with_details(value.clone())
    })?;
    screener_click_point_from_value(point)
}

pub(super) fn screener_click_point_from_value(
    point: &Value,
) -> Result<ScreenerClickPoint, AppError> {
    let x = point.get("x").and_then(Value::as_f64).ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener click x coordinate missing",
        )
        .with_details(point.clone())
    })?;
    let y = point.get("y").and_then(Value::as_f64).ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener click y coordinate missing",
        )
        .with_details(point.clone())
    })?;
    Ok(ScreenMenuClickPoint { x, y })
}

pub(super) async fn dispatch_screen_menu_click(
    runtime: &mut impl RuntimeEvaluator,
    point: ScreenMenuClickPoint,
) -> Result<(), AppError> {
    runtime
        .dispatch_mouse_event(MouseEvent {
            event_type: MouseEventType::Moved,
            x: point.x,
            y: point.y,
            button: None,
            buttons: None,
            click_count: None,
            delta_x: None,
            delta_y: None,
        })
        .await?;
    runtime
        .dispatch_mouse_event(MouseEvent {
            event_type: MouseEventType::Pressed,
            x: point.x,
            y: point.y,
            button: Some("left"),
            buttons: Some(1),
            click_count: Some(1),
            delta_x: None,
            delta_y: None,
        })
        .await?;
    runtime
        .dispatch_mouse_event(MouseEvent {
            event_type: MouseEventType::Released,
            x: point.x,
            y: point.y,
            button: Some("left"),
            buttons: Some(0),
            click_count: Some(1),
            delta_x: None,
            delta_y: None,
        })
        .await
}

pub(super) fn require_active_screen_title(state: &Value) -> Result<String, AppError> {
    state
        .get("screen_title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            AppError::new(
                ErrorKind::InternalApiUnavailable,
                "Active Screener screen title was not available",
            )
            .with_details(state.clone())
        })
}

pub(super) fn normalize_columns(columns: Option<&Value>) -> Vec<Value> {
    columns
        .and_then(Value::as_array)
        .map(|columns| {
            columns
                .iter()
                .enumerate()
                .filter_map(|(index, column)| {
                    column.as_str().map(|name| {
                        json!({
                            "index": index,
                            "name": name,
                        })
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn screener_state_expression(limit: usize) -> String {
    format!(
        r#"
        (function() {{
            {SCREENER_HELPERS}
            return readScreenerState({limit});
        }})()
        "#
    )
}

fn screener_read_expression(limit: usize) -> String {
    format!(
        r#"
        (async function() {{
            function sleep(ms) {{
                return new Promise(function(resolve) {{ setTimeout(resolve, ms); }});
            }}
            REPLACE_HELPERS
            for (var i = 0; i < 20; i++) {{
                var state = readScreenerState({limit});
                var hasTextRow = state.rows.some(function(row) {{
                    return row.text || row.cells.some(function(cell) {{ return cell; }});
                }});
                if (!state.open || hasTextRow || state.visible_row_count === 0) return state;
                await sleep(200);
            }}
            return readScreenerState({limit});
        }})()
        "#
    )
}

const SCREENER_HELPERS: &str = r#"
function visible(el) {
    if (!el || !el.getBoundingClientRect) return false;
    var rect = el.getBoundingClientRect();
    var style = window.getComputedStyle ? window.getComputedStyle(el) : null;
    var inViewport = rect.bottom > 0 && rect.right > 0 && rect.top < window.innerHeight && rect.left < window.innerWidth;
    return rect.width > 0 && rect.height > 0 && inViewport && (!style || (style.visibility !== 'hidden' && style.display !== 'none'));
}
function textOf(el) {
    return (el && (el.textContent || el.innerText) || '').replace(/\s+/g, ' ').trim();
}
function visibleElements(selector) {
    return Array.from(document.querySelectorAll(selector)).filter(visible);
}
function mouseClick(el) {
    var rect = el.getBoundingClientRect();
    var x = rect.left + rect.width / 2;
    var y = rect.top + rect.height / 2;
    var target = document.elementFromPoint(x, y) || el;
    ['mouseover', 'mousedown', 'mouseup', 'click'].forEach(function(type) {
        target.dispatchEvent(new MouseEvent(type, {
            bubbles: true,
            cancelable: true,
            clientX: x,
            clientY: y,
            view: window
        }));
    });
}
function closeScreenerScreenMenu() {
    var title = document.querySelector('[data-name="screener-topbar-screen-title"]');
    if (title && visible(title)) {
        mouseClick(title);
    }
}
function screenerScreenActionText(text) {
    return /スクリーンを保存|スクリーンを共有|コピーを作成|名前を変更|CSV|新規スクリーン|最近使用した項目|スクリーンを開く|Save screen|Share screen|Make a copy|Rename|Download.*CSV|Create new screen|Recent|Open screen/i.test(text);
}
function screenerScreenExactActionText(text) {
    return /^(スクリーンを保存|スクリーンを共有|コピーを作成…?|名前を変更…?|結果をCSVでダウンロード|新規スクリーンを作成…?|最近使用した項目|スクリーンを開く…?|Save screen|Share screen|Make a copy…?|Rename…?|Download.*CSV|Create new screen…?|Recent|Open screen…?)$/i.test(text);
}
function screenerScreenActionKind(text) {
    if (/^(スクリーンを保存|Save screen)$/i.test(text)) return 'save';
    if (/^(スクリーンを共有|Share screen)$/i.test(text)) return 'share';
    if (/^(コピーを作成…?|Make a copy…?)$/i.test(text)) return 'make_copy';
    if (/^(名前を変更…?|Rename…?)$/i.test(text)) return 'rename';
    if (/^(新規スクリーンを作成…?|Create new screen…?)$/i.test(text)) return 'create';
    if (/^(スクリーンを開く…?|Open screen…?)$/i.test(text)) return 'open';
    if (/CSV/i.test(text)) return 'download_csv';
    if (/^(最近使用した項目|Recent)$/i.test(text)) return 'recent';
    return 'unknown';
}
function screenerScreenShortcutText(text) {
    return /^(⌘\s*S|Ctrl\s*\+\s*S|⇧\s*N|Shift\s*\+\s*N|ドット|Dot)$/i.test(text);
}
function screenerElementLabel(el) {
    return {
        text: textOf(el),
        aria_label: el.getAttribute('aria-label') || null,
        title: el.getAttribute('title') || null,
        data_name: el.getAttribute('data-name') || null
    };
}
function collectScreenerScreenActions(menu) {
    var seen = {};
    var actions = [];
    var nodes = Array.from(menu.querySelectorAll('button, [role="menuitem"], [role="option"], [role="button"], div, span')).filter(visible);
    nodes.forEach(function(el) {
        var text = textOf(el);
        if (!text || text.length > 120 || !screenerScreenExactActionText(text)) return;
        if (seen[text]) return;
        var disabled = el.disabled ||
            el.getAttribute('aria-disabled') === 'true' ||
            /disabled/i.test(String(el.className || ''));
        seen[text] = true;
        actions.push({
            index: actions.length,
            text: text,
            kind: screenerScreenActionKind(text),
            enabled: !disabled
        });
    });
    return actions;
}
function findScreenerScreenActionItem(menu, kind) {
    var candidates = Array.from(menu.querySelectorAll('button, [role="menuitem"], [role="option"], [role="button"], div, span')).filter(function(el) {
        return visible(el) && screenerScreenActionKind(textOf(el)) === kind;
    });
    candidates = candidates.map(function(el) {
        var current = el;
        while (current && current !== menu) {
            var cls = String(current.className || '');
            if (screenerScreenActionKind(textOf(current)) === kind &&
                (current.getAttribute('role') === 'menuitem' ||
                 current.getAttribute('role') === 'button' ||
                 current.tagName === 'BUTTON' ||
                 /button|background|item|row/i.test(cls))) {
                return current;
            }
            current = current.parentElement;
        }
        return el;
    });
    candidates.sort(function(a, b) {
        var ar = a.getBoundingClientRect();
        var br = b.getBoundingClientRect();
        return (br.width * br.height) - (ar.width * ar.height);
    });
    return candidates[0] || null;
}
function findScreenerScreenSaveItem(menu) {
    var candidates = Array.from(menu.querySelectorAll('button, [role="menuitem"], [role="option"], [role="button"], div, span')).filter(function(el) {
        return visible(el) && /^(スクリーンを保存|Save screen)$/i.test(textOf(el));
    });
    candidates = candidates.map(function(el) {
        var current = el;
        while (current && current !== menu) {
            var cls = String(current.className || '');
            if (/^(スクリーンを保存|Save screen)$/i.test(textOf(current)) &&
                (current.getAttribute('role') === 'menuitem' ||
                 current.getAttribute('role') === 'button' ||
                 current.tagName === 'BUTTON' ||
                 /button|background|item|row/i.test(cls))) {
                return current;
            }
            current = current.parentElement;
        }
        return el;
    });
    candidates.sort(function(a, b) {
        var ar = a.getBoundingClientRect();
        var br = b.getBoundingClientRect();
        return (br.width * br.height) - (ar.width * ar.height);
    });
    return candidates[0] || null;
}
function screenerScreenNameDialogTitle(dialog, kind) {
    var text = textOf(dialog);
    if (kind === 'create' && /スクリーンを作成|Create screen/i.test(text)) return 'create';
    if (kind === 'rename' && /スクリーン名の変更|Rename screen|Change screen name/i.test(text)) return 'rename';
    if (kind === 'make_copy' && /スクリーンのコピーを作成|Make a copy|Copy screen/i.test(text)) return 'make_copy';
    return null;
}
function findScreenerScreenNameDialog(kind) {
    var inputs = visibleElements('input, textarea');
    for (var i = 0; i < inputs.length; i++) {
        var current = inputs[i];
        while (current && current !== document.body && current !== document.documentElement) {
            var rect = current.getBoundingClientRect();
            if (rect.width >= 240 && rect.height >= 80) {
                var text = textOf(current);
                if (text.length <= 800) {
                    if (kind === 'create' && /スクリーンを作成|Create screen/i.test(text)) return current;
                    if (kind === 'rename' && /スクリーン名の変更|Rename screen|Change screen name/i.test(text)) return current;
                    if (kind === 'make_copy' && /スクリーンのコピーを作成|Make a copy|Copy screen/i.test(text)) return current;
                }
            }
            current = current.parentElement;
        }
    }
    return null;
}
function findScreenerScreenNameDialogSubmit(dialog, kind) {
    return Array.from(dialog.querySelectorAll('button, [role="button"]')).filter(visible).find(function(el) {
        var text = textOf(el);
        if (kind === 'create') return /^(作成|Create)$/i.test(text);
        if (kind === 'rename') return /^(名前を変更|Rename)$/i.test(text);
        if (kind === 'make_copy') return /^(コピーを作成|Make a copy)$/i.test(text);
        return false;
    }) || null;
}
function collectScreenerScreenEntries(menu, activeTitle) {
    var seen = {};
    var entries = [];
    var nodes = Array.from(menu.querySelectorAll('button, [role="menuitem"], [role="option"], div, span')).filter(visible);
    nodes.forEach(function(el) {
        var name = textOf(el);
        if (!name || name.length > 120 || screenerScreenActionText(name)) return;
        if (screenerScreenShortcutText(name)) return;
        if (name.indexOf('\n') >= 0) return;
        if (name.indexOf(activeTitle + ' ') === 0) return;
        if (seen[name]) return;
        seen[name] = true;
        entries.push({
            index: entries.length,
            name: name,
            active: name === activeTitle
        });
    });
    return entries;
}
function findScreenerScreenMenuItem(menu, targetName) {
    var candidates = Array.from(menu.querySelectorAll('button, [role="menuitem"], [role="option"], div, span')).filter(function(el) {
        return visible(el) && textOf(el) === targetName;
    });
    candidates = candidates.map(function(el) {
        var current = el;
        while (current && current !== menu) {
            var cls = String(current.className || '');
            if (textOf(current) === targetName &&
                (current.getAttribute('role') === 'menuitem' ||
                 current.tagName === 'BUTTON' ||
                 /button|background|item|row/i.test(cls))) {
                return current;
            }
            current = current.parentElement;
        }
        return el;
    });
    candidates.sort(function(a, b) {
        var ar = a.getBoundingClientRect();
        var br = b.getBoundingClientRect();
        return (br.width * br.height) - (ar.width * ar.height);
    });
    return candidates[0] || null;
}
function findScreenerScreenMenu() {
    var menus = visibleElements('.portal-lATuqHRX, [role="menu"], [class*="menu"], [class*="portal"]');
    var matches = menus.filter(function(menu) {
        var text = textOf(menu);
        return /最近使用した項目|Recent|スクリーンを開く|Open screen|スクリーンを保存|Save screen/i.test(text) &&
            /米国株|screen|Screen|スクリーン|CLI|株/i.test(text);
    });
    matches.sort(function(a, b) {
        var ar = a.getBoundingClientRect();
        var br = b.getBoundingClientRect();
        return (ar.width * ar.height) - (br.width * br.height);
    });
    return matches[0] || null;
}
function findScreenerCatalogOpenItem(menu) {
    var candidates = Array.from(menu.querySelectorAll('button, [role="menuitem"], [role="option"], div, span')).filter(function(el) {
        return visible(el) && /^(スクリーンを開く|Open screen)/i.test(textOf(el));
    });
    candidates = candidates.map(function(el) {
        var current = el;
        while (current && current !== menu) {
            var cls = String(current.className || '');
            if (/^(スクリーンを開く|Open screen)/i.test(textOf(current)) &&
                (current.getAttribute('role') === 'menuitem' ||
                 current.tagName === 'BUTTON' ||
                 /button|background|item|row/i.test(cls))) {
                return current;
            }
            current = current.parentElement;
        }
        return el;
    });
    candidates.sort(function(a, b) {
        var ar = a.getBoundingClientRect();
        var br = b.getBoundingClientRect();
        return (br.width * br.height) - (ar.width * ar.height);
    });
    return candidates[0] || null;
}
function openScreenerScreenMenu() {
    var title = document.querySelector('[data-name="screener-topbar-screen-title"]');
    if (!title || !visible(title)) {
        return { menu_opened: false, reason: 'title_not_found', screens: [] };
    }
    var activeTitle = textOf(title);
    mouseClick(title);
    var menu = findScreenerScreenMenu();
    if (!menu) {
        return {
            menu_opened: false,
            reason: 'menu_not_found',
            screen_title: activeTitle,
            screens: []
        };
    }
    var entries = collectScreenerScreenEntries(menu, activeTitle);
    return {
        menu_opened: true,
        screen_title: activeTitle,
        catalog_entry_found: /スクリーンを開く|Open screen/i.test(textOf(menu)),
        screens: entries,
        menu: menu
    };
}
function findScreenerScreenCatalog() {
    var containers = visibleElements('[role="dialog"], [class*="dialog"], [class*="Dialog"], [class*="modal"], [class*="Modal"], .portal-lATuqHRX');
    return containers.find(function(container) {
        if (container.querySelector && container.querySelector('table')) return false;
        var text = textOf(container);
        return /マイスクリーン|My screens|Screeners|スクリーン/i.test(text) &&
            !/スクリーンを保存|Save screen/i.test(text);
    }) || null;
}
function closeScreenerScreenCatalog() {
    var catalog = findScreenerScreenCatalog();
    if (!catalog) return;
    var closeButton = Array.from(catalog.querySelectorAll('button, [role="button"], [aria-label], [data-name]')).filter(visible).find(function(el) {
        var label = [el.getAttribute('aria-label'), el.getAttribute('title'), el.getAttribute('data-name'), textOf(el)].filter(Boolean).join(' ');
        return /close|dismiss|閉じる|キャンセル|cancel/i.test(label);
    });
    if (closeButton) {
        mouseClick(closeButton);
        return;
    }
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', code: 'Escape', keyCode: 27, which: 27, bubbles: true, cancelable: true }));
    document.dispatchEvent(new KeyboardEvent('keyup', { key: 'Escape', code: 'Escape', keyCode: 27, which: 27, bubbles: true, cancelable: true }));
}
function closeScreenerScreenLifecyclePopups() {
    var buttons = visibleElements('button, [role="button"], [aria-label], [title]');
    var closeButton = buttons.find(function(el) {
        var label = [textOf(el), el.getAttribute('aria-label'), el.getAttribute('title')].filter(Boolean).join(' ');
        return /^(キャンセル|Cancel|close|閉じる)$/i.test(label);
    });
    if (closeButton) {
        mouseClick(closeButton);
        return;
    }
    closeScreenerScreenCatalog();
    closeScreenerScreenMenu();
}
function findBlockingScreenerMutationDialog() {
    var containers = visibleElements('[role="dialog"], [class*="dialog"], [class*="Dialog"], [class*="modal"], [class*="Modal"], .portal-lATuqHRX');
    return containers.find(function(container) {
        if (container.querySelector && container.querySelector('table')) return false;
        var text = textOf(container);
        if (text.length > 3000) return false;
        if (/スクリーンを保存|Save screen/i.test(text) &&
            /スクリーンを開く|Open screen/i.test(text) &&
            /最近使用した項目|Recent/i.test(text)) return false;
        return /名前を変更|Rename|コピーを作成|Make a copy|新規スクリーン|Create new screen|削除|Delete|保存先|Save as/i.test(text);
    }) || null;
}
function screenerCatalogActionText(text) {
    return /マイスクリーン|My screens|最近|Recent|スクリーンを開く|Open screen|スクリーンを保存|Save screen|スクリーンを共有|Share screen|コピーを作成|Make a copy|名前を変更|Rename|新規スクリーン|Create new screen|検索|Search|キャンセル|Cancel|閉じる|Close/i.test(text);
}
function collectScreenerCatalogScreenEntries(catalog, activeTitle) {
    var seen = {};
    var entries = [];
    var inMyScreens = false;
    var leftMyScreens = false;
    var nodes = Array.from(catalog.querySelectorAll('button, [role="option"], [role="menuitem"], [role="row"], [data-name], div, span')).filter(visible);
    nodes.forEach(function(el) {
        if (leftMyScreens) return;
        var name = textOf(el);
        if (/^(マイスクリーン|My screens)$/i.test(name)) {
            inMyScreens = true;
            return;
        }
        if (/^(人気のスクリーン|Popular screens)$/i.test(name)) {
            leftMyScreens = true;
            return;
        }
        if (!inMyScreens) return;
        if (!name || name.length > 120 || screenerCatalogActionText(name)) return;
        if (name.indexOf('\n') >= 0) return;
        if (seen[name]) return;
        var rect = el.getBoundingClientRect();
        if (rect.width < 20 || rect.height < 8) return;
        seen[name] = true;
        entries.push({
            index: entries.length,
            name: name,
            active: name === activeTitle
        });
    });
    return entries;
}
function findScreenerCatalogScreenItem(catalog, targetName) {
    var candidates = Array.from(catalog.querySelectorAll('button, [role="option"], [role="menuitem"], [role="row"], [data-name], div, span')).filter(function(el) {
        return visible(el) && textOf(el) === targetName;
    });
    candidates = candidates.map(function(el) {
        var current = el;
        while (current && current !== catalog) {
            var cls = String(current.className || '');
            var role = current.getAttribute('role') || '';
            if (textOf(current) === targetName &&
                (role === 'option' || role === 'menuitem' || role === 'row' ||
                 current.tagName === 'BUTTON' || /button|item|row|list|cell/i.test(cls))) {
                return current;
            }
            current = current.parentElement;
        }
        return el;
    });
    candidates.sort(function(a, b) {
        var ar = a.getBoundingClientRect();
        var br = b.getBoundingClientRect();
        return (br.width * br.height) - (ar.width * ar.height);
    });
    return candidates[0] || null;
}
function findScreenerCatalogDeleteButton(catalog) {
    var buttons = Array.from(catalog.querySelectorAll('button, [role="button"], [aria-label], [title], [data-name]')).filter(visible);
    return buttons.find(function(el) {
        var label = [textOf(el), el.getAttribute('aria-label'), el.getAttribute('title'), el.getAttribute('data-name')].filter(Boolean).join(' ');
        return /^(削除|Delete)$|削除|Delete/i.test(label);
    }) || null;
}
function findScreenerDeleteConfirmDialog(targetName) {
    var containers = visibleElements('[role="dialog"], [class*="dialog"], [class*="Dialog"], [class*="modal"], [class*="Modal"], .portal-lATuqHRX');
    return containers.find(function(container) {
        if (container.querySelector && container.querySelector('table')) return false;
        var text = textOf(container);
        if (text.length > 1000) return false;
        return /削除|Delete/i.test(text) && (!targetName || text.indexOf(targetName) >= 0 || /本当に|Are you sure|確認/i.test(text));
    }) || null;
}
function findScreenerDeleteConfirmButton(dialog) {
    return Array.from(dialog.querySelectorAll('button, [role="button"]')).filter(visible).find(function(el) {
        return /^(削除|Delete)$/i.test(textOf(el));
    }) || null;
}
async function openScreenerScreenCatalog() {
    function sleep(ms) {
        return new Promise(function(resolve) { setTimeout(resolve, ms); });
    }
    var title = document.querySelector('[data-name="screener-topbar-screen-title"]');
    if (!title || !visible(title)) {
        return { catalog_opened: false, menu_opened: false, reason: 'title_not_found', screens: [] };
    }
    var activeTitle = textOf(title);
    mouseClick(title);
    var menu = null;
    for (var i = 0; i < 10; i++) {
        menu = findScreenerScreenMenu();
        if (menu) break;
        await sleep(150);
    }
    if (!menu) {
        return { catalog_opened: false, menu_opened: false, reason: 'menu_not_found', screen_title: activeTitle, screens: [] };
    }
    var openItem = findScreenerCatalogOpenItem(menu);
    if (!openItem) {
        closeScreenerScreenMenu();
        return { catalog_opened: false, menu_opened: true, reason: 'catalog_open_item_not_found', screen_title: activeTitle, screens: collectScreenerScreenEntries(menu, activeTitle) };
    }
    mouseClick(openItem);
    for (var j = 0; j < 4; j++) {
        var catalog = findScreenerScreenCatalog();
        if (catalog) {
            return {
                catalog_opened: true,
                menu_opened: true,
                screen_title: activeTitle,
                screens: collectScreenerCatalogScreenEntries(catalog, activeTitle),
                catalog: catalog
            };
        }
        await sleep(100);
    }
    return { catalog_opened: false, menu_opened: true, reason: 'catalog_not_found', screen_title: activeTitle, screens: [] };
}
function normalizeScreenerFilterText(value) {
    return String(value || '').replace(/[\u2066\u2067\u2068\u2069]/g, '').replace(/\s+/g, ' ').trim();
}
function findScreenerAddFilterButton() {
    return visibleElements('button, [role="button"], [title], [aria-label], [data-name]').find(function(el) {
        var label = [el.getAttribute('title'), el.getAttribute('aria-label'), el.getAttribute('data-name'), textOf(el)].filter(Boolean).join(' ');
        return /新しいフィルターを追加|フィルターを追加|Add new filter|Add filter/i.test(label);
    }) || null;
}
function findScreenerAddFilterDialog() {
    return visibleElements('[role="dialog"], [class*="popover"], [class*="contentDefaultAppearance"]').find(function(el) {
        var text = normalizeScreenerFilterText(textOf(el));
        return /フィルター|Filter/i.test(text) && el.getBoundingClientRect().width >= 180;
    }) || null;
}
function findScreenerAddFilterSearchInput() {
    var dialog = findScreenerAddFilterDialog();
    if (!dialog) return null;
    return scopedVisibleElements(dialog, 'input, textarea').find(function(input) {
        var label = [input.getAttribute('placeholder'), input.getAttribute('aria-label'), input.getAttribute('role')].filter(Boolean).join(' ');
        return /検索|Search|combobox/i.test(label);
    }) || null;
}
function findScreenerAddFilterCandidate(name) {
    var dialog = findScreenerAddFilterDialog();
    if (!dialog) return null;
    var normalizedName = normalizeScreenerFilterText(name).toLowerCase();
    var options = scopedVisibleElements(dialog, '[role="option"], button, [role="button"], [data-name]');
    var exact = options.find(function(option) {
        return normalizeScreenerFilterText(textOf(option)).toLowerCase() === normalizedName;
    });
    if (exact) return exact;
    return options.find(function(option) {
        var text = normalizeScreenerFilterText(textOf(option)).toLowerCase();
        return text.indexOf(normalizedName) >= 0 || normalizedName.indexOf(text) >= 0;
    }) || null;
}
function findScreenerAddFilterRangeOption(matchers) {
    var dialog = findScreenerAddFilterDialog();
    if (!dialog) return null;
    var normalizedMatchers = matchers.map(function(matcher) {
        return normalizeScreenerFilterText(matcher).toLowerCase();
    }).filter(Boolean);
    return scopedVisibleElements(dialog, '[role="option"], button, [role="button"], [data-name]').find(function(option) {
        var text = normalizeScreenerFilterText(textOf(option)).toLowerCase();
        return normalizedMatchers.some(function(matcher) {
            return text.indexOf(matcher) >= 0;
        });
    }) || null;
}
function findScreenerNumericFilterPill() {
    var filters = visibleElements('[data-name^="screener-filter-pill-"]');
    return filters.find(function(el) {
        var text = normalizeScreenerFilterText(textOf(el));
        return /:/.test(text) && /〜/.test(text);
    }) || filters.find(function(el) {
        var text = normalizeScreenerFilterText(textOf(el));
        return /EMA|SMA|価格|Price/i.test(text) && /〜/.test(text);
    }) || filters.find(function(el) {
        return /〜/.test(normalizeScreenerFilterText(textOf(el)));
    }) || filters.find(function(el) {
        return /以上|以下|未満/.test(normalizeScreenerFilterText(textOf(el)));
    }) || filters.find(function(el) {
        return /greater|less|between|range/i.test(normalizeScreenerFilterText(textOf(el)));
    }) || null;
}
function findScreenerFallbackNumericFilterPill() {
    var filters = visibleElements('[data-name^="screener-filter-pill-"]');
    return filters.find(function(el) {
        return /%|以上|以下|未満|greater|less|between|range/i.test(normalizeScreenerFilterText(textOf(el)));
    }) || filters[0] || null;
}
function scopedVisibleElements(scope, selector) {
    return Array.from((scope || document).querySelectorAll(selector)).filter(visible);
}
function screenerFilterPopoverTokens(text) {
    return normalizeScreenerFilterText(text)
        .replace(/-?\d+(?:\.\d+)?%\s*(?:〜|to)\s*-?\d+(?:\.\d+)?%/ig, ' ')
        .replace(/-?\d+(?:\.\d+)?%\s*(?:以上|以下|未満|or more|less)/ig, ' ')
        .split(/[^A-Za-z\u3040-\u30ff\u3400-\u9fff]+/)
        .filter(function(token) {
            return token.length >= 2 && !/^(未満|以上|以下|価格|Price)$/.test(token);
        });
}
function findScreenerFilterEditPopoverForPill(pill) {
    if (!pill) return null;
    var tokens = screenerFilterPopoverTokens(textOf(pill));
    var popovers = visibleElements('[role="dialog"], [class*="popover"], [class*="contentDefaultAppearance"]').filter(function(el) {
        var rect = el.getBoundingClientRect();
        return rect.width >= 160 && rect.height >= 80;
    });
    var scored = popovers.map(function(popover) {
        var text = normalizeScreenerFilterText(textOf(popover)).toLowerCase();
        var score = tokens.reduce(function(total, token) {
            return total + (text.indexOf(token.toLowerCase()) >= 0 ? 1 : 0);
        }, 0);
        return { popover: popover, score: score };
    }).filter(function(candidate) {
        return candidate.score > 0;
    });
    scored.sort(function(a, b) {
        if (b.score !== a.score) return b.score - a.score;
        var ar = a.popover.getBoundingClientRect();
        var br = b.popover.getBoundingClientRect();
        return ar.top - br.top;
    });
    return scored.length > 0 ? scored[0].popover : null;
}
function findScreenerOptionPopoverForPill(pill, requestedOption) {
    if (!pill) return null;
    var pillRect = pill.getBoundingClientRect();
    var requested = normalizeScreenerFilterText(requestedOption).toLowerCase();
    var popovers = visibleElements('[role="dialog"], [class*="popover"], [class*="contentDefaultAppearance"], [role="listbox"]').filter(function(el) {
        var rect = el.getBoundingClientRect();
        if (rect.width < 120 || rect.height < 60) return false;
        if (rect.bottom < pillRect.bottom - 4) return false;
        if (rect.left > pillRect.right + 260) return false;
        if (rect.right < pillRect.left - 40) return false;
        var text = normalizeScreenerFilterText(textOf(el)).toLowerCase();
        return !requested || text.indexOf(requested) >= 0 || el.querySelector('[role="option"], [role="listbox"]');
    });
    popovers.sort(function(a, b) {
        var ar = a.getBoundingClientRect();
        var br = b.getBoundingClientRect();
        var ad = Math.abs(ar.left - pillRect.left) + Math.abs(ar.top - pillRect.bottom);
        var bd = Math.abs(br.left - pillRect.left) + Math.abs(br.top - pillRect.bottom);
        if (ad !== bd) return ad - bd;
        return (ar.width * ar.height) - (br.width * br.height);
    });
    return popovers[0] || null;
}
function findScreenerManualFilterButton(scope) {
    return scopedVisibleElements(scope, 'button, [role="button"], [role="menuitem"], div, span').find(function(el) {
        var text = textOf(el);
        return /手動で設定|Set manually|Manual/i.test(text) && text.length < 120;
    }) || null;
}
function findScreenerRangeCombobox(scope) {
    var candidates = scopedVisibleElements(scope, 'button, [role="button"], [role="combobox"], [aria-haspopup], div, span');
    candidates = candidates.filter(function(el) {
        var text = normalizeScreenerFilterText(textOf(el));
        if (!text || text.length > 80) return false;
        if (el.closest && el.closest('[data-name^="screener-filter-pill-"]')) return false;
        return /%/.test(text) && (/〜|以上|以下|未満|to|or more|less/i.test(text));
    });
    candidates.sort(function(a, b) {
        var ar = a.getBoundingClientRect();
        var br = b.getBoundingClientRect();
        return (ar.width * ar.height) - (br.width * br.height);
    });
    return candidates[0] || null;
}
function collectScreenerRangeOptions(scope) {
    var seen = {};
    var options = [];
    function rangeLabel(text) {
        var normalized = normalizeScreenerFilterText(text);
        var match = normalized.match(/-?\d+(?:\.\d+)?%\s*(?:〜|to)\s*-?\d+(?:\.\d+)?%/i) ||
            normalized.match(/-?\d+(?:\.\d+)?%\s*(?:以上|以下|未満)/) ||
            normalized.match(/-?\d+(?:\.\d+)?%\s*(?:or more|less)/i);
        return match ? match[0] : null;
    }
    function optionClickTarget(el, label) {
        var current = el;
        while (current && current !== document.body) {
            var role = current.getAttribute && (current.getAttribute('role') || '');
            var cls = String(current.className || '');
            if (textOf(current).indexOf(label) >= 0 &&
                (role === 'option' || role === 'menuitem' || role === 'button' ||
                 current.tagName === 'BUTTON' || /item|option|row|button/i.test(cls))) {
                return current;
            }
            current = current.parentElement;
        }
        return el;
    }
    var scopes = scopedVisibleElements(scope, '[role="listbox"], [role="dialog"], [class*="popover"], [class*="menu"], [class*="contentDefaultAppearance"]').filter(function(candidateScope) {
        var rect = candidateScope.getBoundingClientRect();
        var text = normalizeScreenerFilterText(textOf(candidateScope));
        return rect.width >= 100 && rect.height >= 80 && /%/.test(text) && (/〜|以上|以下|未満|to|or more|less/i.test(text));
    });
    var nodes = [];
    if (scopes.length > 0) {
        scopes.forEach(function(scope) {
            nodes = nodes.concat(Array.from(scope.querySelectorAll('[role="option"], [role="menuitem"], button, div, span')).filter(visible));
        });
    } else {
        nodes = visibleElements('[role="option"], [role="menuitem"], button, div, span');
    }
    nodes.forEach(function(el) {
        var label = rangeLabel(textOf(el));
        if (!label) return;
        if (seen[label]) return;
        seen[label] = true;
        options.push({
            index: options.length,
            text: label,
            normalized_text: label,
            element: optionClickTarget(el, label)
        });
    });
    return options;
}
function collectScreenerOptionChoices(scope) {
    if (!scope) return [];
    var seen = {};
    var options = [];
    function optionClickTarget(el, label) {
        var current = el;
        while (current && current !== document.body) {
            var role = current.getAttribute && (current.getAttribute('role') || '');
            var cls = String(current.className || '');
            if (textOf(current).indexOf(label) >= 0 &&
                (role === 'option' || role === 'menuitem' || role === 'button' ||
                 current.tagName === 'BUTTON' || /item|option|row|button/i.test(cls))) {
                return current;
            }
            if (current === scope) break;
            current = current.parentElement;
        }
        return el;
    }
    var choiceScopes = scopedVisibleElements(scope, '[role="listbox"], [role="menu"]').filter(function(candidateScope) {
        var rect = candidateScope.getBoundingClientRect();
        return rect.width >= 80 && rect.height >= 30;
    });
    if (choiceScopes.length === 0) choiceScopes = [scope];
    var nodes = [];
    choiceScopes.forEach(function(choiceScope) {
        nodes = nodes.concat(scopedVisibleElements(choiceScope, '[role="option"], [role="menuitem"], button, [role="button"]'));
    });
    nodes.forEach(function(el) {
        var text = normalizeScreenerFilterText(textOf(el));
        if (!text || text.length > 80) return;
        if (/%/.test(text) && /〜|以上|以下|未満|to|or more|less/i.test(text)) return;
        if (/手動で設定|Set manually|Manual|削除|Remove|Delete/i.test(text)) return;
        if (text.indexOf('\n') >= 0) return;
        if (seen[text]) return;
        var rect = el.getBoundingClientRect();
        if (rect.width < 8 || rect.height < 8) return;
        seen[text] = true;
        options.push({
            index: options.length,
            text: text,
            normalized_text: text,
            selected: el.getAttribute('aria-selected') === 'true' || el.getAttribute('aria-checked') === 'true',
            element: optionClickTarget(el, text)
        });
    });
    return options;
}
function closeScreenerTransientPopups() {
    for (var i = 0; i < 3; i++) {
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', code: 'Escape', keyCode: 27, which: 27, bubbles: true, cancelable: true }));
        document.dispatchEvent(new KeyboardEvent('keyup', { key: 'Escape', code: 'Escape', keyCode: 27, which: 27, bubbles: true, cancelable: true }));
    }
}
function nearestScreenerPanelRoot(el) {
    var current = el;
    while (current && current !== document.body && current !== document.documentElement) {
        var rect = current.getBoundingClientRect();
        if (rect.width >= 260 && rect.height >= 160) {
            var hasTitle = !!current.querySelector('[data-name="screener-topbar-screen-title"]');
            var hasFilters = !!current.querySelector('[data-name^="screener-filter-pill-"]');
            var hasTable = !!current.querySelector('table');
            var dataName = current.getAttribute('data-name') || '';
            var className = String(current.className || '');
            if (hasTitle || hasFilters || hasTable || /screenerContainer|screener-container/i.test(className) || /screener/i.test(dataName)) {
                return current;
            }
        }
        current = current.parentElement;
    }
    return null;
}
function findScreenerPanelRoot(button) {
    var roots = visibleElements('[class*="screenerContainer"], [class*="screener-container"]').filter(function(el) {
        return el !== button && el.getBoundingClientRect().width >= 260 && el.getBoundingClientRect().height >= 160;
    });
    if (roots.length > 0) return roots[0];
    var anchors = visibleElements('[data-name="screener-topbar-screen-title"], [data-name^="screener-filter-pill-"], table').filter(function(el) {
        return el !== button && !buttonContains(button, el);
    });
    for (var i = 0; i < anchors.length; i++) {
        var root = nearestScreenerPanelRoot(anchors[i]);
        if (root) return root;
    }
    return null;
}
function buttonContains(button, el) {
    return !!(button && el && button !== el && button.contains(el));
}
function readScreenerState(limit) {
    var button = document.querySelector('[data-name="screener-dialog-button"]');
    var panelRoot = findScreenerPanelRoot(button);
    var screenerDataElements = panelRoot ? scopedVisibleElements(panelRoot, '[data-name*="screener"]') : [];
    var classElements = panelRoot ? scopedVisibleElements(panelRoot, '[class*="screener"]') : [];
    var heading = panelRoot ? Array.from(panelRoot.querySelectorAll('h1, h2, h3'))
        .find(function(el) {
            var text = textOf(el);
            return visible(el) && text.length <= 120 && /screener|スクリーナー/i.test(text);
        }) || Array.from(panelRoot.querySelectorAll('button, div, span'))
        .find(function(el) {
            var text = textOf(el);
            return visible(el) && text.length <= 120 && /screener|スクリーナー/i.test(text);
        }) || null : null;
    var table = panelRoot ? (Array.from(panelRoot.querySelectorAll('table')).filter(visible)[0] || null) : null;
    var title = panelRoot ? panelRoot.querySelector('[data-name="screener-topbar-screen-title"]') : null;
    var open = !!panelRoot;
    var filters = panelRoot ? scopedVisibleElements(panelRoot, '[data-name^="screener-filter-pill-"]').map(function(el) {
        return {
            text: textOf(el),
            data_name: el.getAttribute('data-name') || null,
            visible: visible(el)
        };
    }) : [];
    var columns = table ? Array.from(table.querySelectorAll('th')).map(textOf).filter(function(text) {
        return text.length > 0;
    }) : [];
    var rows = [];
    if (table && limit > 0) {
        rows = Array.from(table.querySelectorAll('tbody tr')).slice(0, limit).map(function(row) {
            var cells = Array.from(row.querySelectorAll('td, th')).map(textOf);
            var fieldValues = {};
            columns.forEach(function(column, index) {
                if (column && index < cells.length) fieldValues[column] = cells[index];
            });
            return {
                cells: cells,
                field_values: fieldValues,
                text: textOf(row).substring(0, 500)
            };
        });
    }
    var visibleRowCount = table ? table.querySelectorAll('tbody tr').length : 0;
    return {
        button_found: !!button,
        open: open,
        dialog_title: heading ? textOf(heading) : null,
        screen_title: title ? textOf(title) : null,
        filters: filters,
        filter_count: filters.length,
        columns: columns,
        column_count: columns.length,
        rows: rows,
        row_count: rows.length,
        visible_row_count: visibleRowCount,
        table_found: !!table,
        panel_root_found: !!panelRoot,
        class_match_count: classElements.length,
        data_name_match_count: screenerDataElements.length
    };
}
"#;

pub(super) fn expanded_expression(template: &str) -> String {
    template.replace("REPLACE_HELPERS", SCREENER_HELPERS)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use tradingview_core::ErrorKind;

    #[test]
    fn ensure_dialog_open_rejects_toolbar_only_state() {
        let value = json!({
            "button_found": true,
            "open": false,
            "panel_root_found": false,
            "dialog_title": null,
            "screen_title": null,
            "filter_count": 0,
            "column_count": 0
        });

        let error = ensure_dialog_open(&value).unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
    }
}
