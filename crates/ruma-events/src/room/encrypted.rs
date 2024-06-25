//! Types for the [`m.room.encrypted`] event.
//!
//! [`m.room.encrypted`]: https://spec.matrix.org/latest/client-server-api/#mroomencrypted

use std::{borrow::Cow, collections::BTreeMap};

use js_int::UInt;
use ruma_common::{serde::JsonObject, OwnedDeviceId, OwnedEventId};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::message;
use crate::relation::{Annotation, CustomRelation, InReplyTo, Reference, RelationType, Thread};

mod relation_serde;

/// The content of an `m.room.encrypted` event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.encrypted", kind = MessageLike)]
pub struct RoomEncryptedEventContent {
    /// Algorithm-specific fields.
    #[serde(flatten)]
    pub scheme: EncryptedEventScheme,

    /// Information about related events.
    #[serde(rename = "m.relates_to", skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<Relation>,
}

impl RoomEncryptedEventContent {
    /// Creates a new `RoomEncryptedEventContent` with the given scheme and relation.
    pub fn new(scheme: EncryptedEventScheme, relates_to: Option<Relation>) -> Self {
        Self { scheme, relates_to }
    }
}

impl From<EncryptedEventScheme> for RoomEncryptedEventContent {
    fn from(scheme: EncryptedEventScheme) -> Self {
        Self { scheme, relates_to: None }
    }
}

/// The to-device content of an `m.room.encrypted` event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.encrypted", kind = ToDevice)]
pub struct ToDeviceRoomEncryptedEventContent {
    /// Algorithm-specific fields.
    #[serde(flatten)]
    pub scheme: EncryptedEventScheme,
}

impl ToDeviceRoomEncryptedEventContent {
    /// Creates a new `ToDeviceRoomEncryptedEventContent` with the given scheme.
    pub fn new(scheme: EncryptedEventScheme) -> Self {
        Self { scheme }
    }
}

impl From<EncryptedEventScheme> for ToDeviceRoomEncryptedEventContent {
    fn from(scheme: EncryptedEventScheme) -> Self {
        Self { scheme }
    }
}

/// The encryption scheme for `RoomEncryptedEventContent`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "algorithm")]
pub enum EncryptedEventScheme {
    /// An event encrypted with `m.olm.v1.curve25519-aes-sha2`.
    #[serde(rename = "m.olm.v1.curve25519-aes-sha2")]
    OlmV1Curve25519AesSha2(OlmV1Curve25519AesSha2Content),

    /// An event encrypted with `m.megolm.v1.aes-sha2`.
    #[serde(rename = "m.megolm.v1.aes-sha2")]
    MegolmV1AesSha2(MegolmV1AesSha2Content),
}

/// Relationship information about an encrypted event.
///
/// Outside of the encrypted payload to support server aggregation.
#[derive(Clone, Debug)]
#[allow(clippy::manual_non_exhaustive)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum Relation {
    /// An `m.in_reply_to` relation indicating that the event is a reply to another event.
    Reply {
        /// Information about another message being replied to.
        in_reply_to: InReplyTo,
    },

    /// An event that replaces another event.
    Replacement(Replacement),

    /// A reference to another event.
    Reference(Reference),

    /// An annotation to an event.
    Annotation(Annotation),

    /// An event that belongs to a thread.
    Thread(Thread),

    #[doc(hidden)]
    _Custom(CustomRelation),
}

impl Relation {
    /// The type of this `Relation`.
    ///
    /// Returns an `Option` because the `Reply` relation does not have a`rel_type` field.
    pub fn rel_type(&self) -> Option<RelationType> {
        match self {
            Relation::Reply { .. } => None,
            Relation::Replacement(_) => Some(RelationType::Replacement),
            Relation::Reference(_) => Some(RelationType::Reference),
            Relation::Annotation(_) => Some(RelationType::Annotation),
            Relation::Thread(_) => Some(RelationType::Thread),
            Relation::_Custom(c) => c.rel_type(),
        }
    }

    /// The associated data.
    ///
    /// The returned JSON object holds the contents of `m.relates_to`, including `rel_type` and
    /// `event_id` if present, but not things like `m.new_content` for `m.replace` relations that
    /// live next to `m.relates_to`.
    ///
    /// Prefer to use the public variants of `Relation` where possible; this method is meant to
    /// be used for custom relations only.
    pub fn data(&self) -> Cow<'_, JsonObject> {
        if let Relation::_Custom(CustomRelation(data)) = self {
            Cow::Borrowed(data)
        } else {
            Cow::Owned(self.serialize_data())
        }
    }
}

impl<C> From<message::Relation<C>> for Relation {
    fn from(rel: message::Relation<C>) -> Self {
        match rel {
            message::Relation::Reply { in_reply_to } => Self::Reply { in_reply_to },
            message::Relation::Replacement(re) => {
                Self::Replacement(Replacement { event_id: re.event_id })
            }
            message::Relation::Thread(t) => Self::Thread(Thread {
                event_id: t.event_id,
                in_reply_to: t.in_reply_to,
                is_falling_back: t.is_falling_back,
            }),
            message::Relation::_Custom(c) => Self::_Custom(c),
        }
    }
}

/// The event this relation belongs to [replaces another event].
///
/// In contrast to [`relation::Replacement`](crate::relation::Replacement), this
/// struct doesn't store the new content, since that is part of the encrypted content of an
/// `m.room.encrypted` events.
///
/// [replaces another event]: https://spec.matrix.org/latest/client-server-api/#event-replacements
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "rel_type", rename = "m.replace")]
pub struct Replacement {
    /// The ID of the event being replaced.
    pub event_id: OwnedEventId,
}

impl Replacement {
    /// Creates a new `Replacement` with the given event ID.
    pub fn new(event_id: OwnedEventId) -> Self {
        Self { event_id }
    }
}

/// The content of an `m.room.encrypted` event using the `m.olm.v1.curve25519-aes-sha2` algorithm.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct OlmV1Curve25519AesSha2Content {
    /// A map from the recipient Curve25519 identity key to ciphertext information.
    pub ciphertext: BTreeMap<String, CiphertextInfo>,

    /// The Curve25519 key of the sender.
    pub sender_key: String,
}

impl OlmV1Curve25519AesSha2Content {
    /// Creates a new `OlmV1Curve25519AesSha2Content` with the given ciphertext and sender key.
    pub fn new(ciphertext: BTreeMap<String, CiphertextInfo>, sender_key: String) -> Self {
        Self { ciphertext, sender_key }
    }
}

/// Ciphertext information holding the ciphertext and message type.
///
/// Used for messages encrypted with the `m.olm.v1.curve25519-aes-sha2` algorithm.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct CiphertextInfo {
    /// The encrypted payload.
    pub body: String,

    /// The Olm message type.
    #[serde(rename = "type")]
    pub message_type: UInt,
}

impl CiphertextInfo {
    /// Creates a new `CiphertextInfo` with the given body and type.
    pub fn new(body: String, message_type: UInt) -> Self {
        Self { body, message_type }
    }
}

/// The content of an `m.room.encrypted` event using the `m.megolm.v1.aes-sha2` algorithm.
///
/// To create an instance of this type, first create a `MegolmV1AesSha2ContentInit` and convert it
/// via `MegolmV1AesSha2Content::from` / `.into()`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct MegolmV1AesSha2Content {
    /// The encrypted content of the event.
    pub ciphertext: String,

    /// The Curve25519 key of the sender.
    #[deprecated = "this field still needs to be sent but should not be used when received"]
    pub sender_key: String,

    /// The ID of the sending device.
    #[deprecated = "this field still needs to be sent but should not be used when received"]
    pub device_id: OwnedDeviceId,

    /// The ID of the session used to encrypt the message.
    pub session_id: String,
}

/// Mandatory initial set of fields of `MegolmV1AesSha2Content`.
///
/// This struct will not be updated even if additional fields are added to `MegolmV1AesSha2Content`
/// in a new (non-breaking) release of the Matrix specification.
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct MegolmV1AesSha2ContentInit {
    /// The encrypted content of the event.
    pub ciphertext: String,

    /// The Curve25519 key of the sender.
    pub sender_key: String,

    /// The ID of the sending device.
    pub device_id: OwnedDeviceId,

    /// The ID of the session used to encrypt the message.
    pub session_id: String,
}

impl From<MegolmV1AesSha2ContentInit> for MegolmV1AesSha2Content {
    /// Creates a new `MegolmV1AesSha2Content` from the given init struct.
    fn from(init: MegolmV1AesSha2ContentInit) -> Self {
        let MegolmV1AesSha2ContentInit { ciphertext, sender_key, device_id, session_id } = init;
        #[allow(deprecated)]
        Self { ciphertext, sender_key, device_id, session_id }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use js_int::uint;
    use ruma_common::{owned_event_id, serde::Raw};
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{
        EncryptedEventScheme, InReplyTo, MegolmV1AesSha2ContentInit, Relation,
        RoomEncryptedEventContent,
    };

    #[test]
    fn serialization() {
        let key_verification_start_content = RoomEncryptedEventContent {
            scheme: EncryptedEventScheme::MegolmV1AesSha2(
                MegolmV1AesSha2ContentInit {
                    ciphertext: "ciphertext".into(),
                    sender_key: "sender_key".into(),
                    device_id: "device_id".into(),
                    session_id: "session_id".into(),
                }
                .into(),
            ),
            relates_to: Some(Relation::Reply {
                in_reply_to: InReplyTo { event_id: owned_event_id!("$h29iv0s8:example.com") },
            }),
        };

        let json_data = json!({
            "algorithm": "m.megolm.v1.aes-sha2",
            "ciphertext": "ciphertext",
            "sender_key": "sender_key",
            "device_id": "device_id",
            "session_id": "session_id",
            "m.relates_to": {
                "m.in_reply_to": {
                    "event_id": "$h29iv0s8:example.com"
                }
            },
        });

        assert_eq!(to_json_value(&key_verification_start_content).unwrap(), json_data);
    }

    #[test]
    #[allow(deprecated)]
    fn deserialization() {
        let json_data = json!({
            "algorithm": "m.megolm.v1.aes-sha2",
            "ciphertext": "ciphertext",
            "sender_key": "sender_key",
            "device_id": "device_id",
            "session_id": "session_id",
            "m.relates_to": {
                "m.in_reply_to": {
                    "event_id": "$h29iv0s8:example.com"
                }
            },
        });

        let content: RoomEncryptedEventContent = from_json_value(json_data).unwrap();

        assert_matches!(content.scheme, EncryptedEventScheme::MegolmV1AesSha2(scheme));
        assert_eq!(scheme.ciphertext, "ciphertext");
        assert_eq!(scheme.sender_key, "sender_key");
        assert_eq!(scheme.device_id, "device_id");
        assert_eq!(scheme.session_id, "session_id");

        assert_matches!(content.relates_to, Some(Relation::Reply { in_reply_to }));
        assert_eq!(in_reply_to.event_id, "$h29iv0s8:example.com");
    }

    #[test]
    fn deserialization_olm() {
        let json_data = json!({
            "sender_key": "test_key",
            "ciphertext": {
                "test_curve_key": {
                    "body": "encrypted_body",
                    "type": 1
                }
            },
            "algorithm": "m.olm.v1.curve25519-aes-sha2"
        });
        let content: RoomEncryptedEventContent = from_json_value(json_data).unwrap();

        assert_matches!(content.scheme, EncryptedEventScheme::OlmV1Curve25519AesSha2(c));
        assert_eq!(c.sender_key, "test_key");
        assert_eq!(c.ciphertext.len(), 1);
        assert_eq!(c.ciphertext["test_curve_key"].body, "encrypted_body");
        assert_eq!(c.ciphertext["test_curve_key"].message_type, uint!(1));

        assert_matches!(content.relates_to, None);
    }

    #[test]
    fn deserialization_failure() {
        from_json_value::<Raw<RoomEncryptedEventContent>>(
            json!({ "algorithm": "m.megolm.v1.aes-sha2" }),
        )
        .unwrap()
        .deserialize()
        .unwrap_err();
    }
}
