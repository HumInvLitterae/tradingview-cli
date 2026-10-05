async function createSavedScript(name, source) {
    const notSaved = reason => ({status: 'not_saved', reason});
    const unknown = reason => ({status: 'unknown', reason});
    const version = value => {
        if (typeof value === 'string' && value.trim()) return value;
        if (typeof value === 'number' && Number.isFinite(value)) return String(value);
        return null;
    };
    async function readJson(path) {
        const response = await fetch('https://pine-facade.tradingview.com/pine-facade/' + path, {
            credentials: 'include', redirect: 'error', cache: 'no-store'
        });
        if (!response.ok) throw new Error('read failed');
        return response.json();
    }
    async function catalog() {
        const rows = await readJson('list/?filter=saved');
        if (!Array.isArray(rows)) throw new Error('catalog unavailable');
        const ids = new Set();
        for (const row of rows) {
            if (!row || typeof row.scriptIdPart !== 'string' || !row.scriptIdPart.trim() ||
                typeof row.scriptName !== 'string' || !row.scriptName.trim() ||
                ids.has(row.scriptIdPart)) throw new Error('catalog unavailable');
            ids.add(row.scriptIdPart);
        }
        return rows;
    }

    let api;
    let save;
    let before;
    try {
        api = window.TradingViewApi && window.TradingViewApi._pineEditorApi;
        save = api && api.saveNewScript;
        if (typeof save !== 'function') return notSaved('save_api_unavailable');
        before = await catalog();
    } catch (_) {
        return notSaved('catalog_unavailable');
    }
    if (before.some(row => row.scriptName.trim() === name)) return notSaved('name_conflict');

    let result;
    try {
        result = await save.call(api, {source, name});
    } catch (_) {
        return unknown('save_request_failed');
    }
    const meta = result && result.metaInfo;
    let id;
    let savedVersion;
    try {
        id = meta && meta.scriptIdPart;
        savedVersion = meta && meta.pine && version(meta.pine.version);
        if (typeof id !== 'string' || !id.trim() || savedVersion === null ||
            savedVersion === undefined || before.some(row => row.scriptIdPart === id)) {
            return unknown('save_identity_unavailable');
        }
    } catch (_) {
        return unknown('save_identity_unavailable');
    }

    try {
        const rows = await catalog();
        const named = rows.filter(row => row.scriptName.trim() === name);
        if (named.length !== 1 || named[0].scriptIdPart !== id ||
            version(named[0].version) !== savedVersion) return unknown('saved_identity_mismatch');
    } catch (_) {
        return unknown('catalog_readback_failed');
    }
    let saved;
    try {
        saved = await readJson('get/' + encodeURIComponent(id) + '/' + encodeURIComponent(savedVersion));
        if (!saved || saved.success === false || saved.error != null || saved.err != null ||
            typeof saved.source !== 'string') return unknown('source_readback_failed');
    } catch (_) {
        return unknown('source_readback_failed');
    }
    return {
        status: 'saved',
        script: {id, name, version: savedVersion},
        saved_source: saved.source,
        compilation: {
            compiled: result.success,
            errors: result.compileErrors && result.compileErrors.errors,
            warnings: result.compileErrors && result.compileErrors.warnings
        }
    };
}
