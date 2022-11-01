//! `GET /_matrix/federation/*/state/{roomId}`
//!
//! Retrieves a snapshot of a room's state at a given event.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/server-server-api/#get_matrixfederationv1stateroomid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, EventId, RoomId,
    };
    use serde_json::value::RawValue as RawJsonValue;

    const METADATA: Metadata = metadata! {
        description: "Retrieves a snapshot of a room's state at a given event.",
        method: GET,
        name: "get_room_state",
        rate_limited: false,
        authentication: ServerSignatures,
        history: {
            1.0 => "/_matrix/federation/v1/state/:room_id",
        }
    };

    #[request]
    pub struct Request<'a> {
        /// The room ID to get state for.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// An event ID in the room to retrieve the state at.
        #[ruma_api(query)]
        pub event_id: &'a EventId,
    }

    #[response]
    pub struct Response {
        /// The full set of authorization events that make up the state of the
        /// room, and their authorization events, recursively.
        pub auth_chain: Vec<Box<RawJsonValue>>,

        /// The fully resolved state of the room at the given event.
        pub pdus: Vec<Box<RawJsonValue>>,
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given event ID and room ID.
        pub fn new(event_id: &'a EventId, room_id: &'a RoomId) -> Self {
            Self { room_id, event_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given auth chain and room state.
        pub fn new(auth_chain: Vec<Box<RawJsonValue>>, pdus: Vec<Box<RawJsonValue>>) -> Self {
            Self { auth_chain, pdus }
        }
    }
}
