const assert = require('node:assert/strict');
const originalNow = Date.now;
const originalTimeout = global.setTimeout;
(async () => {
    for (const mode of ['rename', 'rename_folded', 'collapse', 'expand', 'noop', 'selected', 'sorted', 'duplicate', 'collision',
        'stale', 'switch_before', 'select_before', 'switched', 'saved_only', 'display_only', 'rollback', 'read_failure', 'inspect', 'inspect_unknown', 'wrong_id']) {
        let now = 0;
        Date.now = () => (now += 150);
        global.setTimeout = (callback, delay) => delay === 1000
            ? originalTimeout(() => { now += delay; callback(); }, 0)
            : originalTimeout(callback, delay);
        let calls = 0;
        let reads = 0;
        let displayed = 'Old';
        let folded = mode === 'expand' || mode === 'rename_folded';
        let saved = {id: 12, type: 'custom', name: 'Example', description: null, shared: false,
            symbols: [folded ? '###\u2064Old' : '###Old', 'NASDAQ:EXAMPLE', '###Other']};
        if (mode === 'duplicate') saved.symbols.push('###Old');
        const state = {widgets: {'watchlist-widget': {listId: 12, selectedSymbols: mode === 'selected' ? ['NASDAQ:EXAMPLE'] : [], sorting: mode === 'sorted' ? {} : null}},
            customLists: {lists: {byId: {'12': {id: 12, name: 'Example', description: null,
                symbols: saved.symbols.map(s => s.replace('###\u2064', '###'))}}}},
            collapsedSeparators: {'12': folded ? {'###Old': true} : {}}};
        if (mode === 'stale') state.customLists.lists.byId['12'].symbols.pop();
        const store = {getState: () => state};
        const change = (name, collapsed) => {
            calls++;
            if (mode === 'rollback') throw Error('synthetic failure');
            if (mode !== 'display_only') saved.symbols[0] = '###' + (collapsed ? '\u2064' : '') + name;
            if (mode !== 'saved_only') {
                displayed = name;
                folded = collapsed;
                state.customLists.lists.byId['12'].symbols[0] = '###' + name;
                state.collapsedSeparators['12'] = collapsed ? {['###' + name]: true} : {};
            }
            if (mode === 'switched') state.widgets['watchlist-widget'].listId = 99;
        };
        const props = {widgetId: 'watchlist-widget', get id() {return '###' + displayed;},
            renameSymbol: (widget, old, next) => {
                assert.equal(widget, 'watchlist-widget');
                assert.equal(old, '###Old');
                change(next.slice(3), folded);
            },
            setSeparatorCollapse: (widget, old, next) => {
                assert.equal(widget, 'watchlist-widget');
                assert.equal(old, '###Old');
                change(displayed, next);
            }};
        const element = {get innerText() {return displayed;}, isConnected: true,
            getAttribute: () => String(folded),
            __reactFiber$fixture: {memoizedProps: props, dependencies: {firstContext: {memoizedValue: {store}}}}};
        global.document = {querySelectorAll: () => [element]};
        global.fetch = async (url, options) => {
            assert.ok(url.includes('/custom/12/'));
            assert.equal(options.method, undefined, 'adapter must not POST directly');
            reads++;
            if (mode === 'switch_before') state.widgets['watchlist-widget'].listId = 99;
            if (mode === 'select_before') state.widgets['watchlist-widget'].selectedSymbols = ['NASDAQ:EXAMPLE'];
            if (mode === 'read_failure' && calls) throw Error('unavailable');
            const value = structuredClone(saved);
            if (mode === 'wrong_id') value.id = 99;
            if (mode === 'inspect_unknown') delete value.symbols;
            return {ok: true, json: async () => value};
        };
        const inspecting = mode.startsWith('inspect') || mode === 'wrong_id';
        const request = inspecting ? {action: 'inspect', id: '12'}
            : mode === 'collapse' ? {action: 'collapse', section: 'Old'}
            : mode === 'expand' ? {action: 'expand', section: 'Old'}
            : {action: 'rename', section: 'Old', name: mode === 'noop' ? 'Old' : mode === 'collision' ? 'Other' : 'New'};
        const result = await operation(request);
        const preflight = ['selected', 'sorted', 'duplicate', 'collision', 'stale', 'switch_before', 'select_before', 'wrong_id'].includes(mode);
        const success = ['rename', 'rename_folded', 'collapse', 'expand', 'noop', 'inspect', 'inspect_unknown'].includes(mode);
        assert.equal(result.ok, success, mode);
        assert.equal(calls, inspecting || preflight || mode === 'noop' ? 0 : 1, mode);
        assert.equal(result.write_attempted, calls === 1, mode);
        if (success && !inspecting) {
            assert.equal(result.saved_confirmation, 'matched', mode);
            assert.equal(result.display_confirmation, 'matched', mode);
            assert.equal(result.changed, mode !== 'noop', mode);
        }
        if (mode === 'saved_only') {
            assert.equal(result.saved_confirmation, 'matched');
            assert.equal(result.display_confirmation, 'mismatch');
        }
        if (mode === 'display_only') {
            assert.equal(result.saved_confirmation, 'mismatch');
            assert.equal(result.display_confirmation, 'matched');
        }
        if (mode === 'inspect_unknown') assert.equal(result.watchlist.symbols, null);
        assert.ok(reads >= 1);
    }
    Date.now = originalNow;
    global.setTimeout = originalTimeout;
})().catch(error => { Date.now = originalNow; global.setTimeout = originalTimeout; console.error(error); process.exitCode = 1; });
