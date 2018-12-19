//! [GET /_matrix/client/r0/rooms/{roomId}/state/{eventType}/{stateKey}](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-rooms-roomid-state-eventtype-state-key)

use ruma_api_macros::ruma_api;
use ruma_events::EventType;
use ruma_identifiers::RoomId;
use serde_derive::{Deserialize, Serialize};

ruma_api! {
    metadata {
        description: "Get state events associated with a given key.",
        method: GET,
        name: "get_state_events_for_key",
        path: "/_matrix/client/r0/rooms/:room_id/state/:event_type/:state_key",
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
        /// The key of the state to look up.
        #[ruma_api(path)]
        pub state_key: String,
    }

    response {
        /// The content of the state event.
        #[ruma_api(body)]
        pub content: ::serde_json::Value,
    }
}
