//! `GET /_matrix/federation/*/state_ids/{roomId}`
//!
//! Retrieves a snapshot of a room's state at a given event, in the form of event IDs.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/server-server-api/#get_matrixfederationv1state_idsroomid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedEventId, OwnedRoomId,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: ServerSignatures,
        history: {
            1.0 => "/_matrix/federation/v1/state_ids/{room_id}",
        }
    };

    /// Request type for the `get_room_state_ids` endpoint.
    #[request]
    pub struct Request {
        /// The room ID to get state for.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// An event ID in the room to retrieve the state at.
        #[ruma_api(query)]
        pub event_id: OwnedEventId,
    }

    /// Response type for the `get_room_state_ids` endpoint.
    #[response]
    pub struct Response {
        /// The full set of authorization events that make up the state of the
        /// room, and their authorization events, recursively.
        pub auth_chain_ids: Vec<OwnedEventId>,

        /// The fully resolved state of the room at the given event.
        pub pdu_ids: Vec<OwnedEventId>,
    }

    impl Request {
        /// Creates a new `Request` with the given event id and room id.
        pub fn new(event_id: OwnedEventId, room_id: OwnedRoomId) -> Self {
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
