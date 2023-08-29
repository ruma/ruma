//! Canonical JSON types and related functions.

use std::{fmt, mem};

use serde::Serialize;
use serde_json::Value as JsonValue;

mod value;

pub use self::value::{CanonicalJsonObject, CanonicalJsonValue};
use crate::{serde::Raw, RoomVersionId};

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
) -> Result<CanonicalJsonValue, CanonicalJsonError> {
    serde_json::to_value(value).map_err(CanonicalJsonError::SerDe)?.try_into()
}

/// The value to put in `unsigned.redacted_because`.
#[derive(Clone, Debug)]
pub struct RedactedBecause(CanonicalJsonObject);

impl RedactedBecause {
    /// Create a `RedactedBecause` from an arbitrary JSON object.
    pub fn from_json(obj: CanonicalJsonObject) -> Self {
        Self(obj)
    }

    /// Create a `RedactedBecause` from a redaction event.
    ///
    /// Fails if the raw event is not valid canonical JSON.
    pub fn from_raw_event(ev: &Raw<impl RedactionEvent>) -> serde_json::Result<Self> {
        ev.deserialize_as().map(Self)
    }
}

/// Marker trait for redaction events.
pub trait RedactionEvent {}

/// Redacts an event using the rules specified in the Matrix client-server specification.
///
/// This is part of the process of signing an event.
///
/// Redaction is also suggested when verifying an event with `verify_event` returns
/// `Verified::Signatures`. See the documentation for `Verified` for details.
///
/// Returns a new JSON object with all applicable fields redacted.
///
/// # Parameters
///
/// * `object`: A JSON object to redact.
/// * `version`: The room version, determines which keys to keep for a few event types.
/// * `redacted_because`: If this is set, an `unsigned` object with a `redacted_because` field set
///   to the given value is added to the event after redaction.
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
    mut object: CanonicalJsonObject,
    version: &RoomVersionId,
    redacted_because: Option<RedactedBecause>,
) -> Result<CanonicalJsonObject, RedactionError> {
    redact_in_place(&mut object, version, redacted_because)?;
    Ok(object)
}

/// Redacts an event using the rules specified in the Matrix client-server specification.
///
/// Functionally equivalent to `redact`, only this'll redact the event in-place.
pub fn redact_in_place(
    event: &mut CanonicalJsonObject,
    version: &RoomVersionId,
    redacted_because: Option<RedactedBecause>,
) -> Result<(), RedactionError> {
    // Get the content keys here even if they're only needed inside the branch below, because we
    // can't teach rust that this is a disjoint borrow with `get_mut("content")`.
    let allowed_content_keys = match event.get("type") {
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

        object_retain_keys(content, allowed_content_keys)?;
    }

    let mut old_event = mem::take(event);

    for &key in allowed_event_keys_for(version) {
        if let Some(value) = old_event.remove(key) {
            event.insert(key.to_owned(), value);
        }
    }

    if let Some(redacted_because) = redacted_because {
        let unsigned = CanonicalJsonObject::from_iter([(
            "redacted_because".to_owned(),
            redacted_because.0.into(),
        )]);
        event.insert("unsigned".to_owned(), unsigned.into());
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
) -> Result<(), RedactionError> {
    object_retain_keys(object, allowed_content_keys_for(event_type.as_ref(), version))
}

fn object_retain_keys(
    object: &mut CanonicalJsonObject,
    allowed_keys: &AllowedKeys,
) -> Result<(), RedactionError> {
    match *allowed_keys {
        AllowedKeys::All => {}
        AllowedKeys::Some { keys, nested } => {
            object_retain_some_keys(object, keys, nested)?;
        }
        AllowedKeys::None => {
            object.clear();
        }
    }

    Ok(())
}

fn object_retain_some_keys(
    object: &mut CanonicalJsonObject,
    keys: &[&str],
    nested: &[(&str, &AllowedKeys)],
) -> Result<(), RedactionError> {
    let mut old_object = mem::take(object);

    for &(nested_key, nested_allowed_keys) in nested {
        if let Some((key, mut nested_object_value)) = old_object.remove_entry(nested_key) {
            let nested_object = match &mut nested_object_value {
                CanonicalJsonValue::Object(map) => map,
                _ => return Err(RedactionError::not_of_type(nested_key, JsonType::Object)),
            };

            object_retain_keys(nested_object, nested_allowed_keys)?;

            // If the object is empty, it means none of the nested fields were found so we
            // don't want to keep the object.
            if !nested_object.is_empty() {
                object.insert(key, nested_object_value);
            }
        }
    }

    for &key in keys {
        if let Some((key, value)) = old_object.remove_entry(key) {
            object.insert(key, value);
        }
    }

    Ok(())
}

/// The fields that are allowed to remain in an event during redaction depending on the room
/// version.
fn allowed_event_keys_for(version: &RoomVersionId) -> &'static [&'static str] {
    match version {
        RoomVersionId::V1
        | RoomVersionId::V2
        | RoomVersionId::V3
        | RoomVersionId::V4
        | RoomVersionId::V5
        | RoomVersionId::V6
        | RoomVersionId::V7
        | RoomVersionId::V8
        | RoomVersionId::V9
        | RoomVersionId::V10 => &[
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
        ],
        _ => &[
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
            "auth_events",
            "origin_server_ts",
        ],
    }
}

/// List of keys to preserve on an object.
#[derive(Clone, Copy)]
enum AllowedKeys {
    /// All keys are preserved.
    All,
    /// Some keys are preserved.
    Some {
        /// The keys to preserve on this object.
        keys: &'static [&'static str],

        /// Keys to preserve on nested objects.
        ///
        /// A list of `(nested_object_key, nested_allowed_keys)`.
        nested: &'static [(&'static str, &'static AllowedKeys)],
    },
    /// No keys are preserved.
    None,
}

impl AllowedKeys {
    /// Creates an new `AllowedKeys::Some` with the given keys at this level.
    const fn some(keys: &'static [&'static str]) -> Self {
        Self::Some { keys, nested: &[] }
    }

    /// Creates an new `AllowedKeys::Some` with the given keys and nested keys.
    const fn some_nested(
        keys: &'static [&'static str],
        nested: &'static [(&'static str, &'static AllowedKeys)],
    ) -> Self {
        Self::Some { keys, nested }
    }
}

/// Allowed keys in `m.room.member`'s content according to room version 1.
static ROOM_MEMBER_V1: AllowedKeys = AllowedKeys::some(&["membership"]);
/// Allowed keys in `m.room.member`'s content according to room version 9.
static ROOM_MEMBER_V9: AllowedKeys =
    AllowedKeys::some(&["membership", "join_authorised_via_users_server"]);
/// Allowed keys in `m.room.member`'s content according to room version 11.
static ROOM_MEMBER_V11: AllowedKeys = AllowedKeys::some_nested(
    &["membership", "join_authorised_via_users_server"],
    &[("third_party_invite", &ROOM_MEMBER_THIRD_PARTY_INVITE_V11)],
);
/// Allowed keys in the `third_party_invite` field of `m.room.member`'s content according to room
/// version 11.
static ROOM_MEMBER_THIRD_PARTY_INVITE_V11: AllowedKeys = AllowedKeys::some(&["signed"]);

/// Allowed keys in `m.room.create`'s content according to room version 1.
static ROOM_CREATE_V1: AllowedKeys = AllowedKeys::some(&["creator"]);

/// Allowed keys in `m.room.join_rules`'s content according to room version 1.
static ROOM_JOIN_RULES_V1: AllowedKeys = AllowedKeys::some(&["join_rule"]);
/// Allowed keys in `m.room.join_rules`'s content according to room version 8.
static ROOM_JOIN_RULES_V8: AllowedKeys = AllowedKeys::some(&["join_rule", "allow"]);

/// Allowed keys in `m.room.power_levels`'s content according to room version 1.
static ROOM_POWER_LEVELS_V1: AllowedKeys = AllowedKeys::some(&[
    "ban",
    "events",
    "events_default",
    "kick",
    "redact",
    "state_default",
    "users",
    "users_default",
]);
/// Allowed keys in `m.room.power_levels`'s content according to room version 11.
static ROOM_POWER_LEVELS_V11: AllowedKeys = AllowedKeys::some(&[
    "ban",
    "events",
    "events_default",
    "invite",
    "kick",
    "redact",
    "state_default",
    "users",
    "users_default",
]);

/// Allowed keys in `m.room.aliases`'s content according to room version 1.
static ROOM_ALIASES_V1: AllowedKeys = AllowedKeys::some(&["aliases"]);

/// Allowed keys in `m.room.server_acl`'s content according to MSC2870.
#[cfg(feature = "unstable-msc2870")]
static ROOM_SERVER_ACL_MSC2870: AllowedKeys =
    AllowedKeys::some(&["allow", "deny", "allow_ip_literals"]);

/// Allowed keys in `m.room.history_visibility`'s content according to room version 1.
static ROOM_HISTORY_VISIBILITY_V1: AllowedKeys = AllowedKeys::some(&["history_visibility"]);

/// Allowed keys in `m.room.redaction`'s content according to room version 11.
static ROOM_REDACTION_V11: AllowedKeys = AllowedKeys::some(&["redacts"]);

fn allowed_content_keys_for(event_type: &str, version: &RoomVersionId) -> &'static AllowedKeys {
    match event_type {
        "m.room.member" => match version {
            RoomVersionId::V1
            | RoomVersionId::V2
            | RoomVersionId::V3
            | RoomVersionId::V4
            | RoomVersionId::V5
            | RoomVersionId::V6
            | RoomVersionId::V7
            | RoomVersionId::V8 => &ROOM_MEMBER_V1,
            RoomVersionId::V9 | RoomVersionId::V10 => &ROOM_MEMBER_V9,
            _ => &ROOM_MEMBER_V11,
        },
        "m.room.create" => match version {
            RoomVersionId::V1
            | RoomVersionId::V2
            | RoomVersionId::V3
            | RoomVersionId::V4
            | RoomVersionId::V5
            | RoomVersionId::V6
            | RoomVersionId::V7
            | RoomVersionId::V8
            | RoomVersionId::V9
            | RoomVersionId::V10 => &ROOM_CREATE_V1,
            _ => &AllowedKeys::All,
        },
        "m.room.join_rules" => match version {
            RoomVersionId::V1
            | RoomVersionId::V2
            | RoomVersionId::V3
            | RoomVersionId::V4
            | RoomVersionId::V5
            | RoomVersionId::V6
            | RoomVersionId::V7 => &ROOM_JOIN_RULES_V1,
            _ => &ROOM_JOIN_RULES_V8,
        },
        "m.room.power_levels" => match version {
            RoomVersionId::V1
            | RoomVersionId::V2
            | RoomVersionId::V3
            | RoomVersionId::V4
            | RoomVersionId::V5
            | RoomVersionId::V6
            | RoomVersionId::V7
            | RoomVersionId::V8
            | RoomVersionId::V9
            | RoomVersionId::V10 => &ROOM_POWER_LEVELS_V1,
            _ => &ROOM_POWER_LEVELS_V11,
        },
        "m.room.aliases" => match version {
            RoomVersionId::V1
            | RoomVersionId::V2
            | RoomVersionId::V3
            | RoomVersionId::V4
            | RoomVersionId::V5 => &ROOM_ALIASES_V1,
            // All other room versions, including custom ones, are treated by version 6 rules.
            // TODO: Should we return an error for unknown versions instead?
            _ => &AllowedKeys::None,
        },
        #[cfg(feature = "unstable-msc2870")]
        "m.room.server_acl" if version.as_str() == "org.matrix.msc2870" => &ROOM_SERVER_ACL_MSC2870,
        "m.room.history_visibility" => &ROOM_HISTORY_VISIBILITY_V1,
        "m.room.redaction" => match version {
            RoomVersionId::V1
            | RoomVersionId::V2
            | RoomVersionId::V3
            | RoomVersionId::V4
            | RoomVersionId::V5
            | RoomVersionId::V6
            | RoomVersionId::V7
            | RoomVersionId::V8
            | RoomVersionId::V9
            | RoomVersionId::V10 => &AllowedKeys::None,
            _ => &ROOM_REDACTION_V11,
        },
        _ => &AllowedKeys::None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use assert_matches2::assert_matches;
    use js_int::int;
    use serde_json::{
        from_str as from_json_str, json, to_string as to_json_string, to_value as to_json_value,
    };

    use super::{
        redact_in_place, to_canonical_value, try_from_json_map, value::CanonicalJsonValue,
    };
    use crate::RoomVersionId;

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

    #[test]
    fn redact_allowed_keys_some() {
        let original_event = json!({
            "content": {
                "ban": 50,
                "events": {
                    "m.room.avatar": 50,
                    "m.room.canonical_alias": 50,
                    "m.room.history_visibility": 100,
                    "m.room.name": 50,
                    "m.room.power_levels": 100
                },
                "events_default": 0,
                "invite": 0,
                "kick": 50,
                "redact": 50,
                "state_default": 50,
                "users": {
                    "@example:localhost": 100
                },
                "users_default": 0
            },
            "event_id": "$15139375512JaHAW:localhost",
            "origin_server_ts": 45,
            "sender": "@example:localhost",
            "room_id": "!room:localhost",
            "state_key": "",
            "type": "m.room.power_levels",
            "unsigned": {
                "age": 45
            }
        });

        assert_matches!(
            CanonicalJsonValue::try_from(original_event),
            Ok(CanonicalJsonValue::Object(mut object))
        );

        redact_in_place(&mut object, &RoomVersionId::V1, None).unwrap();

        let redacted_event = to_json_value(&object).unwrap();

        assert_eq!(
            redacted_event,
            json!({
                "content": {
                    "ban": 50,
                    "events": {
                        "m.room.avatar": 50,
                        "m.room.canonical_alias": 50,
                        "m.room.history_visibility": 100,
                        "m.room.name": 50,
                        "m.room.power_levels": 100
                    },
                    "events_default": 0,
                    "kick": 50,
                    "redact": 50,
                    "state_default": 50,
                    "users": {
                        "@example:localhost": 100
                    },
                    "users_default": 0
                },
                "event_id": "$15139375512JaHAW:localhost",
                "origin_server_ts": 45,
                "sender": "@example:localhost",
                "room_id": "!room:localhost",
                "state_key": "",
                "type": "m.room.power_levels",
            })
        );
    }

    #[test]
    fn redact_allowed_keys_none() {
        let original_event = json!({
            "content": {
                "aliases": ["#somewhere:localhost"]
            },
            "event_id": "$152037280074GZeOm:localhost",
            "origin_server_ts": 1,
            "sender": "@example:localhost",
            "state_key": "room.com",
            "room_id": "!room:room.com",
            "type": "m.room.aliases",
            "unsigned": {
                "age": 1
            }
        });

        assert_matches!(
            CanonicalJsonValue::try_from(original_event),
            Ok(CanonicalJsonValue::Object(mut object))
        );

        redact_in_place(&mut object, &RoomVersionId::V10, None).unwrap();

        let redacted_event = to_json_value(&object).unwrap();

        assert_eq!(
            redacted_event,
            json!({
                "content": {},
                "event_id": "$152037280074GZeOm:localhost",
                "origin_server_ts": 1,
                "sender": "@example:localhost",
                "state_key": "room.com",
                "room_id": "!room:room.com",
                "type": "m.room.aliases",
            })
        );
    }

    #[test]
    fn redact_allowed_keys_all() {
        let original_event = json!({
            "content": {
              "m.federate": true,
              "predecessor": {
                "event_id": "$something",
                "room_id": "!oldroom:example.org"
              },
              "room_version": "11",
            },
            "event_id": "$143273582443PhrSn",
            "origin_server_ts": 1_432_735,
            "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
            "sender": "@example:example.org",
            "state_key": "",
            "type": "m.room.create",
            "unsigned": {
              "age": 1234,
            },
        });

        assert_matches!(
            CanonicalJsonValue::try_from(original_event),
            Ok(CanonicalJsonValue::Object(mut object))
        );

        redact_in_place(&mut object, &RoomVersionId::V11, None).unwrap();

        let redacted_event = to_json_value(&object).unwrap();

        assert_eq!(
            redacted_event,
            json!({
                "content": {
                  "m.federate": true,
                  "predecessor": {
                    "event_id": "$something",
                    "room_id": "!oldroom:example.org"
                  },
                  "room_version": "11",
                },
                "event_id": "$143273582443PhrSn",
                "origin_server_ts": 1_432_735,
                "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
                "sender": "@example:example.org",
                "state_key": "",
                "type": "m.room.create",
            })
        );
    }
}
