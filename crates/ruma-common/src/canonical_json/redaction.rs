use std::{fmt, mem};

use super::value::{CanonicalJsonObject, CanonicalJsonType, CanonicalJsonValue};
use crate::{room_version_rules::RedactionRules, serde::Raw};

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
    retained_event_keys(event)?.apply(rules, event);

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
) {
    retained_event_content_keys(event_type.as_ref(), rules).apply(rules, content);
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

/// Errors that can happen in redaction.
#[derive(Debug)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum RedactionError {
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
    MissingField {
        /// The path of the missing field.
        path: String,
    },
}

impl fmt::Display for RedactionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RedactionError::InvalidType { path, expected, found } => {
                write!(f, "invalid type at `{path}`: expected {expected:?}, found {found:?}")
            }
            RedactionError::MissingField { path } => {
                write!(f, "missing field: `{path}`")
            }
        }
    }
}

impl std::error::Error for RedactionError {}

/// A function that takes redaction rules and a key and returns whether the field should be
/// retained.
type RetainKeyFn = dyn Fn(&RedactionRules, &str) -> RetainKey;

/// Whether a key should be retained.
enum RetainKey {
    /// The key should be retained.
    Yes {
        /// The rules to apply to the child keys if the value of this key is an object.
        ///
        /// If the value is an object and this is `None`, the default is [`RetainedKeys::All`].
        child_retained_keys: Option<RetainedKeys>,
    },

    /// The key should be redacted.
    No,
}

impl From<bool> for RetainKey {
    fn from(value: bool) -> Self {
        if value { Self::Yes { child_retained_keys: None } } else { Self::No }
    }
}

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
        F: Fn(&RedactionRules, &str) -> RetainKey + Clone + 'static,
    {
        Self::Some(Box::new(retain_key_fn))
    }

    /// Apply this `RetainedKeys` on the given object.
    fn apply(&self, rules: &RedactionRules, object: &mut CanonicalJsonObject) {
        match self {
            Self::All => {}
            Self::Some(retain_key_fn) => {
                let old_object = mem::take(object);

                for (key, mut value) in old_object {
                    if let RetainKey::Yes { child_retained_keys } = retain_key_fn(rules, &key) {
                        if let Some(child_retained_keys) = child_retained_keys
                            && let CanonicalJsonValue::Object(child_object) = &mut value
                        {
                            child_retained_keys.apply(rules, child_object);
                        }

                        object.insert(key, value);
                    }
                }
            }
            Self::None => object.clear(),
        }
    }
}

/// Get the given keys should be retained at the top level of an event.
fn retained_event_keys(event: &CanonicalJsonObject) -> Result<RetainedKeys, RedactionError> {
    let event_type = match event.get("type") {
        Some(CanonicalJsonValue::String(event_type)) => event_type.clone(),
        Some(value) => {
            return Err(RedactionError::InvalidType {
                path: "type".to_owned(),
                expected: CanonicalJsonType::String,
                found: value.json_type(),
            });
        }
        None => return Err(RedactionError::MissingField { path: "type".to_owned() }),
    };

    Ok(RetainedKeys::some(move |rules, key| match key {
        "content" => RetainKey::Yes {
            child_retained_keys: Some(retained_event_content_keys(&event_type, rules)),
        },
        "event_id" | "type" | "room_id" | "sender" | "state_key" | "hashes" | "signatures"
        | "depth" | "prev_events" | "auth_events" | "origin_server_ts" => true.into(),
        "origin" | "membership" | "prev_state" => rules.keep_origin_membership_prev_state.into(),
        _ => false.into(),
    }))
}

/// Get the keys that should be retained in the `content` of an event with the given type.
fn retained_event_content_keys(event_type: &str, rules: &RedactionRules) -> RetainedKeys {
    match event_type {
        "m.room.member" => RetainedKeys::some(is_room_member_content_key_retained),
        "m.room.create" => room_create_content_retained_keys(rules),
        "m.room.join_rules" => RetainedKeys::some(is_room_join_rules_content_key_retained),
        "m.room.power_levels" => RetainedKeys::some(is_room_power_levels_content_key_retained),
        "m.room.history_visibility" => {
            RetainedKeys::some(|_rules, key| is_room_history_visibility_content_key_retained(key))
        }
        "m.room.redaction" => room_redaction_content_retained_keys(rules),
        "m.room.aliases" => room_aliases_content_retained_keys(rules),
        #[cfg(feature = "unstable-msc2870")]
        "m.room.server_acl" => RetainedKeys::some(is_room_server_acl_content_key_retained),
        _ => RetainedKeys::None,
    }
}

/// Whether the given key in the `content` of an `m.room.member` event is retained after redaction.
fn is_room_member_content_key_retained(rules: &RedactionRules, key: &str) -> RetainKey {
    match key {
        "membership" => true.into(),
        "join_authorised_via_users_server" => {
            rules.keep_room_member_join_authorised_via_users_server.into()
        }
        "third_party_invite" if rules.keep_room_member_third_party_invite_signed => {
            RetainKey::Yes {
                child_retained_keys: Some(RetainedKeys::some(|_rules, key| {
                    (key == "signed").into()
                })),
            }
        }
        _ => false.into(),
    }
}

/// Get the retained keys in the `content` of an `m.room.create` event.
fn room_create_content_retained_keys(rules: &RedactionRules) -> RetainedKeys {
    if rules.keep_room_create_content {
        RetainedKeys::All
    } else {
        RetainedKeys::some(|_rules, field| (field == "creator").into())
    }
}

/// Whether the given key in the `content` of an `m.room.join_rules` event is retained after
/// redaction.
fn is_room_join_rules_content_key_retained(rules: &RedactionRules, key: &str) -> RetainKey {
    match key {
        "join_rule" => true,
        "allow" => rules.keep_room_join_rules_allow,
        _ => false,
    }
    .into()
}

/// Whether the given key in the `content` of an `m.room.power_levels` event is retained after
/// redaction.
fn is_room_power_levels_content_key_retained(rules: &RedactionRules, key: &str) -> RetainKey {
    match key {
        "ban" | "events" | "events_default" | "kick" | "redact" | "state_default" | "users"
        | "users_default" => true,
        "invite" => rules.keep_room_power_levels_invite,
        _ => false,
    }
    .into()
}

/// Whether the given key in the `content` of an `m.room.history_visibility` event is retained after
/// redaction.
fn is_room_history_visibility_content_key_retained(key: &str) -> RetainKey {
    (key == "history_visibility").into()
}

/// Get the retained keys in the `content` of an `m.room.redaction` event.
fn room_redaction_content_retained_keys(rules: &RedactionRules) -> RetainedKeys {
    if rules.keep_room_redaction_redacts {
        RetainedKeys::some(|_rules, field| (field == "redacts").into())
    } else {
        RetainedKeys::None
    }
}

/// Get the retained keys in the `content` of an `m.room.aliases` event.
fn room_aliases_content_retained_keys(rules: &RedactionRules) -> RetainedKeys {
    if rules.keep_room_aliases_aliases {
        RetainedKeys::some(|_rules, field| (field == "aliases").into())
    } else {
        RetainedKeys::None
    }
}

/// Whether the given key in the `content` of an `m.room.server_acl` event is retained after
/// redaction.
#[cfg(feature = "unstable-msc2870")]
fn is_room_server_acl_content_key_retained(rules: &RedactionRules, key: &str) -> RetainKey {
    match key {
        "allow" | "deny" | "allow_ip_literals" => {
            rules.keep_room_server_acl_allow_deny_allow_ip_literals
        }
        _ => false,
    }
    .into()
}
