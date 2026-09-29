use serde_json::{Value, json};

use super::super::test_support::FakeRuntime;
use super::{ALERT_LIST_READER, alert_delete, alert_delete_all, alert_list};

#[tokio::test]
#[ignore = "run through scripts/check-account-js-contract.py with pinned Node.js"]
async fn javascript_account_alert_lists_reject_failed_reads() {
    let mut runtime = FakeRuntime::new([json!({"alerts": []})]);
    alert_list(&mut runtime).await.unwrap();
    let list = runtime.evaluated[0].0.clone();
    let mut runtime = FakeRuntime::new([]);
    let _ = alert_delete(&mut runtime, "12").await;
    let delete = runtime.evaluated[0].0.clone();
    let mut runtime = FakeRuntime::new([]);
    let _ = alert_delete_all(&mut runtime, false).await;
    let delete_all = runtime.evaluated[0].0.clone();
    let mut runtime = FakeRuntime::new([]);
    let _ = super::create::alert_create_via_api(&mut runtime, 100.0, "crossing", None).await;
    let create = runtime.evaluated[0].0.clone();
    let mut runtime = FakeRuntime::new([
        json!({"match_count": 1, "match": {
            "name": "Signals", "script_id": "synthetic-script", "version": 1,
            "script_id_available": true
        }}),
        Value::Null,
    ]);
    let _ = super::alert_create_indicator(&mut runtime, super::IndicatorAlertRequest {
        script: "Signals",
        source: "indicator(\"Signals\")\nalertcondition(close > open, \"Long\", \"Long message\")",
        input_source: "inline",
        condition_title: Some("Long"),
        alert_cond_id: None,
        symbol: Some("NASDAQ:EXAMPLE"),
        resolution: Some("1D"),
        message: None,
        dry_run: false,
    }).await;
    let indicator = runtime.evaluated[1].0.clone();
    let expressions =
        serde_json::to_string(&[list, delete, delete_all, create, indicator]).unwrap();
    let script = format!(
        r#"
        const expressions = {expressions};
        {ALERT_LIST_READER}
        const assert = require('node:assert/strict');
        async function read(mode, body) {{
            global.fetch = async () => {{
                if (mode === 'network') throw new Error('private transport text');
                return {{
                    ok: mode !== 'http',
                    json: async () => {{
                        if (mode === 'json') throw new Error('private body');
                        return body;
                    }}
                }};
            }};
            return __readAlertRows();
        }}
        (async () => {{
            for (const [mode, body, phase] of [
                ['network', null, 'list_unavailable'],
                ['http', null, 'list_unavailable'],
                ['json', null, 'invalid_response'],
                ['ok', {{err: {{code: 'private'}}}}, 'list_unavailable'],
                ['ok', {{success: false, r: []}}, 'list_unavailable'],
                ['ok', null, 'invalid_response'],
                ['ok', {{}}, 'invalid_response'],
                ['ok', {{r: {{}}}}, 'invalid_response'],
                ['ok', {{r: [null]}}, 'invalid_response'],
                ['ok', {{r: ['private']}}, 'invalid_response']
            ]) {{
                const result = await read(mode, body);
                assert.equal(result.ok, false);
                assert.equal(result.phase, phase);
                assert.equal(JSON.stringify(result).includes('private'), false);
                const listed = await eval(expressions[0]);
                assert.equal(listed.phase, phase);
                assert.equal(listed.alert_count, undefined);
            }}
            await read('ok', {{r: []}});
            assert.deepEqual(await eval(expressions[0]), {{
                alert_count: 0, source: 'internal_api', alerts: []
            }});
            global.window = {{ TradingViewApi: {{ _activeChartWidgetWV: {{
                value: () => ({{ symbolExt: () => ({{symbol: 'NASDAQ:EXAMPLE'}}) }})
            }} }} }};
            for (const expression of expressions.slice(1)) {{
                for (const preflight of [true, false]) {{
                    let writes = 0;
                    let reads = 0;
                    global.fetch = async (_, options) => {{
                        if (options.method === 'POST') {{
                            writes++;
                            return {{ok: true, json: async () => ({{}}), text: async () => '{{}}'}};
                        }}
                        const body = !preflight && reads++ === 0 ? {{r: [{{id: 12}}]}} : {{}};
                        return {{ok: true, json: async () => body}};
                    }};
                    const result = await eval(expression);
                    assert.equal(typeof result.error, 'string');
                    assert.notEqual(result.deleted, true);
                    assert.notEqual(result.created, true);
                    assert.equal(writes, preflight ? 0 : 1);
                }}
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

#[tokio::test]
async fn alert_list_keeps_empty_success_and_transport_errors_distinct() {
    let mut runtime = FakeRuntime::new([json!({"alerts": [], "alert_count": 0})]);
    assert_eq!(alert_list(&mut runtime).await.unwrap()["alerts"], json!([]));
    let mut runtime =
        FakeRuntime::new([]).with_evaluate_error(tradingview_core::ErrorKind::Timeout);
    assert_eq!(alert_list(&mut runtime).await.unwrap_err().exit_code(), 4);
    for data in [json!({"alerts": [null]}), Value::Null] {
        let mut runtime = FakeRuntime::new([data]);
        let error = alert_list(&mut runtime).await.unwrap_err();
        assert_eq!(error.exit_code(), 3);
        let envelope = tradingview_core::ErrorEnvelope::new("alert list", error.into());
        assert_eq!(serde_json::to_value(envelope).unwrap()["success"], false);
    }
}
