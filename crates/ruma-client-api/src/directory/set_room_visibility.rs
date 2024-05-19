//! `PUT /_matrix/client/*/directory/list/room/{roomId}`
//!
//! Set the visibility of a public room on a directory.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#put_matrixclientv3directorylistroomroomid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedRoomId,
    };

    use crate::room::Visibility;

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/directory/list/room/{room_id}",
            1.1 => "/_matrix/client/v3/directory/list/room/{room_id}",
        }
    };

    /// Request type for the `set_room_visibility` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The ID of the room of which to set the visibility.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// New visibility setting for the room.
        pub visibility: Visibility,
    }

    /// Response type for the `set_room_visibility` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given room ID and visibility.
        pub fn new(room_id: OwnedRoomId, visibility: Visibility) -> Self {
            Self { room_id, visibility }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
