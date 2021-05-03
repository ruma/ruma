//! [PUT /_matrix/client/r0/rooms/{roomId}/redact/{eventId}/{txnId}](https://matrix.org/docs/spec/client_server/r0.6.0#put-matrix-client-r0-rooms-roomid-redact-eventid-txnid)

use ruma_api::ruma_api;
use ruma_identifiers::{EventId, RoomId};

ruma_api! {
    metadata: {
        description: "Redact an event, stripping all information not critical to the event graph integrity.",
        method: PUT,
        name: "redact_event",
        path: "/_matrix/client/r0/rooms/:room_id/redact/:event_id/:txn_id",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The ID of the room of the event to redact.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The ID of the event to redact.
        #[ruma_api(path)]
        pub event_id: &'a EventId,

        /// The transaction ID for this event.
        ///
        /// Clients should generate a unique ID; it will be used by the server to ensure idempotency
        /// of requests.
        #[ruma_api(path)]
        pub txn_id: &'a str,

        /// The reason for the redaction.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub reason: Option<&'a str>,
    }

    response: {
        /// The ID of the redacted event.
        pub event_id: EventId,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given room ID, event ID and transation ID.
    pub fn new(room_id: &'a RoomId, event_id: &'a EventId, txn_id: &'a str) -> Self {
        Self { room_id, event_id, txn_id, reason: None }
    }
}

impl Response {
    /// Creates a new `Response` with the given event ID.
    pub fn new(event_id: EventId) -> Self {
        Self { event_id }
    }
}
