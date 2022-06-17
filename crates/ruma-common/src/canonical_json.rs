//! Canonical JSON types and related functions.

use std::fmt;

use serde::Serialize;
use serde_json::{Error as JsonError, Value as JsonValue};

mod value;

pub use self::value::{CanonicalJsonObject, CanonicalJsonValue};

/// The set of possible errors when serializing to canonical JSON.
#[cfg(feature = "canonical-json")]
#[derive(Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum CanonicalJsonError {
    /// The numeric value failed conversion to js_int::Int.
    IntConvert,

    /// An error occurred while serializing/deserializing.
    SerDe(JsonError),
}

impl fmt::Display for CanonicalJsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CanonicalJsonError::IntConvert => {
                f.write_str("number found is not a valid `js_int::Int`")
            }
            CanonicalJsonError::SerDe(err) => write!(f, "serde Error: {err}"),
        }
    }
}

impl std::error::Error for CanonicalJsonError {}

/// Fallible conversion from a `serde_json::Map` to a `CanonicalJsonObject`.
pub fn try_from_json_map(
    json: serde_json::Map<String, JsonValue>,
) -> Result<CanonicalJsonObject, CanonicalJsonError> {
    json.into_iter().map(|(k, v)| Ok((k, v.try_into()?))).collect()
}

/// Fallible conversion from any value that impl's `Serialize` to a `CanonicalJsonValue`.
pub fn to_canonical_value<T: Serialize>(
    value: T,
) -> Result<value::CanonicalJsonValue, CanonicalJsonError> {
    serde_json::to_value(value).map_err(CanonicalJsonError::SerDe)?.try_into()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use js_int::int;
    use serde_json::{from_str as from_json_str, json, to_string as to_json_string};

    use super::{to_canonical_value, try_from_json_map, value::CanonicalJsonValue};

    #[test]
    fn serialize_canon() {
        let json: CanonicalJsonValue = json!({
            "a": [1, 2, 3],
            "other": { "stuff": "hello" },
            "string": "Thing"
        })
        .try_into()
        .unwrap();

        let ser = to_json_string(&json).unwrap();
        let back = from_json_str::<CanonicalJsonValue>(&ser).unwrap();

        assert_eq!(json, back);
    }

    #[test]
    fn check_canonical_sorts_keys() {
        let json: CanonicalJsonValue = json!({
            "auth": {
                "success": true,
                "mxid": "@john.doe:example.com",
                "profile": {
                    "display_name": "John Doe",
                    "three_pids": [
                        {
                            "medium": "email",
                            "address": "john.doe@example.org"
                        },
                        {
                            "medium": "msisdn",
                            "address": "123456789"
                        }
                    ]
                }
            }
        })
        .try_into()
        .unwrap();

        assert_eq!(
            to_json_string(&json).unwrap(),
            r#"{"auth":{"mxid":"@john.doe:example.com","profile":{"display_name":"John Doe","three_pids":[{"address":"john.doe@example.org","medium":"email"},{"address":"123456789","medium":"msisdn"}]},"success":true}}"#
        );
    }

    #[test]
    fn serialize_map_to_canonical() {
        let mut expected = BTreeMap::new();
        expected.insert("foo".into(), CanonicalJsonValue::String("string".into()));
        expected.insert(
            "bar".into(),
            CanonicalJsonValue::Array(vec![
                CanonicalJsonValue::Integer(int!(0)),
                CanonicalJsonValue::Integer(int!(1)),
                CanonicalJsonValue::Integer(int!(2)),
            ]),
        );

        let mut map = serde_json::Map::new();
        map.insert("foo".into(), json!("string"));
        map.insert("bar".into(), json!(vec![0, 1, 2,]));

        assert_eq!(try_from_json_map(map).unwrap(), expected);
    }

    #[test]
    fn to_canonical() {
        #[derive(Debug, serde::Serialize)]
        struct Thing {
            foo: String,
            bar: Vec<u8>,
        }
        let t = Thing { foo: "string".into(), bar: vec![0, 1, 2] };

        let mut expected = BTreeMap::new();
        expected.insert("foo".into(), CanonicalJsonValue::String("string".into()));
        expected.insert(
            "bar".into(),
            CanonicalJsonValue::Array(vec![
                CanonicalJsonValue::Integer(int!(0)),
                CanonicalJsonValue::Integer(int!(1)),
                CanonicalJsonValue::Integer(int!(2)),
            ]),
        );

        assert_eq!(to_canonical_value(t).unwrap(), CanonicalJsonValue::Object(expected));
    }
}
