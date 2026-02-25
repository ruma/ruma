//! `PUT /_matrix/client/*/rooms/{roomId}/redact/{eventId}/{txnId}`
//!
//! Redact an event, stripping all information not critical to the event graph integrity.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#put_matrixclientv3roomsroomidredacteventidtxnid

    use ruma_common::{
        EventId, RoomId, TransactionId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    metadata! {
        method: PUT,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/rooms/{room_id}/redact/{event_id}/{txn_id}",
            1.1 => "/_matrix/client/v3/rooms/{room_id}/redact/{event_id}/{txn_id}",
        }
    }

    /// Request type for the `redact_event` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The ID of the room of the event to redact.
        #[ruma_api(path)]
        pub room_id: RoomId,

        /// The ID of the event to redact.
        #[ruma_api(path)]
        pub event_id: EventId,

        /// The transaction ID for this event.
        ///
        /// Clients should generate a unique ID across requests within the
        /// same session. A session is identified by an access token, and
        /// persists when the [access token is refreshed].
        ///
        /// It will be used by the server to ensure idempotency of requests.
        ///
        /// [access token is refreshed]: https://spec.matrix.org/latest/client-server-api/#refreshing-access-tokens
        #[ruma_api(path)]
        pub txn_id: TransactionId,

        /// The reason for the redaction.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub reason: Option<String>,
    }

    /// Response type for the `redact_event` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The ID of the redacted event.
        pub event_id: EventId,
    }

    impl Request {
        /// Creates a new `Request` with the given room ID, event ID and transaction ID.
        pub fn new(room_id: RoomId, event_id: EventId, txn_id: TransactionId) -> Self {
            Self { room_id, event_id, txn_id, reason: None }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given event ID.
        pub fn new(event_id: EventId) -> Self {
            Self { event_id }
        }
    }
}
