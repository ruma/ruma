//! Edu type and variant content structs.

use std::collections::BTreeMap;

use js_int::UInt;
use ruma_common::{
    encryption::DeviceKeys, presence::PresenceState, to_device::DeviceIdOrAllDevices,
};
use ruma_events::{from_raw_json_value, receipt::Receipt, EventType};
use ruma_identifiers::{DeviceIdBox, EventId, RoomId, UserId};
use serde::{de, Deserialize, Serialize};
use serde_json::{value::RawValue as RawJsonValue, Value as JsonValue};

// There is one more edu_type synapse recognizes with the note:
// FIXME: switch to "m.signing_key_update" when MSC1756 is merged into the
// spec from "org.matrix.signing_key_update"

/// Type for passing ephemeral data to homeservers.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
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
    /// a new device to their account, required for E2E encryption to correctly target the
    /// current set of devices for a given user.
    #[serde(rename = "m.device_list_update")]
    DeviceListUpdate(DeviceListUpdateContent),

    /// An EDU that lets servers push send events directly to a specific device on a
    /// remote server - for instance, to maintain an Olm E2E encrypted message channel
    /// between a local and remote device.
    #[serde(rename = "m.direct_to_device")]
    DirectToDevice(DirectDeviceContent),

    #[doc(hidden)]
    _Custom(JsonValue),
}

#[derive(Debug, Deserialize)]
struct EduDeHelper {
    /// The message type field
    edu_type: String,
    content: Box<RawJsonValue>,
}

impl<'de> Deserialize<'de> for Edu {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let EduDeHelper { edu_type, content } = from_raw_json_value(&json)?;

        Ok(match edu_type.as_ref() {
            "m.presence" => Self::Presence(from_raw_json_value(&content)?),
            "m.receipt" => Self::Receipt(from_raw_json_value(&content)?),
            "m.typing" => Self::Typing(from_raw_json_value(&content)?),
            "m.device_list_update" => Self::DeviceListUpdate(from_raw_json_value(&content)?),
            "m.direct_to_device" => Self::DirectToDevice(from_raw_json_value(&content)?),
            _ => Self::_Custom(from_raw_json_value(&content)?),
        })
    }
}

/// The content for "m.presence" Edu.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct PresenceContent {
    /// A list of presence updates that the receiving server is likely to be interested in.
    pub push: Vec<PresenceUpdate>,
}

impl PresenceContent {
    /// Creates a new `PresenceContent`.
    pub fn new(push: Vec<PresenceUpdate>) -> Self {
        Self { push }
    }
}

/// An update to the presence of a user.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct PresenceUpdate {
    /// The user ID this presence EDU is for.
    pub user_id: UserId,

    /// The presence of the user.
    pub presence: PresenceState,

    /// An optional description to accompany the presence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_msg: Option<String>,

    /// The number of milliseconds that have elapsed since the user last did something.
    pub last_active_ago: UInt,

    /// Whether or not the user is currently active. Defaults to false.
    #[serde(default)]
    pub currently_active: bool,
}

impl PresenceUpdate {
    /// Creates a new `PresenceUpdate` with default `status_msg` and `currently_active`.
    pub fn new(user_id: UserId, presence: PresenceState, last_activity: UInt) -> Self {
        Self {
            user_id,
            presence,
            last_active_ago: last_activity,
            status_msg: None,
            currently_active: false,
        }
    }
}

/// The content for "m.receipt" Edu.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ReceiptContent {
    /// Receipts for a particular room.
    #[serde(flatten)]
    pub receipts: BTreeMap<RoomId, ReceiptMap>,
}

impl ReceiptContent {
    /// Creates a new `ReceiptContent`.
    pub fn new(receipts: BTreeMap<RoomId, ReceiptMap>) -> Self {
        Self { receipts }
    }
}

/// Mapping between user and `ReceiptData`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ReceiptMap {
    /// Read receipts for users in the room.
    #[serde(rename = "m.read")]
    pub read: BTreeMap<UserId, ReceiptData>,
}

impl ReceiptMap {
    /// Creates a new `ReceiptMap`.
    pub fn new(read: BTreeMap<UserId, ReceiptData>) -> Self {
        Self { read }
    }
}

/// Metadata about the event that was last read and when.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ReceiptData {
    /// Metadata for the read receipt.
    pub data: Receipt,

    /// The extremity event ID the user has read up to.
    pub event_ids: Vec<EventId>,
}

impl ReceiptData {
    /// Creates a new `ReceiptData`.
    pub fn new(data: Receipt, event_ids: Vec<EventId>) -> Self {
        Self { data, event_ids }
    }
}

/// The content for "m.typing" Edu.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct TypingContent {
    /// The room where the user's typing status has been updated.
    pub room_id: RoomId,

    /// The user ID that has had their typing status changed.
    pub user_id: UserId,

    /// Whether the user is typing in the room or not.
    pub typing: bool,
}

impl TypingContent {
    /// Creates a new `TypingContent`.
    pub fn new(room_id: RoomId, user_id: UserId, typing: bool) -> Self {
        Self { room_id, user_id, typing }
    }
}

/// The description of the direct-to- device message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
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
    /// which have not been referred to already in an EDU's prev_id field.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prev_id: Vec<UInt>,

    /// True if the server is announcing that this device has been deleted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted: Option<bool>,

    /// The updated identity keys (if any) for this device.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keys: Option<DeviceKeys>,
}

impl DeviceListUpdateContent {
    /// Creates a new `DeviceListUpdateContent` with default `prev_id`, `deleted`
    /// and `keys` fields.
    pub fn new(user_id: UserId, device_id: DeviceIdBox, name: String, stream_id: UInt) -> Self {
        Self {
            user_id,
            device_id,
            device_display_name: name,
            stream_id,
            prev_id: vec![],
            deleted: None,
            keys: None,
        }
    }
}

/// The description of the direct-to- device message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct DirectDeviceContent {
    /// The user ID of the sender.
    pub sender: UserId,

    /// Event type for the message.
    #[serde(rename = "type")]
    pub ev_type: EventType,

    /// Unique utf8 string ID for the message, used for idempotency.
    pub message_id: String,

    /// The contents of the messages to be sent. These are arranged in a map
    /// of user IDs to a map of device IDs to message bodies. The device ID may
    /// also be *, meaning all known devices for the user.
    pub messages: BTreeMap<UserId, BTreeMap<DeviceIdOrAllDevices, Box<RawJsonValue>>>,
}

impl DirectDeviceContent {
    /// Creates a new `DirectDeviceContent` with an empty `messages` map.
    pub fn new(sender: UserId, ev_type: EventType, message_id: String) -> Self {
        Self { sender, ev_type, message_id, messages: BTreeMap::new() }
    }
}

#[cfg(test)]
mod test {
    use js_int::uint;
    use matches::assert_matches;
    use ruma_identifiers::{device_id, room_id, user_id};
    use serde_json::json;

    use super::*;

    #[test]
    fn device_list_update_edu() {
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
                && device_id == &device_id!("QBUAZIFURK")
                && device_display_name == "Mobile"
                && stream_id == &uint!(6)
                && prev_id.is_empty()
                && deleted == &Some(false)
                && keys.is_some()
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
                                "ts": 1_533_358
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

    #[test]
    fn typing_edu() {
        let json = json!({
            "content": {
                "room_id": "!somewhere:matrix.org",
                "typing": true,
                "user_id": "@john:matrix.org"
            },
            "edu_type": "m.typing"
        });

        let edu = serde_json::from_value::<Edu>(json).unwrap();
        assert_matches!(
            &edu,
            Edu::Typing(TypingContent {
                room_id, user_id, typing
            }) if room_id == &room_id!("!somewhere:matrix.org")
                && user_id == &user_id!("@john:matrix.org")
                && *typing
        );

        assert_eq!(
            serde_json::to_string(&edu).unwrap(),
            r#"{"edu_type":"m.typing","content":{"room_id":"!somewhere:matrix.org","user_id":"@john:matrix.org","typing":true}}"#
        );
    }

    #[test]
    fn direct_to_device_edu() {
        let json = json!({
            "content": {
                "message_id": "hiezohf6Hoo7kaev",
                "messages": {
                    "@alice:example.org": {
                        "IWHQUZUIAH": {
                            "algorithm": "m.megolm.v1.aes-sha2",
                            "room_id": "!Cuyf34gef24t:localhost",
                            "session_id": "X3lUlvLELLYxeTx4yOVu6UDpasGEVO0Jbu+QFnm0cKQ",
                            "session_key": "AgAAAADxKHa9uFxcXzwYoNueL5Xqi69IkD4sni8LlfJL7qNBEY..."
                        }
                    }
                },
                "sender": "@john:example.com",
                "type": "m.room_key_request"
            },
            "edu_type": "m.direct_to_device"
        });

        let edu = serde_json::from_value::<Edu>(json).unwrap();
        assert_matches!(
            &edu,
            Edu::DirectToDevice(DirectDeviceContent {
                sender, ev_type, message_id, messages
            }) if sender == &user_id!("@john:example.com")
                && ev_type == &EventType::RoomKeyRequest
                && message_id == "hiezohf6Hoo7kaev"
                && messages.get(&user_id!("@alice:example.org")).is_some()
        );

        assert_eq!(
            serde_json::to_string(&edu).unwrap(),
            r#"{"edu_type":"m.direct_to_device","content":{"sender":"@john:example.com","type":"m.room_key_request","message_id":"hiezohf6Hoo7kaev","messages":{"@alice:example.org":{"IWHQUZUIAH":{"algorithm":"m.megolm.v1.aes-sha2","room_id":"!Cuyf34gef24t:localhost","session_id":"X3lUlvLELLYxeTx4yOVu6UDpasGEVO0Jbu+QFnm0cKQ","session_key":"AgAAAADxKHa9uFxcXzwYoNueL5Xqi69IkD4sni8LlfJL7qNBEY..."}}}}}"#
        );
    }
}
