//! Types for the [`m.room_key.withheld`] event.
//!
//! [`m.room_key.withheld`]: https://spec.matrix.org/latest/client-server-api/#mroom_keywithheld

use std::borrow::Cow;

use ruma_common::{
    EventEncryptionAlgorithm, RoomId,
    serde::{Base64, JsonObject, from_raw_json_value},
};
use ruma_macros::{EventContent, StringEnum};
use serde::{Deserialize, Serialize, de};
use serde_json::value::RawValue as RawJsonValue;

use crate::PrivOwnedStr;

/// The content of an [`m.room_key.withheld`] event.
///
/// Typically encrypted as an `m.room.encrypted` event, then sent as a to-device event.
///
/// [`m.room_key.withheld`]: https://spec.matrix.org/latest/client-server-api/#mroom_keywithheld
#[derive(Clone, Debug, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "m.room_key.withheld", kind = ToDevice)]
pub struct ToDeviceRoomKeyWithheldEventContent {
    /// The encryption algorithm the key in this event is to be used with.
    ///
    /// Must be `m.megolm.v1.aes-sha2`.
    pub algorithm: EventEncryptionAlgorithm,

    /// A machine-readable code for why the megolm key was not sent.
    #[serde(flatten)]
    pub code: RoomKeyWithheldCodeInfo,

    /// A human-readable reason for why the key was not sent.
    ///
    /// The receiving client should only use this string if it does not understand the code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// The unpadded base64-encoded device curve25519 key of the event's sender.
    pub sender_key: Base64,
}

impl ToDeviceRoomKeyWithheldEventContent {
    /// Creates a new `ToDeviceRoomKeyWithheldEventContent` with the given algorithm, code and
    /// sender key.
    pub fn new(
        algorithm: EventEncryptionAlgorithm,
        code: RoomKeyWithheldCodeInfo,
        sender_key: Base64,
    ) -> Self {
        Self { algorithm, code, reason: None, sender_key }
    }
}

impl<'de> Deserialize<'de> for ToDeviceRoomKeyWithheldEventContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ToDeviceRoomKeyWithheldEventContentDeHelper {
            algorithm: EventEncryptionAlgorithm,
            reason: Option<String>,
            sender_key: Base64,
        }

        let json = Box::<RawJsonValue>::deserialize(deserializer)?;

        let ToDeviceRoomKeyWithheldEventContentDeHelper { algorithm, reason, sender_key } =
            from_raw_json_value(&json)?;
        let code = from_raw_json_value(&json)?;

        Ok(Self { algorithm, code, reason, sender_key })
    }
}

/// The possible codes for why a megolm key was not sent, and the associated session data.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[serde(tag = "code")]
pub enum RoomKeyWithheldCodeInfo {
    /// `m.blacklisted`
    ///
    /// The user or device was blacklisted.
    #[serde(rename = "m.blacklisted")]
    Blacklisted(Box<RoomKeyWithheldSessionData>),

    /// `m.unverified`
    ///
    /// The user or device was not verified, and the sender is only sharing keys with verified
    /// users or devices.
    #[serde(rename = "m.unverified")]
    Unverified(Box<RoomKeyWithheldSessionData>),

    /// `m.unauthorised`
    ///
    /// The user or device is not allowed to have the key. For example, this could be sent in
    /// response to a key request if the user or device was not in the room when the original
    /// message was sent.
    #[serde(rename = "m.unauthorised")]
    Unauthorized(Box<RoomKeyWithheldSessionData>),

    /// `m.unavailable`
    ///
    /// Sent in reply to a key request if the device that the key is requested from does not have
    /// the requested key.
    #[serde(rename = "m.unavailable")]
    Unavailable(Box<RoomKeyWithheldSessionData>),

    /// `m.no_olm`
    ///
    /// An olm session could not be established.
    #[serde(rename = "m.no_olm")]
    NoOlm,

    #[doc(hidden)]
    #[serde(untagged)]
    _Custom(Box<CustomRoomKeyWithheldCodeInfo>),
}

impl RoomKeyWithheldCodeInfo {
    /// Get the code of this `RoomKeyWithheldCodeInfo`.
    pub fn code(&self) -> RoomKeyWithheldCode {
        match self {
            Self::Blacklisted(_) => RoomKeyWithheldCode::Blacklisted,
            Self::Unverified(_) => RoomKeyWithheldCode::Unverified,
            Self::Unauthorized(_) => RoomKeyWithheldCode::Unauthorized,
            Self::Unavailable(_) => RoomKeyWithheldCode::Unavailable,
            Self::NoOlm => RoomKeyWithheldCode::NoOlm,
            Self::_Custom(info) => info.code.as_str().into(),
        }
    }
}

impl<'de> Deserialize<'de> for RoomKeyWithheldCodeInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        struct ExtractCode<'a> {
            #[serde(borrow)]
            code: Cow<'a, str>,
        }

        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let ExtractCode { code } = from_raw_json_value(&json)?;

        Ok(match code.as_ref() {
            "m.blacklisted" => Self::Blacklisted(from_raw_json_value(&json)?),
            "m.unverified" => Self::Unverified(from_raw_json_value(&json)?),
            "m.unauthorised" => Self::Unauthorized(from_raw_json_value(&json)?),
            "m.unavailable" => Self::Unavailable(from_raw_json_value(&json)?),
            "m.no_olm" => Self::NoOlm,
            _ => Self::_Custom(from_raw_json_value(&json)?),
        })
    }
}

/// The session data associated to a withheld room key.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct RoomKeyWithheldSessionData {
    /// The room for the key.
    pub room_id: RoomId,

    /// The session ID of the key.
    pub session_id: String,
}

impl RoomKeyWithheldSessionData {
    /// Construct a new `RoomKeyWithheldSessionData` with the given room ID and session ID.
    pub fn new(room_id: RoomId, session_id: String) -> Self {
        Self { room_id, session_id }
    }
}

/// The payload for a custom room key withheld code.
#[doc(hidden)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CustomRoomKeyWithheldCodeInfo {
    /// A custom code.
    code: String,

    /// Remaining event content.
    #[serde(flatten)]
    data: JsonObject,
}

/// The possible codes for why a megolm key was not sent.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, StringEnum)]
#[ruma_enum(rename_all(prefix = "m.", rule = "snake_case"))]
#[non_exhaustive]
pub enum RoomKeyWithheldCode {
    /// `m.blacklisted`
    ///
    /// The user or device was blacklisted.
    Blacklisted,

    /// `m.unverified`
    ///
    /// The user or device was not verified, and the sender is only sharing keys with verified
    /// users or devices.
    Unverified,

    /// `m.unauthorised`
    ///
    /// The user or device is not allowed to have the key. For example, this could be sent in
    /// response to a key request if the user or device was not in the room when the original
    /// message was sent.
    Unauthorized,

    /// `m.unavailable`
    ///
    /// Sent in reply to a key request if the device that the key is requested from does not have
    /// the requested key.
    Unavailable,

    /// `m.no_olm`
    ///
    /// An olm session could not be established.
    NoOlm,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use ruma_common::{
        EventEncryptionAlgorithm, canonical_json::assert_to_canonical_json_eq, room_id,
        serde::Base64,
    };
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{
        RoomKeyWithheldCodeInfo, RoomKeyWithheldSessionData, ToDeviceRoomKeyWithheldEventContent,
    };

    const PUBLIC_KEY: &[u8] = b"key";
    const BASE64_ENCODED_PUBLIC_KEY: &str = "a2V5";

    #[test]
    fn serialization_no_olm() {
        let content = ToDeviceRoomKeyWithheldEventContent::new(
            EventEncryptionAlgorithm::MegolmV1AesSha2,
            RoomKeyWithheldCodeInfo::NoOlm,
            Base64::new(PUBLIC_KEY.to_owned()),
        );

        assert_to_canonical_json_eq!(
            content,
            json!({
                "algorithm": "m.megolm.v1.aes-sha2",
                "code": "m.no_olm",
                "sender_key": BASE64_ENCODED_PUBLIC_KEY,
            })
        );
    }

    #[test]
    fn serialization_blacklisted() {
        let room_id = room_id!("!roomid:localhost");
        let content = ToDeviceRoomKeyWithheldEventContent::new(
            EventEncryptionAlgorithm::MegolmV1AesSha2,
            RoomKeyWithheldCodeInfo::Blacklisted(
                RoomKeyWithheldSessionData::new(room_id.clone(), "unique_id".to_owned()).into(),
            ),
            Base64::new(PUBLIC_KEY.to_owned()),
        );

        assert_to_canonical_json_eq!(
            content,
            json!({
                "algorithm": "m.megolm.v1.aes-sha2",
                "code": "m.blacklisted",
                "sender_key": BASE64_ENCODED_PUBLIC_KEY,
                "room_id": room_id,
                "session_id": "unique_id",
            })
        );
    }

    #[test]
    fn deserialization_no_olm() {
        let json = json!({
            "algorithm": "m.megolm.v1.aes-sha2",
            "code": "m.no_olm",
            "sender_key": BASE64_ENCODED_PUBLIC_KEY,
            "reason": "Could not find an olm session",
        });

        let content = from_json_value::<ToDeviceRoomKeyWithheldEventContent>(json).unwrap();
        assert_eq!(content.algorithm, EventEncryptionAlgorithm::MegolmV1AesSha2);
        assert_eq!(content.sender_key, Base64::new(PUBLIC_KEY.to_owned()));
        assert_eq!(content.reason.as_deref(), Some("Could not find an olm session"));
        assert_matches!(content.code, RoomKeyWithheldCodeInfo::NoOlm);
    }

    #[test]
    fn deserialization_blacklisted() {
        let room_id = room_id!("!roomid:localhost");
        let json = json!({
            "algorithm": "m.megolm.v1.aes-sha2",
            "code": "m.blacklisted",
            "sender_key": BASE64_ENCODED_PUBLIC_KEY,
            "room_id": room_id,
            "session_id": "unique_id",
        });

        let content = from_json_value::<ToDeviceRoomKeyWithheldEventContent>(json).unwrap();
        assert_eq!(content.algorithm, EventEncryptionAlgorithm::MegolmV1AesSha2);
        assert_eq!(content.sender_key, Base64::new(PUBLIC_KEY.to_owned()));
        assert_eq!(content.reason, None);
        assert_matches!(content.code, RoomKeyWithheldCodeInfo::Blacklisted(session_data));
        assert_eq!(session_data.room_id, room_id);
        assert_eq!(session_data.session_id, "unique_id");
    }

    #[test]
    fn custom_room_key_withheld_code_info_round_trip() {
        let room_id = room_id!("!roomid:localhost");
        let json = json!({
            "algorithm": "m.megolm.v1.aes-sha2",
            "code": "dev.ruma.custom_code",
            "sender_key": BASE64_ENCODED_PUBLIC_KEY,
            "room_id": room_id,
            "key": "value",
        });

        let content = from_json_value::<ToDeviceRoomKeyWithheldEventContent>(json.clone()).unwrap();
        assert_eq!(content.code.code().as_str(), "dev.ruma.custom_code");

        assert_eq!(to_json_value(&content).unwrap(), json);
    }
}
