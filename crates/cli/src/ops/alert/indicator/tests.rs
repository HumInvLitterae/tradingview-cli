use std::collections::VecDeque;

use serde_json::json;

use super::super::super::test_support::FakeRuntime;
use super::*;

#[tokio::test]
async fn alert_indicator_rejects_mismatched_saved_source_before_alert_operations() {
    let local = "indicator(\"Signals\")\nplot(close)\nalertcondition(close > open, \"Long\")";
    let saved = "indicator(\"Signals\")\nalertcondition(close > open, \"Long\")";
    for dry_run in [true, false] {
        let mut runtime = FakeRuntime::new([
            json!({"match_count": 1, "match": {
                "name": "Signals", "script_id": "synthetic-script", "version": 4,
                "script_id_available": true
            }}),
            json!({"source": saved}),
        ]);
        let error = alert_create_indicator(
            &mut runtime,
            IndicatorAlertRequest {
                script: "Signals",
                source: local,
                input_source: "stdin",
                condition_title: Some("Long"),
                alert_cond_id: None,
                symbol: None,
                resolution: None,
                message: None,
                dry_run,
            },
        )
        .await
        .expect_err("a different saved source must prevent preview and creation");

        assert_eq!(error.kind, ErrorKind::Validation);
        assert_eq!(
            error.details,
            Some(json!({
                "phase": "saved_source_verification", "reason": "source_mismatch"
            }))
        );
        assert_eq!(runtime.evaluated.len(), 2);
        assert!(
            runtime
                .evaluated
                .iter()
                .all(|(expression, _)| { !expression.contains("pricealerts.tradingview.com") })
        );
    }
}

#[tokio::test]
async fn alert_indicator_dry_run_returns_sanitized_preview() {
    let source = r#"//@version=6
indicator("Signals")
plot(close)
alertcondition(close > open, "Long", "Long message")"#;
    let mut runtime = FakeRuntime::new(VecDeque::from([
        json!({
            "requested": "Signals",
            "match_count": 1,
            "match": {
                "name": "Signals",
                "title": "Signals",
                "version": 4,
                "modified": 123,
                "script_id": "synthetic-script",
                "script_id_available": true
            },
            "candidates": []
        }),
        json!({"source": source}),
    ]));

    let data = alert_create_indicator(
        &mut runtime,
        IndicatorAlertRequest {
            script: "Signals",
            source,
            input_source: "stdin",
            condition_title: Some("Long"),
            alert_cond_id: None,
            symbol: Some("NASDAQ:AAPL"),
            resolution: Some("1D"),
            message: Some("Test alert"),
            dry_run: true,
        },
    )
    .await
    .unwrap();

    assert_eq!(data["action"], "dry_run");
    assert_eq!(data["dry_run"], true);
    assert_eq!(data["would_create"], true);
    assert_eq!(data["mutation_supported"], true);
    assert_eq!(data["script"]["requested"], "Signals");
    assert_eq!(data["script"]["name"], "Signals");
    assert_eq!(data["script"]["script_id_available"], true);
    assert!(data["script"].get("id").is_none());
    assert_eq!(data["condition"]["alert_cond_id"], "plot_1");
    assert_eq!(data["condition"]["title"], "Long");
    assert_eq!(data["request"]["symbol"], "NASDAQ:AAPL");
    assert!(runtime.evaluated[0].0.contains("pine-facade/list"));
    assert!(data["script"].get("script_id").is_none());
}

#[tokio::test]
async fn alert_indicator_dry_run_rejects_ambiguous_condition_selector() {
    let mut runtime = FakeRuntime::new(VecDeque::new());

    let error = alert_create_indicator(
        &mut runtime,
        IndicatorAlertRequest {
            script: "Signals",
            source: "alertcondition(close > open, \"Long\")",
            input_source: "stdin",
            condition_title: Some("Long"),
            alert_cond_id: Some("plot_0"),
            symbol: None,
            resolution: None,
            message: None,
            dry_run: true,
        },
    )
    .await
    .unwrap_err();

    assert_eq!(error.kind, ErrorKind::Validation);
    assert!(runtime.evaluated.is_empty());
}

#[tokio::test]
async fn alert_indicator_dry_run_rejects_missing_saved_script_match() {
    let source = "alertcondition(close > open, \"Long\")";
    let mut runtime = FakeRuntime::new(VecDeque::from([json!({
        "requested": "Signals",
        "match_count": 0,
        "match": null,
        "candidates": [
            { "name": "Other", "title": "Other", "version": 1, "modified": null, "script_id_available": true }
        ]
    })]));

    let error = alert_create_indicator(
        &mut runtime,
        IndicatorAlertRequest {
            script: "Signals",
            source,
            input_source: "stdin",
            condition_title: Some("Long"),
            alert_cond_id: None,
            symbol: None,
            resolution: None,
            message: None,
            dry_run: true,
        },
    )
    .await
    .unwrap_err();

    assert_eq!(error.kind, ErrorKind::Validation);
    assert_eq!(error.message, "No saved Pine script matches --script");
    assert_eq!(error.details.unwrap()["match_count"], 0);
}

#[tokio::test]
async fn alert_indicator_create_returns_sanitized_success() {
    let source = r#"//@version=6
indicator("Signals")
plot(close)
alertcondition(close > open, "Long", "Long message")"#;
    let mut runtime = FakeRuntime::new(VecDeque::from([
        json!({
            "requested": "Signals",
            "match_count": 1,
            "match": {
                "name": "Signals",
                "title": "Signals",
                "version": 4,
                "modified": 123,
                "script_id": "SAVED_SCRIPT_ID_REDACTED",
                "script_id_available": true
            },
            "candidates": []
        }),
        json!({"source": source}),
        json!({
            "action": "create_indicator",
            "dry_run": false,
            "alert_id": "4550000001",
            "created": true,
            "source": "indicator_alert_api",
            "symbol": "NASDAQ:AAPL",
            "resolution": "1D",
            "message": "Long message",
            "before_count": 1,
            "after_count": 2,
            "script": {
                "requested": "Signals",
                "name": "Signals",
                "title": "Signals",
                "version": "4",
                "script_id_available": true
            },
            "condition": {
                "alert_cond_id": "plot_1",
                "title": "Long",
                "message": "Long message",
                "plot_index": 1,
                "confidence": "best_effort"
            },
            "input_metadata": {
                "source": "default_no_inputs",
                "input_count": 0,
                "study_matched": false,
                "source_has_inputs": false
            },
            "matched_alert": {
                "alert_id": "4550000001",
                "message": "Long message",
                "condition": {
                    "type": "alert_cond",
                    "alert_cond_id": "plot_1",
                    "has_study_series": true
                }
            }
        }),
    ]));

    let data = alert_create_indicator(
        &mut runtime,
        IndicatorAlertRequest {
            script: "Signals",
            source,
            input_source: "stdin",
            condition_title: Some("Long"),
            alert_cond_id: None,
            symbol: Some("NASDAQ:AAPL"),
            resolution: Some("1D"),
            message: None,
            dry_run: false,
        },
    )
    .await
    .unwrap();

    assert_eq!(data["action"], "create_indicator");
    assert_eq!(data["dry_run"], false);
    assert_eq!(data["created"], true);
    assert_eq!(data["source"], "indicator_alert_api");
    assert_eq!(data["alert_id"], "4550000001");
    assert_eq!(data["condition"]["alert_cond_id"], "plot_1");
    assert!(data["script"].get("id").is_none());
    assert!(data["matched_alert"]["condition"].get("pine_id").is_none());
    assert_eq!(runtime.evaluated.len(), 3);
    assert!(runtime.evaluated[2].0.contains("create_alert"));
    assert!(runtime.evaluated[2].0.contains("list_alerts"));
    assert!(!runtime.evaluated[2].0.contains("Content-Type"));
}

#[tokio::test]
async fn alert_indicator_create_rejects_missing_script_id_before_create_request() {
    let source = "alertcondition(close > open, \"Long\")";
    let mut runtime = FakeRuntime::new(VecDeque::from([json!({
        "requested": "Signals",
        "match_count": 1,
        "match": {
            "name": "Signals",
            "title": "Signals",
            "version": 1,
            "modified": null,
            "script_id": null,
            "script_id_available": false
        },
        "candidates": []
    })]));

    let error = alert_create_indicator(
        &mut runtime,
        IndicatorAlertRequest {
            script: "Signals",
            source,
            input_source: "stdin",
            condition_title: Some("Long"),
            alert_cond_id: None,
            symbol: None,
            resolution: None,
            message: None,
            dry_run: false,
        },
    )
    .await
    .unwrap_err();

    assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
    assert_eq!(
        error.message,
        "Saved Pine script identity or version is unavailable"
    );
    assert_eq!(runtime.evaluated.len(), 1);
}

#[tokio::test]
async fn alert_indicator_create_post_check_failure_does_not_fallback() {
    let source = "alertcondition(close > open, \"Long\")";
    let mut runtime = FakeRuntime::new(VecDeque::from([
        json!({
            "requested": "Signals",
            "match_count": 1,
            "match": {
                "name": "Signals",
                "title": "Signals",
                "version": 1,
                "modified": null,
                "script_id": "SAVED_SCRIPT_ID_REDACTED",
                "script_id_available": true
            },
            "candidates": []
        }),
        json!({"source": source}),
        json!({
            "error": "Indicator alert create did not confirm a matching new alert",
            "error_kind": "internal_api_unavailable",
            "phase": "post_check_failed",
            "created": false,
            "source": "indicator_alert_api"
        }),
    ]));

    let error = alert_create_indicator(
        &mut runtime,
        IndicatorAlertRequest {
            script: "Signals",
            source,
            input_source: "stdin",
            condition_title: Some("Long"),
            alert_cond_id: None,
            symbol: None,
            resolution: None,
            message: None,
            dry_run: false,
        },
    )
    .await
    .unwrap_err();

    assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
    assert_eq!(
        error.message,
        "Indicator alert create did not confirm a matching new alert"
    );
    assert_eq!(runtime.evaluated.len(), 3);
}

fn source_check_request(source: &str, dry_run: bool) -> IndicatorAlertRequest<'_> {
    IndicatorAlertRequest {
        script: "Signals",
        source,
        input_source: "stdin",
        condition_title: Some("Long"),
        alert_cond_id: None,
        symbol: None,
        resolution: None,
        message: None,
        dry_run,
    }
}

fn saved_script_fixture() -> Value {
    json!({"match_count": 1, "match": {
        "name": "Signals", "script_id": "private-script-id", "version": "4.0",
        "script_id_available": true
    }})
}

#[tokio::test]
async fn alert_indicator_source_comparison_accepts_only_line_ending_changes() {
    let source = "indicator(\"Signals\")\nalertcondition(close > open, \"Long\")\n";
    for saved in [
        source.to_owned(),
        source.replace('\n', "\r\n"),
        source.replace('\n', "\r"),
    ] {
        let mut runtime = FakeRuntime::new([saved_script_fixture(), json!({"source": saved})]);
        let data = alert_create_indicator(&mut runtime, source_check_request(source, true))
            .await
            .unwrap();
        assert_eq!(data["would_create"], true);
        assert_eq!(data["script"]["version"], "4.0");
        assert_eq!(runtime.evaluated.len(), 2);
        assert!(runtime.evaluated.iter().all(|(expression, _)| {
            !expression.contains("pricealerts.tradingview.com") && !expression.contains(source)
        }));
    }
    for saved in [
        source.trim_end().to_owned(),
        format!("\u{feff}{source}"),
        format!("{source}// extra comment\n"),
        source.replace("close > open", "close < open"),
        source.replace("close > open", "close  > open"),
    ] {
        for dry_run in [true, false] {
            let mut runtime = FakeRuntime::new([saved_script_fixture(), json!({"source": saved})]);
            let error = alert_create_indicator(&mut runtime, source_check_request(source, dry_run))
                .await
                .unwrap_err();
            assert_eq!(error.kind, ErrorKind::Validation);
            assert_eq!(error.details.unwrap()["reason"], "source_mismatch");
            assert_eq!(runtime.evaluated.len(), 2);
        }
    }
}

#[tokio::test]
async fn alert_indicator_requires_id_and_version_in_both_modes() {
    let source = "alertcondition(close > open, \"Long\")";
    for (field, value) in [
        ("script_id", Value::Null),
        ("script_id", json!(" ")),
        ("script_id", json!(17)),
        ("version", Value::Null),
        ("version", json!("")),
        ("version", json!(" ")),
        ("version", json!(false)),
        ("version", json!([])),
        ("version", json!({})),
    ] {
        for dry_run in [true, false] {
            let mut saved = saved_script_fixture();
            saved["match"][field] = value.clone();
            let mut runtime = FakeRuntime::new([saved]);
            let error = alert_create_indicator(&mut runtime, source_check_request(source, dry_run))
                .await
                .unwrap_err();
            assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
            assert_eq!(
                error.details,
                Some(json!({
                    "phase": "saved_source_verification", "reason": "saved_identity_unavailable"
                }))
            );
            assert_eq!(runtime.evaluated.len(), 1);
        }
    }
}

#[tokio::test]
async fn alert_indicator_source_read_failures_are_sanitized_and_do_not_create() {
    let source = "alertcondition(close > open, \"Long\")";
    for response in [
        Value::Null,
        json!({}),
        json!({"source": null}),
        json!({"source": ""}),
        json!({"source": 42}),
        json!({"source": ["private-source"]}),
        json!({"error": "private-source and private-script-id"}),
    ] {
        for dry_run in [true, false] {
            let mut runtime = FakeRuntime::new([saved_script_fixture(), response.clone()]);
            let error = alert_create_indicator(&mut runtime, source_check_request(source, dry_run))
                .await
                .unwrap_err();
            assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
            assert_eq!(error.message, "Saved Pine script source is unavailable");
            assert_eq!(
                error.details,
                Some(json!({
                    "phase": "saved_source_verification", "reason": "saved_source_unavailable"
                }))
            );
            assert_eq!(runtime.evaluated.len(), 2);
            assert!(
                runtime
                    .evaluated
                    .iter()
                    .all(|(expression, _)| { !expression.contains("pricealerts.tradingview.com") })
            );
        }
    }
    for dry_run in [true, false] {
        let mut runtime = FakeRuntime::new([saved_script_fixture()])
            .with_evaluate_app_error_after_responses(
                AppError::new(ErrorKind::Timeout, "private-source and private-script-id")
                    .with_details(json!({"description": "private source stack"})),
            );
        let error = alert_create_indicator(&mut runtime, source_check_request(source, dry_run))
            .await
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.message, "Saved Pine script source is unavailable");
        assert_eq!(
            error.details,
            Some(json!({
                "phase": "saved_source_verification", "reason": "saved_source_unavailable"
            }))
        );
        assert_eq!(runtime.evaluated.len(), 2);
    }
}

#[tokio::test]
async fn alert_indicator_invalid_catalog_is_not_a_public_payload() {
    let source = "alertcondition(close > open, \"Long\")";
    for response in [
        Value::Null,
        json!({"error": "private catalog"}),
        json!({"match_count": 1}),
        json!({"match_count": 1, "match": {"script_id": "private-script-id"}}),
    ] {
        let mut runtime = FakeRuntime::new([response]);
        let error = alert_create_indicator(&mut runtime, source_check_request(source, true))
            .await
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(
            error.message,
            "Saved Pine script identity or version is unavailable"
        );
        assert_eq!(
            error.details,
            Some(json!({
                "phase": "saved_source_verification", "reason": "saved_identity_unavailable"
            }))
        );
        assert_eq!(runtime.evaluated.len(), 1);
    }
}

#[tokio::test]
#[ignore = "run through scripts/check-account-js-contract.py with pinned Node.js"]
async fn javascript_account_indicator_source_fetch_is_versioned_and_read_only() {
    let source = "indicator(\"Signals\")\nalertcondition(close > open, \"Long\")";
    let mut catalog = saved_script_fixture();
    catalog["match"]["script_id"] = json!("synthetic/id?#");
    let mut runtime = FakeRuntime::new([catalog, json!({"source": source})]);
    alert_create_indicator(&mut runtime, source_check_request(source, true))
        .await
        .unwrap();
    let expressions = serde_json::to_string(
        &runtime
            .evaluated
            .iter()
            .map(|(expression, _)| expression)
            .collect::<Vec<_>>(),
    )
    .unwrap();
    let source = serde_json::to_string(source).unwrap();
    let script = format!(
        r#"
        const assert = require('node:assert/strict');
        const expressions = {expressions};
        const source = {source};
        const forbidden = new Proxy({{}}, {{get: () => {{throw new Error('unexpected UI access');}}}});
        global.window = forbidden;
        global.document = forbidden;
        const listUrl = 'https://pine-facade.tradingview.com/pine-facade/list/?filter=saved';
        const sourceUrl = 'https://pine-facade.tradingview.com/pine-facade/get/synthetic%2Fid%3F%23/4.0';
        async function run(expression, expectedUrl, mode, body) {{
            const calls = [];
            global.fetch = async (url, options) => {{
                calls.push({{url, options}});
                if (mode === 'network' || mode === 'redirect') throw new Error('private source and id');
                return {{
                    ok: mode !== 'http',
                    status: mode === 'http' ? 429 : 200,
                    json: async () => {{
                        if (mode === 'json') throw new Error('private response');
                        return body;
                    }}
                }};
            }};
            const result = await eval(expression);
            assert.equal(calls.length, 1);
            assert.equal(calls[0].url, expectedUrl);
            assert.deepEqual(calls[0].options, expectedUrl === sourceUrl
                ? {{credentials: 'include', redirect: 'error'}}
                : {{credentials: 'include'}});
            return result;
        }}
        (async () => {{
            const saved = {{scriptName: 'Signals', scriptIdPart: 'synthetic/id?#', version: '4.0'}};
            const result = await run(expressions[0], listUrl, 'ok', [saved]);
            assert.equal(result.match.script_id, saved.scriptIdPart);
            assert.equal(result.match.version, saved.version);
            assert.equal(result.match_count, 1);
            const ambiguous = await run(expressions[0], listUrl, 'ok', [saved, saved]);
            assert.equal(ambiguous.match_count, 2);
            assert.equal(ambiguous.match, null);
            assert.equal(JSON.stringify(ambiguous.candidates).includes(saved.scriptIdPart), false);
            assert.equal((await run(expressions[0], listUrl, 'ok', [])).match_count, 0);
            for (const mode of ['http', 'network', 'json']) {{
                assert.equal(await run(expressions[0], listUrl, mode, [saved]), null);
            }}
            assert.equal(await run(expressions[0], listUrl, 'ok', {{error: 'private'}}), null);
            assert.deepEqual(await run(expressions[1], sourceUrl, 'ok', {{source, extra: 'private'}}), {{source}});
            for (const mode of ['http', 'network', 'json', 'redirect']) {{
                assert.equal(await run(expressions[1], sourceUrl, mode, {{source}}), null);
            }}
            for (const body of [null, {{}}, {{source: null}}, {{source: ''}}, {{source: 17}},
                {{source: []}}, {{source: {{}}}}, {{success: false, source}},
                {{error: 'private', source}}, {{err: 'private', source}}]) {{
                assert.equal(await run(expressions[1], sourceUrl, 'ok', body), null);
            }}
        }})().catch(error => {{ console.error(error); process.exitCode = 1; }});
    "#
    );
    let output = std::process::Command::new("node")
        .args(["-e", &script])
        .output()
        .expect("Node.js is required for the account JavaScript contract");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
