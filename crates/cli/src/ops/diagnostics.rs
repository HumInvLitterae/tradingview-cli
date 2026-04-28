use serde_json::{Value, json};

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};

pub async fn discover(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    let paths = runtime
        .evaluate(
            r#"
            (function() {
                var results = {};
                try {
                    var chart = window.TradingViewApi._activeChartWidgetWV.value();
                    var methods = [];
                    for (var k in chart) { if (typeof chart[k] === 'function') methods.push(k); }
                    results.chartApi = { available: true, path: 'window.TradingViewApi._activeChartWidgetWV.value()', methodCount: methods.length, methods: methods.slice(0, 50) };
                } catch(e) { results.chartApi = { available: false, error: e.message }; }
                try {
                    var col = window.TradingViewApi._chartWidgetCollection;
                    var colMethods = [];
                    for (var k in col) { if (typeof col[k] === 'function') colMethods.push(k); }
                    results.chartWidgetCollection = { available: !!col, path: 'window.TradingViewApi._chartWidgetCollection', methodCount: colMethods.length, methods: colMethods.slice(0, 30) };
                } catch(e) { results.chartWidgetCollection = { available: false, error: e.message }; }
                try {
                    var ws = window.ChartApiInstance;
                    var wsMethods = [];
                    for (var k in ws) { if (typeof ws[k] === 'function') wsMethods.push(k); }
                    results.chartApiInstance = { available: !!ws, path: 'window.ChartApiInstance', methodCount: wsMethods.length, methods: wsMethods.slice(0, 30) };
                } catch(e) { results.chartApiInstance = { available: false, error: e.message }; }
                try {
                    var bwb = window.TradingView && window.TradingView.bottomWidgetBar;
                    var bwbMethods = [];
                    if (bwb) { for (var k in bwb) { if (typeof bwb[k] === 'function') bwbMethods.push(k); } }
                    results.bottomWidgetBar = { available: !!bwb, path: 'window.TradingView.bottomWidgetBar', methodCount: bwbMethods.length, methods: bwbMethods.slice(0, 20) };
                } catch(e) { results.bottomWidgetBar = { available: false, error: e.message }; }
                try {
                    var replay = window.TradingViewApi._replayApi;
                    results.replayApi = { available: !!replay, path: 'window.TradingViewApi._replayApi' };
                } catch(e) { results.replayApi = { available: false, error: e.message }; }
                try {
                    var alerts = window.TradingViewApi._alertService;
                    results.alertService = { available: !!alerts, path: 'window.TradingViewApi._alertService' };
                } catch(e) { results.alertService = { available: false, error: e.message }; }
                return results;
            })()
            "#,
            false,
        )
        .await?;
    let apis = paths.as_object().ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "TradingView API discovery did not return an object",
        )
    })?;
    let available = apis
        .values()
        .filter(|value| {
            value
                .get("available")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
        .count();
    Ok(json!({
        "apis_available": available,
        "apis_total": apis.len(),
        "apis": paths,
    }))
}

pub async fn ui_state(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            r#"
            (function() {
                var ui = {};
                var bottom = document.querySelector('[class*="layout__area--bottom"]');
                ui.bottom_panel = { open: !!(bottom && bottom.offsetHeight > 50), height: bottom ? bottom.offsetHeight : 0 };
                var right = document.querySelector('[class*="layout__area--right"]');
                ui.right_panel = { open: !!(right && right.offsetWidth > 50), width: right ? right.offsetWidth : 0 };
                var monacoEl = document.querySelector('.monaco-editor.pine-editor-monaco');
                ui.pine_editor = { open: !!monacoEl, width: monacoEl ? monacoEl.offsetWidth : 0, height: monacoEl ? monacoEl.offsetHeight : 0 };
                var stratPanel = document.querySelector('[data-name="backtesting"]') || document.querySelector('[class*="strategyReport"]');
                ui.strategy_tester = { open: !!(stratPanel && stratPanel.offsetParent) };
                var widgetbar = document.querySelector('[data-name="widgetbar-wrap"]');
                ui.widgetbar = { open: !!(widgetbar && widgetbar.offsetWidth > 50) };
                ui.buttons = {};
                var btns = document.querySelectorAll('button');
                var seen = {};
                for (var i = 0; i < btns.length; i++) {
                    var b = btns[i];
                    if (b.offsetParent === null || b.offsetWidth < 15) continue;
                    var text = b.textContent.trim();
                    var aria = b.getAttribute('aria-label') || '';
                    var dn = b.getAttribute('data-name') || '';
                    var label = text || aria || dn;
                    if (!label || label.length > 60) continue;
                    var key = label.replace(/[^a-zA-Z0-9 ]/g, '').substring(0, 40);
                    if (seen[key]) continue;
                    seen[key] = true;
                    var rect = b.getBoundingClientRect();
                    var region = 'other';
                    if (rect.y < 50) region = 'top_bar';
                    else if (rect.y < 90 && rect.x < 650) region = 'toolbar';
                    else if (rect.x < 45) region = 'left_sidebar';
                    else if (rect.x > 650 && rect.y < 100) region = 'pine_header';
                    else if (rect.y > 750) region = 'bottom_bar';
                    if (!ui.buttons[region]) ui.buttons[region] = [];
                    ui.buttons[region].push({ label: label.substring(0, 40), disabled: b.disabled, x: Math.round(rect.x), y: Math.round(rect.y) });
                }
                ui.key_buttons = {};
                var keyLabels = {
                    'add_to_chart': /add to chart/i, 'save_and_add': /save and add/i,
                    'update_on_chart': /update on chart/i, 'save': /^Save(Save)?$/,
                    'saved': /^Saved/, 'publish_script': /publish script/i,
                    'compile_errors': /error/i, 'unsaved_version': /unsaved version/i
                };
                for (var i = 0; i < btns.length; i++) {
                    var b = btns[i];
                    if (b.offsetParent === null) continue;
                    var text = b.textContent.trim();
                    for (var k in keyLabels) {
                        if (keyLabels[k].test(text)) {
                            ui.key_buttons[k] = { text: text.substring(0, 40), disabled: b.disabled, visible: b.offsetWidth > 0 };
                        }
                    }
                }
                try {
                    var chart = window.TradingViewApi._activeChartWidgetWV.value();
                    ui.chart = { symbol: chart.symbol(), resolution: chart.resolution(), chartType: chart.chartType(), study_count: chart.getAllStudies().length };
                } catch(e) { ui.chart = { error: e.message }; }
                try {
                    var replay = window.TradingViewApi._replayApi;
                    function unwrap(v) { return (v && typeof v === 'object' && typeof v.value === 'function') ? v.value() : v; }
                    ui.replay = { available: unwrap(replay.isReplayAvailable()), started: unwrap(replay.isReplayStarted()) };
                } catch(e) { ui.replay = { error: e.message }; }
                return ui;
            })()
            "#,
            false,
        )
        .await
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::test_support::FakeRuntime;
    use super::*;

    #[tokio::test]
    async fn discover_counts_available_api_paths() {
        let mut runtime = FakeRuntime::new([json!({
            "chartApi": {"available": true, "path": "window.TradingViewApi._activeChartWidgetWV.value()", "methodCount": 1, "methods": ["symbol"]},
            "chartWidgetCollection": {"available": true, "path": "window.TradingViewApi._chartWidgetCollection", "methodCount": 1, "methods": ["getAll"]},
            "chartApiInstance": {"available": false, "error": "missing"},
            "bottomWidgetBar": {"available": true, "path": "window.TradingView.bottomWidgetBar", "methodCount": 1, "methods": ["open"]},
            "replayApi": {"available": true, "path": "window.TradingViewApi._replayApi"},
            "alertService": {"available": false, "path": "window.TradingViewApi._alertService"}
        })]);

        let result = discover(&mut runtime).await.unwrap();

        assert_eq!(result["apis_available"], 4);
        assert_eq!(result["apis_total"], 6);
        assert_eq!(
            result["apis"]["chartApi"]["path"],
            "window.TradingViewApi._activeChartWidgetWV.value()"
        );
        assert!(runtime.evaluated[0].0.contains("_alertService"));
        assert!(!runtime.evaluated[0].1);
    }

    #[tokio::test]
    async fn ui_state_returns_runtime_payload() {
        let payload = json!({
            "bottom_panel": {"open": false, "height": 0},
            "right_panel": {"open": true, "width": 320},
            "pine_editor": {"open": false, "width": 0, "height": 0},
            "strategy_tester": {"open": false},
            "widgetbar": {"open": false},
            "buttons": {"top_bar": [{"label": "AAPL", "disabled": false, "x": 1, "y": 2}]},
            "key_buttons": {},
            "chart": {"symbol": "NASDAQ:AAPL", "resolution": "D", "chartType": 1, "study_count": 2},
            "replay": {"available": true, "started": false}
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);

        let result = ui_state(&mut runtime).await.unwrap();

        assert_eq!(result, payload);
        assert!(runtime.evaluated[0].0.contains("layout__area--bottom"));
        assert!(runtime.evaluated[0].0.contains("_replayApi"));
        assert!(!runtime.evaluated[0].1);
    }
}
