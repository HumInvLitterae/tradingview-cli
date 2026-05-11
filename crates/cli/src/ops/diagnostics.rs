use serde_json::{Value, json};

use tradingview_cdp::{
    CdpClient, RuntimeEvaluator, Target, TargetSelection, TransportConfig, fetch_targets,
    select_target, target_title_for_handoff, target_url_for_handoff,
};
use tradingview_core::{AppError, ErrorKind};

use super::market::{QUOTE_DATA_CONTRACT_VERSION, quote_data_bounded_read, quote_symbol};

const DIAGNOSE_QUOTE_DATA_SOURCE: &str = "quote_data_diagnostics";

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

pub async fn diagnose_quote_data(
    config: &TransportConfig,
    symbol: &str,
) -> Result<Value, AppError> {
    let requested_symbol = symbol.trim();
    if requested_symbol.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "diagnose quote-data symbol must not be empty",
        ));
    }

    let scanner_reference = scanner_reference(requested_symbol).await;
    let targets = match fetch_targets(config).await {
        Ok(targets) => targets,
        Err(err) => {
            return Ok(diagnostic_payload(
                requested_symbol,
                "blocked",
                desktop_target_connection_error(config, &err),
                quote_data_not_attempted("desktop_target_unavailable"),
                scanner_reference,
                vec!["run_tab_list", "use_target_id"],
            ));
        }
    };

    let target = match resolve_diagnostic_target(config, &targets) {
        DiagnosticTarget::Selected { target, summary } => (target, summary),
        DiagnosticTarget::Blocked {
            status,
            summary,
            hints,
        } => {
            return Ok(diagnostic_payload(
                requested_symbol,
                "blocked",
                summary,
                quote_data_not_attempted(status),
                scanner_reference,
                hints,
            ));
        }
    };

    let (target, desktop_target) = target;
    let mut runtime = match CdpClient::connect(&target).await {
        Ok(runtime) => runtime,
        Err(err) => {
            return Ok(diagnostic_payload(
                requested_symbol,
                "blocked",
                desktop_target_connect_error(desktop_target, &err),
                quote_data_not_attempted("desktop_target_connection_error"),
                scanner_reference,
                vec!["run_tab_list", "use_target_id"],
            ));
        }
    };

    match quote_data_bounded_read(&mut runtime, requested_symbol).await {
        Ok(payload) => Ok(diagnostic_payload(
            requested_symbol,
            "available",
            desktop_target,
            quote_data_success(payload),
            scanner_reference,
            vec![],
        )),
        Err(err) if is_quote_data_unavailable(&err) => {
            let details = err.details.clone().unwrap_or_else(|| json!({}));
            let next_action_hints = next_action_hints_from_quote_data_details(&details);
            Ok(diagnostic_payload(
                requested_symbol,
                "unavailable",
                desktop_target,
                quote_data_unavailable(details),
                scanner_reference,
                next_action_hints,
            ))
        }
        Err(err) => Ok(diagnostic_payload(
            requested_symbol,
            "blocked",
            desktop_target,
            quote_data_blocked(&err),
            scanner_reference,
            vec!["retry_quote_data"],
        )),
    }
}

enum DiagnosticTarget {
    Selected {
        target: Target,
        summary: Value,
    },
    Blocked {
        status: &'static str,
        summary: Value,
        hints: Vec<&'static str>,
    },
}

fn resolve_diagnostic_target(config: &TransportConfig, targets: &[Target]) -> DiagnosticTarget {
    if let Some(target_id) = config.target_id.as_deref() {
        return targets
            .iter()
            .find(|target| target.id == target_id)
            .cloned()
            .map(|target| DiagnosticTarget::Selected {
                summary: desktop_target_selected(&target, true),
                target,
            })
            .unwrap_or_else(|| DiagnosticTarget::Blocked {
                status: "target_id_not_found",
                summary: desktop_target_blocked(
                    "target_id_not_found",
                    true,
                    targets,
                    "Run `tv tab list`, choose a current chart target, then retry with `tv --target-id <ID> diagnose quote-data <SYMBOL>`.",
                ),
                hints: vec!["run_tab_list", "use_target_id"],
            });
    }

    match select_target(targets) {
        TargetSelection::Selected(target) => DiagnosticTarget::Selected {
            summary: desktop_target_selected(&target, false),
            target,
        },
        TargetSelection::None => DiagnosticTarget::Blocked {
            status: "missing",
            summary: desktop_target_blocked(
                "missing",
                false,
                targets,
                "Run `tv tab list` to inspect available CDP targets, then retry with `tv --target-id <ID> diagnose quote-data <SYMBOL>` when a chart target is available.",
            ),
            hints: vec!["run_tab_list"],
        },
        TargetSelection::Ambiguous(targets) => DiagnosticTarget::Blocked {
            status: "ambiguous",
            summary: desktop_target_blocked(
                "ambiguous",
                false,
                &targets,
                "Run `tv tab list`, choose the intended chart target, then retry with `tv --target-id <ID> diagnose quote-data <SYMBOL>`.",
            ),
            hints: vec!["run_tab_list", "use_target_id"],
        },
    }
}

fn diagnostic_payload(
    requested_symbol: &str,
    diagnostic_status: &str,
    desktop_target: Value,
    quote_data: Value,
    scanner_reference: Value,
    next_action_hints: Vec<&str>,
) -> Value {
    json!({
        "source": DIAGNOSE_QUOTE_DATA_SOURCE,
        "source_category": "desktop_backed_read",
        "requires_desktop": true,
        "non_mutating": true,
        "requested_symbol": requested_symbol,
        "diagnostic_status": diagnostic_status,
        "desktop_target": desktop_target,
        "quote_data_contract_version": QUOTE_DATA_CONTRACT_VERSION,
        "quote_data": quote_data,
        "scanner_reference": scanner_reference,
        "scanner_values_merged": false,
        "chart_values_merged": false,
        "quote_data_added_to_auto": false,
        "next_action_hints": next_action_hints,
    })
}

fn desktop_target_selected(target: &Target, target_id_specified: bool) -> Value {
    json!({
        "status": "selected",
        "target_id_specified": target_id_specified,
        "selected": sanitized_target(target),
        "next_action_hint": Value::Null,
    })
}

fn desktop_target_blocked(
    status: &str,
    target_id_specified: bool,
    targets: &[Target],
    next_action_hint: &str,
) -> Value {
    json!({
        "status": status,
        "target_id_specified": target_id_specified,
        "candidate_count": targets.len(),
        "candidates": targets.iter().map(sanitized_target).collect::<Vec<_>>(),
        "next_action_hint": next_action_hint,
    })
}

fn desktop_target_connection_error(config: &TransportConfig, err: &AppError) -> Value {
    json!({
        "status": "connection_error",
        "target_id_specified": config.target_id.as_ref().is_some_and(|value| !value.trim().is_empty()),
        "cdp_host": config.host,
        "cdp_port": config.port,
        "error": public_error(err),
        "next_action_hint": "Run `tv status` to confirm the CDP endpoint, or run `tv launch` to start TradingView Desktop with remote debugging enabled.",
    })
}

fn desktop_target_connect_error(mut selected: Value, err: &AppError) -> Value {
    selected["status"] = json!("connection_error");
    selected["error"] = public_error(err);
    selected["next_action_hint"] = json!(
        "Run `tv tab list`, choose a current chart target, then retry with `tv --target-id <ID> diagnose quote-data <SYMBOL>`."
    );
    selected
}

fn sanitized_target(target: &Target) -> Value {
    json!({
        "title": target_title_for_handoff(target),
        "type": target.kind,
        "url": target_url_for_handoff(target),
        "has_websocket_debugger_url": target.web_socket_debugger_url.is_some(),
    })
}

async fn scanner_reference(symbol: &str) -> Value {
    match quote_symbol(symbol).await {
        Ok(payload) => json!({
            "ok": true,
            "source": payload.get("source").cloned().unwrap_or(Value::Null),
            "source_category": payload.get("source_category").cloned().unwrap_or(Value::Null),
            "update_mode": payload.get("update_mode").cloned().unwrap_or(Value::Null),
            "delay_seconds": payload.get("delay_seconds").cloned().unwrap_or(Value::Null),
            "time": payload.get("time").cloned().unwrap_or(Value::Null),
            "extended_hours_included": payload.get("extended_hours").is_some(),
            "merged_with_quote_data": false,
        }),
        Err(err) => json!({
            "ok": false,
            "error": public_error(&err),
            "merged_with_quote_data": false,
        }),
    }
}

fn quote_data_success(payload: Value) -> Value {
    json!({
        "ok": true,
        "payload_status": "available",
        "source": payload.get("source").cloned().unwrap_or(Value::Null),
        "observed_symbol": payload.get("observed_symbol").cloned().unwrap_or(Value::Null),
        "source_availability": payload.get("source_availability").cloned().unwrap_or(Value::Null),
        "readback": payload.get("quote_data").cloned().unwrap_or(Value::Null),
    })
}

fn quote_data_unavailable(details: Value) -> Value {
    json!({
        "ok": false,
        "payload_status": "unavailable",
        "source": details.get("source").cloned().unwrap_or(Value::Null),
        "observed_symbol": details.get("observed_symbol").cloned().unwrap_or(Value::Null),
        "source_availability": details.get("source_availability").cloned().unwrap_or(Value::Null),
        "unavailable_reason": details
            .pointer("/source_availability/unavailable_reason")
            .cloned()
            .unwrap_or(Value::Null),
        "wait_summary": details.get("wait_summary").cloned().unwrap_or(Value::Null),
    })
}

fn quote_data_not_attempted(reason: &str) -> Value {
    json!({
        "ok": false,
        "payload_status": "not_attempted",
        "not_attempted_reason": reason,
        "source_availability": Value::Null,
    })
}

fn quote_data_blocked(err: &AppError) -> Value {
    json!({
        "ok": false,
        "payload_status": "blocked",
        "error": public_error(err),
        "source_availability": Value::Null,
    })
}

fn is_quote_data_unavailable(err: &AppError) -> bool {
    err.kind == ErrorKind::InternalApiUnavailable
        && err
            .details
            .as_ref()
            .and_then(|details| details.get("source"))
            .and_then(Value::as_str)
            == Some("desktop_quote_data_ws")
}

fn next_action_hints_from_quote_data_details(details: &Value) -> Vec<&'static str> {
    match details
        .pointer("/source_availability/next_action")
        .and_then(Value::as_str)
    {
        Some("check_desktop_streaming_symbol") => vec!["check_desktop_streaming_symbol"],
        Some("use_scanner_if_delayed_rest_ok") => vec!["use_scanner_if_delayed_rest_ok"],
        _ => vec!["retry_quote_data"],
    }
}

fn public_error(err: &AppError) -> Value {
    json!({
        "kind": err.kind,
        "message": err.message,
    })
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
    use tradingview_cdp::{Target, TransportConfig};
    use tradingview_core::{AppError, ErrorKind};

    use super::super::test_support::FakeRuntime;
    use super::*;

    fn target(id: &str, url: &str) -> Target {
        Target {
            id: id.to_string(),
            title: "TradingView chart".to_string(),
            kind: "page".to_string(),
            url: url.to_string(),
            web_socket_debugger_url: Some(format!("ws://example/{id}")),
        }
    }

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

    #[test]
    fn diagnose_target_resolution_reports_ambiguous_without_raw_ids() {
        let config = TransportConfig::default();
        let targets = vec![
            target("secret-a", "https://www.tradingview.com/chart/aaa"),
            target("secret-b", "https://www.tradingview.com/chart/bbb"),
        ];

        let DiagnosticTarget::Blocked {
            status, summary, ..
        } = resolve_diagnostic_target(&config, &targets)
        else {
            panic!("expected blocked target resolution");
        };

        assert_eq!(status, "ambiguous");
        assert_eq!(summary["status"], "ambiguous");
        assert_eq!(summary["candidate_count"], 2);
        assert!(!summary.to_string().contains("secret-a"));
        assert!(!summary.to_string().contains("secret-b"));
        assert_eq!(summary["candidates"][0]["has_websocket_debugger_url"], true);
    }

    #[test]
    fn diagnose_target_resolution_reports_target_id_not_found_without_echoing_id() {
        let config = TransportConfig {
            target_id: Some("private-target".to_string()),
            ..TransportConfig::default()
        };
        let targets = vec![target(
            "other-target",
            "https://www.tradingview.com/chart/aaa",
        )];

        let DiagnosticTarget::Blocked {
            status, summary, ..
        } = resolve_diagnostic_target(&config, &targets)
        else {
            panic!("expected blocked target resolution");
        };

        assert_eq!(status, "target_id_not_found");
        assert_eq!(summary["status"], "target_id_not_found");
        assert_eq!(summary["target_id_specified"], true);
        assert!(!summary.to_string().contains("private-target"));
        assert!(!summary.to_string().contains("other-target"));
    }

    #[test]
    fn diagnose_payload_wraps_quote_data_unavailable_details() {
        let details = json!({
            "source": "desktop_quote_data_ws",
            "observed_symbol": null,
            "source_availability": {
                "available": false,
                "status": "unavailable",
                "rtc_observed": false,
                "unavailable_reason": "no_rtc",
                "timed_out": true,
                "next_action": "use_scanner_if_delayed_rest_ok",
                "raw_frame_included": false,
                "wait_summary": {
                    "websocket_events_seen": 1,
                    "websocket_frames_seen": 1,
                    "qsd_messages_seen": 1,
                    "matching_symbol_qsd_seen": 1,
                    "raw_frame_included": false
                }
            },
            "wait_summary": {
                "websocket_events_seen": 1,
                "websocket_frames_seen": 1,
                "qsd_messages_seen": 1,
                "matching_symbol_qsd_seen": 1,
                "raw_frame_included": false
            }
        });

        let wrapped = quote_data_unavailable(details.clone());

        assert_eq!(wrapped["ok"], false);
        assert_eq!(wrapped["payload_status"], "unavailable");
        assert_eq!(wrapped["unavailable_reason"], "no_rtc");
        assert_eq!(
            wrapped["source_availability"]["wait_summary"]["raw_frame_included"],
            false
        );
        assert!(!wrapped.to_string().contains("payloadData"));
    }

    #[test]
    fn diagnose_payload_reports_connection_error_as_blocked_status() {
        let err = AppError::new(ErrorKind::Connection, "connection refused");
        let target = desktop_target_connection_error(&TransportConfig::default(), &err);
        let payload = diagnostic_payload(
            "NASDAQ:RKLB",
            "blocked",
            target,
            quote_data_not_attempted("desktop_target_unavailable"),
            json!({"ok": false}),
            vec!["run_tab_list", "use_target_id"],
        );

        assert_eq!(payload["diagnostic_status"], "blocked");
        assert_eq!(payload["desktop_target"]["status"], "connection_error");
        assert_eq!(payload["quote_data"]["payload_status"], "not_attempted");
        assert_eq!(payload["quote_data_added_to_auto"], false);
    }
}
