use serde_json::{Map, Value};

pub(super) fn field_values_object(fields: &[String], values: &[Value]) -> Map<String, Value> {
    fields
        .iter()
        .zip(values)
        .map(|(field, value)| (field.clone(), value.clone()))
        .collect()
}
