const assert = require('node:assert/strict');
const realNow = Date.now;
const realTimeout = global.setTimeout;

const U = 'NASDAQ:U';
const X = 'NASDAQ:X';
const Y = 'NASDAQ:Y';
const Z = 'NASDAQ:Z';
const W = 'NASDAQ:W';
const Q = 'NASDAQ:Q';
const A = '###A';
const B = '###\u2064B';
const E = '###Empty';
const C = '###C';

const initial = [U, A, X, Y, B, Z, W, E, C, Q];
const cases = [
    ['flat', { action: 'move_symbol', symbol: Y, before: X }, [Y, X], [X, Y]],
    [
        'empty_section',
        { action: 'move_section', section: 'Empty', after: 'C' },
        [U, A, X, Y, B, Z, W, C, Q, E],
    ],
    ['within', { action: 'move_symbol', symbol: X, after: Y }, [U, A, Y, X, B, Z, W, E, C, Q]],
    ['cross', { action: 'move_symbol', symbol: X, before: Q }, [U, A, Y, B, Z, W, E, C, X, Q]],
    [
        'empty',
        { action: 'move_symbol', symbol: X, to_section: 'Empty' },
        [U, A, Y, B, Z, W, E, X, C, Q],
    ],
    [
        'section_after',
        { action: 'move_section', section: 'A', after: 'C' },
        [U, B, Z, W, E, C, Q, A, X, Y],
    ],
    [
        'section_before',
        { action: 'move_section', section: 'B', before: 'A' },
        [U, B, Z, W, A, X, Y, E, C, Q],
    ],
    ['noop', { action: 'move_symbol', symbol: X, before: Y }, initial],
    ['self', { action: 'move_section', section: 'A', after: 'A' }, initial],
    ['hidden_source', { action: 'move_symbol', symbol: Z, before: X }],
    ['hidden_destination', { action: 'move_symbol', symbol: X, before: Z }],
    ['hidden_section', { action: 'move_symbol', symbol: X, to_section: 'B' }],
    ...[
        'duplicate_symbol',
        'duplicate_section',
        'reserved_marker',
        'conflict',
        'revision',
        'switched',
        'selected',
        'sorted',
        'immovable',
        'stale_dom',
        'saved_only',
        'display_only',
        'throw',
        'lost_read',
        'changed_metadata',
        'post_switch',
    ].map(mode => [mode, { action: 'move_symbol', symbol: X, after: Y }]),
];

(async () => {
    for (const [mode, request, expected, source = initial] of cases) {
        let now = 0;
        let reads = 0;
        let calls = 0;

        Date.now = () => (now += 20);
        global.setTimeout = (fn, delay) =>
            delay === 1000
                ? realTimeout(() => {
                      now += delay;
                      fn();
                  }, 0)
                : realTimeout(fn, delay);

        let saved = {
            id: 12,
            type: 'custom',
            name: 'Example',
            description: null,
            shared: false,
            modified: 'initial',
            symbols: source.slice(),
        };
        if (mode === 'duplicate_symbol') {
            saved.symbols.push(Q);
        }
        if (mode === 'duplicate_section') {
            saved.symbols.push('###\u2064A');
        }
        if (mode === 'reserved_marker') {
            saved.symbols.push('###Odd\u2064name');
        }

        const logical = symbols => symbols.map(s => s.replace('###\u2064', '###'));
        const shown = symbols => logical(symbols).filter(s => s !== Z && s !== W);
        const state = {
            widgets: {
                'watchlist-widget': {
                    listId: 12,
                    selectedSymbols: [],
                    sorting: mode === 'sorted' ? {} : null,
                    options: { isMovable: mode !== 'immovable' },
                },
            },
            customLists: {
                lists: {
                    byId: {
                        '12': {
                            id: 12,
                            name: 'Example',
                            description: null,
                            symbols: logical(saved.symbols),
                        },
                    },
                },
            },
            collapsedSeparators: { '12': { '###B': true } },
        };

        const row = state.customLists.lists.byId['12'];
        let display = shown(saved.symbols);
        let dom = mode === 'stale_dom' ? display.slice().reverse() : display.slice();
        const store = { getState: () => state };
        const props = {
            widgetId: 'watchlist-widget',
            current: { id: 12, type: 'custom' },
            get symbols() {
                return row.symbols;
            },
            get displaySymbols() {
                return display;
            },
            reorderSymbols(widget, values) {
                assert.equal(widget, 'watchlist-widget');
                calls++;
                if (mode === 'throw') {
                    throw Error('synthetic failure');
                }
                const expanded = values.flatMap(s => (s === '###B' ? [B, Z, W] : [s]));
                if (mode !== 'saved_only') {
                    row.symbols = logical(expanded);
                    display = values.slice();
                    dom = values.slice();
                }
                if (mode !== 'display_only') {
                    saved.symbols = expanded;
                }
                if (mode === 'changed_metadata') {
                    saved.description = 'unexpected';
                }
                if (mode === 'post_switch') {
                    state.widgets['watchlist-widget'].listId = 99;
                }
            },
        };

        const fiber = {
            memoizedProps: props,
            dependencies: { firstContext: { memoizedValue: { store } } },
        };
        const container = {
            isConnected: true,
            querySelectorAll: () =>
                dom.map(symbol => ({
                    innerText: symbol.slice(3),
                    getAttribute: name =>
                        name === 'data-collapsed'
                            ? symbol.startsWith('###')
                                ? String(symbol === '###B')
                                : null
                            : name === 'data-symbol-full' && !symbol.startsWith('###')
                              ? symbol
                              : null,
                })),
        };
        const element = { __reactFiber$fixture: fiber, closest: () => container };
        global.document = { querySelectorAll: () => [element] };

        global.fetch = async (url, options) => {
            assert.ok(url.includes('/custom/12/'));
            assert.equal(options.method, undefined, 'adapter must never POST directly');
            reads++;
            if (reads === 2) {
                if (mode === 'conflict') {
                    saved.symbols.push('NASDAQ:NEW');
                }
                if (mode === 'revision') {
                    saved.modified = 'concurrent';
                }
                if (mode === 'switched') {
                    state.widgets['watchlist-widget'].listId = 99;
                }
                if (mode === 'selected') {
                    state.widgets['watchlist-widget'].selectedSymbols = [X];
                }
            }
            if (mode === 'lost_read' && calls) {
                throw Error('read unavailable');
            }
            return { ok: true, json: async () => structuredClone(saved) };
        };

        const result = await operation(request);
        const postFailure = [
            'saved_only',
            'display_only',
            'throw',
            'lost_read',
            'changed_metadata',
            'post_switch',
        ].includes(mode);
        assert.equal(result.ok, !!expected, mode);
        assert.equal(
            calls,
            (expected && !['noop', 'self'].includes(mode)) || postFailure ? 1 : 0,
            mode,
        );
        assert.equal(result.write_attempted, calls === 1, mode);
        assert.equal(result.concurrency, 'non_atomic_full_replace', mode);
        if (expected) {
            assert.deepEqual(saved.symbols, expected, mode);
            assert.equal(result.saved_confirmation, 'matched', mode);
            assert.equal(result.display_confirmation, 'matched', mode);
            assert.equal(result.changed, calls === 1, mode);
            assert.ok(reads >= 2);
        }
        if (mode === 'saved_only') {
            assert.equal(result.saved_confirmation, 'matched');
            assert.equal(result.display_confirmation, 'mismatch');
        }
        if (mode === 'display_only') {
            assert.equal(result.saved_confirmation, 'mismatch');
            assert.equal(result.display_confirmation, 'matched');
        }
        if (postFailure) {
            assert.equal(result.changed, null);
        }
    }
})().then(
    () => {
        Date.now = realNow;
        global.setTimeout = realTimeout;
    },
    error => {
        Date.now = realNow;
        global.setTimeout = realTimeout;
        console.error(error);
        process.exitCode = 1;
    },
);
