//! Edu type and variant content structs.

use std::collections::BTreeMap;

use js_int::UInt;
use ruma_common::{
    DeviceId, EventId, RoomId, TransactionId, UserId,
    encryption::{CrossSigningKey, DeviceKeys},
    presence::PresenceState,
    serde::{Raw, from_raw_json_value},
    to_device::DeviceIdOrAllDevices,
};
use ruma_events::{AnyToDeviceEventContent, ToDeviceEventType, receipt::Receipt};
use serde::{Deserialize, Serialize, de};
use serde_json::{Value as JsonValue, value::RawValue as RawJsonValue};

/// Type for passing ephemeral data to homeservers.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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

    /// An EDU that lets servers push details to each other when one of their users updates their
    /// cross-signing keys.
    #[serde(rename = "m.signing_key_update")]
    SigningKeyUpdate(SigningKeyUpdateContent),

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
            "m.signing_key_update" => Self::SigningKeyUpdate(from_raw_json_value(&content)?),
            _ => Self::_Custom(from_raw_json_value(&content)?),
        })
    }
}

/// The content for "m.presence" Edu.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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

    /// Whether or not the user is currently active.
    ///
    /// Defaults to false.
    #[serde(default)]
    pub currently_active: bool,
}

impl PresenceUpdate {
    /// Creates a new `PresenceUpdate` with the given `user_id`, `presence` and `last_activity`.
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
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct DeviceListUpdateContent {
    /// The user ID who owns the device.
    pub user_id: UserId,

    /// The ID of the device whose details are changing.
    pub device_id: DeviceId,

    /// The public human-readable name of this device.
    ///
    /// Will be absent if the device has no name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_display_name: Option<String>,

    /// An ID sent by the server for this update, unique for a given user_id.
    pub stream_id: UInt,

    /// The stream_ids of any prior m.device_list_update EDUs sent for this user which have not
    /// been referred to already in an EDU's prev_id field.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prev_id: Vec<UInt>,

    /// True if the server is announcing that this device has been deleted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted: Option<bool>,

    /// The updated identity keys (if any) for this device.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keys: Option<Raw<DeviceKeys>>,
}

impl DeviceListUpdateContent {
    /// Create a new `DeviceListUpdateContent` with the given `user_id`, `device_id` and
    /// `stream_id`.
    pub fn new(user_id: UserId, device_id: DeviceId, stream_id: UInt) -> Self {
        Self {
            user_id,
            device_id,
            device_display_name: None,
            stream_id,
            prev_id: vec![],
            deleted: None,
            keys: None,
        }
    }
}

/// The description of the direct-to- device message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct DirectDeviceContent {
    /// The user ID of the sender.
    pub sender: UserId,

    /// Event type for the message.
    #[serde(rename = "type")]
    pub ev_type: ToDeviceEventType,

    /// Unique utf8 string ID for the message, used for idempotency.
    pub message_id: TransactionId,

    /// The contents of the messages to be sent.
    ///
    /// These are arranged in a map of user IDs to a map of device IDs to message bodies. The
    /// device ID may also be *, meaning all known devices for the user.
    pub messages: DirectDeviceMessages,
}

impl DirectDeviceContent {
    /// Creates a new `DirectDeviceContent` with the given `sender, `ev_type` and `message_id`.
    pub fn new(sender: UserId, ev_type: ToDeviceEventType, message_id: TransactionId) -> Self {
        Self { sender, ev_type, message_id, messages: DirectDeviceMessages::new() }
    }
}

/// Direct device message contents.
///
/// Represented as a map of `{ user-ids => { device-ids => message-content } }`.
pub type DirectDeviceMessages =
    BTreeMap<UserId, BTreeMap<DeviceIdOrAllDevices, Raw<AnyToDeviceEventContent>>>;

/// The content for an `m.signing_key_update` EDU.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct SigningKeyUpdateContent {
    /// The user ID whose cross-signing keys have changed.
    pub user_id: UserId,

    /// The user's master key, if it was updated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub master_key: Option<Raw<CrossSigningKey>>,

    /// The users's self-signing key, if it was updated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_signing_key: Option<Raw<CrossSigningKey>>,
}

impl SigningKeyUpdateContent {
    /// Creates a new `SigningKeyUpdateContent`.
    pub fn new(user_id: UserId) -> Self {
        Self { user_id, master_key: None, self_signing_key: None }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use js_int::uint;
    use ruma_common::{room_id, user_id};
    use ruma_events::ToDeviceEventType;
    use serde_json::json;

    use super::{DeviceListUpdateContent, Edu, ReceiptContent};

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

        let edu = serde_json::from_value::<Edu>(json.clone()).unwrap();
        assert_matches!(
            &edu,
            Edu::DeviceListUpdate(DeviceListUpdateContent {
                user_id,
                device_id,
                device_display_name,
                stream_id,
                prev_id,
                deleted,
                keys,
            })
        );

        assert_eq!(user_id, "@john:example.com");
        assert_eq!(device_id, "QBUAZIFURK");
        assert_eq!(device_display_name.as_deref(), Some("Mobile"));
        assert_eq!(*stream_id, uint!(6));
        assert_eq!(*prev_id, vec![]);
        assert_eq!(*deleted, Some(false));
        assert_matches!(keys, Some(_));

        assert_eq!(serde_json::to_value(&edu).unwrap(), json);
    }

    #[test]
    fn minimal_device_list_update_edu() {
        let json = json!({
            "content": {
                "device_id": "QBUAZIFURK",
                "stream_id": 6,
                "user_id": "@john:example.com"
            },
            "edu_type": "m.device_list_update"
        });

        let edu = serde_json::from_value::<Edu>(json.clone()).unwrap();
        assert_matches!(
            &edu,
            Edu::DeviceListUpdate(DeviceListUpdateContent {
                user_id,
                device_id,
                device_display_name,
                stream_id,
                prev_id,
                deleted,
                keys,
            })
        );

        assert_eq!(user_id, "@john:example.com");
        assert_eq!(device_id, "QBUAZIFURK");
        assert_eq!(*device_display_name, None);
        assert_eq!(*stream_id, uint!(6));
        assert_eq!(*prev_id, vec![]);
        assert_eq!(*deleted, None);
        assert_matches!(keys, None);

        assert_eq!(serde_json::to_value(&edu).unwrap(), json);
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

        let edu = serde_json::from_value::<Edu>(json.clone()).unwrap();
        assert_matches!(&edu, Edu::Receipt(ReceiptContent { receipts }));
        assert!(receipts.get(room_id!("!some_room:example.org")).is_some());

        assert_eq!(serde_json::to_value(&edu).unwrap(), json);
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

        let edu = serde_json::from_value::<Edu>(json.clone()).unwrap();
        assert_matches!(&edu, Edu::Typing(content));
        assert_eq!(content.room_id, "!somewhere:matrix.org");
        assert_eq!(content.user_id, "@john:matrix.org");
        assert!(content.typing);

        assert_eq!(serde_json::to_value(&edu).unwrap(), json);
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

        let edu = serde_json::from_value::<Edu>(json.clone()).unwrap();
        assert_matches!(&edu, Edu::DirectToDevice(content));
        assert_eq!(content.sender, "@john:example.com");
        assert_eq!(content.ev_type, ToDeviceEventType::RoomKeyRequest);
        assert_eq!(content.message_id, "hiezohf6Hoo7kaev");
        assert!(content.messages.get(user_id!("@alice:example.org")).is_some());

        assert_eq!(serde_json::to_value(&edu).unwrap(), json);
    }

    #[test]
    fn signing_key_update_edu() {
        let json = json!({
            "content": {
                "master_key": {
                    "keys": {
                        "ed25519:alice+base64+public+key": "alice+base64+public+key",
                        "ed25519:base64+master+public+key": "base64+master+public+key"
                    },
                    "signatures": {
                        "@alice:example.com": {
                            "ed25519:alice+base64+master+key": "signature+of+key"
                        }
                    },
                    "usage": [
                        "master"
                    ],
                    "user_id": "@alice:example.com"
                },
                "self_signing_key": {
                    "keys": {
                        "ed25519:alice+base64+public+key": "alice+base64+public+key",
                        "ed25519:base64+self+signing+public+key": "base64+self+signing+master+public+key"
                    },
                    "signatures": {
                        "@alice:example.com": {
                            "ed25519:alice+base64+master+key": "signature+of+key",
                            "ed25519:base64+master+public+key": "signature+of+self+signing+key"
                        }
                    },
                    "usage": [
                        "self_signing"
                    ],
                    "user_id": "@alice:example.com"
                  },
                "user_id": "@alice:example.com"
            },
            "edu_type": "m.signing_key_update"
        });

        let edu = serde_json::from_value::<Edu>(json.clone()).unwrap();
        assert_matches!(&edu, Edu::SigningKeyUpdate(content));
        assert_eq!(content.user_id, "@alice:example.com");
        assert!(content.master_key.is_some());
        assert!(content.self_signing_key.is_some());

        assert_eq!(serde_json::to_value(&edu).unwrap(), json);
    }
}
