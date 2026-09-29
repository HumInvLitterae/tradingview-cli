async function __readAlertRows() {
    const unavailable = { ok: false, error: 'Alert list unavailable', phase: 'list_unavailable' };
    const invalid = { ok: false, error: 'Invalid alert list response', phase: 'invalid_response' };
    let response;
    try {
        response = await fetch('https://pricealerts.tradingview.com/list_alerts', {
            credentials: 'include',
            headers: { 'accept': 'application/json' }
        });
    } catch (_) {
        return unavailable;
    }
    if (!response.ok) return unavailable;

    let data;
    try {
        data = await response.json();
    } catch (_) {
        return invalid;
    }
    if (!data || typeof data !== 'object' || Array.isArray(data)) return invalid;
    if (data.err || data.success === false) return unavailable;
    if (!Array.isArray(data.r) || data.r.some(row => !row || typeof row !== 'object' || Array.isArray(row))) {
        return invalid;
    }
    return { ok: true, alerts: data.r };
}
