//! `GET /_matrix/client/*/directory/list/room/{roomId}`
//!
//! Get the visibility of a public room on a directory.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#get_matrixclientv3directorylistroomroomid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, RoomId,
    };

    use crate::room::Visibility;

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: None,
        history: {
            1.0 => "/_matrix/client/r0/directory/list/room/:room_id",
            1.1 => "/_matrix/client/v3/directory/list/room/:room_id",
        }
    };

    /// Request type for the `get_room_visibility` endpoint.
    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// The ID of the room of which to request the visibility.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,
    }

    /// Response type for the `get_room_visibility` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// Visibility of the room.
        pub visibility: Visibility,
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room ID.
        pub fn new(room_id: &'a RoomId) -> Self {
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
