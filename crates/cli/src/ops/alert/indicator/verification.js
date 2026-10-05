function verificationFailure(phase, reason, kind = 'internal_api_unavailable', extra = {}) {
    return Object.assign({
        error: 'Indicator alert verification failed: ' + reason,
        error_kind: kind,
        phase,
        reason,
        created: false,
        source
    }, extra);
}

function savedVersion(value) {
    if (typeof value === 'string' && value.trim()) return value;
    if (typeof value === 'number' && Number.isFinite(value)) return String(value);
    return null;
}

async function savedCompilation() {
    const phase = 'compiled_condition_verification';
    const unavailable = () => verificationFailure(phase, 'compiled_metadata_unavailable');
    try {
        const response = await fetch(
            'https://pine-facade.tradingview.com/pine-facade/translate/' +
                encodeURIComponent(pineId) + '/' + encodeURIComponent(pineVersion),
            {credentials: 'include', redirect: 'error'}
        );
        if (!response.ok) return unavailable();
        const body = await response.json();
        const meta = body && body.success === true && body.result && body.result.metaInfo;
        if (!meta || typeof meta.scriptIdPart !== 'string' || !meta.scriptIdPart.trim() ||
            !meta.pine || savedVersion(meta.pine.version) === null) return unavailable();
        if (meta.scriptIdPart !== pineId || savedVersion(meta.pine.version) !== pineVersion) {
            return verificationFailure(phase, 'saved_revision_mismatch', 'validation');
        }
        const userIds = declaredUserInputIds(meta.inputs);
        if (!userIds || !Array.isArray(meta.plots)) return unavailable();
        const plotIds = new Set();
        for (const plot of meta.plots) {
            if (!plot || typeof plot.id !== 'string' || typeof plot.type !== 'string' ||
                plotIds.has(plot.id)) return unavailable();
            plotIds.add(plot.id);
        }
        const condition = meta.plots.find(plot => plot.id === requestedAlertCondId);
        if (!condition || condition.type !== 'alertcondition') {
            return verificationFailure(phase, 'condition_mismatch', 'validation');
        }
        if (requestedConditionTitle !== null) {
            const title = meta.styles && meta.styles[condition.id] && meta.styles[condition.id].title;
            if (typeof title !== 'string') return unavailable();
            if (title !== requestedConditionTitle) {
                return verificationFailure(phase, 'condition_mismatch', 'validation');
            }
        }
        return {ok: true, userIds};
    } catch (_) {
        return unavailable();
    }
}

function nativeStudyIdentity(chart, row) {
    try {
        const study = chart.getStudyById(row.id);
        const native = study && (study._study || study);
        const meta = native && typeof native.metaInfo === 'function' ? native.metaInfo() : null;
        if (!meta || typeof meta !== 'object') return {kind: 'unavailable'};
        const pineStudy = meta.isTVScript === true || meta.isTVScriptStrategy === true ||
            meta.scriptIdPart !== undefined || meta.pine !== undefined ||
            (typeof meta.id === 'string' && /@tv-scripting(?:-|$)/.test(meta.id));
        if (!pineStudy) {
            const otherEngine = typeof meta.id === 'string' && /@tv-(?!scripting(?:-|$))[\w-]+$/.test(meta.id);
            return {kind: otherEngine ? 'other' : 'unavailable'};
        }
        if (typeof meta.scriptIdPart !== 'string' || !meta.scriptIdPart.trim() ||
            !meta.pine || savedVersion(meta.pine.version) === null ||
            typeof study.getInputValues !== 'function') return {kind: 'unavailable'};
        const values = study.getInputValues();
        if (!Array.isArray(values)) return {kind: 'unavailable'};
        const ids = values.filter(input => input && input.id === 'pineId');
        const versions = values.filter(input => input && input.id === 'pineVersion');
        if (ids.length !== 1 || versions.length !== 1) return {kind: 'unavailable'};
        const id = ids[0].value !== undefined ? ids[0].value : ids[0].val;
        const version = versions[0].value !== undefined ? versions[0].value : versions[0].val;
        if (id !== meta.scriptIdPart || savedVersion(version) !== savedVersion(meta.pine.version)) {
            return {kind: 'unavailable'};
        }
        return {kind: 'pine', entityId: row.id, id, version: savedVersion(version), meta, values};
    } catch (_) {
        return {kind: 'unavailable'};
    }
}

function selectVerifiedStudy(chart, entityId) {
    const phase = 'study_identity_verification';
    const unavailable = () => verificationFailure(phase, 'study_metadata_unavailable');
    try {
        if (!chart || typeof chart.getAllStudies !== 'function' || typeof chart.getStudyById !== 'function') {
            return unavailable();
        }
        const rows = chart.getAllStudies();
        if (!Array.isArray(rows)) return unavailable();
        const candidates = entityId === null ? rows : rows.filter(row => row && row.id === entityId);
        if (entityId !== null && candidates.length === 0) {
            return verificationFailure(phase, 'study_not_found', 'validation');
        }
        const seen = new Set();
        const matches = [];
        for (const row of candidates) {
            if (!row || typeof row.id !== 'string' || !row.id.trim() || seen.has(row.id)) return unavailable();
            seen.add(row.id);
            const identity = nativeStudyIdentity(chart, row);
            if (identity.kind === 'unavailable') return unavailable();
            if (identity.kind === 'pine' && identity.id === pineId && identity.version === pineVersion) {
                matches.push(identity);
            } else if (entityId !== null) {
                return verificationFailure(phase, 'saved_revision_mismatch', 'validation');
            }
        }
        if (matches.length === 0) return verificationFailure(phase, 'no_matching_study', 'validation');
        if (matches.length !== 1) {
            return verificationFailure(phase, 'ambiguous_study', 'validation', {match_count: matches.length});
        }
        return {ok: true, study: matches[0]};
    } catch (_) {
        return unavailable();
    }
}

function verifiedStudyInputs(chart, base, entityId, compiledUserIds) {
    let selected = null;
    let inputs = {ok: true, inputs: base, input_count: 0, input_source: 'default_no_inputs', study_matched: false};
    if (compiledUserIds.size !== 0 || entityId !== null) {
        const resolved = selectVerifiedStudy(chart, entityId);
        if (!resolved.ok) return resolved;
        selected = resolved.study;
        const nativeIds = declaredUserInputIds(selected.meta.inputs);
        if (!nativeIds || nativeIds.size !== compiledUserIds.size ||
            [...nativeIds].some(id => !compiledUserIds.has(id))) {
            return verificationFailure('study_input_metadata_unavailable', 'input_declarations_mismatch');
        }
        inputs = nativeStudyInputs(selected.meta.inputs, selected.values, base);
        if (!inputs.ok) {
            return verificationFailure('study_input_metadata_unavailable', 'input_values_unavailable');
        }
        inputs.input_source = 'active_chart_study';
        inputs.study_matched = true;
    }
    try {
        inputs.inputs = structuredClone(inputs.inputs);
    } catch (_) {
        return verificationFailure('study_input_metadata_unavailable', 'input_values_unavailable');
    }
    inputs.verification = {
        condition_source: 'saved_compilation',
        input_source: compiledUserIds.size === 0 ? 'none' : 'active_chart_study',
        input_count: inputs.input_count,
        study: selected ? {entity_id: selected.entityId, selection: entityId === null ? 'unique' : 'explicit'} : null
    };
    return inputs;
}
