use serde_json::{Value, json};
use std::time::Duration;
use tokio::time::Instant;

use tradingview_cdp::{KeyEvent, KeyEventType, RuntimeEvaluator};
use tradingview_core::{AppError, ErrorKind};

pub(super) const FIND_MONACO: &str = r#"
(function findMonacoEditor() {
    function visible(node) {
        if (!node || typeof node.getBoundingClientRect !== 'function') return false;
        try {
            var rect = node.getBoundingClientRect();
            var style = window.getComputedStyle ? window.getComputedStyle(node) : null;
            return rect.width > 0 && rect.height > 0
                && (!style || (style.display !== 'none' && style.visibility !== 'hidden'));
        } catch (e) {
            return false;
        }
    }
    function candidate(editor, env) {
        try {
            var node = editor && typeof editor.getContainerDomNode === 'function'
                ? editor.getContainerDomNode()
                : null;
            var owner = node && node.closest ? node.closest('[data-name="pine-dialog"]') : null;
            if (!owner || !visible(node) || !visible(owner)) return null;
            var focused = false;
            try {
                focused = typeof editor.hasTextFocus === 'function' && editor.hasTextFocus();
            } catch (e) {}
            if (!focused) {
                try { focused = node.contains(document.activeElement); } catch (e) {}
            }
            return { editor: editor, env: env, container: node, owner: owner, focused: focused };
        } catch (e) {
            return null;
        }
    }
    function select(candidates) {
        var unique = [];
        for (var i = 0; i < candidates.length; i++) {
            var item = candidates[i];
            if (item && !unique.some(function(existing) { return existing.editor === item.editor; })) {
                unique.push(item);
            }
        }
        var focused = unique.filter(function(item) { return item.focused; });
        if (focused.length === 1) {
            focused[0].candidate_count = unique.length;
            return focused[0];
        }
        if (focused.length === 0 && unique.length === 1) {
            unique[0].candidate_count = 1;
            return unique[0];
        }
        return null;
    }

    var candidates = [];
    try {
        if (window.monaco && window.monaco.editor && typeof window.monaco.editor.getEditors === 'function') {
            var globalEditors = window.monaco.editor.getEditors();
            for (var g = 0; g < globalEditors.length; g++) {
                candidates.push(candidate(globalEditors[g], { editor: window.monaco.editor }));
            }
        }
    } catch(e) {}
    var containers = [];
    try { containers = Array.from(document.querySelectorAll('[data-name="pine-dialog"] .monaco-editor')); } catch (e) {}
    for (var c = 0; c < containers.length; c++) {
        var el = containers[c];
        var fiberKey = null;
        for (var up = 0; up < 20 && el; up++, el = el.parentElement) {
            try { fiberKey = Object.keys(el).find(function(k) { return k.startsWith('__reactFiber$'); }); } catch (e) {}
            if (fiberKey) break;
        }
        if (!fiberKey || !el) continue;
        var current = el[fiberKey];
        for (var d = 0; d < 40 && current; d++, current = current.return) {
            var props = current.memoizedProps;
            var env = props && (props.monacoEnv || (props.value && props.value.monacoEnv));
            if (env && env.editor && typeof env.editor.getEditors === 'function') {
                try {
                    var editors = env.editor.getEditors();
                    for (var e = 0; e < editors.length; e++) candidates.push(candidate(editors[e], env));
                } catch (e) {}
            }
        }
    }
    return select(candidates);
})()
"#;

const OPEN_PINE_PANEL_EXPRESSION: &str = r#"
(function() {
    var bwb = window.TradingView && window.TradingView.bottomWidgetBar;
    if (bwb) {
        if (typeof bwb.activateScriptEditorTab === 'function') {
            bwb.activateScriptEditorTab();
            return 'activateScriptEditorTab';
        }
        if (typeof bwb.showWidget === 'function') {
            bwb.showWidget('pine-editor');
            return 'showWidget';
        }
        if (typeof bwb.open === 'function') {
            bwb.open('pine-editor');
            return 'open';
        }
        if (typeof bwb.show === 'function') {
            bwb.show('pine-editor');
            return 'show';
        }
    }
    var btn = document.querySelector('[aria-label="Pine"]')
        || document.querySelector('[data-name="pine-dialog-button"]');
    if (btn) {
        btn.click();
        return 'button-click';
    }
    return null;
})()
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct EditorOpenState {
    pub(super) editor_open_before: bool,
    pub(super) opened_editor: bool,
}

#[cfg(test)]
pub(super) const PINE_COMPILE_WAIT: Duration = Duration::from_millis(0);
#[cfg(not(test))]
pub(super) const PINE_COMPILE_WAIT: Duration = Duration::from_millis(2500);

#[cfg(test)]
pub(super) const PINE_SAVE_WAIT: Duration = Duration::from_millis(0);
#[cfg(not(test))]
pub(super) const PINE_SAVE_WAIT: Duration = Duration::from_millis(800);

pub(super) async fn ensure_pine_editor_open(
    runtime: &mut impl RuntimeEvaluator,
) -> Result<EditorOpenState, AppError> {
    let deadline = Instant::now() + Duration::from_secs(10);
    let inspect_expression = with_monaco("var m = __FIND_MONACO__; return m !== null;");
    let editor_open_before =
        tokio::time::timeout_at(deadline, runtime.evaluate(&inspect_expression, false))
            .await
            .map_err(|_| pine_editor_deadline_error(false))?
            .map_err(|error| pine_editor_evaluation_error(error.kind, false))?
            .as_bool()
            .unwrap_or(false);
    if editor_open_before {
        return Ok(EditorOpenState {
            editor_open_before,
            opened_editor: false,
        });
    }

    tokio::time::timeout_at(
        deadline,
        runtime.evaluate(OPEN_PINE_PANEL_EXPRESSION, false),
    )
    .await
    .map_err(|_| pine_editor_deadline_error(false))?
    .map_err(|error| pine_editor_evaluation_error(error.kind, true))?;

    for attempt in 0..50 {
        #[cfg(not(test))]
        tokio::time::sleep_until((Instant::now() + Duration::from_millis(200)).min(deadline)).await;
        let ready = tokio::time::timeout_at(deadline, runtime.evaluate(&inspect_expression, false))
            .await
            .map_err(|_| pine_editor_deadline_error(true))?
            .map_err(|error| pine_editor_evaluation_error(error.kind, true))?
            .as_bool()
            .unwrap_or(false);
        if ready {
            return Ok(EditorOpenState {
                editor_open_before,
                opened_editor: true,
            });
        }
        if attempt > 0 && attempt % 10 == 0 {
            tokio::time::timeout_at(
                deadline,
                runtime.evaluate(OPEN_PINE_PANEL_EXPRESSION, false),
            )
            .await
            .map_err(|_| pine_editor_deadline_error(true))?
            .map_err(|error| pine_editor_evaluation_error(error.kind, true))?;
        }
    }

    Err(AppError::new(
        ErrorKind::InternalApiUnavailable,
        "Could not open Pine Editor or Monaco was not found",
    )
    .with_details(json!({
        "editor_open_before": editor_open_before,
        "opened_editor": false,
    })))
}

fn pine_editor_deadline_error(open_attempted: bool) -> AppError {
    AppError::new(
        ErrorKind::InternalApiUnavailable,
        "Pine Editor readiness exceeded its bounded deadline",
    )
    .with_details(json!({
        "editor_open_before": false,
        "opened_editor": false,
        "open_attempted": open_attempted,
        "stage": "editor_readiness",
    }))
}

fn pine_editor_evaluation_error(kind: ErrorKind, open_attempted: bool) -> AppError {
    AppError::new(kind, "Pine Editor readiness evaluation failed").with_details(json!({
        "editor_open_before": false,
        "opened_editor": false,
        "open_attempted": open_attempted,
        "stage": "editor_readiness",
    }))
}

pub(super) fn with_monaco(body: &str) -> String {
    format!(
        "(function() {{ {} }})()",
        body.replace("__FIND_MONACO__", FIND_MONACO)
    )
}

pub(super) fn normalize_array(value: Value, error_message: &str) -> Result<Vec<Value>, AppError> {
    value.as_array().cloned().ok_or_else(|| {
        AppError::new(ErrorKind::InternalApiUnavailable, error_message).with_details(value)
    })
}

pub(super) fn normalize_button_text(text: &str) -> String {
    let trimmed = text.trim();
    let chars = trimmed.chars().collect::<Vec<_>>();
    if chars.len() % 2 == 0 {
        let midpoint = chars.len() / 2;
        if chars[..midpoint] == chars[midpoint..] {
            return chars[..midpoint].iter().collect();
        }
    }
    trimmed.to_string()
}

pub(super) async fn dispatch_ctrl_enter(
    runtime: &mut impl RuntimeEvaluator,
) -> Result<(), AppError> {
    dispatch_key(runtime, KeyEventType::KeyDown, "Enter", "Enter", 13, 2).await?;
    dispatch_key(runtime, KeyEventType::KeyUp, "Enter", "Enter", 13, 0).await
}

pub(super) async fn dispatch_key(
    runtime: &mut impl RuntimeEvaluator,
    event_type: KeyEventType,
    key: &'static str,
    code: &'static str,
    windows_virtual_key_code: i64,
    modifiers: i64,
) -> Result<(), AppError> {
    runtime
        .dispatch_key_event(KeyEvent {
            event_type,
            key,
            code,
            windows_virtual_key_code,
            modifiers,
        })
        .await
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::ops::test_support::FakeRuntime;

    use super::*;

    #[tokio::test]
    async fn ensure_pine_editor_open_errors_when_monaco_never_appears() {
        let mut responses = vec![json!(false), json!(true)];
        responses.extend(std::iter::repeat_n(json!(false), 50));
        let mut runtime = FakeRuntime::new(responses);

        let error = ensure_pine_editor_open(&mut runtime).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert!(error.message.contains("Could not open Pine Editor"));
        let open_attempts = runtime
            .evaluated
            .iter()
            .filter(|(expression, _)| expression.contains("activateScriptEditorTab"))
            .count();
        assert_eq!(open_attempts, 5);
    }

    #[tokio::test(start_paused = true)]
    async fn ensure_pine_editor_open_uses_one_absolute_deadline() {
        let mut runtime = FakeRuntime::new([json!(false), json!(true)])
            .with_evaluate_delay(Duration::from_secs(6));

        let error = ensure_pine_editor_open(&mut runtime).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(
            error.message,
            "Pine Editor readiness exceeded its bounded deadline"
        );
        assert_eq!(runtime.evaluated.len(), 2);
    }

    #[tokio::test]
    async fn ensure_pine_editor_open_sanitizes_runtime_evaluation_failure() {
        let runtime_error = AppError::new(ErrorKind::Connection, "private runtime exception")
            .with_details(json!({
                "exceptionDetails": {
                    "description": "private source and stack",
                    "scriptId": "private-script-id"
                }
            }));
        let mut runtime = FakeRuntime::new([]).with_evaluate_app_error(runtime_error);

        let error = ensure_pine_editor_open(&mut runtime).await.unwrap_err();

        assert_eq!(error.kind, ErrorKind::Connection);
        assert_eq!(error.message, "Pine Editor readiness evaluation failed");
        let serialized = error.details.unwrap().to_string();
        assert!(!serialized.contains("private runtime"));
        assert!(!serialized.contains("private source"));
        assert!(!serialized.contains("private-script-id"));
        assert!(serialized.contains("editor_readiness"));
    }

    #[test]
    fn find_monaco_includes_global_monaco_fast_path_and_fiber_fallback() {
        assert!(FIND_MONACO.contains("window.monaco.editor.getEditors"));
        assert!(FIND_MONACO.contains("getContainerDomNode"));
        assert!(FIND_MONACO.contains("[data-name=\"pine-dialog\"]"));
        assert!(FIND_MONACO.contains("hasTextFocus"));
        assert!(FIND_MONACO.contains("__reactFiber$"));
        assert!(FIND_MONACO.contains("props.value && props.value.monacoEnv"));
        assert!(!FIND_MONACO.contains("editors[0]"));
        assert!(!FIND_MONACO.contains("if (selected) return selected"));
    }
}
