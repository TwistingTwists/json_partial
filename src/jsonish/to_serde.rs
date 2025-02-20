use crate::jsonish;
pub fn jsonish_to_serde(value: &jsonish::Value) -> serde_json::Value {
    match value {
        jsonish::Value::String(s) => serde_json::Value::String(s.clone()),
        jsonish::Value::Number(n) => serde_json::Value::Number(n.clone()),
        jsonish::Value::Boolean(b) => serde_json::Value::Bool(*b),
        jsonish::Value::Null => serde_json::Value::Null,
        jsonish::Value::Object(fields) => {
            let mut map = serde_json::Map::new();
            for (k, v) in fields {
                map.insert(k.clone(), jsonish_to_serde(v));
            }
            serde_json::Value::Object(map)
        }
        jsonish::Value::Array(elements) => {
            serde_json::Value::Array(elements.iter().map(jsonish_to_serde).collect())
        }
        jsonish::Value::Markdown(_, inner) => jsonish_to_serde(inner),
        jsonish::Value::FixedJson(inner, _) => jsonish_to_serde(inner),
        jsonish::Value::AnyOf(values, _) => values
            .iter()
            .find_map(|v| match jsonish_to_serde(v) {
                serde_json::Value::Null => None,
                val => Some(val),
            })
            .unwrap_or(serde_json::Value::Null),
    }
}
