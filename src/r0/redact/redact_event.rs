//! [PUT /_matrix/client/r0/rooms/{roomId}/redact/{eventId}/{txnId}](https://matrix.org/docs/spec/client_server/r0.4.0.html#put-matrix-client-r0-rooms-roomid-redact-eventid-txnid)

use ruma_api::ruma_api;
use ruma_identifiers::{EventId, RoomId};

ruma_api! {
    metadata {
        description: "Redact an event, stripping all information not critical to the event graph integrity.",
        method: PUT,
        name: "redact_event",
        path: "/_matrix/client/r0/rooms/:room_id/redact/:event_id/:txn_id",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The ID of the event to redact.
        #[ruma_api(path)]
        pub event_id: EventId,
        /// The reason for the redaction.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub reason: Option<String>,
        /// The ID of the room of the event to redact.
        #[ruma_api(path)]
        pub room_id: RoomId,
        /// The transaction ID for this event.
        ///
        /// Clients should generate a unique ID; it will be used by the server to ensure idempotency of requests.
        #[ruma_api(path)]
        pub txn_id: String,
    }

    response {
        /// The ID of the redacted event.
        pub event_id: EventId,
    }

    error: crate::Error
}
