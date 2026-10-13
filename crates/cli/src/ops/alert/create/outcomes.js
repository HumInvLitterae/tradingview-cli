const assert = require('node:assert/strict');

(async () => {
    const results = [];
    const modes = ['post_list', 'post_invalid', 'post_json', 'post_check', 'network',
        'body', 'http', 'rejected', 'chart', 'pre_list', 'pre_invalid', 'success'];
    for (const mode of modes) {
        let writes = 0;
        let reads = 0;
        let payload;
        global.window = {TradingViewApi: {_activeChartWidgetWV: {
            value: () => ({symbolExt: () => mode === 'chart' ? {} : {symbol: 'NASDAQ:EXAMPLE'}})
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
                        return JSON.stringify(mode === 'rejected' ? {err: {code: 'rejected'}} : {});
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
                const rows = [{id: 'existing'}];
                if (mode === 'success' && writes === 1) rows.push({
                    id: 'new', symbol: payload.symbol, message: payload.message,
                    condition: payload.conditions[0]
                });
                return {r: rows};
            }};
        };
        const result = await eval(expression);
        const preflight = mode.startsWith('pre_') || mode === 'chart';
        assert.equal(writes, preflight ? 0 : 1, mode);
        const requestFailure = ['network', 'body', 'http', 'rejected'].includes(mode);
        assert.equal(reads, mode === 'chart' ? 0 : (preflight || requestFailure ? 1 : 2), mode);
        if (mode === 'success') {
            assert.equal(result.created, true);
            assert.equal(result.alert_id, 'new');
            assert.equal(result.creation_outcome, undefined);
        } else {
            assert.equal(result.api_fallback_allowed, false);
            assert.equal(result.price_set, !preflight);
            assert.equal(result.created, preflight ? false : null, mode + ': creation cannot be ruled out');
            assert.equal(result.creation_outcome, preflight ? undefined : 'unknown', mode);
            assert.equal(result.phase, {
                chart: 'chart_metadata_unavailable', pre_list: 'pre_list_unavailable',
                pre_invalid: 'pre_list_unavailable', post_list: 'post_list_unavailable',
                post_invalid: 'post_list_unavailable', post_json: 'post_list_unavailable',
                post_check: 'post_check_failed', network: 'create_request_unavailable',
                body: 'create_request_unavailable', http: 'create_request_failed', rejected: 'create_request_failed'
            }[mode]);
        }
        results.push({mode, result});
    }
    console.log(JSON.stringify(results));
})().catch(error => { console.error(error); process.exitCode = 1; });
