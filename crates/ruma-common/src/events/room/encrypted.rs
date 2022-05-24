//! Types for the [`m.room.encrypted`] event.
//!
//! [`m.room.encrypted`]: https://spec.matrix.org/v1.2/client-server-api/#mroomencrypted

use std::collections::BTreeMap;

use js_int::UInt;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::message::{self, InReplyTo};
use crate::{OwnedDeviceId, OwnedEventId};

mod relation_serde;

/// The content of an `m.room.encrypted` event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.encrypted", kind = MessageLike, kind = ToDevice)]
pub struct RoomEncryptedEventContent {
    /// Algorithm-specific fields.
    #[serde(flatten)]
    pub scheme: EncryptedEventScheme,

    /// Information about related events.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
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
    #[cfg(feature = "unstable-msc2676")]
    Replacement(Replacement),

    /// A reference to another event.
    Reference(Reference),

    /// An annotation to an event.
    #[cfg(feature = "unstable-msc2677")]
    Annotation(Annotation),

    /// An event that belongs to a thread.
    #[cfg(feature = "unstable-msc3440")]
    Thread(Thread),

    #[doc(hidden)]
    _Custom,
}

impl From<message::Relation> for Relation {
    fn from(rel: message::Relation) -> Self {
        match rel {
            message::Relation::Reply { in_reply_to } => Self::Reply { in_reply_to },
            #[cfg(feature = "unstable-msc2676")]
            message::Relation::Replacement(re) => {
                Self::Replacement(Replacement { event_id: re.event_id })
            }
            #[cfg(feature = "unstable-msc3440")]
            message::Relation::Thread(t) => Self::Thread(Thread {
                event_id: t.event_id,
                in_reply_to: t.in_reply_to,
                is_falling_back: t.is_falling_back,
            }),
            message::Relation::_Custom => Self::_Custom,
        }
    }
}

/// The event this relation belongs to replaces another event.
///
/// In contrast to [`message::Replacement`](super::message::Replacement), this struct doesn't
/// store the new content, since that is part of the encrypted content of an `m.room.encrypted`
/// events.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg(feature = "unstable-msc2676")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Replacement {
    /// The ID of the event being replacing.
    pub event_id: OwnedEventId,
}

/// A reference to another event.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Reference {
    /// The event we are referencing.
    pub event_id: OwnedEventId,
}

impl Reference {
    /// Creates a new `Reference` with the given event ID.
    pub fn new(event_id: OwnedEventId) -> Self {
        Self { event_id }
    }
}

/// An annotation for an event.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg(feature = "unstable-msc2677")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Annotation {
    /// The event that is being annotated.
    pub event_id: OwnedEventId,

    /// The annotation.
    pub key: String,
}

#[cfg(feature = "unstable-msc2677")]
impl Annotation {
    /// Creates a new `Annotation` with the given event ID and key.
    pub fn new(event_id: OwnedEventId, key: String) -> Self {
        Self { event_id, key }
    }
}

/// A thread relation for an event.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg(feature = "unstable-msc3440")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Thread {
    /// The ID of the root message in the thread.
    pub event_id: OwnedEventId,

    /// A reply relation.
    ///
    /// If this event is a reply and belongs to a thread, this points to the message that is being
    /// replied to, and `is_falling_back` must be set to `false`.
    ///
    /// If this event is not a reply, this is used as a fallback mechanism for clients that do not
    /// support threads. This should point to the latest message-like event in the thread and
    /// `is_falling_back` must be set to `true`.
    pub in_reply_to: InReplyTo,

    /// Whether the `m.in_reply_to` field is a fallback for older clients or a real reply in a
    /// thread.
    pub is_falling_back: bool,
}

#[cfg(feature = "unstable-msc3440")]
impl Thread {
    /// Convenience method to create a regular `Thread` with the given event ID and latest
    /// message-like event ID.
    pub fn plain(event_id: OwnedEventId, latest_event_id: OwnedEventId) -> Self {
        Self { event_id, in_reply_to: InReplyTo::new(latest_event_id), is_falling_back: false }
    }

    /// Convenience method to create a reply `Thread` with the given event ID and replied-to event
    /// ID.
    pub fn reply(event_id: OwnedEventId, reply_to_event_id: OwnedEventId) -> Self {
        Self { event_id, in_reply_to: InReplyTo::new(reply_to_event_id), is_falling_back: true }
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
    #[cfg_attr(
        feature = "unstable-msc3700",
        deprecated = "this field still needs to be sent but should not be used when received"
    )]
    pub sender_key: String,

    /// The ID of the sending device.
    #[cfg_attr(
        feature = "unstable-msc3700",
        deprecated = "this field still needs to be sent but should not be used when received"
    )]
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
    use crate::{event_id, serde::Raw};
    use assert_matches::assert_matches;
    use js_int::uint;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{
        EncryptedEventScheme, InReplyTo, MegolmV1AesSha2Content, Relation,
        RoomEncryptedEventContent,
    };

    #[test]
    fn serialization() {
        let key_verification_start_content = RoomEncryptedEventContent {
            scheme: EncryptedEventScheme::MegolmV1AesSha2(MegolmV1AesSha2Content {
                ciphertext: "ciphertext".into(),
                sender_key: "sender_key".into(),
                device_id: "device_id".into(),
                session_id: "session_id".into(),
            }),
            relates_to: Some(Relation::Reply {
                in_reply_to: InReplyTo { event_id: event_id!("$h29iv0s8:example.com").to_owned() },
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

        assert_matches!(
            content.scheme,
            EncryptedEventScheme::MegolmV1AesSha2(MegolmV1AesSha2Content {
                ciphertext,
                sender_key,
                device_id,
                session_id,
            }) if ciphertext == "ciphertext"
                && sender_key == "sender_key"
                && device_id == "device_id"
                && session_id == "session_id"
        );

        assert_matches!(
            content.relates_to,
            Some(Relation::Reply { in_reply_to })
                if in_reply_to.event_id == event_id!("$h29iv0s8:example.com")
        );
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

        match content.scheme {
            EncryptedEventScheme::OlmV1Curve25519AesSha2(c) => {
                assert_eq!(c.sender_key, "test_key");
                assert_eq!(c.ciphertext.len(), 1);
                assert_eq!(c.ciphertext["test_curve_key"].body, "encrypted_body");
                assert_eq!(c.ciphertext["test_curve_key"].message_type, uint!(1));
            }
            _ => panic!("Wrong content type, expected a OlmV1 content"),
        }
    }

    #[test]
    fn deserialization_failure() {
        assert!(from_json_value::<Raw<RoomEncryptedEventContent>>(
            json!({ "algorithm": "m.megolm.v1.aes-sha2" })
        )
        .unwrap()
        .deserialize()
        .is_err());
    }
}
