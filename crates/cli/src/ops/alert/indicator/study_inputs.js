const systemInputIds = new Set(['text', 'pineId', 'pineVersion', 'pineFeatures', '__fast_calc', '__profile']);

function declaredUserInputIds(declared) {
    if (!Array.isArray(declared)) return null;
    const declaredIds = new Set();
    const userIds = new Set();
    for (const input of declared) {
        if (!input || typeof input.id !== 'string' || declaredIds.has(input.id)) return null;
        declaredIds.add(input.id);
        if (/^in_\d+$/.test(input.id)) userIds.add(input.id);
        else if (!systemInputIds.has(input.id)) return null;
    }
    return userIds;
}

function nativeStudyInputs(declared, values, base) {
    const unavailable = {
        ok: false,
        error: 'Active study inputs could not be verified against declared input IDs'
    };
    const userIds = declaredUserInputIds(declared);
    if (!userIds || !Array.isArray(values)) return unavailable;
    const inputs = Object.assign({}, base);
    const returnedIds = new Set();
    let inputCount = 0;
    for (const input of values) {
        if (!input || typeof input.id !== 'string' || returnedIds.has(input.id)) return unavailable;
        returnedIds.add(input.id);
        if (systemInputIds.has(input.id)) continue;
        if (!userIds.has(input.id)) return unavailable;
        const value = input.value !== undefined ? input.value : input.val;
        if (value === undefined) return unavailable;
        inputs[input.id] = value;
        inputCount++;
    }
    if (inputCount !== userIds.size) return unavailable;
    return {ok: true, inputs, input_count: inputCount};
}
