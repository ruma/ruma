//! Canonical JSON types and related functions.

use std::{fmt, mem};

use serde::Serialize;
use serde_json::Value as JsonValue;

mod serializer;
mod value;

pub use self::{
    serializer::Serializer,
    value::{CanonicalJsonObject, CanonicalJsonValue},
};
use crate::{room_version_rules::RedactionRules, serde::Raw};

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

/// Errors that can happen in redaction.
#[derive(Debug)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum RedactionError {
    /// The field `field` is not of the correct type `of_type` ([`JsonType`]).
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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
        ev.deserialize_as_unchecked().map(Self)
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
    rules: &RedactionRules,
    redacted_because: Option<RedactedBecause>,
) -> Result<CanonicalJsonObject, RedactionError> {
    redact_in_place(&mut object, rules, redacted_because)?;
    Ok(object)
}

/// Redacts an event using the rules specified in the Matrix client-server specification.
///
/// Functionally equivalent to `redact`, only this'll redact the event in-place.
pub fn redact_in_place(
    event: &mut CanonicalJsonObject,
    rules: &RedactionRules,
    redacted_because: Option<RedactedBecause>,
) -> Result<(), RedactionError> {
    // Get the content keys here even if they're only needed inside the branch below, because we
    // can't teach rust that this is a disjoint borrow with `get_mut("content")`.
    let retained_event_content_keys = match event.get("type") {
        Some(CanonicalJsonValue::String(event_type)) => {
            retained_event_content_keys(event_type.as_ref(), rules)
        }
        Some(_) => return Err(RedactionError::not_of_type("type", JsonType::String)),
        None => return Err(RedactionError::field_missing_from_object("type")),
    };

    if let Some(content_value) = event.get_mut("content") {
        let CanonicalJsonValue::Object(content) = content_value else {
            return Err(RedactionError::not_of_type("content", JsonType::Object));
        };

        retained_event_content_keys.apply(rules, content)?;
    }

    let retained_event_keys =
        RetainedKeys::some(|rules, key, _value| Ok(is_event_key_retained(rules, key)));
    retained_event_keys.apply(rules, event)?;

    if let Some(redacted_because) = redacted_because {
        let unsigned = CanonicalJsonObject::from_iter([(
            "redacted_because".to_owned(),
            redacted_because.0.into(),
        )]);
        event.insert("unsigned".to_owned(), unsigned.into());
    }

    Ok(())
}

/// Redacts the given event content using the given redaction rules for the version of the current
/// room.
///
/// Edits the `content` in-place.
pub fn redact_content_in_place(
    content: &mut CanonicalJsonObject,
    rules: &RedactionRules,
    event_type: impl AsRef<str>,
) -> Result<(), RedactionError> {
    retained_event_content_keys(event_type.as_ref(), rules).apply(rules, content)
}

/// A function that takes redaction rules, a key and its value, and returns whether the field
/// should be retained.
type RetainKeyFn =
    dyn Fn(&RedactionRules, &str, &mut CanonicalJsonValue) -> Result<bool, RedactionError>;

/// Keys to retain on an object.
enum RetainedKeys {
    /// All keys are retained.
    All,

    /// Some keys are retained, they are determined by the inner function.
    Some(Box<RetainKeyFn>),

    /// No keys are retained.
    None,
}

impl RetainedKeys {
    /// Construct a `RetainedKeys::Some(_)` with the given function.
    fn some<F>(retain_key_fn: F) -> Self
    where
        F: Fn(&RedactionRules, &str, &mut CanonicalJsonValue) -> Result<bool, RedactionError>
            + 'static,
    {
        Self::Some(Box::new(retain_key_fn))
    }

    /// Apply this `RetainedKeys` on the given object.
    fn apply(
        &self,
        rules: &RedactionRules,
        object: &mut CanonicalJsonObject,
    ) -> Result<(), RedactionError> {
        match self {
            Self::All => {}
            Self::Some(allow_field_fn) => {
                let old_object = mem::take(object);

                for (key, mut value) in old_object {
                    if allow_field_fn(rules, &key, &mut value)? {
                        object.insert(key, value);
                    }
                }
            }
            Self::None => object.clear(),
        }

        Ok(())
    }
}

/// Get the given keys should be retained at the top level of an event.
fn is_event_key_retained(rules: &RedactionRules, key: &str) -> bool {
    match key {
        "event_id" | "type" | "room_id" | "sender" | "state_key" | "content" | "hashes"
        | "signatures" | "depth" | "prev_events" | "auth_events" | "origin_server_ts" => true,
        "origin" | "membership" | "prev_state" => rules.keep_origin_membership_prev_state,
        _ => false,
    }
}

/// Get the keys that should be retained in the `content` of an event with the given type.
fn retained_event_content_keys(event_type: &str, rules: &RedactionRules) -> RetainedKeys {
    match event_type {
        "m.room.member" => RetainedKeys::some(is_room_member_content_key_retained),
        "m.room.create" => room_create_content_retained_keys(rules),
        "m.room.join_rules" => RetainedKeys::some(|rules, key, _value| {
            is_room_join_rules_content_key_retained(rules, key)
        }),
        "m.room.power_levels" => RetainedKeys::some(|rules, key, _value| {
            is_room_power_levels_content_key_retained(rules, key)
        }),
        "m.room.history_visibility" => RetainedKeys::some(|_rules, key, _value| {
            is_room_history_visibility_content_key_retained(key)
        }),
        "m.room.redaction" => room_redaction_content_retained_keys(rules),
        "m.room.aliases" => room_aliases_content_retained_keys(rules),
        #[cfg(feature = "unstable-msc2870")]
        "m.room.server_acl" => RetainedKeys::some(|rules, key, _value| {
            is_room_server_acl_content_key_retained(rules, key)
        }),
        _ => RetainedKeys::None,
    }
}

/// Whether the given key in the `content` of an `m.room.member` event is retained after redaction.
fn is_room_member_content_key_retained(
    rules: &RedactionRules,
    key: &str,
    value: &mut CanonicalJsonValue,
) -> Result<bool, RedactionError> {
    Ok(match key {
        "membership" => true,
        "join_authorised_via_users_server" => {
            rules.keep_room_member_join_authorised_via_users_server
        }
        "third_party_invite" if rules.keep_room_member_third_party_invite_signed => {
            let Some(third_party_invite) = value.as_object_mut() else {
                return Err(RedactionError::not_of_type("third_party_invite", JsonType::Object));
            };

            third_party_invite.retain(|key, _| key == "signed");

            // Keep the field only if it's not empty.
            !third_party_invite.is_empty()
        }
        _ => false,
    })
}

/// Get the retained keys in the `content` of an `m.room.create` event.
fn room_create_content_retained_keys(rules: &RedactionRules) -> RetainedKeys {
    if rules.keep_room_create_content {
        RetainedKeys::All
    } else {
        RetainedKeys::some(|_rules, field, _value| Ok(field == "creator"))
    }
}

/// Whether the given key in the `content` of an `m.room.join_rules` event is retained after
/// redaction.
fn is_room_join_rules_content_key_retained(
    rules: &RedactionRules,
    key: &str,
) -> Result<bool, RedactionError> {
    Ok(match key {
        "join_rule" => true,
        "allow" => rules.keep_room_join_rules_allow,
        _ => false,
    })
}

/// Whether the given key in the `content` of an `m.room.power_levels` event is retained after
/// redaction.
fn is_room_power_levels_content_key_retained(
    rules: &RedactionRules,
    key: &str,
) -> Result<bool, RedactionError> {
    Ok(match key {
        "ban" | "events" | "events_default" | "kick" | "redact" | "state_default" | "users"
        | "users_default" => true,
        "invite" => rules.keep_room_power_levels_invite,
        _ => false,
    })
}

/// Whether the given key in the `content` of an `m.room.history_visibility` event is retained after
/// redaction.
fn is_room_history_visibility_content_key_retained(key: &str) -> Result<bool, RedactionError> {
    Ok(key == "history_visibility")
}

/// Get the retained keys in the `content` of an `m.room.redaction` event.
fn room_redaction_content_retained_keys(rules: &RedactionRules) -> RetainedKeys {
    if rules.keep_room_redaction_redacts {
        RetainedKeys::some(|_rules, field, _value| Ok(field == "redacts"))
    } else {
        RetainedKeys::None
    }
}

/// Get the retained keys in the `content` of an `m.room.aliases` event.
fn room_aliases_content_retained_keys(rules: &RedactionRules) -> RetainedKeys {
    if rules.keep_room_aliases_aliases {
        RetainedKeys::some(|_rules, field, _value| Ok(field == "aliases"))
    } else {
        RetainedKeys::None
    }
}

/// Whether the given key in the `content` of an `m.room.server_acl` event is retained after
/// redaction.
#[cfg(feature = "unstable-msc2870")]
fn is_room_server_acl_content_key_retained(
    rules: &RedactionRules,
    key: &str,
) -> Result<bool, RedactionError> {
    Ok(match key {
        "allow" | "deny" | "allow_ip_literals" => {
            rules.keep_room_server_acl_allow_deny_allow_ip_literals
        }
        _ => false,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use assert_matches2::assert_matches;
    use js_int::int;
    use serde_json::{
        from_str as from_json_str, json, to_string as to_json_string, to_value as to_json_value,
        value::RawValue as RawJsonValue,
    };

    use super::{
        CanonicalJsonError, redact_in_place, to_canonical_value, try_from_json_map,
        value::CanonicalJsonValue,
    };
    use crate::room_version_rules::RedactionRules;

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

        assert_eq!(to_canonical_value(t).unwrap(), CanonicalJsonValue::Object(expected));
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

        redact_in_place(&mut object, &RedactionRules::V1, None).unwrap();

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

        redact_in_place(&mut object, &RedactionRules::V9, None).unwrap();

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

        redact_in_place(&mut object, &RedactionRules::V11, None).unwrap();

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
