//! `PUT /_matrix/app/*/transactions/{txnId}`
//!
//! Endpoint to push an event (or batch of events) to the application service.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/application-service-api/#put_matrixappv1transactionstxnid

    use ruma_common::{
        api::ruma_api, events::AnyRoomEvent, serde::Raw, OwnedTransactionId, TransactionId,
    };

    ruma_api! {
        metadata: {
            description: "This API is called by the homeserver when it wants to push an event (or batch of events) to the application service.",
            method: PUT,
            name: "push_events",
            stable_path: "/_matrix/app/v1/transactions/:txn_id",
            rate_limited: false,
            authentication: QueryOnlyAccessToken,
            added: 1.0,
        }

        request: {
            /// The transaction ID for this set of events.
            ///
            /// Homeservers generate these IDs and they are used to ensure idempotency of results.
            #[ruma_api(path)]
            pub txn_id: &'a TransactionId,

            /// A list of events.
            pub events: &'a [Raw<AnyRoomEvent>],
        }

        #[derive(Default)]
        response: {}
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given transaction ID and list of events.
        pub fn new(txn_id: &'a TransactionId, events: &'a [Raw<AnyRoomEvent>]) -> Self {
            Self { txn_id, events }
        }
    }

    impl IncomingRequest {
        /// Creates an `IncomingRequest` with the given transaction ID and list of events.
        pub fn new(txn_id: OwnedTransactionId, events: Vec<Raw<AnyRoomEvent>>) -> IncomingRequest {
            IncomingRequest { txn_id, events }
        }

        /// Consumes the `IncomingRequest` and tries to convert it to a `sync_events::Response`
        ///
        /// This is a helper conversion in cases where it's easier to work with
        /// `sync_events::Response` instead of the original `push_events::IncomingRequest`.
        /// It puts all events with a `room_id` into the `JoinedRoom`'s `timeline`. The
        /// rationale behind that is that incoming Appservice transactions from the
        /// homeserver are not necessarily bound to a specific user but can cover
        /// a multitude of namespaces, and as such the Appservice basically only "observes joined
        /// rooms".
        ///
        /// Note: Currently homeservers only push PDUs to appservices, no EDUs. There's the open
        /// [MSC2409] regarding supporting EDUs in the future, though it seems to be planned to put
        /// EDUs into a different JSON key than `events` to stay backwards compatible.
        ///
        /// [MSC2409]: https://github.com/matrix-org/matrix-spec-proposals/pull/2409
        #[cfg(feature = "helper")]
        pub fn try_into_sync_response(
            self,
            next_batch: impl Into<String>,
        ) -> serde_json::Result<ruma_client_api::sync::sync_events::v3::Response> {
            use ruma_client_api::sync::sync_events;
            use ruma_common::OwnedRoomId;
            use serde::Deserialize;
            use tracing::warn;

            #[derive(Debug, Deserialize)]
            struct EventDeHelper {
                room_id: Option<OwnedRoomId>,
            }

            let mut response = sync_events::v3::Response::new(next_batch.into());

            for raw_event in self.events {
                let helper = raw_event.deserialize_as::<EventDeHelper>()?;
                let event_json = Raw::into_json(raw_event);

                if let Some(room_id) = helper.room_id {
                    let join = response.rooms.join.entry(room_id).or_default();
                    join.timeline.events.push(Raw::from_json(event_json));
                } else {
                    warn!("Event without room_id: {}", event_json);
                }
            }

            Ok(response)
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }

    #[cfg(feature = "helper")]
    #[cfg(test)]
    mod helper_tests {
        use ruma_client_api::sync::sync_events;
        use ruma_common::{room_id, TransactionId};
        use serde_json::{json, value::to_raw_value as to_raw_json_value};

        use super::{IncomingRequest, Raw};

        #[test]
        fn convert_incoming_request_to_sync_response() {
            let txn_id = <&TransactionId>::from("any_txn_id");
            let state_event = Raw::from_json(
                to_raw_json_value(&json!({
                    "content": {},
                    "event_id": "$h29iv0s8:example.com",
                    "origin_server_ts": 1,
                    "room_id": "!roomid:room.com",
                    "sender": "@carl:example.com",
                    "state_key": "",
                    "type": "m.room.name"
                }))
                .unwrap(),
            );
            let message_event = Raw::from_json(
                to_raw_json_value(&json!({
                    "type": "m.room.message",
                    "event_id": "$143273582443PhrSn:example.com",
                    "origin_server_ts": 1,
                    "room_id": "!roomid:room.com",
                    "sender": "@user:example.com",
                    "content": {
                        "body": "test",
                        "msgtype": "m.audio",
                        "url": "mxc://example.com/AuDi0",
                    }
                }))
                .unwrap(),
            );

            let events = vec![state_event, message_event];
            let incoming_request = IncomingRequest { txn_id: txn_id.into(), events };

            let response: sync_events::v3::Response =
                incoming_request.try_into_sync_response("token").unwrap();

            let response_rooms_join = response
                .rooms
                .join
                .get(room_id!("!roomid:room.com"))
                .expect("joined room response");

            assert_eq!(response_rooms_join.timeline.events.len(), 2);
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
