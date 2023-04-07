use js_int::Int;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{to_value as to_json_value, value::Value as JsonValue};
use std::collections::BTreeMap;
use thiserror::Error;
use tracing::{instrument, warn};

use crate::serde::Raw;

/// The flattened representation of a JSON object.
#[derive(Clone, Debug)]
pub struct FlattenedJson {
    /// The internal map containing the flattened JSON as a pair path, value.
    map: BTreeMap<String, FlattenedJsonValue>,
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
            value => {
                if let Some(v) = FlattenedJsonValue::from_json_value(value) {
                    if self.map.insert(path.clone(), v).is_some() {
                        warn!("Duplicate path in flattened JSON: {path}");
                    }
                }
            }
        }
    }

    /// Get the value associated with the given `path`.
    pub fn get(&self, path: &str) -> Option<&FlattenedJsonValue> {
        self.map.get(path)
    }

    /// Get the value associated with the given `path`, if it is a string.
    pub fn get_str(&self, path: &str) -> Option<&str> {
        self.map.get(path).and_then(|v| v.as_str())
    }
}

/// Escape a key for path matching.
///
/// This escapes the dots (`.`) and backslashes (`\`) in the key with a backslash.
fn escape_key(key: &str) -> String {
    key.replace('\\', r"\\").replace('.', r"\.")
}

/// The set of possible errors when converting to a JSON subset.
#[derive(Debug, Error)]
#[allow(clippy::exhaustive_enums)]
enum IntoJsonSubsetError {
    /// The numeric value failed conversion to js_int::Int.
    #[error("number found is not a valid `js_int::Int`")]
    IntConvert,

    /// The JSON type is not accepted in this subset.
    #[error("JSON type is not accepted in this subset")]
    NotInSubset,
}

/// Scalar (non-compound) JSON values.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
#[allow(clippy::exhaustive_enums)]
pub enum ScalarJsonValue {
    /// Represents a `null` value.
    #[default]
    Null,

    /// Represents a boolean.
    Bool(bool),

    /// Represents an integer.
    Integer(Int),

    /// Represents a string.
    String(String),
}

impl ScalarJsonValue {
    fn try_from_json_value(val: JsonValue) -> Result<Self, IntoJsonSubsetError> {
        Ok(match val {
            JsonValue::Bool(b) => Self::Bool(b),
            JsonValue::Number(num) => Self::Integer(
                Int::try_from(num.as_i64().ok_or(IntoJsonSubsetError::IntConvert)?)
                    .map_err(|_| IntoJsonSubsetError::IntConvert)?,
            ),
            JsonValue::String(string) => Self::String(string),
            JsonValue::Null => Self::Null,
            _ => Err(IntoJsonSubsetError::NotInSubset)?,
        })
    }

    /// If the `ScalarJsonValue` is a `Bool`, return the inner value.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// If the `ScalarJsonValue` is an `Integer`, return the inner value.
    pub fn as_integer(&self) -> Option<Int> {
        match self {
            Self::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// If the `ScalarJsonValue` is a `String`, return a reference to the inner value.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }
}

impl Serialize for ScalarJsonValue {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Null => serializer.serialize_unit(),
            Self::Bool(b) => serializer.serialize_bool(*b),
            Self::Integer(n) => n.serialize(serializer),
            Self::String(s) => serializer.serialize_str(s),
        }
    }
}

impl<'de> Deserialize<'de> for ScalarJsonValue {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = JsonValue::deserialize(deserializer)?;
        ScalarJsonValue::try_from_json_value(val).map_err(serde::de::Error::custom)
    }
}

impl From<bool> for ScalarJsonValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<Int> for ScalarJsonValue {
    fn from(value: Int) -> Self {
        Self::Integer(value)
    }
}

impl From<String> for ScalarJsonValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for ScalarJsonValue {
    fn from(value: &str) -> Self {
        value.to_owned().into()
    }
}

impl PartialEq<FlattenedJsonValue> for ScalarJsonValue {
    fn eq(&self, other: &FlattenedJsonValue) -> bool {
        match self {
            Self::Null => *other == FlattenedJsonValue::Null,
            Self::Bool(b) => other.as_bool() == Some(*b),
            Self::Integer(i) => other.as_integer() == Some(*i),
            Self::String(s) => other.as_str() == Some(s),
        }
    }
}

/// Possible JSON values after an object is flattened.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
#[allow(clippy::exhaustive_enums)]
pub enum FlattenedJsonValue {
    /// Represents a `null` value.
    #[default]
    Null,

    /// Represents a boolean.
    Bool(bool),

    /// Represents an integer.
    Integer(Int),

    /// Represents a string.
    String(String),

    /// Represents an array.
    Array(Vec<ScalarJsonValue>),
}

impl FlattenedJsonValue {
    fn from_json_value(val: JsonValue) -> Option<Self> {
        Some(match val {
            JsonValue::Bool(b) => Self::Bool(b),
            JsonValue::Number(num) => Self::Integer(Int::try_from(num.as_i64()?).ok()?),
            JsonValue::String(string) => Self::String(string),
            JsonValue::Null => Self::Null,
            JsonValue::Array(vec) => Self::Array(
                // Drop values we don't need instead of throwing an error.
                vec.into_iter()
                    .filter_map(|v| ScalarJsonValue::try_from_json_value(v).ok())
                    .collect::<Vec<_>>(),
            ),
            _ => None?,
        })
    }

    /// If the `FlattenedJsonValue` is a `Bool`, return the inner value.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// If the `FlattenedJsonValue` is an `Integer`, return the inner value.
    pub fn as_integer(&self) -> Option<Int> {
        match self {
            Self::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// If the `FlattenedJsonValue` is a `String`, return a reference to the inner value.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// If the `FlattenedJsonValue` is an `Array`, return a reference to the inner value.
    pub fn as_array(&self) -> Option<&[ScalarJsonValue]> {
        match self {
            Self::Array(a) => Some(a),
            _ => None,
        }
    }
}

impl From<bool> for FlattenedJsonValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<Int> for FlattenedJsonValue {
    fn from(value: Int) -> Self {
        Self::Integer(value)
    }
}

impl From<String> for FlattenedJsonValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for FlattenedJsonValue {
    fn from(value: &str) -> Self {
        value.to_owned().into()
    }
}

impl From<Vec<ScalarJsonValue>> for FlattenedJsonValue {
    fn from(value: Vec<ScalarJsonValue>) -> Self {
        Self::Array(value)
    }
}

impl PartialEq<ScalarJsonValue> for FlattenedJsonValue {
    fn eq(&self, other: &ScalarJsonValue) -> bool {
        match self {
            Self::Null => *other == ScalarJsonValue::Null,
            Self::Bool(b) => other.as_bool() == Some(*b),
            Self::Integer(i) => other.as_integer() == Some(*i),
            Self::String(s) => other.as_str() == Some(s),
            Self::Array(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use js_int::int;
    use maplit::btreemap;
    use serde_json::Value as JsonValue;

    use super::{FlattenedJson, FlattenedJsonValue};
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
        assert_eq!(
            flattened.map,
            btreemap! {
                "string".into() => "Hello World".into(),
                "number".into() => int!(10).into(),
                "array".into() => vec![int!(1).into(), int!(2).into()].into(),
                "boolean".into() => true.into(),
                "null".into() => FlattenedJsonValue::Null,
            }
        );
    }

    #[test]
    fn flattened_json_nested() {
        let raw = serde_json::from_str::<Raw<JsonValue>>(
            r#"{
                "desc": "Level 0",
                "desc.bis": "Level 0 bis",
                "up": {
                    "desc": 1,
                    "desc.bis": null,
                    "up": {
                        "desc": ["Level 2a", "Level 2b"],
                        "desc\\bis": true
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
                "up.desc".into() => int!(1).into(),
                r"up.desc\.bis".into() => FlattenedJsonValue::Null,
                "up.up.desc".into() => vec!["Level 2a".into(), "Level 2b".into()].into(),
                r"up.up.desc\\bis".into() => true.into(),
            },
        );
    }
}
