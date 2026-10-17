(async () => {
    const cases = ['concurrent', 'active', 'pending', 'unknown', 'stopped', 'reordered',
        'missing_id', 'string_id', 'unsafe_id', 'existing_id', 'duplicate', 'missing_row',
        'alias_conflict', 'wrong_version', 'wrong_input_type', 'extra_input', 'missing_input',
        'missing_symbol', 'wrong_resolution', 'wrong_message', 'wrong_notification',
        'wrong_expiry', 'wrong_cross_interval', 'wrong_condition_alias', 'malformed_conditions',
        'wrong_symbol_settings', 'wrong_offsets', 'two_series', 'two_conditions', 'contradictory_status'];
    const results = [];
    for (const mode of cases) {
        let reads = 0, writes = 0, payload;
        const chart = {symbolExt: () => ({symbol: 'NASDAQ:TEST', currency: 'USD'}), resolution: () => '1D', getAllStudies: () => []};
        global.window = {TradingViewApi: {_activeChartWidgetWV: {value: () => chart}}};
        global.fetch = async (url, options) => {
            if (options?.method === 'POST') {
                writes++; payload = JSON.parse(options.body).payload;
                const id = mode === 'existing_id' ? 1 : mode === 'string_id' ? '2' : mode === 'unsafe_id' ? 2 ** 53 : 2;
                return {ok: true, text: async () => JSON.stringify({s: 'ok', id: 'request-id', r: mode === 'missing_id' ? {} : {alert_id: id}})};
            }
            reads++;
            const rows = [{alert_id: 1}];
            if (writes) {
                const row = {...structuredClone(payload), alert_id: 2, active: true, last_error: null, last_stop_reason: null};
                delete row.ignore_warnings;
                row.expiration = new Date(Math.floor(Date.parse(payload.expiration) / 1000) * 1000).toISOString();
                row.conditions[0].cross_interval = false;
                row.condition = structuredClone(row.conditions[0]);
                if (mode === 'wrong_symbol_settings') row.symbol = payload.symbol.replace('USD', 'JPY');
                if (mode === 'wrong_offsets') row.conditions[0].series[0].offsets_by_plot.extra = 1;
                if (mode === 'two_series') row.conditions[0].series.push(structuredClone(row.conditions[0].series[0]));
                if (mode === 'two_conditions') row.conditions.push(structuredClone(row.conditions[0]));
                if (mode === 'contradictory_status') row.last_stop_reason = 'manual';
                if (mode === 'pending') row.active = false;
                if (mode === 'unknown') delete row.active;
                if (mode === 'stopped') {row.active = false; row.last_stop_reason = 'error'; row.last_error = 'private-provider-error';}
                if (mode === 'alias_conflict') row.id = 3;
                if (mode === 'wrong_version') row.conditions[0].series[0].pine_version = 'wrong';
                if (mode === 'wrong_input_type') row.conditions[0].series[0].inputs.__fast_calc = 'false';
                if (mode === 'extra_input') row.conditions[0].series[0].inputs.in_99 = 'private-extra';
                if (mode === 'missing_input') delete row.conditions[0].series[0].inputs.__fast_calc;
                if (mode === 'missing_symbol') delete row.symbol;
                if (mode === 'wrong_resolution') row.resolution = '5';
                if (mode === 'wrong_message') row.message = 'private-message';
                if (mode === 'wrong_notification') row.popup = true;
                if (mode === 'wrong_expiry') row.expiration = '2040-01-01T00:00:00Z';
                if (mode === 'wrong_cross_interval') row.conditions[0].cross_interval = true;
                if (mode === 'wrong_condition_alias') row.condition.alert_cond_id = 'wrong';
                if (mode === 'malformed_conditions') row.conditions = {};
                if (mode === 'reordered') row.conditions[0].series[0].inputs = Object.fromEntries(Object.entries(row.conditions[0].series[0].inputs).reverse());
                if (mode === 'concurrent') rows.push({...structuredClone(row), alert_id: 3});
                if (mode !== 'missing_row') rows.push(row);
                if (mode === 'duplicate') rows.push(structuredClone(row));
            }
            return {ok: true, json: async () => ({r: rows})};
        };
        const result = await eval(expression);
        assert.equal(writes, 1, mode);
        assert.equal(reads, 2, mode);
        const success = ['active', 'pending', 'unknown', 'reordered', 'concurrent', 'contradictory_status'].includes(mode);
        const unknown = ['missing_id', 'string_id', 'unsafe_id', 'existing_id', 'duplicate', 'missing_row', 'alias_conflict'].includes(mode);
        assert.equal(Boolean(result.error), !success, mode);
        assert.equal(result.created, unknown ? null : true, mode);
        if (success) {
            assert.equal(result.alert_id, 2, mode);
            assert.deepEqual(result.confirmation, {identity: 'matched', settings: 'matched'});
            assert.equal(result.activation.state, mode === 'contradictory_status' ? 'unknown' : ['pending', 'unknown'].includes(mode) ? mode : 'active');
            if (mode === 'unknown') assert.equal(result.matched_alert.active, null);
        }
        if (!success && !unknown) {
            const reason = mode === 'stopped' ? 'activation_stopped'
                : ['missing_input', 'missing_symbol', 'malformed_conditions'].includes(mode) ? 'settings_unavailable' : 'settings_mismatch';
            assert.equal(result.reason, reason, mode);
            assert.equal(result.confirmation.identity, 'matched', mode);
        }
        assert.ok(!JSON.stringify(result).includes('private-provider-error'));
        assert.ok(!JSON.stringify(result).includes('private-extra'));
        results.push({mode, success, unknown, result});
    }
    console.log(JSON.stringify(results));
})().catch(error => {console.error(error); process.exitCode = 1;});
