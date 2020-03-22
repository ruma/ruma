//! [PUT /_matrix/client/r0/rooms/{roomId}/state/{eventType}](https://matrix.org/docs/spec/client_server/r0.4.0.html#put-matrix-client-r0-rooms-roomid-state-eventtype)

use ruma_api::ruma_api;
use ruma_events::EventType;
use ruma_identifiers::{EventId, RoomId};
use serde_json::Value;

ruma_api! {
    metadata {
        description: "Send a state event to a room associated with the empty state key.",
        method: PUT,
        name: "create_state_event_for_empty_key",
        path: "/_matrix/client/r0/rooms/:room_id/state/:event_type",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The room to set the state in.
        #[ruma_api(path)]
        pub room_id: RoomId,
        /// The type of event to send.
        #[ruma_api(path)]
        pub event_type: EventType,
        /// The event's content.
        #[ruma_api(body)]
        pub data: Value,
    }

    response {
        /// A unique identifier for the event.
        pub event_id: EventId,
    }

    error: crate::Error
}
