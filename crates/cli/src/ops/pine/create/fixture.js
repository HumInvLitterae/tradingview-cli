const assert = require('node:assert/strict');
const expression = __EXPRESSION__;
const name = __NAME__;
const source = __SOURCE__;
const existing = {scriptIdPart: 'synthetic-existing', scriptName: 'Existing', version: '4.0'};
const created = {scriptIdPart: 'synthetic/new? id', scriptName: name, version: '1.0'};
const cases = [
    ['success', 'saved', 1], ['crlf', 'saved', 1], ['compile-error', 'saved', 1],
    ['compile-unknown', 'saved', 1], ['numeric-version', 'saved', 1],
    ['missing-api', 'not_saved', 0], ['collision', 'not_saved', 0],
    ['catalog-http', 'not_saved', 0], ['catalog-rejected', 'not_saved', 0],
    ['catalog-json', 'not_saved', 0], ['catalog-null', 'not_saved', 0],
    ['catalog-row', 'not_saved', 0], ['catalog-duplicate-id', 'not_saved', 0],
    ['save-rejected', 'unknown', 1], ['racing-conflict', 'unknown', 1],
    ['result-null', 'unknown', 1], ['missing-id', 'unknown', 1],
    ['existing-id', 'unknown', 1], ['missing-version', 'unknown', 1],
    ['readback-http', 'unknown', 1], ['readback-rejected', 'unknown', 1],
    ['readback-json', 'unknown', 1], ['readback-missing', 'unknown', 1],
    ['readback-name', 'unknown', 1], ['readback-version', 'unknown', 1],
    ['readback-duplicate-name', 'unknown', 1], ['source-http', 'unknown', 1],
    ['source-rejected', 'unknown', 1], ['source-json', 'unknown', 1],
    ['source-missing', 'unknown', 1], ['source-error', 'unknown', 1],
    ['source-mismatch', 'unknown', 1], ['source-final-newline', 'unknown', 1]
];
(async () => {
    const results = [];
    for (const [test, expected, expectedSaves] of cases) {
        let saves = 0;
        let reads = 0;
        const row = structuredClone(created);
        const api = {
            saveNewScript: async function (request) {
                assert.equal(this, api);
                assert.deepEqual(request, {source, name});
                saves++;
                if (test === 'save-rejected' || test === 'racing-conflict') throw Error('private native error');
                if (test === 'result-null') return null;
                const meta = {scriptIdPart: row.scriptIdPart, pine: {version: row.version}};
                if (test === 'missing-id') delete meta.scriptIdPart;
                if (test === 'existing-id') meta.scriptIdPart = existing.scriptIdPart;
                if (test === 'missing-version') delete meta.pine.version;
                if (test === 'numeric-version') meta.pine.version = 1;
                return {
                    success: test !== 'compile-error', metaInfo: meta,
                    compileErrors: test === 'compile-unknown' ? null : {
                        errors: test === 'compile-error' ? [{message: 'Synthetic compile error'}] : [],
                        warnings: []
                    }
                };
            }
        };
        global.window = {TradingViewApi: new Proxy({_pineEditorApi: test === 'missing-api' ? {} : api}, {
            get(target, key) {
                assert.equal(key, '_pineEditorApi', 'must not access editor/chart');
                return target[key];
            }
        })};
        global.document = new Proxy({}, {get() { throw Error('must not access DOM'); }});
        global.fetch = async (url, options) => {
            reads++;
            assert.deepEqual(options, {credentials: 'include', redirect: 'error', cache: 'no-store'});
            assert.equal(options.method, undefined, 'no direct mutation request');
            if (url.endsWith('list/?filter=saved')) {
                const phase = saves === 0 ? 'catalog' : 'readback';
                if (test === phase + '-rejected') throw Error('private catalog error');
                if (test === phase + '-http') return {ok: false};
                if (test === phase + '-json') return {ok: true, json: async () => {throw Error('private JSON');}};
                let rows = saves === 0 ? [existing] : [existing, row];
                if (test === 'collision') rows = [existing, row];
                if (test === 'catalog-null') rows = null;
                if (test === 'catalog-row') rows = [{}];
                if (test === 'catalog-duplicate-id') rows = [existing, existing];
                if (saves > 0) {
                    if (test === 'readback-missing') rows = [existing];
                    if (test === 'readback-name') row.scriptName = 'Other';
                    if (test === 'readback-version') row.version = '2.0';
                    if (test === 'numeric-version') row.version = 1;
                    if (test === 'readback-duplicate-name') rows.push({...row, scriptIdPart: 'other-new'});
                }
                return {ok: true, json: async () => structuredClone(rows)};
            }
            const version = test === 'numeric-version' ? '1' : '1.0';
            assert.equal(url, 'https://pine-facade.tradingview.com/pine-facade/get/' +
                encodeURIComponent(created.scriptIdPart) + '/' + encodeURIComponent(version));
            if (test === 'source-rejected') throw Error('private source error');
            if (test === 'source-http') return {ok: false};
            if (test === 'source-json') return {ok: true, json: async () => {throw Error('private JSON');}};
            let body = {source};
            if (test === 'crlf') body.source = source.replaceAll('\n', '\r\n');
            if (test === 'source-missing') body = {};
            if (test === 'source-error') body.error = 'private source error';
            if (test === 'source-mismatch') body.source += 'plot(open)';
            if (test === 'source-final-newline') body.source = source.trimEnd();
            return {ok: true, json: async () => body};
        };
        const raw = await eval(expression);
        assert.equal(saves, expectedSaves, test);
        assert.equal(JSON.stringify(raw).includes('private '), false, test);
        if (expected === 'not_saved') assert.equal(raw.status, 'not_saved', test);
        if (expected === 'saved') assert.equal(raw.status, 'saved', test);
        if (test === 'missing-api') assert.equal(reads, 0);
        results.push({test, expected, raw});
    }
    process.stdout.write(JSON.stringify(results));
})().catch(error => {console.error(error); process.exitCode = 1;});
