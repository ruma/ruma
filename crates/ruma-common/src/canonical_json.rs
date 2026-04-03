//! Canonical JSON types and related functions.

use std::fmt;

use serde::Serialize;
use serde_json::Value as JsonValue;

mod macros;
mod redaction;
mod serializer;
mod value;

pub use self::{
    redaction::{
        RedactedBecause, RedactingSerializer, RedactionEvent, redact, redact_content_in_place,
        redact_in_place,
    },
    serializer::Serializer,
    value::{CanonicalJsonObject, CanonicalJsonType, CanonicalJsonValue},
};
#[doc(inline)]
pub use crate::assert_to_canonical_json_eq;

/// Fallible conversion from any value that implements [`Serialize`] to a [`CanonicalJsonValue`].
///
/// This behaves similarly to [`serde_json::to_value()`], except for the following restrictions
/// which return errors:
///
/// - Integers must be in the range accepted by [`js_int::Int`].
/// - Floats and bytes are not serializable.
/// - Booleans and integers cannot be used as keys for an object. `serde_json` accepts those types
///   as keys by serializing them as strings.
/// - The same key cannot be serialized twice in an object. `serde_json` uses the last value that is
///   serialized for the same key.
pub fn to_canonical_value<T: Serialize>(
    value: T,
) -> Result<CanonicalJsonValue, CanonicalJsonError> {
    value.serialize(Serializer)
}

/// Fallible conversion from a `serde_json::Map` to a `CanonicalJsonObject`.
pub fn try_from_json_map(
    json: serde_json::Map<String, JsonValue>,
) -> Result<CanonicalJsonObject, CanonicalJsonError> {
    json.into_iter().map(|(k, v)| Ok((k, v.try_into()?))).collect()
}

/// The set of possible errors when serializing to canonical JSON.
#[derive(Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum CanonicalJsonError {
    /// The integer value is out of the range of [`js_int::Int`].
    IntegerOutOfRange,

    /// The given type cannot be serialized to canonical JSON.
    InvalidType(String),

    /// The given type cannot be serialized to an object key.
    InvalidObjectKeyType(String),

    /// The same object key was serialized twice.
    DuplicateObjectKey(String),

    /// An error occurred while re-serializing a [`serde_json::value::RawValue`].
    InvalidRawValue(serde_json::Error),

    /// An other error happened.
    Other(String),
}

impl fmt::Display for CanonicalJsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IntegerOutOfRange => f.write_str("integer is out of the range of `js_int::Int`"),
            Self::InvalidType(ty) => write!(f, "{ty} cannot be serialized as canonical JSON"),
            Self::InvalidObjectKeyType(ty) => {
                write!(f, "{ty} cannot be used as an object key, expected a string type")
            }
            Self::InvalidRawValue(error) => {
                write!(f, "invalid raw value: {error}")
            }
            Self::DuplicateObjectKey(key) => write!(f, "duplicate object key `{key}`"),
            Self::Other(msg) => f.write_str(msg),
        }
    }
}

impl std::error::Error for CanonicalJsonError {}

impl serde::ser::Error for CanonicalJsonError {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self::Other(msg.to_string())
    }
}

/// The possible types of a JSON value.
#[derive(Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum JsonType {
    /// A JSON Object.
    Object,

    /// A JSON String.
    String,

    /// A JSON Integer.
    Integer,

    /// A JSON Array.
    Array,

    /// A JSON Boolean.
    Boolean,

    /// JSON Null.
    Null,
}

/// Helper trait to interact with a [`CanonicalJsonObject`].
pub trait CanonicalJsonObjectExt {
    /// Get the given field as an object.
    ///
    /// # Parameters
    ///
    /// * `field`: The name of the field to access.
    /// * `path`: The full path of the field that will be used in errors. This can be different than
    ///   the `field`, to clarify if this is a field nested under several objects.
    ///
    /// # Errors
    ///
    /// Returns an error if the field is invalid.
    fn get_as_object(
        &self,
        field: &str,
        path: impl Into<String>,
    ) -> Result<Option<&CanonicalJsonObject>, CanonicalJsonFieldError>;

    /// Get the given required field as an object.
    ///
    /// # Parameters
    ///
    /// * `field`: The name of the field to access.
    /// * `path`: The full path of the field that will be used in errors. This can be different than
    ///   the `field`, to clarify if this is a field nested under several objects.
    ///
    /// # Errors
    ///
    /// Returns an error if the field is missing or invalid.
    fn get_as_required_object(
        &self,
        field: &str,
        path: impl Into<String>,
    ) -> Result<&CanonicalJsonObject, CanonicalJsonFieldError> {
        let path = path.into();
        self.get_as_object(field, &path)?.ok_or(CanonicalJsonFieldError::Missing { path })
    }

    /// Get the given field as a mutable object.
    ///
    /// # Parameters
    ///
    /// * `field`: The name of the field to access.
    /// * `path`: The full path of the field that will be used in errors. This can be different than
    ///   the `field`, to clarify if this is a field nested under several objects.
    ///
    /// # Errors
    ///
    /// Returns an error if the field is invalid.
    fn get_as_object_mut(
        &mut self,
        field: &str,
        path: impl Into<String>,
    ) -> Result<Option<&mut CanonicalJsonObject>, CanonicalJsonFieldError>;

    /// Get the given required field as a mutable object.
    ///
    /// # Parameters
    ///
    /// * `field`: The name of the field to access.
    /// * `path`: The full path of the field that will be used in errors. This can be different than
    ///   the `field`, to clarify if this is a field nested under several objects.
    ///
    /// # Errors
    ///
    /// Returns an error if the field is missing or invalid.
    fn get_as_required_object_mut(
        &mut self,
        field: &str,
        path: impl Into<String>,
    ) -> Result<&mut CanonicalJsonObject, CanonicalJsonFieldError> {
        let path = path.into();
        self.get_as_object_mut(field, &path)?.ok_or(CanonicalJsonFieldError::Missing { path })
    }

    /// Get the given required field as a mutable object or insert it if it is missing.
    ///
    /// # Parameters
    ///
    /// * `field`: The name of the field to access.
    /// * `path`: The full path of the field that will be used in errors. This can be different than
    ///   the `field`, to clarify if this is a field nested under several objects.
    ///
    /// # Errors
    ///
    /// Returns an error if the field is already be present but invalid.
    fn get_as_object_or_insert_default(
        &mut self,
        field: impl Into<String>,
        path: impl Into<String>,
    ) -> Result<&mut CanonicalJsonObject, CanonicalJsonFieldError>;

    /// Get the given field as a string.
    ///
    /// # Parameters
    ///
    /// * `field`: The name of the field to access.
    /// * `path`: The full path of the field that will be used in errors. This can be different than
    ///   the `field`, to clarify if this is a field nested under several objects.
    ///
    /// # Errors
    ///
    /// Returns an error if the field is invalid.
    fn get_as_string(
        &self,
        field: &str,
        path: impl Into<String>,
    ) -> Result<Option<&str>, CanonicalJsonFieldError>;

    /// Get the given required field as a string.
    ///
    /// # Parameters
    ///
    /// * `field`: The name of the field to access.
    /// * `path`: The full path of the field that will be used in errors. This can be different than
    ///   the `field`, to clarify if this is a field nested under several objects.
    ///
    /// # Errors
    ///
    /// Returns an error if the field is missing or invalid.
    fn get_as_required_string(
        &self,
        field: &str,
        path: impl Into<String>,
    ) -> Result<&str, CanonicalJsonFieldError> {
        let path = path.into();
        self.get_as_string(field, &path)?.ok_or(CanonicalJsonFieldError::Missing { path })
    }
}

impl CanonicalJsonObjectExt for CanonicalJsonObject {
    fn get_as_object(
        &self,
        field: &str,
        path: impl Into<String>,
    ) -> Result<Option<&CanonicalJsonObject>, CanonicalJsonFieldError> {
        match self.get(field) {
            Some(CanonicalJsonValue::Object(object)) => Ok(Some(object)),
            Some(value) => Err(CanonicalJsonFieldError::InvalidType {
                path: path.into(),
                expected: CanonicalJsonType::Object,
                found: value.json_type(),
            }),
            None => Ok(None),
        }
    }

    fn get_as_object_mut(
        &mut self,
        field: &str,
        path: impl Into<String>,
    ) -> Result<Option<&mut CanonicalJsonObject>, CanonicalJsonFieldError> {
        match self.get_mut(field) {
            Some(CanonicalJsonValue::Object(object)) => Ok(Some(object)),
            Some(value) => Err(CanonicalJsonFieldError::InvalidType {
                path: path.into(),
                expected: CanonicalJsonType::Object,
                found: value.json_type(),
            }),
            None => Ok(None),
        }
    }

    fn get_as_object_or_insert_default(
        &mut self,
        field: impl Into<String>,
        path: impl Into<String>,
    ) -> Result<&mut CanonicalJsonObject, CanonicalJsonFieldError> {
        let value = self
            .entry(field.into())
            .or_insert_with(|| CanonicalJsonValue::Object(Default::default()));
        match value {
            CanonicalJsonValue::Object(object) => Ok(object),
            value => Err(CanonicalJsonFieldError::InvalidType {
                path: path.into(),
                expected: CanonicalJsonType::String,
                found: value.json_type(),
            }),
        }
    }

    fn get_as_string(
        &self,
        field: &str,
        path: impl Into<String>,
    ) -> Result<Option<&str>, CanonicalJsonFieldError> {
        match self.get(field) {
            Some(CanonicalJsonValue::String(string)) => Ok(Some(string)),
            Some(value) => Err(CanonicalJsonFieldError::InvalidType {
                path: path.into(),
                expected: CanonicalJsonType::String,
                found: value.json_type(),
            }),
            None => Ok(None),
        }
    }
}

/// Errors that can happen when trying to access a field from a [`CanonicalJsonObject`].
#[derive(Debug)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum CanonicalJsonFieldError {
    /// The field at `path` was expected to be of type `expected`, but was received as `found`.
    InvalidType {
        /// The path of the invalid field.
        path: String,

        /// The type that was expected.
        expected: CanonicalJsonType,

        /// The type that was found.
        found: CanonicalJsonType,
    },

    /// A required field is missing from a JSON object.
    Missing {
        /// The path of the missing field.
        path: String,
    },
}

impl fmt::Display for CanonicalJsonFieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidType { path, expected, found } => {
                write!(f, "invalid type at `{path}`: expected {expected:?}, found {found:?}")
            }
            Self::Missing { path } => {
                write!(f, "missing field: `{path}`")
            }
        }
    }
}

impl std::error::Error for CanonicalJsonFieldError {}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use assert_matches2::assert_matches;
    use js_int::int;
    use serde_json::{
        from_str as from_json_str, json, to_string as to_json_string,
        value::RawValue as RawJsonValue,
    };

    use super::{
        CanonicalJsonError, assert_to_canonical_json_eq, to_canonical_value, try_from_json_map,
        value::CanonicalJsonValue,
    };

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
    fn to_canonical_value_success() {
        #[derive(Debug, serde::Serialize)]
        struct MyStruct {
            string: String,
            array: Vec<u8>,
            boolean: Option<bool>,
            object: BTreeMap<String, MyEnum>,
            null: (),
            raw: Box<RawJsonValue>,
        }

        #[derive(Debug, serde::Serialize)]
        enum MyEnum {
            Foo,
            #[serde(rename = "bar")]
            Bar,
        }

        let t = MyStruct {
            string: "string".into(),
            array: vec![0, 1, 2],
            boolean: Some(true),
            object: [("foo".to_owned(), MyEnum::Foo), ("bar".to_owned(), MyEnum::Bar)].into(),
            null: (),
            raw: RawJsonValue::from_string(r#"{"baz":false}"#.to_owned()).unwrap(),
        };

        let mut expected = BTreeMap::new();
        expected.insert("string".to_owned(), CanonicalJsonValue::String("string".to_owned()));
        expected.insert(
            "array".to_owned(),
            CanonicalJsonValue::Array(vec![
                CanonicalJsonValue::Integer(int!(0)),
                CanonicalJsonValue::Integer(int!(1)),
                CanonicalJsonValue::Integer(int!(2)),
            ]),
        );
        expected.insert("boolean".to_owned(), CanonicalJsonValue::Bool(true));
        let mut child_object = BTreeMap::new();
        child_object.insert("foo".to_owned(), CanonicalJsonValue::String("Foo".to_owned()));
        child_object.insert("bar".to_owned(), CanonicalJsonValue::String("bar".to_owned()));
        expected.insert("object".to_owned(), CanonicalJsonValue::Object(child_object));
        expected.insert("null".to_owned(), CanonicalJsonValue::Null);
        let mut raw_object = BTreeMap::new();
        raw_object.insert("baz".to_owned(), CanonicalJsonValue::Bool(false));
        expected.insert("raw".to_owned(), CanonicalJsonValue::Object(raw_object));

        let expected = CanonicalJsonValue::Object(expected);
        assert_eq!(to_canonical_value(&t).unwrap(), expected);
        assert_to_canonical_json_eq!(t, expected.into());
    }

    #[test]
    fn to_canonical_value_out_of_range_int() {
        #[derive(Debug, serde::Serialize)]
        struct StructWithInt {
            foo: i64,
        }

        let t = StructWithInt { foo: i64::MAX };
        assert_matches!(to_canonical_value(t), Err(CanonicalJsonError::IntegerOutOfRange));
    }

    #[test]
    fn to_canonical_value_invalid_type() {
        #[derive(Debug, serde::Serialize)]
        struct StructWithFloat {
            foo: f32,
        }

        let t = StructWithFloat { foo: 10.0 };
        assert_matches!(to_canonical_value(t), Err(CanonicalJsonError::InvalidType(_)));
    }

    #[test]
    fn to_canonical_value_invalid_object_key_type() {
        {
            #[derive(Debug, serde::Serialize)]
            struct StructWithBoolKey {
                foo: BTreeMap<bool, String>,
            }

            let t = StructWithBoolKey { foo: [(true, "bar".to_owned())].into() };
            assert_matches!(
                to_canonical_value(t),
                Err(CanonicalJsonError::InvalidObjectKeyType(_))
            );
        }

        {
            #[derive(Debug, serde::Serialize)]
            struct StructWithIntKey {
                foo: BTreeMap<i8, String>,
            }

            let t = StructWithIntKey { foo: [(4, "bar".to_owned())].into() };
            assert_matches!(
                to_canonical_value(t),
                Err(CanonicalJsonError::InvalidObjectKeyType(_))
            );
        }

        {
            #[derive(Debug, serde::Serialize)]
            struct StructWithUnitKey {
                foo: BTreeMap<(), String>,
            }

            let t = StructWithUnitKey { foo: [((), "bar".to_owned())].into() };
            assert_matches!(
                to_canonical_value(t),
                Err(CanonicalJsonError::InvalidObjectKeyType(_))
            );
        }

        {
            #[derive(Debug, serde::Serialize)]
            struct StructWithTupleKey {
                foo: BTreeMap<(String, String), bool>,
            }

            let t =
                StructWithTupleKey { foo: [(("bar".to_owned(), "baz".to_owned()), false)].into() };
            assert_matches!(
                to_canonical_value(t),
                Err(CanonicalJsonError::InvalidObjectKeyType(_))
            );
        }
    }

    #[test]
    fn to_canonical_value_duplicate_object_key() {
        #[derive(Debug, serde::Serialize)]
        struct StructWithDuplicateKey {
            foo: String,
            #[serde(rename = "foo")]
            bar: Vec<u8>,
        }

        let t = StructWithDuplicateKey { foo: "string".into(), bar: vec![0, 1, 2] };
        assert_matches!(to_canonical_value(t), Err(CanonicalJsonError::DuplicateObjectKey(_)));
    }
}
