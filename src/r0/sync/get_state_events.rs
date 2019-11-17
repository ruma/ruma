//! [GET /_matrix/client/r0/rooms/{roomId}/state](https://matrix.org/docs/spec/client_server/r0.4.0.html#get-matrix-client-r0-rooms-roomid-state)

use ruma_api::ruma_api;
use ruma_events::{collections::only, EventResult};
use ruma_identifiers::RoomId;

ruma_api! {
    metadata {
        description: "Get state events for a room.",
        method: GET,
        name: "get_state_events",
        path: "/_matrix/client/r0/rooms/:room_id/state",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The room to look up the state for.
        #[ruma_api(path)]
        pub room_id: RoomId,
    }

    response {
        /// If the user is a member of the room this will be the current state of the room as a
        /// list of events. If the user has left the room then this will be the state of the
        /// room when they left as a list of events.
        #[ruma_api(body)]
        #[wrap_incoming(only::StateEvent with EventResult)]
        pub room_state: Vec<only::StateEvent>,
    }
}
