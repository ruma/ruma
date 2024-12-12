//! `PUT /_matrix/app/*/transactions/{txnId}`
//!
//! Endpoint to push an event (or batch of events) to the application service.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/application-service-api/#put_matrixappv1transactionstxnid

    #[cfg(any(feature = "unstable-msc2409", feature = "unstable-msc3202"))]
    use std::collections::BTreeMap;
    #[cfg(feature = "unstable-msc2409")]
    use std::{
        collections::btree_map,
        ops::{Deref, DerefMut},
    };

    #[cfg(any(feature = "unstable-msc2409", feature = "unstable-msc3202"))]
    use js_int::UInt;
    #[cfg(any(feature = "unstable-msc2409", feature = "unstable-msc3202"))]
    use ruma_common::OwnedUserId;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::Raw,
        OwnedTransactionId,
    };
    #[cfg(feature = "unstable-msc2409")]
    use ruma_common::{
        presence::PresenceState, serde::from_raw_json_value, OwnedEventId, OwnedRoomId,
    };
    #[cfg(feature = "unstable-msc3202")]
    use ruma_common::{OneTimeKeyAlgorithm, OwnedDeviceId};
    #[cfg(feature = "unstable-msc2409")]
    use ruma_events::receipt::Receipt;
    use ruma_events::AnyTimelineEvent;
    #[cfg(feature = "unstable-msc4203")]
    use ruma_events::AnyToDeviceEvent;
    #[cfg(feature = "unstable-msc2409")]
    use serde::Deserializer;
    #[cfg(any(feature = "unstable-msc2409", feature = "unstable-msc3202"))]
    use serde::{Deserialize, Serialize};
    #[cfg(feature = "unstable-msc2409")]
    use serde_json::{value::RawValue as RawJsonValue, Value as JsonValue};

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/app/v1/transactions/:txn_id",
        }
    };

    /// Request type for the `push_events` endpoint.
    #[request]
    pub struct Request {
        /// The transaction ID for this set of events.
        ///
        /// Homeservers generate these IDs and they are used to ensure idempotency of results.
        #[ruma_api(path)]
        pub txn_id: OwnedTransactionId,

        /// A list of events.
        pub events: Vec<Raw<AnyTimelineEvent>>,

        /// Information on E2E device updates.
        #[cfg(feature = "unstable-msc3202")]
        #[serde(
            default,
            skip_serializing_if = "DeviceLists::is_empty",
            rename = "org.matrix.msc3202.device_lists"
        )]
        pub device_lists: DeviceLists,

        /// The number of unclaimed one-time keys currently held on the server for this device, for
        /// each algorithm.
        #[cfg(feature = "unstable-msc3202")]
        #[serde(
            default,
            skip_serializing_if = "BTreeMap::is_empty",
            rename = "org.matrix.msc3202.device_one_time_keys_count"
        )]
        pub device_one_time_keys_count:
            BTreeMap<OwnedUserId, BTreeMap<OwnedDeviceId, BTreeMap<OneTimeKeyAlgorithm, UInt>>>,

        /// A list of key algorithms for which the server has an unused fallback key for the
        /// device.
        #[cfg(feature = "unstable-msc3202")]
        #[serde(
            default,
            skip_serializing_if = "BTreeMap::is_empty",
            rename = "org.matrix.msc3202.device_unused_fallback_key_types"
        )]
        pub device_unused_fallback_key_types:
            BTreeMap<OwnedUserId, BTreeMap<OwnedDeviceId, Vec<OneTimeKeyAlgorithm>>>,

        /// A list of EDUs.
        #[cfg(feature = "unstable-msc2409")]
        #[serde(
            default,
            skip_serializing_if = "<[_]>::is_empty",
            rename = "de.sorunome.msc2409.ephemeral"
        )]
        pub ephemeral: Vec<Edu>,

        /// A list of to-device messages.
        #[cfg(feature = "unstable-msc4203")]
        #[serde(
            default,
            skip_serializing_if = "<[_]>::is_empty",
            rename = "de.sorunome.msc2409.to_device"
        )]
        pub to_device: Vec<Raw<AnyToDeviceEvent>>,
    }

    /// Response type for the `push_events` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates an `Request` with the given transaction ID and list of events.
        pub fn new(txn_id: OwnedTransactionId, events: Vec<Raw<AnyTimelineEvent>>) -> Request {
            Request {
                txn_id,
                events,
                #[cfg(feature = "unstable-msc3202")]
                device_lists: DeviceLists::new(),
                #[cfg(feature = "unstable-msc3202")]
                device_one_time_keys_count: BTreeMap::new(),
                #[cfg(feature = "unstable-msc3202")]
                device_unused_fallback_key_types: BTreeMap::new(),
                #[cfg(feature = "unstable-msc2409")]
                ephemeral: Vec::new(),
                #[cfg(feature = "unstable-msc4203")]
                to_device: Vec::new(),
            }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }

    /// Information on E2E device updates.
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    #[cfg(feature = "unstable-msc3202")]
    pub struct DeviceLists {
        /// List of users who have updated their device identity keys or who now
        /// share an encrypted room with the client since the previous sync.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub changed: Vec<OwnedUserId>,

        /// List of users who no longer share encrypted rooms since the previous sync
        /// response.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub left: Vec<OwnedUserId>,
    }

    #[cfg(feature = "unstable-msc3202")]
    impl DeviceLists {
        /// Creates an empty `DeviceLists`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Returns true if there are no device list updates.
        pub fn is_empty(&self) -> bool {
            self.changed.is_empty() && self.left.is_empty()
        }
    }

    /// Type for passing ephemeral data to homeservers.
    #[cfg(feature = "unstable-msc2409")]
    #[derive(Clone, Debug, Serialize)]
    #[non_exhaustive]
    pub enum Edu {
        /// An EDU representing presence updates for users of the sending homeserver.
        Presence(PresenceContent),

        /// An EDU representing receipt updates for users of the sending homeserver.
        Receipt(ReceiptContent),

        /// A typing notification EDU for a user in a room.
        Typing(TypingContent),

        #[doc(hidden)]
        #[serde(skip)]
        _Custom(JsonValue),
    }

    #[derive(Debug, Deserialize)]
    #[cfg(feature = "unstable-msc2409")]
    struct EduDeHelper {
        /// The message type field
        r#type: String,
        content: Box<RawJsonValue>,
    }

    #[cfg(feature = "unstable-msc2409")]
    impl<'de> Deserialize<'de> for Edu {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let json = Box::<RawJsonValue>::deserialize(deserializer)?;
            let EduDeHelper { r#type, content } = from_raw_json_value(&json)?;

            Ok(match r#type.as_ref() {
                "m.presence" => Self::Presence(from_raw_json_value(&content)?),
                "m.receipt" => Self::Receipt(from_raw_json_value(&content)?),
                "m.typing" => Self::Typing(from_raw_json_value(&content)?),
                _ => Self::_Custom(from_raw_json_value(&content)?),
            })
        }
    }

    /// The content for "m.presence" Edu.
    #[cfg(feature = "unstable-msc2409")]
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct PresenceContent {
        /// A list of presence updates that the receiving server is likely to be interested in.
        pub push: Vec<PresenceUpdate>,
    }

    #[cfg(feature = "unstable-msc2409")]
    impl PresenceContent {
        /// Creates a new `PresenceContent`.
        pub fn new(push: Vec<PresenceUpdate>) -> Self {
            Self { push }
        }
    }

    /// An update to the presence of a user.
    #[cfg(feature = "unstable-msc2409")]
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct PresenceUpdate {
        /// The user ID this presence EDU is for.
        pub user_id: OwnedUserId,

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
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub currently_active: bool,
    }

    #[cfg(feature = "unstable-msc2409")]
    impl PresenceUpdate {
        /// Creates a new `PresenceUpdate` with the given `user_id`, `presence` and `last_activity`.
        pub fn new(user_id: OwnedUserId, presence: PresenceState, last_activity: UInt) -> Self {
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
    #[cfg(feature = "unstable-msc2409")]
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    #[serde(transparent)]
    pub struct ReceiptContent(pub BTreeMap<OwnedRoomId, ReceiptMap>);

    #[cfg(feature = "unstable-msc2409")]
    impl ReceiptContent {
        /// Creates a new `ReceiptContent`.
        pub fn new(receipts: BTreeMap<OwnedRoomId, ReceiptMap>) -> Self {
            Self(receipts)
        }
    }

    #[cfg(feature = "unstable-msc2409")]
    impl Deref for ReceiptContent {
        type Target = BTreeMap<OwnedRoomId, ReceiptMap>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[cfg(feature = "unstable-msc2409")]
    impl DerefMut for ReceiptContent {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    #[cfg(feature = "unstable-msc2409")]
    impl IntoIterator for ReceiptContent {
        type Item = (OwnedRoomId, ReceiptMap);
        type IntoIter = btree_map::IntoIter<OwnedRoomId, ReceiptMap>;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }

    #[cfg(feature = "unstable-msc2409")]
    impl FromIterator<(OwnedRoomId, ReceiptMap)> for ReceiptContent {
        fn from_iter<T>(iter: T) -> Self
        where
            T: IntoIterator<Item = (OwnedRoomId, ReceiptMap)>,
        {
            Self(BTreeMap::from_iter(iter))
        }
    }

    /// Mapping between user and `ReceiptData`.
    #[cfg(feature = "unstable-msc2409")]
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ReceiptMap {
        /// Read receipts for users in the room.
        #[serde(rename = "m.read")]
        pub read: BTreeMap<OwnedUserId, ReceiptData>,
    }

    #[cfg(feature = "unstable-msc2409")]
    impl ReceiptMap {
        /// Creates a new `ReceiptMap`.
        pub fn new(read: BTreeMap<OwnedUserId, ReceiptData>) -> Self {
            Self { read }
        }
    }

    /// Metadata about the event that was last read and when.
    #[cfg(feature = "unstable-msc2409")]
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ReceiptData {
        /// Metadata for the read receipt.
        pub data: Receipt,

        /// The extremity event ID the user has read up to.
        pub event_ids: Vec<OwnedEventId>,
    }

    #[cfg(feature = "unstable-msc2409")]
    impl ReceiptData {
        /// Creates a new `ReceiptData`.
        pub fn new(data: Receipt, event_ids: Vec<OwnedEventId>) -> Self {
            Self { data, event_ids }
        }
    }

    /// The content for "m.typing" Edu.
    #[cfg(feature = "unstable-msc2409")]
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct TypingContent {
        /// The room where the user's typing status has been updated.
        pub room_id: OwnedRoomId,

        /// The user ID that has had their typing status changed.
        pub user_id: OwnedUserId,

        /// Whether the user is typing in the room or not.
        pub typing: bool,
    }

    #[cfg(feature = "unstable-msc2409")]
    impl TypingContent {
        /// Creates a new `TypingContent`.
        pub fn new(room_id: OwnedRoomId, user_id: OwnedUserId, typing: bool) -> Self {
            Self { room_id, user_id, typing }
        }
    }

    #[cfg(feature = "server")]
    #[cfg(test)]
    mod tests {
        use ruma_common::api::{OutgoingRequest, SendAccessToken};
        use serde_json::json;

        use super::Request;

        #[test]
        fn decode_request_contains_events_field() {
            let dummy_event = serde_json::from_value(json!({
                "type": "m.room.message",
                "event_id": "$143273582443PhrSn:example.com",
                "origin_server_ts": 1,
                "room_id": "!roomid:room.com",
                "sender": "@user:example.com",
                "content": {
                    "body": "test",
                    "msgtype": "m.text",
                },
            }))
            .unwrap();
            let events = vec![dummy_event];

            let req = Request::new("any_txn_id".into(), events)
                .try_into_http_request::<Vec<u8>>(
                    "https://homeserver.tld",
                    SendAccessToken::IfRequired("auth_tok"),
                    &[ruma_common::api::MatrixVersion::V1_1],
                )
                .unwrap();
            let json_body: serde_json::Value = serde_json::from_slice(req.body()).unwrap();

            assert_eq!(
                1,
                json_body.as_object().unwrap().get("events").unwrap().as_array().unwrap().len()
            );
        }
    }
}
