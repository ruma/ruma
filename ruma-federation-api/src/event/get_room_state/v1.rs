//! [GET /_matrix/federation/v1/state/{roomId}](https://matrix.org/docs/spec/server_server/r0.1.4#get-matrix-federation-v1-state-roomid)

use ruma_api::ruma_api;
use ruma_events::pdu::Pdu;
use ruma_identifiers::{EventId, RoomId};
use ruma_serde::Raw;

ruma_api! {
    metadata: {
        description: "Retrieves a snapshot of a room's state at a given event.",
        method: GET,
        name: "get_room_state",
        path: "/_matrix/federation/v1/state/:room_id",
        rate_limited: false,
        authentication: ServerSignatures,
    }

    request: {
        /// The room ID to get state for.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// An event ID in the room to retrieve the state at.
        #[ruma_api(query)]
        pub event_id: &'a EventId,
    }

    response: {
        /// The full set of authorization events that make up the state of the
        /// room, and their authorization events, recursively.
        pub auth_chain: Vec<Raw<Pdu>>,

        /// The fully resolved state of the room at the given event.
        pub pdus: Vec<Raw<Pdu>>,
    }
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given event ID and room ID.
    pub fn new(event_id: &'a EventId, room_id: &'a RoomId) -> Self {
        Self { event_id, room_id }
    }
}

impl Response {
    /// Creates a new `Response` with the given auth chain and room state.
    pub fn new(auth_chain: Vec<Raw<Pdu>>, pdus: Vec<Raw<Pdu>>) -> Self {
        Self { auth_chain, pdus }
    }
}
