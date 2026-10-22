function validAlertId(id) {
    return Number.isSafeInteger(id) && id > 0;
}

function activationState(row) {
    const hasStatus = Object.hasOwn(row, 'last_stop_reason') && Object.hasOwn(row, 'last_error');
    if (!hasStatus || typeof row.active !== 'boolean') return 'unknown';
    const statuses = [row.last_stop_reason, row.last_error];
    if (statuses.some(value => value !== null && (typeof value !== 'string' || !value))) return 'unknown';
    if (row.active) return statuses.every(value => value === null) ? 'active' : 'unknown';
    return statuses.every(value => value === null) ? 'pending' : 'stopped';
}

function confirmCreatedAlert(response, before, after, submitted, kind) {
    const priceAlert = kind === 'price';
    const unknown = reason => ({
        error: 'Alert creation could not be attributed',
        error_kind: 'internal_api_unavailable', phase: 'post_check_failed', reason,
        created: null, creation_outcome: 'unknown', source
    });
    const id = response && response.r && response.r.alert_id;
    if (!validAlertId(id)) return unknown('response_id_unavailable');
    const hasId = row => [row.alert_id, row.id].some(value => value === id || value === String(id));
    if (before.some(hasId)) return unknown('response_id_preexisting');
    const matches = after.filter(hasId);
    if (matches.length !== 1) return unknown(matches.length ? 'response_id_ambiguous' : 'created_row_unavailable');
    const row = matches[0];
    if (row.alert_id !== id || (Object.hasOwn(row, 'id') && row.id !== id)) return unknown('response_id_ambiguous');

    let unavailable = false;
    let mismatch = false;
    function compare(expected, actual, exact = true) {
        if (actual === undefined) { unavailable = true; return; }
        if (expected === null || typeof expected !== 'object') {
            if (actual !== expected) mismatch = true;
            return;
        }
        if (actual === null || typeof actual !== 'object' || Array.isArray(expected) !== Array.isArray(actual)) {
            unavailable = true; return;
        }
        if (Array.isArray(expected)) {
            if (expected.length !== actual.length) mismatch = true;
            expected.forEach((value, index) => compare(value, actual[index]));
        } else {
            if (exact && Object.keys(actual).some(key => !Object.hasOwn(expected, key))) mismatch = true;
            for (const key of Object.keys(expected)) compare(expected[key], actual[key]);
        }
    }
    function condition(actual) {
        if (!actual || typeof actual !== 'object' || Array.isArray(actual)) { unavailable = true; return; }
        const expected = submitted.conditions[0];
        if (priceAlert) {
            compare(expected.type, actual.type);
            compare(expected.frequency, actual.frequency);
            compare(expected.series, actual.series);
            compare('1', actual.resolution);
            compare(true, actual.cross_interval);
            return;
        }
        for (const key of ['type', 'frequency', 'alert_cond_id', 'resolution']) compare(expected[key], actual[key]);
        if (Object.hasOwn(actual, 'cross_interval')) compare(false, actual.cross_interval);
        if (!Array.isArray(actual.series)) { unavailable = true; return; }
        if (actual.series.length !== 1) mismatch = true;
        const series = actual.series[0];
        if (!series || typeof series !== 'object' || Array.isArray(series)) { unavailable = true; return; }
        for (const key of Object.keys(expected.series[0])) compare(expected.series[0][key], series[key]);
    }
    for (const key of ['symbol', 'resolution', 'message', 'name', 'sound_file', 'sound_duration',
        'popup', 'auto_deactivate', 'email', 'sms_over_email', 'mobile_push', 'web_hook']) {
        if (priceAlert && key === 'symbol') {
            try {
                if (typeof row.symbol !== 'string' || !row.symbol.startsWith('=')) { unavailable = true; continue; }
                compare(JSON.parse(submitted.symbol.slice(1)), JSON.parse(row.symbol.slice(1)));
            } catch (_) { unavailable = true; }
        } else compare(submitted[key], row[key]);
    }
    if (priceAlert) compare(true, row.cross_interval);
    const expiry = typeof row.expiration === 'string' && /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})$/.test(row.expiration)
        ? Date.parse(row.expiration) : NaN;
    if (!Number.isFinite(expiry)) unavailable = true;
    else if (expiry !== Math.floor(Date.parse(submitted.expiration) / 1000) * 1000) mismatch = true;
    if (!Array.isArray(row.conditions)) unavailable = true;
    else {
        if (row.conditions.length !== 1) mismatch = true;
        condition(row.conditions[0]);
    }
    if (Object.hasOwn(row, 'condition')) condition(row.condition);
    const settings = mismatch ? 'mismatch' : unavailable ? 'unknown' : 'matched';
    const confirmation = {identity: 'matched', settings};
    const activation = {state: activationState(row)};
    if (settings !== 'matched' || activation.state === 'stopped') return {
        error: 'Created alert did not pass confirmation',
        error_kind: 'internal_api_unavailable', phase: 'post_check_failed',
        reason: settings === 'mismatch' ? 'settings_mismatch' : settings === 'unknown' ? 'settings_unavailable' : 'activation_stopped',
        created: true, alert_id: id, confirmation, activation, source
    };
    return {row, confirmation, activation};
}
