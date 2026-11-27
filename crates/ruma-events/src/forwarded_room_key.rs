//! Types for the [`m.forwarded_room_key`] event.
//!
//! [`m.forwarded_room_key`]: https://spec.matrix.org/v1.19/client-server-api/#mforwarded_room_key

use ruma_common::{EventEncryptionAlgorithm, OwnedRoomId};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::room_key::withheld::{RoomKeyWithheldCode, ToDeviceRoomKeyWithheldEventContent};

/// The content of an `m.forwarded_room_key` event.
///
/// To create an instance of this type, first create a `ToDeviceForwardedRoomKeyEventContentInit`
/// and convert it via `ToDeviceForwardedRoomKeyEventContent::from` / `.into()`.
///
/// If a portion of the session was withheld, set the [`withheld`](Self::withheld) field after the
/// conversion. A [`ForwardedRoomKeyWithheldInfo`] can be created from the received
/// [`ToDeviceRoomKeyWithheldEventContent`] with `ForwardedRoomKeyWithheldInfo::from`.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "m.forwarded_room_key", kind = ToDevice)]
pub struct ToDeviceForwardedRoomKeyEventContent {
    /// The encryption algorithm the key in this event is to be used with.
    pub algorithm: EventEncryptionAlgorithm,

    /// The room where the key is used.
    pub room_id: OwnedRoomId,

    /// The Curve25519 key of the device which initiated the session originally.
    pub sender_key: String,

    /// The ID of the session that the key is for.
    pub session_id: String,

    /// The key to be exchanged.
    pub session_key: String,

    /// The Ed25519 key of the device which initiated the session originally.
    ///
    /// It is "claimed" because the receiving device has no way to tell that the original
    /// room_key actually came from a device which owns the private part of this key unless
    /// they have done device verification.
    pub sender_claimed_ed25519_key: String,

    /// Chain of Curve25519 keys.
    ///
    /// It starts out empty, but each time the key is forwarded to another device, the
    /// previous sender in the chain is added to the end of the list. For example, if the
    /// key is forwarded from A to B to C, this field is empty between A and B, and contains
    /// A's Curve25519 key between B and C.
    pub forwarding_curve25519_key_chain: Vec<String>,

    /// Indicates that the key cannot be used to decrypt all the messages from the session because
    /// a portion of the session was withheld.
    ///
    /// This contains the `code` and `reason` properties from the `m.room_key.withheld` message
    /// that was received by the sender of this message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub withheld: Option<ForwardedRoomKeyWithheldInfo>,
}

/// Initial set of fields of `ToDeviceForwardedRoomKeyEventContent`.
///
/// This struct will not be updated even if additional fields are added to
/// `ToDeviceForwardedRoomKeyEventContent` in a new (non-breaking) release of the Matrix
/// specification.
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct ToDeviceForwardedRoomKeyEventContentInit {
    /// The encryption algorithm the key in this event is to be used with.
    pub algorithm: EventEncryptionAlgorithm,

    /// The room where the key is used.
    pub room_id: OwnedRoomId,

    /// The Curve25519 key of the device which initiated the session originally.
    pub sender_key: String,

    /// The ID of the session that the key is for.
    pub session_id: String,

    /// The key to be exchanged.
    pub session_key: String,

    /// The Ed25519 key of the device which initiated the session originally.
    ///
    /// It is "claimed" because the receiving device has no way to tell that the original
    /// room_key actually came from a device which owns the private part of this key unless
    /// they have done device verification.
    pub sender_claimed_ed25519_key: String,

    /// Chain of Curve25519 keys.
    ///
    /// It starts out empty, but each time the key is forwarded to another device, the
    /// previous sender in the chain is added to the end of the list. For example, if the
    /// key is forwarded from A to B to C, this field is empty between A and B, and contains
    /// A's Curve25519 key between B and C.
    pub forwarding_curve25519_key_chain: Vec<String>,
}

impl From<ToDeviceForwardedRoomKeyEventContentInit> for ToDeviceForwardedRoomKeyEventContent {
    fn from(init: ToDeviceForwardedRoomKeyEventContentInit) -> Self {
        Self {
            algorithm: init.algorithm,
            room_id: init.room_id,
            sender_key: init.sender_key,
            session_id: init.session_id,
            session_key: init.session_key,
            sender_claimed_ed25519_key: init.sender_claimed_ed25519_key,
            forwarding_curve25519_key_chain: init.forwarding_curve25519_key_chain,
            withheld: None,
        }
    }
}

/// Information about why a portion of a forwarded room key session was withheld.
///
/// This is copied from the `m.room_key.withheld` message that was received by the sender of the
/// `m.forwarded_room_key` event.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct ForwardedRoomKeyWithheldInfo {
    /// The machine-readable code for why the key was withheld.
    pub code: RoomKeyWithheldCode,

    /// A human-readable reason for why the key was withheld.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl ForwardedRoomKeyWithheldInfo {
    /// Creates a new `ForwardedRoomKeyWithheldInfo` with the given code.
    pub fn new(code: RoomKeyWithheldCode) -> Self {
        Self { code, reason: None }
    }
}

impl From<&ToDeviceRoomKeyWithheldEventContent> for ForwardedRoomKeyWithheldInfo {
    fn from(content: &ToDeviceRoomKeyWithheldEventContent) -> Self {
        Self { code: content.code.code(), reason: content.reason.clone() }
    }
}

#[cfg(test)]
mod tests {
    use ruma_common::{
        EventEncryptionAlgorithm, canonical_json::assert_to_canonical_json_eq, owned_room_id,
        serde::Base64,
    };
    use serde_json::{from_value as from_json_value, json};

    use super::{
        ForwardedRoomKeyWithheldInfo, ToDeviceForwardedRoomKeyEventContent,
        ToDeviceForwardedRoomKeyEventContentInit,
    };
    use crate::room_key::withheld::{
        RoomKeyWithheldCode, RoomKeyWithheldCodeInfo, RoomKeyWithheldSessionData,
        ToDeviceRoomKeyWithheldEventContent,
    };

    fn content() -> ToDeviceForwardedRoomKeyEventContent {
        ToDeviceForwardedRoomKeyEventContentInit {
            algorithm: EventEncryptionAlgorithm::MegolmV1AesSha2,
            room_id: owned_room_id!("!Cuyf34gef24t:localhost"),
            sender_key: "RF3s+E7RkTQTGF2d8Deol0FkQvgII2aJDf3/Jp5mxVU".to_owned(),
            session_id: "X3lUlvLELLYxeTx4yOVu6UDpasGEVO0Jbu+QFnm0cKQ".to_owned(),
            session_key: "AgAAAADxKHa9uFxcXzwYoNueL5Xqi69IkD4sni8Llf".to_owned(),
            sender_claimed_ed25519_key: "aj40p+aw64yPIdsxoog8jhPu9i7l7NcFRecuOQblE3Y".to_owned(),
            forwarding_curve25519_key_chain: vec![
                "hPQNcabIABgGnx3/ACv/jmMmiQHoeFfuLB17tzWp6Hw".to_owned(),
            ],
        }
        .into()
    }

    fn json() -> serde_json::Value {
        json!({
            "algorithm": "m.megolm.v1.aes-sha2",
            "room_id": "!Cuyf34gef24t:localhost",
            "sender_key": "RF3s+E7RkTQTGF2d8Deol0FkQvgII2aJDf3/Jp5mxVU",
            "session_id": "X3lUlvLELLYxeTx4yOVu6UDpasGEVO0Jbu+QFnm0cKQ",
            "session_key": "AgAAAADxKHa9uFxcXzwYoNueL5Xqi69IkD4sni8Llf",
            "sender_claimed_ed25519_key": "aj40p+aw64yPIdsxoog8jhPu9i7l7NcFRecuOQblE3Y",
            "forwarding_curve25519_key_chain": [
                "hPQNcabIABgGnx3/ACv/jmMmiQHoeFfuLB17tzWp6Hw",
            ],
        })
    }

    #[test]
    fn serialization_without_withheld() {
        assert_to_canonical_json_eq!(content(), json());
    }

    #[test]
    fn serialization_with_withheld() {
        let mut content = content();
        let mut withheld = ForwardedRoomKeyWithheldInfo::new(RoomKeyWithheldCode::Unverified);
        withheld.reason = Some("Device not verified".to_owned());
        content.withheld = Some(withheld);

        let mut json = json();
        json["withheld"] = json!({
            "code": "m.unverified",
            "reason": "Device not verified",
        });

        assert_to_canonical_json_eq!(content, json);
    }

    #[test]
    fn deserialization_without_withheld() {
        let content = from_json_value::<ToDeviceForwardedRoomKeyEventContent>(json()).unwrap();
        assert_eq!(content.room_id, "!Cuyf34gef24t:localhost");
        assert!(content.withheld.is_none());
    }

    #[test]
    fn deserialization_with_withheld() {
        let mut json = json();
        json["withheld"] = json!({
            "code": "m.unverified",
            "reason": "Device not verified",
        });

        let content = from_json_value::<ToDeviceForwardedRoomKeyEventContent>(json).unwrap();
        let withheld = content.withheld.unwrap();
        assert_eq!(withheld.code, RoomKeyWithheldCode::Unverified);
        assert_eq!(withheld.reason.as_deref(), Some("Device not verified"));
    }

    #[test]
    fn withheld_codes_round_trip() {
        for (code, s) in [
            (RoomKeyWithheldCode::Blacklisted, "m.blacklisted"),
            (RoomKeyWithheldCode::Unverified, "m.unverified"),
            (RoomKeyWithheldCode::Unauthorized, "m.unauthorised"),
            (RoomKeyWithheldCode::Unavailable, "m.unavailable"),
            (RoomKeyWithheldCode::NoOlm, "m.no_olm"),
            (RoomKeyWithheldCode::HistoryNotShared, "m.history_not_shared"),
        ] {
            let withheld = ForwardedRoomKeyWithheldInfo::new(code);
            let json = json!({ "code": s });

            assert_to_canonical_json_eq!(withheld.clone(), json.clone());
            assert_eq!(from_json_value::<ForwardedRoomKeyWithheldInfo>(json).unwrap(), withheld);
        }
    }

    #[test]
    fn withheld_custom_code_without_reason() {
        let json = json!({ "code": "org.example.custom" });

        let withheld = from_json_value::<ForwardedRoomKeyWithheldInfo>(json.clone()).unwrap();
        assert_eq!(withheld.code.as_str(), "org.example.custom");
        assert_eq!(withheld.reason, None);

        assert_to_canonical_json_eq!(withheld, json);
    }

    #[test]
    fn withheld_from_room_key_withheld_content() {
        let mut withheld_content = ToDeviceRoomKeyWithheldEventContent::new(
            EventEncryptionAlgorithm::MegolmV1AesSha2,
            RoomKeyWithheldCodeInfo::Unauthorized(Box::new(RoomKeyWithheldSessionData::new(
                owned_room_id!("!Cuyf34gef24t:localhost"),
                "X3lUlvLELLYxeTx4yOVu6UDpasGEVO0Jbu+QFnm0cKQ".to_owned(),
            ))),
            Base64::new(vec![0; 32]),
        );
        withheld_content.reason = Some("Not in the room".to_owned());

        let withheld = ForwardedRoomKeyWithheldInfo::from(&withheld_content);
        assert_eq!(withheld.code, RoomKeyWithheldCode::Unauthorized);
        assert_eq!(withheld.reason.as_deref(), Some("Not in the room"));

        assert_to_canonical_json_eq!(
            withheld,
            json!({ "code": "m.unauthorised", "reason": "Not in the room" })
        );
    }
}
