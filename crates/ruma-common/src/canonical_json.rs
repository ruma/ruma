//! Canonical JSON types and related functions.

use std::{fmt, mem};

use serde::Serialize;
use serde_json::Value as JsonValue;

mod value;

use crate::RoomVersionId;

pub use self::value::{CanonicalJsonObject, CanonicalJsonValue};

/// The set of possible errors when serializing to canonical JSON.
#[cfg(feature = "canonical-json")]
#[derive(Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum CanonicalJsonError {
    /// The numeric value failed conversion to js_int::Int.
    IntConvert,

    /// An error occurred while serializing/deserializing.
    SerDe(serde_json::Error),
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

/// Errors that can happen in redaction.
#[cfg(feature = "canonical-json")]
#[derive(Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum RedactionError {
    /// The field `field` is not of the correct type `of_type` ([`JsonType`]).
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    NotOfType {
        /// The field name.
        field: String,
        /// The expected JSON type.
        of_type: JsonType,
    },

    /// The given required field is missing from a JSON object.
    JsonFieldMissingFromObject(String),
}

impl fmt::Display for RedactionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RedactionError::NotOfType { field, of_type } => {
                write!(f, "Value in {field:?} must be a JSON {of_type:?}")
            }
            RedactionError::JsonFieldMissingFromObject(field) => {
                write!(f, "JSON object must contain the field {field:?}")
            }
        }
    }
}

impl std::error::Error for RedactionError {}

impl RedactionError {
    fn not_of_type(target: &str, of_type: JsonType) -> Self {
        Self::NotOfType { field: target.to_owned(), of_type }
    }

    fn field_missing_from_object(target: &str) -> Self {
        Self::JsonFieldMissingFromObject(target.to_owned())
    }
}

/// A JSON type enum for [`RedactionError`] variants.
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

/// Redacts an event using the rules specified in the Matrix client-server specification.
///
/// This is part of the process of signing an event.
///
/// Redaction is also suggested when a verifying an event with `verify_event` returns
/// `Verified::Signatures`. See the documentation for `Verified` for details.
///
/// Returns a new JSON object with all applicable fields redacted.
///
/// # Parameters
///
/// * object: A JSON object to redact.
///
/// # Errors
///
/// Returns an error if:
///
/// * `object` contains a field called `content` that is not a JSON object.
/// * `object` contains a field called `hashes` that is not a JSON object.
/// * `object` contains a field called `signatures` that is not a JSON object.
/// * `object` is missing the `type` field or the field is not a JSON string.
pub fn redact(
    object: &CanonicalJsonObject,
    version: &RoomVersionId,
) -> Result<CanonicalJsonObject, RedactionError> {
    let mut val = object.clone();
    redact_in_place(&mut val, version)?;
    Ok(val)
}

/// Redacts an event using the rules specified in the Matrix client-server specification.
///
/// Functionally equivalent to `redact`, only;
/// * upon error, the event is not touched.
/// * this'll redact the event in-place.
pub fn redact_in_place(
    event: &mut CanonicalJsonObject,
    version: &RoomVersionId,
) -> Result<(), RedactionError> {
    // Get the content keys here instead of the event type, because we cant teach rust that this is
    // a disjoint borrow.
    let allowed_content_keys: &[&str] = match event.get("type") {
        Some(CanonicalJsonValue::String(event_type)) => {
            allowed_content_keys_for(event_type, version)
        }
        Some(_) => return Err(RedactionError::not_of_type("type", JsonType::String)),
        None => return Err(RedactionError::field_missing_from_object("type")),
    };

    if let Some(content_value) = event.get_mut("content") {
        let content = match content_value {
            CanonicalJsonValue::Object(map) => map,
            _ => return Err(RedactionError::not_of_type("content", JsonType::Object)),
        };

        object_retain_keys(content, allowed_content_keys);
    }

    let mut old_event = mem::take(event);

    for &key in ALLOWED_KEYS {
        if let Some(value) = old_event.remove(key) {
            event.insert(key.to_owned(), value);
        }
    }

    Ok(())
}

/// Redacts event content using the rules specified in the Matrix client-server specification.
///
/// Edits the `object` in-place.
pub fn redact_content_in_place(
    object: &mut CanonicalJsonObject,
    version: &RoomVersionId,
    event_type: impl AsRef<str>,
) {
    object_retain_keys(object, allowed_content_keys_for(event_type.as_ref(), version));
}

fn object_retain_keys(object: &mut CanonicalJsonObject, keys: &[&str]) {
    let mut old_content = mem::take(object);

    for &key in keys {
        if let Some(value) = old_content.remove(key) {
            object.insert(key.to_owned(), value);
        }
    }
}

/// The fields that are allowed to remain in an event during redaction.
static ALLOWED_KEYS: &[&str] = &[
    "event_id",
    "type",
    "room_id",
    "sender",
    "state_key",
    "content",
    "hashes",
    "signatures",
    "depth",
    "prev_events",
    "prev_state",
    "auth_events",
    "origin",
    "origin_server_ts",
    "membership",
];

fn allowed_content_keys_for(event_type: &str, version: &RoomVersionId) -> &'static [&'static str] {
    match event_type {
        "m.room.member" => match version {
            RoomVersionId::V9 | RoomVersionId::V10 => {
                &["membership", "join_authorised_via_users_server"]
            }
            _ => &["membership"],
        },
        "m.room.create" => &["creator"],
        "m.room.join_rules" => match version {
            RoomVersionId::V8 | RoomVersionId::V9 | RoomVersionId::V10 => &["join_rule", "allow"],
            _ => &["join_rule"],
        },
        "m.room.power_levels" => &[
            "ban",
            "events",
            "events_default",
            "kick",
            "redact",
            "state_default",
            "users",
            "users_default",
        ],
        "m.room.aliases" => match version {
            RoomVersionId::V1
            | RoomVersionId::V2
            | RoomVersionId::V3
            | RoomVersionId::V4
            | RoomVersionId::V5 => &["aliases"],
            // All other room versions, including custom ones, are treated by version 6 rules.
            // TODO: Should we return an error for unknown versions instead?
            _ => &[],
        },
        #[cfg(feature = "unstable-msc2870")]
        "m.room.server_acl" if version.as_str() == "org.matrix.msc2870" => {
            &["allow", "deny", "allow_ip_literals"]
        }
        "m.room.history_visibility" => &["history_visibility"],
        _ => &[],
    }
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
