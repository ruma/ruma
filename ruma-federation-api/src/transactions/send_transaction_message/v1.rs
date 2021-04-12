//! [PUT /_matrix/federation/v1/send/{txnId}](https://matrix.org/docs/spec/server_server/r0.1.3#put-matrix-federation-v1-send-txnid)

use std::{collections::BTreeMap, time::SystemTime};

use js_int::UInt;
use ruma_api::ruma_api;
use ruma_common::{encryption::DeviceKeys, presence::PresenceState};
use ruma_events::{from_raw_json_value, pdu::Pdu, receipt::Receipt};
use ruma_identifiers::{DeviceId, DeviceIdBox, EventId, RoomId, ServerName, UserId};
use ruma_serde::Raw;
use serde::{de, Deserialize, Serialize};
use serde_json::{value::RawValue as RawJsonValue, Value as JsonValue};

ruma_api! {
    metadata: {
        description: "Send transaction messages to another server",
        name: "send_transaction_message",
        method: PUT,
        path: "/_matrix/federation/v1/send/:transaction_id",
        rate_limited: false,
        authentication: ServerSignatures,
    }

    request: {
        /// A transaction ID unique between sending and receiving homeservers.
        #[ruma_api(path)]
        pub transaction_id: &'a str,

        /// The server_name of the homeserver sending this transaction.
        pub origin: &'a ServerName,

        /// POSIX timestamp in milliseconds on the originating homeserver when this transaction
        /// started.
        #[serde(with = "ruma_serde::time::ms_since_unix_epoch")]
        pub origin_server_ts: SystemTime,

        /// List of persistent updates to rooms.
        ///
        /// Must not be more than 50 items.
        #[cfg_attr(feature = "unstable-pre-spec", serde(default, skip_serializing_if = "<[_]>::is_empty"))]
        pub pdus: &'a [Raw<Pdu>],

        /// List of ephemeral messages.
        ///
        /// Must not be more than 100 items.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        pub edus: &'a [Raw<Edu>],
    }

    #[derive(Default)]
    response: {
        /// Map of event IDs and response for each PDU given in the request.
        #[serde(with = "crate::serde::pdu_process_response")]
        pub pdus: BTreeMap<EventId, Result<(), String>>,
    }
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given transaction ID, origin, timestamp.
    ///
    /// The PDU and EDU lists will start off empty.
    pub fn new(
        transaction_id: &'a str,
        origin: &'a ServerName,
        origin_server_ts: SystemTime,
    ) -> Self {
        Self { transaction_id, origin, origin_server_ts, pdus: &[], edus: &[] }
    }
}

impl Response {
    /// Creates a new `Response` with the given PDUs.
    pub fn new(pdus: BTreeMap<EventId, Result<(), String>>) -> Self {
        Self { pdus }
    }
}

// /// Type for passing ephemeral data to homeservers.
// #[derive(Clone, Debug, Deserialize, Serialize)]
// pub struct Edu {
//     /// Type of the ephemeral message.
//     pub edu_type: String,

//     /// Content of ephemeral message
//     pub content: JsonValue,
// }

/// Type for passing ephemeral data to homeservers.
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum Edu {
    /// An EDU representing presence updates for users of the sending homeserver.
    Presence(PresenceContent),

    /// An EDU representing receipt updates for users of the sending homeserver.
    Receipt(ReceiptContent),

    /// A typing notification EDU for a user in a room.
    Typing(TypingContent),

    /// An EDU that lets servers push details to each other when one of their users adds
    // a new device to their account, required for E2E encryption to correctly target the
    // current set of devices for a given user.
    DeviceListUpdate(DeviceListUpdateContent),

    /// An EDU that lets servers push send events directly to a specific device on a
    // remote server - for instance, to maintain an Olm E2E encrypted message channel
    // between a local and remote device.
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "edu_type", rename = "m.presence")]
pub struct PresenceContent {
    pub push: Vec<PresenceUpdate>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PresenceUpdate {
    pub user_id: UserId,
    pub presence: PresenceState,
    pub status_msg: String,
    pub last_active_ago: UInt,
    pub currently_active: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "edu_type", rename = "m.receipt")]
pub struct ReceiptContent {
    pub receipts: BTreeMap<RoomId, ReceiptMap>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReceiptMap {
    pub read: BTreeMap<UserId, ReceiptData>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReceiptData {
    pub data: Receipt,
    pub event_ids: Vec<EventId>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "edu_type", rename = "m.typing")]
pub struct TypingContent {
    pub notifi: TypingNotice,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TypingNotice {
    pub room_id: RoomId,
    pub user_id: UserId,
    pub typing: bool,
}

/// The description of the direct-to- device message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "edu_type", rename = "m.device_list_update")]
pub struct DeviceListUpdateContent {
    /// The user ID who owns the device.
    pub user_id: UserId,

    /// The ID of the device whose details are changing.
    pub device_id: String,

    /// The public human-readable name of this device. Will be absent if the device has no name.
    pub device_display_name: String,

    /// An ID sent by the server for this update, unique for a given user_id.
    pub stream_id: UInt,

    /// The stream_ids of any prior m.device_list_update EDUs sent for this user
    // which have not been referred to already in an EDU's prev_id field.
    pub prev_id: Vec<UInt>,

    /// True if the server is announcing that this device has been deleted.
    pub deleted: bool,

    /// The updated identity keys (if any) for this device.
    pub keys: DeviceKeys,
}

/// The description of the direct-to- device message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "edu_type", rename = "m.direct_to_device")]
pub struct DirectDeviceContent {
    /// The user ID of the sender.
    pub sender: UserId,

    /// Event type for the message.
    #[serde(rename = "type")]
    pub ev_type: String,

    /// Unique utf8 string ID for the message, used for idempotence.
    pub message_id: String,

    /// The contents of the messages to be sent. These are arranged in a map
    // of user IDs to a map of device IDs to message bodies. The device ID may
    // also be *, meaning all known devices for the user.
    pub messages: BTreeMap<String, BTreeMap<DeviceIdBox, JsonValue>>,
}
// "m.presence", "m.receipt", "m.typing", "m.device_list_update", "m.direct_to_device"

// There is one more edu_type synapse recognizes with the note:
// FIXME: switch to "m.signing_key_update" when MSC1756 is merged into the
// spec from "org.matrix.signing_key_update"

#[cfg(test)]
mod test {
    use matches::assert_matches;
    use ruma_identifiers::user_id;
    use serde_json::json;

    use super::*;

    #[test]
    fn direct_to_device() {
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
                    "prev_id": [
                        5
                    ],
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
        );

        println!("{}", serde_json::to_string_pretty(&edu).unwrap());
    }
}
