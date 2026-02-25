//! `GET /_matrix/client/*/directory/list/room/{roomId}`
//!
//! Get the visibility of a public room on a directory.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3directorylistroomroomid

    use ruma_common::{
        RoomId,
        api::{auth_scheme::NoAuthentication, request, response},
        metadata,
    };

    use crate::room::Visibility;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        history: {
            1.0 => "/_matrix/client/r0/directory/list/room/{room_id}",
            1.1 => "/_matrix/client/v3/directory/list/room/{room_id}",
        }
    }

    /// Request type for the `get_room_visibility` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The ID of the room of which to request the visibility.
        #[ruma_api(path)]
        pub room_id: RoomId,
    }

    /// Response type for the `get_room_visibility` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// Visibility of the room.
        pub visibility: Visibility,
    }

    impl Request {
        /// Creates a new `Request` with the given room ID.
        pub fn new(room_id: RoomId) -> Self {
            Self { room_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given visibility.
        pub fn new(visibility: Visibility) -> Self {
            Self { visibility }
        }
    }
}
