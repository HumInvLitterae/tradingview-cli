use serde_json::{Value, json};

use super::super::test_support::{FakeRuntime, run_node_json};
use super::{ALERT_LIST_READER, alert_delete, alert_delete_all, alert_list};

#[tokio::test]
#[ignore = "run through `node scripts/check-js-contracts.mjs account`"]
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
    let _ = super::alert_create(&mut runtime, 100.0, "crossing", None).await;
    let create = runtime.evaluated[0].0.clone();
    let source = "indicator(\"Signals\")\nalertcondition(close > open, \"Long\", \"Long message\")";
    let mut runtime = FakeRuntime::new([
        json!({"match_count": 1, "match": {
            "name": "Signals", "script_id": "synthetic-script", "version": 1,
            "script_id_available": true
        }}),
        json!({"source": source}),
        Value::Null,
    ]);
    let _ = super::alert_create_indicator(
        &mut runtime,
        super::IndicatorAlertRequest {
            script: "Signals",
            source,
            input_source: "inline",
            condition_title: Some("Long"),
            alert_cond_id: None,
            symbol: Some("NASDAQ:EXAMPLE"),
            resolution: Some("1D"),
            message: None,
            dry_run: false,
        },
    )
    .await;
    let indicator = runtime.evaluated[2].0.clone();
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
                        const body = !preflight && reads++ === 0 ? {{r: [{{id: 12, alert_id: 12}}]}} : {{}};
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
#[ignore = "run through `node scripts/check-js-contracts.mjs account`"]
async fn javascript_account_alert_delete_all_writes_only_after_dry_run_and_reads_back() {
    let mut expressions = Vec::new();
    for dry_run in [true, false] {
        let mut runtime = FakeRuntime::new([]);
        let _ = alert_delete_all(&mut runtime, dry_run).await;
        expressions.push(runtime.evaluated[0].0.clone());
    }
    let expressions = serde_json::to_string(&expressions).unwrap();
    let results = run_node_json(&format!(
        r#"
        const [dryRun, deleteAll] = {expressions};
        // Each list read returns the next listing; the last one repeats.
        function account(listings) {{
            const requests = [];
            global.fetch = async (url, options = {{}}) => {{
                requests.push({{ url, method: options.method || 'GET', body: options.body || null }});
                if (options.method === 'POST') {{
                    return {{ ok: true, json: async () => ({{ s: 'ok' }}) }};
                }}
                const rows = listings[Math.min(requests.filter(r => r.method === 'GET').length - 1, listings.length - 1)];
                return {{ ok: true, json: async () => ({{ s: 'ok', r: rows }}) }};
            }};
            return requests;
        }}
        const rows = [{{ alert_id: 12, message: 'one' }}, {{ id: 'abc', message: 'two' }}];
        (async () => {{
            const results = [];
            for (const [expression, listings] of [
                [dryRun, [rows]],
                [deleteAll, [rows, []]],
                [deleteAll, [rows, [rows[1]]]],
                [deleteAll, [[]]]
            ]) {{
                const requests = account(listings);
                const result = await eval(expression);
                results.push({{ result, requests }});
            }}
            console.log(JSON.stringify(results));
        }})().catch(error => {{ console.error(error); process.exitCode = 1; }});
        "#
    ));
    let list = json!({
        "url": "https://pricealerts.tradingview.com/list_alerts", "method": "GET", "body": null
    });
    let delete = json!({
        "url": "https://pricealerts.tradingview.com/delete_alerts",
        "method": "POST",
        "body": "{\"payload\":{\"alert_ids\":[12,\"abc\"]}}"
    });

    // A dry run lists once and never posts a deletion.
    assert_eq!(results[0]["requests"], json!([list]));
    assert_eq!(results[0]["result"]["action"], "dry_run");
    assert_eq!(results[0]["result"]["target_alert_ids"], json!([12, "abc"]));

    // A deletion posts every listed ID once, then lists again to confirm.
    assert_eq!(results[1]["requests"], json!([list, delete, list]));
    assert_eq!(results[1]["result"]["deleted"], true);
    assert_eq!(results[1]["result"]["after_count"], 0);
    assert_eq!(
        results[1]["result"]["remaining_target_alert_ids"],
        json!([])
    );

    // An alert still listed after the deletion is reported, not hidden.
    assert_eq!(results[2]["result"]["deleted"], false);
    assert_eq!(
        results[2]["result"]["remaining_target_alert_ids"],
        json!(["abc"])
    );

    // An empty account posts nothing.
    assert_eq!(results[3]["requests"], json!([list]));
    assert_eq!(results[3]["result"]["action"], "noop");
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
