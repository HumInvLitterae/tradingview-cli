use serde_json::{Value, json};
use tradingview_model::screener::{
    columns::ScreenerStorageColumnTarget, filters::ScreenerStorageFilterTarget,
    screens::ScreenerScreenTarget,
};

use super::super::test_support::FakeRuntime;
use super::{
    columns::save_screener_storage_columns,
    engine::{fetch_active_screener_storage_config, fetch_current_screener_storage_config},
    filters::{
        refresh_full_page_screener_after_storage_filter_save,
        request_full_page_screener_reload_after_storage_filter_save, save_screener_storage_filters,
    },
    screens::{delete_screener_storage_screen, fetch_screener_storage_screens},
};

fn runtime() -> FakeRuntime {
    FakeRuntime::new([json!({})])
}

fn only_expression(runtime: FakeRuntime) -> String {
    assert_eq!(runtime.evaluated.len(), 1);
    runtime.evaluated[0].0.clone()
}

#[tokio::test]
#[ignore = "run through `node scripts/check-js-contracts.mjs account`"]
async fn javascript_account_screener_storage_expressions_avoid_dom() {
    let config = json!({
        "storage_screen": { "id": "scr/1", "title": "Momentum", "filters": [] }
    });
    let mut expressions = Vec::new();

    let mut rt = runtime();
    let _ = fetch_active_screener_storage_config(&mut rt, "Momentum \"Q4\"").await;
    expressions.push(only_expression(rt));

    let mut rt = runtime();
    let _ = fetch_current_screener_storage_config(&mut rt).await;
    expressions.push(only_expression(rt));

    let mut rt = runtime();
    let column = ScreenerStorageColumnTarget {
        index: 0,
        id: "Price".to_string(),
        name: None,
        params: json!({}),
    };
    let _ = save_screener_storage_columns(&mut rt, &config, &[column]).await;
    expressions.push(only_expression(rt));

    let mut rt = runtime();
    let filter = ScreenerStorageFilterTarget {
        index: 0,
        text: None,
        filter_type: None,
        subtype: None,
        operation: None,
        raw: json!({ "id": "market_cap" }),
    };
    let _ = save_screener_storage_filters(&mut rt, &config, &[filter]).await;
    expressions.push(only_expression(rt));

    let mut rt = runtime();
    let _ = refresh_full_page_screener_after_storage_filter_save(&mut rt, &json!({}), 1).await;
    expressions.push(only_expression(rt));

    let mut rt = runtime();
    let _ = request_full_page_screener_reload_after_storage_filter_save(&mut rt).await;
    expressions.push(only_expression(rt));

    let mut rt = runtime();
    let _ = fetch_screener_storage_screens(&mut rt, Some("Momentum")).await;
    expressions.push(only_expression(rt));

    let mut rt = runtime();
    let target = ScreenerScreenTarget {
        index: 0,
        id: Some("scr/1".to_string()),
        name: "Momentum".to_string(),
        active: false,
        owner: None,
        shared: None,
    };
    let _ = delete_screener_storage_screen(&mut rt, &target).await;
    expressions.push(only_expression(rt));

    for expression in &expressions {
        assert!(!expression.contains("REPLACE_HELPERS"));
        assert!(!expression.contains("function readScreenerState"));
    }
    let script = format!(
        "const expressions = {};\n{STORAGE_CONTRACT}",
        Value::from(expressions)
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

const STORAGE_CONTRACT: &str = r#"
const assert = require('node:assert/strict');
const [activeGet, currentGet, columnsPut, filtersPut, refreshReload, requestReload,
    screensGet, screenDelete] = expressions;

globalThis.document = new Proxy({}, {
    get(_, key) { throw new Error('document access: ' + String(key)); }
});
function setWindow(initData, location) {
    const allowed = { initData, location };
    globalThis.window = new Proxy({}, {
        get(_, key) {
            if (key in allowed) return allowed[key];
            throw new Error('window access: ' + String(key));
        }
    });
}
let calls = [];
function respond(status, body) {
    calls = [];
    globalThis.fetch = async (url, options) => {
        calls.push({ url, options });
        return {
            ok: status >= 200 && status < 300,
            status,
            statusText: 'status ' + status,
            json: async () => {
                if (body instanceof Error) throw body;
                return body;
            }
        };
    };
}
const initData = {
    SCREENER_STORAGE_URL: 'https://storage.example/',
    screener_storage_release_version: '7',
    screen_data: { id: 'scr/1', title: 'Visible', screener_key: 'stock' }
};
const screenUrl = 'https://storage.example/api/v2/screens/scr%2F1/';
const run = expression => eval(expression);

(async () => {
    // Active and current screen GET share one body.
    setWindow({});
    respond(200, {});
    assert.deepEqual(await run(activeGet), {
        storage_available: false,
        reason: 'missing_screener_storage_init_data',
        columns: []
    });
    assert.equal(calls.length, 0);

    setWindow(initData);
    respond(500, {});
    assert.deepEqual(await run(activeGet), {
        storage_available: true,
        fetch_ok: false,
        status: 500,
        status_text: 'status 500',
        reason: 'screen_fetch_failed',
        columns: []
    });
    // A 200 whose body is not a screen document is a failed read, not an empty screen.
    respond(200, new Error('invalid json'));
    const unreadable = await run(currentGet);
    assert.equal(unreadable.fetch_ok, false);
    assert.equal(unreadable.reason, 'screen_body_invalid');
    respond(200, []);
    const listBody = await run(activeGet);
    assert.equal(listBody.fetch_ok, false);
    assert.equal(listBody.reason, 'screen_body_invalid');

    const body = {
        id: 'scr/1',
        title: 'Momentum "Q4"',
        version: 9,
        active_column_set: 'custom',
        default_custom_column_set: [{ id: 'Price', params: { length: 1 } }, { id: '' }, null]
    };
    respond(200, body);
    assert.deepEqual(await run(activeGet), {
        storage_available: true,
        fetch_ok: true,
        status: 200,
        screener_key: 'stock',
        version: 9,
        screen_id: 'scr/1',
        screen_title: 'Momentum "Q4"',
        expected_title: 'Momentum "Q4"',
        title_matches: true,
        active_column_set: 'custom',
        storage_screen: body,
        column_count: 3,
        columns: [{ index: 0, id: 'Price', params: { length: 1 } }]
    });
    assert.deepEqual(calls, [{
        url: screenUrl + '?screener_key=stock&version=7',
        options: { credentials: 'include' }
    }]);

    respond(200, { id: 'scr/1', title: 'Other' });
    const mismatch = await run(activeGet);
    assert.equal(mismatch.expected_title, 'Momentum "Q4"');
    assert.equal(mismatch.title_matches, false);
    const current = await run(currentGet);
    assert.equal(current.expected_title, 'Other');
    assert.equal(current.title_matches, true);
    respond(200, { id: 'scr/1' });
    const fallback = await run(currentGet);
    assert.equal(fallback.screen_title, 'Visible');
    assert.equal(fallback.expected_title, 'Visible');
    assert.equal(fallback.version, '7');
    assert.deepEqual(fallback.columns, []);

    // Column and filter PUT.
    for (const [expression, key, countKey, saved] of [
        [columnsPut, 'default_custom_column_set', 'column_count', [{ id: 'Price', params: {} }]],
        [filtersPut, 'filters', 'filter_count', [{ id: 'market_cap' }]]
    ]) {
        const responseCountKey = 'response_' + countKey;
        setWindow({});
        respond(200, {});
        assert.deepEqual(await run(expression), {
            saved: false,
            reason: 'missing_screener_storage_init_data'
        });
        assert.equal(calls.length, 0);

        setWindow(initData);
        respond(200, { title: 'Momentum', [key]: [1, 2] });
        assert.deepEqual(await run(expression), {
            saved: true,
            status: 200,
            status_text: 'status 200',
            screen_id: 'scr/1',
            [countKey]: 1,
            [responseCountKey]: 2,
            response_title: 'Momentum'
        });
        assert.equal(calls.length, 1);
        assert.equal(calls[0].url, screenUrl);
        assert.equal(calls[0].options.method, 'PUT');
        assert.equal(calls[0].options.credentials, 'include');
        assert.deepEqual(calls[0].options.headers, { 'Content-Type': 'application/json' });
        const sent = JSON.parse(calls[0].options.body);
        assert.equal(sent.id, 'scr/1');
        assert.deepEqual(sent[key], saved);

        respond(409, new Error('invalid json'));
        const rejected = await run(expression);
        assert.equal(rejected.saved, false);
        assert.equal(rejected.status, 409);
        assert.equal(rejected[responseCountKey], null);
        assert.equal(rejected.response_title, null);
    }

    // Visible reloads.
    let reloads = 0;
    setWindow(initData, { reload() { reloads++; } });
    assert.deepEqual(await run(refreshReload), { requested: true });
    assert.deepEqual(await run(requestReload), {
        requested: true,
        confirmed: false,
        reason: 'reload_requested_after_storage_post_check'
    });
    assert.equal(reloads, 2);
    setWindow(initData);
    assert.deepEqual(await run(refreshReload), {
        requested: false,
        reason: 'location_reload_unavailable'
    });
    assert.deepEqual(await run(requestReload), {
        requested: false,
        confirmed: false,
        reason: 'location_reload_unavailable'
    });

    // Custom screen list GET.
    setWindow({ SCREENER_STORAGE_URL: 'https://storage.example/' });
    respond(200, []);
    assert.deepEqual(await run(screensGet), {
        storage_available: false,
        reason: 'missing_screener_storage_init_data',
        screens: []
    });
    assert.equal(calls.length, 0);

    setWindow(initData);
    // A 200 whose body is not a list is a failed read, not an empty list.
    respond(200, {});
    const notList = await run(screensGet);
    assert.equal(notList.fetch_ok, false);
    assert.equal(notList.reason, 'custom_screens_body_invalid');
    respond(503, []);
    const unavailable = await run(screensGet);
    assert.equal(unavailable.fetch_ok, false);
    assert.equal(unavailable.reason, 'custom_screens_fetch_failed');
    respond(200, [
        { id: 1, title: 'Momentum', is_owner: true, is_shared: false },
        { id: 'b', title: 'Other' },
        { id: '', title: 'No id' },
        { id: 'c' }
    ]);
    assert.deepEqual(await run(screensGet), {
        storage_available: true,
        fetch_ok: true,
        status: 200,
        screener_key: 'stock',
        screen_title: 'Momentum',
        screen_count: 4,
        screens: [
            { index: 0, id: '1', name: 'Momentum', active: true, owner: true, shared: false },
            { index: 1, id: 'b', name: 'Other', active: false, owner: null, shared: null }
        ]
    });
    assert.deepEqual(calls, [{
        url: 'https://storage.example/api/v2/screens/?screener_key=stock&version=7' +
            '&sort_by=updated&sort_order=desc',
        options: { credentials: 'include' }
    }]);

    // Custom screen DELETE.
    setWindow({});
    respond(204, new Error('no body'));
    assert.deepEqual(await run(screenDelete), {
        deleted: false,
        reason: 'missing_screener_storage_init_data'
    });
    assert.equal(calls.length, 0);
    setWindow(initData);
    assert.deepEqual(await run(screenDelete), {
        deleted: true,
        status: 204,
        status_text: 'status 204',
        id: 'scr/1'
    });
    assert.deepEqual(calls, [{
        url: screenUrl,
        options: {
            method: 'DELETE',
            credentials: 'include',
            headers: { 'Content-Type': 'application/json' }
        }
    }]);
    respond(404, {});
    assert.equal((await run(screenDelete)).deleted, false);
})().catch(error => { console.error(error); process.exitCode = 1; });
"#;
