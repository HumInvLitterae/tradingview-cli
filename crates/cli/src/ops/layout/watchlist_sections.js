const inspecting = request.action === 'inspect';
// Leave time for the CDP response inside the transport's ten-second deadline.
const deadline = Date.now() + 8000;
const report = {
    contract_version: inspecting ? 'watchlist_inspect.v1' : 'watchlist_section_mutation.v1',
    source: inspecting ? 'watchlist_api' : 'desktop_watchlist_handler',
    source_category: inspecting ? 'desktop_backed_read' : 'desktop_backed_operation',
    action: request.action,
    requires_desktop: true,
    non_mutating: inspecting,
    write_attempted: false,
    changed: false,
    saved_confirmation: 'not_performed',
    display_confirmation: 'not_performed',
};
function failure(reason, validation = false) {
    return {...report, ok: false, reason, error_kind: validation ? 'validation' : 'internal_api_unavailable'};
}
function remaining() {
    const time = deadline - Date.now();
    if (time <= 0) throw Error('deadline');
    return time;
}
async function bounded(promise) {
    let timer;
    try {
        return await Promise.race([promise, new Promise((_, reject) => {
            timer = setTimeout(() => reject(Error('deadline')), remaining());
        })]);
    } finally {
        clearTimeout(timer);
    }
}
function canonicalId(id) {
    if (typeof id === 'number' && (!Number.isSafeInteger(id) || id < 0)) throw Error('invalid_id');
    const text = String(id);
    if (!/^(0|[1-9][0-9]*)$/.test(text)) throw Error('invalid_id');
    return text;
}
function entries(symbols) {
    if (!Array.isArray(symbols) || !symbols.every(s => typeof s === 'string')) throw Error('unreadable_symbols');
    return symbols.map(symbol => {
        if (!symbol.startsWith('###')) return {kind: 'symbol', symbol};
        const folded = symbol.startsWith('###\u2064');
        const name = symbol.slice(folded ? 4 : 3);
        if (!name || name.startsWith('\u2064')) throw Error('unreadable_section');
        return {kind: 'section', name, collapsed: folded};
    });
}
const same = (a, b) => JSON.stringify(a) === JSON.stringify(b);
function comparable(row) {
    return {id: row.id, name: row.name, description: row.description, shared: row.shared, symbols: row.symbols};
}
async function read(id) {
    const response = await fetch('/api/v1/symbols_list/custom/' + encodeURIComponent(id) + '/?source=web-tvd', {
        credentials: 'include', cache: 'no-store', signal: AbortSignal.timeout(remaining()),
    });
    if (!response.ok) throw Error('read_failed');
    const raw = await response.json();
    if (!raw || canonicalId(raw.id) !== id || (raw.type !== undefined && raw.type !== 'custom')) throw Error('wrong_list');
    if (raw.symbols !== null && raw.symbols !== undefined) entries(raw.symbols);
    return {id, name: raw.name ?? null, description: raw.description ?? null, shared: raw.shared ?? null,
        symbols: raw.symbols ?? null, modified: raw.modified ?? null};
}
function context(name) {
    const marker = '###' + name;
    const found = [];
    for (const element of document.querySelectorAll('[data-collapsed]')) {
        let fiber = element[Object.keys(element).find(k => k.startsWith('__reactFiber$'))];
        let props, store;
        for (let depth = 0; fiber && depth < 100; depth++, fiber = fiber.return) {
            const p = fiber.memoizedProps;
            if (p?.id === marker && p.widgetId === 'watchlist-widget'
                && typeof p.renameSymbol === 'function' && typeof p.setSeparatorCollapse === 'function') props = p;
            for (let d = fiber.dependencies?.firstContext; d; d = d.next) {
                const candidate = d.memoizedValue?.store;
                if (typeof candidate?.getState === 'function' && candidate.getState().customLists) store = candidate;
            }
        }
        if (props && store) found.push({element, props, store});
    }
    if (found.length !== 1) throw Error('section_not_uniquely_rendered');
    return found[0];
}
function local(ctx, id) {
    const state = ctx.store.getState();
    const widget = state.widgets?.[ctx.props.widgetId];
    if (!widget || canonicalId(widget.listId) !== id) throw Error('displayed_list_changed');
    const row = state.customLists?.lists?.byId?.[id];
    if (!row || canonicalId(row.id) !== id) throw Error('not_custom_list');
    const parsed = entries(row.symbols);
    const flags = state.collapsedSeparators?.[id] ?? {};
    for (const entry of parsed) {
        if (entry.kind === 'section') entry.collapsed = flags['###' + entry.name] === true;
    }
    return {widget, row, entries: parsed};
}
function displayMatches(name, folded, id, expected) {
    try {
        const ctx = context(name);
        const state = local(ctx, id);
        return ctx.element.isConnected && same(state.entries, entries(expected.symbols))
            && ctx.element.innerText === name
            && ctx.element.getAttribute('data-collapsed') === String(folded);
    } catch (_) {
        return false;
    }
}
try {
    if (inspecting) {
        report.watchlist = await read(request.id);
        report.saved_confirmation = 'matched';
        return {...report, ok: true};
    }
    const marker = '###' + request.section;
    const ctx = context(request.section);
    const first = ctx.store.getState();
    const id = canonicalId(first.widgets?.[ctx.props.widgetId]?.listId);
    report.list_id = id;
    report.section = request.section;
    const before = await read(id);
    const parsed = entries(before.symbols);
    const matches = parsed.map((e, i) => e.kind === 'section' && e.name === request.section ? i : -1).filter(i => i >= 0);
    if (matches.length !== 1) return failure('section_not_unique', true);
    const index = matches[0];
    const renaming = request.action === 'rename';
    const name = renaming ? request.name : request.section;
    const folded = renaming ? parsed[index].collapsed : request.action === 'collapse';
    if (renaming && name !== request.section && parsed.some(e => e.kind === 'section' && e.name === name)) return failure('name_collision', true);
    const state = local(ctx, id);
    if (!Array.isArray(state.widget.selectedSymbols) || state.widget.selectedSymbols.length !== 0) return failure('selected_rows', true);
    if (state.widget.sorting != null) return failure('sorted_view', true);
    if (!ctx.element.isConnected || !same(state.entries, parsed) || state.row.name !== before.name || (state.row.description ?? null) !== before.description
        || ctx.element.innerText !== request.section || ctx.element.getAttribute('data-collapsed') !== String(parsed[index].collapsed)) {
        return failure('saved_display_disagreement', true);
    }
    const expected = {...before, symbols: before.symbols.slice()};
    expected.symbols[index] = '###' + (folded ? '\u2064' : '') + name;
    if (same(expected.symbols, before.symbols)) {
        report.saved_confirmation = report.display_confirmation = 'matched';
        report.watchlist = before;
        return {...report, ok: true};
    }
    // Capture the handler before dispatch; never find another list after a switch.
    remaining();
    report.write_attempted = true;
    report.changed = null;
    let handlerFailed = false;
    try {
        const result = renaming
            ? ctx.props.renameSymbol(ctx.props.widgetId, marker, '###' + name)
            : ctx.props.setSeparatorCollapse(ctx.props.widgetId, marker, folded);
        await bounded(Promise.resolve(result));
    } catch (_) {
        handlerFailed = true;
    }
    let confirmed;
    while (Date.now() < deadline) {
        if (report.saved_confirmation !== 'matched') {
            try {
                const saved = await read(id);
                report.saved_confirmation = same(comparable(saved), comparable(expected)) ? 'matched' : 'mismatch';
                confirmed = saved;
            } catch (_) {
                if (report.saved_confirmation === 'not_performed') report.saved_confirmation = 'unavailable';
            }
        }
        report.display_confirmation = displayMatches(name, folded, id, expected) ? 'matched' : 'mismatch';
        if (!handlerFailed && report.saved_confirmation === 'matched' && report.display_confirmation === 'matched') {
            report.changed = true;
            report.watchlist = confirmed;
            return {...report, ok: true};
        }
        if (Date.now() + 1000 >= deadline) break;
        await new Promise(resolve => setTimeout(resolve, 1000));
    }
    return failure(handlerFailed ? 'handler_failed' : 'confirmation_failed');
} catch (_) {
    if (report.write_attempted) {
        report.saved_confirmation = report.display_confirmation = 'unavailable';
    }
    return failure(report.write_attempted ? 'outcome_unknown' : 'preflight_unavailable');
}
