//! [GET /_matrix/federation/v1/event_auth/{roomId}/{eventId}](https://matrix.org/docs/spec/server_server/r0.1.4#get-matrix-federation-v1-event-auth-roomid-eventid)

use ruma_api::ruma_api;
use ruma_events::pdu::Pdu;
use ruma_identifiers::{EventId, RoomId};
use ruma_serde::Raw;

ruma_api! {
    metadata: {
        description: "Retrieves the complete auth chain for a given event.",
        name: "get_event_authorization",
        method: GET,
        path: "/_matrix/federation/v1/event_auth/:room_id/:event_id",
        rate_limited: false,
        authentication: ServerSignatures,
    }

    request: {
        /// The room ID to get the auth chain for.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The event ID to get the auth chain for.
        #[ruma_api(path)]
        pub event_id: &'a EventId,
    }

    response: {
        /// The full set of authorization events that make up the state of the room,
        /// and their authorization events, recursively.
        pub auth_chain: Vec<Raw<Pdu>>,
    }
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given room id and event id.
    pub fn new(room_id: &'a RoomId, event_id: &'a EventId) -> Self {
        Self { room_id, event_id }
    }
}

impl Response {
    /// Creates a new `Response` with the given auth chain.
    pub fn new(auth_chain: Vec<Raw<Pdu>>) -> Self {
        Self { auth_chain }
    }
}
