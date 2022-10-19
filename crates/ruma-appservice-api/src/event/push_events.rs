//! `PUT /_matrix/app/*/transactions/{txnId}`
//!
//! Endpoint to push an event (or batch of events) to the application service.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/application-service-api/#put_matrixappv1transactionstxnid

    use ruma_common::{
        api::ruma_api, events::AnyTimelineEvent, serde::Raw, OwnedTransactionId, TransactionId,
    };

    #[cfg(feature = "unstable-msc3202")]
    use js_int::UInt;
    #[cfg(feature = "unstable-msc3202")]
    use ruma_common::{DeviceKeyAlgorithm, OwnedDeviceId, OwnedUserId};
    #[cfg(feature = "unstable-msc3202")]
    use serde::{Deserialize, Serialize};
    #[cfg(feature = "unstable-msc3202")]
    use std::collections::BTreeMap;

    ruma_api! {
        metadata: {
            description: "This API is called by the homeserver when it wants to push an event (or batch of events) to the application service.",
            method: PUT,
            name: "push_events",
            stable_path: "/_matrix/app/v1/transactions/:txn_id",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The transaction ID for this set of events.
            ///
            /// Homeservers generate these IDs and they are used to ensure idempotency of results.
            #[ruma_api(path)]
            pub txn_id: &'a TransactionId,

            /// A list of events.
            pub events: &'a [Raw<AnyTimelineEvent>],

            /// Information on E2E device updates.
            #[cfg(feature = "unstable-msc3202")]
            #[serde(
                default,
                skip_serializing_if = "DeviceLists::is_empty",
                rename = "org.matrix.msc3202.device_lists"
            )]
            pub device_lists: DeviceLists,

            /// The number of unclaimed one-time keys currently held on the server for this device, for each algorithm.
            #[cfg(feature = "unstable-msc3202")]
            #[serde(
                default,
                skip_serializing_if = "BTreeMap::is_empty",
                rename = "org.matrix.msc3202.device_one_time_keys_count"
            )]
            pub device_one_time_keys_count: BTreeMap<OwnedUserId, BTreeMap<OwnedDeviceId, BTreeMap<DeviceKeyAlgorithm, UInt>>>,

            /// A list of key algorithms for which the server has an unused fallback key for the device.
            #[cfg(feature = "unstable-msc3202")]
            #[serde(
                default,
                skip_serializing_if = "BTreeMap::is_empty",
                rename = "org.matrix.msc3202.device_unused_fallback_key_types"
            )]
            pub device_unused_fallback_key_types: BTreeMap<OwnedUserId, BTreeMap<OwnedDeviceId, Vec<DeviceKeyAlgorithm>>>,
        }

        #[derive(Default)]
        response: {}
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given transaction ID and list of events.
        pub fn new(txn_id: &'a TransactionId, events: &'a [Raw<AnyTimelineEvent>]) -> Self {
            Self {
                txn_id,
                events,
                #[cfg(feature = "unstable-msc3202")]
                device_lists: DeviceLists::new(),
                #[cfg(feature = "unstable-msc3202")]
                device_one_time_keys_count: BTreeMap::new(),
                #[cfg(feature = "unstable-msc3202")]
                device_unused_fallback_key_types: BTreeMap::new(),
            }
        }
    }

    impl IncomingRequest {
        /// Creates an `IncomingRequest` with the given transaction ID and list of events.
        pub fn new(
            txn_id: OwnedTransactionId,
            events: Vec<Raw<AnyTimelineEvent>>,
        ) -> IncomingRequest {
            IncomingRequest {
                txn_id,
                events,
                #[cfg(feature = "unstable-msc3202")]
                device_lists: DeviceLists::new(),
                #[cfg(feature = "unstable-msc3202")]
                device_one_time_keys_count: BTreeMap::new(),
                #[cfg(feature = "unstable-msc3202")]
                device_unused_fallback_key_types: BTreeMap::new(),
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
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
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

            let req = Request { events: &events, txn_id: "any_txn_id".into() }
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
