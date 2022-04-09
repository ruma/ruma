//! `GET /_matrix/federation/*/state_ids/{roomId}`
//!
//! Retrieves a snapshot of a room's state at a given event, in the form of event IDs.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/server-server-api/#get_matrixfederationv1state_idsroomid

    use ruma_common::{api::ruma_api, EventId, OwnedEventId, RoomId};

    ruma_api! {
        metadata: {
            description: "Retrieves a snapshot of a room's state at a given event, in the form of event IDs",
            method: GET,
            name: "get_room_state_ids",
            stable_path: "/_matrix/federation/v1/state_ids/:room_id",
            rate_limited: false,
            authentication: ServerSignatures,
            added: 1.0,
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
            pub auth_chain_ids: Vec<OwnedEventId>,

            /// The fully resolved state of the room at the given event.
            pub pdu_ids: Vec<OwnedEventId>,
        }
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given event id and room id.
        pub fn new(event_id: &'a EventId, room_id: &'a RoomId) -> Self {
            Self { room_id, event_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given auth chain IDs and room state IDs.
        pub fn new(auth_chain_ids: Vec<OwnedEventId>, pdu_ids: Vec<OwnedEventId>) -> Self {
            Self { auth_chain_ids, pdu_ids }
        }
    }
}
