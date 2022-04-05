//! `GET /_matrix/client/*/rooms/{roomId}/state`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3roomsroomidstate

    use ruma_common::{api::ruma_api, events::AnyStateEvent, serde::Raw, RoomId};

    ruma_api! {
        metadata: {
            description: "Get state events for a room.",
            method: GET,
            name: "get_state_events",
            r0_path: "/_matrix/client/r0/rooms/:room_id/state",
            stable_path: "/_matrix/client/v3/rooms/:room_id/state",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The room to look up the state for.
            #[ruma_api(path)]
            pub room_id: &'a RoomId,
        }

        response: {
            /// If the user is a member of the room this will be the current state of the room as a
            /// list of events.
            ///
            /// If the user has left the room then this will be the state of the room when they left as
            /// a list of events.
            #[ruma_api(body)]
            pub room_state: Vec<Raw<AnyStateEvent>>,
        }

        error: crate::Error
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
