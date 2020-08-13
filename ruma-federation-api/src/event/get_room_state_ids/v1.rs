//! [GET /_matrix/federation/v1/state_ids/{roomId}](https://matrix.org/docs/spec/server_server/r0.1.4#get-matrix-federation-v1-state-ids-roomid)

use ruma_api::ruma_api;
use ruma_identifiers::{EventId, RoomId};

ruma_api! {
    metadata: {
        description: "Retrieves a snapshot of a room's state at a given event, in the form of event IDs",
        method: GET,
        name: "get_room_state_ids",
        path: "/_matrix/federation/v1/state_ids/:room_id",
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
        pub auth_chain_ids: Vec<EventId>,

        /// The fully resolved state of the room at the given event.
        pub pdu_ids: Vec<EventId>,
    }
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given event id and room id.
    pub fn new(event_id: &'a EventId, room_id: &'a RoomId) -> Self {
        Self { event_id, room_id }
    }
}

impl Response {
    /// Creates a new `Response` with the given auth chain IDs and room state IDs.
    pub fn new(auth_chain_ids: Vec<EventId>, pdu_ids: Vec<EventId>) -> Self {
        Self { auth_chain_ids, pdu_ids }
    }
}
