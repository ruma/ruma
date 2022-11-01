//! `GET /_matrix/client/*/rooms/{roomId}/state`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#get_matrixclientv3roomsroomidstate

    use ruma_common::{
        api::{request, response, Metadata},
        events::AnyStateEvent,
        metadata,
        serde::Raw,
        RoomId,
    };

    const METADATA: Metadata = metadata! {
        description: "Get state events for a room.",
        method: GET,
        name: "get_state_events",
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/rooms/:room_id/state",
            1.1 => "/_matrix/client/v3/rooms/:room_id/state",
        }
    };

    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// The room to look up the state for.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,
    }

    #[response(error = crate::Error)]
    pub struct Response {
        /// If the user is a member of the room this will be the current state of the room as a
        /// list of events.
        ///
        /// If the user has left the room then this will be the state of the room when they left as
        /// a list of events.
        #[ruma_api(body)]
        pub room_state: Vec<Raw<AnyStateEvent>>,
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room ID.
        pub fn new(room_id: &'a RoomId) -> Self {
            Self { room_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given room state.
        pub fn new(room_state: Vec<Raw<AnyStateEvent>>) -> Self {
            Self { room_state }
        }
    }
}
