//! [PUT /_matrix/federation/v1/send/{txnId}](https://matrix.org/docs/spec/server_server/r0.1.3#put-matrix-federation-v1-send-txnid)

use std::collections::BTreeMap;

use js_int::UInt;
use ruma_common::{encryption::DeviceKeys, presence::PresenceState};
use ruma_events::{from_raw_json_value, receipt::Receipt};
use ruma_identifiers::{DeviceIdBox, EventId, RoomId, UserId};
use serde::{de, Deserialize, Serialize};
use serde_json::{value::RawValue as RawJsonValue, Value as JsonValue};

// There is one more edu_type synapse recognizes with the note:
// FIXME: switch to "m.signing_key_update" when MSC1756 is merged into the
// spec from "org.matrix.signing_key_update"

/// Type for passing ephemeral data to homeservers.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "edu_type", content = "content")]
pub enum Edu {
    /// An EDU representing presence updates for users of the sending homeserver.
    #[serde(rename = "m.presence")]
    Presence(PresenceContent),

    /// An EDU representing receipt updates for users of the sending homeserver.
    #[serde(rename = "m.receipt")]
    Receipt(ReceiptContent),

    /// A typing notification EDU for a user in a room.
    #[serde(rename = "m.typing")]
    Typing(TypingContent),

    /// An EDU that lets servers push details to each other when one of their users adds
    // a new device to their account, required for E2E encryption to correctly target the
    // current set of devices for a given user.
    #[serde(rename = "m.device_list_update")]
    DeviceListUpdate(DeviceListUpdateContent),

    /// An EDU that lets servers push send events directly to a specific device on a
    // remote server - for instance, to maintain an Olm E2E encrypted message channel
    // between a local and remote device.
    #[serde(rename = "m.direct_to_device")]
    DirectToDevice(DirectDeviceContent),

    #[doc(hidden)]
    Custom(JsonValue),
}

#[derive(Debug, Deserialize)]
struct EduTpeDeHelper {
    /// The message type field
    edu_type: String,
    content: Box<RawJsonValue>,
}

impl<'de> de::Deserialize<'de> for Edu {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let EduTpeDeHelper { edu_type, ref content } = from_raw_json_value(&json)?;

        Ok(match edu_type.as_ref() {
            "m.presence" => Self::Presence(from_raw_json_value(content)?),
            "m.receipt" => Self::Receipt(from_raw_json_value(content)?),
            "m.typing" => Self::Typing(from_raw_json_value(content)?),
            "m.device_list_update" => Self::DeviceListUpdate(from_raw_json_value(content)?),
            "m.direct_to_device" => Self::DirectToDevice(from_raw_json_value(content)?),
            _ => Self::Custom(from_raw_json_value(content)?),
        })
    }
}

/// The content for "m.presence" Edu.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PresenceContent {
    /// A list of presence updates that the receiving server is likely to be interested in.
    pub push: Vec<PresenceUpdate>,
}

/// An update to the presence of a user.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PresenceUpdate {
    /// The user ID this presence EDU is for.
    pub user_id: UserId,

    /// The presence of the user.
    pub presence: PresenceState,

    /// An optional description to accompany the presence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_msg: Option<String>,

    /// The number of milliseconds that have elapsed since the user last did something.
    pub last_active_ago: UInt,

    /// Whether or not the user is currently active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currently_active: Option<bool>,
}

/// The content for "m.receipt" Edu.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReceiptContent {
    ///  Receipts for a particular room.
    #[serde(flatten)]
    pub receipts: BTreeMap<RoomId, ReceiptMap>,
}

/// Mapping between user and `ReceiptData`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReceiptMap {
    /// Read receipts for users in the room.
    #[serde(rename = "m.read")]
    pub read: BTreeMap<UserId, ReceiptData>,
}

/// Metadata about the event that was last read and when.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReceiptData {
    /// Metadata for the read receipt.
    pub data: Receipt,

    /// The extremity event ID the user has read up to.
    pub event_ids: Vec<EventId>,
}

/// The content for "m.typing" Edu.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TypingContent {
    /// The typing notification.
    pub notifi: TypingNotice,
}

/// Notification of a user typing.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TypingNotice {
    /// The room where the user's typing status has been updated.
    pub room_id: RoomId,

    /// The user ID that has had their typing status changed.
    pub user_id: UserId,

    /// Whether the user is typing in the room or not.
    pub typing: bool,
}

/// The description of the direct-to- device message.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeviceListUpdateContent {
    /// The user ID who owns the device.
    pub user_id: UserId,

    /// The ID of the device whose details are changing.
    pub device_id: DeviceIdBox,

    /// The public human-readable name of this device. Will be absent if the device has no name.
    pub device_display_name: String,

    /// An ID sent by the server for this update, unique for a given user_id.
    pub stream_id: UInt,

    /// The stream_ids of any prior m.device_list_update EDUs sent for this user
    // which have not been referred to already in an EDU's prev_id field.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prev_id: Vec<UInt>,

    /// True if the server is announcing that this device has been deleted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted: Option<bool>,

    /// The updated identity keys (if any) for this device.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keys: Option<DeviceKeys>,
}

/// The description of the direct-to- device message.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DirectDeviceContent {
    /// The user ID of the sender.
    pub sender: UserId,

    /// Event type for the message.
    #[serde(rename = "type")]
    pub ev_type: String,

    /// Unique utf8 string ID for the message, used for idempotence.
    pub message_id: String,

    // TODO: https://matrix.org/docs/spec/server_server/r0.1.4#m-direct-to-device-schema
    /// The contents of the messages to be sent. These are arranged in a map
    /// of user IDs to a map of device IDs to message bodies. The device ID may
    /// also be *, meaning all known devices for the user.
    pub messages: BTreeMap<String, BTreeMap<DeviceIdBox, JsonValue>>,
}

#[cfg(test)]
mod test {
    use matches::assert_matches;
    use ruma_identifiers::{room_id, user_id};
    use serde_json::json;

    use super::*;

    #[test]
    fn device_list_update() {
        let json = json!({
                "content": {
                    "deleted": false,
                    "device_display_name": "Mobile",
                    "device_id": "QBUAZIFURK",
                    "keys": {
                        "algorithms": [
                            "m.olm.v1.curve25519-aes-sha2",
                            "m.megolm.v1.aes-sha2"
                        ],
                        "device_id": "JLAFKJWSCS",
                        "keys": {
                            "curve25519:JLAFKJWSCS": "3C5BFWi2Y8MaVvjM8M22DBmh24PmgR0nPvJOIArzgyI",
                            "ed25519:JLAFKJWSCS": "lEuiRJBit0IG6nUf5pUzWTUEsRVVe/HJkoKuEww9ULI"
                        },
                        "signatures": {
                            "@alice:example.com": {
                                "ed25519:JLAFKJWSCS": "dSO80A01XiigH3uBiDVx/EjzaoycHcjq9lfQX0uWsqxl2giMIiSPR8a4d291W1ihKJL/a+myXS367WT6NAIcBA"
                            }
                        },
                        "user_id": "@alice:example.com"
                    },
                    "stream_id": 6,
                    "user_id": "@john:example.com"
                },
            "edu_type": "m.device_list_update"
        });

        let edu = serde_json::from_value::<Edu>(json).unwrap();
        assert_matches!(
            &edu,
            Edu::DeviceListUpdate(DeviceListUpdateContent {
                user_id,
                device_id,
                device_display_name,
                stream_id,
                prev_id,
                deleted,
                keys
            }) if user_id == &user_id!("@john:example.com")
                && prev_id.is_empty()
        );

        assert_eq!(
            serde_json::to_string(&edu).unwrap(),
            r#"{"edu_type":"m.device_list_update","content":{"user_id":"@john:example.com","device_id":"QBUAZIFURK","device_display_name":"Mobile","stream_id":6,"deleted":false,"keys":{"user_id":"@alice:example.com","device_id":"JLAFKJWSCS","algorithms":["m.olm.v1.curve25519-aes-sha2","m.megolm.v1.aes-sha2"],"keys":{"curve25519:JLAFKJWSCS":"3C5BFWi2Y8MaVvjM8M22DBmh24PmgR0nPvJOIArzgyI","ed25519:JLAFKJWSCS":"lEuiRJBit0IG6nUf5pUzWTUEsRVVe/HJkoKuEww9ULI"},"signatures":{"@alice:example.com":{"ed25519:JLAFKJWSCS":"dSO80A01XiigH3uBiDVx/EjzaoycHcjq9lfQX0uWsqxl2giMIiSPR8a4d291W1ihKJL/a+myXS367WT6NAIcBA"}}}}}"#
        );
    }

    #[test]
    fn receipt_edu() {
        let json = json!({
            "content": {
                "!some_room:example.org": {
                    "m.read": {
                        "@john:matrix.org": {
                            "data": {
                                "ts": 1533358
                            },
                            "event_ids": [
                                "$read_this_event:matrix.org"
                            ]
                        }
                    }
                }
            },
            "edu_type": "m.receipt"
        });

        let edu = serde_json::from_value::<Edu>(json).unwrap();
        assert_matches!(
            &edu,
            Edu::Receipt(ReceiptContent { receipts })
                if receipts.get(&room_id!("!some_room:example.org")).is_some()
        );

        assert_eq!(
            serde_json::to_string(&edu).unwrap(),
            r#"{"edu_type":"m.receipt","content":{"!some_room:example.org":{"m.read":{"@john:matrix.org":{"data":{"ts":1533358},"event_ids":["$read_this_event:matrix.org"]}}}}}"#
        );
    }
}
