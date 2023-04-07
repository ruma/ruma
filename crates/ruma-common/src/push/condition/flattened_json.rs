use serde_json::{to_value as to_json_value, value::Value as JsonValue};
use std::collections::BTreeMap;
use tracing::{instrument, warn};

use crate::serde::Raw;

/// The flattened representation of a JSON object.
#[derive(Clone, Debug)]
pub struct FlattenedJson {
    /// The internal map containing the flattened JSON as a pair path, value.
    map: BTreeMap<String, String>,
}

impl FlattenedJson {
    /// Create a `FlattenedJson` from `Raw`.
    pub fn from_raw<T>(raw: &Raw<T>) -> Self {
        let mut s = Self { map: BTreeMap::new() };
        s.flatten_value(to_json_value(raw).unwrap(), "".into());
        s
    }

    /// Flatten and insert the `value` at `path`.
    #[instrument(skip(self, value))]
    fn flatten_value(&mut self, value: JsonValue, path: String) {
        match value {
            JsonValue::Object(fields) => {
                for (key, value) in fields {
                    let key = escape_key(&key);
                    let path = if path.is_empty() { key } else { format!("{path}.{key}") };
                    self.flatten_value(value, path);
                }
            }
            JsonValue::String(s) => {
                if self.map.insert(path.clone(), s).is_some() {
                    warn!("Duplicate path in flattened JSON: {path}");
                }
            }
            JsonValue::Number(_) | JsonValue::Bool(_) | JsonValue::Array(_) | JsonValue::Null => {}
        }
    }

    /// Value associated with the given `path`.
    pub fn get(&self, path: &str) -> Option<&str> {
        self.map.get(path).map(|s| s.as_str())
    }
}

/// Escape a key for path matching.
///
/// This escapes the dots (`.`) and backslashes (`\`) in the key with a backslash.
fn escape_key(key: &str) -> String {
    key.replace('\\', r"\\").replace('.', r"\.")
}

#[cfg(test)]
mod tests {
    use maplit::btreemap;
    use serde_json::Value as JsonValue;

    use super::FlattenedJson;
    use crate::serde::Raw;

    #[test]
    fn flattened_json_values() {
        let raw = serde_json::from_str::<Raw<JsonValue>>(
            r#"{
                "string": "Hello World",
                "number": 10,
                "array": [1, 2],
                "boolean": true,
                "null": null
            }"#,
        )
        .unwrap();

        let flattened = FlattenedJson::from_raw(&raw);
        assert_eq!(flattened.map, btreemap! { "string".into() => "Hello World".into() });
    }

    #[test]
    fn flattened_json_nested() {
        let raw = serde_json::from_str::<Raw<JsonValue>>(
            r#"{
                "desc": "Level 0",
                "desc.bis": "Level 0 bis",
                "up": {
                    "desc": "Level 1",
                    "desc.bis": "Level 1 bis",
                    "up": {
                        "desc": "Level 2",
                        "desc\\bis": "Level 2 bis"
                    }
                }
            }"#,
        )
        .unwrap();

        let flattened = FlattenedJson::from_raw(&raw);
        assert_eq!(
            flattened.map,
            btreemap! {
                "desc".into() => "Level 0".into(),
                r"desc\.bis".into() => "Level 0 bis".into(),
                "up.desc".into() => "Level 1".into(),
                r"up.desc\.bis".into() => "Level 1 bis".into(),
                "up.up.desc".into() => "Level 2".into(),
                r"up.up.desc\\bis".into() => "Level 2 bis".into(),
            },
        );
    }
}
