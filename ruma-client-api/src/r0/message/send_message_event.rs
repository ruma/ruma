//! [PUT /_matrix/client/r0/rooms/{roomId}/send/{eventType}/{txnId}](https://matrix.org/docs/spec/client_server/r0.6.1#put-matrix-client-r0-rooms-roomid-send-eventtype-txnid)

use ruma_api::ruma_api;
use ruma_events::EventType;
use ruma_identifiers::{EventId, RoomId};
use serde_json::value::RawValue as RawJsonValue;

ruma_api! {
    metadata: {
        description: "Send a message event to a room.",
        method: PUT,
        name: "create_message_event",
        path: "/_matrix/client/r0/rooms/:room_id/send/:event_type/:txn_id",
        rate_limited: false,
        requires_authentication: true,
    }

    request: {
        /// The room to send the event to.
        #[ruma_api(path)]
        pub room_id: RoomId,

        /// The type of event to send.
        #[ruma_api(path)]
        pub event_type: EventType,

        /// The transaction ID for this event.
        ///
        /// Clients should generate an ID unique across requests with the
        /// same access token; it will be used by the server to ensure
        /// idempotency of requests.
        #[ruma_api(path)]
        pub txn_id: String,

        /// The event's content. The type for this field will be updated in a
        /// future release, until then you can create a value using
        /// `serde_json::value::to_raw_value`.
        #[ruma_api(body)]
        pub data: Box<RawJsonValue>,
    }

    response: {
        /// A unique identifier for the event.
        pub event_id: EventId,
    }

    error: crate::Error
}
