//! `PUT /_matrix/client/*/rooms/{roomId}/send/{eventType}/{txnId}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#put_matrixclientv3roomsroomidsendeventtypetxnid

    use ruma_common::{
        api::ruma_api,
        events::{AnyMessageLikeEventContent, MessageLikeEventContent, MessageLikeEventType},
        serde::Raw,
        OwnedEventId, RoomId, TransactionId,
    };
    use serde_json::value::to_raw_value as to_raw_json_value;

    ruma_api! {
        metadata: {
            description: "Send a message event to a room.",
            method: PUT,
            name: "create_message_event",
            r0_path: "/_matrix/client/r0/rooms/:room_id/send/:event_type/:txn_id",
            stable_path: "/_matrix/client/v3/rooms/:room_id/send/:event_type/:txn_id",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The room to send the event to.
            #[ruma_api(path)]
            pub room_id: &'a RoomId,

            /// The type of event to send.
            #[ruma_api(path)]
            pub event_type: MessageLikeEventType,

            /// The transaction ID for this event.
            ///
            /// Clients should generate an ID unique across requests with the
            /// same access token; it will be used by the server to ensure
            /// idempotency of requests.
            #[ruma_api(path)]
            pub txn_id: &'a TransactionId,

            /// The event content to send.
            #[ruma_api(body)]
            pub body: Raw<AnyMessageLikeEventContent>,
        }

        response: {
            /// A unique identifier for the event.
            pub event_id: OwnedEventId,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room id, transaction id and event content.
        ///
        /// # Errors
        ///
        /// Since `Request` stores the request body in serialized form, this function can fail if
        /// `T`s [`Serialize`][serde::Serialize] implementation can fail.
        pub fn new<T>(
            room_id: &'a RoomId,
            txn_id: &'a TransactionId,
            content: &'a T,
        ) -> serde_json::Result<Self>
        where
            T: MessageLikeEventContent,
        {
            Ok(Self {
                room_id,
                txn_id,
                event_type: content.event_type(),
                body: Raw::from_json(to_raw_json_value(content)?),
            })
        }

        /// Creates a new `Request` with the given room id, transaction id, event type and raw event
        /// content.
        pub fn new_raw(
            room_id: &'a RoomId,
            txn_id: &'a TransactionId,
            event_type: MessageLikeEventType,
            body: Raw<AnyMessageLikeEventContent>,
        ) -> Self {
            Self { room_id, event_type, txn_id, body }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given event id.
        pub fn new(event_id: OwnedEventId) -> Self {
            Self { event_id }
        }
    }
}
