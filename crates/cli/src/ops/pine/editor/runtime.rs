use serde_json::{Value, json};
use std::time::Duration;

use tradingview_cdp::{KeyEvent, KeyEventType, RuntimeEvaluator};
use tradingview_core::{AppError, ErrorKind};

pub(super) const FIND_MONACO: &str = r#"
(function findMonacoEditor() {
    try {
        if (window.monaco && window.monaco.editor && typeof window.monaco.editor.getEditors === 'function') {
            var globalEditors = window.monaco.editor.getEditors();
            for (var g = 0; g < globalEditors.length; g++) {
                var editor = globalEditors[g];
                var node = typeof editor.getContainerDomNode === 'function' ? editor.getContainerDomNode() : null;
                if (node && node.closest && node.closest('.pine-editor-monaco')) {
                    return { editor: editor, env: { editor: window.monaco.editor } };
                }
            }
        }
    } catch(e) {}

    var container = document.querySelector('.monaco-editor.pine-editor-monaco');
    if (!container) return null;
    var el = container;
    var fiberKey;
    for (var i = 0; i < 20; i++) {
        if (!el) break;
        fiberKey = Object.keys(el).find(function(k) { return k.startsWith('__reactFiber$'); });
        if (fiberKey) break;
        el = el.parentElement;
    }
    if (!fiberKey) return null;
    var current = el[fiberKey];
    for (var d = 0; d < 15; d++) {
        if (!current) break;
        var env = null;
        if (current.memoizedProps && current.memoizedProps.monacoEnv) {
            env = current.memoizedProps.monacoEnv;
        } else if (current.memoizedProps && current.memoizedProps.value && current.memoizedProps.value.monacoEnv) {
            env = current.memoizedProps.value.monacoEnv;
        }
        if (env) {
            if (env.editor && typeof env.editor.getEditors === 'function') {
                var editors = env.editor.getEditors();
                if (editors.length > 0) return { editor: editors[0], env: env };
            }
        }
        current = current.return;
    }
    return null;
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
    let editor_open_before = runtime
        .evaluate(
            &with_monaco("var m = __FIND_MONACO__; return m !== null;"),
            false,
        )
        .await?
        .as_bool()
        .unwrap_or(false);
    if editor_open_before {
        return Ok(EditorOpenState {
            editor_open_before,
            opened_editor: false,
        });
    }

    runtime.evaluate(OPEN_PINE_PANEL_EXPRESSION, false).await?;

    for attempt in 0..50 {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let ready = runtime
            .evaluate(
                &with_monaco("var m = __FIND_MONACO__; return m !== null;"),
                false,
            )
            .await?
            .as_bool()
            .unwrap_or(false);
        if ready {
            return Ok(EditorOpenState {
                editor_open_before,
                opened_editor: true,
            });
        }
        if attempt > 0 && attempt % 10 == 0 {
            runtime.evaluate(OPEN_PINE_PANEL_EXPRESSION, false).await?;
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

    #[test]
    fn find_monaco_includes_global_monaco_fast_path_and_fiber_fallback() {
        assert!(FIND_MONACO.contains("window.monaco.editor.getEditors"));
        assert!(FIND_MONACO.contains("getContainerDomNode"));
        assert!(FIND_MONACO.contains("__reactFiber$"));
        assert!(FIND_MONACO.contains("memoizedProps.value.monacoEnv"));
    }
}
