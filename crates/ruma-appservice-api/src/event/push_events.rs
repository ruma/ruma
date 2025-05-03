//! `PUT /_matrix/app/*/transactions/{txnId}`
//!
//! Endpoint to push an event (or batch of events) to the application service.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/application-service-api/#put_matrixappv1transactionstxnid

    use std::borrow::Cow;
    #[cfg(feature = "unstable-msc3202")]
    use std::collections::BTreeMap;

    #[cfg(feature = "unstable-msc3202")]
    use js_int::UInt;
    #[cfg(feature = "unstable-msc3202")]
    use ruma_common::OwnedUserId;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::{from_raw_json_value, JsonObject, Raw},
        OwnedTransactionId,
    };
    #[cfg(feature = "unstable-msc3202")]
    use ruma_common::{OneTimeKeyAlgorithm, OwnedDeviceId};
    #[cfg(feature = "unstable-msc4203")]
    use ruma_events::AnyToDeviceEvent;
    use ruma_events::{
        presence::PresenceEvent, receipt::ReceiptEvent, typing::TypingEvent, AnyTimelineEvent,
    };
    use serde::{Deserialize, Deserializer, Serialize};
    use serde_json::value::{RawValue as RawJsonValue, Value as JsonValue};

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

        /// A list of ephemeral data.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        pub ephemeral: Vec<EphemeralData>,

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

    /// Type for passing ephemeral data to application services.
    #[derive(Clone, Debug, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    #[serde(untagged)]
    pub enum EphemeralData {
        /// A presence update for a user.
        Presence(PresenceEvent),

        /// A receipt update for a room.
        Receipt(ReceiptEvent),

        /// A typing notification update for a room.
        Typing(TypingEvent),

        #[doc(hidden)]
        _Custom(_CustomEphemeralData),
    }

    impl EphemeralData {
        /// A reference to the `type` string of the data.
        pub fn data_type(&self) -> &str {
            match self {
                Self::Presence(_) => "m.presence",
                Self::Receipt(_) => "m.receipt",
                Self::Typing(_) => "m.typing",
                Self::_Custom(c) => &c.data_type,
            }
        }

        /// The data as a JSON object.
        ///
        /// Prefer to use the public variants of `EphemeralData` where possible; this method is
        /// meant to be used for unsupported data types only.
        pub fn data(&self) -> Cow<'_, JsonObject> {
            fn serialize<T: Serialize>(obj: &T) -> JsonObject {
                match serde_json::to_value(obj).expect("ephemeral data serialization to succeed") {
                    JsonValue::Object(obj) => obj,
                    _ => panic!("all ephemeral data types must serialize to objects"),
                }
            }

            match self {
                Self::Presence(d) => Cow::Owned(serialize(d)),
                Self::Receipt(d) => Cow::Owned(serialize(d)),
                Self::Typing(d) => Cow::Owned(serialize(d)),
                Self::_Custom(c) => Cow::Borrowed(&c.data),
            }
        }
    }

    impl<'de> Deserialize<'de> for EphemeralData {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            #[derive(Deserialize)]
            struct EphemeralDataDeHelper {
                /// The data type.
                #[serde(rename = "type")]
                data_type: String,
            }

            let json = Box::<RawJsonValue>::deserialize(deserializer)?;
            let EphemeralDataDeHelper { data_type } = from_raw_json_value(&json)?;

            Ok(match data_type.as_ref() {
                "m.presence" => Self::Presence(from_raw_json_value(&json)?),
                "m.receipt" => Self::Receipt(from_raw_json_value(&json)?),
                "m.typing" => Self::Typing(from_raw_json_value(&json)?),
                _ => Self::_Custom(_CustomEphemeralData {
                    data_type,
                    data: from_raw_json_value(&json)?,
                }),
            })
        }
    }

    /// Ephemeral data with an unknown type.
    #[doc(hidden)]
    #[derive(Debug, Clone)]
    pub struct _CustomEphemeralData {
        /// The type of the data.
        data_type: String,
        /// The data.
        data: JsonObject,
    }

    impl Serialize for _CustomEphemeralData {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.data.serialize(serializer)
        }
    }

    #[cfg(test)]
    mod tests {
        use assert_matches2::assert_matches;
        use js_int::uint;
        use ruma_common::{event_id, room_id, user_id, MilliSecondsSinceUnixEpoch};
        use ruma_events::receipt::ReceiptType;
        use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

        use super::EphemeralData;

        #[cfg(feature = "client")]
        #[test]
        fn request_contains_events_field() {
            use ruma_common::api::{
                MatrixVersion, OutgoingRequest, SendAccessToken, SupportedVersions,
            };

            let dummy_event_json = json!({
                "type": "m.room.message",
                "event_id": "$143273582443PhrSn:example.com",
                "origin_server_ts": 1,
                "room_id": "!roomid:room.com",
                "sender": "@user:example.com",
                "content": {
                    "body": "test",
                    "msgtype": "m.text",
                },
            });
            let dummy_event = from_json_value(dummy_event_json.clone()).unwrap();
            let events = vec![dummy_event];
            let supported =
                SupportedVersions { versions: [MatrixVersion::V1_1].into(), features: Vec::new() };

            let req = super::Request::new("any_txn_id".into(), events)
                .try_into_http_request::<Vec<u8>>(
                    "https://homeserver.tld",
                    SendAccessToken::IfRequired("auth_tok"),
                    &supported,
                )
                .unwrap();
            let json_body: serde_json::Value = serde_json::from_slice(req.body()).unwrap();

            assert_eq!(
                json_body,
                json!({
                    "events": [
                        dummy_event_json,
                    ]
                })
            );
        }

        #[test]
        fn serde_ephemeral_data() {
            let room_id = room_id!("!jEsUZKDJdhlrceRyVU:server.local");
            let user_id = user_id!("@alice:server.local");
            let event_id = event_id!("$1435641916114394fHBL");

            // Test m.typing serde.
            let typing_json = json!({
                "type": "m.typing",
                "room_id": room_id,
                "content": {
                    "user_ids": [user_id],
                },
            });

            let data = from_json_value::<EphemeralData>(typing_json.clone()).unwrap();
            assert_matches!(&data, EphemeralData::Typing(typing));
            assert_eq!(typing.room_id, room_id);
            assert_eq!(typing.content.user_ids, &[user_id.to_owned()]);

            let serialized_data = to_json_value(data).unwrap();
            assert_eq!(serialized_data, typing_json);

            // Test m.receipt serde.
            let receipt_json = json!({
                "type": "m.receipt",
                "room_id": room_id,
                "content": {
                    event_id: {
                        "m.read": {
                            user_id: {
                                "ts": 453,
                            },
                        },
                    },
                },
            });

            let data = from_json_value::<EphemeralData>(receipt_json.clone()).unwrap();
            assert_matches!(&data, EphemeralData::Receipt(receipt));
            assert_eq!(receipt.room_id, room_id);
            let event_receipts = receipt.content.get(event_id).unwrap();
            let event_read_receipts = event_receipts.get(&ReceiptType::Read).unwrap();
            let event_user_read_receipt = event_read_receipts.get(user_id).unwrap();
            assert_eq!(event_user_read_receipt.ts, Some(MilliSecondsSinceUnixEpoch(uint!(453))));

            let serialized_data = to_json_value(data).unwrap();
            assert_eq!(serialized_data, receipt_json);

            // Test m.presence serde.
            let presence_json = json!({
                "type": "m.presence",
                "sender": user_id,
                "content": {
                    "avatar_url": "mxc://localhost/wefuiwegh8742w",
                    "currently_active": false,
                    "last_active_ago": 785,
                    "presence": "online",
                    "status_msg": "Making cupcakes",
                },
            });

            let data = from_json_value::<EphemeralData>(presence_json.clone()).unwrap();
            assert_matches!(&data, EphemeralData::Presence(presence));
            assert_eq!(presence.sender, user_id);
            assert_eq!(presence.content.currently_active, Some(false));

            let serialized_data = to_json_value(data).unwrap();
            assert_eq!(serialized_data, presence_json);

            // Test custom serde.
            let custom_json = json!({
                "type": "dev.ruma.custom",
                "key": "value",
                "content": {
                    "foo": "bar",
                },
            });

            let data = from_json_value::<EphemeralData>(custom_json.clone()).unwrap();

            let serialized_data = to_json_value(data).unwrap();
            assert_eq!(serialized_data, custom_json);
        }
    }
}
