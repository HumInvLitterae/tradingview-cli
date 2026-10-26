function reorderContext() {
    const found = [];
    for (const element of document.querySelectorAll('[data-collapsed], [data-symbol-full]')) {
        const container = element.closest('[data-name="symbol-list-wrap"]');
        if (!container) {
            continue;
        }
        let fiber = element[Object.keys(element).find(k => k.startsWith('__reactFiber$'))];
        let props;
        let store;
        for (let depth = 0; fiber && depth < 100; depth++, fiber = fiber.return) {
            const p = fiber.memoizedProps;
            if (p?.widgetId === 'watchlist-widget' && typeof p.reorderSymbols === 'function') {
                props = p;
            }
            for (let d = fiber.dependencies?.firstContext; d; d = d.next) {
                const candidate = d.memoizedValue?.store;
                if (typeof candidate?.getState === 'function' && candidate.getState().customLists) {
                    store = candidate;
                }
            }
        }
        if (props && store && !found.some(c => c.store === store && c.container === container)) {
            found.push({ element, container, props, store });
        }
    }
    if (found.length !== 1) {
        throw Error('reorder_handler_unavailable');
    }
    return found[0];
}

function visibleSymbols(parsed) {
    const result = [];
    let hidden = false;
    for (const entry of parsed) {
        if (entry.kind === 'section') {
            result.push('###' + entry.name);
            hidden = entry.collapsed;
        } else if (!hidden) {
            result.push(entry.symbol);
        }
    }
    return result;
}

function restoreHidden(display, parsed) {
    const sections = new Map();
    let section;
    for (const entry of parsed) {
        if (entry.kind === 'section') {
            section = {
                raw: '###' + (entry.collapsed ? '\u2064' : '') + entry.name,
                folded: entry.collapsed,
                children: [],
            };
            sections.set('###' + entry.name, section);
        } else if (section) {
            section.children.push(entry.symbol);
        }
    }
    return display.flatMap(symbol => {
        const section = sections.get(symbol);
        return section ? [section.raw, ...(section.folded ? section.children : [])] : [symbol];
    });
}

function movePlan(parsed, symbols) {
    const seen = new Set();
    for (const e of parsed) {
        if (e.kind === 'section' && e.name.includes('\u2064')) {
            return { error: 'unsupported_section_encoding' };
        }
        const key = e.kind === 'section' ? '###' + e.name : e.symbol;
        if (seen.has(key)) {
            return { error: 'duplicate_entries' };
        }
        seen.add(key);
    }

    const indexOf = (kind, value) =>
        parsed.findIndex(
            e => e.kind === kind && (kind === 'section' ? e.name : e.symbol) === value,
        );
    const endOfSection = index => {
        let end = index + 1;
        while (end < parsed.length && parsed[end].kind !== 'section') {
            end++;
        }
        return end;
    };
    const parent = index => {
        for (let i = index - 1; i >= 0; i--) {
            if (parsed[i].kind === 'section') {
                return parsed[i];
            }
        }
        return null;
    };

    const sectionMove = request.action === 'move_section';
    const start = indexOf(
        sectionMove ? 'section' : 'symbol',
        sectionMove ? request.section : request.symbol,
    );
    if (start < 0) {
        return { error: 'source_not_found' };
    }
    const destination = request.before ?? request.after ?? request.to_section;
    const anchor = indexOf(
        sectionMove || request.to_section !== undefined ? 'section' : 'symbol',
        destination,
    );
    if (anchor < 0) {
        return { error: 'destination_not_found' };
    }
    if (!sectionMove) {
        if (parent(start)?.collapsed) {
            return { error: 'source_collapsed' };
        }
        const destinationSection =
            request.to_section !== undefined ? parsed[anchor] : parent(anchor);
        if (destinationSection?.collapsed) {
            return { error: 'destination_collapsed' };
        }
    }
    if (start === anchor) {
        return { symbols: symbols.slice() };
    }

    const end = sectionMove ? endOfSection(start) : start + 1;
    let insertion =
        request.to_section !== undefined || (sectionMove && request.after !== undefined)
            ? endOfSection(anchor)
            : anchor + (request.after !== undefined ? 1 : 0);
    const result = symbols.slice();
    const moved = result.splice(start, end - start);
    if (insertion >= end) {
        insertion -= end - start;
    }
    result.splice(insertion, 0, ...moved);
    return { symbols: result };
}

function reorderDisplayMatches(id, expected) {
    try {
        const ctx = reorderContext();
        const state = local(ctx, id);
        const parsed = entries(expected.symbols);
        const display = visibleSymbols(parsed);
        if (
            !ctx.container.isConnected ||
            !same(state.entries, parsed) ||
            !same(ctx.props.displaySymbols, display) ||
            !same(ctx.props.symbols, state.row.symbols)
        ) {
            return false;
        }

        const seen = new Set();
        let last = -1;
        for (const row of ctx.container.querySelectorAll('[data-collapsed], [data-symbol-full]')) {
            const folded = row.getAttribute('data-collapsed');
            const symbol =
                folded !== null ? '###' + row.innerText : row.getAttribute('data-symbol-full');
            const index = display.indexOf(symbol);
            if (index < 0) {
                return false;
            }
            if (folded !== null) {
                const entry = parsed.find(e => e.kind === 'section' && '###' + e.name === symbol);
                if (!entry || folded !== String(entry.collapsed)) {
                    return false;
                }
            }
            if (seen.has(symbol)) {
                continue;
            }
            if (index <= last) {
                return false;
            }
            seen.add(symbol);
            last = index;
        }
        return seen.size > 0;
    } catch (_) {
        return false;
    }
}

async function reorderWatchlist() {
    report.concurrency = 'non_atomic_full_replace';
    const initial = reorderContext();
    const id = canonicalId(initial.store.getState().widgets?.[initial.props.widgetId]?.listId);
    report.list_id = id;
    const before = await read(id);
    const parsed = entries(before.symbols);
    const plan = movePlan(parsed, before.symbols);
    if (plan.error) {
        return failure(plan.error, true);
    }
    const expected = { ...before, symbols: plan.symbols };
    const display = visibleSymbols(entries(expected.symbols));
    if (!same(restoreHidden(display, parsed), expected.symbols)) {
        return failure('hidden_members_disagreement', true);
    }

    // A second GET detects edits already made; it is not an atomic server precondition.
    const fresh = await read(id);
    if (!same(comparable(fresh), comparable(before)) || fresh.modified !== before.modified) {
        return failure('saved_state_changed', true);
    }

    const ctx = reorderContext();
    if (ctx.store !== initial.store) {
        return failure('displayed_list_changed', true);
    }
    const state = local(ctx, id);
    if (
        ctx.props.current?.type !== 'custom' ||
        canonicalId(ctx.props.current.id) !== id ||
        state.widget.options?.isMovable !== true
    ) {
        return failure('list_not_movable', true);
    }
    if (!Array.isArray(state.widget.selectedSymbols) || state.widget.selectedSymbols.length) {
        return failure('selected_rows', true);
    }
    if (state.widget.sorting != null) {
        return failure('sorted_view', true);
    }
    if (
        !same(state.entries, parsed) ||
        state.row.name !== before.name ||
        (state.row.description ?? null) !== before.description ||
        !same(ctx.props.symbols, state.row.symbols) ||
        !reorderDisplayMatches(id, before)
    ) {
        return failure('saved_display_disagreement', true);
    }
    if (same(expected.symbols, before.symbols)) {
        report.saved_confirmation = report.display_confirmation = 'matched';
        report.watchlist = fresh;
        return { ...report, ok: true };
    }

    remaining();
    report.write_attempted = true;
    report.changed = null;
    let handlerFailed = false;
    try {
        await bounded(Promise.resolve(ctx.props.reorderSymbols(ctx.props.widgetId, display)));
    } catch (_) {
        handlerFailed = true;
    }
    return await confirmChange(
        id,
        expected,
        () => reorderDisplayMatches(id, expected),
        handlerFailed,
    );
}
