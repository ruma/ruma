//! [GET /_matrix/federation/v1/event_auth/{roomId}/{eventId}](https://matrix.org/docs/spec/server_server/r0.1.4#get-matrix-federation-v1-event-auth-roomid-eventid)

use ruma_api::ruma_api;
use ruma_events::pdu::Pdu;
use ruma_identifiers::{EventId, RoomId};

ruma_api! {
    metadata: {
        description: "Retrieves the complete auth chain for a given event.",
        name: "get_event_authorization",
        method: GET,
        path: "/_matrix/federation/v1/event_auth/:room_id/:event_id",
        rate_limited: false,
        requires_authentication: true,
    }

    request: {
        /// The room ID to get the auth chain for.
        #[ruma_api(path)]
        pub room_id: RoomId,

        /// The event ID to get the auth chain for.
        #[ruma_api(path)]
        pub event_id: EventId,
    }

    response: {
        /// The full set of authorization events that make up the state of the room,
        /// and their authorization events, recursively.
        pub auth_chain: Vec<Pdu>,
    }
}
