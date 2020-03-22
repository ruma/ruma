//! [GET /_matrix/client/r0/rooms/{roomId}/state/{eventType}](https://matrix.org/docs/spec/client_server/r0.4.0.html#get-matrix-client-r0-rooms-roomid-state-eventtype)

use ruma_api::ruma_api;
use ruma_events::EventType;
use ruma_identifiers::RoomId;
use serde_json::Value;

ruma_api! {
    metadata {
        description: "Get state events of a given type associated with the empty key.",
        method: GET,
        name: "get_state_events_for_empty_key",
        path: "/_matrix/client/r0/rooms/:room_id/state/:event_type",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The room to look up the state for.
        #[ruma_api(path)]
        pub room_id: RoomId,
        /// The type of state to look up.
        #[ruma_api(path)]
        pub event_type: EventType,
    }

    response {
        /// The content of the state event.
        #[ruma_api(body)]
        pub content: Value,
    }

    error: crate::Error
}
