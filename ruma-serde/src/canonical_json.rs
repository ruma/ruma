use std::{collections::BTreeMap, convert::TryInto, fmt};

use serde::{ser::Error as _, Serialize};
use serde_json::{Error as JsonError, Map as JsonObject, Value as JsonValue};

pub mod value;

pub type CanonicalJsonObject = BTreeMap<String, value::CanonicalJsonValue>;

/// Returns a canonical JSON string according to Matrix specification.
///
/// This function should be preferred over `serde_json::to_string` since it checks the size of the
/// canonical string. Matrix canonical JSON enforces a size limit of less than 65,535 when sending
/// PDU's for the server-server protocol.
pub fn to_string<T: Serialize>(val: &T) -> Result<String, Error> {
    let s = serde_json::to_string(val).map_err(Error::SerDe)?;

    if s.as_bytes().len() > 65_535 {
        Err(Error::JsonSize)
    } else {
        Ok(s)
    }
}

/// The set of possible errors when serializing to canonical JSON.
#[derive(Debug)]
pub enum Error {
    /// The numeric value failed conversion to js_int::Int.
    IntConvert,

    /// The `CanonicalJsonValue` being serialized was larger than 65,535 bytes.
    JsonSize,

    /// An error occurred while serializing/deserializing.
    SerDe(JsonError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IntConvert => f.write_str("number found is not a valid `js_int::Int`"),
            Error::JsonSize => f.write_str("JSON is larger than 65,535 byte max"),
            Error::SerDe(err) => write!(f, "serde Error: {}", err),
        }
    }
}

impl std::error::Error for Error {}

pub fn try_from_json_map(
    json: JsonObject<String, JsonValue>,
) -> Result<CanonicalJsonObject, Error> {
    json.into_iter().map(|(k, v)| Ok((k, v.try_into()?))).collect()
}

pub fn to_canonical_obj<T: Serialize>(value: T) -> Result<CanonicalJsonObject, Error> {
    match serde_json::to_value(value).map_err(Error::SerDe)? {
        JsonValue::Object(map) => try_from_json_map(map),
        _ => Err(Error::SerDe(JsonError::custom("`value` must be an object"))),
    }
}

#[cfg(test)]
mod test {
    use std::{collections::BTreeMap, convert::TryInto};

    use super::{
        to_canonical_obj, to_string as to_canonical_json_string, value::CanonicalJsonValue,
    };
    use serde_json::{from_str as from_json_str, json, to_string as to_json_string};

    #[test]
    fn serialize_canon() {
        let json: CanonicalJsonValue = json!({
            "a": [1, 2, 3],
            "other": { "stuff": "hello" },
            "string": "Thing"
        })
        .try_into()
        .unwrap();

        let ser = to_canonical_json_string(&json).unwrap();
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
        )
    }

    #[test]
    fn check_serialize_to_canonical() {
        #[derive(Debug, serde::Serialize)]
        struct Thing {
            foo: String,
            bar: Vec<u8>,
        }
        let t = Thing { foo: "foo".into(), bar: vec![0, 1, 2] };

        let mut expected = BTreeMap::new();
        expected.insert("foo".into(), CanonicalJsonValue::String("foo".into()));
        expected.insert(
            "bar".into(),
            CanonicalJsonValue::Array(vec![
                CanonicalJsonValue::Integer(js_int::int!(0)),
                CanonicalJsonValue::Integer(js_int::int!(1)),
                CanonicalJsonValue::Integer(js_int::int!(2)),
            ]),
        );

        assert_eq!(to_canonical_obj(t).unwrap(), expected)
    }
}
