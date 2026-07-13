use serde_json::{Map, Value};

const MAX_INPUTS: usize = 32;
const MAX_INPUT_KEY_CHARS: usize = 100;
const MAX_INPUT_STRING_CHARS: usize = 200;
const MAX_INPUT_ARRAY_ITEMS: usize = 16;

pub(crate) fn identity_helper_js() -> &'static str {
    r#"
    function tvStudyValueSafeScalar(value) {
        if (value === null || typeof value === 'boolean') return { ok: true, value: value };
        if (typeof value === 'number' && Number.isFinite(value)) return { ok: true, value: value };
        if (typeof value === 'string' && value.length <= 200) return { ok: true, value: value };
        return { ok: false, value: null };
    }
    function tvStudyValueSafeInput(value) {
        var scalar = tvStudyValueSafeScalar(value);
        if (scalar.ok) return scalar;
        if (!Array.isArray(value) || value.length > 16) return { ok: false, value: null };
        var result = [];
        for (var i = 0; i < value.length; i++) {
            var item = tvStudyValueSafeScalar(value[i]);
            if (!item.ok) return { ok: false, value: null };
            result.push(item.value);
        }
        return { ok: true, value: result };
    }
    function tvStudyValueCompactInputs(source, wrapper) {
        try {
            var raw = null;
            try {
                if (source && typeof source.inputs === 'function') raw = source.inputs();
            } catch (e) {}
            if (!raw) {
                if (wrapper && typeof wrapper.getInputValues === 'function') raw = wrapper.getInputValues();
            }
            if (!raw || typeof raw !== 'object') return null;
            var candidates = [];
            if (Array.isArray(raw)) {
                for (var ai = 0; ai < raw.length; ai++) {
                    var entry = raw[ai];
                    if (entry && typeof entry.id === 'string') candidates.push([entry.id, entry.value]);
                }
            } else {
                var rawKeys = Object.keys(raw);
                for (var oi = 0; oi < rawKeys.length; oi++) candidates.push([rawKeys[oi], raw[rawKeys[oi]]]);
            }
            candidates.sort(function(a, b) { return a[0] < b[0] ? -1 : (a[0] > b[0] ? 1 : 0); });
            var compact = {};
            var count = 0;
            for (var ci = 0; ci < candidates.length && count < 32; ci++) {
                var key = candidates[ci][0].trim();
                var lower = key.toLowerCase();
                if (!key || key.length > 100 || lower === 'text' || lower.indexOf('source') !== -1 || lower.indexOf('script') !== -1) continue;
                var safe = tvStudyValueSafeInput(candidates[ci][1]);
                if (!safe.ok) continue;
                compact[key] = safe.value;
                count++;
            }
            return count > 0 ? compact : null;
        } catch (e) {
            return null;
        }
    }
    function tvStudyValueIdentity(source, wrapper, meta, fallbackId) {
        var result = {
            entity_id: null,
            short_name: null,
            study_kind: 'unknown',
            inputs: null,
            visible: null
        };
        try {
            var candidateId = source && typeof source.id === 'function' ? source.id() : null;
            if (typeof candidateId === 'string' && candidateId.trim()) result.entity_id = candidateId.trim();
        } catch (e) {}
        try {
            if (!result.entity_id && typeof fallbackId === 'string' && fallbackId.trim()) result.entity_id = fallbackId.trim();
        } catch (e) {}

        try {
            var shortCandidate = meta && (meta.shortDescription || meta.shortName);
            if (typeof shortCandidate === 'string' && shortCandidate.trim()) result.short_name = shortCandidate.trim();
        } catch (e) {}

        try {
            if (meta && meta.isStrategy === true) result.study_kind = 'strategy';
            else if (meta && meta.isStrategy === false) result.study_kind = 'indicator';
            else {
                var typeMarker = meta && (meta.studyType || meta.type);
                if (typeof typeMarker === 'string') {
                    var normalizedType = typeMarker.toLowerCase();
                    if (normalizedType.indexOf('strategy') !== -1) result.study_kind = 'strategy';
                    else if (normalizedType.indexOf('study') !== -1 || normalizedType.indexOf('indicator') !== -1) result.study_kind = 'indicator';
                }
            }
        } catch (e) {}

        try {
            if (source && typeof source.isVisible === 'function') {
                var sourceVisible = source.isVisible();
                if (typeof sourceVisible === 'boolean') result.visible = sourceVisible;
            }
        } catch (e) {}
        if (result.visible === null) {
            try {
                if (wrapper && typeof wrapper.isVisible === 'function') {
                    var wrapperVisible = wrapper.isVisible();
                    if (typeof wrapperVisible === 'boolean') result.visible = wrapperVisible;
                }
            } catch (e) {}
        }

        result.inputs = tvStudyValueCompactInputs(source, wrapper);
        return result;
    }
    "#
}

pub(crate) fn normalize_study_value_rows(payload: &mut Value) {
    let Some(studies) = payload.get_mut("studies").and_then(Value::as_array_mut) else {
        return;
    };
    for study in studies {
        let Some(row) = study.as_object_mut() else {
            continue;
        };
        normalize_optional_string(row, "entity_id");
        normalize_optional_string(row, "short_name");
        normalize_study_kind(row);
        normalize_inputs(row);
        if !row.get("visible").is_some_and(Value::is_boolean) {
            row.insert("visible".to_string(), Value::Null);
        }
    }
}

fn normalize_optional_string(row: &mut Map<String, Value>, key: &str) {
    let value = row
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty() && value.chars().count() <= MAX_INPUT_STRING_CHARS)
        .map(|value| Value::String(value.to_string()))
        .unwrap_or(Value::Null);
    row.insert(key.to_string(), value);
}

fn normalize_study_kind(row: &mut Map<String, Value>) {
    let kind = match row.get("study_kind").and_then(Value::as_str) {
        Some("indicator") => "indicator",
        Some("strategy") => "strategy",
        _ => "unknown",
    };
    row.insert("study_kind".to_string(), Value::String(kind.to_string()));
}

fn normalize_inputs(row: &mut Map<String, Value>) {
    let compact = row.get("inputs").and_then(compact_input_object);
    row.insert("inputs".to_string(), compact.unwrap_or(Value::Null));
}

fn compact_input_object(value: &Value) -> Option<Value> {
    let object = value.as_object()?;
    let mut keys = object.keys().collect::<Vec<_>>();
    keys.sort();
    let mut compact = Map::new();
    for key in keys {
        if compact.len() == MAX_INPUTS {
            break;
        }
        let trimmed = key.trim();
        let lower = trimmed.to_ascii_lowercase();
        if trimmed.is_empty()
            || trimmed.chars().count() > MAX_INPUT_KEY_CHARS
            || lower == "text"
            || lower.contains("source")
            || lower.contains("script")
        {
            continue;
        }
        if let Some(safe) = compact_input_value(&object[key]) {
            compact.insert(trimmed.to_string(), safe);
        }
    }
    (!compact.is_empty()).then_some(Value::Object(compact))
}

fn compact_input_value(value: &Value) -> Option<Value> {
    match value {
        Value::Null | Value::Bool(_) => Some(value.clone()),
        Value::Number(number) if number.as_f64().is_some_and(f64::is_finite) => Some(value.clone()),
        Value::String(text) if text.chars().count() <= MAX_INPUT_STRING_CHARS => {
            Some(value.clone())
        }
        Value::Array(items) if items.len() <= MAX_INPUT_ARRAY_ITEMS => items
            .iter()
            .map(|item| match item {
                Value::Null | Value::Bool(_) => Some(item.clone()),
                Value::Number(number) if number.as_f64().is_some_and(f64::is_finite) => {
                    Some(item.clone())
                }
                Value::String(text) if text.chars().count() <= MAX_INPUT_STRING_CHARS => {
                    Some(item.clone())
                }
                _ => None,
            })
            .collect::<Option<Vec<_>>>()
            .map(Value::Array),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use serde_json::json;

    use super::*;

    fn execute_identity_fixture(script_body: &str) -> Value {
        let script = format!("{}\n{}", identity_helper_js(), script_body);
        let output = Command::new("node")
            .args(["-e", &script])
            .output()
            .expect("Node.js is required to execute the JavaScript identity contract fixture");
        assert!(
            output.status.success(),
            "identity fixture failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).expect("identity fixture should return JSON")
    }

    #[test]
    #[ignore = "run through scripts/check-study-values-js-contract.py with pinned Node.js"]
    fn javascript_collector_distinguishes_same_name_instances_and_is_total() {
        let output = execute_identity_fixture(
            r#"
            var firstSource = {
                id: function() { return 'first'; },
                inputs: function() { return { length: 9, text: 'private', pineSource: 'private', nested: { value: 1 } }; },
                isVisible: function() { return false; }
            };
            var secondSource = {
                id: function() { return 'second'; },
                inputs: function() { return { length: 20 }; },
                isVisible: function() { return true; }
            };
            var conflictingWrapper = {
                getInputValues: function() { return [{ id: 'length', value: 999 }]; },
                isVisible: function() { return true; }
            };
            var first = tvStudyValueIdentity(firstSource, conflictingWrapper, { shortDescription: 'EMA', isStrategy: false }, 'wrong-first');
            var second = tvStudyValueIdentity(secondSource, conflictingWrapper, { shortDescription: 'EMA', isStrategy: false }, 'wrong-second');

            var throwingInputs = new Proxy({}, { ownKeys: function() { throw new Error('boom'); } });
            var partial = tvStudyValueIdentity({
                id: function() { return 'partial'; },
                inputs: function() { return throwingInputs; },
                isVisible: function() { return true; }
            }, null, { shortDescription: 'Safe', isStrategy: false }, null);

            var hostile = new Proxy({}, { get: function() { throw new Error('boom'); } });
            var conservative = tvStudyValueIdentity(hostile, hostile, hostile, null);
            process.stdout.write(JSON.stringify({ first: first, second: second, partial: partial, conservative: conservative }));
            "#,
        );

        assert_eq!(output["first"]["entity_id"], "first");
        assert_eq!(output["first"]["inputs"], json!({"length": 9}));
        assert_eq!(output["first"]["visible"], false);
        assert_eq!(output["second"]["entity_id"], "second");
        assert_eq!(output["second"]["inputs"], json!({"length": 20}));
        assert_eq!(output["partial"]["entity_id"], "partial");
        assert!(output["partial"]["inputs"].is_null());
        assert_eq!(output["partial"]["visible"], true);
        assert_eq!(
            output["conservative"],
            json!({
                "entity_id": null,
                "short_name": null,
                "study_kind": "unknown",
                "inputs": null,
                "visible": null
            })
        );
    }

    #[test]
    fn same_name_rows_keep_order_and_gain_distinct_identity() {
        let mut payload = json!({
            "study_count": 2,
            "studies": [
                {"name": "EMA", "values": {"MA": "1"}, "entity_id": "first", "short_name": "EMA", "study_kind": "indicator", "inputs": {"length": 9}, "visible": true},
                {"name": "EMA", "values": {"MA": "2"}, "entity_id": "second", "short_name": "EMA", "study_kind": "indicator", "inputs": {"length": 20}, "visible": true}
            ]
        });

        normalize_study_value_rows(&mut payload);

        assert_eq!(payload["study_count"], 2);
        assert_eq!(payload["studies"][0]["name"], "EMA");
        assert_eq!(payload["studies"][0]["values"], json!({"MA": "1"}));
        assert_eq!(payload["studies"][0]["entity_id"], "first");
        assert_eq!(payload["studies"][0]["inputs"]["length"], 9);
        assert_eq!(payload["studies"][1]["entity_id"], "second");
        assert_eq!(payload["studies"][1]["inputs"]["length"], 20);
    }

    #[test]
    fn malformed_identity_degrades_and_unsafe_inputs_are_omitted() {
        let mut inputs = Map::new();
        inputs.insert("length".to_string(), json!(20));
        inputs.insert("text".to_string(), json!("hidden"));
        inputs.insert("pineSource".to_string(), json!("hidden"));
        inputs.insert("nested".to_string(), json!({"value": 1}));
        inputs.insert("tooLong".to_string(), json!("x".repeat(201)));
        inputs.insert("array".to_string(), json!([1, true, "ok", null]));
        inputs.insert("nestedArray".to_string(), json!([[1]]));
        inputs.insert("longArray".to_string(), json!(vec![1; 17]));
        inputs.insert("x".repeat(101), json!(1));
        let mut payload = json!({"studies": [{
            "name": "Study",
            "values": {"value": 1},
            "entity_id": 7,
            "short_name": "   ",
            "study_kind": "other",
            "inputs": Value::Object(inputs),
            "visible": "yes"
        }]});

        normalize_study_value_rows(&mut payload);

        let row = &payload["studies"][0];
        assert!(row["entity_id"].is_null());
        assert!(row["short_name"].is_null());
        assert_eq!(row["study_kind"], "unknown");
        assert!(row["visible"].is_null());
        let compact = row["inputs"].as_object().unwrap();
        assert_eq!(compact.len(), 2);
        assert_eq!(compact["array"], json!([1, true, "ok", null]));
        assert_eq!(compact["length"], 20);
        assert!(!compact.contains_key("text"));
        assert!(!compact.contains_key("pineSource"));
        assert!(!compact.contains_key("nested"));
        assert!(!compact.contains_key("tooLong"));
        assert!(!compact.contains_key("nestedArray"));
        assert!(!compact.contains_key("longArray"));
    }

    #[test]
    fn compact_inputs_are_sorted_and_limited_to_thirty_two_entries() {
        let mut inputs = Map::new();
        for index in (0..40).rev() {
            inputs.insert(format!("key{index:02}"), json!(index));
        }
        let mut payload = json!({"studies": [{
            "name": "Study",
            "values": {"value": 1},
            "inputs": Value::Object(inputs)
        }]});

        normalize_study_value_rows(&mut payload);

        let compact = payload["studies"][0]["inputs"].as_object().unwrap();
        assert_eq!(compact.len(), 32);
        assert_eq!(compact.keys().next().map(String::as_str), Some("key00"));
        assert_eq!(
            compact.keys().next_back().map(String::as_str),
            Some("key31")
        );
    }

    #[test]
    fn missing_identity_fields_are_added_as_conservative_defaults() {
        let mut payload = json!({"studies": [{"name": "Study", "values": {"v": 1}}]});

        normalize_study_value_rows(&mut payload);

        assert_eq!(payload["studies"][0]["study_kind"], "unknown");
        for key in ["entity_id", "short_name", "inputs", "visible"] {
            assert!(payload["studies"][0][key].is_null());
        }
    }
}
