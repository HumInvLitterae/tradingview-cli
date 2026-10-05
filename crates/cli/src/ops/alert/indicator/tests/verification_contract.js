const assert = require('node:assert/strict');
const scriptId = 'private-script-id';
const version = '4.0';
const systemIds = ['text', 'pineId', 'pineVersion', 'pineFeatures', '__fast_calc', '__profile'];

function metadata(inputs) {
    return {
        id: 'Script@tv-scripting-101', isTVScript: true, scriptIdPart: scriptId,
        pine: {version}, version: 101,
        inputs: [...systemIds.map(id => ({id})), ...(inputs ? [{id: 'in_0', defval: 20}] : [])],
        plots: [{id: 'plot_0', type: 'alertcondition'}], styles: {plot_0: {title: 'Long'}}
    };
}

function study(inputs, entityId = 'chosen', value = 42) {
    return {
        entityId, meta: metadata(inputs),
        values: [
            {id: 'text', value: 'secret-marker-source'}, {id: 'pineId', value: scriptId},
            {id: 'pineVersion', value: version}, {id: 'pineFeatures', value: 'secret-marker-features'},
            {id: '__fast_calc', value: true}, {id: '__profile', value: true},
            ...(inputs ? [{id: 'in_0', value}] : [])
        ]
    };
}

function revise(study, id, revision) {
    study.meta.scriptIdPart = id;
    study.meta.pine.version = revision;
    study.values.find(input => input.id === 'pineId').value = id;
    study.values.find(input => input.id === 'pineVersion').value = revision;
}

const cases = [
    {name: 'no-input-no-study', inputs: false},
    {name: 'condition-id-without-local-title', inputs: false, idOnly: true, change: s => {
        delete s.compiled.styles;
    }},
    {name: 'no-input-no-inventory-api', inputs: false, change: s => {s.noInventory = true;}},
    {name: 'no-input-explicit', inputs: false, explicit: true, change: s => {s.studies = [study(false)];}},
    {name: 'unique-inputs', inputs: true},
    {name: 'chart-context-after-compilation-read', inputs: true, symbol: 'EXCHANGE:UPDATED', resolution: '60', change: s => {
        s.afterCompilation = () => {s.symbol = 'EXCHANGE:UPDATED'; s.resolution = '60';};
    }},
    {name: 'explicit-inputs', inputs: true, explicit: true},
    {name: 'other-native-study', inputs: true, change: s => {
        s.studies.unshift({entityId: 'builtin', meta: {id: 'Volume@tv-basicstudies', inputs: []}, values: []});
    }},
    {name: 'other-saved-revision', inputs: true, change: s => {
        const other = study(true, 'other'); revise(other, 'other-script', '2.0'); s.studies.unshift(other);
    }},
    {name: 'duplicates-explicit', inputs: true, explicit: true, change: s => {
        s.studies.unshift(study(true, 'other', 99));
    }},
    {name: 'unreadable-unselected-explicit', inputs: true, explicit: true, change: s => {
        s.studies.unshift({entityId: 'other', throws: true});
    }},
    {name: 'native-value-alias', inputs: true, change: s => {
        for (const input of s.studies[0].values) {input.val = input.value; delete input.value;}
    }},
    {name: 'reordered-input-ids', inputs: true, expected: {in_0: 42, in_2: false}, change: s => {
        s.compiled.inputs.push({id: 'in_2', defval: true});
        s.studies[0].meta.inputs.unshift({id: 'in_2'});
        s.studies[0].values.unshift({id: 'in_2', value: false});
    }},
    {name: 'input-array-snapshot', inputs: true, expected: {in_0: [0, '', false]}, change: s => {
        s.studies[0].values.find(i => i.id === 'in_0').value = [0, '', false];
        s.beforeList = () => s.studies[0].values.find(i => i.id === 'in_0').value.push('changed');
    }},
    {name: 'missing-study', inputs: true, reason: 'no_matching_study', change: s => {s.studies = [];}},
    {name: 'explicit-missing', inputs: true, explicit: true, reason: 'study_not_found', change: s => {
        s.studies[0].entityId = 'other';
    }},
    {name: 'duplicates-automatic', inputs: true, reason: 'ambiguous_study', change: s => {
        s.studies.push(study(true, 'other', 99));
    }},
    {name: 'wrong-label-match', inputs: true, reason: 'no_matching_study', change: s => {
        revise(s.studies[0], 'other-script', version);
    }},
    {name: 'wrong-revision-explicit', inputs: true, explicit: true, reason: 'saved_revision_mismatch', change: s => {
        revise(s.studies[0], scriptId, '3.0');
    }},
    {name: 'native-identities-disagree', inputs: true, reason: 'study_metadata_unavailable', change: s => {
        s.studies[0].values.find(i => i.id === 'pineVersion').value = '3.0';
    }},
    {name: 'native-missing-version', inputs: true, reason: 'study_metadata_unavailable', change: s => {
        delete s.studies[0].meta.pine;
    }},
    {name: 'native-missing-system-id', inputs: true, reason: 'study_metadata_unavailable', change: s => {
        s.studies[0].values = s.studies[0].values.filter(i => i.id !== 'pineId');
    }},
    {name: 'unknown-study', inputs: true, reason: 'study_metadata_unavailable', change: s => {
        s.studies.unshift({entityId: 'other', meta: {}, values: []});
    }},
    {name: 'unreadable-possible-duplicate', inputs: true, reason: 'study_metadata_unavailable', change: s => {
        s.studies.unshift({entityId: 'other', throws: true});
    }},
    {name: 'missing-inventory', inputs: true, reason: 'study_metadata_unavailable', change: s => {s.noInventory = true;}},
    {name: 'duplicate-entity-id', inputs: true, reason: 'study_metadata_unavailable', change: s => {
        s.studies.push(study(true));
    }},
    {name: 'native-declaration-mismatch', inputs: true, reason: 'input_declarations_mismatch', change: s => {
        s.studies[0].meta.inputs.push({id: 'in_1'});
    }},
    {name: 'native-missing-user-value', inputs: true, reason: 'input_values_unavailable', change: s => {
        s.studies[0].values = s.studies[0].values.filter(i => i.id !== 'in_0');
    }},
    {name: 'native-default-not-actual', inputs: true, reason: 'input_values_unavailable', change: s => {
        const value = s.studies[0].values.find(i => i.id === 'in_0'); delete value.value; value.defval = 20;
    }},
    {name: 'native-duplicate-user-value', inputs: true, reason: 'input_values_unavailable', change: s => {
        s.studies[0].values.push({id: 'in_0', value: 99});
    }},
    {name: 'compiled-wrong-id', inputs: false, reason: 'saved_revision_mismatch', change: s => {
        s.compiled.scriptIdPart = 'other-script';
    }},
    {name: 'compiled-wrong-version', inputs: false, reason: 'saved_revision_mismatch', change: s => {
        s.compiled.pine.version = '3.0';
    }},
    {name: 'compiled-engine-version-only', inputs: false, reason: 'compiled_metadata_unavailable', change: s => {
        delete s.compiled.pine;
    }},
    {name: 'compiled-missing-inputs', inputs: false, reason: 'compiled_metadata_unavailable', change: s => {
        delete s.compiled.inputs;
    }},
    {name: 'compiled-duplicate-input', inputs: true, reason: 'compiled_metadata_unavailable', change: s => {
        s.compiled.inputs.push({id: 'in_0'});
    }},
    {name: 'compiled-unknown-input-id', inputs: false, reason: 'compiled_metadata_unavailable', change: s => {
        s.compiled.inputs.push({id: 'unknown'});
    }},
    {name: 'compiled-wrong-condition-id', inputs: false, reason: 'condition_mismatch', change: s => {
        s.compiled.plots[0].id = 'plot_1';
    }},
    {name: 'compiled-wrong-condition-type', inputs: false, reason: 'condition_mismatch', change: s => {
        s.compiled.plots[0].type = 'line';
    }},
    {name: 'compiled-wrong-title', inputs: false, reason: 'condition_mismatch', change: s => {
        s.compiled.styles.plot_0.title = 'Other';
    }},
    {name: 'compiled-missing-title', inputs: false, reason: 'compiled_metadata_unavailable', change: s => {
        delete s.compiled.styles;
    }},
    {name: 'compiled-duplicate-plot', inputs: false, reason: 'compiled_metadata_unavailable', change: s => {
        s.compiled.plots.push(s.compiled.plots[0]);
    }},
    ...['network', 'http', 'json', 'rejected', 'missing'].map(failure => ({
        name: 'compiled-' + failure, inputs: false, reason: 'compiled_metadata_unavailable',
        change: s => {s.compileFailure = failure;}
    })),
    ...['pre-list', 'create', 'post-list', 'post-match'].map(failure => ({
        name: 'account-' + failure, inputs: true, normalOnly: true, accountFailure: failure,
        change: s => {s.accountFailure = failure;}
    }))
];

(async () => {
    const results = [];
    for (const test of cases) {
        for (const dryRun of test.normalOnly ? [false] : [false, true]) {
            const explicit = test.explicit === true;
            const state = {compiled: metadata(test.inputs), studies: test.inputs ? [study(true)] : []};
            if (test.change) test.change(state);
            let compilations = 0, reads = 0, writes = 0, inventories = 0, payload;
            const chart = {
                symbolExt: () => ({symbol: state.symbol || 'EXCHANGE:EXAMPLE'}),
                resolution: () => state.resolution || '1D',
                getAllStudies: () => {inventories++; return state.studies.map(s => ({id: s.entityId, name: 'Signals'}));},
                getStudyById: id => {
                    const selected = state.studies.find(s => s.entityId === id);
                    assert.ok(selected, 'selector must use an inventoried entity');
                    if (selected.throws) throw new Error('secret-marker-native-error');
                    return {_study: {metaInfo: () => selected.meta}, getInputValues: () => selected.values};
                }
            };
            if (state.noInventory) delete chart.getAllStudies;
            global.window = {TradingViewApi: {_activeChartWidgetWV: {value: () => chart}}};
            global.fetch = async (url, options) => {
                if (url.startsWith('https://pine-facade.tradingview.com/')) {
                    compilations++;
                    assert.equal(url, 'https://pine-facade.tradingview.com/pine-facade/translate/private-script-id/4.0');
                    assert.deepEqual(options, {credentials: 'include', redirect: 'error'});
                    if (state.compileFailure === 'network') throw new Error('secret-marker-fetch-error');
                    return {
                        ok: state.compileFailure !== 'http',
                        json: async () => {
                            if (state.compileFailure === 'json') throw new Error('secret-marker-json-error');
                            if (state.compileFailure === 'rejected') return {success: false, error: 'secret-marker'};
                            if (state.afterCompilation) state.afterCompilation();
                            return {success: true, result: state.compileFailure === 'missing' ? {} : {metaInfo: state.compiled}};
                        }
                    };
                }
                if (url.endsWith('/create_alert')) {
                    assert.equal(options.method, 'POST');
                    writes++;
                    payload = JSON.parse(options.body).payload;
                    if (state.accountFailure === 'create') throw new Error('synthetic creation loss');
                    return {ok: true, text: async () => '{"s":"ok"}'};
                }
                assert.equal(url, 'https://pricealerts.tradingview.com/list_alerts');
                reads++;
                if (state.beforeList) state.beforeList();
                if (state.accountFailure === 'pre-list' || (payload && state.accountFailure === 'post-list')) {
                    return {ok: false};
                }
                return {ok: true, json: async () => ({r: payload && state.accountFailure !== 'post-match' ? [{
                    id: 'new', symbol: payload.symbol, message: payload.message, condition: payload.conditions[0]
                }] : []})};
            };
            const key = test.idOnly ? `idOnly:${dryRun}` : `${test.inputs}:${dryRun}:${explicit}`;
            const result = await eval(expressions[key]);
            assert.equal(compilations, 1, test.name);
            assert.equal(JSON.stringify(result).includes('secret-marker'), false, test.name);
            if (test.reason) {
                assert.equal(result.reason, test.reason, test.name);
                assert.equal(result.created, false, test.name);
                assert.equal(reads, 0, test.name);
                assert.equal(writes, 0, test.name);
                assert.equal(result.creation_outcome, undefined, test.name);
                if (test.reason === 'ambiguous_study') assert.equal(result.match_count, 2);
            } else if (test.accountFailure) {
                const phases = {'pre-list': 'pre_list_unavailable', create: 'create_request_unavailable',
                    'post-list': 'post_list_unavailable', 'post-match': 'post_check_failed'};
                assert.equal(result.phase, phases[test.accountFailure], test.name);
                assert.equal(writes, test.accountFailure === 'pre-list' ? 0 : 1, test.name);
                assert.equal(result.created, test.accountFailure === 'pre-list' ? false : null, test.name);
                if (writes) assert.equal(result.creation_outcome, 'unknown', test.name);
            } else {
                const expected = test.expected || (test.inputs ? {in_0: 42} : {});
                assert.equal(result.symbol, test.symbol || 'EXCHANGE:EXAMPLE', test.name);
                assert.equal(result.resolution, test.resolution || '1D', test.name);
                assert.deepEqual(result.verification, {
                    condition_source: 'saved_compilation', input_source: test.inputs ? 'active_chart_study' : 'none',
                    input_count: Object.keys(expected).length,
                    study: test.inputs || explicit ? {entity_id: 'chosen', selection: explicit ? 'explicit' : 'unique'} : null
                }, test.name);
                assert.equal(reads, dryRun ? 0 : 2, test.name);
                assert.equal(writes, dryRun ? 0 : 1, test.name);
                if (!test.inputs && !explicit) assert.equal(inventories, 0, test.name);
                if (!dryRun) {
                    const sent = payload.conditions[0].series[0].inputs;
                    assert.deepEqual(Object.fromEntries(Object.entries(sent).filter(([id]) => /^in_\d+$/.test(id))), expected, test.name);
                    assert.equal(sent.text, undefined, test.name);
                    assert.equal(sent.pineId, undefined, test.name);
                    assert.equal(sent.__fast_calc, false, test.name);
                    assert.equal(sent.__profile, false, test.name);
                }
            }
            results.push({name: test.name, inputs: test.inputs, id_only: test.idOnly === true, explicit, dry_run: dryRun,
                success: !test.reason && !test.accountFailure, result});
        }
    }
    console.log(JSON.stringify(results));
})().catch(error => {console.error(error); process.exitCode = 1;});
