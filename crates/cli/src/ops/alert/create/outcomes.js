const assert = require('node:assert/strict');

(async () => {
    const results = [];
    const modes = ['post_list', 'post_invalid', 'post_json', 'post_check', 'network',
        'body', 'http', 'rejected', 'chart', 'pre_list', 'pre_invalid', 'success', 'pending', 'unknown', 'stopped',
        'missing_id', 'string_id', 'unsafe_id', 'existing_id', 'duplicate', 'alias_conflict', 'string_existing', 'string_duplicate',
        'concurrent', 'wrong_price', 'wrong_resolution', 'wrong_cross_interval', 'wrong_outer_cross',
        'wrong_notification', 'wrong_expiry', 'missing_setting', 'wrong_alias', 'extra_series',
        'wrong_frequency', 'reordered_symbol'];
    for (const mode of modes) {
        let writes = 0;
        let reads = 0;
        let payload;
        global.window = {TradingViewApi: {_activeChartWidgetWV: {
            value: () => ({symbolExt: () => mode === 'chart' ? {} : {symbol: 'NASDAQ:EXAMPLE'}, resolution: () => '1D'})
        }}};
        global.fetch = async (url, options) => {
            if (options.method === 'POST') {
                assert.equal(url, 'https://pricealerts.tradingview.com/create_alert');
                assert.equal(options.headers, undefined);
                writes++;
                payload = JSON.parse(options.body).payload;
                assert.equal(payload.popup, true);
                assert.equal(payload.mobile_push, true);
                assert.equal(payload.auto_deactivate, true);
                assert.equal(payload.conditions[0].frequency, 'on_first_fire');
                if (mode === 'network') throw new Error('connection lost');
                return {
                    ok: mode !== 'http', status: mode === 'http' ? 503 : 200,
                    statusText: 'synthetic response',
                    text: async () => {
                        if (mode === 'body') throw new Error('response lost');
                        const id = mode === 'existing_id' ? 1 : mode === 'string_id' ? '2' : mode === 'unsafe_id' ? 2 ** 53 : 2;
                        return JSON.stringify(mode === 'rejected' ? {s: 'error'} : {s: 'ok', r: mode === 'missing_id' ? {} : {alert_id: id}});
                    }
                };
            }
            assert.equal(url, 'https://pricealerts.tradingview.com/list_alerts');
            reads++;
            if (mode === 'pre_list' || (mode === 'post_list' && reads === 2)) {
                throw new Error('list unavailable');
            }
            return {ok: true, json: async () => {
                if (mode === 'post_json' && reads === 2) throw new Error('invalid JSON');
                if (mode === 'pre_invalid' || (mode === 'post_invalid' && reads === 2)) return {};
                const rows = [{alert_id: mode === 'string_existing' ? '2' : 1}];
                if (writes === 1 && mode !== 'post_check') {
                    const row = {...structuredClone(payload), alert_id: 2, active: true, last_error: null, last_stop_reason: null, cross_interval: true};
                    row.expiration = new Date(Math.floor(Date.parse(payload.expiration) / 1000) * 1000).toISOString();
                    row.conditions[0].resolution = '1';
                    row.conditions[0].cross_interval = true;
                    if (mode === 'pending') row.active = false;
                    if (mode === 'unknown') delete row.active;
                    if (mode === 'stopped') { row.active = false; row.last_stop_reason = 'synthetic'; }
                    if (mode === 'wrong_price') row.conditions[0].series[1].value += 0.000001;
                    if (mode === 'wrong_resolution') row.conditions[0].resolution = '1D';
                    if (mode === 'wrong_cross_interval') row.conditions[0].cross_interval = false;
                    if (mode === 'wrong_outer_cross') row.cross_interval = false;
                    if (mode === 'wrong_frequency') row.conditions[0].frequency = 'on_bar_close';
                    if (mode === 'wrong_notification') row.mobile_push = false;
                    if (mode === 'wrong_expiry') row.expiration = '2030-01-01T00:00:00Z';
                    if (mode === 'missing_setting') delete row.popup;
                    if (mode === 'extra_series') row.conditions[0].series.push({type: 'value', value: 5});
                    if (mode === 'alias_conflict') row.id = 3;
                    if (mode === 'reordered_symbol') row.symbol = '=' + JSON.stringify({'currency-id': 'USD', adjustment: 'splits', symbol: 'NASDAQ:EXAMPLE'});
                    row.condition = structuredClone(row.conditions[0]);
                    if (mode === 'wrong_alias') row.condition.type = 'cross_down';
                    if (mode === 'concurrent') rows.push({...structuredClone(row), alert_id: 3});
                    rows.push(row);
                    if (mode === 'duplicate') rows.push(structuredClone(row));
                    if (mode === 'string_duplicate') rows.push({...structuredClone(row), alert_id: '2'});
                }
                return {r: rows};
            }};
        };
        const result = await eval(expression);
        const preflight = mode.startsWith('pre_') || mode === 'chart';
        assert.equal(writes, preflight ? 0 : 1, mode);
        const requestFailure = ['network', 'body', 'http', 'rejected'].includes(mode);
        assert.equal(reads, mode === 'chart' ? 0 : (preflight || requestFailure ? 1 : 2), mode);
        const success = ['success', 'pending', 'unknown', 'concurrent', 'reordered_symbol'].includes(mode);
        const attributed = ['stopped', 'wrong_price', 'wrong_resolution', 'wrong_cross_interval',
            'wrong_outer_cross', 'wrong_notification', 'wrong_expiry', 'missing_setting', 'wrong_alias',
            'extra_series', 'wrong_frequency'].includes(mode);
        if (success) {
            assert.equal(result.created, true);
            assert.equal(result.alert_id, 2);
            assert.deepEqual(result.confirmation, {identity: 'matched', settings: 'matched'});
            assert.equal(result.activation.state, ['pending', 'unknown'].includes(mode) ? mode : 'active');
            assert.equal(result.creation_outcome, undefined);
        } else {
            assert.equal(result.api_fallback_allowed, false);
            assert.equal(result.price_set, !preflight);
            assert.equal(result.created, preflight ? false : attributed ? true : null, mode);
            assert.equal(result.creation_outcome, preflight || attributed ? undefined : 'unknown', mode);
            if (attributed) {
                assert.equal(result.alert_id, 2);
                assert.equal(result.reason, mode === 'stopped' ? 'activation_stopped' : mode === 'missing_setting' ? 'settings_unavailable' : 'settings_mismatch');
            }
            assert.equal(result.phase, {
                chart: 'chart_metadata_unavailable', pre_list: 'pre_list_unavailable',
                pre_invalid: 'pre_list_unavailable', post_list: 'post_list_unavailable',
                post_invalid: 'post_list_unavailable', post_json: 'post_list_unavailable',
                network: 'create_request_unavailable', body: 'create_request_unavailable',
                http: 'create_request_failed', rejected: 'create_request_failed'
            }[mode] || 'post_check_failed');
        }
        results.push({mode, result, success, attributed});
    }
    console.log(JSON.stringify(results));
})().catch(error => { console.error(error); process.exitCode = 1; });
