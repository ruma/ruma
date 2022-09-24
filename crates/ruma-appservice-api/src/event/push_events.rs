//! `PUT /_matrix/app/*/transactions/{txnId}`
//!
//! Endpoint to push an event (or batch of events) to the application service.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/application-service-api/#put_matrixappv1transactionstxnid

    use ruma_common::{
        api::ruma_api, events::AnyTimelineEvent, serde::Raw, OwnedTransactionId, TransactionId,
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
            pub events: &'a [Raw<AnyTimelineEvent>],
        }

        #[derive(Default)]
        response: {}
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given transaction ID and list of events.
        pub fn new(txn_id: &'a TransactionId, events: &'a [Raw<AnyTimelineEvent>]) -> Self {
            Self { txn_id, events }
        }
    }

    impl IncomingRequest {
        /// Creates an `IncomingRequest` with the given transaction ID and list of events.
        pub fn new(
            txn_id: OwnedTransactionId,
            events: Vec<Raw<AnyTimelineEvent>>,
        ) -> IncomingRequest {
            IncomingRequest { txn_id, events }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
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
