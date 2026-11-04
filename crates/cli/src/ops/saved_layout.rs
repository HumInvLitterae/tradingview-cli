use serde_json::{Value, json};

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};

use super::common::js_string;

pub async fn saved_layout_list(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            r#"
            new Promise(function(resolve) {
                try {
                    var api = window.TradingViewApi;
                    if (!api || typeof api.getSavedCharts !== 'function') {
                        resolve({
                            layout_count: 0,
                            source: 'internal_api',
                            layouts: [],
                            error: 'getSavedCharts is unavailable'
                        });
                        return;
                    }

                    var settled = false;
                    function finish(payload) {
                        if (settled) return;
                        settled = true;
                        resolve(payload);
                    }

                    api.getSavedCharts(function(charts) {
                        if (!Array.isArray(charts)) {
                            finish({
                                layout_count: 0,
                                source: 'internal_api',
                                layouts: [],
                                error: 'getSavedCharts returned no data'
                            });
                            return;
                        }

                        var layouts = charts.map(function(chart) {
                            return {
                                id: chart.id || chart.chartId || null,
                                name: chart.name || chart.title || 'Untitled',
                                url: chart.url || chart.image_url || null,
                                symbol: chart.symbol || null,
                                resolution: chart.resolution || null,
                                modified: chart.timestamp || chart.modified || null
                            };
                        });

                        finish({
                            layout_count: layouts.length,
                            source: 'internal_api',
                            layouts: layouts
                        });
                    });

                    setTimeout(function() {
                        finish({
                            layout_count: 0,
                            source: 'internal_api',
                            layouts: [],
                            error: 'getSavedCharts timed out'
                        });
                    }, 5000);
                } catch (error) {
                    resolve({
                        layout_count: 0,
                        source: 'internal_api',
                        layouts: [],
                        error: error && error.message ? error.message : String(error)
                    });
                }
            })
            "#,
            true,
        )
        .await
        .map(normalize_saved_layout_list_payload)
}

pub async fn saved_layout_switch(
    runtime: &mut impl RuntimeEvaluator,
    target: &str,
    dry_run: bool,
) -> Result<Value, AppError> {
    let target = target.trim();
    if target.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Layout target must not be empty",
        ));
    }

    let target_literal = js_string(target)?;
    let expression = format!(
        r#"
            new Promise(function(resolve) {{
                try {{
                    var api = window.TradingViewApi;
                    var target = {target_literal};
                    var dryRun = {dry_run};
                    if (!api || typeof api.getSavedCharts !== 'function') {{
                        resolve({{
                            error: 'getSavedCharts is unavailable',
                            error_kind: 'internal_api_unavailable',
                            source: 'internal_api'
                        }});
                        return;
                    }}
                    if (!dryRun && typeof api.loadChartFromServer !== 'function') {{
                        resolve({{
                            error: 'loadChartFromServer is unavailable',
                            error_kind: 'internal_api_unavailable',
                            source: 'internal_api'
                        }});
                        return;
                    }}

                    var settled = false;
                    function finish(payload) {{
                        if (settled) return;
                        settled = true;
                        resolve(payload);
                    }}

                    function normalizeChart(chart) {{
                        return {{
                            id: chart.id || chart.chartId || null,
                            name: chart.name || chart.title || 'Untitled',
                            url: chart.url || chart.image_url || null,
                            symbol: chart.symbol || null,
                            resolution: chart.resolution || null,
                            modified: chart.timestamp || chart.modified || null
                        }};
                    }}

                    function findUnsavedDialog() {{
                        var dialogs = Array.from(document.querySelectorAll('[role="dialog"], [class*="dialog"], [class*="modal"], [class*="popup"]'));
                        for (var i = 0; i < dialogs.length; i++) {{
                            var text = (dialogs[i].textContent || '').trim();
                            if (/unsaved|save changes|変更|保存/i.test(text)) return true;
                        }}
                        return false;
                    }}

                    api.getSavedCharts(function(charts) {{
                        if (!Array.isArray(charts)) {{
                            finish({{
                                error: 'getSavedCharts returned no data',
                                error_kind: 'internal_api_unavailable',
                                source: 'internal_api',
                                target: target
                            }});
                            return;
                        }}

                        var normalized = charts.map(normalizeChart);
                        var matches = normalized.filter(function(layout) {{
                            return layout.id !== null && String(layout.id) === String(target);
                        }});

                        if (matches.length === 0) {{
                            var lowered = String(target).toLowerCase();
                            matches = normalized.filter(function(layout) {{
                                return String(layout.name || '').toLowerCase() === lowered;
                            }});
                        }}

                        if (matches.length === 0) {{
                            finish({{
                                error: 'Layout not found: ' + target,
                                error_kind: 'validation',
                                source: 'internal_api',
                                target: target,
                                layout_count: normalized.length
                            }});
                            return;
                        }}
                        if (matches.length > 1) {{
                            finish({{
                                error: 'Multiple layouts match: ' + target,
                                error_kind: 'validation',
                                source: 'internal_api',
                                target: target,
                                matches: matches
                            }});
                            return;
                        }}

                        var match = matches[0];
                        var rawMatch = charts[normalized.indexOf(match)];
                        if (match.id === null || match.id === undefined || String(match.id).trim() === '') {{
                            finish({{
                                error: 'Matched layout does not include an id',
                                error_kind: 'internal_api_unavailable',
                                source: 'internal_api',
                                target: target,
                                layout: match
                            }});
                            return;
                        }}

                        if (dryRun) {{
                            finish({{
                                action: 'dry_run',
                                dry_run: true,
                                target: target,
                                layout: match,
                                layout_id: match.id,
                                layout_url: match.url,
                                source: 'internal_api',
                                layout_count: normalized.length
                            }});
                            return;
                        }}

                        var method = 'loadChartFromServer';
                        var chartUrl = match.url === null || match.url === undefined ? '' : String(match.url).trim();
                        if (/^[A-Za-z0-9_-]+$/.test(chartUrl)) {{
                            method = 'location.assign';
                            setTimeout(function() {{
                                window.location.assign('/chart/' + chartUrl + '/');
                            }}, 50);
                            finish({{
                                action: 'switched',
                                dry_run: false,
                                target: target,
                                layout: match,
                                layout_id: match.id,
                                layout_url: chartUrl,
                                source: 'internal_api',
                                method: method,
                                navigation_expected: true,
                                unsaved_dialog_observed: findUnsavedDialog(),
                                unsaved_dialog_dismissed: false
                            }});
                            return;
                        }} else if (api._loadChartService && typeof api._loadChartService.loadChart === 'function' && rawMatch) {{
                            method = '_loadChartService.loadChart';
                            api._loadChartService.loadChart(rawMatch, false, false);
                        }} else {{
                            api.loadChartFromServer(String(match.id));
                        }}
                        setTimeout(function() {{
                            finish({{
                                action: 'switched',
                                dry_run: false,
                                target: target,
                                layout: match,
                                layout_id: match.id,
                                layout_url: chartUrl || null,
                                source: 'internal_api',
                                method: method,
                                navigation_expected: method === 'location.assign',
                                unsaved_dialog_observed: findUnsavedDialog(),
                                unsaved_dialog_dismissed: false
                            }});
                        }}, 500);
                    }});

                    setTimeout(function() {{
                        finish({{
                            error: 'getSavedCharts timed out',
                            error_kind: 'internal_api_unavailable',
                            source: 'internal_api',
                            target: target
                        }});
                    }}, 5000);
                }} catch (error) {{
                    resolve({{
                        error: error && error.message ? error.message : String(error),
                        error_kind: 'internal_api_unavailable',
                        source: 'internal_api',
                        target: {target_literal}
                    }});
                }}
            }})
            "#
    );

    let data = runtime.evaluate(&expression, true).await?;
    normalize_saved_layout_switch_payload(data)
}

fn normalize_saved_layout_list_payload(data: Value) -> Value {
    let layouts = data
        .get("layouts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut payload = json!({
        "layout_count": data
            .get("layout_count")
            .and_then(Value::as_u64)
            .unwrap_or(layouts.len() as u64),
        "source": data
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or("internal_api"),
        "layouts": layouts,
    });

    if let Some(error) = data.get("error").cloned() {
        payload["error"] = error;
    }

    payload
}

fn normalize_saved_layout_switch_payload(data: Value) -> Result<Value, AppError> {
    if let Some(message) = data.get("error").and_then(Value::as_str) {
        let kind = match data.get("error_kind").and_then(Value::as_str) {
            Some("validation") => ErrorKind::Validation,
            _ => ErrorKind::InternalApiUnavailable,
        };
        return Err(AppError::new(kind, message.to_string()).with_details(data));
    }

    let layout_id = data.get("layout_id").cloned().unwrap_or(Value::Null);
    if layout_id.is_null() {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Layout switch payload did not include layout_id",
        )
        .with_details(data));
    }

    Ok(json!({
        "action": data.get("action").cloned().unwrap_or_else(|| json!("switched")),
        "dry_run": data.get("dry_run").and_then(Value::as_bool).unwrap_or(false),
        "target": data.get("target").cloned().unwrap_or(Value::Null),
        "layout": data.get("layout").cloned().unwrap_or(Value::Null),
        "layout_id": layout_id,
        "layout_url": data.get("layout_url").cloned().unwrap_or(Value::Null),
        "source": data.get("source").and_then(Value::as_str).unwrap_or("internal_api"),
        "method": data.get("method").cloned().unwrap_or(Value::Null),
        "navigation_expected": data
            .get("navigation_expected")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        "layout_count": data.get("layout_count").cloned().unwrap_or(Value::Null),
        "unsaved_dialog_observed": data
            .get("unsaved_dialog_observed")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        "unsaved_dialog_dismissed": data
            .get("unsaved_dialog_dismissed")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    }))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::test_support::{FakeRuntime, run_node_json};
    use super::*;

    #[tokio::test]
    async fn saved_layout_list_returns_normalized_payload() {
        let payload = json!({
            "layout_count": 2,
            "source": "internal_api",
            "layouts": [
                {
                    "id": "chart-1",
                    "name": "Swing Layout",
                    "url": "abc123",
                    "symbol": "NASDAQ:AAPL",
                    "resolution": "1D",
                    "modified": 1777000000
                },
                {
                    "id": "chart-2",
                    "name": "Intraday",
                    "url": "def456",
                    "symbol": null,
                    "resolution": "15",
                    "modified": null
                }
            ]
        });
        let mut runtime = FakeRuntime::new([payload.clone()]);

        let result = saved_layout_list(&mut runtime).await.unwrap();

        assert_eq!(result, payload);
        assert!(runtime.evaluated[0].0.contains("getSavedCharts"));
        assert!(!runtime.evaluated[0].0.contains("loadChartFromServer"));
        assert!(runtime.evaluated[0].1);
    }

    #[tokio::test]
    async fn saved_layout_list_preserves_error_payload() {
        let mut runtime = FakeRuntime::new([json!({
            "layout_count": 0,
            "source": "internal_api",
            "layouts": [],
            "error": "getSavedCharts timed out"
        })]);

        let result = saved_layout_list(&mut runtime).await.unwrap();

        assert_eq!(result["layout_count"], 0);
        assert_eq!(result["source"], "internal_api");
        assert_eq!(result["layouts"].as_array().unwrap().len(), 0);
        assert_eq!(result["error"], "getSavedCharts timed out");
    }

    #[tokio::test]
    async fn saved_layout_list_defaults_malformed_payload_to_empty_list() {
        let mut runtime = FakeRuntime::new([json!({
            "source": "internal_api"
        })]);

        let result = saved_layout_list(&mut runtime).await.unwrap();

        assert_eq!(result["layout_count"], 0);
        assert_eq!(result["source"], "internal_api");
        assert_eq!(result["layouts"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    #[ignore = "run through `node scripts/check-js-contracts.mjs desktop`"]
    async fn javascript_desktop_saved_layout_switch_loads_only_the_matched_layout() {
        let mut expressions = Vec::new();
        for (target, dry_run) in [
            ("Swing Layout", true),
            ("swing layout", false),
            ("chart-2", false),
            ("Twin", false),
        ] {
            let mut runtime = FakeRuntime::new([]);
            let _ = saved_layout_switch(&mut runtime, target, dry_run).await;
            expressions.push(runtime.evaluated[0].0.clone());
        }
        let expressions = serde_json::to_string(&expressions).unwrap();
        let results = run_node_json(&format!(
            r#"
            const expressions = {expressions};
            // Shorten the expression's waits while keeping their order.
            const realSetTimeout = setTimeout;
            global.setTimeout = (callback, ms) => realSetTimeout(callback, ms / 100);
            const charts = [
                {{ id: 'chart-1', name: 'Swing Layout', url: 'abc123' }},
                {{ id: 'chart-2', name: 'Scalping', url: null }},
                {{ id: 'chart-3', name: 'Twin', url: 'twin1' }},
                {{ id: 'chart-4', name: 'twin', url: 'twin2' }}
            ];
            function desktop(withService) {{
                const calls = [];
                const dialog = {{ textContent: 'Save changes?', click: () => calls.push(['dialog.click']) }};
                global.document = {{ querySelectorAll: () => [dialog] }};
                global.window = {{
                    location: {{ assign: url => calls.push(['location.assign', url]) }},
                    TradingViewApi: {{
                        getSavedCharts: callback => callback(charts),
                        loadChartFromServer: id => calls.push(['loadChartFromServer', id]),
                        _loadChartService: withService
                            ? {{ loadChart: (chart, a, b) => calls.push(['loadChart', chart.id, a, b]) }}
                            : undefined
                    }}
                }};
                return calls;
            }}
            (async () => {{
                const results = [];
                for (const [index, withService] of [[0, true], [1, true], [2, true], [2, false], [3, true]]) {{
                    const calls = desktop(withService);
                    const result = await eval(expressions[index]);
                    await new Promise(resolve => realSetTimeout(resolve, 20));
                    results.push({{ result, calls }});
                }}
                console.log(JSON.stringify(results));
            }})().catch(error => {{ console.error(error); process.exitCode = 1; }});
            "#
        ));

        // A dry run resolves the layout without loading or navigating.
        assert_eq!(results[0]["calls"], json!([]));
        assert_eq!(results[0]["result"]["layout_id"], "chart-1");
        // A layout with a short URL navigates there once; the unsaved-changes
        // dialog is reported, never confirmed.
        assert_eq!(
            results[1]["calls"],
            json!([["location.assign", "/chart/abc123/"]])
        );
        assert_eq!(results[1]["result"]["unsaved_dialog_observed"], true);
        assert_eq!(results[1]["result"]["unsaved_dialog_dismissed"], false);
        // Without a URL the layout is loaded by its own record or ID.
        assert_eq!(
            results[2]["calls"],
            json!([["loadChart", "chart-2", false, false]])
        );
        assert_eq!(
            results[3]["calls"],
            json!([["loadChartFromServer", "chart-2"]])
        );
        // Two layouts whose names differ only in case are ambiguous.
        assert_eq!(results[4]["calls"], json!([]));
        assert_eq!(
            results[4]["result"]["error"],
            "Multiple layouts match: Twin"
        );
    }

    #[tokio::test]
    async fn saved_layout_switch_returns_dry_run_payload() {
        let mut runtime = FakeRuntime::new([json!({
            "action": "dry_run",
            "dry_run": true,
            "target": "Swing Layout",
            "layout": {
                "id": "chart-1",
                "name": "Swing Layout",
                "url": "abc123"
            },
            "layout_id": "chart-1",
            "layout_url": "abc123",
            "source": "internal_api",
            "layout_count": 2
        })]);

        let result = saved_layout_switch(&mut runtime, "Swing Layout", true)
            .await
            .unwrap();

        assert_eq!(result["action"], "dry_run");
        assert_eq!(result["dry_run"], true);
        assert_eq!(result["layout_id"], "chart-1");
        assert_eq!(result["layout_url"], "abc123");
        assert_eq!(result["layout"]["name"], "Swing Layout");
        assert!(runtime.evaluated[0].0.contains("getSavedCharts"));
        assert!(runtime.evaluated[0].0.contains("loadChartFromServer"));
        assert!(runtime.evaluated[0].0.contains("\"Swing Layout\""));
        assert!(runtime.evaluated[0].1);
    }

    #[tokio::test]
    async fn saved_layout_switch_returns_switched_payload() {
        let mut runtime = FakeRuntime::new([json!({
            "action": "switched",
            "dry_run": false,
            "target": "chart-1",
            "layout": {
                "id": "chart-1",
                "name": "Swing Layout",
                "url": "abc123"
            },
            "layout_id": "chart-1",
            "layout_url": "abc123",
            "source": "internal_api",
            "method": "location.assign",
            "navigation_expected": true,
            "unsaved_dialog_observed": true,
            "unsaved_dialog_dismissed": false
        })]);

        let result = saved_layout_switch(&mut runtime, "chart-1", false)
            .await
            .unwrap();

        assert_eq!(result["action"], "switched");
        assert_eq!(result["dry_run"], false);
        assert_eq!(result["layout_id"], "chart-1");
        assert_eq!(result["layout_url"], "abc123");
        assert_eq!(result["method"], "location.assign");
        assert_eq!(result["navigation_expected"], true);
        assert_eq!(result["unsaved_dialog_observed"], true);
        assert_eq!(result["unsaved_dialog_dismissed"], false);
    }

    #[tokio::test]
    async fn saved_layout_switch_maps_missing_target_to_validation() {
        let mut runtime = FakeRuntime::new([json!({
            "error": "Layout not found: missing",
            "error_kind": "validation",
            "source": "internal_api",
            "target": "missing"
        })]);

        let error = saved_layout_switch(&mut runtime, "missing", true)
            .await
            .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
    }

    #[tokio::test]
    async fn saved_layout_switch_rejects_empty_target_before_evaluating() {
        let mut runtime = FakeRuntime::new([]);

        let error = saved_layout_switch(&mut runtime, " ", true)
            .await
            .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(runtime.evaluated.is_empty());
    }
}
